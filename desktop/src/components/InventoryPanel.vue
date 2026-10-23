<script setup>//the inventory panel, which starts an inventory and shows how it goes; the inventory is inventory.js's, so it outlives the panel

import {computed, onMounted, onBeforeUnmount} from 'vue'
import {Time, saySize4, sayDuration} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in
import {inventoryProgress, inventoryStop, inventoryOptions, inventoryFolder, inventoryOutput, inventoryRunning, inventoryTrouble, inventoryCounts, inventoryTime, inventorySince, inventoryRun, sayCount, sayRate} from '../inventory.js'
import {dialogOpen} from '../dialog.js'
import {picturesFolder} from '../paths.js'
import {modelFolder} from '../model.js'//the folder the sheet shows, where the open box starts
import {logTrouble} from '../log.js'
import {brandName, brandStem} from '../brand.js'
import {backize, raf} from './library.js'

async function inventoryAsk() {
	try {
		let [chosen] = await dialogOpen({folder: true, start: modelFolder.value || await picturesFolder()})
		if (chosen) inventoryRun(chosen)//not awaited, since it resolves only when the inventory ends
	} catch (error) { logTrouble('inventory panel: choosing a folder', error) }
}
function stop() { inventoryStop(inventoryOutput.value).catch(error => logTrouble('inventory panel: stopping', error)) }

let showing = true
async function frames() {//once an animation frame, so not at all while the window is hidden
	while (showing) {
		if (inventoryRunning.value && inventoryOutput.value) {
			let progress = await inventoryProgress(inventoryOutput.value)
			if (progress && inventoryRunning.value) inventoryCounts.value = progress//null once it has ended, when inventory_take's answer is the last word
			if (inventoryRunning.value) inventoryTime.value = inventorySince()
		}
		await raf()
	}
}
onMounted(() => { frames().catch(error => logTrouble('inventory panel: reading how it is going', error)) })
onBeforeUnmount(() => { showing = false })//the frames stop, and the inventory goes on

const hourly = computed(() => inventoryTime.value >= Time.second || (!inventoryRunning.value && inventoryTime.value > 0 && !inventoryTrouble.value))//live from a second in, before which a few small files project to nonsense, and at the end however short the run
function sayProgress(read, size) {//rounded down, so 100% only ever means done
	let percent = size > 0 ? `${Math.floor(read / size * 100)}% ` : ''
	return ` (${percent}${saySize4(read)}/${saySize4(size)})`
}
function timeSay() {
	let said = sayDuration(inventoryTime.value, '#h #m #s')
	if (inventoryRunning.value || inventoryTrouble.value) return said
	return `${said}, ${inventoryCounts.value.stopped ? 'stopped' : 'done'}`
}

</script>
<template>

<div class="myMono bg-paper w-full h-full overflow-y-auto p-8 wrap-break-word"><!-- only a word too long for a line, like a path, breaks inside -->
	<h1 class="mb-8 text-strong">{{brandName}} Inventory</h1>
	<p>Takes an inventory of a folder and every folder in it, writing a line for each file to a text file in Documents: fuji's hash of its start, middle and end, its path and size, what the scout reads in its first bytes, and what the file system keeps about it.</p>
	<p class="mt-4 flex flex-wrap gap-x-4"><span>Also read every byte for</span>
		<label v-for="(name, key) in {sha1: 'SHA-1', sha256: 'SHA-256', blake3: 'BLAKE3'}" :key="key"><input type="checkbox" v-model="inventoryOptions[key]" :disabled="inventoryRunning"> {{name}}</label>
	</p>
	<p class="mt-2"><label><input type="checkbox" v-model="inventoryOptions.indent" :disabled="inventoryRunning"> Indent each file's JSON across lines</label></p>
	<p class="mt-2"><label><input type="checkbox" v-model="inventoryOptions.dated" :disabled="inventoryRunning"> Name the file for the moment it begins, rather than writing over {{brandStem}}-inventory.txt</label></p>
	<p class="mt-8 flex flex-wrap gap-2"><button class="myButton" :disabled="inventoryRunning" @click="inventoryAsk">Choose a folder…</button><button class="myButton" :disabled="!inventoryRunning || !inventoryOutput" @click="stop">Stop</button></p>
	<div v-if="inventoryFolder" class="mt-8 grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-1"><!-- minmax(0, 1fr), so a long path wraps rather than widening the column -->
		<span>Folder</span><span class="text-strong">{{backize(inventoryFolder)}}</span>
		<span>Writing</span><span class="text-strong">{{backize(inventoryOutput)}}</span>
		<span>Time</span><span>{{timeSay()}}</span>
		<span>Folders</span><span>{{sayCount(inventoryCounts.folders)}}</span>
		<span>Files</span><span>{{sayCount(inventoryCounts.files)}}<template v-if="hourly">{{sayRate(inventoryCounts.files, inventoryTime, sayCount)}}</template></span>
		<span>Data</span><span>{{saySize4(inventoryCounts.bytes)}}<template v-if="hourly">{{sayRate(inventoryCounts.bytes, inventoryTime, saySize4)}}</template></span>
		<template v-if="inventoryRunning && inventoryCounts.path">
			<span>Reading</span><span>{{backize(inventoryCounts.path)}}<template v-if="inventoryOptions.sha1 || inventoryOptions.sha256 || inventoryOptions.blake3">{{sayProgress(inventoryCounts.read, inventoryCounts.size)}}</template></span>
		</template>
	</div>
	<p v-if="inventoryTrouble" class="mt-8 text-warn">{{inventoryTrouble}}</p>
</div>

</template>
