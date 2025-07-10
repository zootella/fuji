<script setup>//./components/Viewer4.vue

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {ioRead} from '../io.js'

const sourceRef = ref('')

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
	console.log(typeof bytes)//says "object"
	const blob = new Blob([bytes.buffer], {type: 'image/png'})//ttd july, set this correctly from file extension
	sourceRef.value = await asyncBlobToDataUrl(blob)
}

</script>
<template>

<div class="w-screen h-screen flex items-center justify-center bg-black overflow-hidden">
	<img
		v-if="sourceRef"
		:src="sourceRef"
		class="max-w-full max-h-full object-contain"
	/>
	<div v-else class="absolute inset-0 flex items-center justify-center text-gray-400 italic select-none pointer-events-none">
		Viewer4 - switched from img src to canvas for gamma and lightweight thumbnails
	</div>
</div>

</template>
<style scoped>

</style>
