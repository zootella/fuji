<script setup>//./components/Lister.vue

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {stat, readDir} from '@tauri-apps/plugin-fs'
import path from 'path-browserify'

const refLines = ref([])

function log(msg) {
	refLines.value.push(msg)
}

onMounted(async () => {
	log('hi from lister')
	const w = getCurrentWindow()
	unlistenFileDrop = await w.onDragDropEvent(async (event) => {
		if (event.payload.type === 'drop' && event.payload.paths.length) {
			let droppedPath = event.payload.paths[0]
			log(`\ndropped in path "${droppedPath}"...`)
			
			let s = await lookPath(path.dirname(droppedPath), true)
			log(s)
		}
	})
})
let unlistenFileDrop//will hold the unsubscribe function set above and called below
onBeforeUnmount(() => {
	if (unlistenFileDrop) unlistenFileDrop()
})

async function lookPath(p, descend) {//given a path, return text all about it
	let s = ``
	try {

		let i = await stat(p)
		s += `\n${p}`
		if (i.isDirectory) s += ' 🗂️'
		if (i.isFile) s += ' 📑'
		s += ` <${i.size}>`

		if (i.isDirectory) {

			let entries = await readDir(p)
			s += ` and contains ${entries.length}`

			if (descend) {
				for (let entry of entries) {
					let q = path.join(p, entry.name)
					s += await lookPath(q, false)
				}
			}
		}
	} catch (e) {
		s += `\n⚠️ caught exception "${e}" looking at path "${p}" ⚠️`
	}
	return s

}
/*
findings on mac:
home folder and below, can see files, but get exceptions trying to stat .DS_Store and .localized
dragging in a file from drive k works, but then you can't do the listing around it

dropped in path "/Users/kevin/Documents/folder1/7187Gc5WBwL.jpg"...

/Users/kevin/Documents/folder1 🗂️ <192> and contains 4
/Users/kevin/Documents/folder1/cat.jpg 📑 <427319>
/Users/kevin/Documents/folder1/fuji.png 📑 <17501>
/Users/kevin/Documents/folder1/7187Gc5WBwL.jpg 📑 <195706>
/Users/kevin/Documents/folder1/fuji.svg 📑 <162>

dropped in path "/Volumes/DriveK/folder1/folder2/cat.jpg"...

⚠️ caught exception "forbidden path: /Volumes/DriveK/folder1/folder2" looking at path "/Volumes/DriveK/folder1/folder2" ⚠️

so, more fiddling with permissions will be necessary
but this enough of a proof of concept to move on to neighboring functionality tests
*/

</script>
<template>

<div class="h-full overflow-y-auto bg-white p-4 font-mono text-xs leading-tight">
<div v-for="(line, idx) in refLines" :key="idx"><pre>{{line}}</pre></div>
</div>

</template>
<style scoped>

</style>
