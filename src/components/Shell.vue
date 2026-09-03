<script setup>//./components/Shell.vue - owns the window; draws nothing

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {raf, forwardize, revealWindow} from './library.js'
import {settings, settingsLoad, settingsChanged, settingsWindowRect} from '../settings.js'
import DiamondTable from './DiamondTable.vue'

/*
The shell owns the window and none of the pixels. It reads the settings file, sizes and reveals the window, records where the user puts it, and holds the one listener for each window event. It has no background, no chrome, and no HUD, so a view never has to negotiate with a parent about how it looks.

It exists because window events are global and everything else is not. A view's wheel, pointer, and double-click handlers live on its own element, so a hidden view is handed none of them and two views cannot collide. But window.addEventListener fires no matter what is visible, and so does a tauri window event, so keydown, resize, and drag-drop are the entire interference surface between views. One listener each lives here and gives the event to the view that is showing. A hidden view cannot react to a key because it is never given one, rather than because it remembered to check.

Startup runs in one order for one reason: a view cannot measure itself until the window is real. The window is created hidden at 800 by 600, so a frame measured before the reveal is the wrong size, and a view that is not showing measures nothing at all, because v-show is display none and that destroys the layout box. Vue also runs a child's onMounted before its parent's, so a view cannot do this for itself from down there. Hence the contract: a view exposes start(), and the shell calls it once the window is up and the view is on screen.

The settings read is wrapped because the reveal below must happen either way. A window that never appears is an application with no way to tell anyone what went wrong, which is the same reason revealWindow shows the window from a finally.
*/

const viewRef = ref(null)//the view that is showing, and the only one handed a window event

onMounted(async () => {
	let w = getCurrentWindow()
	try {
		await settingsLoad()//before the reveal, because the window's size and position come out of the file
	} catch (error) {
		console.error('reading settings:', error)//carry on to the reveal on factory settings rather than leave the window hidden
	}
	await revealWindow(settingsWindowRect())

	await raf()//the window is up and resized; let the viewport report its new dimensions before the view measures them
	viewRef.value.start()

	window.addEventListener('keydown', onKey)
	window.addEventListener('resize', onResize)
	unlistenFileDrop = await w.onDragDropEvent(event => {
		if (event.payload.type == 'drop' && event.payload.paths.length) reportTrouble(() => viewRef.value.onDrop(forwardize(event.payload.paths[0])))//forwardized here, at the boundary where a path enters fuji
	})
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

function onKey(e)   { reportTrouble(() => viewRef.value.onKey(e)) }//the only keydown listener in fuji; the view is always mounted by the time this can fire, so there is nothing to guard against
function onResize() { reportTrouble(() => viewRef.value.onResize()) }
async function reportTrouble(work) {//a window event is where the platform starts fuji's code running, so anything the view throws has nowhere to land but here
	try { await work() } catch (error) { console.error('handling a window event:', error) }//the work is handed in unrun so this catches a handler that throws on the way in, not only one that rejects later
}

async function recordWindow(w) {//write down the window fuji has right now, for the settings file to carry to the next launch
	let position = await w.outerPosition()//outer, matching setPosition and the onMoved payload
	let size = await w.innerSize()//inner, matching setSize and the onResized payload; mixing the two would grow the window by a titlebar every launch
	settings.window.x     = position.x; settings.window.y      = position.y
	settings.window.width = size.width; settings.window.height = size.height
	settingsChanged()
}
function isFullscreen() {//a window the size of the screen is not one the user placed, so it must not become the one fuji remembers
	return viewRef.value.isFullscreen()//the view is asked because the view initiates the transition, and simple fullscreen is not something the operating system will report; this moves up here when fullscreen does
}

</script>
<template>

<DiamondTable ref="viewRef" />

</template>
