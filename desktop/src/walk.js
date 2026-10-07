import {ref, shallowRef} from 'vue'
import {listFolder, peekFolder, walkFolders} from './components/library.js'
import {settings} from './settings.js'
import {log} from './log.js'

/*
The walk: every image on the volume as one list, and the contact sheet as a window onto it that moves by a page at a time.

Picture the output of find run at the top of the drive, kept to the images, each folder's contents sorted: one list, weaving into every folder and out again, with the whole collection on it somewhere. Next and Previous move the sheet down and up that list. They are not a browser's back and forward through where the user has been; they are the next and the previous stretch of the disk, so Previous from the first bucket of Downloads lands in the last bucket of Documents whether or not the user has ever looked at Documents, and Previous after three Nexts lands exactly where the user was before the third. Nothing is remembered but the position.

In the textbook's words the walk is an iterator over a tree in preorder, and the iterator itself is in Rust, disk_walk, with the stack the textbook gives it: handed a folder and a direction it reads the ancestors to find its place and then every folder once, and answers the next folders that hold an image, each with its count. This file is the page: it turns those folders into buckets, keeps the position, and looks ahead. Nothing below is cleverer than that.

The list is defined, and then never built. The definition: a folder contributes its own images, in the Sort's order, and then each of its subfolders in name order, each contributing the same way, which is preorder over the tree; a folder with no images contributes nothing; a folder whose name starts with a dot, or that is a symlink, is not entered, the second being how a walk loops forever. Every folder's images are cut into buckets of bucket.images from the folder's own start, so a bucket is a folder and an index into its images, and a page is sheet.buckets consecutive buckets. The top of the list is the volume's root, / on the Mac and the drive on Windows, and the bottom is the last image under the last folder. The opened folder is only where the walk starts.

The promise this file keeps is that the pages come out the same as they would from an implementation that scanned the whole drive into that list first and then cut it, for any arrangement of files and folders, from any start, after any sequence of presses. It holds because everything is derived from local facts that never depend on where the walk came from: a folder's own listing, sorted, and its parent's listing, sorted, and bucket boundaries anchored at each folder's start. The bucket after a position is the next bucket of the same folder if there is one, otherwise the first bucket of the next folder with images, and the next folder in preorder is the first subfolder, or else the next sibling by name, or else the parent's next sibling, climbing until one is found or the root runs out. The bucket before a position is the mirror image, and its one step with any thought in it is the folder before this one: if this folder has an earlier sibling, it is that sibling's deepest last descendant, reached by stepping into the last subfolder until a folder has none; if it has no earlier sibling, it is the parent itself, whose own images come before all of its subfolders. Both directions compute the same neighbor from the same two listings, so each undoes the other, and a folder the filter would not list, like a dot-folder the user opened on purpose, still has well defined neighbors, since siblings are found by comparing names rather than by looking the folder up among them.

What a press costs is directory reads, one per folder touched, never a scan. A step touches the folder's ancestors to find its place, and then the folders passed through, which is bounded by the depth of the tree and the number of imageless folders in the way, never by the size of the disk. Three kinds of read, and the difference is the whole cost. To pass through folders the page asks Rust to walk, disk_walk: one call, as many directory reads as there are folders in the stretch, no stat per entry, and an answer of the folders that hold an image with a count each, so a thousand imageless folders are a thousand readdirs and one trip across the bridge, where asking for each would have paid half a millisecond a folder on the trip alone. To count the buckets of the folder a page starts from it asks for a glance, disk_peek, the same read of one folder. To cut a bucket that will be on the page it asks for the full listing, with each file's name, size and modified time for the captions, which is a stat per entry and a record per entry across the bridge. The first walk over this Mac, on 2026-10-06, met a browser cache folder of forty-nine thousand files next to a folder of four pictures, and the full listing of it took seconds for a folder that would never be shown; a glance at it is one call. The first version of this file was the iterator without the stack, asking Rust for a glance at each folder and at a parent again for each of its subfolders it stepped through, which met a folder of nine thousand subfolders under Caches and spent fifty seconds on a step the disk could answer in five; the stack in Rust reads that parent once. Nothing is kept between presses, because a readdir reads the kernel's own caches rather than the disk, a press is a thing a user does a few times an hour, and a listing never remembered is never stale.

The look-ahead is what makes a press instant, and the buttons wait for it. Once a page is on screen, this file finds the page after it and the page before it in the background, a bounded stretch of the disk per call, and each button is enabled only once the look-ahead has found where its page starts, or disabled for good when it has found there is none. So a press never lists anything: the folders and the ranges of the next buckets are known, and only the thumbnails remain to load, which is the work the user actually asked for. It also means two presses can never overlap, since the second cannot happen until the first page is on screen and its own look-ahead has answered. The one thing that can still interrupt work in flight is a different folder dropped on the sheet, which starts a new walk, and a counter handed to each piece of work is how a look-ahead for a walk that has gone notices and stops.

Two things are known to hurt, and this version meets them honestly rather than cleverly. Imageless regions: walking forward out of a home folder reaches Library, tens of thousands of folders holding almost no pictures, and each costs a readdir before the next bucket appears; the buttons stay disabled while that happens and the look-ahead means it mostly happens before the press rather than after, and the memory report and the log say how many folders were read and how long that took, once it is done; no folder is named anywhere, since the names are the user's. The worst of it is the tail of the volume: past the last image the look-ahead has to walk to the very end to learn that Next has nowhere to go, so Next stays disabled for as long as that takes, and the top of the volume is the same for Previous. A skip list is the next version, and it is policy rather than mechanism, which is why it is not this one. Unreadable folders: a readdir that fails, under another user's home or where the system refuses, is passed through as empty, and the page's row in the log says how many were. One more thing is not this file's: a listed image that then fails to render. The walk's ranges are the listing's, so a bucket says Images 41 through 60 whatever rendered, shows the thumbnails that did, and should say how many did not; that is the bucket's caption, and rescanning to hide a failure would be complexity for nothing.

How the sheet uses this: it renders walkPage, with a bar of Previous and Next at each end of the page; the bucket and the flow take listing entries rather than bare paths, because the captions' date and size come from the entry and the walk's buckets are from many folders; the sheet calls walkRefresh when it comes back on screen, which re-cuts the page only when the bucket counts in settings have changed; and the table still flips within one folder, the one a double-click lists for it through the model, which is the one place the sheet and the table no longer share a list.
*/

//the two numbers in this file, each with its defense, since neither is a law

const walkExamine = 1000//how many folders one call to disk_walk may read before answering with where it stopped, so the page continues with another call. A press's reach is unbounded, since the last press before the end of a volume walks to the very end to learn that Next has nowhere to go, and the ceiling is what cuts that into pieces: between calls the count and the time on the page are published, and a walk whose folder has been replaced by a drop notices and stops, so the ceiling is how long either waits at most. A thousand small folders read from the kernel's cache is a fraction of a second on this Mac, and on a cold spinning disk perhaps a seek apiece, which is seconds but no more than the stretch would have cost in any number of calls; the trip across the bridge is half a millisecond, so against a thousand reads it is nothing, where against ten it was most of the cost. Larger would buy nothing but a staler readout; smaller would start paying the trip again. It is a count of folders rather than of entries, so a call that meets a folder of a hundred thousand files runs long by that one read, which is the read the walk had to make anyway
const walkPublishEvery = 250//milliseconds between updates of walkListed while listings run. Each update is a text change and a layout on the page's main thread, the same thread the table's frames run on, and a glance at a small folder is about half a millisecond, so publishing per listing would spend a fair share of the walk on the readout itself. A quarter second is about as fast as a person can read a changing number anyway, and the count is a diagnostic rather than a frame-critical readout, so it is published on that clock and once more, exactly, when the listings stop; faster would cost frames for no one, and slower would make the box look stuck during a long press

export const walkPage = shallowRef([])//the buckets on screen, in list order, each {folder, index, first, total, files}: where in the folder's images it starts, counted from 0 and from 1, how many the folder has, and the listing entries it shows
export const walkPreviousState = ref('none')//what the Previous button is: 'looking' while the look-ahead works behind the page, 'ready' once it has found where the previous page starts, and 'none' when there is nothing before this page
export const walkNextState = ref('none')//the same for Next
export const walkListed = ref({count: 0, milliseconds: 0})//what the disk was asked since the last start or press: how many folders were read and how long that took together, the page's own cut and the look-ahead both, a folder the walk passed through in Rust counted the same as one the page listed; published every walkPublishEvery milliseconds while the disk is being read, and once more, exactly, when it stops

let walkFirst = null//the position of the page's first bucket, {folder, index}, or null before a walk starts or on an empty volume
let walkAhead = []//the positions of the next page's buckets, once the look-ahead has found them; the button's state says whether it has
let walkBehind = []//the same for the previous page
let walkCount = 0; let walkMilliseconds = 0//the running totals behind walkListed, kept plain and published after each listing
let walkPages = 0//how many pages this run has shown, which is how the log's rows tell one page from the next without naming a folder
let walkCutWith = null//the bucket counts the page on screen was cut with, so a refresh under the same counts can leave the page and its known neighbors standing
let walkRefused = 0//how many folders the walk could not read since the last start or press, passed through as empty, for the log's row
let walkPublished = 0//when it was last published
let walkGeneration = 0//goes up with every start, press and refresh, and a copy goes with each piece of work, so a look-ahead or a page being cut for a walk that has since gone sees the number moved on at its next await and stops

/// Start a walk at a folder, showing its first bucket and what follows; the folder itself may hold no images, and the walk then begins at the first folder after it that does
export async function walkStart(folder) {
	if (typeof folder != 'string' || !folder.trim()) throw new Error(`walk: expected a folder: ${folder}`)
	let generation = ++walkGeneration
	await _walkShow(generation, await _walkPositionsFrom(generation, {folder, index: 0}, _walkSettings().buckets))
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
	let {images, buckets} = _walkSettings()
	if (walkCutWith && walkCutWith.images == images && walkCutWith.buckets == buckets) return//nothing to re-cut: the sheet comes back from the table many times an hour, and re-cutting the same page would run the look-ahead again through everything it already walked
	let generation = ++walkGeneration
	let first = {folder: walkFirst.folder, index: Math.floor(walkFirst.index / images) * images}//back onto the grid the folder's start lays down, which is where every bucket boundary is
	await _walkShow(generation, await _walkPositionsFrom(generation, first, buckets))
}

function _walkSettings() { return {images: settings.bucket.images, buckets: settings.sheet.buckets} }//read at every press rather than once, so a change in the settings panel is the next page's

async function _walkShow(generation, positions) {//the page from its positions, cut from their folders' listings; then the look-ahead, both ways, for the press after this one
	walkCount = 0; walkMilliseconds = 0; walkRefused = 0; walkListed.value = {count: 0, milliseconds: 0}//a fresh count for this page and its look-ahead; a start's own reads before this point are not counted, since they found where to begin rather than the page
	let page = await Promise.all(positions.map(position => _walkBucket(position)))//one listing per bucket, asked again; the essay says why that is fine
	if (generation != walkGeneration) return//a new walk owns the page now, checked after the last await so this one never lands on top of it
	walkFirst = positions[0] || null
	walkPage.value = page
	walkCutWith = _walkSettings()
	walkAhead = []; walkBehind = []
	walkPreviousState.value = walkFirst ? 'looking' : 'none'; walkNextState.value = walkFirst ? 'looking' : 'none'//both buttons wait for the look-ahead, and an empty volume has neither
	_walkLook(generation).catch(error => log(`walk: looking ahead, ${error}`))//not awaited: the page is on screen and this works the disk behind it
}

async function _walkLook(generation) {//find every bucket of the next page and of the previous page, so a press lists nothing, and enable each button as its page is found, or settle it when there is none
	let count = _walkSettings().buckets
	let last = walkPage.value[walkPage.value.length - 1]
	let ahead = last ? await _walkPositionsAfter(generation, last, count) : []
	if (generation != walkGeneration) return//a new walk started while this looked, and owns the buttons now
	walkAhead = ahead; walkNextState.value = ahead.length > 0 ? 'ready' : 'none'
	let behind = walkFirst ? await _walkPositionsBefore(generation, walkFirst, count) : []
	if (generation != walkGeneration) return
	walkBehind = behind; walkPreviousState.value = behind.length > 0 ? 'ready' : 'none'
	_walkPublish(true)//the final numbers, now that the walk is idle
	if (walkFirst) log(`walk: page ${++walkPages}, read ${walkCount} folders in ${Math.round(walkMilliseconds)} ms for the page and its look-ahead, ${walkRefused} unreadable, ${ahead.length ? 'next ready' : 'nothing after'}, ${behind.length ? 'previous ready' : 'nothing before'}`)//one row per page once both directions are known, which is what a walkabout leaves to read back when the log is recording. No folder is named, here or in the row below: the log is read by people and sessions the folders are none of, and a count and a time are the whole question
}

async function _walkPositionsFrom(generation, position, count) {//up to count positions from this one on, this one first, in list order, as many as the list has: the rest of this folder's buckets, then the buckets of the folders after it; a position past its folder's images, a start in a folder with none or a refresh after the folder shrank, contributes nothing and the list goes on from the folder
	let images = _walkSettings().images
	let positions = []
	let total = (await _walkPeek(position.folder)).images
	for (let index = position.index; index < total && positions.length < count; index += images) positions.push({folder: position.folder, index})
	await _walkFolders(generation, position.folder, true, count - positions.length, (folder, found) => { for (let index = 0; index < found && positions.length < count; index += images) positions.push({folder, index}) })
	return positions
}

async function _walkPositionsAfter(generation, position, count) {//up to count positions after this one, in list order, or none at all at the bottom: the folder's next bucket if it has one, else the first bucket of the next folder with images, and on from there
	let images = _walkSettings().images
	return _walkPositionsFrom(generation, {folder: position.folder, index: position.index + images}, count)
}

async function _walkPositionsBefore(generation, position, count) {//up to count positions before this one, in list order, or none at all at the top: the folder's earlier buckets, then the folders before it, each from its last bucket down, since the walk back hands them over in reverse; fewer than count means the page is against the top, and it is filled out from the page on screen so it stays a full page, overlapping this one
	let images = _walkSettings().images
	let positions = []//gathered walking backward, then turned around
	for (let index = position.index; index > 0 && positions.length < count;) { index = Math.max(0, index - images); positions.push({folder: position.folder, index}) }//the max only matters between a settings change and the refresh that puts the page back on the grid
	await _walkFolders(generation, position.folder, false, count - positions.length, (folder, found) => { for (let index = Math.floor((found - 1) / images) * images; index >= 0 && positions.length < count; index -= images) positions.push({folder, index}) })
	if (positions.length == 0) return []
	positions.reverse()
	for (let bucket of walkPage.value) { if (positions.length >= count) break; positions.push({folder: bucket.folder, index: bucket.index}) }
	return positions
}

async function _walkFolders(generation, folder, forward, want, each) {//the next want folders with images after this one in the list, or before it, each handed to the callback with its count, in walk order: Rust walks a stretch of at most walkExamine folders per call, and this continues from where each call stopped until it has enough, the volume ends, or the walk has been replaced by a drop, which its caller sees by the same counter and discards what was gathered. A call that fails is logged with its reason and ends the walk short, which the page shows as the end
	for (let from = folder; want > 0;) {
		let began = performance.now()
		let walk
		try { walk = await walkFolders(from, forward, want, walkExamine) }
		catch (error) { log(`walk: could not walk, ${error}`); return }//the error says why and never where
		if (generation != walkGeneration) return
		_walkListed(began, walk.examined); walkRefused += walk.refused
		for (let found of walk.found) each(found.folder, found.images)
		want -= walk.found.length
		if (walk.done) return
		from = walk.stopped
	}
}

async function _walkBucket(position) {//the bucket at a position, from the folder's listing: its slice of the images and the numbers its caption says
	let listing = await _walkList(position.folder)
	let images = _walkSettings().images
	return {folder: position.folder, index: position.index, first: position.index + 1, total: listing.images.length, files: listing.images.slice(position.index, position.index + images)}
}

async function _walkPeek(folder) {//a glance at a folder, how many images it holds, through the call that stats nothing, for the folder a page starts from: every other folder is counted by the walk in Rust. A folder that cannot be read is logged and passed through as empty
	let began = performance.now()
	let peek
	try { peek = await peekFolder(folder) }
	catch (error) { log(`walk: could not list a folder, ${error}`); peek = {images: 0, folders: []} }//the error says why, like permission denied, and never which; the folder is the user's
	_walkListed(began, 1)
	return peek
}

async function _walkList(folder) {//a folder's full listing, its images as entries sorted as the list orders them, for a bucket that will be on the page: the one listing that stats every file, since the captions need each one's size and modified time. Its count and _walkPeek's agree by the rule library.js and disk.rs share, which is what lets a bucket be counted by one and cut by the other
	let began = performance.now()
	let images
	try {
		images = await listFolder(folder)
		images.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0)//the Alphabet sort's own order, by path within one folder; when fuji has a second sort the model should order these, as it orders the table's folder
	} catch (error) {
		log(`walk: could not list a folder, ${error}`)
		images = []
	}
	_walkListed(began, 1)
	return {images}
}

function _walkListed(began, folders) { walkCount += folders; walkMilliseconds += performance.now() - began; _walkPublish(false) }//so many more folders read, a glance, a full listing or a stretch walked in Rust alike, for the count and the time the memory report and the log show

function _walkPublish(done) {//put the count and the time on the page, on a clock while listings run and once more, exactly, when they stop
	let now = performance.now()
	if (!done && now - walkPublished < walkPublishEvery) return
	walkPublished = now
	walkListed.value = {count: walkCount, milliseconds: Math.round(walkMilliseconds)}
}
