<script setup>//./components/Space1.vue - mvp infinite lighttable for full screen

import {getCurrentWindow} from '@tauri-apps/api/window'
import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window full screen
const cardRef = ref(null)//a rectangle in space the user can drag to pan around, anywhere including far outside the frame viewport

onMounted(() => {
	frameRef.value.addEventListener('wheel', onWheel, {passive: false})
	window.addEventListener('keydown', onKey)
	window.addEventListener('resize', onResize); onResize()//and call right away to look at the starting size
})
onBeforeUnmount(() => {
	frameRef.value.removeEventListener('wheel', onWheel)
	window.removeEventListener('keydown', onKey)
	window.removeEventListener('resize', onResize)
	if (drag?.pointer && frameRef.value.hasPointerCapture(drag.pointer)) {
		frameRef.value.releasePointerCapture(drag.pointer)
		drag.pointer = null
	}
})

async function onKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//ignore keystrokes into a form field

	if      (e.key == 'f') { console.log('my key F') }
	else if (e.key == 'q') { console.log('my key Q') }
	else if ((e.ctrlKey || e.metaKey) && e.key == 's') { console.log('my key Ctrl+S')
		e.preventDefault()//tell the browser not to show the file save dialog box
	} else if (e.key == 'Escape') {
		await setFullscreen(false)//macos will also exit fullscreen, but this call doesn't mess anything up with that
	}
	else if (e.key == 'ArrowLeft')  { console.log('my key ArrowLeft')  }
	else if (e.key == 'ArrowRight') { console.log('my key ArrowRight') }
	else if (e.key == 'ArrowUp')    { console.log('my key ArrowUp')    }
	else if (e.key == 'ArrowDown')  { console.log('my key ArrowDown')  }
	else if (e.key == 'PageUp')     { console.log('my key PageUp')     }
	else if (e.key == 'PageDown')   { console.log('my key PageDown')   }
	else if (e.key == '+')          { zoom(true)  }
	else if (e.key == '-')          { zoom(false) }
}
async function setFullscreen(set) { let w = getCurrentWindow(); let current = await w.isFullscreen()
	if (set != current) w.setFullscreen(set)
}
async function toggleFullscreen() { let w = getCurrentWindow(); let current = await w.isFullscreen()
	w.setFullscreen(!current)
}
function onWheel(e) {
	e.preventDefault()//tell the browser not to scroll

	let s = `wheel ${e.deltaX} Δx, ${e.deltaY} Δy`
	if (e.ctrlKey)  s += ' +Ctrl'
	if (e.shiftKey) s += ' +Shift'//with shift held on mac, delta y is 0 and x is positive or negative
	if (e.metaKey)  s += ' +Meta'//testing on mac with karabiner elements and the microsoft keyboard, always seeing meta, never ctrl
	console.log(s)

	if      (e.deltaY < 0) { zoom(true)  }
	else if (e.deltaY > 0) { zoom(false) }
}//ttd august, see how this flips out on a touchpad, though

async function onDoubleClick(e) { await toggleFullscreen() }
function onPointerDown(e) {
	if (e.button == 0 && e.detail == 2 && e.buttons == 1) {//primary button 0, 2nd quick click, first bit value 1 only button down right now
		//ignoring this because listening for browser double click event
	} else if (e.button == 2 && e.detail == 2 && e.buttons == 2) {//secondary button 2, 2nd quick click, second bit value 2 only button down right now
		console.log('pointer down: right double click')
	} else {
		dragStart(e)
	}
}
let drag//an object of positions and ids during a left or right click drag
function dragStart(e) {
	drag = {
		button: e.button,//0 primary or 2 secondary mouse button
		start: {x: e.clientX, y: e.clientY},//record where this drag started
		pointer: e.pointerId,
	}
	frameRef.value.setPointerCapture(e.pointerId)//watch the mouse during the drag; works even when dragged outside the window!
}
function onUp(e) {
	frameRef.value.releasePointerCapture(e.pointerId)//should be the same as drag.pointer
	drag = null//discard the drag object, getting things ready for the next drag
}

let arrow1 = {x: 0, y: 0}//arrow1 points from the corner of the frame to the center of the pannable space; changes when the user drags to pan
let arrow2 = {x: 0, y: 0}//arrow2 points from the center of the pannable space to corner of the card; changes when we zoom
function onResize() {//called once on mounted and whenever the viewport size changes
	console.log('on resize! ↘️')
	//...
}
function move(segment) {//move the card under the frame by the given segment
	arrow1 = math(arrow1, '+', segment)
	let a12 = math(arrow1, '+', arrow2)
	cardRef.value.style.transform = `translate(${a12.x}px, ${a12.y}px)`//have the GPU move the card to the new pan location; the stuff within it rides along
}

const cardSize = {w: 800, h: 600}//dimensions of rectangular div the user can drag to pan around in space
let zoomAmount = 1//2 for 2x, in css pixels, 1 for exactly the card size
const zoomStep = 1.1
function zoom(direction) {
	zoomAmount = direction ? zoomAmount * zoomStep : zoomAmount / zoomStep
	//...
}

function onPointerMove(e) { if (!drag) return
	let segment = {//the segment, positive x to the right and y down, of the segment the mouse just did during the current drag
		x: e.clientX - drag.start.x,
		y: e.clientY - drag.start.y
	}
	drag.start = {//get ready for the next drag segment
		x: e.clientX,
		y: e.clientY
	}
	move(segment)
}

function arrow(x, y) { return {x, y} }
function math(a1, operator, a2) {
	if      (operator == '+') { return {x: a1.x + a2.x, y: a1.y + a2.y} }//use with two {x, y} objects
	else if (operator == '-') { return {x: a1.x - a2.x, y: a1.y - a2.y} }
	else if (operator == '*') { return {x: a1.x * a2,   y: a1.y * a2  } }//use with ane xy object and a number, like 2
	else if (operator == '/') { return {x: a1.x / a2,   y: a1.y / a2  } }
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
	@pointermove="onPointerMove"
	@pointerup="onUp" @pointercancel="onUp" @lostpointercapture="onUp"
>

	<!-- inner div image the user drags in the frame to pan the card and its contents around in the infinite space -->
	<div
		ref="cardRef"
		class="myCard myDry bg-gray-200 border border-cyan-500 will-change-transform"
		:style="{width: cardSize.w+'px', height: cardSize.h+'px'}"
	>

		<!-- orange-bordered inner box -->
		<div class="border border-orange-500 m-4 h-32"></div>

		<!-- caption lives inside the card, but sits below its border -->
		<div
			class="absolute bottom-0 translate-y-full text-gray-600 py-2 whitespace-nowrap"
		>
			<p>
				This paragraph, and the next, demonstrate a caption beneath the card.
				They don't affect the dimensions of the card.
			</p>
			<p>
				Here's a second line, another paragraph tag.
				They do pan with the card.
				If the card size changes, such as a zoom in, the text stays the same size and spot beneath the card.
			</p>
		</div>

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
