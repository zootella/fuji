<script setup>//./components/Pan1.vue

const imageSize = 900//counted in raster image file pixels which render 1:1 to device pixels
const tableSize = 2500//counted in CSS pixels, which make sense to the user

import {ref, onMounted, onBeforeUnmount} from 'vue'

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
	ctx.fillStyle = 'cyan'
	ctx.fillRect(0, 0, imageSize, imageSize)

	//red 1px circle
	drawCircle(canvas, ctx, 'red')
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

function fixCanvas(canvas) {//ensure the given canvas ref draws to device pixels on the screen, not css pixels

	const dpr = window.devicePixelRatio || 1//likely 1.0, 1.25, 1.5, 2.0; possible 1.3333333333333333, 1.6666666666666667, 2.25, 2.5, 3.0
	console.log(`the device pixel ratio here is: ${dpr} 🖥️`)

	// 2) CSS size = imageSize / dpr → 1 buffer px → 1 device px
	canvas.style.width  = (canvas.width / dpr) + 'px'
	canvas.style.height = (canvas.height / dpr) + 'px'
}

onMounted(async () => {

	drawCanvas(canvasRef.value)//pass declared canvas ref for the function to draw into
	fixCanvas(canvasRef.value)//fix canvas pixels to hardware display physical pixels
})

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
