<script setup>//the bench fuji's list and tree are built on: the tree and the list side by side, as a file manager lays them out, each with mock data the page makes for it, a bar between them that sizes them, and how long the list takes to build, filter and scroll, on screen and in the log

import {ref, onMounted, onBeforeUnmount} from 'vue'
import {Time} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in
import {log, logTrouble} from '../log.js'
import {settings} from '../settings.js'
import {modelCompare} from '../model.js'
import {listMake, listRows, listFilter, listKey, listSelected} from '../list.js'
import {treeMake, treeRoots, treeKey} from '../tree.js'
import {sayListSize, sayListDate, multiplicationSign} from './library.js'
import {getCurrentWindow} from '@tauri-apps/api/window'
import FujiList from './FujiList.vue'
import FujiTree from './FujiTree.vue'
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
function onFocus(focused) { if (!focused) menuRef.value?.hide() }
defineExpose({onKey, onFocus})

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

const field = ref(false)//whether the list's place shows the field instead, which FujiTree.vue fills with what the tree's lines are cut from
const magnified = ref(false)//the magnifier the tree's lines are looked at by, a rule in FujiTree.vue that an attribute of its name turns on

const stripes = ref('Solid')//an experiment on the tree's lines, a block of rules in FujiTree.vue that an attribute turns on: what the lines are cut from, Solid for none, Dashes for the list's focus rectangle's rhythm, or Offbeat for specks of noise, one at a time, since they paint the same lines
const stripeKinds = ['Solid', 'Dashes', 'Offbeat']

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
function mockFolders() {//folders for the tree, chosen by hand so each kind of row is there: folders holding folders, folders holding only files, empty ones, a deep run, a long row of siblings, several roots and a root by itself with nothing in it, a name too long for the tree's width, and names in other alphabets
	let folder = (name, files, ...children) => ({name, files, children})//files says whether it holds any, which only decides its icon
	let roots = [
		folder('Pictures', true,
			folder('1999', true, folder('Summer', true), folder('Winter', true)),
			folder('2004', false, folder('Screenshots', true), folder('Wallpapers', true, folder('Anime', true), folder('Space', true))),
			folder('Fanart', false, folder('Artists', false, folder('A', false, folder('Akira', true, folder('Sketches', false, folder('Old', true, folder('Older', true))))))),
			folder('Scans', false,
				folder('Manuals', true, folder('PlayStation', true), folder('Sega Saturn', true)),
				folder('Magazines', false, folder('Next Generation', false, folder('1996', true), folder('1997', true), folder('1998', false)))),
			folder('Unsorted', false),
			folder('夏の空', true),
			folder('A folder with a name long enough to run past the right edge of the tree, so the tree scrolls sideways to show it', true)),
		folder('Downloads', true,
			folder('Album', false, folder('Disc 1', true), folder('Disc 2', true)),
			folder('New Folder', false),
			folder('New Folder (2)', false),
			folder('Москва', true)),
		folder('Archive', false,
			folder('Backup 2014', true, folder('Old', true)),
			...[2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023, 2024, 2025, 2026].map(year => folder(`Backup ${year}`, year % 3 != 0))),//every third one empty
		folder('Empty Drive', false),
	]
	let shape = (f, above) => {//each folder as the tree reads it, the way an app would hand over its own records: an identity, a name, an icon for when the tree draws one, and the folders inside
		let path = `${above}/${f.name}`
		return {path, name: f.name, icon: (f.files || f.children.length > 0) ? 'F' : 'E', children: f.children.map(child => shape(child, path))}//E for a folder with nothing inside at all
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
	<div class="mySans text-[11px] flex-none h-[28px] flex items-center gap-x-4 px-4 whitespace-nowrap overflow-hidden border-b border-zed-border [&>*]:flex-none"><!-- one line of a set height, so readouts that grow and shrink never move the tree and the list below; what runs past the right edge is cut off -->
		<span>Tree <span class="text-zed-text">{{treeMade}}</span></span>
		<span>Selected <span class="text-zed-text">{{treeSaid}}</span></span>
		<span class="flex gap-1">
			<button class="myButton" :aria-pressed="stripes != 'Solid'" @click="stripes = stripeKinds[(stripeKinds.indexOf(stripes) + 1) % stripeKinds.length]">{{stripes}}</button><!-- one button that steps through them, named for the one showing -->
			<button class="myButton" :aria-pressed="field" @click="field = !field">Field</button>
			<button class="myButton" :aria-pressed="magnified" @click="magnified = !magnified">8{{multiplicationSign}}</button>
		</span>
		<span class="flex gap-1"><button v-for="n in counts" :key="n" class="myButton" :aria-pressed="n == count" :disabled="building" @click="build(n).catch(trouble)">{{say(n)}}</button></span>
		<input class="px-2 border border-zed-border bg-zed-surface-background text-zed-text" placeholder="Filter" :value="filter" @input="e => onFilter(e).catch(trouble)">
		<span>Build <span class="text-zed-text">{{built}}</span></span>
		<span>Filter <span class="text-zed-text">{{filtered}}</span></span>
		<span>Scroll <span class="text-zed-text">{{scrolled}}</span></span>
		<span>Select <span class="text-zed-text">{{selected}}</span></span>
		<span>Sort <span class="text-zed-text">{{sorted}}</span></span>
	</div>
	<div class="myBench flex-1 min-h-0 flex p-4"><!-- the tree and the list set on a ground of their own, with room around them, so where each component ends shows -->
		<FujiTree class="myBenchCard flex-none" :style="{width: `${treeWidth}px`}" :tree="tree" :data-dashes="stripes == 'Dashes' || null" :data-offbeat="stripes == 'Offbeat' || null" :data-magnified="magnified || null" /><!-- null takes an attribute away -->
		<div class="flex-none w-4 cursor-col-resize" @pointerdown="onSplit" @mousedown.prevent></div><!-- the bar, which is the gap between them; the press's default cancelled so a drag selects no text in the strip -->
		<FujiList v-show="!field" class="myBenchCard flex-1 min-w-0" :list="list" @scroll.capture.passive="onScroll" /><!-- the scroll heard on its way down, since the rows scroll inside the list and a scroll event does not bubble -->
		<div v-show="field" class="myBenchCard flex-1 min-w-0 myTreeField" :data-dashes="stripes == 'Dashes' || null" :data-offbeat="stripes == 'Offbeat' || null"></div><!-- kept mounted beside the list rather than swapped for it, so the list keeps its place and its selection -->
	</div>
	<MockMenu ref="menuRef" />
</div>

</template>
<style scoped>

.myBench { /* tight diagonal stripes of two grays behind the components, so a component that paints no ground of its own, or paints past its edge, shows it */
	background-image: repeating-linear-gradient(-45deg, var(--color-zed-border-variant) 0 3px, var(--color-zed-surface-background) 3px 6px);
}
.myBenchCard { box-shadow: 0 2px 8px var(--color-fuji-shade) } /* no border, so the shadow alone says where each component ends */

</style>
