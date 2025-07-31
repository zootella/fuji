<script setup>//./components/Pan1.vue - first experiment panning around a large tabletop with an image in the middle

const imageSize = 900
const tableSize = 2500

import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window or whole screen
const tableRef = ref(null)//background yellow on black grid the user will be able to drag to pan around
const imageRef = ref(null)//on top of that, centered, rendered to device pixels, the img tag that holds the image

/*
ok, here im saving the dragging pointer id--all look simple correct good?
*/

let tablePosition = {x: 0, y: 0}
let dragStart = {x: 0, y: 0}
let draggingPointer = null

onMounted(() => {

	tableRef.value.style.transformOrigin = '0 0'
	tableRef.value.style.transform = `translate(${tablePosition.x}px, ${tablePosition.y}px)`
	tableRef.value.addEventListener('pointerdown', panDown)//start listening for drags

	frameRef.value.addEventListener('pointermove', showCursor)//mouse moving over frame, show the pointer
	frameRef.value.addEventListener('pointerup',   showCursor)//mouse finished drag, show the pointer
	frameRef.value.addEventListener('pointerdown', hideCursor)//mouse starting drag, hide the pointer
	hideTimer = setTimeout(() => frameRef.value.classList.add('cursor-hidden'), HIDE_DELAY)
})
onBeforeUnmount(() => {

	tableRef.value.removeEventListener('pointermove', panMove)//remove any remaining drag listeners
	tableRef.value.removeEventListener('pointerup', panUp)
	tableRef.value.removeEventListener('pointerdown', panDown)
	if (draggingPointer) {
		tableRef.value.releasePointerCapture(draggingPointer)
		draggingPointer = null
	}

	frameRef.value.removeEventListener('pointermove', showCursor)
	frameRef.value.removeEventListener('pointerup',   showCursor)
	frameRef.value.removeEventListener('pointerdown', hideCursor)
	clearTimeout(hideTimer)
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

/*
watch for changes to the frame size--you need to keep the table over the frame, like dragging the upper left corner bigger

with this design as a starting point, how hard will it be to...
- during a drag, show the pointer with the os's grabbing hand?
- when the pointer is at rest above the tabletop, hiding it entirely?

grab and grabbing are a mac only thing, and look weird
ok so here's what you want
over the tabletop, pointer disappears when it falls to rest
during a drag, pointer is also off
*/

/*
ok, my work today is done, in that we've got the tabletop dragging around!

now, let's peek at what i have to do tomorrow
ill add an img tag below
the img will always be smaller than the tabletop (and the tabletop will always be larger than the screen!)
is this going to mess up anything with the clicks and drags?
my hope is that no, the html components that i put within the two divs will just stay where they are on the tabletop
even while the user pans the tabletop around
and that won't slow down gpu or negatively impact the current achievement of buttery smoothness at all!

after that, ill add a HUD--just a few lines of text that float above the tabletop and image
they'll be pinned to one corner or edge of the frame
they are just information, they should not be nor need to be clickable, selectable, anything
for instance, it might be text that shows information about the image in the lightbox
is that going to require or indicate any changes into how we've coded panning here?
my expectation and hope is no, it's not

and as to buttery smoothness, while the text may be antialiased, or on a rectangular panel that is translucent
GPUs are ready for this, so if the user flips the info hud on, they can still pan the image around below with the same experience

but let's take a step back and check now, ahead of more coding, for potential issues with this waypoint's design
looking now ahead at these two new features we'll work on tomorrow
1 image sitting on tabletop
2 hud in frame

on, in full screen mode, hide the mouse pointer entirely, likely
*/

// Track our hide timer and delay
let hideTimer = null
const HIDE_DELAY = 200  // milliseconds
// Show the cursor and restart the hide timer
function showCursor() {
	frameRef.value.classList.remove('cursor-hidden')
	clearTimeout(hideTimer)
	hideTimer = setTimeout(() => {
		frameRef.value.classList.add('cursor-hidden')
	}, HIDE_DELAY)
}
// Hide the cursor immediately
function hideCursor() {
	clearTimeout(hideTimer)
	frameRef.value.classList.add('cursor-hidden')
}











/*
factor this into one tight little area
one function
one object of state

command:

mount
unmount - component mounted and on before unmounted

enable
disable - turn on the disappearing behavior

move
up
down - things we've seen the mouse do in this component

*/

const pointerState = {//single object of factory settings and current state
	delay: 200,
	moved: 0,//tick when we last saw the pointer move
	dragging: false,
	visible: true,

}
function pointerReport(o) {//single control function for one line calls elsewhere in this component

	if        (o.command == 'mount') {
	} else if (o.command == 'unmount') {

	} else if (o.command == 'enable') {
	} else if (o.command == 'disable') {

	} else if (o.command == 'down') {
	} else if (o.command == 'up') {
	} else if (o.command == 'move') {
	}
}
function pointerFrame() {//single function that, with behavior enabled, runs quickly on every animation frame

}



const HIDE_DELAY = 200  // ms threshold

//–– State for our loop
let lastMoveTime    = performance.now()
let isDragging      = false
let isCursorHidden  = false
let rafId           = null

function showCursor(makeVisible, givenLastMoveTime, givenDetails) {
	if (givenLastMoveTime) lastMoveTime = givenLastMoveTime
	if (givenDetails) isDragging = givenDetails.dragging

	if (makeVisible && isCursorHidden) {//show
		isCursorHidden = false
		frameRef.value.classList.remove('cursor-hidden')
	} else if (!makeVisible && !isCursorHidden) {//hide
		isCursorHidden = true
		frameRef.value.classList.add('cursor-hidden')
	}
}

function trackPointerMove() {
  lastMoveTime = performance.now()
}

function panDown(e) {
  showCursor(false, performance.now(), {dragging: true})
}

function panMove(e) {
  lastMoveTime = performance.now()
}

function panUp(e) {
  showCursor(true, performance.now(), {dragging: false})
}

function cursorLoop(now) {
  if (isDragging) {
    showCursor(false)
  } else {
    if (HIDE_DELAY + lastMoveTime < now) {
      showCursor(false)
    } else {
      showCursor(true)
    }
  }
  rafId = requestAnimationFrame(cursorLoop)
}

onMounted(() => {
  const frame = frameRef.value
  // start tracking moves
  frame.addEventListener('pointermove', trackPointerMove)
  // start the loop
  rafId = requestAnimationFrame(cursorLoop)
})

onBeforeUnmount(() => {
  const frame = frameRef.value
  frame.removeEventListener('pointermove', trackPointerMove)
  cancelAnimationFrame(rafId)
})
</script>





































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

.myFrame { cursor: default; }/* By default, show the regular arrow pointer anywhere in the frame */
.myFrame.cursor-hidden { cursor: none !important; }/* When we’re “hidden,” nuke the cursor */

</style>
