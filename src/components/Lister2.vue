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

//forwardize all new paths that come into the system, then backize to show on the page
function forwardize(path) {
	//rotate backslashes forward given what looks like a windows drive letter path; the forwardized path will still work with path-browserify and our rust io module code
	return /^[a-zA-Z]:[\\/]/.test(path) ? path.replace(/\\/g, '/') : path
}
function backize(path) {
	//but will look weird on windows, so use this in template code before showing to a Windows user
	return /^[a-zA-Z]:[\\/]/.test(path) ? path.replace(/\//g, '\\') : path
}


const imageTypes = {
	'.bmp': 'image/bmp',//1986, Microsoft: Simple uncompressed raster format for Windows graphics, easy to decode
	'.gif': 'image/gif',//1987, CompuServe: 256-color palette with animation support, early web staple, now 😺🍔

	'.jpg':  'image/jpeg',//1992, Joint Photographic Experts Group: Lossy compression for photographs
	'.jpeg': 'image/jpeg',
	'.jpe':  'image/jpeg',
	'.jfif': 'image/jpeg',

	'.png':  'image/png',//1996, PNG Development Group/W3C: lossless compression and full alpha transparency
	'.svg':  'image/svg+xml',//2001, W3C: Scalable vector graphics for resolution-independent diagrams and icons
	'.avif': 'image/avif',//2019, Alliance for Open Media: from AV1 codec, supports HDR and wide color gamut
	'.webp': 'image/webp',//2010, Google: recent format for smaller file size
}
async function lookPath(path) {//given a path, return text all about it

	let folder = parse.dirname(path)
	let contents = await ioReadDir(folder)
	let files = contents.filter(f => f.is_file && !f.is_dir && !f.is_symlink)//only include files
	files = files.map(f => ({...f,
		path: parse.join(folder, f.name),
		extension: parse.extname(f.name).toLowerCase(),
	}))
	let images = files
		.filter(f => !f.name.startsWith('.'))//skip the .name.ext files macos makes for every file on a removable drive
		.filter(f => imageTypes[f.extension])//only include known extensions
		.map(f => ({
		...f,
		mime: imageTypes[f.extension],//include the mime type that goes with that extension
	}))

	log(`
${path} <- path
${folder} <- folder
found ${images.length} images
`)
	console.log(images)









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
