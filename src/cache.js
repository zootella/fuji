//./src/cache.js

import parse from 'path-browserify'
import {diskRead} from './disk.js'
import {imageTypes} from './components/library.js'
import {meterLoad} from './meter.js'//the store already records what a load cost; this is only reporting it, and a load nothing ever flipped to counts too

/*
A store, not a strategy. It holds what the views tell it to hold and lets go when they say to let go. It does not decide, schedule, prioritise, or expire, and it knows nothing about folders, order, or who is asking. Every clever decision fuji makes about images lives in the view that is showing them; cache.md carries the long version of why.

Three shapes came before this one and each failed the same way: intelligence in the middle needs knowledge only the edges have. A shared queue has to be told whose request matters, which is the view's knowledge moved somewhere it does not belong — so there is no queue here, and every load races every other load, exactly as if the view had called the disk itself. A shared eviction policy has to know what the user is looking at — so there is no policy, and nothing is ever freed except on command.

What is left is worth one place precisely because it is not clever: a path's bytes, its object url, its decoded pixels, and what each of those cost.

An entry holds two things over one file. The blob is raw material: any decode at any size, and a hash later. The img is the finished thing, and a table shows that element itself rather than pointing an element of its own at the same picture — because pointing a second element at the same source costs the whole decode again. The measurement that said otherwise was taken on elements nobody was painting, and the running app overruled it.

The store owns the object url for as long as it owns the entry. library.js says why fuji avoided object urls until now — "URL.createObjectURL saves memory, but creates a resource that could leak" — and that is exactly right, so urls are made in one place here and revoked in one place, cacheFree, and no view touches either. Revoking the moment decode() resolves was tried and taken back out: a loaded element does go on displaying without its url, but an element the page is not showing can have its decoded frame dropped by the engine, and then showing it again has to rebuild from source. A revoked url leaves nothing cheap to rebuild through. Whether that is really happening here is what the store and paint halves of the flip on the hud are for; until they say, keeping the url costs one line in a function that already runs.

This first iteration decodes only at natural size, because a table is the only caller today. The sheet will want small ones, made from the same blob with createImageBitmap, and that is a second product beside img rather than a second store.
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
			blob: null,//the file's bytes, the raw material for any decode at any size and for a hash later
			url: '',//one object url over that blob, kept alive so re-showing the element is a rebuild from source rather than a full decode
			img: null,//the decoded element, which is the thing a table puts on screen
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

		entry.url = URL.createObjectURL(entry.blob)//kept until cacheFree, because a hidden element can lose its decoded frame and needs this source to get it back cheaply
		entry.img = new Image()
		entry.img.src = entry.url
		await entry.img.decode()//throws on data an image decoder cannot use
		if (entry.img) {//still ours: a release landing during the decode has already run cacheFree, which empties the entry, and measuring what it no longer holds would record a good file as a broken one
			entry.rendered = Date.now()
			entry.pixelBytes = entry.img.naturalWidth * entry.img.naturalHeight * 4//an estimate, and known to be low: a decoder may pad rows or keep a copy on the gpu
			cachePixelBytes += entry.pixelBytes
		}
	} catch (error) {
		entry.error = error//remembered, so one broken file in a folder is not read again on every pass
	}
	meterLoad(entry)//before the free below, while the entry still says what it cost
	if (cacheEntries.get(entry.path) != entry) cacheFree(entry)//released while it was still loading, so let go of what arrived after nobody wanted it any more
	return entry
}

function cacheFree(entry) {//the only place the store lets go, and the only place it subtracts from the totals
	if (entry.img) { entry.img.remove(); entry.img.src = ''; entry.img = null }//out of whatever card was showing it, then emptied so the engine can take the pixels back
	if (entry.url) { URL.revokeObjectURL(entry.url); entry.url = '' }//the only place fuji revokes a url, after the element that used it has let go
	entry.blob = null
	cacheBlobBytes -= entry.blobBytes; entry.blobBytes = 0
	cachePixelBytes -= entry.pixelBytes; entry.pixelBytes = 0
}
