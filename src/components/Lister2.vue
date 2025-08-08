<script setup>//./components/Lister2.vue - getting functionality ready for mvpersonal use

import {ref, onMounted, onBeforeUnmount, nextTick} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {ioRead, ioReadDir} from '../io.js'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path

const refLines = ref([])

function log(msg) {
	refLines.value.push(msg)
}

onMounted(async () => {
	const w = getCurrentWindow()
	unlistenFileDrop = await w.onDragDropEvent(async (event) => {
		if (event.payload.type == 'drop' && event.payload.paths.length) {
			let path = forwardize(event.payload.paths[0])

			await lookPath(path)
		}
	})
})
let unlistenFileDrop//will hold the unsubscribe function set above and called below
onBeforeUnmount(() => {
	if (unlistenFileDrop) unlistenFileDrop()
})

//forwardize all new paths that come into the system, and backize them for display to the user
function forwardize(path) {
	//rotate backslashes forward given what looks like a windows drive letter path; the forwardized path will still work with path-browserify and our rust io module code
	return /^[a-zA-Z]:[\\/]/.test(path) ? path.replace(/\\/g, '/') : path
}
function backize(path) {
	//but will look weird on windows, so use this in template code before showing to a Windows user
	return /^[a-zA-Z]:[\\/]/.test(path) ? path.replace(/\//g, '\\') : path
}

async function lookPath(path) {//given a path, return text all about it

	/*
	ok, you're right, this works on mac, but not windows
	ok so i want to write a little helper function which leaves unix style paths the same, but converts windows style paths into a format so that my code after this function can be cross platform
	i need these converted windows paths to still work as we use them, both with js here, with path-browserify, and with the rust functions in the style as i've showed them to you. i do not want to convert back and forth, for instance
	also, in scope is mac and windows paths, but we can leave windows network share style paths out of scope for now; we'll worry about supporting and testing those at a much later time
	*/

	let folder = parse.dirname(path)
	let raw = await ioReadDir(folder)

	log(`${path} <- path, backized to ${backize(path)}
${folder} <- folder, backized to ${backize(folder)}
`)









	/*
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
	*/

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



also, if you need to drop down to Rust to get plugin-fs to allow more paths
maybe just write all your own I/O io in Rust,
as there might be more plugin-fs limitations after this one
and as there aren't really many different things you need to do with I/O



*/

</script>
<template>

<div class="h-full overflow-y-auto bg-white p-4 font-mono text-xs leading-tight">
<div v-for="(line, idx) in refLines" :key="idx"><pre>{{line}}</pre></div>
</div>

</template>
<style scoped>

</style>
