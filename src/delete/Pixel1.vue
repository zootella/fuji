<script setup>//./components/Pixel1.vue - a boondoggle trying to draw a raster image to hardware pixels

//delete because fine you win we'll never get to be sure we can write to actual device pixels; upcoming rust code may still try to get the monitor resolution, though

const imageSize = 900//counted in raster image file pixels which render 1:1 to device pixels
const tableSize = 2500//counted in CSS pixels, which make sense to the user

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {invoke} from '@tauri-apps/api/core'

const tableRef = ref(null)//background yellow on black grid the user will be able to drag to pan around
const canvasRef = ref(null)//on top of that, centered, rendered to device pixels, the canvas where we render the image

function drawCanvas(canvas) {//passing the ref value because we must draw into declared canvas

	const dpr = window.devicePixelRatio || 1

	//set its internal buffer to device pixels
	canvas.width  = imageSize
	canvas.height = imageSize

	//draw into it
	const ctx = canvas.getContext('2d')
	ctx.imageSmoothingEnabled = false

	//cyan fill
	ctx.fillStyle = '#fff'
	ctx.fillRect(0, 0, imageSize, imageSize)

	//red 1px circle
	drawStripes(canvas, ctx, '#000', 1)
}

function drawCircle(canvas, ctx, color) {
	const size   = canvas.width//assume square canvas
	const center = (size - 1) / 2//true pixel‑center, e.g. 449.5 for size=900
	const radius = center//radius = distance to any edge

	// number of steps ≈ one per pixel of circumference
	const steps = Math.max(12, Math.round(2 * Math.PI * radius))
	let prevX, prevY

	ctx.fillStyle = color

	// Bresenham‑style 1px line
	function drawLine(x0, y0, x1, y1) {
		let dx = Math.abs(x1 - x0),
				dy = Math.abs(y1 - y0),
				sx = x0 < x1 ? 1 : -1,
				sy = y0 < y1 ? 1 : -1,
				err = dx - dy;

		while (true) {
			ctx.fillRect(x0, y0, 1, 1)
			if (x0 === x1 && y0 === y1) break;
			const e2 = err * 2
			if (e2 > -dy) { err -= dy; x0 += sx; }
			if (e2 <  dx) { err += dx; y0 += sy; }
		}
	}

	for (let i = 0; i <= steps; i++) {
		const theta = (2 * Math.PI * i) / steps
		const x = Math.round(center + Math.cos(theta) * radius)
		const y = Math.round(center + Math.sin(theta) * radius)
		if (prevX !== undefined) {
			drawLine(prevX, prevY, x, y)
		} else {
			ctx.fillRect(x, y, 1, 1)
		}
		prevX = x
		prevY = y
	}
}

function drawStripes(canvas, ctx, color, stripeWidth) {
	const width  = canvas.width
	const height = canvas.height
	ctx.fillStyle = color
	for (let x = 0; x < width; x += stripeWidth * 2) {
		// draw a stripe of given width, but clip at canvas edge
		const w = Math.min(stripeWidth, width - x)
		ctx.fillRect(x, 0, w, height)
	}
}

function fixCanvas(canvas) {//ensure the given canvas ref draws to device pixels on the screen, not css pixels

	let actual = {width: 3840, height: 2160}//imagine we can get this from rust somehow, no known way yet!!
	let mistake = 5//bump slightly wider like 5 to see beats

	/*
	actual, known,   real pixels, whole screen, from rust
	screen, known,   fake pixels, whole screen, from window.screen.height
	canvas, known,   real pixels, just image, from canvas.height
	style,  compute, fake pixels, just image, set canvas.style.height

	style * actual = canvas * screen
	style = canvas * (screen / actual)
	style = canvas * scale
	*/

	let scale = window.screen.height / actual.height
	console.log(`🖥️ ${window.devicePixelRatio} device pixel ratio, ${window.screen.height} window screen height. ${actual.height} known actual height, ${scale} computed scale`)

	// 2) CSS size = imageSize / dpr → 1 buffer px → 1 device px
	canvas.style.width  = ((canvas.width  * scale) + mistake) + 'px'
	canvas.style.height = ((canvas.height * scale) + 0)       + 'px'
}

onMounted(async () => {

	drawCanvas(canvasRef.value)//pass declared canvas ref for the function to draw into
	fixCanvas(canvasRef.value)//fix canvas pixels to hardware display physical pixels

	await snippet()
})

async function snippet() {

	let [w1, h1, s1] = await invoke('count_pixels_1')
	let [w2, h2, s2] = await invoke('count_pixels_2')
	/*
	console.log(`${w1} × ${h1} scale ${s1}, from rust 1`)
	console.log(`${w2} × ${h2} scale ${s2}, from rust 2`)
	*/
}

</script>
<template>

<div class="relative w-screen h-screen overflow-hidden bg-black">
	<div
		ref="tableRef"
		class="absolute top-0 left-0 myTableGrid"
		:style="{ width: tableSize + 'px', height: tableSize + 'px' }"
	>
		<canvas
			ref="canvasRef"
			class="absolute top-0 left-0 myPixelCanvas"
			:style="{ width: imageSize + 'px', height: imageSize + 'px' }"
		/>
	</div>
</div>

</template>
<style scoped>

.myPixelCanvas {
	/* nearest‑neighbor scaling */
	image-rendering: pixelated;
	image-rendering: crisp-edges;
	-ms-interpolation-mode: nearest-neighbor;
}

.myTableGrid {
	background-color: black;
	background-image:
		linear-gradient(to right, yellow 1px, transparent 1px),
		linear-gradient(to bottom, yellow 1px, transparent 1px);
	background-size: 500px 500px;
}

</style>
