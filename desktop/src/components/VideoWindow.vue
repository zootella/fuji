<script setup>//fuji video: a window of its own, beside a browsing window or without one, and for now a placeholder of where the video will play, with the path of its video beneath, or its own name when v opened it, the workspace the video viewer is built in a step at a time

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {getCurrentWebview} from '@tauri-apps/api/webview'
import {platform, backize, revealWindow, windowTitle, screenAreas, rectSheet} from './library.js'
import {settings, settingsLoad} from '../settings.js'
import {log, logStart, logTrouble, sayTrouble} from '../log.js'
import {themeShow, facesShow} from '../appearance.js'
import {windowFrameSet, windowFullscreenEnter, windowFullscreenLeave, windowTitleHide, windowConstraintLift, windowDrag, windowNew} from '../window.js'
import {menuAnswer, menuEnable} from '../menu.js'//the mac's menu bar, which sends this window Fuji Help alone, and whose Edit items this window grays
import {processOpen} from '../process.js'
import {urlHelp, brandName} from '../brand.js'
import iconMinimize from 'lucide-static/icons/minus.svg?raw'//the window buttons' drawings, as lucide drew them, each a file of markup the page holds as text
import iconMaximize from 'lucide-static/icons/square.svg?raw'
import iconRestore  from 'lucide-static/icons/copy.svg?raw'
import iconClose    from 'lucide-static/icons/x.svg?raw'

/*
The root of a video window, which App.vue mounts in place of the shell when a page asked for this window with the note Video, or the operating system opened it on a video. It owns its window the way the shell owns a browsing one, and it is much smaller, because a video window is a one-off: the one thing it is handed is the video, which for now it shows by its path, it never answers, and nothing in another window waits to hear what became of it. So it reads the settings for the look and the log and never writes them, and closing it is the end of it.

The window appears at its own size every time, somewhere at random in the work area by the rule the contact sheet's window is placed by, and fuji remembers nothing about it. Its page fills it to the top edge, and no title is drawn there, since text above a video pulls the eye from it. On the mac the title bar stays, invisible, holding the real traffic lights over the page, which keeps the system's rounded corners and shadow at whatever radius the running macOS gives them. On windows and linux the title bar comes off. Fuji draws its own minimize, maximize and close at the top right, on the mac too while buttonsOnMac says so: lucide's drawings in Windows 10's places, sizes and colors, the same on every platform, since fuji would rather look like itself than like a slightly different copy of one platform's. Those three and nothing the platforms have added around them, like the arrangements Windows 11 shows over maximize and the Mac over the green button, which are for setting one document beside another more than for watching a video. A hover or a press shows at once and whole, with no fade, as everything in fuji does. A drag from anywhere on the window moves it, since a window whose top has gone past the screen's edge has no title bar left to grab. The drag begins once the pointer has moved a few pixels, so a press that stays put is still half of a double click, and from then the mouse is the drag's until the button comes up, and the release never reaches the page. On the Mac and Windows fuji runs that drag itself, placing the window on every move, which lets it go past the top of the screen where the platforms' own drags stop, as window.rs says; on Linux the desktop's own drag runs it. A double click takes it into fuji's own simple fullscreen and back, by the same calls the shell's fullscreenSet makes and without its curtain, since nothing here is drawn to a size yet, and never over a macOS Space. Esc closes it, as its close button does, and so does v.

On the mac the one menu bar serves this window too. It answers Fuji Help itself and says so through menuAnswer; every other item of fuji's, Settings…, About, Open… and the View item, goes past it to the contact sheet's window most recently in front, which menu.rs brings forward, or to a new one, and the Edit items go gray while this window is in front, since it has nothing to type into.
*/

const props = defineProps({
	path: {type: String, default: ''},//the video the operating system opened this window on, forwardized; blank for one v opened
})

const buttonsOnMac = true//fuji draws its own three buttons on the mac as well, beside the real traffic lights, while they are being built and looked at; false leaves the mac its own alone
const buttonsShow = platform() != 'mac' || buttonsOnMac//this window draws fuji's buttons: always on windows and linux, which have no title bar left to hold any

/*
How tall a title bar is, which the video window answers with one number on every platform.

The page draws the window's top band itself: the three buttons fill it, and the same width again is the margin around everything below, so the window's edge is as deep as a title bar would make it. The platforms each answer the height differently. A Mac's standard title bar is 28 points on Sequoia, measured on the Mac mini, and 32 on Tahoe by developers' reports, which also find its layout following the SDK an application is built with; nobody has published Golden Gate's. A Windows 10 title bar is 31 backing pixels at 100 percent, its top 8 the resize border painted as the bar's upper band, measured on the Windows box, and Microsoft documents 32 for the standard title bar of its WinUI apps on Windows 11. Linux has no one answer: libadwaita's header bars, which Ubuntu's and GNOME's own apps draw, are 46 or 47, and Ubuntu's older GTK3 apps sit in about 28.

Fuji could match each one. On the Mac and Windows a window built with an ordinary title bar can measure it, as its visible frame less its content, before the bar goes invisible or comes off, and the video window once did, writing what it found to the log; Linux will not say, since the window manager frames a window only once it is shown, or the application draws its own. One number serves as well with less code. 32 is within 4 of every platform's own but libadwaita's, and fuji's buttons are its own drawings rather than a copy of any desktop's. The Mac's traffic lights stay where AppKit puts them, whatever the band around them measures. And 32 lands on whole backing pixels at every scale Windows offers, 40 at 125 percent, 48 at 150 and 56 at 175, where 31 would land on fractions.
*/
const titleBar = 32//the band across the top in css pixels, which the buttons fill, and the margin around everything below it

const placeholderName = [...`${brandName}video`.toUpperCase()].join(' ')//what the page shows beneath the screen while it holds no video, the product's name and the word a letter at a time, F U J I V I D E O
const videoSize = {width: 800, height: 600}//the window every time it appears, in css pixels, which with the page running to the top edge is the page's own size; a size to start from, until the window takes a video's own
const dragAfter = 4//how far the pointer moves along either axis, in css pixels with the button down, before a press is a drag
let press = null//where the left button went down, or null while it is up and once the press has become a drag
const fullscreenOurs = ref(false)//fuji's own simple fullscreen is on in this window, which tauri will not report on the mac; and while it is, fuji draws no buttons, since a fullscreen window has no title bar
const focused = ref(true)//this window has the keyboard, and its buttons are drawn as the active window's
const maximized = ref(false)//this window is maximized, so the middle button shows restore

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
	await reportTrouble(() => w.setTitle(windowTitle('Video', props.path)))//the name the Mac's Window menu and the Windows taskbar show, since the window draws no title of its own
	await reportTrouble(placeWindow)
	await revealWindow()
	await reportTrouble(() => getCurrentWebview().setFocus())//the title bar changed while the window was hidden, which is a change of style mask, so the keyboard is handed to the page
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

/*
The edge a Windows 10 window keeps on three sides once its title bar is gone, and why fuji leaves it.

On Windows the title bar comes off through setDecorations, and the window keeps its shadow, which is Tauri's default. With the shadow kept, tao still counts the left, the right and the bottom as frame, the resize border about 8 backing pixels deep at 100 percent, and Windows draws its border line there, which Tauri's documentation for set_shadow calls a 1px white border, and which showed dark on the Windows box. At the top tao leaves no frame on Windows 10, because there any frame at the top makes Windows draw a whole title bar, as calculate_insets_for_dpi in tao 0.35.3 says. So on Windows 10 the window's edge runs down both sides and along the bottom and not across the top, seen on the Windows box on 2026-10-22. On Windows 11 tao keeps a pixel of frame at the top as well, so the border goes all the way round, with the rounded corners Windows 11 gives a window with a shadow; that is read in the same function and not yet seen, since there is no Windows 11 box.

Five answers were weighed, and only leaving it passes every test a change here has to: easy, more correct than what it replaces, nothing changed off Windows, and nothing put at risk on Windows 11. Turning the shadow off takes the line away, and takes the shadow with it, and on Windows 11 the rounded corners too. Drawing a matching line across the top in the page would copy a color Windows chooses from the theme, the accent color setting and whether the window is in front, and be wrong whenever one of those changes. Handling the frame message in fuji's own code, as Chromium does, would fight tao's handling of the same message, and risk Windows 11's corners and its snap layouts on a platform fuji has no box to test. And taking the line off all four sides, rather than adding the fourth, has no switch on Windows 10: it means turning the shadow off, which leaves tao no frame and Windows no line, and calling DwmExtendFrameIntoClientArea with a pixel of margin so the window manager draws the shadow again. That moves Tauri's resize band from the top alone to all four edges inside the window, over the page and fuji's buttons, as set_drag_hwnd_rgn in tauri-runtime-wry 2.11.4 lays it; grows the window by its invisible border, on both sides and along the bottom, the moment the title bar comes off, since window_frame_set in window.rs places the frame a window shows while it still has its title bar; wants a Rust command of its own; and leaves unknown whether Windows 11 still rounds such a window's corners. The line is Windows' own border, so the window reads as a Windows 10 window without its title bar, which is what it is. The preview loses its title bar the same way, in Shell.vue, and keeps the same edge.

Windows 11 alone has the switch Windows 10 lacks. DwmSetWindowAttribute with DWMWA_BORDER_COLOR set to DWMWA_COLOR_NONE takes the line off all four sides and keeps the shadow and the rounded corners, from build 22000, and does nothing on Windows 10. It is one call, and it would make Windows 11 differ from Windows 10 in a way nobody has seen, so it is the step to take once there is a Windows 11 box to look at it on, rather than one to ship unseen.
*/
async function placeWindow() {//put the hidden window where it will first be seen
	let w = getCurrentWindow()
	let areas = await screenAreas()
	if (areas) await windowFrameSet(rectSheet(areas.work, videoSize))//the window's size, or the contact sheet's preset on a work area too small for it, at a random place, as a contact sheet is placed; no monitor to measure leaves it where the operating system put it
	if (platform() == 'mac') {
		await w.setTitleBarStyle('overlay')//the title bar kept but drawn transparent, with the page running up under it, so the traffic lights float over the page and the system still rounds the corners; after the frame, never before, for the reason placeWindow in Shell.vue gives
		await windowTitleHide()//and the title in it not drawn, though the title stays set
		await windowConstraintLift()//AppKit no longer holds this window below the menu bar or to the screen's height, so where fuji places it, it stays; the mac's own drag still stops at the menu bar, as window.rs says
	} else {
		await w.setDecorations(false)//elsewhere the title bar off, and fuji's own buttons in its place; after the frame too, and the page grows to fill the window, inside the edge the essay above says Windows 10 keeps on three sides
	}
}

function onKey(e) {
	if (e.altKey || e.ctrlKey || e.metaKey) return//chords are the system's, and on windows alt is, as the shell leaves it
	if (e.key == 'v' && !e.repeat) reportTrouble(() => windowNew('Video'))//v asks for another video window, as it does in a browsing window, and this one carries on; nothing here ever goes back to a browsing window
	if (e.key == 'Escape' && !e.repeat) reportTrouble(closeWindow)//escape as the close button does, from fuji's fullscreen too, as escape closes a table's window. Once for a held key, as the shell closes a table's window, so the window behind, which takes the keyboard as this one goes, never gets the repeats
}
async function minimizeWindow() { await getCurrentWindow().minimize() }
async function maximizeWindow() { await getCurrentWindow().toggleMaximize() }//maximized, or back to where it was; the resize that follows tells the middle button which to show
function onPress(e) { if (e.button == 0) press = e.target.closest('button') ? null : {x: e.clientX, y: e.clientY} }//a press on a button is the button's, and never starts a drag
function onMove(e) {//a press that has moved is a drag, and from here the window follows the pointer; never in fuji's fullscreen, which fills the screen
	if (!press || fullscreenOurs.value || (Math.abs(e.clientX - press.x) < dragAfter && Math.abs(e.clientY - press.y) < dragAfter)) return
	press = null
	windowDrag().catch(error => logTrouble('video: dragging the window', error))
}
function onDouble(e) { if (!e.target.closest('button')) reportTrouble(fullscreenToggle) }//two quick presses on a button are two presses, of that button
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

<div class="myVideo myMono bg-zed-editor-background text-zed-text relative w-full h-full flex flex-col overflow-hidden select-none" :style="{'--title-bar': `${titleBar}px`}" @mousedown="onPress" @mousemove="onMove" @mouseup="press = null" @mouseleave="press = null" @dblclick="onDouble" @contextmenu="onContextMenu"><!-- hidden overflow, so a window too short for its text cuts the text off rather than growing a scroll bar -->
	<div v-if="buttonsShow && !fullscreenOurs" class="myButtons absolute top-0 right-0 flex" :class="{myAway: !focused}"><!-- the window's buttons fuji draws, at the top right over whatever the page shows there; each holds a drawing fuji bundles, never anything from outside, which is what lets it go in as markup -->
		<button tabindex="-1" aria-label="Minimize" v-html="iconMinimize" @click="reportTrouble(minimizeWindow)"></button><!-- named for a screen reader, since a drawing says nothing to one, with the words Windows gives its own -->
		<button tabindex="-1" :aria-label="maximized ? 'Restore' : 'Maximize'" v-html="maximized ? iconRestore : iconMaximize" @click="reportTrouble(maximizeWindow)"></button>
		<button tabindex="-1" aria-label="Close" v-html="iconClose" class="myClose" @click="reportTrouble(closeWindow)"></button>
	</div>
	<div class="myStage"><div class="myScreen mx-auto"></div></div><!-- the room for the screen, centered left to right in it -->
	<div class="myPage grow wrap-break-word">{{path ? backize(path) : placeholderName}}</div><!-- the page below the screen, which takes whatever height the screen leaves, its text in plain flow from the top left, wrapping a long path at the window's width -->
</div>

</template>
<style scoped>

.myVideo {
	padding: var(--title-bar); /* a margin as wide as the band across the top, which the page sets on this element */
	container-type: size; /* so the room below can measure the window inside that margin */
	background-image: radial-gradient(circle at center, var(--color-zed-border) 1px, transparent 1px); /* a spike, for now: small dots across the whole page, to show how far up under the title bar it reaches */
	background-size: 8px 8px;
}
.myStage {
	flex: 0 1 calc(100cqw * 9 / 16); /* as tall as a 16 by 9 screen across the window's whole width, and the one part that gives up height when the window is too short for that, the margin and the text: a size container counts as empty, so its least height is nothing, where the page's is its margin and text */
	container-type: size; /* and the screen inside measures this room */
}
.myScreen {
	width: min(100cqw, 100cqh * 16 / 9); /* the room's whole width, or 16 by 9 at the room's whole height once the room has shrunk, whichever is less, so the screen always fits */
	aspect-ratio: 16 / 9;
	background-color: black; /* a placeholder for now, where the video will play */
	box-shadow: 4px 4px 12px var(--color-fuji-shade-deep); /* the shape of the shadow a thumbnail on the sheet casts, lit from the top left, square as the screen is, and darker; painted outside the box, which the video will fill to the pixel, and where the video's own native view will not cover it */
}
.myPage {
	padding-top: var(--title-bar); /* the same space again between the screen and the text */
}

.myButtons button { /* each of the three window buttons, as tall as the band across the top and as wide as Windows 10 makes its own: Microsoft publishes no width, and 46 is the one Firefox and Windows Terminal both draw theirs at. Its drawing in the middle, and the arrow over it, as over a title bar. On Windows the top few pixels of each are the window's top edge rather than the button, Tauri's resize band, SM_CYFRAME deep at the window's DPI: 4 backing pixels at 100 and 125 percent and 5 at 150 and 175, read on the Windows box on 2026-10-22, where Windows' own buttons leave 1. Seen and left, since Tauri takes the band away from a maximized window, where buttons in the screen's corner matter most */
	width: 46px;
	height: var(--title-bar);
	display: flex;
	align-items: center;
	justify-content: center;
}
.myButtons :deep(svg) { /* lucide's drawings, which the page holds as their own markup, beyond the reach of a scoped rule without deep: 16 css pixels square, with a stroke of a css pixel, which is 1.5 in the drawing's own 24 */
	width: 16px;
	height: 16px;
	stroke-width: 1.5;
}
.myButtons button:hover    { background-color: color-mix(in srgb, var(--color-fuji-window-button) 10%, transparent); } /* the gray behind minimize and maximize, on every platform */
.myButtons button:active   { background-color: var(--color-fuji-dayglo); color: var(--color-fuji-purple); } /* and pressed, minimize, maximize and close alike, a moment of purple on day-glo so the click is seen to land */
.myButtons .myClose:hover  { background-color: var(--color-fuji-close-red); color: var(--color-fuji-on-close-red); } /* and its bright red behind close, the drawing white on it */
.myButtons .myClose:active { background-color: var(--color-fuji-dayglo); color: var(--color-fuji-purple); }
.myAway button:not(:hover) { opacity: 0.4; } /* in a window behind another, the drawings step back until the pointer comes over one; a button at rest has no background, so only its drawing dims */

</style>
