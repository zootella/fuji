<script setup>//./components/Pan1.vue - first experiment panning around a large tabletop with an image in the middle

const imageSize = 900
const tableSize = 2500

import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window or whole screen
const tableRef = ref(null)//background yellow on black grid the user will be able to drag to pan around
const imageRef = ref(null)//on top of that, centered, rendered to device pixels, the img tag that holds the image

let tablePosition = {x: 0, y: 0}
let dragStart = {x: 0, y: 0}
let draggingPointer = null

onMounted(() => {

	tableRef.value.style.transformOrigin = '0 0'
	tableRef.value.style.transform = `translate(${tablePosition.x}px, ${tablePosition.y}px)`
	tableRef.value.addEventListener('pointerdown', panDown)//start listening for drags
})
onBeforeUnmount(() => {

	tableRef.value.removeEventListener('pointermove', panMove)//remove any remaining drag listeners
	tableRef.value.removeEventListener('pointerup', panUp)
	tableRef.value.removeEventListener('pointerdown', panDown)
	if (draggingPointer) {
		tableRef.value.releasePointerCapture(draggingPointer)
		draggingPointer = null
	}
})

function panDown(panEvent) {
	draggingPointer = panEvent.pointerId

	dragStart.x = panEvent.clientX//remember where the drag started
	dragStart.y = panEvent.clientY

	tableRef.value.setPointerCapture(panEvent.pointerId)//watch the mouse
	tableRef.value.addEventListener('pointermove', panMove)
	tableRef.value.addEventListener('pointerup', panUp)
}
function panMove(panEvent) {
	const segment = {//how far did the mouse move in this segment of the drag?
		x: panEvent.clientX - dragStart.x,
		y: panEvent.clientY - dragStart.y,
	}
	dragStart.x = panEvent.clientX//get ready for the next segment if the drag continues
	dragStart.y = panEvent.clientY

	const candidate = {//what would our new table position be?
		x: tablePosition.x + segment.x,
		y: tablePosition.y + segment.y,
	}
	let bound = {//what are the bounds of possible new positions, keeping the frame entirely on top of the table
		minimum: {
			x: frameRef.value.clientWidth  - tableSize,
			y: frameRef.value.clientHeight - tableSize,
		},
		maximum: {x: 0, y: 0},//don't let the table move to the right or below the upper left corner of the frame!
	}
	tablePosition.x = Math.min(bound.maximum.x, Math.max(bound.minimum.x, candidate.x))//clamp to those bounds
	tablePosition.y = Math.min(bound.maximum.y, Math.max(bound.minimum.y, candidate.y))

	tableRef.value.style.transform = `translate(${tablePosition.x}px, ${tablePosition.y}px)`//push to the GPU
}
function panUp(panEvent) {

	tableRef.value.removeEventListener('pointermove', panMove)
	tableRef.value.removeEventListener('pointerup',   panUp)
	tableRef.value.releasePointerCapture(panEvent.pointerId)
	draggingPointer = null
}

</script>
<template>

<!-- outer div frame sized to component, often the tauri window renderer viewport, frequently the full screen -->
<div ref="frameRef" class="relative w-screen h-screen overflow-hidden myFrame">

	<!-- inner div tabletop the user drags around to pan, always larger than the frame -->
	<div
		ref="tableRef"
		class="bg-black myTable"
		:style="{width: tableSize + 'px', height: tableSize + 'px'}"
	>

		<!-- the img tag will go here, where we expect it will "ride along" with the drag to pan tabletop -->

	</div>

	<!-- hud text will go here, inside the frame, alongside the tabletop -->

</div>

</template>
<style scoped>

.myTable {
	will-change: transform; /* pan with the compositor thread; no layout or paint; buttery-smooth 🧈 */
	touch-action: none; /* tell the browser to not try to scroll or zoom behind a drag that we are handling */

	/* temporary grid lines so we can see panning */
	background-image:
		linear-gradient(to right, yellow 1px, transparent 1px),
		linear-gradient(to bottom, yellow 1px, transparent 1px);
	background-size: 500px 500px;
}

.myHud {
	pointer-events: none; /* the HUD is information, only--no buttons, no selectable text */
}

</style>
