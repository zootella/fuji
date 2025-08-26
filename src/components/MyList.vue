<script setup>//./components/MyList.vue

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {
xy, measureScreen,
} from './library.js'//our javascript library

onMounted(async () => {
	let s = await measureScreen()

log(`measuring the screen

${s.windowDevicePixelRatio} from HTML, window.devicePixelRatio
${s.tauriWindowScaleFactor} from Tauri, await getCurrentWindow().scaleFactor()
${s.tauriMonitorScaleFactor} from Tauri, (await currentMonitor()).scaleFactor

${s.cssScreen.x} × ${s.cssScreen.y} from HTML, screen.width and screen.height (CSS pixels)
${s.backingScreen.x} × ${s.backingScreen.y} from Tauri, (await currentMonitor()).size.width and .height (macOS backing pixels)
${s.physicalScreen.x} × ${s.physicalScreen.y} from our own Rust code, panelResolution() 🦀 (physical pixels)
`)
})

/*ttd august
test this
[]mac X []windows
[]old dell monitor X []new 4k monitor
on mac, []different looks like resolutions
on windows, []different zoom numbers in settings

leaving out mac and windows set to blur the monitor to a not native resolution, though
leaving out multiple monitors, also
*/

/*ttd august
strategy to use hard vertical
if window.devicePixelRatio is 1, then just use screen.height
if hard_vertical returns 0, <1080, or something not listed above, screen.height

ttd august, note that on a macbook air you're seeing
1710 / 1112 = 1.537 769 screen css
2560 / 1664 = 1.538 461 panel resolution
so following from the panel resolution, screen css really needs to be either
1710.769 x 1112, or
1710 x 1111.5
yeah that's better so use width, not height, when computing the scale ratio css to physical
*/

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
