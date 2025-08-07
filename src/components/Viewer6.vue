<script setup>//./components/Viewer6.vue - decode image in script, create new element

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {ioRead} from '../io.js'

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

let loaded//details about the image we last loaded
async function loadImage(p) {//takes a path to an image on the disk
	loaded = {}
	loaded.t1 = performance.now()//start time
	loaded.path = p

	//read file and convert to data url
	let bytes = new Uint8Array(await ioRead(p))
	loaded.t2 = performance.now()//time spent in io from disk
	let blob = new Blob([bytes.buffer], {type: 'image/png'})
	let data = await asyncBlobToDataUrl(blob)
	loaded.t3 = performance.now()//time converting formats in memory
	loaded.size = bytes.length//byte size of file
	loaded.data = data//keep a reference to the data url even though we don't use it yet

	//make a new img tag and render the image into it
	let img = new Image()
	img.src = data
	try {
		await img.decode()//throws on problem with the image data
	} catch (e) { loaded.e = e; console.error(e); return }
	loaded.t4 = performance.now()//time rendering image to bitmap
	loaded.w = img.naturalWidth//and now we can get its pixel dimensions
	loaded.h = img.naturalHeight
	console.log(loaded)

	//style the img so it fills the container div, which will be the correct aspect ratio
	img.style.position = 'absolute'
	img.style.top = '0'
	img.style.left = '0'
	img.style.width = '100%'
	img.style.height = '100%'
	img.style.objectFit = 'contain'//letterbox for now; later will leave this out and size the container exactly right based on the natural width and height we got above

	containerRef.value.innerHTML = ''//discard any previously loaded img
	containerRef.value.appendChild(img)//put the img tag we made into the container, causing it to show up on the page
}

const containerRef = ref(null)

</script>
<template>
<div>

<div ref="containerRef" class="relative w-screen h-screen bg-black overflow-hidden">
	<!-- the img tag we made on load will get put in here to show up on the page! -->
</div>

</div>
</template>
<style scoped>

</style>
