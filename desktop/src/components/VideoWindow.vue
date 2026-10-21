<script setup>//fuji video: a window of its own, beside a browsing window or without one, and for now a placeholder of where the video will play, with the path of its video beneath, or its own name when v opened it, and a way to close it, the workspace the video viewer is built in a step at a time

import {ref, computed, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {getCurrentWebview} from '@tauri-apps/api/webview'
import {platform, backize, revealWindow, windowTitle, screenAreas, rectSheet} from './library.js'
import {settings, settingsLoad} from '../settings.js'
import {log, logStart, logTrouble, sayTrouble} from '../log.js'
import {themeShow, facesShow} from '../appearance.js'
import {windowFrame, windowFrameSet, windowFullscreenEnter, windowFullscreenLeave} from '../window.js'
import {menuAnswer, menuEnable} from '../menu.js'//the mac's menu bar, which sends this window Fuji Help alone, and whose Edit items this window grays
import {processOpen} from '../process.js'
import {urlHelp, brandName} from '../brand.js'
import iconMinimize from 'lucide-static/icons/minus.svg?raw'//the buttons' drawings where windows' own glyphs are not there to draw with, as lucide drew them, each a file of markup the page holds as text
import iconMaximize from 'lucide-static/icons/square.svg?raw'
import iconRestore  from 'lucide-static/icons/copy.svg?raw'
import iconClose    from 'lucide-static/icons/x.svg?raw'

/*
The root of a video window, which App.vue mounts in place of the shell when a page asked for this window with the note Video, or the operating system opened it on a video. It owns its window the way the shell owns a browsing one, and it is much smaller, because a video window is a one-off: the one thing it is handed is the video, which for now it shows by its path, it never answers, and nothing in another window waits to hear what became of it. So it reads the settings for the look and the log and never writes them, and closing it is the end of it.

The window appears at its own size every time, somewhere at random in the work area by the rule the contact sheet's window is placed by, and fuji remembers nothing about it. Its page fills it to the top edge. On the mac the title bar stays and draws nothing but its own three buttons, the real ones, over the page, which keeps the system's rounded corners and shadow at whatever radius the running macOS gives them. On windows and linux the title bar comes off and fuji draws the three buttons itself at the top right: on windows from windows' own glyphs, the four Microsoft names for an app that draws its own caption buttons, in the sizes and colors of Windows 10 or Windows 11 by the version the engine reports; on linux, which has too many desktops to copy any one of them, a plain set of fuji's own, lucide's drawings in windows' places. A hover or a press shows at once and whole, with no fade, as everything in fuji does. A drag from anywhere on the window moves it, once the pointer has moved a few pixels, so a press that stays put is still half of a double click; the drag waits because once the platform's drag begins, Windows runs the move itself and the release never reaches the page, as the audio panel found. A double click takes it into fuji's own simple fullscreen and back, by the same calls the shell's fullscreenSet makes and without its curtain, since nothing here is drawn to a size yet, and never over a macOS Space. Esc closes it, as its red button would if it had one, and so do v and the Close beside its name.

On the mac the one menu bar serves this window too. It answers Fuji Help itself and says so through menuAnswer; every other item of fuji's, Settings…, About, Open… and the View item, goes past it to the contact sheet's window most recently in front, which menu.rs brings forward, or to a new one, and the Edit items go gray while this window is in front, since it has nothing to type into.
*/

const props = defineProps({
	path: {type: String, default: ''},//the video the operating system opened this window on, forwardized; blank for one v opened from the contact sheet
})

const videoSize = {width: 800, height: 600}//the window every time it appears, in css pixels, which with no title bar is the page's own size; a size to start from, until the window takes a video's own
const titleBarAssumed = 28//a title bar's height in css pixels, where this window does not measure its own: always on linux, where the window manager adds the frame only once a window is first shown, or GTK draws it, and nobody has seen whether a hidden window counts it; and on the mac and windows when the measurement comes back with no height. The Mac's own, measured on the Mac mini with Sequoia
const frame = ref(titleBarAssumed)//the margin around this window's contents and between its screen and its text, in css pixels, as wide as the platform's own title bar, so the edge of a window without one is as deep as the edge of a window with one
const dragAfter = 4//how far the pointer moves along either axis, in css pixels with the button down, before a press is a drag
let press = null//where the left button went down, or null while it is up and once the press has become a drag
const fullscreenOurs = ref(false)//fuji's own simple fullscreen is on in this window, which tauri will not report on the mac; and while it is, fuji draws no buttons, since a fullscreen window has no title bar

const buttonLook = ref('')//the window buttons the page draws, Windows10, Windows11 or Fuji, and blank on the mac, whose title bar keeps the real ones
const focused = ref(true)//this window has the keyboard, and its buttons are drawn as the active window's
const maximized = ref(false)//this window is maximized, so the middle button shows restore
const glyphs = {minimize: String.fromCodePoint(0xE921), maximize: String.fromCodePoint(0xE922), restore: String.fromCodePoint(0xE923), close: String.fromCodePoint(0xE8BB)}//windows' own caption glyphs in its icon font, at the code points Microsoft names, characters from the private use area that no editor shows
const buttonGlyphs = computed(() => {//what each button holds, as markup: windows' own glyph on windows, and lucide's drawing everywhere else, which is every look b steps through in a development build on another platform
	let font = platform() == 'windows' && (buttonLook.value == 'Windows10' || buttonLook.value == 'Windows11')
	if (font) return {font, minimize: glyphs.minimize, maximize: maximized.value ? glyphs.restore : glyphs.maximize, close: glyphs.close}
	return {font, minimize: iconMinimize, maximize: maximized.value ? iconRestore : iconMaximize, close: iconClose}
})

let unlistenMenu, unlistenFocus, unlistenResize//will hold the unsubscribe functions set below and called as the window goes
onMounted(async () => {
	let w = getCurrentWindow()
	await menuAnswer(['menu-help']).catch(error => console.error('video: telling the menu bar what this window answers', error))//first, before the window can be in front; a top gate with no log yet, and a failure leaves this window answering every item, as a window that never says does
	let notices = []//what the settings read said, from before there was a log to put it in
	try {
		notices = await settingsLoad()//for the look and the log switch; this window changes no setting, so it never hands the file down
	} catch (error) {
		notices.push(sayTrouble('video: reading settings', error))//carry on to the reveal on factory settings rather than leave the window hidden
	}
	logStart({label: 'video', record: settings.log.record})//joins the run a browsing window started, or starts one when this is the only window
	for (let notice of notices) log(notice)
	await facesShow()//before the reveal, so the first frame is already in the faces and the colors the settings name
	await themeShow().catch(error => logTrouble('video: setting the theme', error))
	await reportTrouble(() => w.setTitle(windowTitle('Video', props.path)))//the name the Mac's Window menu and the Windows taskbar show, since there is no title bar to show it
	await reportTrouble(placeWindow)
	buttonLook.value = await buttonLookOwn()//before the reveal, so the first frame has its buttons
	log(`video: buttons ${buttonLook.value || `the title bar's own`}`)//which look, since on windows it is the engine's answer about the version
	await revealWindow()
	await reportTrouble(() => getCurrentWebview().setFocus())//the title bar went while the window was hidden, which is a change of style mask, so the keyboard is handed to the page
	window.addEventListener('keydown', onKey)
	unlistenMenu = await w.listen('menu', event => { if (event.payload == 'menu-help') reportTrouble(() => processOpen(`https://${urlHelp}`)) })//this window's own listener, which menu.rs aims Fuji Help at while this window is in front; the help address in the system's browser, as the shell opens it
	unlistenFocus = await w.onFocusChanged(event => { focused.value = event.payload; if (event.payload) editGray() })//the buttons fuji draws dim in a window behind another; and the one menu bar serves every window, so this one coming to the front grays the Edit items another window lit
	unlistenResize = await w.onResized(() => reportTrouble(async () => { maximized.value = await w.isMaximized() }))//a maximize or a restore, from the middle button or from the system, changes what that button shows
	focused.value = await w.isFocused()//and once now, since the focus arrived with the reveal, before there was anyone listening for it
	if (focused.value) editGray()
})
onBeforeUnmount(() => {
	window.removeEventListener('keydown', onKey)
	if (unlistenMenu) unlistenMenu()
	if (unlistenFocus) unlistenFocus()
	if (unlistenResize) unlistenResize()
})
function editGray() { for (let item of ['Undo', 'Redo', 'Cut', 'Copy', 'Paste', 'Select All']) menuEnable('Edit', item, false).catch(error => logTrouble('video: graying the Edit menu', error)) }//nothing here takes typing or holds a selection; the shell lights them again as its window comes to the front

async function placeWindow() {//put the hidden window where it will first be seen
	let measured = await titleBarMeasure().catch(error => { logTrouble('video: measuring the title bar', error); return 0 })//first, while the window still has its title bar to measure
	frame.value = measured > 0 ? measured : titleBarAssumed
	log(`video: frame ${frame.value} css pixels, ${measured > 0 ? 'measured from the title bar' : 'assumed'}`)//a measurement each platform answers its own way, for the log on a machine where the frame looks wrong
	let areas = await screenAreas()
	if (areas) await windowFrameSet(rectSheet(areas.work, videoSize))//the window's size, or the contact sheet's preset on a work area too small for it, at a random place, as a contact sheet is placed; no monitor to measure leaves it where the operating system put it
	if (platform() == 'mac') await getCurrentWindow().setTitleBarStyle('overlay')//the title bar kept but drawn transparent, with the page running up under it, so the traffic lights float over the page and the system still rounds the corners; after the frame, never before, for the reason placeWindow in Shell.vue gives
	else await getCurrentWindow().setDecorations(false)//elsewhere the title bar off, and fuji's own buttons in its place; after the frame too, and the page grows to fill the window
}
async function buttonLookOwn() {//the buttons this platform gets fuji's drawing of: none on the mac, windows' own on windows, by its version, and fuji's own on linux
	if (platform() == 'mac') return ''
	if (platform() == 'linux') return 'Fuji'
	let hints = await navigator.userAgentData.getHighEntropyValues(['platformVersion']).catch(error => { logTrouble('video: asking the engine which windows this is', error); return {platformVersion: ''} })//the engine's client hints, whose platform version Microsoft documents as 13 and above on Windows 11 and 1 to 10 on Windows 10, where the user agent string says 10 for both
	return parseInt(hints.platformVersion) >= 13 ? 'Windows11' : 'Windows10'
}
const development = import.meta.env.DEV//a development build, which b and the line naming the look are for, since a template cannot read import.meta
const buttonLookSaid = computed(() => {//the line under the path that names the buttons showing, and whose drawings they are
	if (!buttonLook.value) return platform() == 'mac' ? `the Mac's own buttons` : 'no buttons'
	let look = {Windows10: `Windows 10's buttons`, Windows11: `Windows 11's buttons`, Fuji: `${brandName}'s own buttons`}[buttonLook.value]
	return `${look}, in ${buttonGlyphs.value.font ? `Windows' own glyphs` : `Lucide's drawings`}`
})
function buttonLookNext() { let looks = ['', 'Windows10', 'Windows11', 'Fuji']; buttonLook.value = looks[(looks.indexOf(buttonLook.value) + 1) % looks.length] }//the next look along, with none between the last and the first, for b in a development build
async function titleBarMeasure() {//the height of the title bar this hidden window was built with, in css pixels, as its visible frame less its content, the way window_frame_set measures it; rounded to whole backing pixels, so the frame drawn at that thickness lands on the pixel grid. An answer of 0 or less is no measurement, which linux always answers, for the reason titleBarAssumed gives
	if (platform() == 'linux') return 0
	let w = getCurrentWindow()
	let [visible, inner, backingPerCss] = await Promise.all([windowFrame(), w.innerSize(), w.scaleFactor()])//the content from the window rather than the page, whose viewport reports its size only after the reveal
	return Math.round(visible.height * backingPerCss - inner.height) / backingPerCss//inner is in backing pixels already
}

function onKey(e) {
	if (e.altKey || e.ctrlKey || e.metaKey) return//chords are the system's, and on windows alt is, as the shell leaves it
	if ((e.key == 'Escape' || e.key == 'v') && !e.repeat) reportTrouble(closeWindow)//escape as the red button would, from fuji's fullscreen too, as escape closes a table's window; and v, the key that opened it from the sheet. Once for a held key, as the shell closes a table's window, so the window behind, which takes the keyboard as this one goes, never gets the repeats
	if (import.meta.env.DEV && e.key == 'b' && !e.repeat) buttonLookNext()//a development build steps through every look of the buttons fuji draws, so one machine can show another platform's, with lucide's drawings where windows' glyphs are not installed
}
async function minimizeWindow() { await getCurrentWindow().minimize() }
async function maximizeWindow() { await getCurrentWindow().toggleMaximize() }//maximized, or back to where it was; the resize that follows tells the middle button which to show
function onPress(e) { if (e.button == 0) press = e.target.closest('button') ? null : {x: e.clientX, y: e.clientY} }//a press on Close is the button's, and never starts a drag
function onMove(e) {//a press that has moved is a drag, and from here the window follows the pointer
	if (!press || (Math.abs(e.clientX - press.x) < dragAfter && Math.abs(e.clientY - press.y) < dragAfter)) return
	press = null
	getCurrentWindow().startDragging().catch(error => logTrouble('video: dragging the window', error))
}
function onDouble(e) { if (!e.target.closest('button')) reportTrouble(fullscreenToggle) }//two quick presses on Close are two presses, and the first closes the window anyway
function onContextMenu(e) { if (!import.meta.env.DEV) e.preventDefault() }//the web view's own right-click menu is a browser's, and a release build turns it away, as the shell does; a development build keeps it, for inspecting the page

async function fullscreenToggle() {//fuji's own fullscreen on or off, instant, with no Space and no animation; on windows and linux, the ordinary fullscreen
	let w = getCurrentWindow()
	if (!fullscreenOurs.value && platform() == 'mac' && await w.isFullscreen()) return//already in a macOS Space, which fuji's fullscreen never stacks on; only the mac is asked, since on windows and linux isFullscreen reports fuji's own
	fullscreenOurs.value = !fullscreenOurs.value
	if (fullscreenOurs.value) await windowFullscreenEnter()
	else await windowFullscreenLeave()//the one way fuji leaves it, which on windows never shows a hidden window
	await getCurrentWebview().setFocus()//simple fullscreen changes the style mask on the mac, which drops the keyboard, as the essay above fullscreenSet in Shell.vue says
}
async function closeWindow() {//close the window as the system's close would, hidden and out of fuji's fullscreen first, so on the mac the menu bar and the dock, which simple fullscreen hides for the whole application, come back once no window is left in it
	let w = getCurrentWindow()
	if (fullscreenOurs.value) {
		await w.hide()
		await windowFullscreenLeave(); fullscreenOurs.value = false
	}
	await w.close()
}

async function reportTrouble(work) {//a window event is where the platform starts fuji's code running, so anything thrown has nowhere to land but here
	try { await work() } catch (error) { logTrouble('video: handling a window event', error) }
}

</script>
<template>

<div class="myVideo myMono bg-paper text-strong relative w-full h-full flex flex-col overflow-hidden select-none" :style="{'--frame': `${frame}px`}" @mousedown="onPress" @mousemove="onMove" @mouseup="press = null" @mouseleave="press = null" @dblclick="onDouble" @contextmenu="onContextMenu"><!-- hidden overflow, so a window too short for its text cuts the text off rather than growing a scroll bar -->
	<div v-if="buttonLook && !fullscreenOurs" class="myButtons absolute top-0 right-0 flex" :class="[`my${buttonLook}`, {myFont: buttonGlyphs.font, myAway: !focused}]"><!-- the window's buttons where fuji draws them, at the top right over whatever the page shows there; each holds a glyph or a drawing fuji bundles, never anything from outside, which is what lets it go in as markup -->
		<button tabindex="-1" v-html="buttonGlyphs.minimize" @click="reportTrouble(minimizeWindow)"></button>
		<button tabindex="-1" v-html="buttonGlyphs.maximize" @click="reportTrouble(maximizeWindow)"></button>
		<button tabindex="-1" v-html="buttonGlyphs.close" class="myClose" @click="reportTrouble(closeWindow)"></button>
	</div>
	<div class="myStage"><div class="myScreen bg-screen mx-auto"></div></div><!-- the room for the screen, centered left to right in it -->
	<div class="myPage grow wrap-break-word">{{path ? backize(path) : 'V I D E O'}}<button class="ml-[2ch]" @click="reportTrouble(closeWindow)">Close</button><div v-if="development">{{buttonLookSaid}}</div></div><!-- the page below the screen, which takes whatever height the screen leaves, its text in plain flow from the top left, wrapping a long path at the window's width -->
</div>

</template>
<style scoped>

.myVideo {
	padding: var(--frame); /* a margin as wide as the platform's title bar, which the page sets on this element as --frame */
	container-type: size; /* so the room below can measure the window inside that margin */
	background-image: radial-gradient(circle at center, var(--color-line) 1px, transparent 1px); /* a spike, for now: small dots across the whole page, to show how far up under the title bar it reaches */
	background-size: 8px 8px;
}
.myStage {
	flex: 0 1 calc(100cqw * 9 / 16); /* as tall as a 16 by 9 screen across the window's whole width, and the one part that gives up height when the window is too short for that, the margin and the text: a size container counts as empty, so its least height is nothing, where the page's is its margin and text */
	container-type: size; /* and the screen inside measures this room */
}
.myScreen {
	width: min(100cqw, 100cqh * 16 / 9); /* the room's whole width, or 16 by 9 at the room's whole height once the room has shrunk, whichever is less, so the screen always fits */
	aspect-ratio: 16 / 9;
}
.myPage {
	padding-top: var(--frame); /* the same space again between the screen and the text */
}

.myPage button { /* Close, which the reset leaves bare: underlined, with the hand, as the empty sheet's buttons are */
	text-decoration: underline;
	cursor: pointer;
}

.myButtons button { /* each of the three window buttons, as wide as windows makes its own and as tall as the band the margin leaves across the top, its glyph in the middle and the arrow over it, as over a title bar's */
	width: 46px;
	height: var(--frame);
	display: flex;
	align-items: center;
	justify-content: center;
}
.myFont button { /* windows' own glyphs, from the icon font windows installs: Segoe Fluent Icons on Windows 11, and Segoe MDL2 Assets on Windows 10, where the same code points hold the older drawings */
	font-family: 'Segoe Fluent Icons', 'Segoe MDL2 Assets';
	font-size: 10px;
}
.myButtons :deep(svg) { /* lucide's drawings, which the page holds as their own markup, beyond the reach of a scoped rule without deep: 16 css pixels square, with a stroke of a css pixel, which is 1.5 in the drawing's own 24 */
	width: 16px;
	height: 16px;
	stroke-width: 1.5;
}

.myWindows10 button:hover,  .myFuji button:hover  { background-color: var(--color-caption-hover); } /* the gray behind minimize and maximize, Windows 10's and fuji's own */
.myWindows10 button:active, .myFuji button:active { background-color: var(--color-caption-press); }
.myWindows11 button:hover                         { background-color: var(--color-caption-hover-eleven); } /* Windows 11's, fainter */
.myWindows11 button:active                        { background-color: var(--color-caption-press-eleven); }

.myWindows10 .myClose:hover                             { background-color: var(--color-close);              color: var(--color-on-close); } /* close turns red, with its glyph white, Windows 10's red */
.myWindows10 .myClose:active                            { background-color: var(--color-close-press); }
.myWindows11 .myClose:hover,  .myFuji .myClose:hover    { background-color: var(--color-close-eleven);       color: var(--color-on-close); } /* and Windows 11's, which fuji's own take */
.myWindows11 .myClose:active, .myFuji .myClose:active   { background-color: var(--color-close-press-eleven); color: color-mix(in srgb, var(--color-on-close) 70%, transparent); } /* pressed, the white glyph dims a little too, as Windows Terminal has it */

.myAway button:not(:hover) { opacity: 0.4; } /* in a window behind another, the glyphs step back until the pointer comes over one; a button at rest has no background, so only its glyph dims */

</style>
