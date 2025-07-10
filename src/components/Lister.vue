<script setup>//./components/Lister.vue

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {stat} from '@tauri-apps/plugin-fs'

const refLines = ref([])

function log(msg) {
	refLines.value.push(msg)
}

onMounted(async () => {
	log('hi from lister')
	const w = getCurrentWindow()
	unlistenFileDrop = await w.onDragDropEvent(async (event) => {
		if (event.payload.type === 'drop' && event.payload.paths.length) {
			let p = event.payload.paths[0]
			console.log(`dropped path "${p}"`)
			await listPath(p)
		}
	})
})
let unlistenFileDrop//will hold the unsubscribe function set above and called below
onBeforeUnmount(() => {
	if (unlistenFileDrop) unlistenFileDrop()
})

async function listPath(p) {
	let info = await stat(p)
	log(`dropped in path "${p}" — ${info.size} bytes`)
}

</script>
<template>

<div class="h-full overflow-y-auto bg-white p-4 font-mono text-xs leading-tight">
<div v-for="(line, idx) in refLines" :key="idx">{{line}}</div>
</div>

</template>
<style scoped>

</style>
