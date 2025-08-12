<script setup>//./components/MyFlip.vue - image load pipeline and img triad

//keep, current best image pipeline, and img triad under construction here

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {xy, raf, blobToDataUrl, forwardize, backize, lookPath, readAndRenderImage} from './library.js'//our javascript library

onMounted(async () => {
	const w = getCurrentWindow()
	window.addEventListener('keydown', onKey)
	unlistenFileDrop = await w.onDragDropEvent(async (event) => {
		if (event.payload.type == 'drop' && event.payload.paths.length) {
			let path = event.payload.paths[0]
			await onDrop(path)
		}
	})
	onStart()
})
let unlistenFileDrop//will hold the unsubscribe function set above and called below
onBeforeUnmount(() => {
	window.removeEventListener('keydown', onKey)
	if (unlistenFileDrop) unlistenFileDrop()
})

async function onKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//ignore keystrokes into a form field

	if      (e.key == 'ArrowLeft')  { await onFlip(-1) }
	else if (e.key == 'ArrowRight') { await onFlip(1)  }
}

function onStart() {
	console.log('⭕ on start - once on startup, component loaded')
}
async function onDrop(path) {
	console.log(`⭕ on dropped path "${path}" - load and show right away`)

	let img = img8Ref.value//right now we just load everything into img8!

	let details = await readAndRenderImage(img, path)
	console.log(details.note)

	//style the img so it fills the container div, which will be the correct aspect ratio
	img.style.position = 'absolute'
	img.style.top = '0'
	img.style.left = '0'
	img.style.width = '100%'
	img.style.height = '100%'
	img.style.objectFit = 'contain'//letterbox for now; later will leave this out and size the container exactly right based on the natural width and height we got above

	img.style.display = ''//show the image now that it's ready; later will do this as part of the flip system
}
async function onFlip(direction) {
	console.log(`⭕ on command to flip ${direction > 0 ? 'forward' : 'back'} - flip immediately if ready, or upon loaded`)

}

function fillImage(imgRef, path) {//start loading the image on the disk at path into the given img7Ref, img8Ref, or img9Ref
	const image = {imgRef, path, promise: null, details: null}
	image.promise = readAndRenderImage(imgRef.value, path).then(details => {//await image.promise to wait for it to finish
		image.details = details//once image.promise is resolved, you can get details about the image here
		return details
	})
	return image//return the image object to await image.promise and then check out image.details
}




const triad = {prev: null, here: null, next: null}
/*
triad.prev, here, next
null means intentionally left blank
or there's an object, which looks like this:

{
	imgRef,//reference to img7Ref, 8, or 9; get to the img dom element with imgRef.value
	promise,//resolved once the image is loaded and ready to flip to it
	details,//information about how it loaded, including how long that took, the file size, the image's natural dimensions
}
*/

const img7Ref = ref(null)
const img8Ref = ref(null)
const img9Ref = ref(null)//our template contains these three img tags

</script>
<template>
<div>

<div class="relative w-screen h-screen bg-black overflow-hidden">
	<img ref="img7Ref" style="display: none;" />
	<img ref="img8Ref" style="display: none;" /><!-- three img tags for current (shown), previous (cached), and next (preloaded) -->
	<img ref="img9Ref" style="display: none;" />
</div>

</div>
</template>
<style scoped>

</style>
