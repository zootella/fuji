<script setup>//paint each image down into a canvas, to own the memory rather than lend it to the engine

import {onMounted, onBeforeUnmount} from 'vue'
import {cacheNeed, cacheRelease} from '../cache.js'
import {settingsThumbnailBox} from '../settings.js'
import {logTrouble} from '../log.js'
import {xy} from './library.js'

/*
The other half of the question TagFlow asks. That one hands the engine full-size originals and lets it decide what to keep; this one reads each image, paints it small, and lets the original go — so what a card holds is a number fuji chose rather than one the engine is managing out of sight.

One image at a time, in a plain loop. Need it, paint it, release it, take the next: the store is never holding more than the one being worked on — one per card, since every card fills at once — which is the strongest form of the promise this flow is making and also the least code that can make it. TagFlow races every load at once and lets the operating system schedule the disk, which is right for a flow that only hands work to the engine; here the point is the ceiling, and a loop is the whole mechanism. If filling a card turns out too slow to live with, the next thing to try is a few in flight at once — a small fixed count, not the race TagFlow runs. Every load here is a forced full-size decode, so racing them all would hand the peak straight back to the engine, and any raster it dropped before its turn would be decoded again, synchronously, inside drawImage. A bounded count keeps the ceiling and is a few lines, which is still not a scheduler of fuji's own.

createImageBitmap was here and is deliberately gone. Asking it for a scaled bitmap with resizeWidth and resizeQuality looks like the purpose-built tool, and on Chromium it is. WebKit has the code but does not ship the options, so on macOS it returned a full-size bitmap — a whole copy of a large photograph, allocated for nothing, then scaled by the same drawImage that could have done the job alone.

One draw from the store's element came out rough. The roof tiles of a 26-megapixel photograph aliased into jaggies at 240 pixels, where Safari showed the same file smooth as an img, and canvas.md has the two reasons. WebKit decodes a large picture small for a small paint, for an img and for drawImage alike — but a frame already decoded at full size counts as good enough for any smaller request, and the store's decode() makes exactly that frame, so drawImage was handed all 26 megapixels. And CoreGraphics' high interpolation reads a fixed footprint of source pixels around each output pixel, which at 26 to 1 leaves most of the picture unread, and unread pixels alias. flowShrink below halves through scratch canvases until the last draw is within 2 to 1, the ratio at which a fixed footprint is an honest average. The cheaper cure — asking the store for the bytes only and letting drawImage decode at its own scale, which is what Safari's img does — is written up in canvas.md as the next thing to measure, because on WebKit it would move the decode onto the main thread.

The pixel math is ours to get wrong, where an img cannot be talked into a wrong answer. devicePixelRatio is the right ratio because it reaches the backing store the renderer composites into, which is as far as anything drawn on this page can reach; panel.rs answers the different question of what is truly on the glass, which is what "100%" must mean on a table and not what a thumbnail needs. card.md carries the rest of what this flow costs against TagFlow — that the ratio and the gamut go stale when the window changes monitors, and that a size change here is a repaint where TagFlow restyles.
*/

const flowHolder = 'CanvasFlow'//on every reference this flow takes, so a leak has a name and not just a size
const flowBox = settingsThumbnailBox()//read once, because every canvas below is painted to it and a change means painting them all again
const flowGamut = matchMedia('(color-gamut: p3)').matches ? 'display-p3' : 'srgb'//the color space every canvas is made in, read once like the box. A canvas is sRGB unless asked, and drawing a Display P3 photograph — any recent iPhone's — into an sRGB canvas clamps its most saturated colors away for good, so the thumbnail comes out duller than the table shows the same file. Asking for the screen's own gamut keeps those colors on a wide-gamut panel and changes nothing on an ordinary one: an sRGB screen gets the sRGB canvas it always got. This asks about the screen rather than the engine, which is what keeps it from being a feature check, and it goes stale on a change of monitor exactly as devicePixelRatio does; canvas.md has what each platform answers, and why a Linux build can never be asked for a value it lacks

const props = defineProps({
	paths: {type: Array, required: true},//already in the model's order
})

const flowCanvases = new Map()//path to its canvas element, keyed by path rather than position so nothing depends on the order the template renders in
let flowClosed = false//the card is going away, so the loop should stop rather than paint into elements nobody will see

onMounted(flowPaintAll)//not at setup: the canvases below do not exist until this view has been rendered, and every paint needs one
onBeforeUnmount(() => { flowClosed = true })//nothing to release here: each image is given back the moment its canvas has the pixels

async function flowPaintAll() {//down the card, one image at a time
	for (let path of props.paths) {
		if (flowClosed) return
		await flowPaint(path)
	}
}

async function flowPaint(path) {//one image, from the store to a canvas and back out of the store again
	let promise = cacheNeed(path, flowHolder)//the reference is taken right here, before any await, so the release below is owed from this line on and the try has to start after it
	try {
		let entry = await promise
		let canvas = flowCanvases.get(path)
		if (flowClosed || entry.error || !canvas) return//a file that would not read or decode leaves a canvas of nothing, so a broken picture never appears in the flow

		let natural = xy(entry.img.naturalWidth, entry.img.naturalHeight)
		if (!(natural.x > 0 && natural.y > 0)) return//an svg with no width and height of its own measures zero, and there is nothing to scale a drawing to; TagFlow shows these and this flow cannot

		let {scale, css} = flowFit(natural, flowBox)
		let detail = Math.min(window.devicePixelRatio, 1 / scale)//never ask for more backing pixels than the file has detail to fill; read from the scale rather than from the sizes, because a sliver rounds up to one css pixel on its short side and would report no detail there at all
		let backing = xy(Math.max(1, Math.round(css.x * detail)), Math.max(1, Math.round(css.y * detail)))

		canvas.width  = backing.x//the backing size, in the pixels the renderer composites; assigning either of these also clears the canvas and its context settings, so they come first
		canvas.height = backing.y
		canvas.style.width  = css.x+'px'//and the css size, which is what the flow lays out
		canvas.style.height = css.y+'px'

		let context = canvas.getContext('2d', {colorSpace: flowGamut})//in the screen's own gamut, explained above; everything else about the context is left at its default, alpha included, so a png's transparency shows the sheet through it
		flowShrink(context, entry.img, natural, backing)//by halving, not in one draw: the essay above says why one draw came out rough
	} catch (error) {
		logTrouble('CanvasFlow: painting a thumbnail', error)//one image that will not paint must not break the loop and quietly stop the rest of the card
	} finally {
		cacheRelease(path, flowHolder)//the canvas holds these pixels now; the store may drop the bytes, the url, and the full-size decode
	}
}

function flowShrink(context, source, size, target) {//draw source, which is size pixels, into context at target pixels, halving through scratch canvases until the last draw is within two to one; a fixed-footprint filter is only an honest average at that ratio
	let scratch = [document.createElement('canvas'), document.createElement('canvas')]//two, taken in turn, because a canvas cannot be shrunk into itself
	let step = 0
	while (size.x > target.x * 2) {//one axis is enough, because the fit kept the aspect; strictly more than double, so the final draw below is between one to one and two to one
		let half = xy(Math.ceil(size.x / 2), Math.ceil(size.y / 2))
		let canvas = scratch[step % 2]//never the one that is the current source
		canvas.width = half.x; canvas.height = half.y//sizing also clears it and resets its context, so the quality is set again below
		let c = canvas.getContext('2d', {colorSpace: flowGamut})//the same gamut all the way down, so nothing is converted twice
		c.imageSmoothingQuality = 'high'
		c.drawImage(source, 0, 0, half.x, half.y)//the first step reads the store's element, so the engine applies the file's orientation there; every later step reads a canvas
		source = canvas; size = half; step++
	}
	context.imageSmoothingQuality = 'high'//the last draw, into the real canvas: high rather than the default low, which reads only the four nearest source pixels per output pixel
	context.drawImage(source, 0, 0, target.x, target.y)
	for (let canvas of scratch) { canvas.width = 0; canvas.height = 0 }//give the backing stores back now rather than whenever collection gets to them; the first is a quarter of the picture's own size
}

function flowFit(natural, box) {//the size this image shows at, and the scale that got it there
	let scale = Math.min(box / natural.x, box / natural.y, 1)//the 1 keeps a 32 by 32 icon at its own size rather than blowing it up
	return {scale, css: xy(Math.max(1, Math.round(natural.x * scale)), Math.max(1, Math.round(natural.y * scale)))}
}

</script>
<template>

<!-- items-start so a short image keeps its own height instead of stretching to the tallest in its row; each canvas starts at nothing and takes its size when it is painted -->
<div class="flex flex-wrap items-start">
	<canvas
		v-for="path in props.paths" :key="path"
		:ref="el => el ? flowCanvases.set(path, el) : flowCanvases.delete(path)"
		class="myThumbnail" width="0" height="0"
	></canvas>
</div>

</template>
<style scoped>

.myThumbnail {
	display: block; /* a canvas is inline by default, and its baseline would show as a stripe under every row */
}

</style>
