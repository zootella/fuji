<script setup>//./components/MyLens.vue - io module, FileReader, Blob, img tag, SVG gamma filter option, CSS pixelated option

//keep, here's where you've got SVG gamma and CSS pixelated

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {xy, raf, blobToDataUrl, forwardize, backize, lookPath, readImage, renderImage} from './library.js'//our javascript library

const imageRef = ref(null)//ref of the img tag that shows the image
const sourceRef = ref('')//ref of the src attribute in that image tag, keeping both right now as we decide which to use!
const gammaRef = ref(1)//1 no change, 0.9 slight shadow boost, 0.5 strong boost, 0 whiteout; a UI that lowers this number linearly should work well to first allow for subtle change and then drastic exploration
const roughRef = ref(false)//false for the high quality scaling which we get by default, true for pixel art and blocky pixels at high zoom

onMounted(async () => {
	const w = getCurrentWindow()
	unlistenFileDrop = await w.onDragDropEvent(event => {
		if (event.payload.type === 'drop' && event.payload.paths.length) {
			let p = event.payload.paths[0]
			console.log(`dropped path "${p}"`)
			loadImage(p)
		}
	})
})
let unlistenFileDrop//will hold the unsubscribe function set above and called below
onBeforeUnmount(() => {
	if (unlistenFileDrop) unlistenFileDrop()
})

function asyncBlobToDataUrl(blob) {
	return new Promise((resolve, reject) => {
		const reader = new FileReader()
		reader.onerror   = () => reject(reader.error)
		reader.onloadend = () => resolve(reader.result)
		reader.readAsDataURL(blob)
	})
}

async function loadImage(p) {
	const bytes = new Uint8Array(await ioRead(p))// Uint8Array
	const blob = new Blob([bytes.buffer], {type: 'image/png'})//ttd july, set this correctly from file extension
	sourceRef.value = await asyncBlobToDataUrl(blob)
}

</script>
<template>
<div>

<!-- SVG filter definition, hidden -->
<svg aria-hidden="true" width="0" height="0" class="absolute overflow-hidden w-0 h-0 pointer-events-none">
	<defs><filter id="my-gamma"><feComponentTransfer>
		<feFuncR type="gamma" :exponent="gammaRef"/>
		<feFuncG type="gamma" :exponent="gammaRef"/>
		<feFuncB type="gamma" :exponent="gammaRef"/>
	</feComponentTransfer></filter></defs>
</svg>

<div class="w-screen h-screen flex items-center justify-center bg-black overflow-hidden">
	<img
		ref="imageRef"
		v-if="sourceRef"
		:src="sourceRef"
		class="w-full h-full object-contain"
		:class="{'my-rough': roughRef}"
		:style="{filter: gammaRef == 1 ? 'none' : 'url(#my-gamma)'}"
	/>
	<div v-else class="absolute inset-0 flex items-center justify-center text-gray-400 italic select-none pointer-events-none">
		Viewer5 - io module, FileReader, Blob, img tag, SVG gamma filter option, CSS pixelated option
	</div>
</div>

</div>
</template>
<style scoped>

.my-rough {
	image-rendering: pixelated; /* use nearest-neighbor upsampling instead of the default high quality smooth interpolation algorithm */
}

</style>
