<script setup>//./components/Lister2.vue - drop in image and list images in that folder (current best)

//keep, drop events we list files from, and lots of native resolution working and thinking in here, too

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import {ioRead, ioReadDir} from '../io.js'
import {ref, onMounted, onBeforeUnmount, nextTick} from 'vue'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {lookPath, forwardize, backize} from './library.js'//our javascript library

//tolibrary
const hardVerticals = [
	480,  // Legacy 640×480 VGA; still seen in embedded systems and some virtual modes
	600,  // SVGA (800×600); common in late '90s multimedia PCs
	720,  // HD (1280×720); entry-level TVs, older budget laptops
	768,  // WXGA (1366×768); dominant in budget laptops for over a decade [mainstream]
	864,  // FWXGA variant (1536×864); occasional in mid-range laptops
	900,  // 1440×900 / 1600×900; mid-range panels, especially Dell and HP [mainstream]
	1024, // SXGA (1280×1024); popular 4:3 office monitors in the 2000s
	1050, // WSXGA+ (1680×1050); mid-to-high-end 16:10 panels in the late 2000s
	1080, // Full HD (1920×1080); the most common resolution today [mainstream]
	1152, // QWXGA (2048×1152); uncommon wide format, some niche monitors
	1200, // UXGA (1600×1200) / WUXGA (1920×1200); pro and workstation panels [mainstream]
	1440, // QHD (2560×1440); high-end monitors, gaming setups, premium laptops [mainstream]
	1536, // QXGA (2048×1536); uncommon high-res 4:3, some tablets
	1600, // WQXGA (2560×1600); premium 16:10 laptops and monitors [mainstream]
	1664, // MacBook Air M2 native (2560×1664)
	1964, // MacBook Pro 14" native (3024×1964)
	2160, // 4K UHD (3840×2160); widespread in premium laptops and monitors [mainstream]
	2234, // MacBook Pro 16" native (3456×2234)
	2520, // iMac 24" 4.5K Retina (4480×2520)
	2880, // 5K (5120×2880); iMacs and ultra-premium displays
	3200, // 6K (6016×3384); Apple Pro Display XDR and similar
	4320, // 8K UHD (7680×4320); bleeding-edge professional monitors
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
