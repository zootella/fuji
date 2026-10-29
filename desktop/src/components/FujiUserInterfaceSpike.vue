<script setup>//the bench fuji's list is built on: the list, mock rows the page makes for it, and how long the list takes to build, filter and scroll, on screen and in the log

import {ref, shallowRef, onMounted, onBeforeUnmount} from 'vue'
import {Size, Time} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in
import {log, logTrouble} from '../log.js'
import {modelSortRun} from '../model.js'
import {sayDayTime} from './library.js'
import FujiList from './FujiList.vue'

const counts   = [50, 200, 1000, 2000, 5000, 10000]
const count    = ref(50)
const rows     = shallowRef([])
const filter   = ref('')
const building = ref(false)
const listRef  = ref(null)
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
		rows.value = []; await afterPaint()//and the old list gone, so the number is the new list's alone
		let start = performance.now()
		rows.value = names
		await afterPaint()
		let ms = Math.round(performance.now() - start)
		built.value = `${say(n)} rows in ${ms}ms`
		log(`interface spike: built ${say(n)} rows in ${ms}ms`)
	} finally { building.value = false }
}
onMounted(() => build(count.value).catch(trouble))

async function onFilter(e) {
	let start = performance.now()
	filter.value = e.target.value
	await afterPaint()
	let ms = Math.round(performance.now() - start)
	let said = `"${filter.value}" shows ${say(listRef.value.shown().length)} of ${say(rows.value.length)} in ${ms}ms`
	filtered.value = said
	log(`interface spike: ${said}`)
}

let pressed = 0, waited = 0//when the last press reached the bench, which hears it before the list does, and how long it sat in line before that, from the moment the event was made
function press(e) { pressed = performance.now(); waited = pressed - e.timeStamp }
function onKey(e) { press(e); listRef.value?.key(e) }//every key the shell hands this view goes on to the list, timed like a press
defineExpose({onKey})
async function onSelect(p) {//from that press to the selection painted, so the list's own marking is inside the number
	let start = pressed, queued = waited//read before the wait, since a press queued behind this one can reach the bench during it
	await afterPaint()
	let ms = Math.round(performance.now() - start)
	selected.value = `${say(p.selected)} rows in ${ms}ms`
	log(`interface spike: ${p.kind} press selected ${say(p.selected)} of ${say(listRef.value.shown().length)} rows, ${say(p.changed)} changed, waited ${Math.round(queued)}ms in line, list ${p.listMs.toFixed(1)}ms, painted at ${ms}ms`)
}

async function onSort(p) {//from the start of the list's own work to the sorted list painted
	let start = performance.now() - p.listMs
	await afterPaint()
	let ms = Math.round(performance.now() - start)
	let said = `${say(rows.value.length)} rows by ${p.key}${p.descending ? ' descending' : ''} in ${ms}ms`
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
	log(`interface spike: scrolled ${say(listRef.value.shown().length)} rows over ${scrollGaps.length} frames of ${interval.toFixed(1)}ms, longest ${longest}ms, ${late} late`)
}
onBeforeUnmount(() => { scrolling = false; clearTimeout(scrollQuiet) })

function sayKilobytes(bytes) { return `${Math.ceil(bytes / Size.kb).toLocaleString('en-US')} KB` }//as Explorer says a size, in whole kilobytes rounded up, so a file of one byte is 1 KB and every size is in one unit, its order of magnitude read from the width of its digits
const columns = [
	{key: 'name', title: 'Name', sort: modelSortRun, width: 320},//names in fuji's one sort, the order every list and view in fuji shows
	{key: 'size', title: 'Size', say: sayKilobytes, align: 'right', width: 128},
	{key: 'modified', title: 'Date', say: sayDayTime, width: 152},
]
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
	<div class="myMono flex flex-wrap items-center gap-x-6 gap-y-2 px-4 py-2 border-b border-zed-border">
		<span class="flex gap-1"><button v-for="n in counts" :key="n" class="myButton" :aria-pressed="n == count" :disabled="building" @click="build(n).catch(trouble)">{{say(n)}}</button></span>
		<input class="px-2 border border-zed-border bg-zed-surface-background text-zed-text" placeholder="Filter" :value="filter" @input="e => onFilter(e).catch(trouble)">
		<span>Build <span class="text-zed-text">{{built}}</span></span>
		<span>Filter <span class="text-zed-text">{{filtered}}</span></span>
		<span>Scroll <span class="text-zed-text">{{scrolled}}</span></span>
		<span>Select <span class="text-zed-text">{{selected}}</span></span>
		<span>Sort <span class="text-zed-text">{{sorted}}</span></span>
	</div>
	<FujiList ref="listRef" class="flex-1 min-h-0" :columns="columns" :rows="rows" :filter="filter" @scroll.capture.passive="onScroll" @select="p => onSelect(p).catch(trouble)" @sort="p => onSort(p).catch(trouble)" /><!-- the scroll heard on its way down, since the rows scroll inside the list and a scroll event does not bubble -->
</div>

</template>
