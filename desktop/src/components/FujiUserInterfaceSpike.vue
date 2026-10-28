<script setup>//the bench fuji's list is built on: the list, mock rows the page makes for it, and how long the list takes to build, filter and scroll, on screen and in the log

import {ref, shallowRef, onMounted, onBeforeUnmount} from 'vue'
import {log, logTrouble} from '../log.js'
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
		built.value = `${say(n)} rows in ${ms} ms`
		log(`interface spike: built ${say(n)} rows in ${ms} ms`)
	} finally { building.value = false }
}
onMounted(() => build(count.value).catch(trouble))

async function onFilter(e) {
	let start = performance.now()
	filter.value = e.target.value
	await afterPaint()
	let ms = Math.round(performance.now() - start)
	let said = `"${filter.value}" shows ${say(listRef.value.shown.length)} of ${say(rows.value.length)} in ${ms} ms`
	filtered.value = said
	log(`interface spike: ${said}`)
}

async function onSelect(size) {//the click has already changed the selection, and the rows have not yet drawn it
	let start = performance.now()
	await afterPaint()
	let ms = Math.round(performance.now() - start)
	selected.value = `${say(size)} rows in ${ms} ms`
	log(`interface spike: selected ${say(size)} of ${say(listRef.value.shown.length)} rows in ${ms} ms`)
}

let scrolling = false, scrollLast = 0, scrollGaps = [], scrollQuiet = null
const scrollRest = 300//milliseconds with no scroll event that end a scroll
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
	scrolled.value = `${scrollGaps.length} frames, longest ${longest} ms, ${late} late`
	log(`interface spike: scrolled ${say(listRef.value.shown.length)} rows over ${scrollGaps.length} frames of ${interval.toFixed(1)} ms, longest ${longest} ms, ${late} late`)
}
onBeforeUnmount(() => { scrolling = false; clearTimeout(scrollQuiet) })

let mockMade = []//made at the first build each time the panel opens
function mockRows(n) {
	if (!mockMade.length) mockMade = mockNames(counts[counts.length - 1])
	return mockMade.slice(0, n).sort()//the same names at every count, in a folder's order, which the sheet sorts with the same call
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

<div class="bg-paper text-ink w-full h-full flex flex-col">
	<div class="myMono flex flex-wrap items-center gap-x-6 gap-y-2 px-4 py-2 border-b border-line">
		<span class="flex gap-1"><button v-for="n in counts" :key="n" class="myButton" :aria-pressed="n == count" :disabled="building" @click="build(n).catch(trouble)">{{say(n)}}</button></span>
		<input class="px-2 border border-line bg-surface text-strong" placeholder="Filter" :value="filter" @input="e => onFilter(e).catch(trouble)">
		<span>Build <span class="text-strong">{{built}}</span></span>
		<span>Filter <span class="text-strong">{{filtered}}</span></span>
		<span>Scroll <span class="text-strong">{{scrolled}}</span></span>
		<span>Select <span class="text-strong">{{selected}}</span></span>
	</div>
	<FujiList ref="listRef" class="flex-1 min-h-0" :rows="rows" :filter="filter" @scroll.passive="onScroll" @select="size => onSelect(size).catch(trouble)" />
</div>

</template>
