<script setup>//./components/Space1.vue - mvp infinite lighttable for full screen

import {getCurrentWindow} from '@tauri-apps/api/window'
import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window full screen
const cardRef = ref(null)//a rectangle in space the user can drag to pan around, anywhere including far outside the frame viewport

onMounted(async () => {
	moveStartup()
	window.addEventListener('keydown', onKey)
	frameRef.value.addEventListener('wheel', onWheel, {passive: false})
})
onBeforeUnmount(() => {
	window.removeEventListener('keydown', onKey); if (drag?.pointer) {
		frameRef.value.releasePointerCapture(drag.pointer); drag.pointer = null }
	frameRef.value.removeEventListener('wheel', onWheel)
})

async function onKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//ignore keystrokes into a form field

	if (e.key == 'f') {
		console.log('my key F, which goes full screen')
		await setFullscreen(true)

	} else if (e.key == 'q') {
		console.log('my key Q, which quits full screen')
		await setFullscreen(false)

	} else if ((e.ctrlKey || e.metaKey) && e.key == 's') {
		e.preventDefault()//tell the browser not to show the file save dialog box
		console.log('my key Ctrl+S')

	} else if (e.key == 'Escape') {
		console.log('my key Escape, which causes both us and macos to leave full screen')
		await setFullscreen(false)//macos will also exit fullscreen, but this call doesn't mess anything up with that

	} else if (e.key == 'ArrowLeft')  { console.log('my key ArrowLeft')  }
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
function moveStartup() {//called once on mounted
	arrow2 = {//set the starting arrow2 based on the card size with no zoom
		x: -(cardSize.w / 2),
		y: -(cardSize.h / 2)
	}
	move(arrowFrameMiddle())//set arrow1 and pan there to put the middle of the card in the middle of the frame
}
function move(segment) {//move the card under the frame by the given segment
	arrow1 = arrowMath(arrow1, '+', segment)
	let a12 = arrowMath(arrow1, '+', arrow2)
	cardRef.value.style.transform = `translate(${a12.x}px, ${a12.y}px)`//have the GPU move the card to the new pan location; the stuff within it rides along
}

const cardSize = {w: 800, h: 600}//dimensions of rectangular div the user can drag to pan around in space
let zoomAmount = 1//2 for 2x, in css pixels, 1 for exactly the card size
const zoomStep = 1.1
function zoom(direction) {
	zoomAmount = direction ? zoomAmount * zoomStep : zoomAmount / zoomStep

	let m = arrowFrameMiddle()//m points from frame corner to frame middle
	let a12 = arrowMath(arrow1, '+', arrow2)//a12 points from frame corner to card corner

	//calculate the arrows which will grow or shrink with this zoom, all pointing from the frame center
	let f1 = arrowMath(arrow1, '-', m)//f1 points from frame center to card center
	let f2 = arrowMath(a12,    '-', m)//f2 points from frame center to card corner

	console.log(zoomAmount)


	/*
	clear off the drafting table
	code just diamond now, but have a plan to do all of them

	what changes at startup?
	what changes on drag?
	what changes on flip to next image?
	what changes on 

	what are the minimal arrows that need to be kept in state?
	what arrows can derive from what?

	have a single function place(a1, a2)
	which is the only place that sets the arrows (but you can read them elsewhere)
	and is the only place that does the transform and size

	*/


	//zoom them
	f1 = arrowMath(f1, '*', zoomAmount)
	f2 = arrowMath(f2, '*', zoomAmount)

	//calculate new arrows 1 and 2, and new card with and height
	arrow1 = arrowMath(m,  '+', f1)
	arrow2 = arrowMath(f2, '-', f1)
	a12 = arrowMath(arrow1, '+', arrow2)
	a12.w = -2*arrow2.x
	a12.h = -2*arrow2.y

	cardRef.value.style.transform = `translate(${a12.x}px, ${a12.y}px)`
	cardRef.value.style.width  = `${a12.w}px`
	cardRef.value.style.height = `${a12.h}px`
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

function arrowCardCorner() {//based on the image dimensions and zoom amount, points from the card center to corner
	return {
		x: ,
		y: 
	}
}
function arrowFrameMiddle() {//based on how big the window is right now, get the arrow from the frame corner to the frame middle
	return {
		x: frameRef.value.clientWidth  / 2,
		y: frameRef.value.clientHeight / 2
	}
}
function arrowMath(a1, operator, a2) {
	if      (operator == '+') { return {x: a1.x + a2.x, y: a1.y + a2.y} }
	else if (operator == '-') { return {x: a1.x - a2.x, y: a1.y - a2.y} }
	else if (operator == '*') { return {x: a1.x * a2,   y: a1.y * a2  } }
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
