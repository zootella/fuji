import {ref, shallowRef} from 'vue'
import parse from 'path-browserify'
import {listFolder, peekFolder} from './components/library.js'
import {settings} from './settings.js'
import {log} from './log.js'

/*
The walk: every image on the volume as one list, and the contact sheet as a window onto it that moves by a page at a time.

Picture the output of find run at the top of the drive, kept to the images, each folder's contents sorted: one list, weaving into every folder and out again, with the whole collection on it somewhere. Next and Previous move the sheet down and up that list. They are not a browser's back and forward through where the user has been; they are the next and the previous stretch of the disk, so Previous from the first bucket of Downloads lands in the last bucket of Documents whether or not the user has ever looked at Documents, and Previous after three Nexts lands exactly where the user was before the third. Nothing is remembered but the position.

In the textbook's words this file is an iterator over a tree in preorder, written without the stack: where the textbook remembers the path from the root, this lists the parent again when it needs a sibling. Nothing below is cleverer than that.

The list is defined, and then never built. The definition: a folder contributes its own images, in the Sort's order, and then each of its subfolders in name order, each contributing the same way, which is preorder over the tree; a folder with no images contributes nothing; a folder whose name starts with a dot, or that is a symlink, is not entered, the second being how a walk loops forever. Every folder's images are cut into buckets of bucket.images from the folder's own start, so a bucket is a folder and an index into its images, and a page is sheet.buckets consecutive buckets. The top of the list is the volume's root, / on the Mac and the drive on Windows, and the bottom is the last image under the last folder. The opened folder is only where the walk starts.

The promise this file keeps is that the pages come out the same as they would from an implementation that scanned the whole drive into that list first and then cut it, for any arrangement of files and folders, from any start, after any sequence of presses. It holds because everything is derived from local facts that never depend on where the walk came from: a folder's own listing, sorted, and its parent's listing, sorted, and bucket boundaries anchored at each folder's start. The bucket after a position is the next bucket of the same folder if there is one, otherwise the first bucket of the next folder with images, and the next folder in preorder is the first subfolder, or else the next sibling by name, or else the parent's next sibling, climbing until one is found or the root runs out. The bucket before a position is the mirror image, and its one step with any thought in it is the folder before this one: if this folder has an earlier sibling, it is that sibling's deepest last descendant, reached by stepping into the last subfolder until a folder has none; if it has no earlier sibling, it is the parent itself, whose own images come before all of its subfolders. Both directions compute the same neighbor from the same two listings, so each undoes the other, and a folder the filter would not list, like a dot-folder the user opened on purpose, still has well defined neighbors, since siblings are found by comparing names rather than by looking the folder up among them.

What a press costs is listings, one readdir per folder touched, never a scan. A step touches the folder itself, its parent to find a sibling, and the folders passed through, which is bounded by the depth of the tree and the number of imageless folders in the way, never by the size of the disk. Two kinds of listing, and the difference is the whole cost. To pass through a folder, or to count its buckets, the walk asks Rust for a glance, disk_peek: one directory read, no stat per entry, and an answer of one number and the subfolders' names. To cut a bucket that will be on the page it asks for the full listing, with each file's name, size and modified time for the captions, which is a stat per entry and a record per entry across the bridge. The first walk over this Mac, on 2026-10-06, met a browser cache folder of forty-nine thousand files next to a folder of four pictures, and the full listing of it took seconds for a folder that would never be shown; a glance at it is one call. A glance is remembered for the length of one press and its look-ahead, and then forgotten. The iterator without the stack glances at a parent once for each of its subfolders it steps through, which is nothing for a folder of twenty and everything for a folder of nine thousand: the first walk over this Mac met one under Caches, nine thousand glances each answering nine thousand names, and spent fifty seconds on a step the disk could have answered in five. Within a press the memo makes that parent one glance; across presses nothing is kept, because a readdir reads the kernel's own caches rather than the disk, a press is a thing a user does a few times an hour, and a listing never remembered is never stale.

The look-ahead is what makes a press instant, and the buttons wait for it. Once a page is on screen, this file finds the page after it and the page before it in the background, one listing at a time, and each button is enabled only once the look-ahead has found where its page starts, or disabled for good when it has found there is none. So a press never lists anything: the folders and the ranges of the next buckets are known, and only the thumbnails remain to load, which is the work the user actually asked for. It also means two presses can never overlap, since the second cannot happen until the first page is on screen and its own look-ahead has answered. The one thing that can still interrupt work in flight is a different folder dropped on the sheet, which starts a new walk, and a counter handed to each piece of work is how a look-ahead for a walk that has gone notices and stops.

Two things are known to hurt, and this version meets them honestly rather than cleverly. Imageless regions: walking forward out of a home folder reaches Library, tens of thousands of folders holding almost no pictures, and each costs a readdir before the next bucket appears; the buttons stay disabled while that happens and the look-ahead means it mostly happens before the press rather than after, and the memory report and the log say how many folders were listed and how long they took, once it is done; no folder is named anywhere, since the names are the user's. The worst of it is the tail of the volume: past the last image the look-ahead has to walk to the very end to learn that Next has nowhere to go, so Next stays disabled for as long as that takes, and the top of the volume is the same for Previous. A skip list is the next version, and it is policy rather than mechanism, which is why it is not this one. Unreadable folders: a readdir that fails, under another user's home or where the system refuses, is logged once and passed through as empty. One more thing is not this file's: a listed image that then fails to render. The walk's ranges are the listing's, so a bucket says Images 41 through 60 whatever rendered, shows the thumbnails that did, and should say how many did not; that is the bucket's caption, and rescanning to hide a failure would be complexity for nothing.

How the sheet uses this: it renders walkPage, with a bar of Previous and Next at each end of the page; the bucket and the flow take listing entries rather than bare paths, because the captions' date and size come from the entry and the walk's buckets are from many folders; the sheet calls walkRefresh when it comes back on screen, which re-cuts the page only when the bucket counts in settings have changed; and the table still flips within one folder, the one a double-click lists for it through the model, which is the one place the sheet and the table no longer share a list.
*/

//the two numbers in this file, each with its defense, since neither is a law

const walkGlancesMax = 20000//how many glances the per-press memo keeps before dropping the oldest. The memo exists for one case: stepping through a parent's subfolders glances at the parent once per step, so the parent's glance has to stay in the memo for as long as its children are being stepped through, and the children's own glances arrive in between and push it toward the back. So the number must exceed the most subfolders any one folder has, plus the glances taken under them along the way. The most seen is 8,971, one folder under this Mac's Library/Caches, with the next largest at 372, so 20,000 is twice the worst case seen with room for its children's children. Larger buys nothing until a folder has more than twenty thousand direct subfolders, and the cost of this many is a few megabytes of names, which an old 4 GB machine can carry; smaller would start re-glancing the big parent once per cap's worth of steps, which is still linear, so the number is a margin rather than a cliff. It is needed at all because a press's reach is unbounded: the last press before the end of a volume walks to the very end to learn that Next has nowhere to go, and on a drive of half a million folders the memo would otherwise hold them all
const walkPublishEvery = 250//milliseconds between updates of walkListed while listings run. Each update is a text change and a layout on the page's main thread, the same thread the table's frames run on, and a glance at a small folder is about half a millisecond, so publishing per listing would spend a fair share of the walk on the readout itself. A quarter second is about as fast as a person can read a changing number anyway, and the count is a diagnostic rather than a frame-critical readout, so it is published on that clock and once more, exactly, when the listings stop; faster would cost frames for no one, and slower would make the box look stuck during a long press

export const walkPage = shallowRef([])//the buckets on screen, in list order, each {folder, index, first, total, files}: where in the folder's images it starts, counted from 0 and from 1, how many the folder has, and the listing entries it shows
export const walkPreviousState = ref('none')//what the Previous button is: 'looking' while the look-ahead works behind the page, 'ready' once it has found where the previous page starts, and 'none' when there is nothing before this page
export const walkNextState = ref('none')//the same for Next
export const walkListed = ref({count: 0, milliseconds: 0})//what the disk was asked since the last start or press: how many folders were listed and how long those listings took together, the page's own cut and the look-ahead both; published every walkPublishEvery milliseconds while listings run, and once more, exactly, when they stop

let walkFirst = null//the position of the page's first bucket, {folder, index}, or null before a walk starts or on an empty volume
let walkAhead = []//the positions of the next page's buckets, once the look-ahead has found them; the button's state says whether it has
let walkBehind = []//the same for the previous page
let walkCount = 0; let walkMilliseconds = 0//the running totals behind walkListed, kept plain and published after each listing
let walkPages = 0//how many pages this run has shown, which is how the log's rows tell one page from the next without naming a folder
let walkCutWith = null//the bucket counts the page on screen was cut with, so a refresh under the same counts can leave the page and its known neighbors standing
const walkGlances = new Map()//folder to its glance, for the length of one press and its look-ahead: filled as the walk glances, read instead of asking again, and emptied when the look-ahead settles, so a parent is glanced once however many of its subfolders a press steps through, and no glance outlives the press that took it
let walkPublished = 0//when it was last published
let walkGeneration = 0//goes up with every start, press and refresh, and a copy goes with each piece of work, so a look-ahead or a page being cut for a walk that has since gone sees the number moved on at its next await and stops

/// Start a walk at a folder, showing its first bucket and what follows; the folder itself may hold no images, and the walk then begins at the first folder after it that does
export async function walkStart(folder) {
	if (typeof folder != 'string' || !folder.trim()) throw new Error(`walk: expected a folder: ${folder}`)
	let generation = ++walkGeneration
	let first = {folder, index: 0}
	if ((await _walkPeek(folder)).images == 0) first = await _walkAfter(first)
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
	let {images, buckets} = _walkSettings()
	if (walkCutWith && walkCutWith.images == images && walkCutWith.buckets == buckets) return//nothing to re-cut: the sheet comes back from the table many times an hour, and re-cutting the same page would run the look-ahead again through everything it already walked
	let generation = ++walkGeneration
	let first = {folder: walkFirst.folder, index: Math.floor(walkFirst.index / images) * images}//back onto the grid the folder's start lays down, which is where every bucket boundary is
	await _walkShow(generation, await _walkPositionsAfter(first, buckets))
}

function _walkSettings() { return {images: settings.bucket.images, buckets: settings.sheet.buckets} }//read at every press rather than once, so a change in the settings panel is the next page's

async function _walkShow(generation, positions) {//the page from its positions, cut from their folders' listings; then the look-ahead, both ways, for the press after this one
	walkCount = 0; walkMilliseconds = 0; walkListed.value = {count: 0, milliseconds: 0}//a fresh count for this page and its look-ahead; a start's own listings before this point are not counted, since they found where to begin rather than the page
	walkGlances.clear()//and fresh glances: the press keeps what it learns only until its look-ahead settles
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
	let ahead = last ? await _walkPositionsAfter(await _walkAfter(last), count) : []
	if (generation != walkGeneration) return//a new walk started while this looked, and owns the buttons now
	walkAhead = ahead; walkNextState.value = ahead.length > 0 ? 'ready' : 'none'
	let behind = walkFirst ? await _walkPositionsBefore(walkFirst, count) : []
	if (generation != walkGeneration) return
	walkBehind = behind; walkPreviousState.value = behind.length > 0 ? 'ready' : 'none'
	walkGlances.clear()//the press is over, and the next one asks the disk afresh
	_walkPublish(true)//the final numbers, now that the walk is idle
	if (walkFirst) log(`walk: page ${++walkPages}, listed ${walkCount} folders in ${Math.round(walkMilliseconds)} ms for the page and its look-ahead, ${ahead.length ? 'next ready' : 'nothing after'}, ${behind.length ? 'previous ready' : 'nothing before'}`)//one row per page once both directions are known, which is what a walkabout leaves to read back when the log is recording. No folder is named, here or in the row below: the log is read by people and sessions the folders are none of, and a count and a time are the whole question
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
	if (position.index + images < (await _walkPeek(position.folder)).images) return {folder: position.folder, index: position.index + images}
	for (let folder = await _walkFolderAfter(position.folder); folder; folder = await _walkFolderAfter(folder)) {
		if ((await _walkPeek(folder)).images > 0) return {folder, index: 0}
	}
	return null
}

async function _walkBefore(position) {//the bucket before this one, or null at the top of the list: the folder's previous bucket, else the last bucket of the previous folder with images
	let images = _walkSettings().images
	if (position.index > 0) return {folder: position.folder, index: Math.max(0, position.index - images)}//the max only matters between a settings change and the refresh that puts the page back on the grid
	for (let folder = await _walkFolderBefore(position.folder); folder; folder = await _walkFolderBefore(folder)) {
		let total = (await _walkPeek(folder)).images
		if (total > 0) return {folder, index: Math.floor((total - 1) / images) * images}
	}
	return null
}

async function _walkFolderAfter(folder) {//the folder after this one in preorder, or blank at the end: its first subfolder, else the next sibling by name, else the parent's, climbing
	let subfolders = (await _walkPeek(folder)).folders
	if (subfolders.length > 0) return subfolders[0]
	for (let here = folder; here; here = _walkParent(here)) {
		let parent = _walkParent(here)
		if (!parent) return ''
		let name = parse.basename(here)
		let next = (await _walkPeek(parent)).folders.find(sibling => parse.basename(sibling) > name)//by name rather than by position in the list, so a folder the filter would not list still has a next
		if (next) return next
	}
	return ''
}

async function _walkFolderBefore(folder) {//the folder before this one in preorder, or blank at the top: the deepest last descendant of the earlier sibling by name, else the parent itself, whose images come before its subfolders
	let parent = _walkParent(folder)
	if (!parent) return ''
	let name = parse.basename(folder)
	let siblings = (await _walkPeek(parent)).folders
	let earlier = ''
	for (let sibling of siblings) { if (parse.basename(sibling) < name) earlier = sibling; else break }//the last one that sorts before this, in a list already sorted
	if (!earlier) return parent
	for (let subfolders = (await _walkPeek(earlier)).folders; subfolders.length > 0; subfolders = (await _walkPeek(earlier)).folders) earlier = subfolders[subfolders.length - 1]
	return earlier
}

function _walkParent(folder) {//the folder above, or blank at the top of the volume: / on the mac, and C:/ on windows, which path-browserify would reduce to C: and then to a dot, so both forms are caught here
	if (folder == '/' || /^[A-Za-z]:\/$/.test(folder)) return ''
	let parent = parse.dirname(folder)
	if (/^[A-Za-z]:$/.test(parent)) parent += '/'//the drive's root, spelled so readdir reads the root rather than the drive's current folder
	return parent == folder ? '' : parent
}

async function _walkPeek(folder) {//a glance at a folder, how many images it holds and its subfolders sorted by name, through the call that stats nothing: from this press's memo when it has one, else asked of the disk and remembered; a folder that cannot be read is logged and passed through as empty
	let peek = walkGlances.get(folder)
	if (peek) return peek
	let began = performance.now()
	try {
		peek = await peekFolder(folder)
		peek.folders.sort()//by name, since every path here shares the parent; the sorts never apply to folders
	} catch (error) {
		log(`walk: could not list a folder, ${error}`)//the error says why, like permission denied, and never which; the folder is the user's
		peek = {images: 0, folders: []}
	}
	_walkListed(began)
	walkGlances.set(folder, peek)
	if (walkGlances.size > walkGlancesMax) walkGlances.delete(walkGlances.keys().next().value)//the oldest, since a map keeps insertion order
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
	_walkListed(began)
	return {images}
}

function _walkListed(began) { walkCount++; walkMilliseconds += performance.now() - began; _walkPublish(false) }//one more listing, a glance or a full one alike, for the count and the time the memory report and the log show

function _walkPublish(done) {//put the count and the time on the page, on a clock while listings run and once more, exactly, when they stop
	let now = performance.now()
	if (!done && now - walkPublished < walkPublishEvery) return
	walkPublished = now
	walkListed.value = {count: walkCount, milliseconds: Math.round(walkMilliseconds)}
}
