<script setup>//plain img tags, sized small, wrapped like words, and nothing else

import {ref, onBeforeUnmount} from 'vue'
import {cacheNeed, cacheRelease} from '../cache.js'
import {settingsThumbnailBox} from '../settings.js'

/*
A Flow is a thumbnail strategy: given images of unrelated dimensions, it decides how each becomes a thumbnail, how they arrange as the card's width changes, and what is loaded and held. This is the first one, and deliberately the least clever thing that works — a webpage from 1998, before anyone made small copies of their pictures to upload instead. Full-size originals in ordinary img tags, sized down by two lines of CSS, set left to right, wrapping at the edge. No windowing, no scheduling, no canvas.

That is the experiment rather than an oversight. A wall of images that scrolls is the most ordinary thing on the web, and the engines have spent twenty years on it — subsampling large decodes, dropping the pixels of images scrolled away, rebuilding them from source on the way back. Fuji can neither see that happening nor do it as well by hand, so the first question is not how to beat the engine but whether it needs beating, and the honest way to ask is to hand it everything and watch. card.md carries what stacking this against CanvasFlow is meant to reveal.

One cost is knowingly left in, because measuring it comes before fixing it: nothing is released while scrolling, only when the card goes away. A second was taken out on 2026-09-08. The store used to decode every image at full size for whoever asked, so a card of two hundred forced two hundred decodes before the engine had decided anything, and the arm meant to measure the engine deciding for itself never let it. Now this flow asks the store for the bytes and the url only, and the engine's decode of the img below is the only decode there is.
*/

const flowHolder = 'TagFlow'//on every reference this flow takes, so a leak has a name and not just a size
const flowBox = settingsThumbnailBox()//read once: changing the size is a restyle here, which is most of what this flow is cheap at

const props = defineProps({
	paths: {type: Array, required: true},//already in the model's order
})

const flowTiles = ref(props.paths.map(path => ({path, url: ''})))//blank url until the store has the bytes; one small reactive object each, so a load fills one in without rebuilding the array. Built once and never replaced, which is what makes it the record of exactly what this flow needed

for (let tile of flowTiles.value) {
	cacheNeed(tile.path, flowHolder, {decode: false})//bytes and a url, and no element: the img below is the engine's to decode when and how it likes, which is the whole experiment. The loads race, and a file that would not read leaves its tile blank rather than showing a broken picture
		.then(entry => { if (!entry.error) tile.url = entry.url })
		.catch(error => console.error('loading a thumbnail:', error))//the store answers trouble with entry.error rather than a rejection, so this is the gate for a store that broke rather than a file that did
}
onBeforeUnmount(() => {
	for (let tile of flowTiles.value) cacheRelease(tile.path, flowHolder)
})

</script>
<template>

<!-- items-start so a short image keeps its own height instead of stretching to the tallest in its row -->
<div class="flex flex-wrap items-start" :style="{'--flowBox': flowBox+'px'}">
	<template v-for="tile in flowTiles" :key="tile.path">
		<img v-if="tile.url" class="myThumbnail" :src="tile.url" />
	</template>
</div>

</template>
<style scoped>

.myThumbnail {
	max-width: var(--flowBox); /* only ever shrinks: a square lands exactly on the box, a wide one hits the limit on width alone, and an image already smaller keeps its own size */
	max-height: var(--flowBox);
}

</style>
