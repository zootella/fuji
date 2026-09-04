//./src/cache.js

import parse from 'path-browserify'
import {diskRead} from './disk.js'
import {imageTypes} from './components/library.js'

/*
A store, not a strategy. It holds what the views tell it to hold and lets go when they say to let go. It does not decide, schedule, prioritise, or expire, and it knows nothing about folders, order, or who is asking. Every clever decision fuji makes about images lives in the view that is showing them; cache.md carries the long version of why.

Three shapes came before this one and each failed the same way: intelligence in the middle needs knowledge only the edges have. A shared queue has to be told whose request matters, which is the view's knowledge moved somewhere it does not belong — so there is no queue here, and every load races every other load, exactly as if the view had called the disk itself. A shared eviction policy has to know what the user is looking at — so there is no policy, and nothing is ever freed except on command.

What is left is worth one place precisely because it is not clever: a path's bytes, its object url, its decoded pixels, and what each of those cost.

Owning the url is the reason this file exists at all. library.js says why fuji avoided object urls until now — "URL.createObjectURL saves memory, but creates a resource that could leak" — and that is still true. An object url holds its blob alive until somebody revokes it, and nothing revokes it for you. Flip through two hundred large photographs minting a url each time and fuji is holding more than a gigabyte it will never get back. Here there is exactly one place urls are made and one place they are revoked, and no view touches either.

Two numbers say why the url is made once and never remade: pointing a second element at a url that already has pixels costs about a millisecond, and a fresh url over the same blob costs a full decode — 911ms for a 6240 by 4160 progressive jpeg on an M2. A store that minted a url per request would re-decode every time and look, from outside, exactly like a store that works.

This first iteration keeps only the natural-size decode, because a table is the only caller today. Small decodes for the sheet are the next entry point, and the place they land is beside img below.
*/

const cacheCeiling = 1024*1024*1024//a gigabyte held, past which something is probably wrong; not a limit, a line to complain at
const cacheStale = 5*60*1000//five minutes untouched while still referenced, which is what a leak looks like from here

const cacheEntries = new Map()//path to entry, and the only place fuji keeps images
let cacheBlobBytes = 0//running totals rather than a walk, because the hud reads them from the pan path
let cachePixelBytes = 0

export function cacheNeed(path, holder) {//take a reference and get the image; asking and holding are the same act, so nothing can be had without saying who wants it
	let entry = cacheEntries.get(path)
	if (!entry) {
		entry = {
			path,
			blob: null,//the file's bytes, held because the url is made from them
			url: '',//one object url over that blob, made once and never remade
			img: null,//the decoded image at its natural size; the grip that keeps the pixels wanted
			blobBytes: 0, pixelBytes: 0,//counted on the entry so dropping it can subtract exactly what it added
			references: new Map(),//holder name to how many times that holder has asked
			requested: Date.now(), loaded: 0, rendered: 0,//two durations: getting the bytes, then decoding them
			touched: Date.now(),//when it was last asked for, which is the only way to spot a holder that has forgotten about it
			error: null,
		}
		cacheEntries.set(path, entry)
		entry.promise = cacheLoad(entry)//two callers arriving together share this one load, which is bookkeeping rather than judgement
	}
	entry.references.set(holder, (entry.references.get(holder) || 0) + 1)
	entry.touched = Date.now()
	return entry.promise
}

export function cacheRelease(path, holder) {//give a reference back; when the last one goes, so does everything the store was holding
	let entry = cacheEntries.get(path)
	if (!entry) throw new Error(`released a path the cache does not have: ${holder} released ${path}`)//an exact mistake in the view above, caught where it happens rather than leaking quietly below
	let count = entry.references.get(holder)
	if (!count) throw new Error(`released a path this holder never needed: ${holder} released ${path}`)

	if (count > 1) entry.references.set(holder, count - 1)
	else entry.references.delete(holder)
	if (entry.references.size == 0) { cacheEntries.delete(path); cacheFree(entry) }
}

export function cacheFootprint() {//what fuji is holding, in the two units that differ by an order of magnitude and rebuild at completely different costs
	return {count: cacheEntries.size, blobs: cacheBlobBytes, pixels: cachePixelBytes}
}

export function cacheTrouble() {//everything that looks wrong, said plainly; walks the store, so ask rarely and never from a frame
	let trouble = []
	let held = cacheBlobBytes + cachePixelBytes
	if (held > cacheCeiling) trouble.push(`holding ${Math.round(held/1048576)} MiB, past the ${Math.round(cacheCeiling/1048576)} MiB line`)

	let now = Date.now()
	let forgotten = new Map()//holder to how many of its references nobody has asked about in a long time
	for (let entry of cacheEntries.values()) {
		if (now - entry.touched < cacheStale) continue
		for (let holder of entry.references.keys()) forgotten.set(holder, (forgotten.get(holder) || 0) + 1)
	}
	for (let [holder, count] of forgotten) trouble.push(`${holder} still holds ${count} images nothing has asked for in ${Math.round(cacheStale/60000)} minutes`)
	return trouble//nothing is freed here on purpose: a safety that tidied up after a leaking view would hide the bug it exists to show
}

async function cacheLoad(entry) {//read the file and decode it, recording what each half cost
	try {
		let bytes = new Uint8Array(await diskRead(entry.path))
		entry.loaded = Date.now()

		entry.blob = new Blob([bytes.buffer], {type: imageTypes[parse.extname(entry.path).toLowerCase()] || 'application/octet-stream'})//the array is not kept: making a blob copies, so holding both would be two copies of every file
		entry.blobBytes = entry.blob.size; cacheBlobBytes += entry.blobBytes
		entry.url = URL.createObjectURL(entry.blob)

		entry.img = new Image()
		entry.img.src = entry.url
		await entry.img.decode()//throws on data an image decoder cannot use
		entry.rendered = Date.now()
		entry.pixelBytes = entry.img.naturalWidth * entry.img.naturalHeight * 4//an estimate, and known to be low: a decoder may pad rows or keep a copy on the gpu
		cachePixelBytes += entry.pixelBytes
	} catch (error) {
		entry.error = error//remembered, so one broken file in a folder is not read again on every pass
	}
	if (cacheEntries.get(entry.path) != entry) cacheFree(entry)//released while it was still loading, so let go of what arrived after nobody wanted it any more
	return entry
}

function cacheFree(entry) {//the only place fuji revokes a url, and the only place it subtracts from the totals
	if (entry.url) { URL.revokeObjectURL(entry.url); entry.url = '' }
	if (entry.img) { entry.img.src = ''; entry.img = null }//let the engine take the pixels back
	entry.blob = null
	cacheBlobBytes -= entry.blobBytes; entry.blobBytes = 0
	cachePixelBytes -= entry.pixelBytes; entry.pixelBytes = 0
}
