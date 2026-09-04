//./src/cache.js

import parse from 'path-browserify'
import {diskRead} from './disk.js'
import {imageTypes} from './components/library.js'

/*
One place that turns a path into pixels, and remembers. The sheet and every table ask it for images; it alone reads the disk. It knows nothing about folders, order, or who asked, which is what lets two views share it without either knowing the other exists. cache.md carries the reasoning and the measurements; this file is the mechanism.

An entry holds three references to one file, and each does something the other two cannot. The blob is the only way back to the bytes, because an img has a src string and no path back to what it was made from. The url is what a view puts in its own img.src, and is what the engine keys the decode on. The img is what keeps that decode wanted — a grip rather than a delivery, since holding it biases the engine toward keeping the pixels rather than guaranteeing it.

Object URLs are the part worth explaining, because fuji deliberately avoided them until now. The note in library.js says why: "URL.createObjectURL saves memory, but creates a resource that could leak." That was right for code with no cache. An object URL binds a blob to a string and holds it alive until somebody revokes it, and when nothing owns an image's life there is nobody to be that somebody, so a data URL — a plain string the collector handles — was the safe choice, at the cost of base64 making it four thirds the size of the file.

The cache is what makes the granular thing correct rather than risky, because it is the one owner. Every url in fuji is made here, on the line below where its blob is made, and released here when its entry goes. There is no second creator and no second releaser. What was an unowned resource waiting to leak is now the exact lever eviction needs: revoking a url releases the decode, dropping the blob releases the bytes, and the two can happen at different times, which is what lets a later policy throw away an expensive decode while keeping the cheap file it came from.

One rule has to hold or the cache silently stops being one: a url is made once per entry and never remade. Decodes are keyed by url and not by content, so a second createObjectURL over the same blob is a full decode again — 911ms for a 6240 by 4160 progressive jpeg on an M2, against 1ms for reusing the url that already has it. A cache that mints a url per request would re-decode every time and look, from the outside, exactly like a cache that works.

This first iteration never forgets: nothing expires, nothing is evicted, nothing is invalidated, and a file at a path is assumed to be the same file forever. That is not the end state. It is the state that produces honest numbers to design the end state from, which is what every timing and byte count below is for.
*/

const cacheWorkers = 4//how many reads and decodes run at once; a sheet opening a folder asks for everything it can see, and doing all of it at once means the image the user is looking at finishes last

const cacheEntries = new Map()//path to entry, and the only place fuji keeps images
let cacheFiles = 0//running totals rather than a walk, because the hud asks for these on the pan path
let cachePixels = 0

export function cacheImage(path) {//ask for an image; resolves to its entry, whose url a view sets as its own img src
	let entry = cacheEntries.get(path)
	if (entry) { entry.touched = Date.now(); return entry.promise }//already here, or already on its way: a second caller waits on the first one's work rather than starting a second read and a second decode of a file in flight

	entry = {
		path,
		blob: null,//the file's bytes, held once
		url: '',//one object url over that blob, made once and never remade
		img: null,//a decoded img, the grip on the pixels
		requested: Date.now(),
		loaded: 0,//when the bytes arrived
		rendered: 0,//when the decode finished
		touched: Date.now(),//when it was last handed to a caller
		error: null,//what went wrong, if anything did
	}
	cacheEntries.set(path, entry)
	entry.promise = cacheQueue(() => cacheFill(entry))
	return entry.promise
}

export function cacheFootprint() {//what fuji is holding, in the two units that matter, for the hud to show
	return {count: cacheEntries.size, files: cacheFiles, pixels: cachePixels}
}

async function cacheFill(entry) {//read the file, decode it, and record what each half cost
	try {
		let bytes = new Uint8Array(await diskRead(entry.path))
		entry.loaded = Date.now()

		entry.blob = new Blob([bytes.buffer], {type: imageTypes[parse.extname(entry.path).toLowerCase()] || 'application/octet-stream'})//the array is not kept: making a blob copies, so holding both would be two copies of every file
		entry.url = URL.createObjectURL(entry.blob)//made once, here, and revoked only when this entry goes
		entry.img = new Image()
		entry.img.src = entry.url
		await entry.img.decode()//throws on data an image decoder cannot use
		entry.rendered = Date.now()

		cacheFiles += entry.blob.size
		cachePixels += entry.img.naturalWidth * entry.img.naturalHeight * 4//an estimate, and known to be low: a decoder may pad rows or keep a copy on the gpu
	} catch (error) {
		entry.error = error//remembered, so one broken file in a folder is not retried on every scroll
	}
	entry.touched = Date.now()
	return entry
}

let cacheWaiting = []//work asked for and not started
let cacheWorking = 0//how many are running right now
function cacheQueue(work) {//hold work until a worker is free, so a folder's worth of requests arrive in order rather than all at once
	return new Promise((resolve, reject) => {
		cacheWaiting.push({work, resolve, reject})
		cacheStart()
	})
}
function cacheStart() {
	while (cacheWorking < cacheWorkers && cacheWaiting.length > 0) {
		let next = cacheWaiting.shift()
		cacheWorking++
		next.work()
			.then(next.resolve, next.reject)
			.finally(() => { cacheWorking--; cacheStart() })//a finished worker pulls the next thing waiting
	}
}
