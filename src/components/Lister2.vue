<script setup>//./components/Lister2.vue - drop in image and list images in that folder (current best)

//keep, drop events we list files from, and lots of native resolution working and thinking in here, too

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import {ioRead, ioReadDir} from '../io.js'
import {ref, onMounted, onBeforeUnmount, nextTick} from 'vue'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {lookPath, forwardize, backize} from './library.js'//our javascript library

const hardVerticals = [
	480,//Legacy 640×480 VGA; still seen in embedded systems and some virtual modes
	600,//SVGA (800×600); common in late '90s multimedia PCs
	768,//WXGA (1366×768); dominant in budget laptops for over a decade
	900,//1440×900 and 1600×900; mid-range panels, especially Dell and HP
	1024,//SXGA (1280×1024); popular in 4:3 office monitors

	1080,//Full HD (1920×1080); the most common resolution today
	1200,//UXGA (1600×1200); classic 4:3 pro panels, especially Dell UltraSharp
	1440,//QHD (2560×1440); high-end monitors and gaming setups
	2160,//4K UHD (3840×2160); widespread in premium laptops and monitors
	2880,//5K (5120×2880); found in iMacs and ultra-premium displays
]

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
log(`trying to get the true vertical pixel resolution, looking through fake CSS pixels and fake OS looks like resolutions 😫
perhaps some of these numbers will be, or let us compute, that answer

${n2} window.devicePixelRatio
${n3} tauri getCurrentWindow scaleFactor
${n5} tauri currentMonitor scaleFactor

${n1} screen.height
${n6} rust hard_vertical 🦀
${n4} tauri currentMonitor size.height

2160 is the correct answer for a 4K monitor 🏆

running tests right now on an apple silicon mac mini connected to a cheap 4k monitor from amazon
but also, at a macos settings "looks like" resolution of 2560x1440, which i will change to other options in this test
so the correct answer is 2160, but it may be hard to get that number or numbers that product to that number
`)

	unlistenFileDrop = await w.onDragDropEvent(async (event) => {
		if (event.payload.type == 'drop' && event.payload.paths.length) {
			let path = forwardize(event.payload.paths[0])

			log(await lookPath(path))
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
