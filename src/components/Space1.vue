<script setup>//./components/Space1.vue - infinite pannable tabletop (current best)

//keep, infinite pannable tabletop

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {xy, raf, blobToDataUrl, forwardize, backize, lookPath, readImage, renderImage} from './library.js'//our javascript library

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

let arrow1 = xy(0, 0)//arrow1 points from the corner of the frame to the center of the pannable space; changes when the user drags to pan
let arrow2 = xy(0, 0)//arrow2 points from the center of the pannable space to corner of the card; changes when we zoom
function onResize() {//called once on mounted and whenever the viewport size changes
	console.log('on resize! ↘️')
	//...
}
function move(segment) {//move the card under the frame by the given segment
	arrow1 = xy(arrow1, '+', segment)
	let a12 = xy(arrow1, '+', arrow2)
	cardRef.value.style.transform = `translate(${a12.x}px, ${a12.y}px)`
	frameRef.value.style.backgroundPosition = `${a12.x % 60}px ${a12.y % 60}px`
	//^the GPU moves the card to the new pan position, elements nested within the card automatically move along; the second line of code makes the frame's repeating background stay in the same place underneath
}

const cardSpan = xy(800, 600)//dimensions of rectangular div the user can drag to pan around in space
let zoomAmount = 1//2 for 2x, in css pixels, 1 for exactly the card size
const zoomStep = 1.1
function zoom(direction) {
	zoomAmount = direction ? zoomAmount * zoomStep : zoomAmount / zoomStep
	//...
}




const _dimension = {//our complete set of state variables, from which we can position everything
	dhp: 0,//diamond half permimiter, screen width + height, set on startup and upon entering full screen
	zoom: 1,//zoom multiplier, set on zoom step or drag
	arrow1: xy(0, 0),//frame corner to frame middle, set on startup and upon entering full screen
	arrow2: xy(0, 0),//frame middle to space origin, set during pan on each drag segment
	image: xy(0, 0),//image file width and height, set on image load or flip to next image
	tile: xy(60, 60),//table tile width and height
}

function displayCompute(dimension) {//from the given dimension information, calculate display styles

	//going to look at dhp, arrow1, arrow2, zoom, image width and height
	//to then compute translate and card


	return {
		translate:'',
		card:'',
	}

}


function displayUpdate(d) {//given a new display request, change style only if necessary, and save it as official current state

	if (
		xy(_displayed.translate, '==', d.translate) &&//the tabletop center is the same distance panned away from the frame corner
		xy(_displayed.tile,      '==', d.tile)//the repeating tabletop tiled pattern has the same dimensions
	) {/* no space translation necessary */} else {

		cardRef.value.style.transform = `translate(${d.translate.x}px, ${d.translate.y}px)`
		frameRef.value.style.backgroundPosition = `${d.translate.x % d.tile.x}px ${d.translate.y % d.tile.y}px`
	}

	if (
		xy(_displayed.card.home, '==', d.card.home) &&//the card is in the same size and place
		xy(_displayed.card.span, '==', d.card.span)
	) {/* no card top/left/width/height necessary */} else {

		cardRef.value.style.top = d.card.home.x+'px'
		cardRef.value.style.left = d.card.home.y+'px'
		cardRef.value.style.width = d.card.span.x+'px'
		cardRef.value.style.height = d.card.span.y+'px'
	}

	_displayed = d//record what we set on the page to skip an upcomming change that wouldn't be necessary
} let _displayed//our record of how we've set the page to appear, treat as private to display update








function onPointerMove(e) { if (!drag) return
	let segment = xy(e.clientX - drag.start.x, e.clientY - drag.start.y)//the segment, positive x to the right and y down, of the segment the mouse just did during the current drag
	drag.start = xy(e.clientX, e.clientY)//get ready for the next drag segment
	move(segment)
}

const hud1Ref = ref('upper left')
const hud2Ref = ref('System operating according to normal parameters')
const hud3Ref = ref(`lower left, this one is longer
and could involve a second line of text, which could be quite long; text that grows to the width of the frame does wrap
or even a third line`)
const hud4Ref = ref(`middle of frame
this HUD will likely be a card showing the user all the
keyboard shortcuts the app supports, and be really easy to
show and hide, such as by pressing the [H]elp or just [Spacebar]
and here is yet another line`)

const captionRef = ref(`This paragraph, and the next, demonstrate a caption beneath the card. They don't affect the dimensions of the card.
Here's a second line, another paragraph tag. They do pan with the card. If the card size changes, such as a zoom in, the text stays the same size and spot beneath the card.`)//no terminating newline, if that matters

</script>
<template>

<!-- Frame: single outer div sized to component; handles clicks and has repeating background we'll translate along with the card below -->
<div
	ref="frameRef"
	class="myFrame myDots myWillChangeBackgroundPosition relative w-screen h-screen overflow-hidden select-none touch-none"
	@contextmenu.prevent
	@dblclick.prevent="onDoubleClick"
	@pointerdown="onPointerDown"
	@pointermove="onPointerMove"
	@pointerup="onUp" @pointercancel="onUp" @lostpointercapture="onUp"
>

	<!-- Card: rectangular image; drag to pan around in infinite space; caption text is within card but positioned below card -->
	<div
		ref="cardRef"
		class="myCard myShadow myDry myWillChangeTransform bg-gray-200 border border-cyan-500"
		:style="{width: cardSpan.x+'px', height: cardSpan.y+'px'}"
	>

		<!-- we could also put stuff inside the card box, like this orange box -->
		<div class="border border-orange-500 m-4 h-32"></div>
		<!-- caption lives inside the card, but sits below its border -->
		<div class="absolute bottom-0 translate-y-full text-gray-600 py-2 whitespace-nowrap myEmbossed">{{captionRef}}</div>

	</div>

	<!-- hud text here, inside the frame, next to the card -->
	<div v-if="hud1Ref" class="myHud myDry absolute top-4 left-4">{{hud1Ref}}</div>
	<div v-if="hud2Ref" class="myHud myDry absolute top-4 right-4">{{hud2Ref}}</div>
	<div v-if="hud3Ref" class="myHud myDry absolute bottom-0 inset-x-0">{{hud3Ref}}</div>
	<div v-if="hud4Ref" class="myHud myDry absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">{{hud4Ref}}</div>

</div>

</template>
<style scoped>

.myHud {
	color: rgba(255, 255, 255, 0.8); /* transparent text */
	background-color: rgba(0, 0, 0, 0.4); /* smokey plastic from 1980 */
	padding: 0.1rem 0.4rem; /* square corners */
	font-family: monospace;
	font-size: 0.875rem;
	white-space: pre-wrap; /* honor \n and wrap at the container width */
}
.myFrame {
}
.myCard {
}

.myDry, .myDry * { /* on the div with this class and everything deep inside it */
	pointer-events: none; /* none of those elements need to know about clicks */
	user-select: none; /* none of those elements have text the user should be able to select */
}
.myWillChangeTransform          { will-change: transform;           }
.myWillChangeBackgroundPosition { will-change: background-position; } /* with the styles, you get 2 layers in dev tools Layers */

.myDots {
	background-color: #fdfdfd; /* an off-white that still reads white but lets layers below peek through */
	background-image: radial-gradient(circle at center, #ffd1dc 6px, transparent 6px); /* a pastel-pink dot layer, centered in each cell */
	background-size: 60px 60px; /* make each “tile” big enough that the full 16px-diameter dot never hits an edge */
	background-position: 0 0, 30px 30px; /* offset the second layer by half a cell → diamond pattern */
}
.myShadow {
	box-shadow: 0 8px 16px rgba(0, 0, 0, 0.25);
}
.myEmbossed {
	white-space: pre; /* honor \n and overflow the container */
	text-shadow:
		-1px -1px 0 rgba(255,255,255,0.8),
		1px 1px 2px rgba(0,0,0,0.3);
}

</style>
