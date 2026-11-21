<script setup>//a menu from bars.txt drawn in place, as FujiBar draws the card a menu opens in, for a pane on the color page. A menu FujiBar opens goes to the end of the page, so nothing a bar sits inside can clip it, and there it takes the page's light or dark rather than the pane's, so a pane that shows a menu in its mode draws this one. Its rules are FujiBar's own for the card and its rows, copied, and change when those do. A row lights under the pointer, and the row named lit is lit already

import {barGet, barIcons} from '../src/bar.js'

const props = defineProps({
	name: {type: String, required: true},//the menu's name in bars.txt
	lit: {type: String, default: ''},//the word of a row to show lit, as if under the pointer
})
const bar = barGet(props.name)
if (bar.kind != 'menu') throw new Error(`expected a menu to draw, and ${bar.name} is a ${bar.kind}`)

</script>
<template>

<div class="myCard mySans">
	<template v-for="(item, i) in bar.items" :key="i">
		<div v-if="item.separator" class="myCardLine"></div>
		<div v-else class="myItem" :data-lit="item.word == props.lit || null">
			<span class="myGutter"><span v-if="item.icon" class="myIcon" v-html="barIcons[item.icon]"></span></span><!-- a drawing fuji bundles, never anything from outside, which is what lets it go in as markup -->
			<span>{{item.label}}</span>
		</div>
	</template>
</div>

</template>
<style scoped>

.myCard { width: max-content; padding: 1px; border: 1px solid var(--color-gray-500); background-color: var(--color-face); background-image: linear-gradient(to right, var(--color-elevated-surface) 26px, transparent 26px); color: var(--color-text); user-select: none; cursor: default; filter: drop-shadow(2px 2px 2px var(--color-fuji-shade)) } /* FujiBar's myBarCard for a menu, on the menu's own face with its gutter shaded, and the shadow FujiBar casts from the card and its title together */
.myItem { display: flex; align-items: center; height: calc(1rlh + 8px); padding-right: 24px; border: 1px solid transparent; white-space: nowrap } /* FujiBar's myBarItem */
.myItem:hover, .myItem[data-lit] { border-color: var(--color-menu-hot-edge); background-color: var(--color-menu-hot); color: var(--color-on-menu-hot) } /* FujiBar's myBarItem:hover, the platform's own look */
.myGutter { flex: none; width: 24px; margin-right: 8px; display: flex; justify-content: center } /* FujiBar's myBarGutter */
.myIcon { display: flex }
.myIcon :deep(svg) { width: 16px; height: 16px; stroke-width: 1.5 } /* a drawing in a menu, 16 css pixels square, its stroke a css pixel */
.myItem:hover .myIcon :deep(svg), .myItem[data-lit] .myIcon :deep(svg) { position: relative; left: -1px; top: -1px; filter: drop-shadow(2px 2px 0 var(--color-fuji-lift)) } /* lifted over a sharp shadow, as FujiBar lifts the command under the pointer */
.myCardLine { margin: 3px 0 3px 33px; border-top: 1px solid var(--color-border) } /* FujiBar's myBarCardLine */

</style>
