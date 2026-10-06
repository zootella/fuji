<script setup>//the memory report beneath the buckets: what the computer has, what is in use, what fuji's processes take, and what the buckets' thumbnails take, four shells from the outside in

import {ref, computed, onMounted, onBeforeUnmount} from 'vue'
import {memoryReport} from '../memory.js'
import {logTrouble} from '../log.js'
import {saySize4, platform} from './library.js'
import {brandName} from '../brand.js'

//the numbers a user compares when choosing how many buckets and how many thumbnails to run: what the machine was bought with, what everything on it is using, what fuji's processes take of that, and what the thumbnails take of fuji, each as a share of the shell outside it. The outer three come from memory.rs, read every memoryEvery milliseconds and only while this box is on screen, which is what the IntersectionObserver answers: a sheet hidden behind the table, or scrolled up past this box, asks nothing. The innermost is the sheet's own count, exact, since every thumbnail is a canvas fuji sized, and the sheet sums what every bucket's caption shows

const memoryEvery = 2000//milliseconds between readings while the box is on screen; a reading is a few system calls and one ipc round trip, so this is about how fast the numbers move rather than what they cost

const props = defineProps({
	buckets: {type: Number, required: true},//how many buckets the sheet holds
	thumbnails: {type: Number, required: true},//and how many thumbnails across them
	bytes: {type: Number, required: true},//what their canvases cost, summed from every bucket's own total
	beam: {type: String, required: true},//the beam's name and the fit's, so the numbers sit beside the choices that made them
	fit: {type: String, required: true},
	listed: {type: Object, required: true},//{count, milliseconds}: how many folders the walk listed for this page and its look-ahead, and how long those listings took together
})

const memoryRoot = ref(null)//this box, which the observer watches
const memory = ref(null)//the last report, or null before the first
let memoryTimer = 0
let memoryObserver = null

onMounted(() => {
	memoryObserver = new IntersectionObserver(entries => { if (entries.some(entry => entry.isIntersecting)) memoryStart(); else memoryStop() })//fires on every change, so scrolling this into view starts the readings and away from it stops them, and the sheet going display none counts as away
	memoryObserver.observe(memoryRoot.value)
})
onBeforeUnmount(() => { memoryStop(); memoryObserver?.disconnect() })

function memoryStart() { if (memoryTimer) return; memoryRead(); memoryTimer = setInterval(memoryRead, memoryEvery) }//a reading now, then one every interval
function memoryStop() { clearInterval(memoryTimer); memoryTimer = 0 }
function memoryRead() { memoryReport().then(report => { memory.value = report }).catch(error => logTrouble('BucketMemory: reading memory', error)) }//the top gate for a reading: rust answers trouble as a rejection, and a reading that fails leaves the last one showing

const memoryOuter = computed(() => memory.value ? memory.value.processes.reduce((sum, p) => sum + p.bytes, 0) : 0)//the outer number: fuji's processes together
function sayShare(part, whole, of) { return whole > 0 ? `${Math.round(part / whole * 100)}% of the ${saySize4(whole)} ${of}` : '' }//one shell as a share of the one outside it, naming the number it is divided by and the row that number is, like 1% of the 9625 MB in use
const memoryNames = {//what each process is, for its row, by rust's kind: the three a tauri program is made of, and the two more WebView2 adds
	host: 'Rust and the window',
	'web content': 'Web renderer', renderer: 'Web renderer',//WebKit's name and WebView2's for the same thing
	gpu: 'GPU process',
	browser: 'Browser process', utility: 'Utility process', 'sandbox helper': 'Sandbox helper', plugin: 'Plugin process', 'plugin broker': 'Plugin broker',
}
function sayKind(kind) { return memoryNames[kind] || kind }
function sayMilliseconds(ms) { return ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(1)} s` }//milliseconds under a second, and seconds to a tenth from there, since the number is read against a wait

//what each row is and how it is counted, shown to the left of the table for the row the pointer is over; keyed by the fixed rows' names here, the platform's own names for its breakdown rows, and rust's kinds for the processes
const memoryIntro = `What this computer has, what is in use, what ${brandName} takes of that, and what the thumbnails take of ${brandName}, each as a share of the one outside it. Every row under ${brandName} is one process: a Tauri program is a Rust process holding the window, a web renderer the engine runs the page in, and the engine's GPU process, with a browser process and helpers on Windows. Point at a row to read what it is and how it is counted.`
const memoryHints = {
	installed: `The memory this computer was built with, as the system reports it: on a Mac the amount installed, and on Windows the amount the firmware lists, which is the number on the box. Everything below is a share of this.`,
	used: `What everything running on this computer is using right now, counted the way Activity Monitor's Memory Used and Task Manager's In use count it. On a Mac that is app memory plus wired plus compressed, and on Windows the total less what is available.`,
	available: `What the system says an application could take right now without pushing anything out. On a Mac the total less what is in use; on Windows the system's own available figure.`,
	'App Memory': `Memory applications hold and the system cannot take back without their cooperation: every program's own data, ${brandName}'s included. The largest part of what is in use on most Macs.`,
	'Wired Memory': `Memory the kernel keeps for itself and never pages out: drivers, the file system's tables, and its own structures.`,
	'Compressed': `Memory the system has squeezed to make room rather than writing it to disk. It counts as in use, since it belongs to a program that may ask for it back.`,
	'Cached Files': `Files the system keeps in memory because they were read recently. Not counted as in use: the system hands this memory to any application that asks, which is why a Mac always looks nearly full.`,
	'Swap Used': `Memory written out to disk to make room. Zero on a machine with room to spare, and a sign of pressure when it grows.`,
	'Committed': `Memory programs have been promised, whether it sits in memory or in the page file. Task Manager shows it as Committed, and it can exceed what is in use.`,
	'Commit limit': `The most that can be promised at once: physical memory plus the page file. Task Manager shows the two as Committed, this one second.`,
	'Cached': `Files the system keeps in memory because they were read recently, handed to any application that asks. Task Manager's Cached.`,
	fuji: `Every process that is ${brandName}, added up from the rows below: the Rust process with the window, the web renderer, the GPU process, and on Windows the browser process and its helpers. The system charges each one separately, and this is their sum, the way Activity Monitor or Task Manager would read if you added ${brandName}'s rows yourself.`,
	host: `The Rust process: the one the system started when ${brandName} launched. It holds everything Rust does, the window with its menu bar and dock on a Mac, the settings and the log, and each thumbnail's bytes for the moment they cross from the operating system to the page. The webview is a view inside this window, but the page inside that view is not here; it is rendered in the web renderer below. It stays small however long ${brandName} works. Activity Monitor lists it as ${brandName} and Task Manager as the program itself; counted by the Memory column of each.`,
	'web content': `The web renderer: the process WebKit runs ${brandName}'s page in. The JavaScript, the components, the DOM, every decoded picture and every canvas's drawing live here, so this is where the contact sheet's memory is charged, apart from canvases WebKit draws in the GPU process on recent macOS. WebKit starts one per window; Activity Monitor lists it as ${brandName} Web Content and counts it in its Memory column, which is what this reads.`,
	renderer: `The web renderer: the process WebView2 runs ${brandName}'s page in. The JavaScript, the components, the DOM, every decoded picture and every canvas's drawing live here, so this is where the contact sheet's memory is charged, apart from the textures of accelerated canvases, which the GPU process holds. One per webview; Task Manager lists it among Microsoft Edge WebView2's processes, and this reads its private working set, Task Manager's Memory column.`,
	gpu: platform() == 'windows'
		? `WebView2's GPU process, where the engine rasterizes and composites, shared by every webview in the same browser instance. Every thumbnail at Medium or larger is a texture here, and on a machine with its own graphics card those textures sit in the card's memory, which no process counts; the Thumbnails row counts them exactly. Read as its private working set, Task Manager's Memory column.`
		: `WebKit's GPU process, where the engine rasterizes, composites and decodes media: one per application, shared by every ${brandName} window. On recent macOS canvases are drawn here, so the thumbnails' pixels may be charged to this row rather than to the web renderer. Activity Monitor lists it as ${brandName} Graphics and Media; read by its footprint, Activity Monitor's Memory column.`,
	browser: `WebView2's browser process, the one that starts and manages the others: the renderer, the GPU process and the utilities. One per data folder, so every ${brandName} window on this machine shares it, since Windows runs each window in a Rust process of its own. Read as its private working set.`,
	utility: `A WebView2 utility process, for networking, storage, audio and the like, started as the page needs it and shared like the browser process. Read as its private working set.`,
	'sandbox helper': `A WebView2 helper process, read as its private working set.`,
	plugin: `A WebView2 plugin process, read as its private working set.`,
	'plugin broker': `A WebView2 plugin process, read as its private working set.`,
	thumbnails: `What the thumbnails on this contact sheet cost, counted by ${brandName} itself: every thumbnail is a canvas ${brandName} sized, its width times its height in screen pixels times four bytes, summed over every bucket. Exact, and the only number here ${brandName} counts rather than asks the system for. These bytes live in the web renderer or the GPU process above, never in the Rust process, which only passes each thumbnail through.`,
	counts: `How many buckets the sheet holds and how many thumbnails are in them, with the thumbnail size and the fit chosen in settings. Change those in settings, press S to come back, and watch the rows above follow.`,
	listed: `What the walk asked of the disk for this page: how many folders it listed to cut these buckets and to find the pages before and after them, and how long those listings took added together, not how long the page took to show. Every folder the walk passes through costs one listing, so a stretch of the disk with few pictures reads as many folders here, and this is the number that says whether a cache of listings would be worth having.`,
}
const memoryHint = ref(memoryIntro)//what the explanation on the left says: the row under the pointer, or the introduction
function memoryPoint(event) { let row = event.target.closest('[data-hint]'); if (row) memoryHint.value = memoryHints[row.dataset.hint] || memoryIntro }//one listener on the table, finding the row from whichever cell the pointer is over; the gaps between cells belong to no row and leave the last one showing
function memoryLeave() { memoryHint.value = memoryIntro }

</script>
<template>

<!-- the same padding as the flow; no border and no background, like the bucket above it. The table is an aside against the right edge, like the captions in each bucket's upper right, and the explanation takes the room to its left. One grid for the whole table rather than one per row, so every number sits in one straight column whatever the names beside it do; each row is a display contents wrapper, in the grid for nothing but its data-hint -->
<div ref="memoryRoot" class="myMemory myMono flex gap-8" @mouseleave="memoryLeave">
	<div class="myMemoryHint">{{memoryHint}}</div>
	<div v-if="memory" class="myMemoryGrid" @mouseover="memoryPoint">
		<div class="contents" data-hint="installed"><span>System</span><span>This computer has</span><span class="myMemoryBytes">{{saySize4(memory.installed)}}</span><span></span></div>
		<div class="contents" data-hint="used"><span></span><span>In use</span><span class="myMemoryBytes">{{saySize4(memory.used)}}</span><span>{{sayShare(memory.used, memory.installed, 'this computer has')}}</span></div>
		<div class="contents" data-hint="available"><span></span><span>Available</span><span class="myMemoryBytes">{{saySize4(memory.available)}}</span><span></span></div>
		<div v-for="row in memory.details" :key="row.name" class="contents" :data-hint="row.name"><span></span><span>{{row.name}}</span><span class="myMemoryBytes">{{saySize4(row.bytes)}}</span><span></span></div>
		<div class="contents" data-hint="fuji"><span class="myMemoryBreak">{{brandName}}</span><span class="myMemoryBreak">All of it</span><span class="myMemoryBytes myMemoryBreak">{{saySize4(memoryOuter)}}</span><span class="myMemoryBreak">{{sayShare(memoryOuter, memory.used, 'in use')}}</span></div>
		<div v-for="p in memory.processes" :key="p.pid" class="contents" :data-hint="p.kind"><span></span><span>{{sayKind(p.kind)}}</span><span class="myMemoryBytes">{{saySize4(p.bytes)}}</span><span>pid {{p.pid}}</span></div>
		<div class="contents" data-hint="thumbnails"><span class="myMemoryBreak">Thumbnails</span><span class="myMemoryBreak">All of them</span><span class="myMemoryBytes myMemoryBreak">{{saySize4(props.bytes)}}</span><span class="myMemoryBreak">{{sayShare(props.bytes, memoryOuter, `${brandName} takes`)}}</span></div>
		<div class="contents" data-hint="counts"><span></span><span class="col-span-3">{{props.buckets}} buckets, {{props.thumbnails}} thumbnails, {{props.beam}}, {{props.fit.replace(/Fit$/, '')}}</span></div>
		<div class="contents" data-hint="listed"><span class="myMemoryBreak">Walk</span><span class="myMemoryBreak">{{props.listed.count}} folders listed</span><span class="myMemoryBytes myMemoryBreak">{{sayMilliseconds(props.listed.milliseconds)}}</span><span class="myMemoryBreak">for this page and its look-ahead</span></div>
	</div>
	<div v-else>Reading memory</div>
</div>

</template>
<style scoped>

.myMemory {
	padding: 8px; /* the flow's, so the explanation starts under the thumbnails' left edges and the table ends where the bucket captions do */
	color: var(--color-quiet); /* the captions' color: this says something about the sheet rather than being it */
	cursor: default;
}
.myMemoryHint {
	flex: 1; max-width: 60ch; /* the explanation, in the room left of the table: as wide as a paragraph reads well, and no wider */
}
.myMemoryGrid {
	display: grid; grid-template-columns: max-content max-content max-content max-content; column-gap: 16px; /* a group, a name, a number and a note, each column as wide as its widest cell */
	margin-left: auto; text-align: right; /* pushed to the right edge as an aside, like a bucket's caption, every column reading from its right */
	align-self: start;
}
.myMemoryBreak {
	margin-top: 8px; /* a little paper before a group's first row; on each of its cells, since a grid row is only its cells */
}
.myMemoryBytes {
	font-variant-numeric: tabular-nums; /* every digit the same width, so a column of sizes reads down */
}

</style>
