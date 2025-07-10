<script setup>//./components/Viewer4.vue

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {ioRead} from '../io.js'

const canvasRef = ref(null)
const isLoaded  = ref(false)
let unlistenFileDrop = null//will hold the unsubscribe function set above and called below

onMounted(async () => {
	const w = getCurrentWindow()
	unlistenFileDrop = await w.onDragDropEvent(event => {
		if (event.payload.type === 'drop' && event.payload.paths.length) {
			let p = event.payload.paths[0]
			console.log(`dropped path "${p}"`)
			loadImage(p)
		}
	})
})
onBeforeUnmount(() => {
	if (unlistenFileDrop) unlistenFileDrop()
})

async function loadImage(p) {
	if (!canvasRef.value) { console.log('canvas not ready yet'); return }

	isLoaded.value = false
	const cvs = canvasRef.value

	// choose a CSS size that “contains” inside the viewport:
	// here we use the smaller of width/height so it stays fully visible
	const cssWidth  = window.innerWidth
	const cssHeight = window.innerHeight
	const size = Math.min(cssWidth, cssHeight)

	const ctx = setupCanvas(cvs, size)
	await drawImageAtSize(ctx, ioRead, p, size)

	isLoaded.value = true
}

function getMimeType(path) {
	const ext = path.split('.').pop().toLowerCase()
	switch (ext) {
		case 'jpg':
		case 'jpeg': return 'image/jpeg'
		case 'png':  return 'image/png'
		case 'webp': return 'image/webp'
		case 'gif':  return 'image/gif'
		default:     return 'application/octet-stream'
	}
}

/**
 * Prepare a canvas for Hi-DPI rendering.
 * - `size` is the CSS size (e.g. match your container or props.size).
 * - Returns the 2D rendering context, already scaled & smoothed.
 */
function setupCanvas(canvasEl, size) {
	const dpr = window.devicePixelRatio || 1
	canvasEl.width  = size * dpr
	canvasEl.height = size * dpr
	canvasEl.style.width  = `${size}px`
	canvasEl.style.height = `${size}px`

	const ctx = canvasEl.getContext('2d')
	ctx.scale(dpr, dpr)
	ctx.imageSmoothingEnabled = true
	ctx.imageSmoothingQuality = 'high'
	return ctx
}

/**
 * Read raw bytes via your ioRead(), turn into an ImageBitmap,
 * then draw it once onto the given context at (0,0) to fill `size×size`.
 */
async function drawImageAtSize(ctx, ioRead, path, size) {
	try {
		const bytes  = new Uint8Array(await ioRead(path))
		const mime   = getMimeType(path)
		const blob   = new Blob([bytes.buffer], { type: mime })
		const bitmap = await createImageBitmap(blob)
		ctx.drawImage(bitmap, 0, 0, size, size)
		bitmap.close?.()
	} catch (err) {
		console.error('Failed to decode image:', err)
	}
}

</script>
<template>

<div class="relative w-screen h-screen flex items-center justify-center bg-black overflow-hidden">
	<canvas
		ref="canvasRef"
		class="block max-w-full max-h-full"
	></canvas>
	<div v-show="!isLoaded" class="absolute inset-0 flex items-center justify-center text-gray-400 italic select-none pointer-events-none">
		Viewer4 - switched from img src to canvas for gamma and lightweight thumbnails
	</div>
</div>

</template>
<style scoped>

</style>
