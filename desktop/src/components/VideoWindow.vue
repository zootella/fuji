<script setup>//fuji video: a window of its own, beside a browsing window or without one, and for now its name in the middle and a way to close it, the workspace the video viewer is built in a step at a time

import {onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {getCurrentWebview} from '@tauri-apps/api/webview'
import {platform, revealWindow, windowTitle, screenAreas, rectSheet} from './library.js'
import {settings, settingsLoad} from '../settings.js'
import {log, logStart, logTrouble, sayTrouble} from '../log.js'
import {themeShow, facesShow} from '../appearance.js'
import {windowFrameSet, windowFullscreenLeave} from '../window.js'
import {menuAnswer, menuEnable} from '../menu.js'//the mac's menu bar, which sends this window Fuji Help alone, and whose Edit items this window grays
import {processOpen} from '../process.js'
import {urlHelp} from '../brand.js'

/*
The root of a video window, which App.vue mounts in place of the shell when a page asked for this window with the note Video. It owns its window the way the shell owns a browsing one, and it is much smaller, because a video window is a one-off: the one thing it will be handed is what to play, it never answers, and nothing in another window waits to hear what became of it. So it reads the settings for the look and the log and never writes them, and closing it is the end of it.

The window appears at its own size every time, somewhere at random in the work area by the rule the contact sheet's window is placed by, and fuji remembers nothing about it. It has no title bar, so the page fills it, and a drag from anywhere on it moves it, once the pointer has moved a few pixels, so a press that stays put is still half of a double click; the drag waits because once the platform's drag begins, Windows runs the move itself and the release never reaches the page, as the audio panel found. A double click takes it into fuji's own simple fullscreen and back, by the same calls the shell's fullscreenSet makes and without its curtain, since nothing here is drawn to a size yet, and never over a macOS Space. Esc closes it, as its red button would if it had one, and so do v and the Close beside its name.

On the mac the one menu bar serves this window too. It answers Fuji Help itself and says so through menuAnswer; every other item of fuji's, Settings…, About, Open… and the View item, goes past it to the contact sheet's window most recently in front, which menu.rs brings forward, or to a new one, and the Edit items go gray while this window is in front, since it has nothing to type into.
*/

const videoSize = {width: 800, height: 600}//the window every time it appears, in css pixels, which with no title bar is the page's own size; a size to start from, until the window takes a video's own
const dragAfter = 4//how far the pointer moves along either axis, in css pixels with the button down, before a press is a drag
let press = null//where the left button went down, or null while it is up and once the press has become a drag
let fullscreenOurs = false//fuji's own simple fullscreen is on in this window, which tauri will not report on the mac

let unlistenMenu, unlistenFocus//will hold the unsubscribe functions set below and called as the window goes
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
	await reportTrouble(() => w.setTitle(windowTitle('Video')))//the name the Mac's Window menu and the Windows taskbar show, since there is no title bar to show it
	await reportTrouble(placeWindow)
	await revealWindow()
	await reportTrouble(() => getCurrentWebview().setFocus())//the title bar went while the window was hidden, which is a change of style mask, so the keyboard is handed to the page
	window.addEventListener('keydown', onKey)
	unlistenMenu = await w.listen('menu', event => { if (event.payload == 'menu-help') reportTrouble(() => processOpen(`https://${urlHelp}`)) })//this window's own listener, which menu.rs aims Fuji Help at while this window is in front; the help address in the system's browser, as the shell opens it
	unlistenFocus = await w.onFocusChanged(event => { if (event.payload) editGray() })//the one menu bar serves every window, so this one coming to the front grays the Edit items another window lit
	if (await w.isFocused()) editGray()//and once now, since the focus arrived with the reveal, before there was anyone listening for it
})
onBeforeUnmount(() => {
	window.removeEventListener('keydown', onKey)
	if (unlistenMenu) unlistenMenu()
	if (unlistenFocus) unlistenFocus()
})
function editGray() { for (let item of ['Undo', 'Redo', 'Cut', 'Copy', 'Paste', 'Select All']) menuEnable('Edit', item, false).catch(error => logTrouble('video: graying the Edit menu', error)) }//nothing here takes typing or holds a selection; the shell lights them again as its window comes to the front

async function placeWindow() {//put the hidden window where it will first be seen
	let areas = await screenAreas()
	if (areas) await windowFrameSet(rectSheet(areas.work, videoSize))//the window's size, or the contact sheet's preset on a work area too small for it, at a random place, as a contact sheet is placed; no monitor to measure leaves it where the operating system put it
	await getCurrentWindow().setDecorations(false)//the title bar off after the frame, never before, for the reason placeWindow in Shell.vue gives; a window keeps its frame as its title bar goes, and the page grows to fill it
}

function onKey(e) {
	if (e.altKey || e.ctrlKey || e.metaKey) return//chords are the system's, and on windows alt is, as the shell leaves it
	if (e.key == 'Escape' || e.key == 'v') reportTrouble(closeWindow)//escape as the red button would, from fuji's fullscreen too, as escape closes a table's window; and v, the key that opened it from the sheet
}
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
	if (!fullscreenOurs && platform() == 'mac' && await w.isFullscreen()) return//already in a macOS Space, which fuji's fullscreen never stacks on; only the mac is asked, since on windows and linux isFullscreen reports fuji's own
	fullscreenOurs = !fullscreenOurs
	if (fullscreenOurs) await w.setSimpleFullscreen(true)
	else await windowFullscreenLeave()//the one way fuji leaves it, which on windows never shows a hidden window
	await getCurrentWebview().setFocus()//simple fullscreen changes the style mask on the mac, which drops the keyboard, as the essay above fullscreenSet in Shell.vue says
}
async function closeWindow() {//close the window as the system's close would, hidden and out of fuji's fullscreen first, so on the mac the menu bar and the dock, which simple fullscreen hides for the whole application, come back for the windows left
	let w = getCurrentWindow()
	if (fullscreenOurs) {
		await w.hide()
		await windowFullscreenLeave(); fullscreenOurs = false
	}
	await w.close()
}

async function reportTrouble(work) {//a window event is where the platform starts fuji's code running, so anything thrown has nowhere to land but here
	try { await work() } catch (error) { logTrouble('video: handling a window event', error) }
}

</script>
<template>

<div class="myVideo myMono bg-paper text-strong w-full h-full flex items-center justify-center gap-[2ch] select-none" @mousedown="onPress" @mousemove="onMove" @mouseup="press = null" @mouseleave="press = null" @dblclick="onDouble" @contextmenu="onContextMenu">
	<span>V I D E O</span>
	<button @click="reportTrouble(closeWindow)">Close</button>
</div>

</template>
<style scoped>

.myVideo button { /* Close, which the reset leaves bare: underlined, with the hand, as the empty sheet's buttons are */
	text-decoration: underline;
	cursor: pointer;
}

</style>
