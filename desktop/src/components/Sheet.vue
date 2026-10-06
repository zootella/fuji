<script setup>//the contact sheet: one folder, as a stack of cards of thumbnails

import {ref, computed} from 'vue'
import {modelList, modelOpen, modelStand} from '../model.js'
import {settings, settingsSet} from '../settings.js'
import {fitNames} from '../fit.js'
import Card from './Card.vue'

//a stack of cards down one scroll, sheet.cards of them with card.images pictures each, from the start of the folder, and no more: the rest of a large folder is simply not shown, so the two settings together are a ceiling on how many thumbnails the sheet ever holds. This file cuts the list into cards, and sizing, arranging, loading and holding are the flow's, one level down. The settings panel takes this view's place in the same window, and s trades the two

const sheetStarted = ref(false)//the shell has shown this view at least once, which is what the guard below turns on
const sheetImages = ref(0)//how many pictures a card holds
const sheetCardCount = ref(0)//and how many cards the sheet holds; both read from settings each time the sheet comes on screen, because the settings panel is where they change and the sheet is never showing while it does, so reading on arrival is all the watching they need
const sheetFit = ref('')//the fit the toolbar shows chosen, and part of every card's key, so choosing another rebuilds them all; read in start like the two above, because this view is made before the shell has read fuji.toml, and a value taken now would be the factory's

//that guard covers only a sheet that has never been shown. After that, a folder opened on the table rebuilds these cards behind it, and they fill there, hidden, as they would on screen
const sheetCards = computed(() => {
	if (!sheetStarted.value) return []//v-show hides without unmounting, so without this an unlooked-at sheet would still render and quietly load a whole folder on the table's drop
	let cards = []
	for (let i = 0; i < sheetCardCount.value; i++) {
		let card = modelList.value.slice(i * sheetImages.value, (i + 1) * sheetImages.value)
		if (!card.length) break//the folder ran out before the sheet did
		cards.push(card)
	}
	return cards
})

function start() { sheetImages.value = settings.card.images; sheetCardCount.value = settings.sheet.cards; sheetFit.value = settings.thumbnail.fit; sheetStarted.value = true }//the shell calls this each time this view comes on screen; a count that changed rebuilds the cards, and one that did not changes nothing
function fitChoose(fit) {//the toolbar: write the fit to the setting, which every flow reads as it is made, then change the key that remakes the cards; the setting first, so the new cards read the new fit
	if (settingsSet('thumbnail', 'fit', fit)) sheetFit.value = fit
}
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
	<!-- the toolbar, a spike for now and the start of the sheet's own: one radio per fit, sticky so it stays at the top as the cards scroll under it -->
	<div v-if="sheetCards.length" class="mySheetBar myMono sticky top-0 z-10 flex flex-wrap items-center gap-x-4 gap-y-1 px-1.5 py-1" role="radiogroup" aria-label="Fit">
		<span>Fit</span>
		<label v-for="fit in fitNames" :key="fit" class="flex items-center gap-1"><input type="radio" name="fit" :value="fit" :checked="fit == sheetFit" @change="fitChoose(fit)" />{{fit.replace(/Fit$/, '')}}</label>
	</div>
	<!-- keyed on contents and the fit, so a card whose images or fit changed is rebuilt rather than reused: a reused card keeps canvases painted from images it no longer holds and never paints the new ones, and its flow read the fit once, when it was made -->
	<Card v-for="card in sheetCards" :key="card.join() + sheetFit" :paths="card" />
</div>

</template>
<style scoped>

.mySheet {
	background-color: var(--color-paper);
}
.mySheetBar {
	background-color: var(--color-paper); /* opaque, so the cards scroll under it rather than show through */
	color: var(--color-ink);
}
.myEmpty {
	color: var(--color-faint);
}

</style>
