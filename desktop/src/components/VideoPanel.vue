<script setup>//fuji video: for now its name in the middle of a window of its own size, the workspace the video viewer is built in a step at a time

import {getCurrentWindow} from '@tauri-apps/api/window'
import {logTrouble} from '../log.js'

//the shell trades this with the sheet when either asks with v, and gives it the sheet's window at a size of its own; which view is showing, the window's size and fuji's fullscreen are the shell's, so this only asks

const emit = defineEmits(['sheet', 'fullscreen'])//v, for the contact sheet back; a double click, for fuji's own fullscreen on or off with this view staying
function onKey(e) {
	if (e.key == 'v' && !e.ctrlKey && !e.metaKey) emit('sheet')
}

//a press anywhere that moves a few pixels moves the window, and two presses that stay are a double click, which trades the window for the screen and back. The drag waits for the move, as the audio panel's does, because once the platform's drag begins, Windows runs the move itself and the release never reaches the page, so a drag begun on the press would leave a double click nothing to be made of
const dragAfter = 4//how far the pointer moves along either axis, in css pixels with the button down, before a press is a drag
let press = null//where the left button went down, or null while it is up and once the press has become a drag
function onPress(e) { if (e.button == 0) press = {x: e.clientX, y: e.clientY} }
function onMove(e) {
	if (!press || (Math.abs(e.clientX - press.x) < dragAfter && Math.abs(e.clientY - press.y) < dragAfter)) return
	press = null
	getCurrentWindow().startDragging().catch(error => logTrouble('video: dragging the window', error))
}

defineExpose({onKey})//the one call of the shell's this view has a use for

</script>
<template>

<div class="myMono bg-paper text-strong w-full h-full flex items-center justify-center select-none" @mousedown="onPress" @mousemove="onMove" @mouseup="press = null" @mouseleave="press = null" @dblclick="emit('fullscreen')">V I D E O</div>

</template>
