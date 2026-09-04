<script setup>//./components/Shell.vue - owns the window; draws nothing

import {ref, nextTick, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {raf, forwardize, revealWindow} from './library.js'
import {settings, settingsLoad, settingsChanged, settingsWindowRect} from '../settings.js'
import {experimentRun} from '../experiment.js'//TEMPORARY
import Sheet from './Sheet.vue'
import DiamondTable from './DiamondTable.vue'
import ComicTable from './ComicTable.vue'

/*
The shell owns the window and none of the pixels. It reads the settings file, sizes and reveals the window, records where the user puts it, holds the one listener for each window event, and remembers which view was showing. It has no background, no chrome, and no HUD, so a view never has to negotiate with a parent about how it looks.

It exists because window events are global and everything else is not. A view's wheel, pointer, and double-click handlers live on its own element, so a hidden view is handed none of them and two views cannot collide. But window.addEventListener fires no matter what is visible, and so does a tauri window event, so keydown, resize, and drag-drop are the entire interference surface between views. One listener each lives here and gives the event to the view that is showing. A hidden view cannot react to a key because it is never given one, rather than because it remembered to check.

There is one sheet and there are several tables, and the two facts are separate: whether the sheet is showing, and which table is behind it. The sheet and the current table swap with v-show and both stay mounted, because that switch is frequent and has to be instant with nothing reloading. Tables swap with each other by :is, which destroys and creates, because a table nobody is using should not be holding decoded images. Adding a table is one entry in the tables object below.

Startup runs in one order for one reason: a view cannot measure itself until the window is real and it is on screen. The window is created hidden at 800 by 600, so a frame measured before the reveal is the wrong size, and a view that is not showing measures nothing at all, because v-show is display none and that destroys the layout box. Vue also runs a child's onMounted before its parent's, so a view cannot do this for itself from down there. Hence the contract: a view exposes start(), the shell calls it when that view first comes on screen, and the view makes it happen only once.

The settings read is wrapped because the reveal below must happen either way. A window that never appears is an application with no way to tell anyone what went wrong, which is the same reason revealWindow shows the window from a finally.
*/

const tables = {Diamond: DiamondTable, Comic: ComicTable}//every table fuji has; a new one is a line here and nothing else

const sheetRef   = ref(null)
const tableRef   = ref(null)
const showing    = ref('Table')//Sheet or Table: which kind of view the user is looking at
const whichTable = ref('Diamond')//which table is behind the sheet, whether or not it is the one showing

onMounted(async () => {
	let w = getCurrentWindow()
	try {
		await settingsLoad()//before the reveal, because the window's size and position come out of the file
	} catch (error) {
		console.error('reading settings:', error)//carry on to the reveal on factory settings rather than leave the window hidden
	}
	showing.value = settings.view.showing//before the reveal, so the first frame the user sees is the view they left
	whichTable.value = settings.view.table
	if (!tables[whichTable.value]) {//a name settings cannot check, because the tables fuji has are known here and not there
		console.log(`⭕ settings: no table named ${whichTable.value}, showing Diamond instead`)
		whichTable.value = 'Diamond'
		settings.view.table = whichTable.value; settingsChanged()//written back, so a name fuji cannot use is repaired in the file the same way a bad value anywhere else in it is
	}
	await nextTick()//let vue place the right view before the window appears

	await revealWindow(settingsWindowRect())
	await raf()//the window is up and resized; let the viewport report its new dimensions before the view measures them
	activeView().start()

	window.addEventListener('keydown', onKey)
	window.addEventListener('resize', onResize)
	unlistenFileDrop = await w.onDragDropEvent(event => {
		if (event.payload.type == 'drop' && event.payload.paths.length) reportTrouble(() => activeView().onDrop?.(forwardize(event.payload.paths[0])))//forwardized here, at the boundary where a path enters fuji; optional because a view answers only the calls it has a use for
	})
	experimentRun().catch(error => console.error('experiment:', error))//TEMPORARY, and last so nothing above waits on it
	if (settings.window.remember) {//record where the user puts the window, so it comes back there next launch; both events report physical pixels, as the file holds them
		await recordWindow(w)//the events below report only changes, so without this a session where the user never touches the window records nothing
		unlistenMoved   = await w.onMoved(  ({payload}) => { if (isFullscreen()) return; settings.window.x     = payload.x;     settings.window.y      = payload.y;      settingsChanged() })
		unlistenResized = await w.onResized(({payload}) => { if (isFullscreen()) return; settings.window.width = payload.width; settings.window.height = payload.height; settingsChanged() })
	}
})
let unlistenFileDrop, unlistenMoved, unlistenResized//will hold the unsubscribe functions set above and called below
onBeforeUnmount(() => {
	window.removeEventListener('keydown', onKey)
	window.removeEventListener('resize', onResize)
	if (unlistenFileDrop) unlistenFileDrop()
	if (unlistenMoved) unlistenMoved()
	if (unlistenResized) unlistenResized()
})

function activeView() { return showing.value == 'Sheet' ? sheetRef.value : tableRef.value }

function onKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//a keystroke into a form field belongs to the field; this is the only keydown listener in fuji, so this is the only place the guard is needed
	if (e.key == 'c') { reportTrouble(() => showView(showing.value == 'Sheet' ? 'Table' : 'Sheet')); return }//the shell's own key, and never passed down
	reportTrouble(() => activeView().onKey(e))
}
function onResize() { reportTrouble(() => activeView().onResize()) }
async function reportTrouble(work) {//a window event is where the platform starts fuji's code running, so anything the view throws has nowhere to land but here
	try { await work() } catch (error) { console.error('handling a window event:', error) }//the work is handed in unrun so this catches a handler that throws on the way in, not only one that rejects later
}

async function showView(name) {//show the sheet or the current table; both stay mounted, so the one going away keeps its scroll, its pan, and its decoded images
	if (showing.value == name) return
	showing.value = name
	settings.view.showing = name; settingsChanged()
	await nextTick()//v-show has been applied, so the view arriving has a layout box and can measure itself
	activeView().start()
}

async function recordWindow(w) {//write down the window fuji has right now, for the settings file to carry to the next launch
	let position = await w.outerPosition()//outer, matching setPosition and the onMoved payload
	let size = await w.innerSize()//inner, matching setSize and the onResized payload; mixing the two would grow the window by a titlebar every launch
	settings.window.x     = position.x; settings.window.y      = position.y
	settings.window.width = size.width; settings.window.height = size.height
	settingsChanged()
}
function isFullscreen() {//a window the size of the screen is not one the user placed, so it must not become the one fuji remembers
	return tableRef.value.isFullscreen()//the table is asked whichever view is showing, because only a table enters fullscreen and it keeps the flag; the operating system will not report simple fullscreen, so there is nobody else to ask
}

</script>
<template>

<Sheet ref="sheetRef" v-show="showing == 'Sheet'" />
<component :is="tables[whichTable]" ref="tableRef" v-show="showing == 'Table'" />

</template>
