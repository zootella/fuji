<script setup>//the contact sheet: one folder, seen whole

import {ref, computed} from 'vue'
import {modelList, modelOpen, modelStand} from '../model.js'
import {settings} from '../settings.js'
import Card from './Card.vue'

//one card, holding the first card.images pictures of the folder and no more, for now: the rest of a large folder is simply not shown, which keeps a smoke test short. The sheet was a stack of cards down one scroll and will be again; this file cuts the list, and sizing, arranging, loading and holding are the flow's, one level down. The settings panel takes this view's place in the same window, and s trades the two

const sheetStarted = ref(false)//the shell has shown this view at least once, which is what the guard below turns on
const sheetImages = ref(0)//how many pictures the card holds, read from settings each time the sheet comes on screen: the settings panel is where it changes, and the sheet is never showing while it does, so reading on arrival is all the watching it needs

//that guard covers only a sheet that has never been shown. After that, a folder opened on the table rebuilds these cards behind it. The flow waits on modelShowing before it reads, decodes or draws anything, so a hidden sheet does none of that inside the table's frames
const sheetCards = computed(() => {
	if (!sheetStarted.value) return []//v-show hides without unmounting, so without this an unlooked-at sheet would still render and quietly load a whole folder on the table's drop
	if (!modelList.value.length) return []
	return [modelList.value.slice(0, sheetImages.value)]//the first card only; a list of one, so the template and the flow below are the same as when there were many
})

function start() { sheetImages.value = settings.card.images; sheetStarted.value = true }//the shell calls this each time this view comes on screen; a count that changed rebuilds the card, and one that did not changes nothing
const emit = defineEmits(['table', 'settings'])//a double-clicked thumbnail, for the shell to show the table fullscreen, and s, for the settings panel in this same window; which view is showing is the shell's, so this only asks
function onKey(e) {
	if (e.key == 's' && !e.ctrlKey && !e.metaKey) emit('settings')//a temporary way in, until fuji has a better place for one
}
async function onDrop(path) { await modelOpen(path) }//a picture dropped or opened here: the model lists its folder and the cards build from that, and the table shows the picture itself when the user goes to it
function onDoubleClick(e) {//a thumbnail double-clicked: stand the model on its picture and ask for the table, which shows whatever the model is standing on when it comes back
	let path = e.target.closest('[data-path]')?.dataset.path//the flow names every tile's path on the tile itself, so one listener here serves them all
	if (!path) return//between tiles, or on the empty sheet
	modelStand(path)
	emit('table')
}
function onResize() {}//and nothing to remeasure: the wrapping is the engine's job, and this view measures nothing, which is what lets it stay mounted

defineExpose({start, onKey, onResize, onDrop})//the same calls every view answers

</script>
<template>

<!-- display none destroys the layout box and the scroll position with it, so leaving and returning starts at the top; that is the behaviour wanted for now -->
<div class="mySheet w-full h-full overflow-y-auto select-none" @dblclick="onDoubleClick">
	<div v-if="!sheetCards.length" class="myEmpty myMono w-full h-full flex items-center justify-center">contact sheet - drop a picture here to open its folder</div>
	<!-- keyed on contents, so a card whose images changed is rebuilt rather than reused: a reused card keeps canvases painted from images it no longer holds and never paints the new ones -->
	<Card v-for="card in sheetCards" :key="card.join()" :paths="card" />
</div>

</template>
<style scoped>

.mySheet {
	background-color: black;
}
.myEmpty {
	color: #737373;
}

</style>
