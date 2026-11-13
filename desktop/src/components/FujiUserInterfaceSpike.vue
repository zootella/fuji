<script setup>//the bench fuji's bars, list and tree are built on: a menu bar and a toolbar across the top, and below them the tree and the list side by side, as a file manager lays them out, each with mock data the page makes for it, a bar between them that sizes them, and how long the list takes to build, filter and scroll, on screen and in the log

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {Time} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in
import {log, logTrouble} from '../log.js'
import {settings} from '../settings.js'
import {modelCompare} from '../model.js'
import {listMake, listRows, listFilter, listKey, listSelected} from '../list.js'
import {treeMake, treeRoots, treeKey} from '../tree.js'
import {sayListSize, sayListDate} from './library.js'
import {iconFor} from '../iconCache.js'
import {getCurrentWindow} from '@tauri-apps/api/window'
import FujiList from './FujiList.vue'
import FujiTree from './FujiTree.vue'
import FujiBar from './FujiBar.vue'
import MockMenu from './MockMenu.vue'

const counts   = [50, 200, 1000, 2000, 5000, 10000]
const count    = ref(50)
const filter   = ref('')
const building = ref(false)
const built    = ref('')//what the strip says about the last build, filter and scroll, which the log says too
const filtered = ref('')
const scrolled = ref('')
const selected = ref('')
const sorted   = ref('')

function say(n) { return n.toLocaleString('en-US') }
function trouble(error) { logTrouble('interface spike', error) }//for the panel's entry points, where a throw would otherwise vanish
function afterPaint() { return new Promise(resolve => requestAnimationFrame(() => setTimeout(resolve, 0))) }//once the change has painted: a frame callback runs before the engine lays out and paints, and a timeout set from inside it runs after

async function build(n) {
	building.value = true
	try {
		count.value = n
		let names = mockRows(n)//before the clock, since making mock rows is the bench's work and not the list's
		listRows(list, []); await afterPaint()//and the old list gone, so the number is the new list's alone
		let start = performance.now()
		listRows(list, names)
		await afterPaint()
		let ms = Math.round(performance.now() - start)
		built.value = `${say(n)} rows in ${ms}ms`
		log(`interface spike: built ${say(n)} rows in ${ms}ms`)
	} finally { building.value = false }
}
onMounted(() => build(count.value).catch(trouble))
onMounted(() => document.documentElement.style.setProperty('--list-carry', 'grabbing'))//the closed hand while the list carries a selection, for the spike, until the pointer is chosen
onBeforeUnmount(() => document.documentElement.style.removeProperty('--list-carry'))

async function onFilter(e) {
	let start = performance.now()
	filter.value = e.target.value
	listFilter(list, filter.value)
	await afterPaint()
	let ms = Math.round(performance.now() - start)
	let said = `"${filter.value}" shows ${say(list.shown.length)} of ${say(list.rows.length)} in ${ms}ms`
	filtered.value = said
	log(`interface spike: ${said}`)
}

let pressed = 0, waited = 0//when the last press reached the bench, which hears it before the list does, and how long it sat in line before that, from the moment the event was made
function press(e) { pressed = performance.now(); waited = pressed - e.timeStamp }
function onKey(e) { press(e); if (!listKey(list, e)) treeKey(tree, e) }//every key the shell hands this view goes on to the list or the tree, whichever has the keyboard, timed like a press
function onFocus(focused) { if (!focused) for (let menu of [menuRef, menuBarRef, toolbarRef]) menu.value?.hide() }//a menu closes when its window goes behind another, as the system's own do
defineExpose({onKey, onFocus})

const menuBarRef = ref(null), toolbarRef = ref(null)//the two bars across the top, which close what they have open as the window goes behind
const command = ref('')//the word the bars sent last
const viewsShown = ref('Thumbnails')//the mock Views panel's radio choice, which only the panel shows
const benchShowing = ref(false)//whether the strip of the bench's own controls and readouts shows, which the toolbar's Bench button turns on and off; hidden at first, and forgotten when the bench closes
function onCommand(word) {//every word from the bars, which the bench shows, and answers where it has an answer
	command.value = word
	log(`interface spike: command ${word}`)
	if (word == 'Bench') benchShowing.value = !benchShowing.value
}

const menuRef = ref(null)
function onMenu(at) {//the window to the front first, as the sheet's own right click does, since webkit follows the pointer only in the window in front
	getCurrentWindow().setFocus().catch(trouble)
	menuRef.value.show(at.x, at.y, listSelected(list).length > 0 ? 'Files' : 'Folder')//the folder's own menu when the click left nothing selected
}
async function onSelect(p) {//from that press to the selection painted, so the list's own marking is inside the number; a carry, and a click on a row already selected, report on the release, so for those the number includes how long the button was held
	let start = pressed, queued = waited//read before the wait, since a press queued behind this one can reach the bench during it
	await afterPaint()
	let ms = Math.round(performance.now() - start)
	selected.value = `${say(p.selected)} rows in ${ms}ms`
	log(`interface spike: ${p.kind} press selected ${say(p.selected)} of ${say(list.shown.length)} rows, ${say(p.changed)} changed, waited ${Math.round(queued)}ms in line, list ${p.listMs.toFixed(1)}ms, painted at ${ms}ms`)
}

async function onSort(p) {//from the start of the list's own work to the sorted list painted
	let start = performance.now() - p.listMs
	await afterPaint()
	let ms = Math.round(performance.now() - start)
	let said = `${say(list.rows.length)} rows by ${p.key}${p.descending ? ' descending' : ''} in ${ms}ms`
	sorted.value = said
	log(`interface spike: sorted ${said}, list ${p.listMs.toFixed(1)}ms`)
}

let scrolling = false, scrollLast = 0, scrollGaps = [], scrollQuiet = null
const scrollRest = 300*Time.millisecond//how long with no scroll event ends a scroll
function onScroll() {
	if (!scrolling) { scrolling = true; scrollLast = 0; scrollGaps = []; requestAnimationFrame(scrollFrame) }
	clearTimeout(scrollQuiet); scrollQuiet = setTimeout(scrollEnd, scrollRest)
}
function scrollFrame(now) {//once a frame while a scroll lasts, and never otherwise, so the measuring costs nothing at rest
	if (!scrolling) return
	if (scrollLast) scrollGaps.push(now - scrollLast)
	scrollLast = now
	requestAnimationFrame(scrollFrame)
}
function scrollEnd() {
	scrolling = false
	if (scrollGaps.length < 2) return
	let sorted = [...scrollGaps].sort((a, b) => a - b)
	let interval = sorted[Math.floor(sorted.length / 2)]//the display's own frame interval, read as the median gap, so a 60 Hz and a 120 Hz display each measure against themselves
	let late = scrollGaps.filter(gap => gap > interval * 1.5).length
	let longest = Math.round(sorted[sorted.length - 1])
	scrolled.value = `${scrollGaps.length} frames, longest ${longest}ms, ${late} late`
	log(`interface spike: scrolled ${say(list.shown.length)} rows over ${scrollGaps.length} frames of ${interval.toFixed(1)}ms, longest ${longest}ms, ${late} late`)
}
onBeforeUnmount(() => { scrolling = false; clearTimeout(scrollQuiet) })

const columns = [//the settings read as each row is made, so a change shows the next time the bench opens
	{key: 'name', title: 'Name', compare: modelCompare, width: 320},//names in fuji's one sort, the order every list and view in fuji shows
	{key: 'size', title: 'Size', say: bytes => sayListSize(bytes, settings), align: 'right', tabular: settings.font.capitals, width: 128},//digits of one width only while small capitals are on, and otherwise the face's own
	{key: 'modified', title: 'Date', say: tick => sayListDate(tick, settings), tabular: settings.font.capitals, width: 152},
]
const list = listMake({identity: 'name', columns, onSelect: p => onSelect(p).catch(trouble), onSort: p => onSort(p).catch(trouble), onMenu, onTrouble: error => logTrouble('fuji list', error)})//the mock names are unique, so a name is a row's identity

const treeMade = ref('')//what the strip says about the tree: how many folders it made and how long that took, and the name of the one selected
const treeSaid = ref('')
const tree = treeMake({identity: 'path', compare: modelCompare, onSelect: folder => { treeSaid.value = folder.name }, onTrouble: error => logTrouble('fuji tree', error)})//a mock folder's path is its identity, since names repeat in different folders and paths do not
onMounted(() => {
	let folders = mockFolders()//before the clock, as the rows are
	let start = performance.now()
	treeRoots(tree, folders)
	let ms = Math.round(performance.now() - start)
	treeMade.value = `${say(tree.rows.length)} folders in ${ms}ms`
	log(`interface spike: made a tree of ${say(tree.rows.length)} folders in ${ms}ms`)
})

const treeWidth = ref(260)//css pixels, the tree's share of the panel's width, which the bar between it and the list changes
const splitNarrowest = 120//css pixels, the narrowest the bar leaves the tree or the list
function onSplit(e) {//a press on the bar sizes the tree as the pointer moves, and the list takes the rest
	if (e.button != 0) return
	let bar = e.currentTarget, startX = e.clientX, startWidth = treeWidth.value, room = bar.previousElementSibling.offsetWidth + bar.nextElementSibling.offsetWidth//the tree and the list together, which the drag divides between them
	bar.setPointerCapture(e.pointerId)//so the drag keeps the pointer outside the bar and outside the window
	let move = m => { treeWidth.value = Math.round(Math.min(Math.max(startWidth + m.clientX - startX, splitNarrowest), room - splitNarrowest)) }
	let end = () => { bar.removeEventListener('pointermove', move); bar.removeEventListener('pointerup', end); bar.removeEventListener('pointercancel', end) }
	bar.addEventListener('pointermove', move)
	bar.addEventListener('pointerup', end)
	bar.addEventListener('pointercancel', end)
}

let mockMade = []//made at the first build each time the panel opens
function mockRows(n) {
	if (!mockMade.length) {
		let random = seeded(2001), randomWhen = seeded(1995)//generators apart from the names' own and from each other, so giving rows sizes and then dates left every name and size as it was
		let first = Date.UTC(1995, 0, 1), last = Date.UTC(2026, 9, 28)//fixed, so the dates are the same on every run
		mockMade = mockNames(counts[counts.length - 1]).map(name => ({
			name,
			size: Math.floor(2 ** (random() * 37)),//spread evenly by order of magnitude, from a byte to 128 gigabytes, so short sizes and very long ones sit side by side
			modified: first + Math.floor(randomWhen() * (last - first)),//a tick, milliseconds since 1970 in utc
			icon: iconFor({extension: name.slice(name.lastIndexOf('.') + 1)}),//the system's icon for the extension every mock name ends in, as if it were a real file
		}))
	}
	return mockMade.slice(0, n)//the same rows at every count, in the order they were made, as a disk hands over a folder; the list puts them in fuji's sort
}
function mockNames(n) {//names like a collection gathers, each unique, and the same on every run and every machine
	let random = seeded(1984)
	let pick = list => list[Math.floor(random() * list.length)]
	let digits = (length, base) => Array.from({length}, () => Math.floor(random() * base).toString(base)).join('')
	let two = number => String(number).padStart(2, '0')
	let words = ['summer', 'final', 'wallpaper', 'scan', 'cover', 'back', 'render', 'sketch', 'untitled', 'new', 'copy', 'old', 'best', 'cat', 'sunset', 'map', 'level', 'title', 'intro', 'menu', 'boss', 'fanart', 'banner', 'avatar', 'manual', 'page', 'disc', 'track', 'episode', 'trailer', 'demo', 'mix', 'live', 'remaster', 'poster']
	let foreign = ['夏の空', 'ねこ', '東京タワー', 'Ölgemälde', 'Straße', 'café', 'crème brûlée', 'niño', 'año nuevo', 'Москва', 'зима', '안녕하세요', '风景', 'İstanbul', 'Ελλάδα']
	let extensions = ['jpg', 'jpg', 'jpg', 'png', 'png', 'gif', 'webp', 'bmp', 'tif', 'jpeg', 'mp4', 'mkv', 'avi', 'mov', 'mp3', 'flac', 'wav', 'pdf', 'txt', 'zip']
	let runs = {}//the last number in each numbered run
	let made = new Set()
	while (made.size < n) {
		let r = random(), name
		if      (r < 0.15) name = digits(32, 16)//a hash, as boorus name their files
		else if (r < 0.30) name = `1${digits(12, 10)}`//milliseconds since 1970, as imageboards do
		else if (r < 0.50) { let word = pick(words); runs[word] = (runs[word] || 0) + 1; name = `${word}${pick(['', '_', ' ', '-'])}${String(runs[word]).padStart(3, '0')}` }
		else if (r < 0.70) name = Array.from({length: 1 + Math.floor(random() * 4)}, () => pick(words)).join(pick([' ', '_', '-'])) + (random() < 0.2 ? ` (${2 + Math.floor(random() * 5)})` : '')
		else if (r < 0.78) name = Array.from({length: 8 + Math.floor(random() * 7)}, () => pick(words)).join(' ')
		else if (r < 0.88) name = pick(foreign) + (random() < 0.7 ? ` ${digits(2, 10)}` : '')
		else if (r < 0.96) name = `Screenshot ${2004 + Math.floor(random() * 22)}-${two(1 + Math.floor(random() * 12))}-${two(1 + Math.floor(random() * 28))} at ${two(Math.floor(random() * 24))}.${two(Math.floor(random() * 60))}.${two(Math.floor(random() * 60))}`
		else               name = digits(1 + Math.floor(random() * 2), 36)
		made.add(`${name}.${pick(extensions)}`)
	}
	return [...made]
}
function mockFolders() {//folders for the tree, chosen by hand so each kind of row is there: folders holding folders, folders with nothing inside, a deep run, a long row of siblings, several roots and a root by itself with nothing in it, a name too long for the tree's width, and names in other alphabets
	let folder = (name, ...children) => ({name, children})
	let roots = [
		folder('Pictures',
			folder('1999', folder('Summer'), folder('Winter')),
			folder('2004', folder('Screenshots'), folder('Wallpapers', folder('Anime'), folder('Space'))),
			folder('Fanart', folder('Artists', folder('A', folder('Akira', folder('Sketches', folder('Old', folder('Older'))))))),
			folder('Scans',
				folder('Manuals', folder('PlayStation'), folder('Sega Saturn')),
				folder('Magazines', folder('Next Generation', folder('1996'), folder('1997'), folder('1998')))),
			folder('Unsorted'),
			folder('夏の空'),
			folder('A folder with a name long enough to run past the right edge of the tree, so the tree scrolls sideways to show it')),
		folder('Downloads',
			folder('Album', folder('Disc 1'), folder('Disc 2')),
			folder('New Folder'),
			folder('New Folder (2)'),
			folder('Москва')),
		folder('Archive',
			folder('Backup 2014', folder('Old')),
			...[2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023, 2024, 2025, 2026].map(year => folder(`Backup ${year}`))),
		folder('Empty Drive'),
	]
	let shape = (f, above) => {//each folder as the tree reads it, the way an app would hand over its own records: an identity, a name, an icon and the folders inside
		let path = `${above}/${f.name}`
		return {path, name: f.name, icon: iconFor({folder: true}), children: f.children.map(child => shape(child, path))}
	}
	return roots.map(f => shape(f, ''))
}
function seeded(seed) {//mulberry32, a small generator that starts from a seed, which Math.random cannot
	return () => {
		let t = seed = seed + 0x6d2b79f5 | 0
		t = Math.imul(t ^ t >>> 15, t | 1)
		t ^= t + Math.imul(t ^ t >>> 7, t | 61)
		return ((t ^ t >>> 14) >>> 0) / 4294967296
	}
}

</script>
<template>

<div class="bg-zed-editor-background text-zed-text-muted w-full h-full flex flex-col" @pointerdown.capture="press">
	<FujiBar ref="menuBarRef" name="Menu Bar" class="flex-none border-b border-zed-border" @command="onCommand" /><!-- on every platform while the bars are designed; at the very end the mac leaves this row out, for its own menu bar at the top of the screen -->
	<FujiBar ref="toolbarRef" name="Toolbar" class="flex-none border-b border-zed-border" :pressed="benchShowing ? ['Bench'] : []" @command="onCommand">
		<template #panel="{panel}">
			<div v-if="panel == 'Views Panel'" class="flex items-start gap-4 p-3"><!-- the first panel, a mock: a radio group, which the page draws itself, beside a small toolbar of its own whose button opens a menu -->
				<div class="flex flex-col gap-1">
					<label v-for="view in ['Thumbnails', 'Tiles', 'Details']" :key="view" class="flex items-baseline gap-2"><input type="radio" name="viewsShown" :value="view" v-model="viewsShown"><span>{{view}}</span></label>
				</div>
				<FujiBar name="Views Sort" @command="onCommand" />
			</div>
		</template>
	</FujiBar>
	<div v-show="benchShowing" class="mySans text-[11px] flex-none h-[28px] flex items-center gap-x-4 px-4 whitespace-nowrap overflow-hidden border-b border-zed-border [&>*]:flex-none"><!-- one line of a set height, so readouts that grow and shrink never move the tree and the list below; what runs past the right edge is cut off -->
		<span>Tree <span class="text-zed-text">{{treeMade}}</span></span>
		<span>Selected <span class="text-zed-text">{{treeSaid}}</span></span>
		<span class="flex gap-1"><button v-for="n in counts" :key="n" class="myButton" :aria-pressed="n == count" :disabled="building" @click="build(n).catch(trouble)">{{say(n)}}</button></span>
		<input class="px-2 border border-zed-border bg-zed-surface-background text-zed-text" placeholder="Filter" :value="filter" @input="e => onFilter(e).catch(trouble)">
		<span>Build <span class="text-zed-text">{{built}}</span></span>
		<span>Filter <span class="text-zed-text">{{filtered}}</span></span>
		<span>Scroll <span class="text-zed-text">{{scrolled}}</span></span>
		<span>Select <span class="text-zed-text">{{selected}}</span></span>
		<span>Sort <span class="text-zed-text">{{sorted}}</span></span>
		<span>Command <span class="text-zed-text">{{command}}</span></span>
	</div>
	<div class="flex-1 min-h-0 flex"><!-- the tree and the list filling the rest of the window edge to edge, as a file manager lays them out -->
		<FujiTree class="flex-none" :style="{width: `${treeWidth}px`}" :tree="tree" />
		<div class="mySplit flex-none" @pointerdown="onSplit" @mousedown.prevent></div><!-- the bar between them; the press's default cancelled so a drag selects no text in the strip -->
		<FujiList class="flex-1 min-w-0" :list="list" @scroll.capture.passive="onScroll" /><!-- the scroll heard on its way down, since the rows scroll inside the list and a scroll event does not bubble -->
	</div>
	<MockMenu ref="menuRef" />
</div>

</template>
<style scoped>

.mySplit { position: relative; z-index: 1; width: 1px; background-color: var(--color-zed-border) } /* the bar between the tree and the list: a line one css pixel wide in Zed's border gray, as the list draws between its columns and Zed draws beside a panel. Raised, so the place to grab it below lies over the tree and the list rather than under them */
.mySplit::before { content: ""; position: absolute; top: 0; bottom: 0; left: -4px; right: -4px; cursor: col-resize } /* the place to grab it, 4 css pixels either side of the line, as the list's column dividers have */

</style>
