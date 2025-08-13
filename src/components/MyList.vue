<script setup>//./components/MyList.vue - drop in image and list images in that folder (current best)

//keep, drop events we list files from, and lots of native resolution working and thinking in here, too

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {xy, raf, blobToDataUrl, forwardize, backize, listSiblings, readAndRenderImage} from './library.js'//our javascript library

onMounted(async () => {
	const w = getCurrentWindow()
	const m = await currentMonitor()

	let n1 = screen.height
	let n2 = window.devicePixelRatio
	let n3 = await w.scaleFactor()
	let n4 = m.size.height
	let n5 = m.scaleFactor
	let n6 = await invoke('hard_vertical')
	/*
	strategy to use hard vertical
	if window.devicePixelRatio is 1, then just use screen.height
	if hard_vertical returns 0, <1080, or something not listed above, screen.height
	*/
log(`getting the true vertical pixel resolution

${n2} window.devicePixelRatio
${n3} tauri getCurrentWindow scaleFactor
${n5} tauri currentMonitor scaleFactor

${n1} screen.height
${n6} rust hard_vertical 🦀
${n4} tauri currentMonitor size.height

(some correct answers are 1600 classic dell, 1664 macbook air, 2160 4K monitor 🏆)
`)

	unlistenFileDrop = await w.onDragDropEvent(async (event) => {
		if (event.payload.type == 'drop' && event.payload.paths.length) {
			let path = forwardize(event.payload.paths[0])

			log(await listSiblings(path))
		}
	})
})
let unlistenFileDrop//will hold the unsubscribe function set above and called below
onBeforeUnmount(() => {
	if (unlistenFileDrop) unlistenFileDrop()
})

const refLines = ref([])
function log(s) {
	refLines.value.push(s)
	console.log(s)
}

</script>
<template>

<div class="h-full overflow-y-auto bg-white p-4 font-mono text-xs leading-tight">
<div v-for="(line, idx) in refLines" :key="idx"><pre>{{line}}</pre></div>
</div>

</template>
<style scoped>

</style>
