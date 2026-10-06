<script setup>//the contact sheet: one folder, as a stack of buckets of thumbnails

import {ref, computed, onMounted} from 'vue'
import {documentDir, downloadDir} from '@tauri-apps/api/path'
import {modelList, modelOpen, modelOpenFolder, modelStand} from '../model.js'
import {forwardize} from './library.js'
import {logTrouble} from '../log.js'
import {settings, settingsSet} from '../settings.js'
import {fitNames} from '../fit.js'
import Bucket from './Bucket.vue'

//a stack of buckets down one scroll, sheet.buckets of them with bucket.images pictures each, from the start of the folder, and no more: the rest of a large folder is simply not shown, so the two settings together are a ceiling on how many thumbnails the sheet ever holds. This file cuts the list into buckets, and sizing, arranging, loading and holding are the flow's, one level down. The settings panel takes this view's place in the same window, and s trades the two

const sheetStarted = ref(false)//the shell has shown this view at least once, which is what the guard below turns on
const sheetImages = ref(0)//how many pictures a bucket holds
const sheetBucketCount = ref(0)//and how many buckets the sheet holds; both read from settings each time the sheet comes on screen, because the settings panel is where they change and the sheet is never showing while it does, so reading on arrival is all the watching they need
const sheetFit = ref('')//the fit the toolbar shows chosen, and part of every bucket's key, so choosing another rebuilds them all; read in start like the two above, because this view is made before the shell has read fuji.toml, and a value taken now would be the factory's

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

function start() { sheetImages.value = settings.bucket.images; sheetBucketCount.value = settings.sheet.buckets; sheetFit.value = settings.thumbnail.fit; sheetStarted.value = true }//the shell calls this each time this view comes on screen; a count that changed rebuilds the buckets, and one that did not changes nothing
function fitChoose(fit) {//the toolbar: write the fit to the setting, which every flow reads as it is made, then change the key that remakes the buckets; the setting first, so the new buckets read the new fit
	if (settingsSet('thumbnail', 'fit', fit)) sheetFit.value = fit
}
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
	<!-- the toolbar, a spike for now and the start of the sheet's own: one radio per fit, sticky so it stays at the top as the buckets scroll under it -->
	<div v-if="sheetBuckets.length" class="mySheetBar myMono sticky top-0 z-10 flex flex-wrap items-center gap-x-4 gap-y-1 px-1.5 py-1" role="radiogroup" aria-label="Fit">
		<span>Fit</span>
		<label v-for="fit in fitNames" :key="fit" class="flex items-center gap-1"><input type="radio" name="fit" :value="fit" :checked="fit == sheetFit" @change="fitChoose(fit)" />{{fit.replace(/Fit$/, '')}}</label>
	</div>
	<!-- keyed on contents and the fit, so a bucket whose images or fit changed is rebuilt rather than reused: a reused bucket keeps canvases painted from images it no longer holds and never paints the new ones, and its flow read the fit once, when it was made -->
	<Bucket v-for="(bucket, i) in sheetBuckets" :key="bucket.join() + sheetFit" :paths="bucket" :first="i * sheetImages + 1" :total="modelList.length" /><!-- first and total for the bucket's caption, counted from 1 the way a person counts, since the slicing that knows them is here -->
</div>

</template>
<style scoped>

.mySheet {
	background-color: var(--color-paper);
}
.mySheetBar {
	background-color: var(--color-paper); /* the sheet's own color, as a toolbar over its content is in Zed, and opaque, so the buckets scroll under it rather than show through */
	color: var(--color-ink);
}
.myEmpty {
	color: var(--color-faint);
}

</style>
