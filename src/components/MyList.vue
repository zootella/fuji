<script setup>//./components/MyList.vue - drop in image and list images in that folder (current best)

//keep, drop events we list files from, and lots of native resolution working and thinking in here, too

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {xy, raf, blobToDataUrl, forwardize, backize, listSiblings, readAndRenderImage} from './library.js'//our javascript library

async function measureScreen() {//get the screen resolution as {x, y} in all the different real and fake pixel units
	/*
	test this
	[]mac X []windows
	[]old dell monitor X []new 4k monitor
	on mac, []different looks like resolutions
	on windows, []different zoom numbers in settings


	leaving out mac and windows set to blur the monitor to a not native resolution, though
	leaving out multiple monitors, also
	*/
	/*
	strategy to use hard vertical
	if window.devicePixelRatio is 1, then just use screen.height
	if hard_vertical returns 0, <1080, or something not listed above, screen.height
	*/
	const w = getCurrentWindow()
	const m = await currentMonitor()

	let hardVertical = await invoke('hard_vertical')
	let hardHorizontal = hardVertical * screen.width / screen.height

	let arrows = {
		screenCss: xy(screen.width, screen.height),//CSS pixels, matches styles on div tags, matches macOS's "looks like" resolution in settings
		screenOsCanvas: xy(m.size.width, m.size.height),//macOS's scaled canvas, what macOS actually paints text, vectors, and images onto
		screenPhysical: xy(hardHorizontal, hardVertical),//physical hardware lights, had to go deep in Rust to get these
	}
	console.log(arrows)
	return arrows
}

onMounted(async () => {
	const w = getCurrentWindow()
	const m = await currentMonitor()

	let measure = await measureScreen()

log(`getting the true vertical pixel resolution

${window.devicePixelRatio} window.devicePixelRatio
${await w.scaleFactor()} tauri getCurrentWindow scaleFactor
${m.scaleFactor} tauri currentMonitor scaleFactor

${measure.screenCss.x     } × ${measure.screenCss.y     } screen css, from CSSs screen.width and screen.height
${measure.screenPhysical.x} × ${measure.screenPhysical.y} screen physical, from low level graphics apis 🦀
${measure.screenOsCanvas.x} × ${measure.screenOsCanvas.y} screen os canvas, from tauri's currentMonitor() .size.width and .size.height
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
