<script setup>//fuji audio: the view that plays sound, and the square where a video draws. For now it is where the media library is reached from: it shows that the library is linked and answers, plays a dropped file, and is the floor Fuji Audio and Fuji Video are built on

import {ref, nextTick, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {mpvVersion, mpvStart, mpvSend, mpvRead, mpvStop} from '../mpv.js'
import {windowVideoMake, windowVideoDrop} from '../window.js'
import {log, logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {backize} from './library.js'

/*
The shell trades this with the sheet when either asks with a, and brings it from the settings panel the same way; which view is showing is the shell's, so this only asks. A view in the sheet's window rather than a window of its own, so the library and the builds can be worked on before fuji's windows are.

**The player is this view's, started when it comes on screen and stopped when it goes.** libmpv's own profile already leaves mpv with no terminal, no on-screen controller and no key bindings of its own, so the page is the only thing that will ever press a key; the page adds the few options every build has, and a video drawn in the square where the platform gives the page one, through its native window handed over as wid, and the video switched off where it does not, so mpv never opens a window of its own. A dropped file is loaded and played; play loads the last file again; stop stops. No pause control and no seek bar, since those are the products' to shape. While the view shows, it reads the position and the length every half second, which is the one road back from the player until events come up.

**The square is a rectangle of the page that a native window sits over,** so nothing is ever drawn on the video, and the panel does not scroll, because a native window would not scroll with it. A click anywhere on the video toggles pause, and a drag anywhere on the panel, the video included, moves the whole window; both are the page's. The three handlers sit on the panel's root, and a press remembers whether it began on the square, which is what tells the click that toggles pause from a click on a button or on the text. On Windows the mouse over the video reaches the page as the window event video-mouse, which the native window sends up, since mpv's window inside it takes no mouse and Windows hands the mouse to the window around it; window.rs says how. On the Mac the page's own handlers see the mouse directly. A press that moves more than a few pixels is a drag and asks the window to follow the pointer, and Windows runs the move itself from there, so the release never reaches the page; a press that stays is a click, told on the release. A double click anywhere on the panel asks the shell for fuji's own fullscreen, on or off, with this view staying, the way a video wants the screen as a picture does; its two clicks toggle pause twice on the way, so the video is playing or paused after as it was before. A drop on the square reaches the shell as its own event on Windows, and the shell hands it down the road every drop takes, so this view sees one kind of drop.
*/

const version = ref('')//what the library said, or blank until it has answered
const trouble = ref('')//or why it could not, and every error after, one line each, since the first is the one that explains the rest
function troubled(where, error) { trouble.value += `${trouble.value ? '\n' : ''}${where}: ${error}`; logTrouble(`audio: ${where}`, error) }//on the panel and in the log
const path = ref('')//the file playing, or the last one that did
const position = ref('')//time-pos and duration as the player says them, read every tick
const paused = ref('')//yes or no, as the player says it
const square = ref(null)//the rectangle the video draws in
let ticker//the half-second read while the view shows
let press//where the button went down and whether that was on the square, or null while it is up; a drag is told from a click by how far the pointer moved from here, and a click toggles pause only when it began on the video

let startedDone//settles started, below
const started = new Promise(resolve => startedDone = resolve)//settled once start has run, player or no player, for a shell that has a file to hand over the moment the view is ready
async function start() {//the shell calls this each time this view comes on screen
	try { await starting() } finally { startedDone() }
}
async function starting() {
	mpvVersion()
		.then(v => { version.value = `${v.mpv}, client api ${v.api}, FFmpeg ${v.ffmpeg}` })
		.catch(error => { troubled('asking the library its version', error) })
	await nextTick()//the square has a layout box to measure
	try {
		await listenMouse()//before the native window exists, so no event is missed
		const wid = await windowVideoMake(square.value.getBoundingClientRect())//a child window on windows, and the address of the view mpv.rs draws through on the mac and linux
		const begun = performance.now()
		await mpvStart([//nothing here about the terminal, the on-screen controller, mpv's own key bindings or staying idle after a file: libmpv applies its own profile as a player is made, the libmpv section of mpv's etc/builtin.conf, which turns all of those off and idle on. And an option a build lacks is an error rather than a no-op: osc and load-scripts exist only in a build with scripting, which the Windows dll is not, found on the Windows box on 2026-10-13, so the page names only options every build has
			['audio-fallback-to-null', 'yes'],//a machine with no sound device still plays, silently, with time running; without this mpv drops the audio track, and a file with nothing else ends at once
			['keepaspect', 'yes'],//fitted to the square with bars, rather than stretched to it
			['hwdec', 'auto-safe'],//the hardware decoder, where mpv's conservative choice finds one
			...(wid ? [['vid', 'auto'], ['wid', wid]] : [['vid', 'no']]),//the video in the square's native window, or no video at all where the platform gives the page none, so mpv never opens a window of its own
		])
		log(`audio: player started in ${Math.round(performance.now() - begun)} ms`)
		ticker = setInterval(() => tick().catch(error => { clearInterval(ticker); troubled('reading the player', error) }), 500)//once, since a player that cannot be read will not be read next tick either
	} catch (error) { troubled('starting the player', error) }
}

let described = ''//the path whose playing state has been logged, once, the first tick it had a position
async function tick() {//the player's position, every half second while the view shows
	const [time, duration, pause] = await Promise.all([mpvRead('time-pos'), mpvRead('duration'), mpvRead('pause')])
	position.value = time ? `${clock(time)} / ${clock(duration)}` : ''
	paused.value = pause
	if (time && described != path.value) {//one line for the record: what the player made of the file
		described = path.value
		const [params, hwdec, vo, dropped] = await Promise.all([mpvRead('video-params/pixelformat'), mpvRead('hwdec-current'), mpvRead('vo-configured'), mpvRead('frame-drop-count')])
		log(`audio: playing ${path.value}: ${clock(duration)} long, video ${params || 'none'}, hwdec ${hwdec || 'none'}, vo configured ${vo || 'no'}, ${dropped || 0} frames dropped`)
	}
}
function clock(seconds) {//1:05 from 65.3, as the player says a time
	const whole = Math.floor(Number(seconds) || 0)
	return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`
}

async function onDrop(dropped) {//the shell hands this every file dropped on the window while this view shows
	path.value = dropped
	await play()
}
async function play() {//load the file and play it from the start
	if (!path.value) return
	try {
		const begun = performance.now()
		await mpvSend(['loadfile', path.value])
		log(`audio: loadfile answered in ${Math.round(performance.now() - begun)} ms, ${path.value}`)
	} catch (error) { troubled('loading a file', error) }
}
async function stop() {
	try { await mpvSend(['stop']) } catch (error) { troubled('stopping', error) }
}

let unlistenMouse//the window's video-mouse events, which on windows are how the mouse over the video reaches the page: the native window forwards each one, since mpv's own window would otherwise keep them; window.rs says why. On the mac the page's own handlers below see the mouse directly, and both roads end in the same three functions
async function listenMouse() {
	unlistenMouse = await getCurrentWindow().listen('video-mouse', e => {//the point is in the video's rectangle in backing pixels, and only distances are taken from it, so css pixels by the ratio are enough
		const {kind, x, y} = e.payload
		const at = {button: 0, clientX: x / devicePixelRatio, clientY: y / devicePixelRatio, target: square.value}//on the square by definition, since the native window covers nothing else
		if (kind == 'down') onPress(at); else if (kind == 'move') onMove(at); else if (kind == 'double') { onPress(at); onDouble() } else onRelease()//the second press of a double click is a press too, so its release is a click like the first, and both platforms toggle pause twice and go fullscreen once
	})
}
function onPress(e) { if (e.button == 0) press = {x: e.clientX, y: e.clientY, video: square.value?.contains(e.target)} }
function onMove(e) {//a press that has moved is a drag, and from here the window follows the pointer
	if (!press || (Math.abs(e.clientX - press.x) < 4 && Math.abs(e.clientY - press.y) < 4)) return
	press = null
	log('audio: drag on the panel, window follows the pointer')
	getCurrentWindow().startDragging().catch(error => troubled('dragging the window', error))
}
function onRelease() {//a press that stayed is a click, and a click on the video toggles pause; a click anywhere else on the panel is the button's or nobody's
	const was = press
	press = null
	if (!was?.video) return
	if (path.value) mpvSend(['cycle', 'pause']).then(() => mpvRead('pause')).then(pause => log(`audio: click on the square, pause ${pause}`)).catch(error => troubled('toggling pause', error))
}
function onDouble() {//a double click anywhere on the panel, the video included, trades the window for the screen and back, the way the light table has it; the page's own dblclick is the road on the mac and on the page beside the square, and video-mouse's double the road on the square on windows, so neither platform hears it twice
	log('audio: double click, fullscreen toggles')
	emit('fullscreen')
}

onBeforeUnmount(() => {//the view goes away: the sound stops with it, and the native window goes after the player, so mpv takes its own window down before ours is destroyed under it
	clearInterval(ticker)
	if (unlistenMouse) unlistenMouse()
	mpvStop().catch(error => troubled('stopping the player', error))
		.then(() => windowVideoDrop()).catch(error => troubled('dropping the video window', error))
})

const emit = defineEmits(['sheet', 'fullscreen'])//a, for the contact sheet back; a double click, for fuji's own fullscreen on or off with this view staying
function onKey(e) {
	if (e.key == 'a' && !e.ctrlKey && !e.metaKey) emit('sheet')
}

defineExpose({start, started, onDrop, onKey})//the calls of the shell's this view has a use for, and the promise that says start has run

</script>
<template>

<div class="myMono bg-paper w-full h-full overflow-hidden p-8 select-none" @mousedown="onPress" @mousemove="onMove" @mouseup="onRelease" @mouseleave="press = null" @dblclick="onDouble"><!-- a drag anywhere on the panel moves the window and a double click anywhere toggles fullscreen, so the handlers are the root's; a press on the square is the one that can be a click -->
	<h1 class="mb-8 text-strong">{{brandName}} Audio</h1>
	<p v-if="version">{{version}}</p><!-- the library names itself at the start of the line -->
	<p v-else-if="trouble" class="text-warn">{{trouble}}</p>
	<p v-else class="text-fainter italic">Asking the library…</p>
	<div ref="square" class="mt-8 w-[640px] h-[360px] bg-well text-fainter flex items-center justify-center"><!-- the video's rectangle; what shows here is the native window over it, and this text only until it is made -->
		<span>The video draws here. Drop a file anywhere on the page</span><!-- the square too: on windows it is a native window that asks for drops itself, since 2026-10-14, after a day of refusing them -->
	</div>
	<p class="mt-8 text-strong">{{path ? backize(path) : 'Nothing loaded'}}</p>
	<p class="mt-2">{{position || '—'}}<span v-if="paused == 'yes'" class="text-fainter"> paused</span></p>
	<p class="mt-8"><button :disabled="!path" @click="play">Play</button> <button :disabled="!path" @click="stop">Stop</button></p>
	<p v-if="trouble && version" class="mt-8 text-warn whitespace-pre-line">{{trouble}}</p><!-- every error so far, first at the top -->

</div>

</template>
