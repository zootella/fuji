<script setup>//./components/Viewer4.vue - rendering in canvas

//delete because we abandoned canvas

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {ioRead} from '../io.js'

const expectResize = true//ttd july, this will be set by the parent component as a property, for instance
/*
this one image viewer component is designed to be used both for full screen lightbox, and to show hundreds of small thumbnails
true: lightbox use: we'll keep the full rendered bitmap in memory so you can resize without us hitting the disk again
false: thumbnail usage: we'll discard the full rendered bitmap to save memory; resize works but hits the disk again
*/

const pathRef = ref('')
const canvasRef = ref(null)

let unlistenFileDrop = null//will hold the unsubscribe function set above and called below
let unlistenResize = null
let cachedBigBitmap = null//our cache in memory of the bitmap rendered full size, uncompressed

onMounted(async () => {
	try {
		const w = getCurrentWindow()

		unlistenFileDrop = await w.onDragDropEvent(async event => {
			try {
				if (event.payload.type === 'drop' && event.payload.paths.length) {
					let p = event.payload.paths[0]
					await loadImage(p)
				}
			} catch (e) { console.error(e) }
		})

		unlistenResize = await w.onResized(async ({width, height}) => {
			try {
				if (expectResize && cachedBigBitmap) {
					drawBitmap(cachedBigBitmap)// fast path: redraw cached bitmap
				} else if (pathRef.value) {
					await loadImage(pathRef.value)// slow path: re-read + decode + draw
				}
			} catch (e) { console.error(e) }
		})
	} catch (e) { console.error(e) }
})
onBeforeUnmount(() => {
	try {

		unlistenFileDrop?.()
		unlistenResize?.()

	  cachedBigBitmap?.close?.()//let the browser free GPU/textures immediately, rather than waiting for GC heuristics
	  cachedBigBitmap = null

	} catch (e) { console.error(e) }
})

async function loadImage(p) {
	pathRef.value = p
	if (!canvasRef.value) { console.log('canvas not ready yet'); return }

	// 1. Decode bytes → ImageBitmap
	let bigBytes = await ioRead(pathRef.value)
	let bigArray = new Uint8Array(bigBytes)
	let bigBlob  = new Blob([bigArray.buffer], {type: pathToMimeType(pathRef.value)})
	let bigBitmap = await createImageBitmap(bigBlob)
	drawBitmap(bigBitmap)

	if (expectResize) {//we've been told to care about redraw speed, not memory footprint (lightbox use)
		cachedBigBitmap?.close?.()//free a previous one explicitly
		cachedBigBitmap = bigBitmap//cache the bitmap
	} else {//we've been told to care about memory footprint, not redraw speed (thumbnail use)
		bigBitmap.close?.()//free the bitmap immediately
		cachedBigBitmap = null
	}
}

function pathToMimeType(path) {
	let extension = path.split('.').pop()//ttd july, use path-browserify here instead
	switch (extension.toLowerCase()) {
		case 'jpg':
		case 'jpeg': return 'image/jpeg'
		case 'png':  return 'image/png'
		case 'webp': return 'image/webp'
		case 'gif':  return 'image/gif'
		default:     return 'application/octet-stream'
	}
}

/** 
 * Given an ImageBitmap and the available viewport, 
 * returns { w, h } so it “contains” inside the viewport 
 * without changing the bitmap’s aspect ratio.
 */
function fitToViewport(imgW, imgH, maxW, maxH) {
	let imgRatio = imgW / imgH
	// start by fitting to width
	let w = maxW
	let h = w / imgRatio
	// if that overflows height, fit to height instead
	if (h > maxH) {
		h = maxH
		w = h * imgRatio
	}
	return { w, h }
}

/**
 * Prepare a canvas for Hi-DPI rendering.
 * - `size` is the CSS size (e.g. match your container or props.size).
 * - Returns the 2D rendering context, already scaled & smoothed.
 */
function setupCanvas(canvasEl, cssW, cssH) {
	let dpr = window.devicePixelRatio || 1
	canvasEl.width  = cssW * dpr
	canvasEl.height = cssH * dpr
	canvasEl.style.width  = `${cssW}px`
	canvasEl.style.height = `${cssH}px`

	let context = canvasEl.getContext('2d')
	context.scale(dpr, dpr)
	context.imageSmoothingEnabled = true
	context.imageSmoothingQuality = 'high'
	return context
}

// 3) drawBitmap — aspect‐fit + HiDPI canvas setup + draw
function drawBitmap(bitmap) {
	// 1) compute “contain” size
	let vpW = window.innerWidth
	let vpH = window.innerHeight
	let { w: cssW, h: cssH } =
		fitToViewport(bitmap.width, bitmap.height, vpW, vpH)

	// 2) size & clear canvas
	let context = setupCanvas(canvasRef.value, cssW, cssH)
	context.clearRect(0, 0, cssW, cssH)

	// 3) draw at the correct aspect ratio
	context.drawImage(bitmap, 0, 0, cssW, cssH)
}

</script>
<template>

<div class="relative w-screen h-screen flex items-center justify-center bg-black overflow-hidden">
	<canvas
		ref="canvasRef"
		class="block max-w-full max-h-full"
	></canvas>
	<div v-show="!pathRef" class="absolute inset-0 flex items-center justify-center text-gray-400 italic select-none pointer-events-none">
		Viewer4 - switched from img src to canvas for gamma and lightweight thumbnails
	</div>
</div>

</template>
<style scoped>

</style>
