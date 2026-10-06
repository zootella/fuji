import {ref, shallowRef} from 'vue'
import parse from 'path-browserify'
import {listDirectory} from './components/library.js'
import {settings} from './settings.js'
import {log} from './log.js'

/*
The walk: every image on the volume as one list, and the contact sheet as a window onto it that moves by a page at a time.

Picture the output of find run at the top of the drive, kept to the images, each folder's contents sorted: one list, weaving into every folder and out again, with the whole collection on it somewhere. Next and Previous move the sheet down and up that list. They are not a browser's back and forward through where the user has been; they are the next and the previous stretch of the disk, so Previous from the first bucket of Downloads lands in the last bucket of Documents whether or not the user has ever looked at Documents, and Previous after three Nexts lands exactly where the user was before the third. Nothing is remembered but the position.

In the textbook's words this file is an iterator over a tree in preorder, written without the stack: where the textbook remembers the path from the root, this lists the parent again when it needs a sibling. Nothing below is cleverer than that.

The list is defined, and then never built. The definition: a folder contributes its own images, in the Sort's order, and then each of its subfolders in name order, each contributing the same way, which is preorder over the tree; a folder with no images contributes nothing; a folder whose name starts with a dot, or that is a symlink, is not entered, the second being how a walk loops forever. Every folder's images are cut into buckets of bucket.images from the folder's own start, so a bucket is a folder and an index into its images, and a page is sheet.buckets consecutive buckets. The top of the list is the volume's root, / on the Mac and the drive on Windows, and the bottom is the last image under the last folder. The opened folder is only where the walk starts.

The promise this file keeps is that the pages come out the same as they would from an implementation that scanned the whole drive into that list first and then cut it, for any arrangement of files and folders, from any start, after any sequence of presses. It holds because everything is derived from local facts that never depend on where the walk came from: a folder's own listing, sorted, and its parent's listing, sorted, and bucket boundaries anchored at each folder's start. The bucket after a position is the next bucket of the same folder if there is one, otherwise the first bucket of the next folder with images, and the next folder in preorder is the first subfolder, or else the next sibling by name, or else the parent's next sibling, climbing until one is found or the root runs out. The bucket before a position is the mirror image, and its one step with any thought in it is the folder before this one: if this folder has an earlier sibling, it is that sibling's deepest last descendant, reached by stepping into the last subfolder until a folder has none; if it has no earlier sibling, it is the parent itself, whose own images come before all of its subfolders. Both directions compute the same neighbor from the same two listings, so each undoes the other, and a folder the filter would not list, like a dot-folder the user opened on purpose, still has well defined neighbors, since siblings are found by comparing names rather than by looking the folder up among them.

What a press costs is listings, one readdir per folder touched, never a scan. A step touches the folder itself, its parent to find a sibling, and the folders passed through, which is bounded by the depth of the tree and the number of imageless folders in the way, never by the size of the disk. Nothing is cached: a folder is listed every time it is asked about, several times in one press, once to count its images, once to cut each bucket from it and once to look past it, because a readdir reads the kernel's own caches rather than the disk, a press is a thing a user does a few times an hour, and a listing never remembered is never stale. For a folder of a few thousand pictures that is tens of milliseconds a press; a folder of a hundred thousand would make it seconds, and the answer then is a memo that lives for one press, not a cache that lives for the walk.

The look-ahead is what makes a press instant, and the buttons wait for it. Once a page is on screen, this file finds the page after it and the page before it in the background, one listing at a time, and each button is enabled only once the look-ahead has found where its page starts, or disabled for good when it has found there is none. So a press never lists anything: the folders and the ranges of the next buckets are known, and only the thumbnails remain to load, which is the work the user actually asked for. It also means two presses can never overlap, since the second cannot happen until the first page is on screen and its own look-ahead has answered; while that runs the buttons say which folder the walk is in. The one thing that can still interrupt work in flight is a different folder dropped on the sheet, which starts a new walk, and a counter handed to each piece of work is how a look-ahead for a walk that has gone notices and stops.

Two things are known to hurt, and this version meets them honestly rather than cleverly. Imageless regions: walking forward out of a home folder reaches Library, tens of thousands of folders holding almost no pictures, and each costs a readdir before the next bucket appears; walkLooking says which folder the walk is in while that happens, and the look-ahead means it mostly happens before the press rather than after. The worst of it is the tail of the volume: past the last image the look-ahead has to walk to the very end to learn that Next has nowhere to go, so the button stays at looking for as long as that takes, and the top of the volume is the same for Previous. A skip list is the next version, and it is policy rather than mechanism, which is why it is not this one. Unreadable folders: a readdir that fails, under another user's home or where the system refuses, is logged once and passed through as empty. One more thing is not this file's: a listed image that then fails to render. The walk's ranges are the listing's, so a bucket says Images 41 through 60 whatever rendered, shows the thumbnails that did, and should say how many did not; that is the bucket's caption, and rescanning to hide a failure would be complexity for nothing.

What hooking this up changes, when it is hooked up: the sheet renders walkPage instead of slicing the model's list, with Previous above the first bucket and Next between the last bucket and the memory report; the bucket and the flow take listing entries rather than bare paths, because the captions' date and size come from the entry and the walk's buckets are from many folders; the sheet calls walkRefresh when it comes back on screen, since the bucket counts are settings and the page re-cuts to them; and the table still flips within one folder, listed by the model as today, which is the one place the sheet and the table no longer share a list.
*/

export const walkPage = shallowRef([])//the buckets on screen, in list order, each {folder, index, first, total, files}: where in the folder's images it starts, counted from 0 and from 1, how many the folder has, and the listing entries it shows
export const walkLooking = ref('')//the folder being listed right now, for a line by the buttons while a press or the look-ahead works the disk; blank when idle
export const walkPreviousState = ref('none')//what the Previous button is: 'looking' while the look-ahead works behind the page, 'ready' once it has found where the previous page starts, and 'none' when there is nothing before this page
export const walkNextState = ref('none')//the same for Next

let walkFirst = null//the position of the page's first bucket, {folder, index}, or null before a walk starts or on an empty volume
let walkAhead = []//the positions of the next page's buckets, once the look-ahead has found them; the button's state says whether it has
let walkBehind = []//the same for the previous page
let walkGeneration = 0//goes up with every start, press and refresh, and a copy goes with each piece of work, so a look-ahead or a page being cut for a walk that has since gone sees the number moved on at its next await and stops

/// Start a walk at a folder, showing its first bucket and what follows; the folder itself may hold no images, and the walk then begins at the first folder after it that does
export async function walkStart(folder) {
	if (typeof folder != 'string' || !folder.trim()) throw new Error(`walk: expected a folder: ${folder}`)
	let generation = ++walkGeneration
	let first = {folder, index: 0}
	if ((await _walkList(folder)).images.length == 0) first = await _walkAfter(first)
	await _walkShow(generation, await _walkPositionsAfter(first, _walkSettings().buckets))
}

/// Show the next page, the buckets after the last one on screen; only once the look-ahead has found where it starts, which is when the button is enabled
export async function walkNext() {
	if (walkNextState.value != 'ready') return
	await _walkShow(++walkGeneration, walkAhead)
}

/// Show the previous page, the buckets before the first one on screen, the same way; at the top of the volume the page starts there and overlaps this one, as a window pulled up against a wall does
export async function walkPrevious() {
	if (walkPreviousState.value != 'ready') return
	await _walkShow(++walkGeneration, walkBehind)
}

/// Cut the page again from where it starts, under the bucket counts as they are now; the sheet calls this when it comes back on screen, since the settings panel may have changed them
export async function walkRefresh() {
	if (!walkFirst) return
	let generation = ++walkGeneration
	let {images, buckets} = _walkSettings()
	let first = {folder: walkFirst.folder, index: Math.floor(walkFirst.index / images) * images}//back onto the grid the folder's start lays down, which is where every bucket boundary is
	await _walkShow(generation, await _walkPositionsAfter(first, buckets))
}

function _walkSettings() { return {images: settings.bucket.images, buckets: settings.sheet.buckets} }//read at every press rather than once, so a change in the settings panel is the next page's

async function _walkShow(generation, positions) {//the page from its positions, cut from their folders' listings; then the look-ahead, both ways, for the press after this one
	let page = await Promise.all(positions.map(position => _walkBucket(position)))//one listing per bucket, asked again; the essay says why that is fine
	if (generation != walkGeneration) return//a new walk owns the page now, checked after the last await so this one never lands on top of it
	walkFirst = positions[0] || null
	walkPage.value = page
	walkAhead = []; walkBehind = []
	walkPreviousState.value = walkFirst ? 'looking' : 'none'; walkNextState.value = walkFirst ? 'looking' : 'none'//both buttons wait for the look-ahead, and an empty volume has neither
	_walkLook(generation).catch(error => log(`walk: looking ahead, ${error}`))//not awaited: the page is on screen and this works the disk behind it
}

async function _walkLook(generation) {//find every bucket of the next page and of the previous page, so a press lists nothing, and enable each button as its page is found, or settle it when there is none
	let count = _walkSettings().buckets
	let last = walkPage.value[walkPage.value.length - 1]
	let ahead = last ? await _walkPositionsAfter(await _walkAfter(last), count) : []
	if (generation != walkGeneration) return//a new walk started while this looked, and owns the buttons now
	walkAhead = ahead; walkNextState.value = ahead.length > 0 ? 'ready' : 'none'
	let behind = walkFirst ? await _walkPositionsBefore(walkFirst, count) : []
	if (generation != walkGeneration) return
	walkBehind = behind; walkPreviousState.value = behind.length > 0 ? 'ready' : 'none'
}

async function _walkPositionsAfter(first, count) {//up to count positions from this one on, this one first, as many as the list has; none from a null position, which is the bottom
	let positions = []
	for (let position = first; position && positions.length < count; position = await _walkAfter(position)) positions.push(position)
	return positions
}

async function _walkPositionsBefore(position, count) {//up to count positions before this one, in list order, or none at all at the top; fewer than count means the page is against the top, and it is filled out from the page on screen so it stays a full page, overlapping this one
	let positions = []
	for (let before = await _walkBefore(position); before && positions.length < count; before = await _walkBefore(before)) positions.unshift(before)
	if (positions.length == 0) return []
	for (let bucket of walkPage.value) { if (positions.length >= count) break; positions.push({folder: bucket.folder, index: bucket.index}) }
	return positions
}

async function _walkBucket(position) {//the bucket at a position, from the folder's listing: its slice of the images and the numbers its caption says
	let listing = await _walkList(position.folder)
	let images = _walkSettings().images
	return {folder: position.folder, index: position.index, first: position.index + 1, total: listing.images.length, files: listing.images.slice(position.index, position.index + images)}
}

async function _walkAfter(position) {//the bucket after this one, or null at the bottom of the list: the folder's next bucket, else the first bucket of the next folder with images
	let images = _walkSettings().images
	let listing = await _walkList(position.folder)
	if (position.index + images < listing.images.length) return {folder: position.folder, index: position.index + images}
	for (let folder = await _walkFolderAfter(position.folder); folder; folder = await _walkFolderAfter(folder)) {
		if ((await _walkList(folder)).images.length > 0) return {folder, index: 0}
	}
	return null
}

async function _walkBefore(position) {//the bucket before this one, or null at the top of the list: the folder's previous bucket, else the last bucket of the previous folder with images
	let images = _walkSettings().images
	if (position.index > 0) return {folder: position.folder, index: Math.max(0, position.index - images)}//the max only matters between a settings change and the refresh that puts the page back on the grid
	for (let folder = await _walkFolderBefore(position.folder); folder; folder = await _walkFolderBefore(folder)) {
		let total = (await _walkList(folder)).images.length
		if (total > 0) return {folder, index: Math.floor((total - 1) / images) * images}
	}
	return null
}

async function _walkFolderAfter(folder) {//the folder after this one in preorder, or blank at the end: its first subfolder, else the next sibling by name, else the parent's, climbing
	let subfolders = (await _walkList(folder)).folders
	if (subfolders.length > 0) return subfolders[0]
	for (let here = folder; here; here = _walkParent(here)) {
		let parent = _walkParent(here)
		if (!parent) return ''
		let name = parse.basename(here)
		let next = (await _walkList(parent)).folders.find(sibling => parse.basename(sibling) > name)//by name rather than by position in the list, so a folder the filter would not list still has a next
		if (next) return next
	}
	return ''
}

async function _walkFolderBefore(folder) {//the folder before this one in preorder, or blank at the top: the deepest last descendant of the earlier sibling by name, else the parent itself, whose images come before its subfolders
	let parent = _walkParent(folder)
	if (!parent) return ''
	let name = parse.basename(folder)
	let siblings = (await _walkList(parent)).folders
	let earlier = ''
	for (let sibling of siblings) { if (parse.basename(sibling) < name) earlier = sibling; else break }//the last one that sorts before this, in a list already sorted
	if (!earlier) return parent
	for (let subfolders = (await _walkList(earlier)).folders; subfolders.length > 0; subfolders = (await _walkList(earlier)).folders) earlier = subfolders[subfolders.length - 1]
	return earlier
}

function _walkParent(folder) {//the folder above, or blank at the top of the volume: / on the mac, and C:/ on windows, which path-browserify would reduce to C: and then to a dot, so both forms are caught here
	if (folder == '/' || /^[A-Za-z]:\/$/.test(folder)) return ''
	let parent = parse.dirname(folder)
	if (/^[A-Za-z]:$/.test(parent)) parent += '/'//the drive's root, spelled so readdir reads the root rather than the drive's current folder
	return parent == folder ? '' : parent
}

async function _walkList(folder) {//a folder's listing, images sorted as the list orders them and subfolders sorted by name, asked of the disk every time; a folder that cannot be read is logged and passed through as empty
	walkLooking.value = folder
	let listing
	try {
		listing = await listDirectory(folder)
		listing.images.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0)//the Alphabet sort's own order, by path within one folder; when fuji has a second sort the model should order these, as it orders the table's folder
		listing.folders.sort()//by name, since every path here shares the parent; the sorts never apply to folders
	} catch (error) {
		log(`walk: could not list ${folder}, ${error}`)
		listing = {images: [], folders: []}
	}
	walkLooking.value = ''
	return listing
}
