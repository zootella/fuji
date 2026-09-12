<script setup>//the contact sheet: one folder, seen whole

import {ref, computed} from 'vue'
import {modelList} from '../model.js'
import {settings} from '../settings.js'
import Card from './Card.vue'

//one scroll, top to bottom, running over a stack of cards rather than over the thumbnails themselves; each card holds up to a capped number of images from one folder, so 220 at a cap of 200 is a card of 200 and then a card of 20. This file cuts the list; sizing, arranging, loading and holding are the flow's, one level down

const sheetStarted = ref(false)//the shell has shown this view at least once, which is what the guard below turns on

//that guard covers only a sheet that has never been shown. After the first c, every drop on the table rebuilds these cards behind it. The flow waits on modelShowing before it reads, decodes or draws anything, so a hidden sheet does none of that inside the table's frames
const sheetCards = computed(() => {
	if (!sheetStarted.value) return []//v-show hides without unmounting, so without this an unlooked-at sheet would still render and quietly load a whole folder on the table's drop
	let cards = []
	for (let i = 0; i < modelList.value.length; i += settings.card.images) cards.push(modelList.value.slice(i, i + settings.card.images))
	return cards
})

function start()    { sheetStarted.value = true }//the shell calls this when this view first comes on screen
function onKey(e)   {}//nothing to do with a key yet
function onResize() {}//and nothing to remeasure: the wrapping is the engine's job, and this view measures nothing, which is what lets it stay mounted

defineExpose({start, onKey, onResize})//the same calls every view answers; onDrop and isFullscreen arrive when this view has a use for them

</script>
<template>

<!-- display none destroys the layout box and the scroll position with it, so leaving and returning starts at the top; that is the behaviour wanted for now -->
<div class="mySheet w-screen h-screen overflow-y-auto select-none">
	<div v-if="!sheetCards.length" class="myEmpty w-screen h-screen flex items-center justify-center">contact sheet - drop an image on the table to open a folder</div>
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
	font-family: monospace;
	font-size: 0.875rem;
}

</style>
