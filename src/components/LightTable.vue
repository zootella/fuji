<script setup>//./components/LightTable.vue - unified component for mvp[ersonal]

import {invoke} from '@tauri-apps/api/core'
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {xy, raf, blobToDataUrl, forwardize, backize, listSiblings, readAndRenderImage} from './library.js'//our javascript library

//                       _   
//   _____   _____ _ __ | |_ 
//  / _ \ \ / / _ \ '_ \| __|
// |  __/\ V /  __/ | | | |_ 
//  \___| \_/ \___|_| |_|\__|
//                           

onMounted(async () => {
	frameRef.value.addEventListener('wheel', onWheel, {passive: false})
	window.addEventListener('keydown', onKey)
	window.addEventListener('resize', onResize); onResize()//and call right away to look at the starting size
	const w = getCurrentWindow()
	unlistenFileDrop = await w.onDragDropEvent(async (event) => {
		if (event.payload.type == 'drop' && event.payload.paths.length) {
			let path = forwardize(event.payload.paths[0])
			await onDrop(path)
		}
	})
	onStart()
	dimensionStart()
	setText()
})
let unlistenFileDrop//will hold the unsubscribe function set above and called below
onBeforeUnmount(() => {
	frameRef.value.removeEventListener('wheel', onWheel)
	if (drag?.pointer && frameRef.value.hasPointerCapture(drag.pointer)) {
		frameRef.value.releasePointerCapture(drag.pointer)
		drag.pointer = null
	}
	window.removeEventListener('keydown', onKey)
	window.removeEventListener('resize', onResize)
	if (unlistenFileDrop) unlistenFileDrop()
})

async function onKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//ignore keystrokes into a form field

	let Ctrl = e.ctrlKey || e.metaKey
	let Shift = e.shiftKey
	let key = e.key

	if      (key == 'f') { console.log('my key F') }
	else if (key == 'q') { console.log('my key Q') }
	else if (Ctrl && key == 's') { console.log('my key Ctrl+S')
		e.preventDefault()//tell the browser not to show the file save dialog box
	} else if (key == 'Escape') {
		await setFullscreen(false)//macos will also exit fullscreen, but this call doesn't mess anything up with that
	}
	else if (key == 'ArrowLeft')  {  }
	else if (key == 'ArrowRight') {  }
	else if (key == 'ArrowUp')    {  }
	else if (key == 'ArrowDown')  {  }
	else if (key == 'PageDown')   { flip(1)  }
	else if (key == 'PageUp')     { flip(-1) }
	else if (key == '+' || (key == '=' && (Ctrl || Shift))) { zoom(true)  }//control and the [=+] key in browsers zooms in
	else if (key == '-')                                    { zoom(false) }
	else if (key == '0' && Ctrl) {}//ttd august, browser convention to reset zoom to 100%, maybe same as fuji d
}
async function setFullscreen(set) { let w = getCurrentWindow(); let current = await w.isFullscreen()
	if (set != current) w.setFullscreen(set)
}
async function toggleFullscreen() { let w = getCurrentWindow(); let current = await w.isFullscreen()
	w.setFullscreen(!current)
}
async function onDoubleClick(e) { await toggleFullscreen() }

function onWheel(e) {
	e.preventDefault()//tell the browser not to scroll

	let s = `wheel ${e.deltaX} Δx, ${e.deltaY} Δy`
	if (e.ctrlKey)  s += ' +Ctrl'
	if (e.metaKey)  s += ' +Meta'//testing on mac with karabiner elements and the microsoft keyboard, always seeing meta, never ctrl
	if (e.shiftKey) s += ' +Shift'//with shift held on mac, delta y is 0 and x is positive or negative
	//console.log(s)//ttd august, need to test with trackpad | wheel mouse | clicky wheel mouse X microsoft | apple keyboard X windows | mac X normal settings | customized | karabiner elements, phew! lots to test there!!

	let ctrl = e.ctrlKey || e.metaKey
	let direction = e.deltaX > 0 || e.deltaY > 0
	if (ctrl) zoom(!direction); else flip(direction ? 1 : -1)
}

//                    
//  _ __   __ _ _ __  
// | '_ \ / _` | '_ \ 
// | |_) | (_| | | | |
// | .__/ \__,_|_| |_|
// |_|                

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

//      _         
//  ___(_)_______ 
// / __| |_  / _ \
// \__ \ |/ /  __/
// |___/_/___\___|
//                

function onResize() {//called once on mounted and whenever the viewport size changes
	console.log('on resize! ↘️')
}

const zoomStep = 1.25
function zoom(direction) {
	quiverA.zoom = direction ? quiverA.zoom * zoomStep : quiverA.zoom / zoomStep
	quiver()//or, should zoom keep the part of the card that's in the center of the frame in the center? or, should it keep the part of the card that's under the pointer still? this is ok for now but you realize there are at least three ways to do this
}

function onPointerMove(e) { if (!drag) return
	let current = xy(e.clientX, e.clientY)//the new current location of the pointer
	let segment = xy(current, '-', drag.start)//the segment it just moved to get to where it is now
	drag.start = current//get ready for the next drag segment

	quiverA.space = xy(quiverA.space, '+', segment)
	quiver()
}

//the way this works is, change arrows in quiver a, then call quiver(); keep everything in quiver a; don't touch quiver b or c
const quiverA = {}//Quiver A: {x, y} arrows, dimensions, and zoom that completely describe where everything should appear
function dimensionStart() {

	quiverA.diamond = screen.width + screen.height//full size diamond half permineter
	quiverA.zoom = 0.5//zoom factor for the diamond permimiter
	quiverA.space = xy(xy(frameRef.value.clientWidth, frameRef.value.clientHeight), '/', 2)//frame corner to space center
	quiverA.natural = xy(64, 64)//natural image pixel width and height from its own file data
	quiverA.tile = xy(60, 60)//tiled background
	quiver()
}
function quiver() {

	//from quiver a arrows about what we want to show, calculate quiver b arrows which are styles for the page
	let quiverB = {}//Quiver B: a new set of page style dimensions calculated entirely from the current contents of quiver a
	quiverB.space = quiverA.space
	quiverB.tile  = quiverA.tile//these arrows are the same, but copy them from a to b as b is our complete record of page styles
	quiverB.card2 = xy(xy(quiverA.natural, '*', (quiverA.zoom * quiverA.diamond)), '/', (quiverA.natural.x + quiverA.natural.y))//diagonal across card top left to bottom right; math by Ramiel, No. 5
	quiverB.card1 = xy(quiverA.space, '+', xy(quiverB.card2, '/', -2))//from that, frame corner to card corner

	//are there any changes we need to set on the page?
	let same = false
	if (quiverC) {
		same = (
			xy(quiverC.space, '==', quiverB.space) &&
			xy(quiverC.tile,  '==', quiverB.tile)  &&
			xy(quiverC.card1, '==', quiverB.card1) &&
			xy(quiverC.card2, '==', quiverB.card2)
		)
	}
	if (!same) {//only bother the page if necessary

		frameRef.value.style.backgroundPosition = `${quiverB.space.x % quiverB.tile.x}px ${quiverB.space.y % quiverB.tile.y}px`
		cardRef.value.style.transform = `translate(${quiverB.card1.x}px, ${quiverB.card1.y}px)`
		cardRef.value.style.width = quiverB.card2.x+'px'
		cardRef.value.style.height = quiverB.card2.y+'px'
		//ttd august, could isolate width and height to only run if necessary; won't run during a pan, for instance
	}

	//keep a record of what we told the page to only bother it next time it's necessary
	quiverC = quiverB
}
let quiverC//Quiver C: our record of how we've styled the page to appear; treat as private to above

//   __ _ _       
//  / _| (_)_ __  
// | |_| | | '_ \ 
// |  _| | | |_) |
// |_| |_|_| .__/ 
//         |_|    

function onStart() {
	console.log('⭕ on start - once on startup, component loaded')
}
async function onDrop(path) {
	console.log(`⭕ on dropped path "${path}" - load and show right away`)

	triad.here.imgRef.value.style.display = 'none'//hide the image we're on; this is blinkey but ok for a drop, ttd august
	folder = await listSiblings(path)//list all the images in the same folder as path
	triad.here = fillImage(img8Ref, folder.index,     folder.list)//path dropped in

	await triad.here.promise
	await raf()
	quiverA.natural = triad.here.details?.natural || xy(64, 64); quiver()//position and size the card for the aspect ratio ahead
	triad.here.imgRef.value.style.display = 'block'//show the image now that it's ready
	await raf()

	triad.prev = fillImage(img7Ref, folder.index - 1, folder.list)//path alphebetically above
	triad.next = fillImage(img9Ref, folder.index + 1, folder.list)//path alphebetically below
}

let flipQueue = Promise.resolve()//do one flip at a time; start with resolved promise
async function flip(direction) {
	flipQueue = (flipQueue//queue this flip to run after any pending flips
		.then(() => _flip(direction))
		.catch(e => console.error('Flip error:', e))  // Don't break the chain
	)
	return flipQueue
}
async function _flip(direction) {
	if (!folder) return//nothing loaded yet

	let indexAhead1 = folder.index + direction//index where the user wants us to flip to
	let indexAhead2 = folder.index + direction + direction//the next next one, the one beyond that
	if (indexAhead1 < 0 || indexAhead1 >= folder.list.length) { console.log('❌ cannot flip off edge, ignoring command to flip'); return }
	console.log(`⭕ on command to flip ${direction > 0 ? 'forward' : 'back'} - flip immediately if ready, or upon loaded`)

	let behind, upon, ahead//from direction, pick the image functions which are ahead, we'll flip to, and behind, we'll discard and reuse
	if (direction > 0) {behind = 'prev', upon = 'here', ahead = 'next'}//flip forward, so next is ahead
	else               {behind = 'next', upon = 'here', ahead = 'prev'}//flip backwards, so prev is where we're going

	await triad[ahead].promise//delay this flip until the image we're about to show is rendered
	await raf()//🥪 wait for clean frame boundary

	//change page
	triad[upon].imgRef.value.style.display = 'none'//hide the image we're upon
	quiverA.natural = triad[ahead].details?.natural || xy(64, 64); quiver()//position and size the card for the aspect ratio ahead
	triad[ahead].imgRef.value.style.display = 'block'//show the image that's ahead

	//change state
	folder.index = indexAhead1//move our index in the folder image listing
	let [wasBehind, wasUpon, wasAhead] = [triad[behind], triad[upon], triad[ahead]]//rotate the triad forward
	triad[behind] = wasUpon; triad[upon] = wasAhead; triad[ahead] = wasBehind

	await raf()//🥪 wait for above paint to hit the screen

	triad[ahead] = fillImage(triad[ahead].imgRef, indexAhead2, folder.list)//preload the next next image, but don't wait for it
}
function fillImage(imgRef, index, list) {//start loading the image on the disk at list[index] into the given img7Ref, img8Ref, or img9Ref
	let image = {imgRef, path: null, promise: Promise.resolve(), error: null, details: null}//wrap the given imgRef into an object to set in the triad
	if (index < 0 || index >= list.length) return image//no path; mark this spot intentionally left blank

	image.path = list[index]//we do have a path, load the image there into the given imgRef.value
	image.promise = readAndRenderImage(imgRef.value, image.path)
		.then(details => {//await image.promise to wait for it to finish
			image.details = details//once image.promise is resolved, you can get details about the image here
			return details
		})
		.catch(error => {
			imgRef.value.src = errorData
			image.error = error
			return error
		})
	return image//return the image object to await image.promise and then check out image.details or image.error
}

//  _              
// | |_ __ _  __ _ 
// | __/ _` |/ _` |
// | || (_| | (_| |
//  \__\__,_|\__, |
//           |___/ 

//dashed box with center X as a visual placeholder for an image we couldn't render
const errorData = `data:image/svg+xml;base64,${btoa(`
	<svg width="300" height="300" xmlns="http://www.w3.org/2000/svg">
		<rect width="300" height="300" fill="none" stroke="#444" stroke-width="1" stroke-dasharray="2,1"/>
		<line x1="140" y1="140" x2="160" y2="160" stroke="#444" stroke-width="1"/>
		<line x1="160" y1="140" x2="140" y2="160" stroke="#444" stroke-width="1"/>
	</svg>
`)}`

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window full screen
const cardRef = ref(null)//a rectangle in space the user can drag to pan around, anywhere including far outside the frame viewport

const hud1Ref = ref('')
const hud2Ref = ref('')
const hud3Ref = ref('')
const hud4Ref = ref('')
const captionRef = ref('')
function setText() {
/*
hud1Ref.value = 'upper left'
hud2Ref.value = 'System operating according to normal parameters'
hud3Ref.value = `lower left, this one is longer
and could involve a second line of text, which could be quite long; text that grows to the width of the frame does wrap
or even a third line`
hud4Ref.value = `middle of frame
this HUD will likely be a card showing the user all the
keyboard shortcuts the app supports, and be really easy to
show and hide, such as by pressing the [H]elp or just [Spacebar]
and here is yet another line`
*/
captionRef.value = `A multimedia file manager designed
with privacy and precision in mind`//no terminating newline, if that matters
}

const img7Ref = ref(null)
const img8Ref = ref(null)
const img9Ref = ref(null)//our template contains these three img tags
let folder//set on drop, holds listing.list of images in current folder, and listing.index of the image we're on
const triad = {
	prev: {imgRef: img7Ref, path: null, promise: Promise.resolve(), details: null},
	here: {imgRef: img8Ref, path: null, promise: Promise.resolve(), details: null},
	next: {imgRef: img9Ref, path: null, promise: Promise.resolve(), details: null},
}

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

	<!-- Card: rectangular image container; drag to pan around in infinite space; caption text is within card but positioned below card -->
	<div
		ref="cardRef"
		class="myCard myShadow myDry myWillChangeTransform bg-neutral-950 border border-black"
	>

		<!-- three img tags for current (shown), previous (cached), and next (preloaded) -->
		<img ref="img7Ref" class="myImage" />
		<img ref="img8Ref" class="myImage" />
		<img ref="img9Ref" class="myImage" />

		<!-- caption lives inside the card, but sits below its border -->
		<div class="absolute bottom-0 translate-y-full py-2 whitespace-nowrap font-mono myEmbossed">{{captionRef}}</div>

	</div>

	<!-- HUD, inside the frame, next to the card -->
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
.myFrame {}
.myCard {} /* not using these yet, but they're here */

.myImage {
	position: absolute; /* position outside the normal document flow; note the card should not be positioned absolute! */
	top: 0; left: 0; width: 100%; height: 100%;
	object-fit: fill; /* stretch to all four edges; script will set the aspect ratio of the card to match the image's natural dimensions */
	display: none; /* start all three hidden; script will show one image tag from the triad at a time */
}

.myDry, .myDry * { /* on the div with this class and everything deep inside it */
	pointer-events: none; /* none of those elements need to know about clicks */
	user-select: none; /* none of those elements have text the user should be able to select */
}
.myWillChangeTransform          { will-change: transform;           }
.myWillChangeBackgroundPosition { will-change: background-position; } /* with the styles, you get 2 layers in dev tools Layers */

.myDots {
	background-color: #171717; /* an off-white that still reads white but lets layers below peek through */
	background-image: radial-gradient(circle at center, #262626 6px, transparent 6px); /* a pastel-pink dot layer, centered in each cell */
	background-size: 60px 60px; /* make each “tile” big enough that the full 16px-diameter dot never hits an edge */
	background-position: 0 0, 30px 30px; /* offset the second layer by half a cell → diamond pattern */
}
.myShadow {
	box-shadow: 6px 6px 12px rgba(0,0,0,0.5);
}
.myEmbossed {
	white-space: pre; /* honor \n and overflow the container */
	color: #525252;
	text-shadow:
		-1px -1px 2px black,
		1px 1px 2px black,
		0 0 8px black;
}

</style>
