//./src/flipCache.js

import {cacheNeed, cacheRelease} from './cache.js'
import {settings} from './settings.js'

/*
The diamond table's policy about which images to keep. cache.js is a store that holds what it is told and lets go when told; this is the part that does the telling, and it is where the cleverness about flipping belongs.

It replaces the triad. Three img elements held three decoded images, which is why flipping forward and back was instant and flipping twice forward and back was not: the element holding the image you came back to had been recycled, and its decode went with it. Here the horizon is as wide as fuji.toml says, and at flip.back and flip.forward of 1 the window holds the same three images the triad could reach — which is the point of them being settings rather than a number in this file, because it makes the old behaviour a value the user can return to.

What the triad was also doing, and this file keeps, is showing an element that already has its pixels. A flip is a display swap rather than a source assignment, which is what keeps it from flickering, and it is why the table shows the store's own element instead of pointing one of its own at the same picture. That would be worth doing even against a perfect store.

Loads race. There is no queue and no priority here, on purpose — the store has none either, and a table asking for eleven images is not the case that needs scheduling. A sheet asking for five hundred is, and that is the sheet's problem to solve when it exists.

Every reference this file takes is labelled, so if these images are ever held after the table has stopped wanting them, cacheTrouble names the table rather than reporting an anonymous pile of memory.
*/

const flipCacheHolder = 'DiamondTable'//the name that appears in cacheTrouble if these references are ever forgotten

let flipCacheHeld = new Map()//path to the promise the store gave back, for exactly the paths this window has a reference to

export function flipCacheWindow(list, index) {//the user is here: hold everything within reach and let go of everything outside it
	let want = new Map()
	for (let i = index - settings.flip.back; i <= index + settings.flip.forward; i++) {//read every time, so a number changed in fuji.toml means what it says
		if (i < 0 || i >= list.length) continue//the ends of a folder are simply a shorter window
		let path = list[i]
		want.set(path, flipCacheHeld.get(path) || cacheNeed(path, flipCacheHolder))//already held, or asked for now; the loads race each other and the operating system sorts them out
	}
	for (let path of flipCacheHeld.keys()) if (!want.has(path)) cacheRelease(path, flipCacheHolder)//out of reach, so the store may let it go if nobody else wants it
	flipCacheHeld = want
}

export function flipCacheImage(path) {//the image at a path this window is already holding
	let promise = flipCacheHeld.get(path)
	if (!promise) throw new Error('asked for an image outside the window, so flipCacheWindow was not called first: '+path)//a mistake in the table above, caught here rather than quietly taking a reference nothing will ever give back
	return promise
}

export function flipCacheClose() {//let go of everything, for a table being taken off the screen for good
	for (let path of flipCacheHeld.keys()) cacheRelease(path, flipCacheHolder)
	flipCacheHeld = new Map()
}
