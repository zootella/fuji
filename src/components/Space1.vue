<script setup>//./components/Space1.vue - mvp infinite lighttable for full screen

const cardSize = {w: 800, h: 600}//dimensions of rectangular div the user can drag to pan around in space

import {getCurrentWindow} from '@tauri-apps/api/window'
import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window full screen
const cardRef = ref(null)//a rectangle in space the user can drag to pan around, anywhere including far outside the frame viewport

let spacePosition = {x: 0, y: 0}//where we are in space; also sorta where the card is related to the viewport
let drag//an object of positions and ids during a left or right click drag

onMounted(() => {

	//register listeners for input devices that are not related to position; those will go on the table in the template
	window.addEventListener('keydown', onKey)
	frameRef.value.addEventListener('wheel', onWheel, {passive: false})
})
onBeforeUnmount(() => {

	//remove the listeners; there shouldn't be a danglign pointer but release one if there is
	window.removeEventListener('keydown', onKey)
	frameRef.value.removeEventListener('wheel', onWheel)
	if (drag?.pointer) { cardRef.value.releasePointerCapture(drag.pointer); drag.pointer = null }
})

async function onKey(e) {
}
async function setFullscreen(set) { let w = getCurrentWindow(); let current = await w.isFullscreen()
}
async function toggleFullscreen() { let w = getCurrentWindow(); let current = await w.isFullscreen()
}
function onWheel(e) {
}//ttd august, see how this flips out on a touchpad, though
async function onDoubleClick(e) {
}
function onPointerDown(e) {
}
function dragStart(e) {
}
function onMove(e) { if (!drag) return
}
function onUp(e) {
}

</script>
<template>

<!-- outer div frame sized to component, often the tauri window renderer viewport, frequently the full screen -->
<div
	ref="frameRef"
	class="myFrame relative w-screen h-screen overflow-hidden select-none touch-none"
	@contextmenu.prevent
	@dblclick.prevent="onDoubleClick"
	@pointerdown="onPointerDown"
	@pointermove="onMove"
	@pointerup="onUp" @pointercancel="onUp" @lostpointercapture="onUp"
>

	<!-- inner div image the user drags in the frame to pan the card and its contents around in the infinite space -->
	<div
		ref="cardRef"
		class="myCard myDry bg-gray-200 border border-cyan-500"
		:style="{width: cardSize.w+'px', height: cardSize.h+'px'}"
	>

		<!-- the img tag will go here, where we expect it will "ride along" with the drag to pan tabletop -->

	</div>

	<!-- hud text will go here, inside the frame, alongside the tabletop; maybe put all that in a new glassRef div -->

</div>

</template>
<style scoped>

.myDry,
.myDry * { /* on the div with this class and everything deep inside it */
  pointer-events: none; /* none of those elements need to know about clicks */
	user-select: none; /* none of those elements have text the user should be able to select */
}

.myFrame {
}
.myCard {
}

</style>
