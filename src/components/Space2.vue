<script setup>

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {xy, raf, blobToDataUrl, forwardize, backize, lookPath, readImage, renderImage} from './library.js'//our javascript library

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window full screen
const cardRef = ref(null)//a rectangle in space the user can drag to pan around, anywhere including far outside the frame viewport

onMounted(() => {
	dimensionStart()
})
onBeforeUnmount(() => {
	if (drag?.pointer && frameRef.value.hasPointerCapture(drag.pointer)) {
		frameRef.value.releasePointerCapture(drag.pointer)
		drag.pointer = null
	}
})

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
		start: xy(e.clientX, e.clientY),//record where this drag started
		pointer: e.pointerId,
	}
	frameRef.value.setPointerCapture(e.pointerId)//watch the mouse during the drag; works even when dragged outside the window!
}
function onUp(e) {
	frameRef.value.releasePointerCapture(e.pointerId)//should be the same as drag.pointer
	drag = null//discard the drag object, getting things ready for the next drag
}

function onPointerMove(e) { if (!drag) return


	let current = xy(e.clientX, e.clientY)//the new current location of the pointer
	let segment = xy(current, '-', drag.start)//the segment it just moved to get to where it is now
	drag.start = current//get ready for the next drag segment


	quiverA.space = xy(quiverA.space, '+', segment)
	update()


}

/*
quiver A - arrows about what we will show on the page

quiver B - entirely from a, arrows computed to set styles on the page

quiver C - our record of the b arrows we have set on the page
*/


const quiverA = {}
function dimensionStart() {

	quiverA.diamond = screen.width + screen.height//full size diamond half permineter
	quiverA.zoom = 1.5//zoom factor for the diamond permimiter
	quiverA.space = xy(xy(frameRef.value.clientWidth, frameRef.value.clientHeight), '/', 2)//frame corner to space center
	quiverA.natural = xy(300, 200)//natural image pixel width and height from its own file data
	update()
}
function update() {
	let quiverB = {}


	//1 calculate display from _dimension
	let cardSpan = xy(quiverA.natural, '*', quiverA.zoom)
	let cardBackHalf = xy(cardSpan, '/', -2)
	let cardCorner = xy(quiverA.space, '+', cardBackHalf)

	//2 send necessary changes to the page

	frameRef.value.style.backgroundPosition = `${quiverA.space.x % 60}px ${quiverA.space.y % 60}px`
	cardRef.value.style.transform = `translate(${cardCorner.x}px, ${cardCorner.y}px)`
	cardRef.value.style.width = cardSpan.x+'px'
	cardRef.value.style.height = cardSpan.y+'px'

	//3 record what we set on the page to not do something unnecessary next time
	quiverC = quiverB
}
let quiverC//our record of how we've set the page to appear, treat as private to display update


//from here, you might be able to go back and fix myspace
</script>
<template>

<!-- Frame: single outer div sized to component; handles clicks and has repeating background we'll translate along with the card below -->
<div ref="frameRef"
	class="myFrame myDots myWillChangeBackgroundPosition relative w-screen h-screen overflow-hidden select-none touch-none"
	@contextmenu.prevent
	@pointerdown="onPointerDown"
	@pointermove="onPointerMove"
	@pointerup="onUp" @pointercancel="onUp" @lostpointercapture="onUp"
>

	<!-- Card: rectangular image; drag to pan around in infinite space; caption text is within card but positioned below card -->
	<div ref="cardRef" class="myCard myDry myWillChangeTransform bg-gray-200 border border-cyan-500"></div>

</div>

</template>
<style scoped>

.myFrame {}
.myCard {}

.myDry, .myDry * { /* on the div with this class and everything deep inside it */
	pointer-events: none; /* none of those elements need to know about clicks */
	user-select: none; /* none of those elements have text the user should be able to select */
}
.myWillChangeTransform          { will-change: transform;           }
.myWillChangeBackgroundPosition { will-change: background-position; } /* with the styles, you get 2 layers in dev tools Layers */

.myDots {
	background-color: #fdfdfd; /* an off-white that still reads white but lets layers below peek through */
	background-image: radial-gradient(circle at center, #d0f0c0 6px, transparent 6px); /* a pastel green dot layer, centered in each cell */
	background-size: 60px 60px; /* make each “tile” big enough that the full 16px-diameter dot never hits an edge */
	background-position: 0 0, 30px 30px; /* offset the second layer by half a cell → diamond pattern */
}

</style>
