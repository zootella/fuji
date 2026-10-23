<script setup>//fuji audio: the view that plays sound, and the region at its top where a video draws. For now it is where the media library is reached from: it shows that the library is linked and answers, plays a dropped file, and is the floor Fuji Audio and Fuji Video are built on

import {ref, nextTick, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {Time, sayTimeRace} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in
import {mediaVersion, mediaStart, mediaSend, mediaRead, mediaStop, mediaProbe, mediaFrame, mediaFrameUnpack} from '../media.js'
import {windowVideoMake, windowVideoPlace, windowVideoDrop} from '../window.js'
import {dialogOpen} from '../dialog.js'
import {picturesFolder} from '../paths.js'
import {modelFolder} from '../model.js'//the folder the sheet shows, where the open box starts
import {log, logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {backize, xy, raf, sayDimensions, middleDot} from './library.js'

/*
The shell trades this with the sheet when either asks with a, and brings it from the settings panel the same way; which view is showing is the shell's, so this only asks. A view in the sheet's window rather than a window of its own, so the library and the builds can be worked on before fuji's windows are.

**The player is this view's, started when it comes on screen and stopped when it goes.** libmpv's own profile already leaves mpv with no terminal, no on-screen controller and no key bindings of its own, so the page is the only thing that will ever press a key; the page adds the few options every build has, and a video drawn in the region where the platform gives the page one, through its native window handed over as wid, and the video switched off where it does not, so mpv never opens a window of its own. A dropped file is loaded and played, and at its end the player stops on the last frame, paused, with the file still loaded; play loads the last file again from the start; stop stops and unloads it. No pause control and no seek bar, since those are the products' to shape. While the view shows, it reads the position and the length every half second, which is the one road back from the player until events come up.

**The region takes the video's own shape, with no bars and no gap.** It sits at the inner corner of the panel's inset, against its left edge at the inset's full height when the video is taller than it is wide, and against its top edge at the full width otherwise; the panel's lines and buttons fill the rest, beside or below it, in an area that scrolls on its own with its scroll bar always shown, and gives up room to the video only down to what it keeps for itself. The panel itself never scrolls, since the native view over the region would not move with it, and nothing beside or above the region can push it. Its shape is the playing player's own answer, dwidth and dheight, the size after the pixel shape, the file's or square, swapped for a quarter turn, read each animation frame after a load until the first frame is decoded and again on every tick, since a stream can change its size; the probe's headers alone cannot always say it, a DVD's MPEG-2 among them. regionPlace works the rectangle out in backing pixels, sets the css box to the same pixels with no background of its own, and places the native view over it with window_video_place, so the box and the view cover exactly the same pixels and nothing of the page shows around the video; the view is made once as the panel starts, hidden until there is a video, and placed again whenever the window changes size. There is a region only while a video shows: with nothing loaded, or a sound playing, the region and its divider are gone and the rest has the whole inset, so nothing is drawn, and no shape made up, for a video that may never come. Fit fills that edge. The other sizes come in two rows with the same four steps, Quarter, Half, One and Two: each of the video's pixels on that many css pixels in the one, and on that many backing pixels in the other, so the rows agree at one backing pixel to the css pixel and differ everywhere else, and at two backing pixels to the css pixel CSS Half is Backing One. The video's pixels here are dwidth's, which stretch a file's pixels along one edge where they are not square, as mpv's own size does, so none is ever smaller than a screen pixel. A size that does not fit the room shows fitted, with a line saying so, since fuji never crops a video and a test never changes the window, and takes its size the moment the window gives it the room. Beneath the buttons the panel says what the video shows at in css pixels and in backing pixels, and the file's own pixels, all three kept up to date as the window, the choice or the file changes. The player fills the view, keepaspect off, since the view already has the video's shape to within half a backing pixel, where mpv's own fitting could leave a line of black; and it never shows a sound's cover art. The user can have a file's pixels taken as square whatever shape it declares: mpv's video-aspect-method set to ignore rather than container, its default, which mpv applies where it decodes, from the next frame it decodes, so the shape it reports, and the region, follow by the same road. A new file starts as it says. Every placement is a line in the log.

**The region is a rectangle of the page that a native window sits over,** so nothing is ever drawn on the video. A click anywhere on the video toggles pause, and a drag anywhere on the panel but a button, the video included, moves the whole window; both are the page's. The three handlers sit on the panel's root, and a press remembers whether it began on the region, which is what tells the click that toggles pause from a click on the text; a press on a button is the button's alone, so it never drags the window, and two quick presses on one never ask for fullscreen. On Windows the mouse over the video reaches the page as the window event video-mouse, which the native window sends up, since mpv's window inside it takes no mouse and Windows hands the mouse to the window around it; window.rs says how. On the Mac the page's own handlers see the mouse directly. A press that moves more than a few pixels is a drag and asks the window to follow the pointer, and Windows runs the move itself from there, so the release never reaches the page; a press that stays is a click, told on the release. A double click anywhere on the panel but a button asks the shell for fuji's own fullscreen, on or off, with this view staying, the way a video wants the screen as a picture does; its two clicks toggle pause twice on the way, so the video is playing or paused after as it was before, which shows as a flicker of play and pause before the screen changes. Holding a single click back for the double-click interval would end the flicker, at the cost of a click that does not act at once, and so would a key for fullscreen with the double click dropped; the flicker is kept. A drop on the region reaches the shell as its own event on Windows, and the shell hands it down the road every drop takes, so this view sees one kind of drop.

**Below the player, the panel asks a file what it is without playing it.** Open puts up the system's open box, unfiltered like the shell's; one call asks the chosen file for its running time, its format and its tracks, and a second, which knows nothing of the first but the time it is handed, asks for the frame from the middle. Each answers from a player of its own, so the one playing never notices. The frame arrives turned and with its pixels made square, and is shrunk to the column by css, which is an example's shortcut rather than how fuji shows a picture.
*/

const version = ref('')//what the library said, or blank until it has answered
const trouble = ref('')//or why it could not, and every error after, one line each, since the first is the one that explains the rest
function troubled(where, error) { trouble.value += `${trouble.value ? '\n' : ''}${where}: ${error}`; logTrouble(`audio: ${where}`, error) }//on the panel and in the log
const path = ref('')//the file playing, or the last one that did
const position = ref('')//time-pos and duration as the player says them, read every tick
const paused = ref('')//yes or no, as the player says it
const panel = ref(null)//the panel's root, whose width the region fits and whose size the observer watches
const region = ref(null)//the box at the top that the native view covers exactly
const square = ref(false)//the file's pixels taken as square whatever shape it declares, rather than as the file says
const sizeSteps = {Quarter: 1 / 4, Half: 1 / 2, One: 1, Two: 2}//each of the video's pixels on this many css pixels in the CSS row, and on this many backing pixels in the Backing row
const sizing = ref({unit: 'Fit', step: ''})//Fit, or a step in CSS or in Backing, kept from one file to the next
const tall = ref(false)//the video is taller than it is wide, so it stands at the left with the rest beside it, rather than along the top with the rest below
const sizeNote = ref('')//why the size chosen could not be shown, or blank
const showingCss = ref('—')//the region's size in css pixels while a video shows
const showingBacking = ref('—')//and in backing pixels, which is the native view's own
const filePixels = ref('—')//the file's own pixels, as it stores them
const showing = ref(false)//a video is showing, so the region and its divider are there
const frameCss = 10//the panel's inset in css pixels, which the blue frame paints over while it is there; the region's top left sits at its inner corner
const dividerCss = 10//the divider between the region and the rest
const restLeast = {beside: 384, below: 160}//css pixels the rest keeps beside a tall video and below a wide one, so its lines and buttons always have room: the middle frame's 320 with the padding either side, or a few lines
const asked = ref('')//the file chosen to ask about, or blank before one is
const answer = ref('')//what the probe said about it, in one line
const still = ref(null)//the canvas its middle frame is drawn in
let shape = null//{w, h}, the size the video shows at in its own pixels, as the player says it, or null while there is no video to show
let file = null//{w, h, turn}, the pixels the file stores and the turn it declares, read beside the shape
let placed = ''//the last rectangle handed to the native view, as text, so an unchanged one is not sent again
let watching = 0//counts the shape watches, so one a newer one replaced stops
let observer//the panel's ResizeObserver, which places the region again as the window changes size
let ticker//the half-second read while the view shows
let press//where the button went down and whether that was on the region, or null while it is up; a drag is told from a click by how far the pointer moved from here, and a click toggles pause only when it began on the video

let startedDone//settles started, below
const started = new Promise(resolve => startedDone = resolve)//settled once start has run, player or no player, for a shell that has a file to hand over the moment the view is ready
async function start() {//the shell calls this each time this view comes on screen
	try { await starting() } finally { startedDone() }
}
async function starting() {
	mediaVersion()
		.then(v => { version.value = `${v.mpv}, client api ${v.api}, FFmpeg ${v.ffmpeg}` })
		.catch(error => { troubled('asking the library its version', error) })
	await nextTick()//the panel has a layout box to measure
	try {
		await listenMouse()//before the native window exists, so no event is missed
		const wid = await windowVideoMake({x: 0, y: 0, width: 1, height: 1})//made in the corner and hidden at once below, since it takes its place only once a video's shape is known, and the player needs its handle before it starts: a child window on windows, and the address of the view media.rs draws through on the mac and linux
		await windowVideoPlace({x: 0, y: 0, width: 0, height: 0})
		placed = 'hidden'
		const begun = performance.now()
		await mediaStart([//nothing here about the terminal, the on-screen controller, mpv's own key bindings or staying idle after a file: libmpv applies its own profile as a player is made, the libmpv section of mpv's etc/builtin.conf, which turns all of those off and idle on. And an option a build lacks is an error rather than a no-op: osc and load-scripts exist only in a build with scripting, which the Windows dll is not, found on the Windows box on 2026-10-13, so the page names only options every build has
			['audio-fallback-to-null', 'yes'],//a machine with no sound device still plays, silently, with time running; without this mpv drops the audio track, and a file with nothing else ends at once
			['keepaspect', 'no'],//filling the view, which already has the video's shape
			['audio-display', 'no'],//never a sound's cover art shown as though it were video
			['keep-open', 'yes'],//at a file's end the player pauses on the last frame rather than unloading the file, so the video stays where it was, as a player's does
			['hwdec', 'auto-safe'],//the hardware decoder, where mpv's conservative choice finds one
			...(wid ? [['vid', 'auto'], ['wid', wid]] : [['vid', 'no']]),//the video in the region's native window, or no video at all where the platform gives the page none, so mpv never opens a window of its own
		])
		log(`audio: player started in ${Math.round(performance.now() - begun)} ms`)
		observer = new ResizeObserver(() => regionPlace().catch(error => troubled('placing the video', error)))
		observer.observe(panel.value)
		ticker = setInterval(() => tick().catch(error => { clearInterval(ticker); troubled('reading the player', error) }), 500)//once, since a player that cannot be read will not be read next tick either
	} catch (error) { troubled('starting the player', error) }
}

let described = ''//the path whose playing state has been logged, once, the first tick it had a position
async function tick() {//the player's position, every half second while the view shows, and its shape, which can change mid-stream and is gone once a stop unloads the file
	const [time, duration, pause] = await Promise.all([mediaRead('time-pos'), mediaRead('duration'), mediaRead('pause')])
	position.value = time ? `${clock(time)} / ${clock(duration)}` : ''
	paused.value = pause
	if (!(await shapeRead()) && shape) regionHide()//the video gone, after a stop or a load that failed, so no black is left where it was
	if (time && described != path.value) {//one line for the record: what the player made of the file
		described = path.value
		const [params, hwdec, vo, dropped] = await Promise.all([mediaRead('video-params/pixelformat'), mediaRead('hwdec-current'), mediaRead('vo-configured'), mediaRead('frame-drop-count')])
		log(`audio: playing ${path.value}: ${clock(duration)} long, video ${params || 'none'}, hwdec ${hwdec || 'none'}, vo configured ${vo || 'no'}, ${dropped || 0} frames dropped`)
	}
}
function clock(seconds) {//1'05"300 from 65.3, the player's seconds timed the way ergo times a race
	return sayTimeRace((Number(seconds) || 0) * Time.second)
}

async function onDrop(dropped) {//the shell hands this every file dropped on the window while this view shows
	path.value = dropped
	await play()
}
async function play() {//load the file and play it from the start, at its own shape until the user gives it another
	if (!path.value) return
	try {
		regionHide()//until the new file's shape is known, so its first frame never shows stretched to the last one's
		if (square.value) { await mediaSend(['set', 'video-aspect-method', 'container']); square.value = false }
		const begun = performance.now()
		await mediaSend(['loadfile', path.value])
		await mediaSend(['set', 'pause', 'no'])//after the load, since pause is the player's rather than the file's: keep-open paused it at the last file's end, and a file loaded then opens paused on its first frame, found on the Mac mini on 2026-10-16; before the load it would not hold, since a player at a file's end pauses again
		log(`audio: loadfile answered in ${Math.round(performance.now() - begun)} ms, ${path.value}`)
		shapeWatch(null)
	} catch (error) { troubled('loading a file', error) }
}
async function stop() {
	try { await mediaSend(['stop']); regionHide() } catch (error) { troubled('stopping', error) }
}

async function shapeSquare(yes) {//the user's command for the shape: the file's pixels taken as square, or as the file says, with container, mpv's default. mpv applies it where it decodes, from the next frame it decodes, so the shape it reports, and the region with it, follow once the frames already decoded have shown: between 50 and 550 ms into a playing DVD clip, and at once going back even paused, on the Mac mini on 2026-10-16
	try {
		await mediaSend(['set', 'video-aspect-method', yes ? 'ignore' : 'container'])
		square.value = yes
		log(`audio: shape ${yes ? 'square' : 'as the file says'}`)
		shapeWatch(shape)
	} catch (error) { troubled('changing the shape', error) }
}
function sizeChoose(unit, step) {//Fit, or a step in CSS or in Backing
	sizing.value = {unit, step}
	regionPlace().catch(error => troubled('placing the video', error))
}

async function shapeWatch(from) {//the shape read each animation frame, for five seconds at most, until the player gives one other than from: after a load, any shape at all, and after the shape is changed, a new one. A newer watch replaces this one
	const mine = ++watching
	const until = performance.now() + 5*Time.second
	while (mine == watching && performance.now() < until) {
		await shapeRead()
		if (shape && (!from || shape.w != from.w || shape.h != from.h)) return
		await raf()
	}
}
async function shapeRead() {//the shape the player says the video shows at, and the pixels the file stores, kept, and the region placed again if either changed; answers whether the player had a shape to give
	const [w, h, turn, storedW, storedH, storedTurn] = await Promise.all([mediaRead('dwidth'), mediaRead('dheight'), mediaRead('video-out-params/rotate'), mediaRead('width'), mediaRead('height'), mediaRead('video-params/rotate')])
	let across = Number(w), down = Number(h)
	if (!(across > 0 && down > 0)) return false//no video, or none decoded yet
	if (Number(turn) % 180 == 90) [across, down] = [down, across]//where the output turns the picture itself, dwidth and dheight are before the turn and rotate says it; where mpv turned it with a filter first, they are after it and rotate is zero, as with the null output on the Mac mini on 2026-10-16, so this swap is right either way
	const stored = {w: Number(storedW), h: Number(storedH), turn: Number(storedTurn) || 0}//width and height as decoded, before any pixel shape or turn, and the turn the file declares, which video-params keeps whichever way mpv carries it out: 720 by 480 for a DVD that shows at 853 by 480, measured on the Mac mini on 2026-10-16
	if (!shape || shape.w != across || shape.h != down || !file || file.w != stored.w || file.h != stored.h || file.turn != stored.turn) {
		shape = {w: across, h: down}
		file = stored
		await regionPlace()
	}
	return true
}
function regionHide() {//no video to show: the region and its divider go, and the native view is hidden
	shape = null
	file = null
	regionPlace().catch(error => troubled('placing the video', error))
}
function fileSay() {//the file's own pixels, with the turn it declares and, where its pixels are not square, the size they make once they are
	if (!shape || !file?.w) return '—'
	const parts = [sayDimensions(xy(file.w, file.h))]
	if (file.turn) parts.push(`turned ${file.turn}°`)
	const turned = file.turn % 180 == 90 ? xy(file.h, file.w) : xy(file.w, file.h)
	if (turned.x != shape.w || turned.y != shape.h) parts.push(`${sayDimensions(xy(shape.w, shape.h))} with square pixels`)
	return parts.join(', ')
}

async function regionPlace() {//the region's rectangle worked out in whole backing pixels from the video's shape and the room the panel has, the css box set to the same pixels, and the native view placed over it; with no video there is no region, and the view is hidden
	if (!panel.value) return//the view has gone
	const backingPerCss = devicePixelRatio
	showing.value = !!shape
	tall.value = !!shape && shape.h > shape.w
	await nextTick()//the region and its divider made or taken away, and the panel laid out in that direction, before anything is measured
	sizeNote.value = ''
	filePixels.value = fileSay()
	const box = region.value
	if (!shape || !box) {//no video, so no region, and the native view hidden
		showingCss.value = showingBacking.value = '—'
		if (placed == 'hidden') return
		placed = 'hidden'
		await windowVideoPlace({x: 0, y: 0, width: 0, height: 0})
		log('audio: video hidden, no video to show')
		return
	}
	const outer = panel.value.getBoundingClientRect()
	const room = xy(Math.floor((outer.width - 2 * frameCss) * backingPerCss), Math.floor((outer.height - 2 * frameCss) * backingPerCss))//inside the inset, in whole backing pixels
	const most = tall.value ? xy(room.x - Math.round((dividerCss + restLeast.beside) * backingPerCss), room.y) : xy(room.x, room.y - Math.round((dividerCss + restLeast.below) * backingPerCss))//what the region may take, leaving the divider and the rest their room
	let size = tall.value ? xy(Math.round(most.y * shape.w / shape.h), most.y) : xy(most.x, Math.round(most.x * shape.h / shape.w))//against the left edge at the full height, or along the top at the full width, at the video's shape
	if (size.x > most.x) size = xy(most.x, Math.round(most.x * shape.h / shape.w))//narrower, where the full height would leave the rest too little width
	if (size.y > most.y) size = xy(Math.round(most.y * shape.w / shape.h), most.y)//shorter, where the full width would leave it too little height
	const {unit, step} = sizing.value
	if (unit != 'Fit') {
		const backingPerFile = unit == 'CSS' ? sizeSteps[step] * backingPerCss : sizeSteps[step]
		const wanted = xy(Math.round(shape.w * backingPerFile), Math.round(shape.h * backingPerFile))//whole backing pixels, so at a fractional backing per css a css size can come out a fraction of a pixel off its round number, and the line beneath says so
		if (wanted.x <= most.x && wanted.y <= most.y) size = wanted
		else sizeNote.value = `${unit} ${step} would take ${sayDimensions(wanted)} backing pixels and the room has ${sayDimensions(xy(Math.max(0, most.x), Math.max(0, most.y)))}, so the video shows fitted until the window gives it that`
	}
	size = xy(Math.max(0, size.x), Math.max(0, size.y))//none at all in a window too small to leave the rest its room
	box.style.width = `${size.x / backingPerCss}px`//css pixels that are whole backing pixels, so the box ends where the view does
	box.style.height = `${size.y / backingPerCss}px`
	showingCss.value = sayDimensions(xy(Number((size.x / backingPerCss).toFixed(2)), Number((size.y / backingPerCss).toFixed(2))))
	showingBacking.value = sayDimensions(size)
	const at = box.getBoundingClientRect()//where the box landed, in css pixels from the page's top left, which is the web view's
	const rect = {x: Math.round(at.left * backingPerCss), y: Math.round(at.top * backingPerCss), width: size.x, height: size.y}//the native view over the box, exactly, wherever the box's corner is a whole backing pixel. At 125 percent the panel's 10-pixel inset puts it at 12.5, which rounds to 13, half a pixel from where the page draws the box, so a sliver of the page or of the blue frame could show along one edge; 150 is a whole 15. Not yet looked at on Windows, where the region was seen whole at 100 percent on 2026-10-22, and to be looked at at 125 percent when this region moves to the video window, which takes the same rounding with it
	const said = `${rect.x},${rect.y} ${rect.width}x${rect.height}`
	if (said == placed) return
	placed = said
	await windowVideoPlace(rect)
	log(`audio: video placed at ${said} backing pixels, the box ${at.left},${at.top} ${at.width}x${at.height} css at ${backingPerCss} backing per css, for a video showing ${shape.w}x${shape.h}${tall.value ? ', tall' : ''}, ${unit}${step ? ` ${step}` : ''}${sizeNote.value ? ' too big, so fitted' : ''}${square.value ? ', square' : ''}`)
}

async function ask() {//the open box, then the two questions, each its own call down and timed in the log
	const [chosen] = await dialogOpen({start: modelFolder.value || await picturesFolder()})//a file, opened in the folder the sheet shows, or Pictures when it shows none, as the shell's box opens
	if (!chosen) return//the user cancelled
	asked.value = chosen
	answer.value = 'Asking…'
	still.value.width = 0; still.value.height = 0//the last file's frame gone, so it is never shown beside this one's answer
	try {
		let began = performance.now()
		const probe = await mediaProbe(chosen, 10)
		const video = JSON.parse(probe.tracks || '[]').find(track => track.type == 'video' && !track.albumart)//the first picture that moves, never a song's cover art
		const seconds = Number(probe.duration) || 0
		answer.value = describe(probe.format, seconds, video)
		log(`audio: probe answered in ${Math.round(performance.now() - began)} ms, ${chosen}: ${answer.value}`)
		if (!video) return

		began = performance.now()
		const frame = mediaFrameUnpack(await mediaFrame(chosen, seconds / 2, 10))
		log(`audio: frame answered in ${Math.round(performance.now() - began)} ms, ${frame.width}x${frame.height} at ${clock(seconds / 2)}`)
		const canvas = still.value
		canvas.width = frame.width; canvas.height = frame.height
		canvas.getContext('2d').putImageData(new ImageData(frame.pixels, frame.width, frame.height), 0, 0)//the css width is 320 and the height follows the frame's own shape
	} catch (error) {
		answer.value = String(error)//a file the library refuses, or one that gives no answer in time, is an ordinary answer here, said where the answer goes
		logTrouble('audio: asking a file', error)
	}
}
function describe(format, seconds, video) {//one line from the probe alone: the format, the running time, and for a video its stored size and the shape it shows at once its pixel shape and its turn are applied, which the headers of a DVD's MPEG-2 leave out of the pixel shape until a frame is decoded
	const parts = [format, seconds > 0 ? clock(seconds) : 'no running time']
	const width = video?.['demux-w'], height = video?.['demux-h']
	if (!video) parts.push('no video')
	else if (width > 0 && height > 0) {
		const pixel = video['demux-par'], turn = video['demux-rotation'] || 0
		let across = width * (pixel || 1), down = height
		if (turn == 90 || turn == 270) [across, down] = [down, across]
		parts.push(`${sayDimensions(xy(width, height))} stored`, `shows ${(across / down).toFixed(3)} to 1${pixel ? '' : ' if its pixels are square, which its headers do not say'}`)
		if (pixel && pixel != 1) parts.push(`pixels ${pixel.toFixed(3)} wide`)
		if (turn) parts.push(`turned ${turn}°`)
	}
	return parts.join(` ${middleDot} `)
}

let unlistenMouse//the window's video-mouse events, which on windows are how the mouse over the video reaches the page: the native window forwards each one, since mpv's own window would otherwise keep them; window.rs says why. On the mac the page's own handlers below see the mouse directly, and both roads end in the same three functions
async function listenMouse() {
	unlistenMouse = await getCurrentWindow().listen('video-mouse', e => {//the point is in the video's rectangle in backing pixels, and only distances are taken from it, so css pixels by the ratio are enough
		const {kind, x, y} = e.payload
		const at = {button: 0, clientX: x / devicePixelRatio, clientY: y / devicePixelRatio, target: region.value}//on the region by definition, since the native window covers nothing else
		if (kind == 'down') onPress(at); else if (kind == 'move') onMove(at); else if (kind == 'double') { onPress(at); onDouble() } else onRelease()//the second press of a double click is a press too, so its release is a click like the first, and both platforms toggle pause twice and go fullscreen once
	})
}
function onPress(e) { if (e.button == 0) press = e.target?.closest?.('button') ? null : {x: e.clientX, y: e.clientY, video: region.value?.contains(e.target)} }//a press on a button is the button's, and never starts a drag
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
	if (path.value) mediaSend(['cycle', 'pause']).then(() => mediaRead('pause')).then(pause => log(`audio: click on the video, pause ${pause}`)).catch(error => troubled('toggling pause', error))
}
function onDouble(e) {//a double click anywhere on the panel but a button, the video included, trades the window for the screen and back, the way the light table has it; the page's own dblclick is the road on the mac and on the page around the video, and video-mouse's double the road on the video on windows, so neither platform hears it twice
	if (e?.target?.closest?.('button')) return//two quick presses on one button are two presses
	log('audio: double click, fullscreen toggles')
	emit('fullscreen')
}

onBeforeUnmount(() => {//the view goes away: the sound stops with it, and the native window goes after the player, so mpv takes its own window down before ours is destroyed under it
	clearInterval(ticker)
	watching++//any shape watch still running stops at its next turn
	if (observer) observer.disconnect()
	if (unlistenMouse) unlistenMouse()
	mediaStop().catch(error => troubled('stopping the player', error))
		.then(() => windowVideoDrop()).catch(error => troubled('dropping the video window', error))
})

const emit = defineEmits(['sheet', 'fullscreen'])//a, for the contact sheet back; a double click, for fuji's own fullscreen on or off with this view staying
function onKey(e) {
	if (e.key == 'a' && !e.ctrlKey && !e.metaKey) emit('sheet')
}

defineExpose({start, started, onDrop, onKey})//the calls of the shell's this view has a use for, and the promise that says start has run

</script>
<template>

<div ref="panel" class="myMono bg-paper w-full h-full overflow-hidden select-none flex p-[10px]" :class="tall ? 'flex-row' : 'flex-col'" style="outline: 10px solid blue; outline-offset: -10px" @mousedown="onPress" @mousemove="onMove" @mouseup="onRelease" @mouseleave="press = null" @dblclick="onDouble"><!-- a drag anywhere on the panel but a button moves the window and a double click anywhere but a button toggles fullscreen, so the handlers are the root's; a press on the region is the one that can be a click. The panel never scrolls, and its inset is what the blue frame paints over -->
	<template v-if="showing"><!-- the region and its divider, there only while a video shows -->
		<div ref="region" class="shrink-0"></div><!-- the region a video plays in, which the native view covers exactly; regionPlace sets its size in whole backing pixels, and it has no background, so nothing of its own can show around the video -->
		<div class="shrink-0" :class="tall ? 'w-[10px]' : 'h-[10px]'" style="background: blue"></div><!-- a divider between the video and the rest, painted like the frame while the region is being checked -->
	</template>
	<div class="flex-1 min-w-0 min-h-0 overflow-y-scroll p-8 wrap-break-word"><!-- the rest, beside a tall video or below a wide one, scrolling on its own with its scroll bar always shown, since nothing in it moves the video. Every line in it wraps at its spaces, and only a word too long for a line by itself, like a path, breaks inside, which overflow-wrap does once here for every line it holds, since it inherits; break-word rather than anywhere, so no box's own width ever counts a word as breakable -->
		<h1 class="mb-8 text-strong">{{brandName}} Audio</h1>
		<p v-if="version">{{version}}</p><!-- the library names itself at the start of the line -->
		<p v-else-if="trouble" class="text-warn">{{trouble}}</p>
		<p v-else class="text-fainter italic">Asking the library…</p>
		<p class="mt-8 text-strong">{{path ? backize(path) : 'Drop a file anywhere on the page'}}</p>
		<p class="mt-2">{{position || '—'}}<span v-if="paused == 'yes'" class="text-fainter"> paused</span></p>
		<p class="mt-8 flex flex-wrap gap-2"><button class="myButton" :disabled="!path" @click="play">Play</button><button class="myButton" :disabled="!path" @click="stop">Stop</button></p>
		<div class="mt-4 grid grid-cols-[auto_1fr] items-baseline gap-x-4 gap-y-2"><!-- the user's commands for the video's shape and size, here to test the region, the one in force in each row pressed; a grid, so the labels' column is as wide as the widest label in whatever face is showing, and each label sits on the line of its row's first button -->
			<span>Shape</span>
			<span class="flex flex-wrap gap-2">
				<button class="myButton" :aria-pressed="!square" :disabled="!showing" @click="shapeSquare(false)">As the file says</button>
				<button class="myButton" :aria-pressed="square" :disabled="!showing" @click="shapeSquare(true)">Square</button>
			</span>
			<span>Size</span>
			<span class="flex flex-wrap gap-2">
				<button class="myButton" :aria-pressed="sizing.unit == 'Fit'" @click="sizeChoose('Fit', '')">Fit</button>
			</span>
			<template v-for="unit in ['CSS', 'Backing']" :key="unit"><!-- each of the video's pixels on so many css pixels, or on so many backing pixels -->
				<span>{{unit}}</span>
				<span class="flex flex-wrap gap-2">
					<button v-for="(perFile, step) in sizeSteps" :key="step" class="myButton" :aria-pressed="sizing.unit == unit && sizing.step == step" @click="sizeChoose(unit, step)">{{step}}</button>
				</span>
			</template>
		</div>
		<p v-if="sizeNote" class="mt-2 text-warn">{{sizeNote}}</p>
		<div class="mt-4 grid grid-cols-[auto_1fr] gap-x-4"><!-- the three sizes, kept up to date as the window, the choice or the file changes -->
			<span>Showing in CSS pixels</span><span class="text-strong">{{showingCss}}</span>
			<span>Showing in backing pixels</span><span class="text-strong">{{showingBacking}}</span>
			<span>File pixels</span><span class="text-strong">{{filePixels}}</span>
		</div>
		<div class="mt-8">
			<p><button class="myButton" @click="ask">Open…</button></p>
			<p class="mt-2 text-strong">{{asked ? backize(asked) : 'Asks a file its running time and shape, then shows its middle frame'}}</p>
			<p class="mt-2">{{answer}}</p>
			<canvas ref="still" class="mt-4 block w-[320px]" width="0" height="0"></canvas>
		</div>
		<p v-if="trouble && version" class="mt-8 text-warn whitespace-pre-line">{{trouble}}</p><!-- every error so far, first at the top -->
	</div>

</div>

</template>
<style scoped>

.myButton { /* a button drawn as one, with an edge and a face, since the reset leaves a button as bare text, which reads as a link */
	padding: 2px 10px;
	color: var(--color-ink);
	background-color: var(--color-surface);
	border: 1px solid var(--color-line);
	cursor: pointer;
}
.myButton[aria-pressed="true"] { /* the one in force in a row of choices */
	color: var(--color-strong);
	border-color: var(--color-strong);
}
.myButton:disabled { /* nothing for it to do yet: quieter, and no hand */
	color: var(--color-fainter);
	cursor: default;
}

</style>
