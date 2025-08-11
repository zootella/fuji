<script setup>//./components/Viewer2.vue - FileReader and Blob, img tag

//delete because intermediate step

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {readFile} from '@tauri-apps/plugin-fs'

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
	const bytes = await readFile(p)// Uint8Array
	const blob = new Blob([bytes.buffer], {type: 'image/png'})
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
		Viewer2 - using FileReader and Blob instead of base64 for a 4/3rds memory saving, still no potential resource leak
	</div>
</div>

</template>
<style scoped>

</style>
