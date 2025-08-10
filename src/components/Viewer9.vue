<script setup>//./components/Viewer8.vue - inmg triad

import {ref, onMounted, onBeforeUnmount, nextTick} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {ioRead} from '../io.js'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path

import {hello1} from './library.js'

onMounted(async () => {
	const w = getCurrentWindow()
	unlistenFileDrop = await w.onDragDropEvent(async (event) => {
		if (event.payload.type == 'drop' && event.payload.paths.length) {
			let path = event.payload.paths[0]
			await onDroppedPath(path)
		}
	})
})
let unlistenFileDrop//will hold the unsubscribe function set above and called below
onBeforeUnmount(() => {
	if (unlistenFileDrop) unlistenFileDrop()
})

async function onDroppedPath(path) {
	console.log(`dropped path "${path}"`)

	let details = await loadImage1(path)
	await loadImage2(img8Ref.value, details)//right now we just load everything into img8!

	console.log(details)
	console.log(`${details.t2 - details.t1}ms disk + ${details.t3 - details.t2}ms memory + ${details.t4 - details.t3}ms render`)

	console.log(`also, from the new library: ${hello1()}`)
}

async function loadImage1(path) {//read the file at path and get a data url string ready to render
	let details = {}
	details.t1 = performance.now()//start time
	details.path = path

	//read file and convert to data url
	let bytes = new Uint8Array(await ioRead(path))
	details.t2 = performance.now()//time spent in io from disk
	let blob = new Blob([bytes.buffer], {type: 'image/png'})
	let data = await blobToDataUrl(blob)//alternatively, URL.createObjectURL saves memory, but creates a resource that could leak
	details.t3 = performance.now()//time converting formats in memory
	details.size = bytes.length//byte size of file
	details.data = data//keep a reference to the data url even though we don't use it yet
	return details
}
async function loadImage2(img, details) {//render the data url string details.data into the given hidden img tag

	//load the data url into the given img tag and decode it
	img.src = details.data//setting this should cause an earlier call awaiting decode to throw, and this new call to work fine
	await img.decode()//throws on problem with the image data

	//success if there wasn't an exception from that
	details.t4 = performance.now()//time rendering image to bitmap
	details.w = img.naturalWidth//and now we can get its pixel dimensions
	details.h = img.naturalHeight

	//style the img so it fills the container div, which will be the correct aspect ratio
	img.style.position = 'absolute'
	img.style.top = '0'
	img.style.left = '0'
	img.style.width = '100%'
	img.style.height = '100%'
	img.style.objectFit = 'contain'//letterbox for now; later will leave this out and size the container exactly right based on the natural width and height we got above

	img.style.display = ''//show the image now that it's ready; later will do this as part of the flip system
}

const containerRef = ref(null)

const img7Ref = ref(null)
const img8Ref = ref(null)
const img9Ref = ref(null)//our template contains these three img tags
//we change where these module variables point to indicate how we're using them right now; consider flipping through a long list:
let imagePrev = {imgRef: img7Ref, state: 'Blank.', details: null, error: null, loadingPromise: Promise.resolve()}//hidden, but keeping around to make a flip back instant
let imageMain = {imgRef: img8Ref, state: 'Blank.', details: null, error: null, loadingPromise: Promise.resolve()}//only one of the three that's visible
let imageNext = {imgRef: img9Ref, state: 'Blank.', details: null, error: null, loadingPromise: Promise.resolve()}//preloading or preloaded to flip forward without a delay

function blankImage(i) {
	i.imgRef.value.src = ''//free the big base64 string
	i.imgRef.value.style.width = 0
	i.imgRef.value.style.height = 0//keep out of layout

	i.state = 'Blank.'
	i.details = null
	i.error = null
	i.loadingPromise = Promise.resolve()
}//ok if the only place you call blank is at the start of the next async load, then combine them
//and now you see that you want loadImage below to make an internal promise another invocation can look for and await
async function loadImage(i, path) {
	if (i.state != 'Blank.') throw new Error()//won't need this because you're going to blank it
	i.state = 'Loading.'

	let details
	try {
		details = await loadImagePathToRef(i.imgRef, path)
	} catch (e) {
		i.state = 'Error.'
		i.error = e
		i.details = details
		return
	}
	i.state = 'Ready.'
	i.details = details
}

async function flipImage(forward) {//direction forward true, reverse false
	let behind, here, ahead//we're here; this flip will cause us to step to ahead; we'll reuse behind as two notches ahead
	if (forward) { behind = imagePrev; here = imageMain; ahead = imageNext; }//flip forward to next, ahead; previous is behind
	else         { behind = imageNext; here = imageMain; ahead = imagePrev; }//flip back, so previous is ahead and next is behind

	blankImage(behind)
	/*no await*/loadImage(behind, )

	//todo, start the preload of the next next image now, in the behind slot which will be double ahead! 🤯 and don't await it
	await ahead.loadingPromise//set a resolved promise here if nothing to wait for
	//what happens if another flip command comes in here? discarded? queued? either is fine, but make sure you can spin the wheel to cause a race condition that breaks state!

	await raf()//pause here until right before the next repaint, so both display toggles apply in the same frame
	here.imgRef.value.style.display = 'none'//hide the currently visible image to move beyond it
	ahead.imgRef.value.style.display = ''//show the preloaded image ahead to move to it
	//and rewire our pointers to reflect the just changed state
	imagePrev = here//go back to get where we were
	imageMain = ahead//we moved ahead
	imageNext = behind//for double ahead, reuse behind which fell off the horizon
}

//promise helpers
const raf = () => new Promise(r => requestAnimationFrame(r))
function blobToDataUrl(blob) {//promisifed wrapper of FileReader's .readAsDataURL method
	let reader = new FileReader()
	let p = new Promise((resolve, reject) => {
		reader.onload  = () => resolve(reader.result)
		reader.onerror = () => reject(reader.error)
	})
	reader.readAsDataURL(blob)
	return p
}

</script>
<template>
<div>

<div ref="containerRef" class="relative w-screen h-screen bg-black overflow-hidden">
	<img ref="img7Ref" style="display: none;" />
	<img ref="img8Ref" style="display: none;" /><!-- three img tags for current (shown), previous (cached), and next (preloaded) -->
	<img ref="img9Ref" style="display: none;" />
</div>

</div>
</template>
<style scoped>

</style>
