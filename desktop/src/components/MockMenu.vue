<script setup>//a mock context menu in the look fuji's menus will take; its items do nothing, and choosing one only closes it

import {ref, nextTick} from 'vue'

const items = ['Open', 'Open With >', 'Open in New Window', '', 'Copy', 'Copy Path', '~Paste', '', 'Rename', 'Move to Trash', '', 'Get Info'].map(parse)//items a contact sheet's menu might hold: a blank for a separator, a ~ before one that cannot be chosen, and a > after one with a submenu
function parse(item) { return {name: item.replace(/^~|\s>$/g, ''), off: item.startsWith('~'), more: item.endsWith(' >')} }

const open = ref(false)
const at = ref({x: 0, y: 0})//the menu's top left corner in the window, in css pixels
const menu = ref(null)

async function show(x, y) {//open at this point, then move in from any edge of the window the menu would cross
	open.value = true
	at.value = {x, y}
	await nextTick()
	let box = menu.value.getBoundingClientRect()
	at.value = {x: Math.max(0, Math.min(x, innerWidth - box.width)), y: Math.max(0, Math.min(y, innerHeight - box.height))}
}
function hide() { open.value = false }

defineExpose({show, hide})

</script>
<template>

<div v-if="open" class="fixed inset-0" @pointerdown="hide" @contextmenu.prevent.stop="e => show(e.clientX, e.clientY)"><!-- the rest of the window, where a press closes the menu and a right click opens it again there, and which keeps the wheel from scrolling the sheet beneath -->
	<div ref="menu" class="myMenu fixed" :style="{left: `${at.x}px`, top: `${at.y}px`}" @pointerdown.stop>
		<template v-for="(item, i) in items" :key="i">
			<div v-if="item.name" class="myItem" :class="{myOff: item.off}" @click="item.off || hide()">{{item.name}}<span v-if="item.more" class="myMore">›</span></div>
			<div v-else class="myLine"></div>
		</template>
	</div>
</div>

</template>
<style scoped>

.myMenu {/* its material, text, grayed items and separators in the colors of Zed's One Light and One Dark */
	padding: 5px;
	border-radius: 10px;
	color: var(--color-zed-text);
	background-color: color-mix(in srgb, var(--color-zed-elevated-surface-background) 95%, transparent);/* Zed's menu color, a little translucent */
	backdrop-filter: blur(2px);
	box-shadow: inset 0 0 0 1px var(--color-fuji-menu-rim), 0 0 0 1px var(--color-fuji-menu-edge), 0 4px 12px var(--color-fuji-menu-edge);/* the edge, from the inside out: a light rim, a dark hairline, and a soft shadow */
}
.myItem { height: 24px; padding: 0 11px; display: flex; align-items: center; border-radius: 5px }
.myItem:hover:not(.myOff) { background-color: var(--color-accent); color: var(--color-on-accent) }
.myOff { color: var(--color-zed-text-placeholder) }/* Zed's text.disabled, the same gray as its text.placeholder */
.myMore { margin-left: auto; padding-left: 24px }
.myLine { margin: 5px 11px; border-top: 1px solid var(--color-zed-border) }

</style>
