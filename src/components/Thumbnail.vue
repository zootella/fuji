<script setup>//./components/Thumbnail.vue - given a data url and height, render an image into a canvas to save memory

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {
xy, sizeThumbnail,
} from './library.js'//our javascript library

/*
for thumbnails, we have to use canvas to be efficient with memory
here's the memory impact as an image moves from disk to screen:
1. Disk → Data URL: The JPEG bytes become ~33% larger as base64
2. Data URL → Bitmap: The renderer decodes the image into a full size, uncompressed bitmap at the image's full natural resolution (width × height × 4 bytes for RGBA) The renderer keeps the full bitmap in memory even if CSS sizes the <img> small
3. Canvas: So, we paint the image onto a small canvas and tell the renderer to release the decoded bitmap
*/

const props = defineProps({
	data:   {type: String, required: true},//data url of the bytes of the image parent got from the disk already
	height: {type: Number, required: true},//thumbnail height in CSS pixels; we'll render up to 2x wide in this height
})//we're getting these from the parent as props, but don't have a watcher or computed property, so we'll only render once at the start

onMounted(async () => {
	await renderThumbnail()
})

async function renderThumbnail() {
	let canvas = canvasRef.value

	let img = new Image()//make a new <img> tag not in the DOM
	await new Promise((resolve, reject) => {//wait here for it to render the given data URL
		img.onload = resolve
		img.onerror = reject//hookup img event handlers to this alternative promise pattern
		img.src = props.data//here's where we give the tag the data, causing it to start rendering
	})//this rendering step can be slow for big JPEGs with advanced compression; drawImage below is quick

	let natural = xy(img.naturalWidth, img.naturalHeight)//now we know the raster dimensions of the image
	let cssSize = sizeThumbnail('Flickr.', natural, props.height)//and can calculate how big this thumbnail should be
	let canvasSize = xy(cssSize, '*', window.devicePixelRatio)//old monitor 1, macOS Retina always 2 and gets to the os backing bitmap (not physical) resolution; Windows like 1.5 gets to physical pixels

	canvas.width = canvasSize.x
	canvas.height = canvasSize.y//doing this resets the canvas, blanking it, and sets its internal resolution
	canvas.style.width = cssSize.x + 'px'
	canvas.style.height = cssSize.y + 'px'

	let context = canvas.getContext('2d', {alpha: true})//alpha on is the default; false paints onto black background
	context.imageSmoothingQuality = 'high'//low, medium, high are the options, default low looks fine, but for Fuji, only the best
	context.drawImage(img, 0, 0, canvasSize.x, canvasSize.y)//draw the image scaled down into our canvas
	img.src = ''//free the full-size bitmap so the renderer can garbage collect it; we've got it painted into our canvas and don't need it anymore!
}

const canvasRef = ref(null)

</script>
<template>

<canvas ref="canvasRef"></canvas>

</template>
