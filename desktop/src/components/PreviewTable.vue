<script setup>//the picture fuji was opened on, as large as the desktop allows, and nothing to do but look

import {ref, onBeforeUnmount} from 'vue'
import {cacheNeed, cacheRelease} from '../cache.js'
import {log} from '../log.js'

/*
The first thing fuji shows when a picture is double-clicked in the file manager. The shell takes the title bar off and fits the window around this picture, in the part of the desktop the menu bar, dock and taskbar leave free and away from the pointer, so the window is the picture and nothing else. There is no flipping, no panning and no zoom, which is why this is a table of its own rather than a mode of DiamondTable: nothing here can reach the diamond table's code, and nothing done there can break this.

A click is the way in. It asks the shell for the diamond table, built around this picture where it stands and then taken fullscreen. A lost focus is the way out, and so is escape, which the shell takes on every table: the user looked, and went back to what they were doing, so the window closes exactly as the red button or the × would. On the mac fuji stays in the dock after, as a mac application does.

The picture is the store's own element, as it is on the diamond table, so the table that follows adopts the same decoded pixels rather than reading and decoding the file again.
*/

const emit = defineEmits(['expand', 'close'])//a click, for the shell to swap in the diamond table, and a lost focus, for it to close the window; the window is the shell's, so this only asks

const previewHolder = 'PreviewTable'//the name on the one reference this table takes in the store
const frameRef = ref(null)
let path = ''//the picture on show, and the reference held for it
let entry = null//the store's entry for it
let focused = false//whether this window has had the focus yet: a window can report losing it while it is still appearing, and that must not close it

async function onDrop(p) {//show this picture; the shell calls this before the window is revealed, and a drop or file open later lands here too
	let previous = path
	path = p
	entry = await cacheNeed(path, previewHolder)
	if (previous) cacheRelease(previous, previewHolder)//after the new one is held, so a drop of the same picture never frees it in between
	if (entry.error) { log(`preview: could not show ${path}, ${entry.error}`); frameRef.value.replaceChildren(); return }
	entry.img.className = 'myImage'
	entry.img.style.display = 'block'//the diamond table hides the store's elements it is not showing, and this may be one it hid
	frameRef.value.replaceChildren(entry.img)
}
function natural() { return entry?.img && !entry.error ? {x: entry.img.naturalWidth, y: entry.img.naturalHeight} : false }//the picture's own size, for the shell to shape the window to; false when there is nothing to shape it around

function onFocus(now) {//the shell hands every change of this window's focus here
	if (now) focused = true
	else if (focused) emit('close')
}
function onClick() { if (path) emit('expand', path) }

onBeforeUnmount(() => { if (path) cacheRelease(path, previewHolder) })//the shell holds the picture across the swap, so the diamond table finds it still decoded

defineExpose({onDrop, natural, onFocus})//the same calls the shell makes of every view, where this one has a use for them, and natural, which only the preview is asked

</script>
<template>

<div ref="frameRef" class="myPreview w-full h-full select-none" @click="onClick" @contextmenu.prevent></div>

</template>
<style scoped>

.myPreview {
	background-color: black; /* only ever seen for the fraction of a pixel the window's rounding leaves, or behind a picture that would not load */
}
/* the store's element, adopted rather than templated, so reached with :deep() for the reason the essay above .myImage in DiamondTable.vue gives */
.myPreview :deep(.myImage) {
	width: 100%; height: 100%;
	object-fit: contain; /* the window is already the picture's shape, so this only matters within a pixel of rounding, where it keeps the whole picture rather than stretching it */
}

</style>
