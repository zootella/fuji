<script setup>//the color page: a tab for each of its pages across the top, and the one chosen below, which the address remembers so a reload comes back to it

import {ref, computed} from 'vue'
import ColorTheme from './ColorTheme.vue'
import ColorButtons from './ColorButtons.vue'
import ColorAccent from './ColorAccent.vue'
import ColorBrand from './ColorBrand.vue'

const tabs = [
	{key: 'theme',   title: 'Theme',         page: ColorTheme},//every color style.css names
	{key: 'buttons', title: 'Button Color',  page: ColorButtons},//candidate palettes for the bar's drawings
	{key: 'accent',  title: 'Accent Colors', page: ColorAccent},//where the accent paints, and the window buttons
	{key: 'brand',   title: 'Brand Colors',  page: ColorBrand},//the brand colors first proposed, and the one fuji took
]
const chosen = ref(tabs.some(t => t.key == location.hash.slice(1)) ? location.hash.slice(1) : tabs[0].key)
const showing = computed(() => tabs.find(t => t.key == chosen.value).page)
function choose(key) { chosen.value = key; history.replaceState(null, '', `#${key}`) }//in the address without a step in the history, so back leaves the page rather than walking the tabs

</script>
<template>

<div class="myPage">
	<div class="myTabs">
		<button v-for="t in tabs" :key="t.key" class="myButton" :aria-pressed="t.key == chosen" @click="choose(t.key)">{{t.title}}</button><!-- fuji's own button with an edge and a face, which shows the one in force in a row of choices -->
	</div>
	<div class="myShowing"><component :is="showing" /></div>
</div>

</template>
<style scoped>

.myPage { height: 100%; display: flex; flex-direction: column; background-color: var(--color-background); color: var(--color-text) } /* the window's height, which style.css chains down from the root, since fuji's page never scrolls */
.myTabs { flex: none; display: flex; gap: 4px; padding: 8px 24px; border-bottom: 1px solid var(--color-border) }
.myShowing { flex: 1; min-height: 0; overflow: auto; padding: 24px } /* the tab chosen, which scrolls under the tabs */

/* what every tab is made of, said once here and reached into each tab's page: its headings and words, and a row of panes, one in light and one in dark. In a layer, so a tab's own rules, which are in none, win over these wherever the two meet, with no contest of specificity */
@layer components {
	.myShowing :deep(h1) { margin: 0 0 8px; font-size: 18px; font-weight: 600 }
	.myShowing :deep(h2) { margin: 32px 0 4px; font-size: 15px; font-weight: 600 }
	.myShowing :deep(p) { margin: 0 0 12px; max-width: 64em; color: var(--color-text-muted) }
	.myShowing :deep(.myPanes) { display: flex; flex-wrap: wrap; border: 1px solid var(--color-border) }
	.myShowing :deep(.myPane) { flex: 1 1 560px; min-width: 0; padding: 16px 20px; display: flex; flex-direction: column; gap: 16px; background-color: var(--color-background); color: var(--color-text) } /* a pane on the content's own ground, which a tab can set to another of fuji's */
	.myShowing :deep(.myPane[data-mode="light"]) { color-scheme: light } /* every one of fuji's colors inside it in light, which palette.js makes possible */
	.myShowing :deep(.myPane[data-mode="dark"]) { color-scheme: dark }
}

</style>
