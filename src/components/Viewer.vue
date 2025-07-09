<script setup>//./components/Viewer.vue

import {ref, onMounted} from 'vue'
import {readFile} from '@tauri-apps/plugin-fs'

const hardPath = '/Users/kevin/Downloads/example.png'

const sourceRef = ref('')

onMounted(() => {
	loadImage()
})

async function loadImage() {
	// read the raw bytes from disk
	const bytes = await readFile(hardPath)
	// convert to a Base64 Data URL
	let s = ''
	for (const b of bytes) {
		s += String.fromCharCode(b)
	}
	sourceRef.value = `data:image/png;base64,${btoa(s)}`
}

</script>
<template>

<div class="w-screen h-screen flex items-center justify-center bg-black overflow-hidden">
	<img
			v-if="sourceRef"
			:src="sourceRef"
		class="max-w-full max-h-full object-contain"
	/>
</div>

</template>
<style scoped>

</style>
