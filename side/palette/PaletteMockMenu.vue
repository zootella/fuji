<script setup>//a right-click menu as MockMenu draws it, in place, for a pane on the palette. MockMenu opens its menu fixed over the whole window, behind a layer that takes the next press or key, so it cannot sit in a pane and take the pane's light or dark, and this draws its card instead. Its rules and its menu for the list's selected rows are MockMenu's own, copied, and change when those do. A row lights under the pointer, and the row named lit is lit already

const props = defineProps({
	lit: {type: String, default: ''},//the word of a row to show lit, as if under the pointer
})
const items = ['Open', 'Open With >', 'Open in New Window', '', 'Copy', 'Copy Path', '~Paste', '', 'Rename', 'Move to Trash', '', 'Get Info'].map(item => ({name: item.replace(/^~|\s>$/g, ''), off: item.startsWith('~'), more: item.endsWith(' >')}))//MockMenu's Files, the menu for the rows selected in the list, in its own notation: a blank for a separator, a ~ before an item that cannot be chosen, and a > after one with a submenu

</script>
<template>

<div class="myMenu">
	<template v-for="(item, i) in items" :key="i">
		<div v-if="item.name" class="myItem" :class="{myOff: item.off}" :data-lit="(item.name == props.lit && !item.off) || null">{{item.name}}<span v-if="item.more" class="myMore">›</span></div>
		<div v-else class="myLine"></div>
	</template>
</div>

</template>
<style scoped>

.myMenu { width: max-content; padding: 5px; border-radius: 10px; color: var(--color-text); background-color: color-mix(in srgb, var(--color-elevated-surface) 95%, transparent); backdrop-filter: blur(2px); box-shadow: inset 0 0 0 1px var(--color-fuji-menu-rim), 0 0 0 1px var(--color-fuji-menu-edge), 0 4px 12px var(--color-fuji-menu-edge); user-select: none; cursor: default } /* MockMenu's myMenu, with the backdrop's select-none and arrow */
.myItem { height: 24px; padding: 0 11px; display: flex; align-items: center; border-radius: 5px } /* MockMenu's myItem */
.myItem:hover:not(.myOff), .myItem[data-lit] { background-color: var(--color-accent); color: var(--color-on-accent) }
.myOff { color: var(--color-text-placeholder) }
.myMore { margin-left: auto; padding-left: 24px }
.myLine { margin: 5px 11px; border-top: 1px solid var(--color-border) }

</style>
