<script setup>//a mock context menu in each of the looks fuji's menus are tried in; its items do nothing, and choosing one only closes it

import {ref, nextTick} from 'vue'

const looks = {//each look's items, a blank for a separator, a ~ before one that cannot be chosen, and a > after one with a submenu
	Simple: ['Open', 'Open in New Window', '', 'Copy', 'Copy Path', '', 'Rename', 'Move to Trash'],
	Chrome: ['Open Image in New Tab', 'Save Image As...', 'Copy Image', 'Copy Image Address', 'Create QR Code for this Image', 'Search This Image with Google Lens', '', 'Resume blocking ads', 'Save image', '', 'Inspect'],//Chrome's own on a picture, to set beside the screenshot of it
	Fuji: ['Open', 'Open With >', 'Open in New Window', '', 'Copy', 'Copy Path', '~Paste', '', 'Rename', 'Move to Trash', '', 'Get Info'],//items a contact sheet's menu might hold, with one that cannot be chosen and one with a submenu
	Glass: ['New Window >', 'New Tab >', 'Select >', '', '~Download with scp', '~Search the Web for Selection', '', 'Split Pane Vertically', 'Split Pane Horizontally', '', '~Copy', 'Paste', '~Save', '', 'Select All', 'Clear Buffer', '', 'Edit Session...', '', 'Close', 'Restart', '', 'Terminal State >'],//iTerm's on Golden Gate, the same way
}
function parse(item) { return {name: item.replace(/^~|\s>$/g, ''), off: item.startsWith('~'), more: item.endsWith(' >')} }

const open = ref(false)
const look = ref('Simple')
const at = ref({x: 0, y: 0})//the menu's top left corner in the window, in css pixels
const menu = ref(null)

async function show(x, y, name) {//open in this look at this point, then move in from any edge of the window the menu would cross
	open.value = true
	look.value = name
	at.value = {x, y}
	await nextTick()
	let box = menu.value.getBoundingClientRect()
	at.value = {x: Math.max(0, Math.min(x, innerWidth - box.width)), y: Math.max(0, Math.min(y, innerHeight - box.height))}
}
function hide() { open.value = false }

defineExpose({show, hide})

</script>
<template>

<div v-if="open" class="fixed inset-0" @pointerdown="hide" @contextmenu.prevent.stop="e => show(e.clientX, e.clientY, look)"><!-- the rest of the window, where a press closes the menu and a right click opens it again there, and which keeps the wheel from scrolling the sheet beneath -->
	<div ref="menu" class="fixed text-strong" :class="`my${look}`" :style="{left: `${at.x}px`, top: `${at.y}px`}" @pointerdown.stop>
		<template v-for="(item, i) in looks[look].map(parse)" :key="i">
			<div v-if="item.name" class="myItem" :class="{myOff: item.off}" @click="item.off || hide()">{{item.name}}<span v-if="item.more" class="myMore">›</span></div>
			<div v-else class="myLine"></div>
		</template>
	</div>
</div>

</template>
<style scoped>

.myOff { color: var(--color-menu-off) }
.myMore { margin-left: auto; padding-left: 24px }

.mySimple { padding: 4px 0; border: 1px solid var(--color-line); background-color: var(--color-paper) }
.mySimple .myItem { padding: 0 16px }
.mySimple .myItem:hover { background-color: var(--color-select); color: var(--color-on-select) }
.mySimple .myLine { margin: 4px 0; border-top: 1px solid var(--color-line) }

.myChrome {
	padding: 5px;
	border-radius: 10px;
	background-color: var(--color-chrome-menu);/* the translucency: a pale tint with the page showing through it */
	backdrop-filter: blur(24px) saturate(180%);/* the frost: what is behind blurred, and its colors strengthened so they still read through the tint */
	box-shadow: inset 0 0 0 1px var(--color-menu-rim), 0 0 0 1px var(--color-menu-edge), 0 4px 12px var(--color-menu-edge);/* the edge, from the inside out: a light rim, a dark hairline, and a soft shadow */
}
.myChrome .myItem { height: 24px; padding: 0 11px; display: flex; align-items: center; border-radius: 5px }
.myChrome .myItem:hover:not(.myOff) { background-color: var(--color-menu-highlight); color: var(--color-on-select) }
.myChrome .myLine { margin: 5px 11px; border-top: 1px solid var(--color-menu-line) }

.myGlass {
	padding: 5px;
	border-radius: 12px;
	background-color: var(--color-glass-menu);/* white at 40%, so over a light page the glass stays nearly the page's own color and over a dark one it turns gray */
	backdrop-filter: blur(10px) saturate(200%);/* a tighter blur than Chrome's, measured from how far the glass spread a sharp light and dark boundary behind it */
	box-shadow: inset 0 0 0 1px var(--color-menu-rim), 0 0 0 1px var(--color-menu-edge), 0 8px 24px var(--color-menu-edge);
}
.myGlass .myItem { height: 24px; padding: 0 11px; display: flex; align-items: center; border-radius: 6px }
.myGlass .myItem:hover:not(.myOff) { background-color: var(--color-menu-highlight); color: var(--color-on-select) }
.myGlass .myLine { margin: 5px 11px; border-top: 1px solid var(--color-menu-line) }

.myFuji {/* fuji's own, its material, text, grayed items and separators in the colors of Zed's One Light and One Dark */
	padding: 5px;
	border-radius: 10px;
	color: var(--color-fuji-menu-text);
	background-color: var(--color-fuji-menu);
	backdrop-filter: blur(2px);
	box-shadow: inset 0 0 0 1px var(--color-menu-rim), 0 0 0 1px var(--color-menu-edge), 0 4px 12px var(--color-menu-edge);
}
.myFuji .myItem { height: 24px; padding: 0 11px; display: flex; align-items: center; border-radius: 5px }
.myFuji .myItem:hover:not(.myOff) { background-color: var(--color-menu-highlight); color: var(--color-on-select) }
.myFuji .myOff { color: var(--color-quiet) }/* Zed's text.disabled, which fuji already has */
.myFuji .myLine { margin: 5px 11px; border-top: 1px solid var(--color-fuji-menu-line) }

</style>
