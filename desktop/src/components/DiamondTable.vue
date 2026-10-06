<script setup>//one image sized to an invisible diamond, on an infinite plane that pan and zoom move

import {ref, watch, onBeforeUnmount} from 'vue'
import {
xy, xySnap, raf, errorImageData,
sayGroupDigits, saySize4, sayDay, sayDimensions, middleDot, backize,
} from './library.js'//our javascript library
import {modelList, modelPath, modelOpen, modelIndex, modelStand, modelFile} from '../model.js'//the folder, the order it is in, and where the user is; no view owns any of it
import {flipCacheWindow, flipCacheImage, flipCacheClose} from '../flipCache.js'//which images this table keeps, and the store beneath it
import {cacheFootprint} from '../cache.js'//for the hud line saying what the store is holding
import {log, logFlip, logTrouble} from '../log.js'//the log, which writes a file instead of painting a number; the shell starts it, this only adds rows
import {settings, settingsChanged} from '../settings.js'//fuji.toml, read by the shell before this view starts
import {gamma, gammaStep, gammaDrag} from '../gamma.js'//the lens the shell draws every picture through, which shift with the wheel or a right drag here sets

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

const emit = defineEmits(['sheet'])//a double-click, for the shell to show the contact sheet; which view is showing, and the fullscreen that goes with it, are the shell's, so this only asks

let started = false//start() comes every time this view is shown, and the setup below must happen once: running dimensionStart again would throw away the pan and zoom the user left
function start() {//the shell calls this every time this view comes on screen, already fullscreen, so the frame measured here is the one the table keeps
	if (!started) {
		started = true
		log('table: started')
		dimensionStart()
		hudStart()
		frameRef.value.addEventListener('wheel', onWheel, {passive: false})//on the frame, not the window, so a hidden table is handed nothing; and last, so no wheel can reach the quiver before dimensionStart has filled it
	}
	if (modelPath.value && modelPath.value != here?.path) {//the sheet opened a folder or a thumbnail was double-clicked while this table was away, so show where the model is standing
		cardRef.value.style.visibility = 'hidden'//the card still holds the picture this table last showed, which the user has moved on from; the dots show until the new one is on it, however long its decode takes
		queue(async () => { try { await _showModel() } finally { cardRef.value.style.visibility = '' } })
	}
}

defineExpose({start, onKey, onDrop, cardAt})//everything the shell reaches for: window events belong to it, and it hands them to whichever view is showing. cardAt is for a preview handing its picture over, so the card starts where the preview's picture stood

async function onKey(e) {
	let Ctrl = e.ctrlKey || e.metaKey
	let key = e.key

	//q, ctrl+s and ctrl+0 are stubs on purpose: the key map is decided and the behaviour is not, so the branches exist to be filled rather than rediscovered
	//a letter, a digit or space acts only with control and command up, so a chord like command w or command q on the mac, on its way to the menu, does not act on the picture first; the shell has already dropped anything with alt
	if      (!Ctrl && key == 'q') { log('table: q does nothing yet') }
	else if (!Ctrl && key == 'i') { toggleInformation() }
	else if (Ctrl && key == 's') { log('table: ctrl+s does nothing yet')
		e.preventDefault()//tell the browser not to show the file save dialog box
	}
	else if (Ctrl && (key == 'ArrowRight' || key == 'ArrowDown')) { flip(1)  }//control with any arrow flips, right or down for the next picture and left or up for the one before: a control of its own beside the page keys, and on a mac laptop, which has no page keys, the keyboard's only flip; ahead of the plain arrows below, which never ask about control. On the mac, control with an arrow is the system's own shortcut for moving between spaces and never arrives, so there it is command, which Ctrl above already reads
	else if (Ctrl && (key == 'ArrowLeft'  || key == 'ArrowUp'))   { flip(-1) }
	else if (key == 'ArrowLeft')  { panStep(xy(-1,  0)) }//the arrows pan one step that way, so a hand on the keyboard can get around without the mouse
	else if (key == 'ArrowRight') { panStep(xy( 1,  0)) }
	else if (key == 'ArrowUp')    { panStep(xy( 0, -1)) }
	else if (key == 'ArrowDown')  { panStep(xy( 0,  1)) }
	else if (key == 'PageDown')   { flip(1)  }
	else if (key == 'PageUp')     { flip(-1) }
	else if (key == '+' || key == '=')                      { zoomStep(true)  }//the [=+] key zooms in, unshifted or with control as in browsers, and so does the number pad's plus; shift with it is gamma, which the shell takes before this sees it
	else if (key == '-')                                    { zoomStep(false) }
	else if (!Ctrl && key == ' ')                           { dimensionFrame() }//spacebar sizes the diamond to the frame and centers the card in it
	else if (!Ctrl && key == 'f')                           { dimensionFit() }//the whole image inside the frame, one side meeting it exactly
	else if (!Ctrl && key == 'w')                           { dimensionWidth() }//the image's width meeting the frame's exactly, its height overflowing or falling short
	else if (!Ctrl && /^[1-6]$/.test(key)) { zoomNatural(Number(key)) }//the number keys 1 to 6, main row or number pad, which arrive as the same key: exactly that many css pixels per natural pixel. 7, 8 and 9 are left unused, since past 6x the other zooms serve
	else if (key == '0' && Ctrl) {}//ttd august, browser convention to reset zoom to 100%, maybe same as fuji d
}
function onWheel(e) {
	e.preventDefault()//tell the browser not to scroll

	let s = `wheel ${e.deltaX} Δx, ${e.deltaY} Δy`
	if (e.ctrlKey)  s += ' +Ctrl'
	if (e.metaKey)  s += ' +Meta'//testing on mac with karabiner elements and the microsoft keyboard, always seeing meta, never ctrl
	if (e.shiftKey) s += ' +Shift'//with shift held on mac, delta y is 0 and x is positive or negative
	//console.log(s)//ttd august, need to test with trackpad | wheel mouse | clicky wheel mouse X microsoft | apple keyboard X windows | mac X normal settings | customized | karabiner elements, phew! lots to test there!!

	let ctrl = e.ctrlKey || e.metaKey
	let direction = e.deltaX > 0 || e.deltaY > 0
	if (ctrl) zoomStep(!direction)
	else if (e.shiftKey) gammaStep(direction ? -settings.gamma.wheel : settings.gamma.wheel)//away from you brightens, the way ctrl zooms in; the mac reports a shift wheel as sideways, which direction already reads. One step an event, so a trackpad's stream of small events races, the same as the zoom does and the ttd above covers
	else flip(direction ? 1 : -1)
}

//                    
//  _ __   __ _ _ __  
// | '_ \ / _` | '_ \ 
// | |_) | (_| | | | |
// | .__/ \__,_|_| |_|
// |_|                

function onDoubleClick() { emit('sheet') }//back to the contact sheet, which is the way from a picture to the folder around it

function onPointerDown(e) {
	if (e.button == 0 && e.detail == 2 && e.buttons == 1) {//primary button 0, 2nd quick click, first bit value 1 only button down right now
		//ignoring this because listening for browser double click event
	} else if (e.button == 2 && e.detail == 2 && e.buttons == 2) {//secondary button 2, 2nd quick click, second bit value 2 only button down right now
		log('table: pointer down, right double click')
	} else {
		dragStart(e)
	}
}
let drag//an object of positions and ids during a left or right click drag
function dragStart(e) {
	drag = {
		button: e.button,//0 primary or 2 secondary mouse button
		anchor:   xy(e.clientX, e.clientY),//viewport corner to where the button went down, which a right drag zooms about; a pointer position used as a point in the frame, which the essay below says is fine
		previous: xy(e.clientX, e.clientY),//viewport corner to where the pointer was last seen, for the segments a left drag pans by
		diamond: quiverA.diamond, space: quiverA.space,//the diamond as the drag found it, which a right drag sets the zoom from. space is the arrow itself rather than a copy, which is safe only because no code ever changes an arrow in place: xy() always makes a new one
		shift: e.shiftKey, gamma: gamma.value,//a right drag with shift held sets the gamma instead of the zoom, from the gamma it found; read once here, so letting go of shift partway through does not swap one for the other
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

function zoom(diamond, anchor) {//set the diamond's width plus height, holding the plane still at anchor, frame corner to the point that must not move: every arrow from the anchor scales by the same ratio, the card's size and the arrow from the anchor to the diamond's center alike
	let k = diamond / quiverA.diamond//the ratio everything grows by
	quiverA.diamond = diamond//set rather than multiplied, so a caller that computed an exact size gets exactly that size
	quiverA.space = xy(anchor, '+', xy(xy(quiverA.space, '-', anchor), '*', k))//anchor to diamond center, scaled, and put back on the anchor
	quiver()
}
function zoomStep(direction) {//the keys and the wheel: one step in or out about the frame's center, so an image centered there stays put, and one off center drifts further out on the way in and back toward the center on the way out, which means zooming out always brings a lost image home
	zoom(quiverA.diamond * (direction ? settings.zoom.step : 1 / settings.zoom.step), xy(frameSize(), '/', 2))
}
function zoomNatural(n) {//the number keys: the card at exactly n css pixels per natural pixel, so the img stretches each source pixel across an n by n block, smoothed as ever. The math runs the other way here, the card first and the diamond around it: the card is natural times n, and the diamond is that card's width plus height, which quiver() divides back out exactly. About the frame's center, like the step keys
	zoom(n * (quiverA.natural.x + quiverA.natural.y), xy(frameSize(), '/', 2))
}

function onPointerMove(e) { if (!drag) return
	let current = xy(e.clientX, e.clientY)//viewport corner to the pointer now
	if (drag.button == 2 && drag.shift) {//with shift, the same drag is a slider laid up the frame: only its height counts, from where it began, and gamma.js says how far it goes
		gammaDrag(drag.gamma, drag.anchor.y - current.y, frameSize().y)
	} else if (drag.button == 2) {//the secondary button zooms about where it went down, and the height of the drag sets the zoom: up is in, down is out, sideways is nothing
		let height = drag.anchor.y - current.y//how far above where the button went down the pointer is now, negative when below
		quiverA.diamond = drag.diamond; quiverA.space = drag.space//put the diamond back as the drag found it, so the height sets the zoom rather than nudging it
		zoom(drag.diamond * 2 ** (height / settings.zoom.drag), drag.anchor)//and scale from there by the whole height, zoom.drag pixels to a doubling
	} else {//the primary button pans by the segment since the last move
		dragSegment(xy(current, '-', drag.previous))//from where the pointer was last seen to where it is now
		drag.previous = current//the next segment starts here
	}
}
function panStep(way) {//an arrow key: pan one step, instantly, pan.step of the frame's shorter side, so a step is the same share of any screen. way is a unit arrow pointing the way the key points, and the sign of pan.step says whether the picture or the view moves that way
	let frame = frameSize()
	dragSegment(xy(way, '*', settings.pan.step * Math.min(frame.x, frame.y)))
}
function dragSegment(segment) {
	quiverA.space = xy(quiverA.space, '+', segment)//a segment is a difference and carries no origin, so it adds straight onto space
	quiver()
}

/*
The arrows on this table, each from a named point to a named point.

Every arrow is an {x, y} pair made by xy(), in CSS pixels, with x to the right and y downward. Naming both ends is the whole discipline: a width and a height say where an arrow points, and it means nothing until you also know where it points from. The frame is the rectangle the user sees the table through, this component's outer div, and its top left corner is where most arrows start. frameSize() is frame corner to the frame's bottom right corner, measured when it is asked for because the window changes size.

space is frame corner to the center of the infinite plane. The card is always centered on that point. Panning moves space by the segment dragged, and zooming moves it too, scaling the arrow from the zoom's anchor to it by the same factor as the diamond, so the anchor is the point a zoom holds still: the frame's center for the step keys, the wheel and the number keys, and where the button went down for a right drag. card2 is the card's top left corner to its bottom right corner, which is its size. card1 is frame corner to the card's top left corner: space less half of card2. natural is the image's top left corner to its bottom right corner in the image's own pixels rather than CSS pixels, and it enters the math only in CSS pixels per natural pixel, so its unit never reaches the page.

One thing is a number rather than an arrow. diamond is the card's width plus height right now, in CSS pixels, which is the diagonal of the invisible diamond every card fits, vertex to vertex. Every zoom sets it and nothing else, and the card's size follows from it and the image's aspect. A number key, f and w run that the other way, computing the card first, at a whole number of CSS pixels per natural pixel, fitted inside the frame, or fitted to its width, and setting diamond to its width plus height, which the division in quiver() returns.

A pan is made of segments. The pointer events report positions from the viewport corner: previous is where the pointer was last seen, current is where it is now, and a segment is current less previous. A segment is a difference, so it has no origin of its own and adds straight onto space although space starts at the frame corner. An arrow key makes a segment of its own, pan.step of the frame's shorter side, and the sign of pan.step says which way: negative moves the view the way the key points, so the picture slides the other way. A right drag zooms instead of panning: anchor is where the button went down, and the height of the pointer above it sets the zoom, the diamond the drag began with times two to the power of that height over zoom.drag, with the diamond scaled about the anchor from where the drag found it. So the plane holds still under the point where the drag began, and a drag that comes back to it restores what it had. That anchor is a pointer position used as a point in the frame, and a position does care about its origin. It works because the frame fills the window, so the frame corner and the viewport corner are one point; a table with a sidebar would have to subtract the frame's own position first.

Quiver A is real numbers and quiver B is pixels: every arrow B hands the page is snapped to a whole backing pixel, so two histories that agree to within one draw the identical picture. Nothing in A is ever rounded, and nothing on the page is ever read back into A, so there is no path by which error accumulates. The essay above quiver() says which grid and why.
*/
//the way this works is, change arrows in quiver a, then call quiver(); keep everything in quiver a; don't touch quiver b or c
const quiverA = {}//Quiver A: {x, y} arrows and the diamond's size that completely describe where everything should appear
function dimensionStart() {

	quiverA.diamond = (screen.width + screen.height) / 2//the card's width plus height, in css pixels, which is also the diamond's diagonal from vertex to vertex; half the screen's own sum to start, so fuji opens with an image shaped like the screen at half its size
	quiverA.space = xy(frameSize(), '/', 2)//frame corner to space center
	quiverA.natural = xy(64, 64)//natural image pixel width and height from its own file data
	quiver()
}
function frameSize() { return xy(frameRef.value.clientWidth, frameRef.value.clientHeight) }//frame corner to the frame's bottom right corner, measured now rather than kept, because the window changes size and the frame with it
function dimensionFrame() {//spacebar: the diamond's width plus height becomes the frame's, and the card sits centered in the frame. An image shaped like the frame fills it exactly; any other overflows at the two ends of one axis by exactly the margin it leaves at the two ends of the other. Meant for fullscreen, where the frame is the screen
	let frame = frameSize()
	quiverA.space = xy(frame, '/', 2)//frame corner to the frame's center
	quiverA.diamond = frame.x + frame.y//the card's width plus height becomes the frame's
	quiver()
}
function dimensionFit() {//f: the card fits inside the frame, its width or its height meeting the frame's exactly and the other side shorter by the image's aspect, centered. The card first and the diamond around it, like a number key; the rounding in quiver() is what lands the meeting side on the frame's edge to the pixel
	let frame = frameSize()
	let scale = Math.min(frame.x / quiverA.natural.x, frame.y / quiverA.natural.y)//css pixels per natural pixel that brings the tighter side to the frame's edge
	quiverA.space = xy(frame, '/', 2)//frame corner to the frame's center
	quiverA.diamond = scale * (quiverA.natural.x + quiverA.natural.y)//the fitted card's width plus height
	quiver()
}
function cardAt(rect) {//the card exactly at this rectangle of the frame, which is how the shell hands over a preview's picture where it stood; the rectangle is the picture's own shape, so the diamond around it is its width plus height
	quiverA.space = xy(rect.x + rect.width / 2, rect.y + rect.height / 2)//frame corner to the rectangle's center
	quiverA.diamond = rect.width + rect.height
	quiver()
}
function dimensionWidth() {//w: the card's width meets the frame's exactly, centered, whatever that does to its height. A picture taller than the frame at that width overflows equally above and below, for the user to pan down it; a shorter one sits in the middle with dots above and below. Built the way f's is, the card first and the diamond around it
	let frame = frameSize()
	let scale = frame.x / quiverA.natural.x//css pixels per natural pixel that brings the card's width to the frame's
	quiverA.space = xy(frame, '/', 2)//frame corner to the frame's center
	quiverA.diamond = scale * (quiverA.natural.x + quiverA.natural.y)//that card's width plus height
	quiver()
}
/*
Quiver B snaps to the backing grid, and why that is the right grid.

Quiver A is real numbers and is never rounded, so nothing drifts. Quiver B is what the page is told, and a position on a fraction of a pixel is drawn resampled, blurred by the fraction, so B snaps every arrow to a pixel before writing it. The question is which pixel. CSS pixels are the unit of layout, of the window and the frame, and of every number the user asks for, n per natural pixel or fit to the frame. Backing pixels are the unit of what is drawn. On a Mac devicePixelRatio is 1 or 2, so the backing grid contains the CSS grid: everything whole in CSS is whole in backing, and on a Retina display every half CSS pixel is a whole backing pixel as well. Snapping to CSS would throw those halves away for nothing, moving the card up to a backing pixel from where the math put it and leaving it unable to center in a frame with an odd side. Snapping to backing keeps every position the display can show, at the cost of one factor at this one gate.

It is safe because it never makes a fraction of a backing pixel, which is the only thing that can leave a sliver, one row half image and half dots. The number keys keep their exact CSS sizes, since a whole number is on both grids. The card f and w fit meets the frame's edge, since the frame's size is a whole number of CSS pixels and so of backing pixels. And a card the shell hands over at a preview's rectangle lands exactly, since that rectangle is whole CSS pixels.

The grids stop nesting at Windows scales like 150 percent, where a CSS pixel is one and a half backing pixels. Nothing is exact there under any rule: whole CSS pixels put edges on half backing pixels, and this rule puts them on backing pixels while a number key reads 33.333 rather than 33 in the inspector. Whether Chromium honors a fractional CSS size or snaps it back to whole is a thing only the Windows box can check, and the thumbnails page on fuji's site holds what that machine has measured so far. The table reads devicePixelRatio every time rather than keeping it, because a window can move to a display with a different one; B written for the old display stays on its grid until the next pan or zoom, and that is the whole of the gap.
*/
function quiver() {

	//from quiver a arrows about what we want to show, calculate quiver b arrows which are styles for the page
	let quiverB = {}//Quiver B: a new set of page style dimensions calculated entirely from the current contents of quiver a, and snapped to whole backing pixels on the way: a is real numbers and b is pixels, so the picture never depends on the fraction a zoom left behind, and nothing in a is ever rounded, so nothing drifts
	let backingPerCss = window.devicePixelRatio//read every time rather than kept; the one place another pixel unit enters the table, and the essay above says why
	let scale = quiverA.diamond / (quiverA.natural.x + quiverA.natural.y)//css pixels per natural pixel, so the card's width plus height comes out at the diamond; a whole number when a number key set the diamond, since n times the sum over the sum divides exactly
	quiverB.space = xySnap(quiverA.space, backingPerCss)//frame corner to the seam the card is centered on, on a backing pixel
	quiverB.card2 = xySnap(xy(quiverA.natural, '*', scale), backingPerCss)//card top left corner to card bottom right corner, which is its size; math by Ramiel, No. 5
	quiverB.card1 = xySnap(xy(quiverB.space, '-', xy(quiverB.card2, '/', 2)), backingPerCss)//frame corner to card top left corner: the center less half the size, and when a side is an odd number of backing pixels the extra one goes to one end, so the center sits half a backing pixel off the seam

	//only bother the page if necessary
	function same(name) { return quiverC && xy(quiverB[name], '==', quiverC[name]) }
	if (!same('space')) {
		frameRef.value.style.backgroundPosition = `${quiverB.space.x}px ${quiverB.space.y}px`//the dots ride along with the pan; the stylesheet repeats them every 60px, so the position needs no wrapping to that
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

	updateInformation(); updateCaption()
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
	log(`table: dropped ${path}`)

	await modelOpen(path)//the model lists the folder and puts it in the current order, and every other view is reading that list already
	await _showModel()
}
async function _showModel() {//show the picture the model is standing on, wherever it came from: a drop here, or a folder opened or a thumbnail chosen while the sheet was showing
	if (modelIndex() < 0) { log('table: no images in that folder'); return }
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
	if (ahead < 0 || ahead >= modelList.value.length) { log('table: at the edge, no flip'); return }

	let began = performance.now()//the wall clock from the command to pixels on the screen
	await showIndex(ahead)//no need to ask the store for anything first: a flip moves one step and the window already reaches one step, so the image ahead is held before the command arrives
	let painted = await raf()//🥪 wait for above paint to hit the screen, which is also the honest end of the flip
	flipMs = Math.round(painted - began)
	flipFrames = Math.ceil(flipMs / frameMs)//rounded up, so a flip that spilled a millisecond into a second frame does not get to claim it took one; converted rather than counted, because a blocked main thread fires no animation frames at all and counting callbacks would report one frame for a stall that dropped twelve
	paintMs = Math.round(painted - shownAt)//the half of the flip that is the engine putting an image the store says is ready onto the screen
	updateInformation(); updateCaption()
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
let storeMs = 0, paintMs = 0, shownAt = 0//the flip split in two, and the two halves have nothing to do with each other: a large storeMs means the window did not reach this image in time, and a large paintMs on a hit means the engine dropped the pixels while the image was hidden and is rebuilding them
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
	if (img.parentNode != cardRef.value) { img.className = 'myImage'; cardRef.value.insertBefore(img, cardRef.value.firstChild) }//adopted on first showing; the store takes it back out when it lets the image go. The class only styles this element because the rule for it is written with :deep(), as this element is not the template's
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
const showCaptionRef = ref(false); const captionRef = ref('')//caption, below card on table; hidden to start for the same reason
function hudStart() {

hud3Ref.value = ``

	showHud3Ref.value    = settings.hud.information//where these two start; [i] toggles this one from there, and no key toggles the caption yet
	showCaptionRef.value = settings.hud.caption

	updateInformation(); updateCaption()
}
function toggleInformation() {
	showHud3Ref.value = !showHud3Ref.value
	updateInformation()//it built nothing while it was hidden, so fill it now rather than showing whatever it last said
	settings.hud.information = showHud3Ref.value; settingsChanged()//the setting records where the user left this hud, not just where it started
}
function updateInformation() {
	if (!showHud3Ref.value) return//a hidden hud builds no string and touches no ref, so measuring with it off measures fuji rather than fuji plus a readout
	let s = 'no image loaded'
	if (here?.error) s = `${here.path}\ncould not be shown: ${here.error}`//the card is showing the error placeholder, so name the file and what it said rather than claiming nothing is loaded
	else if (here?.img && quiverC?.card2) {
		let f = cacheFootprint()//the store's running totals, free to read because they are kept rather than walked
s = `${here.path}
natural ${here.img.naturalWidth} width x ${here.img.naturalHeight} height, ${saySize4(here.blobBytes)} (${sayGroupDigits(here.blobBytes)} bytes)
displayed ${quiverC.card2.x} width x ${quiverC.card2.y} height (CSS pixels, not backing)
${Math.round(here.loaded - here.requested)}ms disk + ${Math.round(here.rendered - here.loaded)}ms render, to load this one
flip ${flipMs}ms (${flipFrames} frames) = ${storeMs}ms store + ${paintMs}ms paint
cache ${f.count} images, ${saySize4(f.blobs)} of files + ${saySize4(f.pixels)} of pixels`
	}
	s += `\ngamma ${gamma.value == 1 ? '1, off' : gamma.value.toFixed(2)}`//at the end of every reading, loaded or not, because it is a lens over the whole window rather than a fact about one picture
	hud3Ref.value = s
}
function updateCaption() {//the caption below the card: the picture's full path, the day it was last modified, its size and its dimensions, the line the contact sheet writes under a thumbnail with the path in place of the name, since there is room for it here
	if (!showCaptionRef.value || !here) return
	let file = modelFile(here.path)//the listing's entry, with the modified time and the size; false for a picture outside the listed folder, which then shows its path and dimensions alone
	let parts = []
	if (file) parts.push(sayDay(file.mtime), saySize4(file.size))
	if (here.img) parts.push(sayDimensions(xy(here.img.naturalWidth, here.img.naturalHeight)))//none for the error placeholder, whose dimensions are nobody's
	captionRef.value = `${backize(here.path)}\n${parts.filter(part => part).join(` ${middleDot} `)}`//two lines, as under a thumbnail: the path, then the details
}
watch(gamma, updateInformation)//the keys, the wheel and the drag all change it, and neither goes through the quiver, which is what refreshes this hud for everything else

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
	class="myFrame myDots myWillChangeBackgroundPosition relative w-full h-full overflow-hidden select-none touch-none"
	@contextmenu.prevent
	@dblclick.prevent="onDoubleClick"
	@pointerdown="onPointerDown"
	@pointermove="onPointerMove"
	@pointerup="onUp" @pointercancel="onUp" @lostpointercapture="onUp"
>

	<!-- Card: rectangular image container; drag to pan around in infinite space; caption text is within card but positioned below card -->
	<div
		ref="cardRef"
		class="myCard myShadow myDry myWillChangeTransform bg-well"
	>

		<!-- the images the card shows are the store's own elements, put here by cardShow; this one is only for a file fuji could not read -->
		<img ref="errorRef" class="myImage" :src="errorImageData" />

		<!-- caption lives inside the card, but sits below its border; pre keeps its two lines apart and never wraps either -->
		<div v-if="showCaptionRef" class="absolute bottom-0 translate-y-full py-2 whitespace-pre myMono myEmbossed">{{captionRef}}</div>

	</div>

	<!-- HUD, inside the frame, next to the card -->
	<div v-if="showHud2Ref" class="myHud myDry absolute top-4 right-4">{{hud2Ref}}</div>
	<div v-if="showHud3Ref" class="myHud myDry absolute bottom-0 inset-x-0">{{hud3Ref}}</div>

</div>

</template>
<style scoped>

.myHud { /* the table's huds, over the look index.css gives every hud */
	white-space: pre-wrap; /* honor \n and wrap at the container width */
}
.myFrame {} /* not using this yet, but it's here */
.myCard { outline: 1px solid var(--color-frame) } /* the line around the card, drawn outside its box so the card's width and height are the image's exactly; a border would sit inside them, and the img at 100% resolves against the padding box, leaving the image two pixels short of natural times n */

/*
The image on the card is the store's own element: cache.js makes it with new Image() and cardShow adopts it into the card. An element the template did not create never carries this component's data-v attribute, so a plain scoped .myImage rule compiles to .myImage[data-v-...] and can never match it. :deep() compiles to .myCard[data-v-...] .myImage instead, putting the attribute on the card, which the template does own, and reaching the image as a descendant. The error image in the template above is a real template element and matches this rule too.

Without this the only rule landing on an adopted image was tailwind's own img{max-width:100%; height:auto}, which shrinks a picture to fit the card but will never enlarge one. The card kept sizing to the diamond in both directions while the picture stopped at its natural size, sitting at one to one in the card's top left corner with black around it, so the table could zoom out but not in.
*/
.myCard :deep(.myImage) {
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

.myDots { /* the ground the card pans over, the paper every view stands on, its dots a step lighter than it in dark and a step darker in light, from the palette in index.css */
	background-color: var(--color-paper);
	background-image: radial-gradient(circle at center, var(--color-dot) 6px, transparent 6px);
	background-size: 60px 60px;
	background-position: 0 0, 30px 30px;
}
.myShadow {
	box-shadow: 4px 4px 12px var(--color-shade); /* lit from the top left, the same shadow every thumbnail on the sheet casts */
}
.myEmbossed {
	white-space: pre; /* honor \n and overflow the container */
	color: var(--color-fainter);
	text-shadow: /* a glow in the paper's color, black in dark and white in light, which lifts the text off the dots */
		-1px -1px 2px var(--color-paper),
		1px 1px 2px var(--color-paper),
		0 0 8px var(--color-paper);
}

</style>
