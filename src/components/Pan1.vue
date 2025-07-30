<script setup>//./components/Pan1.vue - first experiment panning around a large tabletop with an image in the middle

const imageSize = 900
const tableSize = 2500

import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window or whole screen
const tableRef = ref(null)//background yellow on black grid the user will be able to drag to pan around
const imageRef = ref(null)//on top of that, centered, rendered to device pixels, the img tag that holds the image

/*
ok, starting appearance looks correct, and there are never any errors
but clicking and dragging doesn't move anything!
*/

let tablePosition = {x: 0, y: 0}
let dragStart = {x: 0, y: 0}

onMounted(() => {

	tableRef.value.style.transformOrigin = '0 0'
	tableRef.value.style.transform = `translate(${tablePosition.x}px, ${tablePosition.y}px)`
	tableRef.value.addEventListener('pointerdown', panDown)
})
onBeforeUnmount(() => {

	tableRef.value.removeEventListener('pointerdown', panDown)
})

function panDown(panEvent) {

	dragStart.x = panEvent.clientX//remember where the drag started
	dragStart.y = panEvent.clientY

	tableRef.value.setPointerCapture(panEvent.pointerId)//watch the mouse
	tableRef.value.addEventListener('pointermove', panMove)
	tableRef.value.addEventListener('pointerup', panUp)
}
function panMove(panEvent) {

	tablePosition.x += panEvent.clientX - dragStart.x//move the table
	tablePosition.y += panEvent.clientY - dragStart.y

	dragStart.x = panEvent.clientX//start the next move from here
	dragStart.y = panEvent.clientY

	tableRef.value.style.transform = `translate(${tablePosition.x}px, ${tablePosition.y}px)`//tell the GPU
}

function panUp(panEvent) {

	tableRef.value.releasePointerCapture(panEvent.pointerId)//stop watching the mouse
	tableRef.value.removeEventListener('pointermove', panMove)
	tableRef.value.removeEventListener('pointerup', panUp)
}

</script>
<template>

<!-- outer div sized to page viewport -->
<div ref="frameRef" class="relative w-screen h-screen overflow-hidden">

	<!-- inner tabletop div the user drags around to pan -->
	<div
		ref="tableRef"
		class="bg-black myTable"
		:style="{width: tableSize + 'px', height: tableSize + 'px'}"
	>

		<!-- the img tag will go here, centered and pinned to the moveable tabletop -->

	</div>
</div>

</template>
<style scoped>

.myTable {
	will-change: transform; /* pan with the compositor thread; no layout or paint; buttery-smooth 🧈 */

	/* temporary grid lines so we can see panning */
	background-image:
		linear-gradient(to right, yellow 1px, transparent 1px),
		linear-gradient(to bottom, yellow 1px, transparent 1px);
	background-size: 500px 500px;
}

</style>
