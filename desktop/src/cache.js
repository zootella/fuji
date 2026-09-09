import parse from 'path-browserify'
import {diskRead} from './disk.js'
import {imageTypes} from './components/library.js'
import {logLoad} from './log.js'//the log already records what a load cost; this is only reporting it, and a load nothing ever flipped to counts too

/*
A store, not a strategy. It holds what the views tell it to hold and lets go when they say to let go. It does not decide, schedule, prioritise, or expire, and it knows nothing about folders, order, or who is asking. Every clever decision fuji makes about images lives in the view that is showing them; cache.md carries the long version of why.

Three shapes came before this one and each failed the same way: intelligence in the middle needs knowledge only the edges have. A shared queue has to be told whose request matters, which is the view's knowledge moved somewhere it does not belong — so there is no queue here, and every load races every other load, exactly as if the view had called the disk itself. A shared eviction policy has to know what the user is looking at — so there is no policy, and nothing is ever freed except on command.

What is left is worth one place precisely because it is not clever: a path's bytes, its object url, its decoded pixels, and what each of those cost.

An entry holds two things over one file. The blob is raw material: any decode at any size, and a hash later. The img is the finished thing, and a table shows that element itself rather than pointing an element of its own at the same picture — because pointing a second element at the same source costs the whole decode again. The measurement that said otherwise was taken on elements nobody was painting, and the running app overruled it.

The store owns the object url for as long as it owns the entry. library.js says why fuji avoided object urls until now — "URL.createObjectURL saves memory, but creates a resource that could leak" — and that is exactly right, so urls are made in one place here and revoked in one place, cacheFree, and no view touches either. Revoking the moment decode() resolves was tried and taken back out: a loaded element does go on displaying without its url, but an element the page is not showing can have its decoded frame dropped by the engine, and then showing it again has to rebuild from source. A revoked url leaves nothing cheap to rebuild through. Whether that is really happening here is what the store and paint halves of the flip on the hud are for; until they say, keeping the url costs one line in a function that already runs.

No judgement about how much is too much lives here either. The store counts what it holds and will tell anyone who asks, but deciding that a number is alarming means knowing what the user is looking at, which is the same knowledge an eviction policy would need and the same reason it is not here. A view that wants to complain about its own footprint is the one with enough context to mean it.

A need says which steps it wants. The read is always taken, and the decode is a step a caller can leave out with {decode: false}, getting back the bytes and the url alone. TagFlow does that: it hands the engine an img of its own, so a decode here would be the same work done twice, with the engine's option of doing it lazily taken away. The element is decoded only at natural size; a small one made from the same blob would be another step beside it rather than a second store.
*/

const cacheEntries = new Map()//path to entry, and the only place fuji keeps images
let cacheBlobBytes = 0//running totals rather than a walk, because the hud reads them from the pan path
let cachePixelBytes = 0

export function cacheNeed(path, holder, steps = {}) {//take a reference and get the image; asking and holding are the same act, so nothing can be had without saying who wants it. steps lets a caller leave out work it has no use for: {decode: false} answers with the bytes and the url and never builds an element, for a view that hands the engine an img of its own
	let entry = cacheEntries.get(path)
	if (!entry) {
		entry = {
			path,
			blob: null,//the file's bytes, kept for a decode at another size or a hash later; nothing reads it yet, and it costs nothing, because the url below holds these bytes either way
			url: '',//one object url over that blob, kept alive so re-showing the element is a rebuild from source rather than a full decode
			img: null,//the decoded element, which is the thing a table puts on screen; null for as long as nobody has asked for one
			blobBytes: 0, pixelBytes: 0,//counted on the entry so dropping it can subtract exactly what it added
			references: new Map(),//holder name to how many times that holder has asked
			requested: performance.now(), loaded: 0, rendered: 0,//two durations: getting the bytes, then decoding them. performance.now everywhere, because these are intervals and it cannot jump the way a wall clock can
			error: null,
			reading: null, decoding: null,//the two steps as promises, each made once and shared by everyone who arrives while it runs, which is bookkeeping rather than judgement; decoding stays null until the first caller asks for an element
		}
		cacheEntries.set(path, entry)
		entry.reading = cacheRead(entry)
	}
	entry.references.set(holder, (entry.references.get(holder) || 0) + 1)

	let decode = steps.decode ?? true//the one step a caller can leave out
	if (!decode) return entry.reading//the same entry, resolved as soon as the bytes and the url are in hand
	if (!entry.decoding) entry.decoding = cacheDecode(entry)//the first caller to want an element starts the decode, on an entry that may have been read long ago for someone who wanted only the url
	return entry.decoding
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

async function cacheRead(entry) {//read the file into a blob and make its url, recording what that cost; every need waits on this, and a need that wants no element waits on nothing else
	try {
		let bytes = new Uint8Array(await diskRead(entry.path))
		entry.loaded = performance.now()

		entry.blob = new Blob([bytes.buffer], {type: imageTypes[parse.extname(entry.path).toLowerCase()] || 'application/octet-stream'})//the array is not kept: making a blob copies, so holding both would be two copies of every file
		entry.blobBytes = entry.blob.size; cacheBlobBytes += entry.blobBytes

		entry.url = URL.createObjectURL(entry.blob)//kept until cacheFree, because a hidden element can lose its decoded frame and needs this source to get it back cheaply
	} catch (error) {
		entry.error = error//remembered, so one broken file in a folder is not read again on every pass
	}
	if (cacheEntries.get(entry.path) != entry) { logLoad(entry, 'released while loading'); cacheFree(entry) }//released while it was still reading, so let go of what arrived after nobody wanted it any more; the row first, while the entry still says what it cost
	else if (!entry.decoding) logLoad(entry, 'bytes only')//nobody has asked for an element, so this is the whole load as far as anyone knows; a decode asked for later reports a row of its own
	return entry
}

async function cacheDecode(entry) {//decode the bytes into an element, once the read has them, recording what that cost
	let later = entry.loaded > 0//the read had already finished when this was asked for, so the log has a row for it and this row should say it is the same read
	await entry.reading
	if (!entry.url) return entry//the read failed, or the entry was released before it finished; either way there is nothing to decode, and the read has already reported
	try {
		entry.img = new Image()
		entry.img.src = entry.url
		await entry.img.decode()//throws on data an image decoder cannot use
		if (entry.img) {//still ours: a release landing during the decode has already run cacheFree, which empties the entry, and measuring what it no longer holds would record a good file as a broken one
			entry.rendered = performance.now()
			entry.pixelBytes = entry.img.naturalWidth * entry.img.naturalHeight * 4//an estimate, and known to be low: a decoder may pad rows or keep a copy on the gpu
			cachePixelBytes += entry.pixelBytes
		}
	} catch (error) {
		entry.error = error//remembered like a failed read: a file the decoder will not take is not tried again
	}
	if (cacheEntries.get(entry.path) != entry) { logLoad(entry, 'released while loading'); cacheFree(entry) }//released while it was still decoding
	else logLoad(entry, later ? 'decoded later' : '')
	return entry
}

function cacheFree(entry) {//the only place the store lets go, and the only place it subtracts from the totals
	if (entry.img) { entry.img.remove(); entry.img.src = ''; entry.img = null }//out of whatever card was showing it, then emptied so the engine can take the pixels back
	if (entry.url) { URL.revokeObjectURL(entry.url); entry.url = '' }//the only place fuji revokes a url, after the element that used it has let go
	entry.blob = null
	cacheBlobBytes -= entry.blobBytes; entry.blobBytes = 0
	cachePixelBytes -= entry.pixelBytes; entry.pixelBytes = 0
}
