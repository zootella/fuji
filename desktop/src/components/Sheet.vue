<script setup>//the contact sheet: a page of buckets of thumbnails, a window onto the pager's list, with Previous and Next to move it along the volume

import {ref, computed, reactive, watch, onMounted} from 'vue'
import parse from 'path-browserify'
import {documentDir, downloadDir} from '@tauri-apps/api/path'
import {modelFolder, modelOpen, modelOpenFolder, modelStand} from '../model.js'
import {pagerBuckets, pagerRead, pagerPreviousState, pagerNextState, pagerStart, pagerNext, pagerPrevious, pagerRefresh} from '../pager.js'
import {forwardize} from './library.js'
import {logTrouble} from '../log.js'
import {settings} from '../settings.js'
import Bucket from './Bucket.vue'
import BucketMemory from './BucketMemory.vue'

//a page of buckets down one scroll, sheet.buckets of them with bucket.images pictures each, cut from wherever on the volume the pager is: pager.js holds the list and the page, the buttons move the page along it, and this file renders what it holds. The sheet and the table no longer share one list. The table flips within the folder the model lists, and the sheet's buckets come from many folders, so a double-click lists that thumbnail's folder for the table; and the sheet remembers which folder it last handed the model, to tell a drop on the table, which starts the pager again there, from its own double-click, which leaves the pager where it is. The settings panel takes this view's place in the same window, and s trades the two

const sheetRoot = ref(null)//the scroll, put back to the top when a new page lands
const sheetFit = ref('')//the fit, part of every bucket's key, so one chosen in the settings panel rebuilds them all when the sheet comes back; read in start, because this view is made before the shell has read fuji.toml, and a value taken now would be the factory's
const sheetBeam = ref('')//the beam's name, the same way: a flow reads its length once, as it is made, so a new beam needs new buckets
let sheetOpened = ''//the folder this sheet last handed the model, by a drop, a folder button or a double-click; a model folder that differs when the sheet comes back on screen was opened on the table, and the pager starts again there

function bucketKey(bucket) { return bucket.files.map(file => file.path).join() + sheetFit.value + sheetBeam.value }//what tells one bucket from another below: its contents, the fit and the beam, so a change to any of the three makes a new bucket rather than reusing one that painted other pictures
const sheetBytes = reactive(new Map())//each bucket's canvases, by its key, as each reports them; a reactive map, so the sum below follows
watch([pagerBuckets, sheetFit, sheetBeam], () => {//a new page, or a new fit or beam, and the scroll goes back to the top, where the page begins
	let keep = new Set(pagerBuckets.value.map(bucket => bucketKey(bucket)))
	for (let key of [...sheetBytes.keys()]) if (!keep.has(key)) sheetBytes.delete(key)//pruned to the buckets on the page rather than cleared, because a page re-cut to the same buckets keeps the same components, which never report again; a bucket whose key changed is a new one and reports as it fills
	sheetRoot.value?.scrollTo(0, 0)
})
const sheetBytesSum = computed(() => pagerBuckets.value.reduce((sum, bucket) => sum + (sheetBytes.get(bucketKey(bucket)) || 0), 0))//the inner number: every bucket's thumbnails together
const sheetThumbnails = computed(() => pagerBuckets.value.reduce((count, bucket) => count + bucket.files.length, 0))

function start() {//the shell calls this each time this view comes on screen: read the choices the settings panel may have changed, then start from a folder opened on the table, or re-cut the page where it is under the counts as they are now
	sheetFit.value = settings.thumbnail.fit; sheetBeam.value = settings.thumbnail.beam
	if (modelFolder.value && modelFolder.value != sheetOpened) { sheetOpened = modelFolder.value; pagerStart(modelFolder.value).catch(error => logTrouble(`sheet: starting from ${modelFolder.value}`, error)) }
	else pagerRefresh().catch(error => logTrouble('sheet: refreshing the page', error))
}
const emit = defineEmits(['table', 'settings'])//a double-clicked thumbnail, for the shell to show the table fullscreen, and s, for the settings panel in this same window; which view is showing is the shell's, so this only asks
function onKey(e) {
	if (e.key == 's' && !e.ctrlKey && !e.metaKey) emit('settings')//a temporary way in, until fuji has a better place for one
}
async function onDrop(path) {//a picture dropped or opened here: the model lists its folder for the table, and the pager starts at that folder
	await modelOpen(path)
	sheetOpened = modelFolder.value
	await pagerStart(modelFolder.value)
}
async function onDoubleClick(e) {//a thumbnail double-clicked: list its folder for the table unless the table is in it already, stand the model on the picture, and ask for the table, which shows whatever the model is standing on when it comes back
	let path = e.target.closest('[data-path]')?.dataset.path//the flow names every tile's path on the cell that holds its picture and caption, so one listener here serves them all
	if (!path) return//between tiles, or on the empty sheet
	try {
		let folder = parse.dirname(path)
		if (folder == modelFolder.value) modelStand(path)
		else await modelOpen(path)//the sheet's own listing for the table, which the pager takes no notice of
		sheetOpened = folder
		emit('table')
	} catch (error) { logTrouble(`sheet: opening ${path} on the table`, error) }//a folder that cannot be listed leaves the sheet as it is
}
function previous() { pagerPrevious().catch(error => logTrouble('sheet: the previous page', error)) }//the top gates for the two buttons
function next() { pagerNext().catch(error => logTrouble('sheet: the next page', error)) }
function sayPager(state, name, where) { return state == 'none' ? where : name }//the words on a button: its name, disabled until its page is known, or that the list ends here; no progress, which is the log's and the memory report's to say
const sheetPrevious = computed(() => sayPager(pagerPreviousState.value, '‹ Previous', 'the top of the disk'))//the words for each button, read by the bar at each end of the page
const sheetNext = computed(() => sayPager(pagerNextState.value, 'Next ›', 'the end of the disk'))
const sheetFolders = ref([])//{name, path} for each folder the empty sheet offers to open, found once when it is made; one the platform cannot name is left out
onMounted(async () => {
	let found = []
	for (let [name, ask] of [['Documents', documentDir], ['Downloads', downloadDir]]) {
		try { found.push({name, path: forwardize(await ask())}) }
		catch (error) { logTrouble(`sheet: finding the ${name} folder`, error) }//tauri asks the platform, which can have no answer for a folder the user has removed or never had
	}
	sheetFolders.value = found
})
async function folderOpen(path) {//a folder clicked on the empty sheet, opened as a drop of one of its pictures would open it, standing on its first
	try { await modelOpenFolder(path); sheetOpened = modelFolder.value; await pagerStart(modelFolder.value) }
	catch (error) { logTrouble(`sheet: opening ${path}`, error) }//a folder that cannot be read leaves the sheet empty, as it was
}
function onResize() {}//and nothing to remeasure: the wrapping is the engine's job, and this view measures nothing, which is what lets it stay mounted

defineExpose({start, onKey, onResize, onDrop})//the same calls every view answers

</script>
<template>

<!-- display none destroys the layout box and the scroll position with it, so leaving and returning starts at the top; that is the behaviour wanted for now -->
<div ref="sheetRoot" class="mySheet w-full h-full overflow-y-auto select-none" @dblclick="onDoubleClick">
	<div v-if="!pagerBuckets.length" class="myEmpty myMono w-full h-full flex flex-col items-center justify-center gap-2">
		<p>contact sheet - drop a picture here to open its folder</p>
		<p v-if="sheetFolders.length">or open <template v-for="(folder, i) in sheetFolders" :key="folder.path"><template v-if="i > 0"> or </template><button class="underline cursor-pointer" @click="folderOpen(folder.path)">{{folder.name}}</button></template></p><!-- buttons rather than links, since each one does something here rather than going somewhere -->
	</div>
	<template v-else>
		<div class="myPagerBar"><!-- the same pair at each end of the page, so a neighbor is a click away from wherever the scroll is; each button is disabled while the pager is still finding its page, and for good at the top or the end -->
			<button class="myPager myMono" :disabled="pagerPreviousState != 'ready'" @click="previous">{{sheetPrevious}}</button>
			<button class="myPager myMono" :disabled="pagerNextState != 'ready'" @click="next">{{sheetNext}}</button>
		</div>
		<!-- keyed on contents, the fit and the beam, so a bucket whose images, fit or beam changed is rebuilt rather than reused: a reused bucket keeps canvases painted from images it no longer holds and never paints the new ones, and its flow read the fit and the beam once, when it was made -->
		<Bucket v-for="bucket in pagerBuckets" :key="bucketKey(bucket)" :folder="bucket.folder" :files="bucket.files" :first="bucket.first" :total="bucket.total" @bytes="bytes => sheetBytes.set(bucketKey(bucket), bytes)" /><!-- each bucket's bytes back, for the report beneath -->
		<div class="myPagerBar"><!-- and again between the last bucket and the memory report -->
			<button class="myPager myMono" :disabled="pagerPreviousState != 'ready'" @click="previous">{{sheetPrevious}}</button>
			<button class="myPager myMono" :disabled="pagerNextState != 'ready'" @click="next">{{sheetNext}}</button>
		</div>
		<BucketMemory :buckets="pagerBuckets.length" :thumbnails="sheetThumbnails" :bytes="sheetBytesSum" :beam="sheetBeam" :fit="sheetFit" :read="pagerRead" /><!-- beneath the last bucket and the button, on the same scroll -->
	</template>
</div>

</template>
<style scoped>

.mySheet {
	background-color: var(--color-paper);
}
.myEmpty {
	color: var(--color-faint);
}
.myPagerBar { /* the two buttons side by side across the width of the buckets, with the flow's own margin around them */
	display: flex; gap: 8px; margin: 8px;
}
.myPager { /* each button half the bar */
	flex: 1; min-width: 0;
	padding: 12px; text-align: center;
	color: var(--color-ink);
	background-color: var(--color-surface);
	border: 1px solid var(--color-line);
	cursor: pointer;
}
.myPager:disabled { /* still looking for its page, or the end of the list either way: quieter, and no hand */
	color: var(--color-faint);
	cursor: default;
}

</style>
