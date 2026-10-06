<script setup>//the contact sheet: one folder, as a stack of buckets of thumbnails

import {ref, computed, reactive, watch, onMounted} from 'vue'
import {documentDir, downloadDir} from '@tauri-apps/api/path'
import {modelList, modelOpen, modelOpenFolder, modelStand} from '../model.js'
import {forwardize} from './library.js'
import {logTrouble} from '../log.js'
import {settings} from '../settings.js'
import Bucket from './Bucket.vue'
import BucketMemory from './BucketMemory.vue'

//a stack of buckets down one scroll, sheet.buckets of them with bucket.images pictures each, from the start of the folder, and no more: the rest of a large folder is simply not shown, so the two settings together are a ceiling on how many thumbnails the sheet ever holds. This file cuts the list into buckets, and sizing, arranging, loading and holding are the flow's, one level down. The settings panel takes this view's place in the same window, and s trades the two

const sheetStarted = ref(false)//the shell has shown this view at least once, which is what the guard below turns on
const sheetImages = ref(0)//how many pictures a bucket holds
const sheetBucketCount = ref(0)//and how many buckets the sheet holds; both read from settings each time the sheet comes on screen, because the settings panel is where they change and the sheet is never showing while it does, so reading on arrival is all the watching they need
const sheetFit = ref('')//the fit, and part of every bucket's key, so one chosen in the settings panel rebuilds them all when the sheet comes back; read in start like the two above, because this view is made before the shell has read fuji.toml, and a value taken now would be the factory's
const sheetBeam = ref('')//the beam's name, the same way: a flow reads its length once, as it is made, so a new beam needs new buckets

//that guard covers only a sheet that has never been shown. After that, a folder opened on the table rebuilds these buckets behind it, and they fill there, hidden, as they would on screen
const sheetBuckets = computed(() => {
	if (!sheetStarted.value) return []//v-show hides without unmounting, so without this an unlooked-at sheet would still render and quietly load a whole folder on the table's drop
	let buckets = []
	for (let i = 0; i < sheetBucketCount.value; i++) {
		let bucket = modelList.value.slice(i * sheetImages.value, (i + 1) * sheetImages.value)
		if (!bucket.length) break//the folder ran out before the sheet did
		buckets.push(bucket)
	}
	return buckets
})

function bucketKey(bucket) { return bucket.join() + sheetFit.value + sheetBeam.value }//what tells one bucket from another below: its contents, the fit and the beam, so a change to any of the three makes a new bucket rather than reusing one
const sheetBytes = reactive(new Map())//each bucket's canvases, by its key, as each reports them; a reactive map, so the sum below follows
watch([sheetBuckets, sheetFit, sheetBeam], () => sheetBytes.clear())//buckets being remade, by new contents or a new fit or beam, start their counts over under new keys, and a bucket of imgs alone never reports, so nothing stale may be left behind
const sheetBytesSum = computed(() => sheetBuckets.value.reduce((sum, bucket) => sum + (sheetBytes.get(bucketKey(bucket)) || 0), 0))//the inner number: every bucket's thumbnails together
const sheetThumbnails = computed(() => sheetBuckets.value.reduce((count, bucket) => count + bucket.length, 0))

function start() { sheetImages.value = settings.bucket.images; sheetBucketCount.value = settings.sheet.buckets; sheetFit.value = settings.thumbnail.fit; sheetBeam.value = settings.thumbnail.beam; sheetStarted.value = true }//the shell calls this each time this view comes on screen; a value that changed rebuilds the buckets, and one that did not changes nothing
const emit = defineEmits(['table', 'settings'])//a double-clicked thumbnail, for the shell to show the table fullscreen, and s, for the settings panel in this same window; which view is showing is the shell's, so this only asks
function onKey(e) {
	if (e.key == 's' && !e.ctrlKey && !e.metaKey) emit('settings')//a temporary way in, until fuji has a better place for one
}
async function onDrop(path) { await modelOpen(path) }//a picture dropped or opened here: the model lists its folder and the buckets build from that, and the table shows the picture itself when the user goes to it
function onDoubleClick(e) {//a thumbnail double-clicked: stand the model on its picture and ask for the table, which shows whatever the model is standing on when it comes back
	let path = e.target.closest('[data-path]')?.dataset.path//the flow names every tile's path on the cell that holds its picture and caption, so one listener here serves them all
	if (!path) return//between tiles, or on the empty sheet
	modelStand(path)
	emit('table')
}
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
	try { await modelOpenFolder(path) }
	catch (error) { logTrouble(`sheet: opening ${path}`, error) }//a folder that cannot be read leaves the sheet empty, as it was
}
function onResize() {}//and nothing to remeasure: the wrapping is the engine's job, and this view measures nothing, which is what lets it stay mounted

defineExpose({start, onKey, onResize, onDrop})//the same calls every view answers

</script>
<template>

<!-- display none destroys the layout box and the scroll position with it, so leaving and returning starts at the top; that is the behaviour wanted for now -->
<div class="mySheet w-full h-full overflow-y-auto select-none" @dblclick="onDoubleClick">
	<div v-if="!sheetBuckets.length" class="myEmpty myMono w-full h-full flex flex-col items-center justify-center gap-2">
		<p>contact sheet - drop a picture here to open its folder</p>
		<p v-if="sheetFolders.length">or open <template v-for="(folder, i) in sheetFolders" :key="folder.path"><template v-if="i > 0"> or </template><button class="underline cursor-pointer" @click="folderOpen(folder.path)">{{folder.name}}</button></template></p><!-- buttons rather than links, since each one does something here rather than going somewhere -->
	</div>
	<!-- keyed on contents, the fit and the beam, so a bucket whose images, fit or beam changed is rebuilt rather than reused: a reused bucket keeps canvases painted from images it no longer holds and never paints the new ones, and its flow read the fit and the beam once, when it was made -->
	<Bucket v-for="(bucket, i) in sheetBuckets" :key="bucketKey(bucket)" :paths="bucket" :first="i * sheetImages + 1" :total="modelList.length" @bytes="bytes => sheetBytes.set(bucketKey(bucket), bytes)" /><!-- first and total for the bucket's caption, counted from 1 the way a person counts, since the slicing that knows them is here; and each bucket's bytes back, for the report beneath -->
	<BucketMemory v-if="sheetBuckets.length" :buckets="sheetBuckets.length" :thumbnails="sheetThumbnails" :bytes="sheetBytesSum" :beam="sheetBeam" :fit="sheetFit" /><!-- beneath the last bucket, on the same scroll, and only when there are buckets to weigh -->
</div>

</template>
<style scoped>

.mySheet {
	background-color: var(--color-paper);
}
.myEmpty {
	color: var(--color-faint);
}

</style>
