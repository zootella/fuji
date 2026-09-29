<script setup>//owns the window; draws nothing

import {ref, watch, nextTick, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {getCurrentWebview} from '@tauri-apps/api/webview'
import {open as openDialog} from '@tauri-apps/plugin-dialog'//the picker behind File, Open; the plugin is registered in lib.rs and granted in capabilities/default.json
import {raf, forwardize, platform, revealWindow, windowTitle, screenAreas, pointerPosition, rectPreview, rectSheet} from './library.js'
import {settings, settingsLoad, settingsChanged} from '../settings.js'
import {modelStart, modelShowing, modelPath, modelFolder} from '../model.js'//the sort comes out of the settings file the same way the table below does; which view is showing lives in the model so a flow can wait on it; the path and the folder are here for the title bar, which is the shell's because the window is
import {log, logStart, logTrouble, sayTrouble} from '../log.js'//the log belongs to the run rather than to any one view, and the run is what the shell owns
import {openFiles} from '../open.js'//the pictures the operating system handed fuji, when the user got here by double-clicking one
import {associateStart} from '../associate.js'//and what fuji tells the operating system it can open in return
import {touchBlock} from '../touch.js'//and whether a trackpad's scrolls reach this window at all, which depends on which view is showing
import {gamma, gammaToggle, gammaStep} from '../gamma.js'//the lens every picture is shown through, which the shell draws and its keys step, and a table can wheel and drag
import {cacheNeed, cacheRelease} from '../cache.js'//only to hold a picture across the swap from the preview to the diamond table, which neither table can do for itself
import {windowFrame, windowFrameSet, windowFullscreenLeave} from '../window.js'//to place the window before it is revealed, to read the size the user has given the sheet, and to leave fullscreen without showing a hidden window
import HelpPanel from './HelpPanel.vue'
import Sheet from './Sheet.vue'
import SettingsPanel from './SettingsPanel.vue'
import DiamondTable from './DiamondTable.vue'
import PreviewTable from './PreviewTable.vue'
import ComicTable from './ComicTable.vue'
import MyFlip from './MyFlip.vue'
import MyLens from './MyLens.vue'
import MyList from './MyList.vue'
import MySpace from './MySpace.vue'

/*
The shell owns the window and none of the pixels. It reads the settings file, places and reveals the window rust built, keeps the title bar saying what the user is looking at, holds the one listener for each window event, and puts the window in fullscreen or out of it as the view changes. It has no background and no chrome, so a view never has to negotiate with a parent about how it looks. The one thing it draws is the help panel, floating over whichever view is showing, because h has to work everywhere in fuji and a panel each view drew for itself would be a key each new table had to remember.

It exists because window events are global and everything else is not. A view's wheel, pointer, and double-click handlers live on its own element, so a hidden view is handed none of them and two views cannot collide. But window.addEventListener fires no matter what is visible, and so does a tauri window event, so keydown, resize, and drag-drop are the entire interference surface between views. One listener each lives here and gives the event to the view that is showing. A hidden view cannot react to a key because it is never given one, rather than because it remembered to check.

There is one sheet and there are several tables, and the two facts are separate: whether the sheet is showing, and which table is behind it. The sheet and the current table swap with v-show and both stay mounted, because that switch is frequent and has to be instant with nothing reloading. Tables swap with each other by :is, which destroys and creates, because a table nobody is using should not be holding decoded images. Adding a table is one entry in the tables object below.

The settings panel is the third kind of view, and it belongs to the sheet's window rather than to the table's fullscreen: s on the sheet brings it and s on the panel goes back. Trading those two changes nothing about the window, so it needs none of the curtain the essay further down describes. The panel comes and goes with v-if, made fresh from the settings each time, because a user visits it rarely and it holds nothing worth keeping.

Startup runs in one order for one reason: a view cannot measure itself until the window is real and it is on screen. Rust builds the window hidden, the shell places it, and the viewport does not report its size until a frame after the reveal. A view that is not showing measures nothing at all, because v-show is display none and that destroys the layout box. Vue also runs a child's onMounted before its parent's, so a view cannot do this for itself from down there. Hence the contract: a view exposes start(), the shell calls it each time that view comes on screen, and the view does only once whatever must happen only once. The sheet leans on the repeat, rereading the setting the settings panel may have changed while it was hidden.

Every handoff below is optional, start included. A view answers only the calls it has a use for, which is what lets a retired experiment be listed among the tables and shown without first being taught the contract. A real table that forgot start() would measure nothing rather than throw, which is the price.

The settings read is wrapped because the reveal below must happen either way. A window that never appears is an application with no way to tell anyone what went wrong, which is the same reason revealWindow shows the window from a finally.

The log starts as early as the settings allow, which is the moment the file has been read, because that file is what says whether to keep one at all. Everything after that line reports to it in the ordinary way. The only lines that cannot are the ones the settings read makes about itself, so settingsLoad hands those back and they go in here.
*/

const tables = {//everything the shell can show in place of a table; view.table in fuji.toml names one, so trying another is an edit to that file rather than to this one
	Diamond: DiamondTable,
	Comic:   ComicTable,
	Preview: PreviewTable,//what a launch on a picture shows first, chosen here rather than in fuji.toml

	//retired experiments from before the shell existed, kept runnable rather than only readable. Flip and Space add their own window listeners, from when a table owned its events, so while one of those is showing a key reaches it twice — once from here and once from itself. Harmless for looking at them, and the reason not to build anything new on one
	Flip:  MyFlip,//data url path and img triad
	Lens:  MyLens,//img tag with gamma and pixelated
	List:  MyList,//the images in a dragged folder, and the panel's real resolution from rust
	Space: MySpace,//pink polka dots
}

const sheetRef    = ref(null)
const settingsRef = ref(null)
const tableRef    = ref(null)
const showing     = modelShowing//Sheet, Settings or Table: which kind of view the user is looking at; the model's ref, written only here
const whichTable  = ref('Diamond')//which table is behind the sheet, whether or not it is the one showing
const helpShowing = ref(false)//the help panel, over every view; hidden until the settings say otherwise, so one the user closed never flashes up before they are read

onMounted(async () => {
	let w = getCurrentWindow()
	let notices = []//everything worth saying from before the log existed, which is the settings read and the table name it may have repaired
	try {
		notices = await settingsLoad()//before the reveal, because the window's size comes out of the file
	} catch (error) {
		notices.push(sayTrouble('shell: reading settings', error))//carry on to the reveal on factory settings rather than leave the window hidden
	}
	let opened = []//the pictures the operating system handed fuji, which is empty on an ordinary launch
	try {
		opened = (await openFiles()).map(forwardize)//forwardized here, at the same boundary a dropped path crosses; windows hands these over with backslashes and everything below assumes forward ones
	} catch (error) {
		notices.push(sayTrouble('shell: asking what fuji was opened with', error))//the same reasoning as above: nothing here is worth leaving the window hidden for
	}
	showing.value = opened.length ? 'Table' : 'Sheet'//before the reveal: a double-clicked picture opens on its preview, and every other launch on the contact sheet, since the sheet is the view that lives in a window
	whichTable.value = settings.view.table
	helpShowing.value = settings.hud.help//on at the factory, so a new user is greeted by it
	if (!tables[whichTable.value]) {//a name settings cannot check, because the tables fuji has are known here and not there
		notices.push(`settings: no table named ${whichTable.value}, showing Diamond instead`)
		whichTable.value = 'Diamond'
		settings.view.table = whichTable.value; settingsChanged()//written back, so a name fuji cannot use is repaired in the file the same way a bad value anywhere else in it is
	}
	if (opened.length) whichTable.value = 'Preview'//a double-clicked picture opens alone, fitted to the desktop, before any table the user has to learn; not written back, because opening one picture is not a choice of table
	logStart({label: `${whichTable.value.toLowerCase()}-${settings.flip.back}x${settings.flip.forward}`, record: settings.log.record, notes: [`flip.back ${settings.flip.back}, flip.forward ${settings.flip.forward}`]})//once, naming the run for the table and window it started with; the store reports loads from every view into this one file
	for (let notice of notices) log(notice)//the lines from before there was a log to put them in, first in the file and in the order they happened
	modelStart()//before any view is shown, so the first folder opened is already in the order the file names
	await nextTick()//let vue place the right view before the window appears
	await reportTrouble(() => placeWindow(w, opened[0]))//before the reveal, so the window first appears where it will stay. Only the first picture, because one window shows one picture; a picture opened later gets a window of its own — on the mac inside this same process, and on windows as a whole second fuji the shell starts

	await revealWindow()
	await raf()//the window is up; let the viewport report its dimensions before the view measures them
	activeView()?.start?.()
	associateStart().then(line => { if (line) log(line) }).catch(error => logTrouble('shell: registering what fuji can open', error))//after the reveal, so registering can never be the reason the window is slow to appear; the line is blank on a platform or a copy with nothing to do, and only an installed copy on windows has anything to say

	window.addEventListener('keydown', onKey)
	window.addEventListener('resize', onResize)
	unlistenMenu = await w.listen('menu', event => reportTrouble(() => menuChose(event.payload)))//this window's own listener rather than the global one, and that is load-bearing: listen() from the api registers for any target at all, so every window would answer a menu item meant for the one in front — which it did, opening a file picker per window. w.listen registers this window's label, which is what rust aims the event at. menu.rs sends only the items the page owns, and only to the window in front
	unlistenFileDrop = await w.onDragDropEvent(event => {
		if (event.payload.type == 'drop' && event.payload.paths.length) reportTrouble(() => viewOpen(forwardize(event.payload.paths[0])))//forwardized here, at the boundary where a path enters fuji
	})
	unlistenResized = await w.onResized(() => reportTrouble(recordSheet))//the sheet's size, into settings as the user changes it
	unlistenFocus = await w.onFocusChanged(event => reportTrouble(() => activeView()?.onFocus?.(event.payload)))//a window event like the rest, handed to the view showing; the preview closes on losing it
	reportTrouble(async () => activeView()?.onFocus?.(await w.isFocused()))//and once now, since the focus arrived with the reveal, before there was anyone listening for it
})
let unlistenFileDrop, unlistenMenu, unlistenResized, unlistenFocus//will hold the unsubscribe functions set above and called below
onBeforeUnmount(() => {
	window.removeEventListener('keydown', onKey)
	window.removeEventListener('resize', onResize)
	if (unlistenFileDrop) unlistenFileDrop()
	if (unlistenMenu) unlistenMenu()
	if (unlistenResized) unlistenResized()
	if (unlistenFocus) unlistenFocus()
})

//the title bar follows what the user is looking at: the picture on a table, the folder on the sheet. library.js composes the string, including the one place fuji differs by platform
watch([showing, modelPath, modelFolder], () => {
	getCurrentWindow().setTitle(windowTitle(showing.value, modelPath.value, modelFolder.value))
		.catch(error => logTrouble('shell: setting the window title', error))
}, {immediate: true})

//a trackpad or a magic mouse reaches the page as a stream of wheel events, and a table would read every one as a flip; rust drops them before the page sees them while a table is showing, and lets them through while the sheet is, because the sheet scrolls by them. Immediate, so the window has said which before it is revealed; touch.rs is the whole of it, and does nothing off the mac
watch(showing, value => touchBlock(value == 'Table').catch(error => logTrouble('shell: blocking touch', error)), {immediate: true})

async function menuChose(id) {//the page's half of the menu bar: rust makes a window itself and sends these two down, because the page already knows how to do both
	if (id == 'menu-open') {
		let chosen = await openDialog({multiple: false, directory: false})//every file, deliberately unfiltered: a folder is easier to recognise by everything in it, a filtered list is harder to read, and a picture saved without an extension would be hidden by a filter. Choosing something fuji cannot show is harmless — the model lists the folder and stands on the first picture in it
		if (chosen) await viewOpen(forwardize(chosen))//the same road a dropped file takes, and a double-clicked one ends on the same call: three ways in, one road after that
	}
	else if (id == 'menu-fullscreen') await toggleView()//fuji's own fullscreen rather than macOS's, which is the table: the essay above fullscreenSet says why there are two and how they keep out of each other's way
}

function helpToggle() {
	helpShowing.value = !helpShowing.value
	settings.hud.help = helpShowing.value; settingsChanged()//the setting records where the user left the panel, so help that greeted a new user stays gone once they close it
}

function activeView() { return {Sheet: sheetRef, Settings: settingsRef, Table: tableRef}[showing.value].value }//the view on screen, which every window event goes to

async function viewOpen(path) {//a picture dropped on the window or chosen with File, Open, for the view on screen to show; optional, because a view answers only the calls it has a use for
	if (showing.value == 'Settings') await showView('Sheet')//the settings panel opens nothing, so the sheet comes back to take it
	await activeView()?.onDrop?.(path)
}

function onKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//a keystroke into a form field belongs to the field; this is the only keydown listener in fuji, so this is the only place the guard is needed
	if (e.key == 'Escape' && showing.value == 'Table') { reportTrouble(closeWindow); return }//the shell's own key, never passed down: escape on any table, preview included, closes the window, as the red button or the × would
	if (e.key == 'h') { helpToggle(); return }//and this one, so a user who is lost can always ask, whatever is showing
	if (e.key == 'g') { gammaToggle(); return }//and this one, because gamma is a way of looking at every view at once rather than something one of them does
	if (e.key == '+' && e.shiftKey) { gammaStep(settings.gamma.step); return }//shift and the plus key; on the main row that key's face is =, and shift is what types + there, so the unshifted = is left to the table as zoom in
	if (e.key == '_' || (e.key == '-' && e.shiftKey)) { gammaStep(-settings.gamma.step); return }//shift and minus, which the main row types as an underscore and the number pad as a minus with shift held
	reportTrouble(() => activeView()?.onKey?.(e))
}

/*
Gamma is a lens over every picture fuji shows, and it touches none of their pixels. The filter below is one SVG primitive, feComponentTransfer, whose gamma type computes out = in to the power of the exponent on each channel scaled 0 to 1, so black stays black and white stays white while the shadows lift. CSS points the sheet's tiles and the table's image at it through one custom property on the root element, so a change of gamma is a single style change the engine applies to canvases and imgs alike: the canvases keep what the operating system handed them, the store keeps its decode, and nothing is read, drawn or decoded again. gamma.js holds the number and says what changes it.

There are two filters rather than one, and they take turns. WebKit does not redraw an element when a filter it is already showing through changes underneath it: a drag that rewrote the one filter's exponent moved the number on the hud and left the picture where it was, until leaving fullscreen forced a fresh draw and the right gamma appeared. So a change writes its exponent into the filter nothing is using, then points the pictures at that one. The property's value is different every time, and a different filter value is something every engine has to rebuild for.

Off is no filter at all rather than an exponent of 1, so the pictures at rest are exactly what the thumbnail pipeline document on fuji's site measured. The filters run in sRGB rather than the linearRGB an SVG filter defaults to: a power curve comes out nearly the same in either space, because powers compose, and staying in sRGB spares the round trip to linear light, which at eight bits would merge the very shadow codes this exists to pull apart.
*/
let gammaFilter = 0//which of the two filters below the pictures are pointed at
watch(gamma, value => {
	let root = document.documentElement//where the one property lives, so a rule below reaches both views whichever is showing
	if (value == 1) { root.style.removeProperty('--gamma-filter'); return }//off, and the rule's fallback is no filter at all
	gammaFilter = 1 - gammaFilter//the one nothing is showing through
	for (let f of document.getElementById(`gammaFilter${gammaFilter}`).firstElementChild.children) f.setAttribute('exponent', 1 / value)//its red, green and blue, rewritten while no picture can be looking
	root.style.setProperty('--gamma-filter', `url(#gammaFilter${gammaFilter})`)//and only then pointed at, which is the change the engine redraws for
})
function onResize() {
	for (let landed of resizeWaiting.splice(0)) landed()//a fullscreen change waiting for the window to arrive at its new size
	reportTrouble(() => activeView()?.onResize?.())
}
async function reportTrouble(work) {//a window event is where the platform starts fuji's code running, so anything the view throws has nowhere to land but here
	try { await work() } catch (error) { logTrouble('shell: handling a window event', error) }//the work is handed in unrun so this catches a handler that throws on the way in, not only one that rejects later
}

/*
The sheet lives in a window and a table lives fullscreen. That is the whole of fuji's window, and every rule below follows from it: going to the table puts the window in fullscreen, going to the sheet takes it out, and nothing else ever does either. The preview is the one exception, a table in a window with no title bar, and it only ever happens first.

The shell places every window fuji makes, before it is revealed; Rust builds each one hidden at no particular size, and window.rs says why fuji places them rather than the platform. A launch on a picture opens as a preview, fitted around the picture in the work area, the part of the desktop the menu bar, dock and taskbar leave free, and placed out from under the pointer; library.js has the rule. Every other launch, and the first sheet after a preview, is the contact sheet's window. Its size is the one thing fuji remembers about a window across launches, because it is the only window the user thinks of as one: the size goes into settings as the user resizes it, and comes back at a random place whenever it still fits the work area. A sheet snapped or tiled to half the screen is only a size here, and comes back at that size somewhere random rather than against the same edge; the convenience is the size, not the place.

Maximized is a state rather than a size, and the one state recorded. A maximized sheet, which on the mac is a zoomed one, sets a flag and leaves the size at what it was before; the next sheet is placed at that size while hidden and maximized last, beside showing it, since on Windows maximizing a hidden window shows it. So restoring goes back to a size the user chose, rather than to a window that fills the work area without being maximized. A minimized sheet and a sheet in a macOS Space record nothing at all, since neither is a size the user gave it, and so fuji never reopens into a Space.

Where the sheet's window is between visits to the table needs no record of fuji's own. Leaving fullscreen puts a window back where it was before, at the size it had — tao saves the frame on the mac and the placement on Windows, and linux's window manager keeps its own — and every trip to the table starts from the sheet's window. A sheet snapped to an edge on Windows is the exception fuji accepts. Windows keeps the size from before the snap in the placement, which is how dragging the window off the edge gives it back, so after the table the sheet returns to that size and place rather than snapped. No call snaps a window again, and anything that resizes the sheet's window ends the snap, so doing it right means a table in a window of its own that leaves the sheet's alone: a redesign of the one page that holds both views, and more than the snap is worth for now. The other exception is a preview's: the frame restored after it is the one fitted to a picture, so the first sheet after a preview gets a title bar and an ordinary frame of its own, and the platform keeps that one from then on. That change happens with the window hidden, and every way out of fullscreen goes through windowFullscreenLeave rather than tauri's own call, because on Windows tauri's call shows a hidden window again at its old frame; window.rs says how the command keeps it hidden. A user never takes the preview or the table for a window — neither has a title bar — so fuji's window first exists for them when the sheet appears, and it should simply appear rather than be seen leaving the preview's place.

Every other change of view changes the window's size too, and the two cannot land in the same frame: the native window takes its new frame at once, and the page repaints for it a frame or two later, so for a moment the old view would show stretched to the screen or squeezed into a window. The curtain, a black cover over the whole window, hides those frames. On Windows the change passes through shapes of its own as well, measured on the Windows box on 2026-09-28: going into fullscreen strips the resize border about 10 ms before the window grows to fill the screen, and coming out puts the border back on the screen-sized window for about 14 ms before it returns to its frame. The curtain covers those too. It goes up for the preview becoming the table, where the picture has to hold still across the change, and for the sheet and the table trading places. It lifts as soon as the window has arrived and the new view is showing, and deliberately not later: a table changing to a picture chosen on the sheet shows its dots until that picture is decoded, rather than keep the screen black for the length of a decode, or show the picture the user has moved on from.

Fuji has two fullscreens, and that is on purpose. macOS's own moves the window onto a Space of its own, with a second of animation, and is what Split View is built on; it suits settling in. Fuji's is simple fullscreen, which fills the screen where the window already is, instantly, with no Space and no animation, and is the one a picture viewer wants. The table uses fuji's, and the green button and the View menu's Enter Full Screen still reach the system's. That second menu item is macOS's own, put into any menu titled View, so menu.rs writes one item and two appear.

Tauri reports only the system's fullscreen on the mac, which is why fullscreenOurs keeps fuji's own record, and which also tells the two apart. The rule that keeps them from stacking is that fuji never lays its fullscreen over a Space: a window already in one takes the table as it is, and leaving the table turns off only what fuji turned on. Windows and Linux have one fullscreen, and there Tauri's simple fullscreen is the ordinary one, which isFullscreen does report, so the Space check is asked only on the mac. The obvious shortcut is a trap: NSWindowCollectionBehaviorFullScreenNone shuts every door into the system's fullscreen at once, and takes Split View and a Space of one's own with it. It was tried and taken back out.

On the mac, simple fullscreen also hides the dock and the menu bar for the whole application rather than the window, and only leaving it puts them back — read out of tao 0.35.3 — so closeWindow leaves it before closing. And it clears the window's Titled style mask, which drops the first responder. tao repairs that with its own content view, a level above the WKWebView, so the mouse still works and no key reaches JavaScript until the user clicks; focusing the webview after every change is the same repair aimed a level lower, and needs core:webview:allow-set-webview-focus.
*/
let fullscreenOurs = false//fuji's own simple fullscreen is on, which tauri will not report on the mac
let previewFramed = false//the window still has the frame the preview fitted to one picture, and no title bar
let previewPath = '', previewCard = null//the picture the preview opened on, and where it stood, as a rectangle of the fullscreen frame, for the diamond table to take it over in the same place
let resizeWaiting = []//fullscreen changes waiting for the viewport to arrive at its new size, which onResize answers
const curtainShowing = ref(false)//the curtain, a black cover over the whole window while the view and the window's size change beneath it

async function placeWindow(w, path) {//put the hidden window where it will first be seen: fitted around the picture it opened on, or the sheet's window
	let natural = false
	if (path) { await activeView()?.onDrop?.(path); natural = activeView()?.natural?.() }//the preview shows the picture now, since its size is what shapes the window
	let areas = await screenAreas()
	if (!areas) return//no monitor to measure, so the window stays where the operating system put it
	if (!natural) { await placeSheet(areas.work); await maximizeSheet(w); return }//no picture, or one that would not load: the sheet's window, title bar and all, maximized last because the reveal comes next
	let frame = rectPreview(natural, areas.work, await pointerPosition())//where the user double-clicked, very likely, or wherever the pointer has just gone
	previewPath = path
	previewCard = {x: frame.x - areas.screen.x, y: frame.y - areas.screen.y, width: frame.width, height: frame.height}//fullscreen fills the screen, so its frame's corner is the screen's
	await windowFrameSet(frame)//the frame first and the title bar after, never the other way: tao queues a change of title bar onto the main thread rather than making it, and window_frame_set measures the title bar the moment it is called, so placing a window straight after a change would measure the window as it was. A window keeps its frame when its title bar comes or goes, and the content grows or shrinks inside it
	await w.setDecorations(false); previewFramed = true
}

async function toggleView() {//the View menu's Toggle Full Screen: the table fullscreen or the sheet in a window, and from the preview on into the table
	if (whichTable.value == 'Preview') {
		if (previewPath) return previewExpand(previewPath)
		whichTable.value = settings.view.table//a preview whose picture would not load has nothing to hand over, so the table behind the sheet is the usual one
	}
	return showView(showing.value == 'Table' ? 'Sheet' : 'Table')//from the settings as from the sheet, since both are the window
}
async function showView(name) {//show the sheet or the settings in a window, or the current table fullscreen; the sheet and the table stay mounted, so the one going away keeps its scroll, its pan, and its decoded images
	if (showing.value == name) return
	if (name == 'Sheet' && previewFramed) return sheetFromPreview()
	if (name != 'Table' && showing.value != 'Table') {//the sheet and the settings trading places in the one window, which keeps its size, so nothing needs covering
		showing.value = name
		await nextTick()//the arriving view is on the page, and the panel has been made
		activeView()?.start?.()
		return
	}
	await curtained(async () => {
		await fullscreenSet(name == 'Table')//into fullscreen for the table and out of it for the sheet, before the view is shown, so a table measures the frame it will keep
		showing.value = name
		await nextTick()//v-show has been applied, so the view arriving has a layout box and can measure itself
		activeView()?.start?.()//not awaited: a table changing to a newly chosen picture hides its card until the picture is on it, so the curtain can lift on the dots rather than wait for a decode
	})
}
async function previewExpand(path) {//the preview was clicked: the diamond table, fullscreen, with the card where the preview's picture stood
	cacheNeed(path, 'Shell')//hold the picture across the swap: the preview lets go as it unmounts, which is before the diamond table exists to ask, and the store would free the decode in between
	try {
		await curtained(async () => {
			whichTable.value = 'Diamond'//the diamond by name rather than whatever fuji.toml says, because it is the table that takes a card at a rectangle. Swapped in before the fullscreen change rather than after, so the preview, which closes the window when it loses the focus, is gone before the window's style changes under it
			await nextTick()
			await fullscreenSet(true)
			await activeView()?.start?.()//after the fullscreen change, so the table measures the frame it will keep
			await activeView()?.onDrop?.(path)//the drop road, so the model lists the folder and the table can flip from here
			if (previewCard) activeView()?.cardAt?.(previewCard)
		})
	} finally {
		cacheRelease(path, 'Shell')
	}
}
async function sheetFromPreview() {//the first sheet after a preview: the window goes away and comes back as an ordinary one, because the user never took the preview or the table for a window, and a window appearing where the preview was would say it had been one all along
	let w = getCurrentWindow()
	await w.hide()
	if (fullscreenOurs) { await windowFullscreenLeave(); fullscreenOurs = false }//without fullscreenSet's wait for the resize, which a hidden window cannot be relied on to deliver; nothing measures until the window is back
	previewFramed = false
	let areas = await screenAreas()
	if (areas) await placeSheet(areas.work)//the frame first and the title bar after, for the reason placeWindow gives
	await w.setDecorations(true)
	showing.value = 'Sheet'
	await nextTick()
	activeView()?.start?.()
	await maximizeSheet(w)
	await w.show()
	await getCurrentWebview().setFocus()//the title bar came back, which is a change of style mask, so the keyboard needs handing back for the reason the essay above gives
}
async function placeSheet(work) { await windowFrameSet(rectSheet(work, settings.sheet)) }//the size the user last gave a sheet, or the preset, somewhere in the work area
async function maximizeSheet(w) { if (settings.sheet.maximized) await w.maximize() }//over the frame placeSheet gave it, which is where restoring goes; call it right before the window shows, because on windows maximizing a hidden window shows it
async function recordSheet() {//the sheet's size into settings as the user resizes it, or that it is maximized; cheap, since settings reach the disk only when fuji exits
	if (showing.value == 'Table' || fullscreenOurs || previewFramed) return//only the sheet's ordinary window is a window to the user, whether the sheet or the settings is in it
	let w = getCurrentWindow()
	let [visible, minimized, fullscreen, maximized] = await Promise.all([w.isVisible(), w.isMinimized(), w.isFullscreen(), w.isMaximized()])
	if (!visible) return//fuji placing a hidden sheet, as sheetFromPreview does, rather than the user sizing one; recording it would clear the maximized flag in the moment before maximizeSheet reads it
	if (minimized || fullscreen) return//neither is a size the user gave the sheet: windows measures a minimized window as a sliver parked off the screen, and a macos space fills a screen the sheet was never sized to. Fullscreen can only be a space here, since fuji's own is ruled out above
	settings.sheet.maximized = maximized//zoomed, on the mac, which tao answers as the same thing
	if (!maximized) {//a maximized sheet leaves the size where it was, which is where restoring it goes
		let frame = await windowFrame()
		settings.sheet.width = Math.round(frame.width); settings.sheet.height = Math.round(frame.height)
	}
	settingsChanged()
}
async function fullscreenSet(on) {//turn fuji's own fullscreen on or off, and wait for the window to arrive
	if (on == fullscreenOurs) return
	let w = getCurrentWindow()
	if (on && platform() == 'mac' && await w.isFullscreen()) return//already in a macos space, which the table can have as it is
	fullscreenOurs = on
	let landed = new Promise(resolve => { resizeWaiting.push(resolve); setTimeout(resolve, 1000) })//the resize that says the window got there, or a second, so a change that moves nothing never hangs
	if (on) await w.setSimpleFullscreen(true)//instant, no Space and no animation; on windows and linux, the ordinary fullscreen
	else await windowFullscreenLeave()//the one way fuji leaves it, whether the window is showing or not
	await getCurrentWebview().setFocus()//hand the keyboard back to the page, after the change, for the reason the essay above gives
	await landed
}
async function curtained(work) {//do a change of view behind the curtain the essay above describes
	curtainShowing.value = true
	await raf(); await raf()//the first frame schedules the cover's paint, and the second confirms it reached the screen before anything changes beneath it
	try { await work(); await raf() } finally { curtainShowing.value = false }//one frame more, so what changed has painted before the cover lifts
}
async function closeWindow() {//close the window as the red button or the × would, hidden and out of fuji's fullscreen first
	let w = getCurrentWindow()
	if (fullscreenOurs) {
		await w.hide()//so the user never sees it come back to its window size on the way out
		await windowFullscreenLeave(); fullscreenOurs = false//on the mac, left before closing so the dock and menu bar come back for the whole application
	}
	await w.close()
}

</script>
<template>

<Sheet ref="sheetRef" v-show="showing == 'Sheet'" @table="reportTrouble(() => showView('Table'))" @settings="reportTrouble(() => showView('Settings'))" />
<SettingsPanel v-if="showing == 'Settings'" ref="settingsRef" @sheet="reportTrouble(() => showView('Sheet'))" />
<component :is="tables[whichTable]" ref="tableRef" v-show="showing == 'Table'" @expand="path => reportTrouble(() => previewExpand(path))" @sheet="reportTrouble(() => showView('Sheet'))" @close="reportTrouble(closeWindow)" />
<HelpPanel v-if="helpShowing && whichTable != 'Preview'" class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2" /><!-- after the views, so it paints over them; centered on the window, which is the frame of every view. Never over a preview, whose window is the picture and nothing else, and which a new user meets before anything the panel describes -->
<div v-if="curtainShowing" class="fixed inset-0 bg-black"></div><!-- last, so it covers everything while the view and the fullscreen change -->

<!-- the two gamma filters, taking turns and drawing nothing themselves; the exponents are written by the watch above rather than bound here, because the order of the write and the switch is the whole point. The region is the element's own box, where the default reaches a tenth past each edge for nothing -->
<svg aria-hidden="true" width="0" height="0" class="absolute w-0 h-0">
	<filter v-for="n in [0, 1]" :key="n" :id="`gammaFilter${n}`" color-interpolation-filters="sRGB" x="0" y="0" width="1" height="1">
		<feComponentTransfer>
			<feFuncR type="gamma" exponent="1" />
			<feFuncG type="gamma" exponent="1" />
			<feFuncB type="gamma" exponent="1" />
		</feComponentTransfer>
	</filter>
</svg>

</template>
<style>

/* not scoped, because the pictures are other components' elements, and the table's are not even a template's; a thumbnail is a myTile and the table's image is a myImage, placeholders included, which brighten harmlessly */
.myTile, .myImage {
	filter: var(--gamma-filter, none); /* none whenever the root carries no filter, which is every moment gamma is off */
}

</style>
