<script setup>//one image sized to an invisible diamond, on an infinite plane that pan and zoom move

import {getCurrentWindow} from '@tauri-apps/api/window'
import {getCurrentWebview} from '@tauri-apps/api/webview'

import {ref, onBeforeUnmount} from 'vue'
import {
xy, raf, errorImageData,
screenToViewport, sayGroupDigits, saySize4,
} from './library.js'//our javascript library
import {modelList, modelOpen, modelIndex, modelStand} from '../model.js'//the folder, the order it is in, and where the user is; no view owns any of it
import {flipCacheWindow, flipCacheImage, flipCacheClose} from '../flipCache.js'//which images this table keeps, and the store beneath it
import {cacheFootprint} from '../cache.js'//for the hud line saying what the store is holding
import {log, logFlip, logTrouble} from '../log.js'//the log, which writes a file instead of painting a number; the shell starts it, this only adds rows
import {settings, settingsChanged} from '../settings.js'//fuji.toml, read by the shell before this view starts

//                       _   
//   _____   _____ _ __ | |_ 
//  / _ \ \ / / _ \ '_ \| __|
// |  __/\ V /  __/ | | | |_ 
//  \___| \_/ \___|_| |_|\__|
//                           

onBeforeUnmount(() => {
	frameRef.value.removeEventListener('wheel', onWheel)
	if (drag?.pointer && frameRef.value.hasPointerCapture(drag.pointer)) {
		frameRef.value.releasePointerCapture(drag.pointer)
		drag.pointer = null
	}
	flipCacheClose()//this table is going away, so the store should not still be holding images on its behalf
})

let started = false//start() comes every time this view is shown, and the setup below must happen once: running dimensionStart again would throw away the pan and zoom the user left
function start() {//the shell calls this when this view first comes on screen; measuring any earlier reads the hidden window's size, or nothing at all
	if (started) return
	started = true
	log('⭕ table: the shell has revealed the window and handed this view the screen')
	dimensionStart()
	hudStart()
	frameRef.value.addEventListener('wheel', onWheel, {passive: false})//on the frame, not the window, so a hidden table is handed nothing; and last, so no wheel can reach the quiver before dimensionStart has filled it
}
function isFullscreen() { return fullscreenNow }//the shell asks before recording a window, because a fullscreen one is not one the user placed

defineExpose({start, onKey, onResize, onDrop, isFullscreen})//everything the shell reaches for: window events belong to it, and it hands them to whichever view is showing

async function onKey(e) {
	let Ctrl = e.ctrlKey || e.metaKey
	let Shift = e.shiftKey
	let key = e.key

	if      (key == 'f') { log('⭕ table: key f, a branch with nothing behind it yet') }
	else if (key == 'q') { log('⭕ table: key q, a branch with nothing behind it yet') }
	else if (key == 'h') { toggleHelp()        }
	else if (key == 'i') { toggleInformation() }
	else if (Ctrl && key == 's') { log('⭕ table: key ctrl+s, a branch with nothing behind it yet')
		e.preventDefault()//tell the browser not to show the file save dialog box
	} else if (key == 'Escape') {
		await changeFullscreen(false)//in simple fullscreen, escape is entirely ours to handle; macos no longer intervenes
	}
	//the arrows, f, q, ctrl+s and ctrl+0 are stubs on purpose: the key map is decided and the behaviour is not, so the branches exist to be filled rather than rediscovered
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
async function onDoubleClick(e) { await toggleFullscreen() }
let fullscreenNow = false//our own record of where fullscreen is headed; we initiate every transition, and tauri's isFullscreen() doesn't report the simple mode
async function toggleFullscreen() { await changeFullscreen(!fullscreenNow) }
async function changeFullscreen(destination) {
	if (fullscreenNow == destination) return
	fullscreenNow = destination//record where we're headed before awaiting frames, so a request arriving mid-transition sees the destination and not the state we're leaving
	if (settings.fullscreen.curtain) {
		curtainUp()//black out the frame so the transition's in-between frames can't show the image out of place
		await raf(); await raf()//two frame boundaries: the first schedules the curtain's paint, the second confirms it reached the screen before the window changes beneath it
	}
	screenToViewport1 = await screenToViewport()
	//ttd august, this is pixel perfect now on mac and windows (but you haven't tested high res windows yet) to work around the shift-melt-blink render a black curtain over the frame, go full screen, get the resize event, do the pan, and then remove the curtain. this is a cool idea
	await getCurrentWindow().setSimpleFullscreen(destination)//simple fullscreen: instant, no macos space, no fade animation; on windows, identical to setFullscreen
	await getCurrentWebview().setFocus()//hand the keyboard back to the page; awaited after the line above so it lands second, and explained below
}

/*
Why fuji has to hand the keyboard back after a fullscreen change.

Simple fullscreen hides the titlebar by clearing the window's Titled style mask, and changing an NSWindow's style mask drops its first responder. tao knows this — util::toggle_style_mask carries the comment "If we don't do this, key handling will break" — and repairs it by calling makeFirstResponder with tao's own content view. That is the right view for a window tao draws into itself. Fuji's window has a WKWebView as a subview of that one, so the repair leaves the responder chain pointing a level above the web content: the mouse still works, because a click makes the view under it first responder, but no keystroke reaches JavaScript at all. The symptom is a fullscreen window that ignores the keyboard until the user clicks once.

Focusing the webview is wry's makeFirstResponder aimed a level lower, which is the same repair with the right argument. It needs core:webview:allow-set-webview-focus, which core:webview:default does not grant.

This belongs to the window rather than to this table, and moves to the shell whenever fullscreen does — otherwise the next table has to remember to carry a copy.
*/

const showCurtainRef = ref(false)//a black cover over the whole frame during fullscreen transitions; up before the window changes, down after the pan lands
let curtainTimer//started when the curtain goes up, cleared when it comes down normally
function curtainUp()   { showCurtainRef.value = true;  clearTimeout(curtainTimer); curtainTimer = setTimeout(curtainDown, 800) }//the timer means the curtain always falls, even if the resize event never arrives; a brief shear beats a stuck black window
function curtainDown() { showCurtainRef.value = false; clearTimeout(curtainTimer) }

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
		log('⭕ table: pointer down, right double click')
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

let screenToViewport1//arrow from screen corner to viewport corner before a change in to our out of full screen
async function onResize() {//called whenever the viewport size changes
	if (screenToViewport1) {//we've been waiting for this resize event to see where the viewport moved on the screen
		let stv2 = await screenToViewport()//where it is now, after the full screen change
		if (screenToViewport1 && stv2) dragSegment(xy(screenToViewport1, '-', stv2))
		screenToViewport1 = null//we don't need resize events generally
		if (showCurtainRef.value) {//the curtain is up, waiting on this pan
			await raf()//let the corrective pan reach the screen while the curtain still hides it
			curtainDown()
		}
	}
}

function zoom(direction) {
	let step = settings.zoom.step//how far one press of + or - moves, from the settings file
	quiverA.zoom = direction ? quiverA.zoom * step : quiverA.zoom / step
	quiver()
}

function onPointerMove(e) { if (!drag) return
	let current = xy(e.clientX, e.clientY)//the new current location of the pointer
	let segment = xy(current, '-', drag.start)//the segment it just moved to get to where it is now
	drag.start = current//get ready for the next drag segment
	dragSegment(segment)
}
function dragSegment(segment) {
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
	//ttd august, here's where, if quiverA says pixels are real, you should Math.round quiverB

	//only bother the page if necessary
	function same(name) { return quiverC && xy(quiverB[name], '==', quiverC[name]) }
	if (!(same('space') && same('tile'))) {
		frameRef.value.style.backgroundPosition = `${quiverB.space.x % quiverB.tile.x}px ${quiverB.space.y % quiverB.tile.y}px`
	}
	if (!same('card1')) {
		cardRef.value.style.transform = `translate(${quiverB.card1.x}px, ${quiverB.card1.y}px)`
	}
	if (!same('card2')) {
		cardRef.value.style.width  = quiverB.card2.x+'px'
		cardRef.value.style.height = quiverB.card2.y+'px'
	}

	//keep a record of what we told the page to only bother it next time it's necessary
	quiverC = quiverB

	updateInformation()
}
let quiverC//Quiver C: our record of how we've styled the page to appear; treat as private to above

//   __ _ _       
//  / _| (_)_ __  
// | |_| | | '_ \ 
// |  _| | | |_) |
// |_| |_|_| .__/ 
//         |_|    

async function onDrop(path) { return queue(() => _drop(path)) }//queued with the flips, because a drop replaces the very folder a flip in flight is holding an index into
async function _drop(path) {
	log(`⭕ table: dropped ${path}, loading and showing it right away`)

	await modelOpen(path)//the model lists the folder and puts it in the current order, and every other view is reading that list already
	if (modelIndex() < 0) { log('❌ table: no images in that folder, ignoring the drop'); return }
	flipCacheWindow(modelList.value, modelIndex())//ask for this image and its neighbours before showing anything, because showIndex wants what the window is holding
	//the card empties here rather than by a display none: sliding the window releases the old folder, and the store takes its element back out; this is blinkey but ok for a drop, ttd august
	await showIndex(modelIndex())
}

/*
A flip is three moments in a fixed order, and the order is the design rather than decoration. Wait for a clean frame boundary. Swap which element is shown and size the card to it. Wait again for that paint to reach the screen. Only then ask the store for anything new. The two waits carry a 🥪 so they are findable, and both are load-bearing: the first puts the swap inside a frame the engine is already about to render, and the second holds the flip open until the picture is actually on the glass.

The last moment is the one that gets lost. Asking for the next image is the obvious thing to do first, because it has the longest wait and every instinct says to start it early. That instinct is wrong here. Reading a file and decoding it both want the main thread, and the main thread is what fires animation frames, so a read started before the paint blocks the very frame it was meant to help. The image the user asked for is already decoded and waiting; making them stare at the old one for another three hundred milliseconds to get a head start on an image they have not asked for yet is a trade nobody would make on purpose.

Fuji made it anyway. The triad had this order, and one line of comment explaining it. When the triad became a store and a window, the rewrite moved the window slide to the top of the flip and kept the words without their meaning. Every flip was then a cache hit that took three hundred milliseconds, which is the worst shape a bug can take: the cache reported perfect behaviour while the app grew slower than the thing the cache replaced.

The log caught it, and only by accident. Each flip's paint and the next load's read came back as the same number, to the millisecond, again and again — 372 against 371, 366 against 366, 251 against 251. That is two clocks timing one interval, which is what a blocked frame looks like from outside. No test would have found it, because nothing was broken: no exception, no wrong picture, nothing to assert against, just a frame that took twenty times too long.

Reads are cheap now that disk_read hands its bytes over raw, but decodes still occupy the thread and always will, so the order still holds. The rule for anyone editing below, a later version of whoever wrote this included: show first, then ask. Nothing that can occupy the main thread goes before the paint, and a line that has to move, moves after the second 🥪.
*/
/*
Everything that changes what is on the card goes through one queue, and it is not only about flips arriving faster than they finish. Both _flip and _drop read the model's list, await, and then use what they read — so a drop landing inside a flip's await swaps the listing underneath it, and the flip goes on to show an image from the old folder at an index into the new one. Serialising them means each is the only thing reading the model for its whole run.
*/
let workQueue = Promise.resolve()//one change to the card at a time; start with a resolved promise
function queue(work) {
	workQueue = workQueue
		.then(work)
		.catch(error => logTrouble('table: changing what it shows', error))//report and carry on, so one failure does not stop every command after it
	return workQueue
}
async function flip(direction) { return queue(() => _flip(direction)) }
/*ttd august, as with slow big GIFs that should be MPEGs you've been able to mangle the triad
show Loading... upper right HUD immediately if the flip has to wait at all--if the promise is not already resolved
and when Loading... is shown, in that mode, ignore all additional commands
*/
async function _flip(direction) {
	if (!modelList.value.length) return//nothing loaded yet

	let ahead = modelIndex() + direction//index where the user wants us to flip to
	if (ahead < 0 || ahead >= modelList.value.length) { log('❌ table: cannot flip off the edge, ignoring the command'); return }

	let began = performance.now()//the wall clock from the command to pixels on the screen
	await showIndex(ahead)//no need to ask the store for anything first: a flip moves one step and the window already reaches one step, so the image ahead is held before the command arrives
	let painted = await raf()//🥪 wait for above paint to hit the screen, which is also the honest end of the flip
	flipMs = Math.round(painted - began)
	flipFrames = Math.ceil(flipMs / frameMs)//rounded up, so a flip that spilled a millisecond into a second frame does not get to claim it took one; converted rather than counted, because a blocked main thread fires no animation frames at all and counting callbacks would report one frame for a stall that dropped twelve
	paintMs = Math.round(painted - shownAt)//the half of the flip that is the engine putting an image the store says is ready onto the screen
	updateInformation()
	learnFrameMs(painted)//deliberately not awaited: the flip is over, and the queue behind it must not wait on a measurement
	logFlip({//before the window slides, so nothing the log does can land inside what it just measured
		sequence: ++flipSequence, index: ahead, direction: direction > 0 ? 'fwd' : 'back', hit: storeHit ? 'hit' : 'miss',
		store: storeMs, paint: paintMs, flip: flipMs, frames: flipFrames, path: modelList.value[ahead],
	})

	flipCacheWindow(modelList.value, ahead)//last of all, and this order is the whole point: a read started before the paint blocks the very frame it was meant to help, which is what the triad's "wait for above paint to hit the screen" was protecting and what this file lost when it stopped being a triad
}
let flipSequence = 0//so the log reads in the order the user flipped
let flipMs = 0, flipFrames = 0//what the last flip cost end to end
let storeHit = false//whether the image was already decoded when the flip asked for it
let storeMs = 0, paintMs = 0, shownAt = 0//the flip split in two, and the two halves have nothing to do with each other: a large storeMs means the window did not reach this image in time, and a large paintMs on a hit means the engine dropped the pixels while the image was hidden and is rebuilding them. performance.md has what that second one has already cost
let frameMs = 1000//narrowed toward this display's real frame time by the flips above
async function learnFrameMs(painted) {//one more frame boundary after the flip has let go of the queue, because an interval needs two timestamps and the flip itself can only afford one
	let next = await raf()
	if (next - painted > 1 && next - painted < frameMs) frameMs = next - painted//the shortest gap ever seen between two frames is this display's rate, learned rather than assumed, so a 60hz dell and a 120hz panel each read correctly
}

async function showIndex(index) {//put the image at index on the card, and record where we are
	let asked = performance.now()
	let path = modelList.value[index]
	let entry = await flipCacheImage(path)//already decoded if the window reached it in time; otherwise this is the wait
	storeMs = Math.round(performance.now() - asked)
	storeHit = entry.rendered > 0 && entry.rendered <= asked//decoded before this flip asked, which is the only thing that makes a window worth keeping
	await raf()//🥪 wait for clean frame boundary

	modelStand(path)//the model keeps the path rather than the index, so a change of sort leaves the user on this picture
	here = entry
	cardShow(entry.error ? errorRef.value : entry.img)//the store's own element goes on the card rather than one of ours pointed at the same picture, which would pay the whole decode again
	quiverA.natural = entry.error ? xy(64, 64) : xy(entry.img.naturalWidth, entry.img.naturalHeight); quiver()//position and size the card for the aspect ratio we are showing, and quiver updates the hud on its way out
	shownAt = performance.now()//the swap is done, and the next frame boundary is the paint
}
let showing = null//the element the card is showing right now, which is the store's and not ours
function cardShow(img) {//the one place an image becomes visible
	if (img.parentNode != cardRef.value) { img.className = 'myImage'; cardRef.value.insertBefore(img, cardRef.value.firstChild) }//adopted on first showing; the store takes it back out when it lets the image go
	if (showing && showing != img) showing.style.display = 'none'
	img.style.display = 'block'
	showing = img
}

//  _               _ 
// | |__  _   _  __| |
// | '_ \| | | |/ _` |
// | | | | |_| | (_| |
// |_| |_|\__,_|\__,_|
//                    

const showHud2Ref    = ref(false); const hud2Ref    = ref('')//upper right, empty and unshown: reserved for the Loading the ttd above _flip asks for
const showHud3Ref    = ref(false); const hud3Ref    = ref('')//bottom, information; starts hidden so one the user turned off never flashes up before hudStart reads the setting
const showHud4Ref    = ref(false); const hud4Ref    = ref('')//middle, help
const showCaptionRef = ref(false); const captionRef = ref('')//caption, below card on table; hidden to start for the same reason
function hudStart() {

hud3Ref.value = ``
hud4Ref.value = `middle of frame
this HUD will likely be a card showing the user all the
keyboard shortcuts the app supports, and be really easy to
show and hide, such as by pressing the [H]elp or just [Spacebar]
and here is yet another line`

captionRef.value = `A multimedia file manager designed
with privacy and precision in mind`//placeholder text, set once: the caption is meant to carry the image's path and natural size, and nothing updates it on a flip yet

	showHud3Ref.value    = settings.hud.information//where these two start; [i] toggles this one from there, and no key toggles the caption yet
	showCaptionRef.value = settings.hud.caption

	updateInformation()
}
function toggleInformation() {
	showHud3Ref.value = !showHud3Ref.value
	updateInformation()//it built nothing while it was hidden, so fill it now rather than showing whatever it last said
	settings.hud.information = showHud3Ref.value; settingsChanged()//the setting records where the user left this hud, not just where it started
}
function toggleHelp()        { showHud4Ref.value = !showHud4Ref.value }
function updateInformation() {
	if (!showHud3Ref.value) return//a hidden hud builds no string and touches no ref, so measuring with it off measures fuji rather than fuji plus a readout
	let s = 'no image loaded'
	if (here?.error) s = `${here.path}\ncould not be shown: ${here.error}`//the card is showing the error placeholder, so name the file and what it said rather than claiming nothing is loaded
	else if (here?.img && quiverC?.card2) {
		let f = cacheFootprint()//the store's running totals, free to read because they are kept rather than walked
s = `${here.path}
natural ${here.img.naturalWidth} width x ${here.img.naturalHeight} height, ${saySize4(here.blobBytes)} (${sayGroupDigits(here.blobBytes)} bytes)
displayed ${Math.round(quiverC.card2.x)} width x ${Math.round(quiverC.card2.y)} height (CSS, not physical, pixels)
${Math.round(here.loaded - here.requested)}ms disk + ${Math.round(here.rendered - here.loaded)}ms render, to load this one
flip ${flipMs}ms (${flipFrames} frames) = ${storeMs}ms store + ${paintMs}ms paint
cache ${f.count} images, ${saySize4(f.blobs)} of files + ${saySize4(f.pixels)} of pixels`
	}
	hud3Ref.value = s
}

//  _              
// | |_ __ _  __ _ 
// | __/ _` |/ _` |
// | || (_| | (_| |
//  \__\__,_|\__, |
//           |___/ 

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window full screen
const cardRef = ref(null)//a rectangle in space the user can drag to pan around, anywhere including far outside the frame viewport

const errorRef = ref(null)//the one img tag the template still owns, shown in place of a picture fuji could not read
let here = null//the store's entry for the image on the card, which is where the hud reads everything about it

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

		<!-- the images the card shows are the store's own elements, put here by cardShow; this one is only for a file fuji could not read -->
		<img ref="errorRef" class="myImage" :src="errorImageData" />

		<!-- caption lives inside the card, but sits below its border -->
		<div v-if="showCaptionRef" class="absolute bottom-0 translate-y-full py-2 whitespace-nowrap font-mono myEmbossed">{{captionRef}}</div>

	</div>

	<!-- HUD, inside the frame, next to the card -->
	<div v-if="showHud2Ref" class="myHud myDry absolute top-4 right-4">{{hud2Ref}}</div>
	<div v-if="showHud3Ref" class="myHud myDry absolute bottom-0 inset-x-0">{{hud3Ref}}</div>
	<div v-if="showHud4Ref" class="myHud myDry absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">{{hud4Ref}}</div>

	<!-- curtain, last so it covers everything: blacks out the frame during fullscreen transitions -->
	<div v-if="showCurtainRef" class="myDry absolute inset-0 bg-black"></div>

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
	display: none; /* every image starts hidden; cardShow shows one at a time */
}

.myDry, .myDry * { /* on the div with this class and everything deep inside it */
	pointer-events: none; /* none of those elements need to know about clicks */
	user-select: none; /* none of those elements have text the user should be able to select */
}
.myWillChangeTransform          { will-change: transform;           }
.myWillChangeBackgroundPosition { will-change: background-position; } /* with the styles, you get 2 layers in dev tools Layers */

.myDots {
	background-color: #171717;
	background-image: radial-gradient(circle at center, #262626 6px, transparent 6px);
	background-size: 60px 60px;
	background-position: 0 0, 30px 30px;
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
