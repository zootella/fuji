<script setup>//context menus in the look fuji's menus will take, by name; show hands back the item chosen, which the caller of a mock menu, like Files, ignores

import {ref, nextTick, onBeforeUnmount} from 'vue'
import {platform} from './library.js'

const menus = {//menus a file manager might show, by name: a blank for a separator, a ~ before an item that cannot be chosen, and a > after one with a submenu
	Thumbnail: ['Open', 'Open With >', 'Show in Folder', '', 'Copy', 'Copy Path', '', 'Rename', 'Move to Trash', '', 'Get Info'],//for the thumbnail clicked on the contact sheet
	Files:     ['Open', 'Open With >', 'Open in New Window', '', 'Copy', 'Copy Path', '~Paste', '', 'Rename', 'Move to Trash', '', 'Get Info'],//for the rows selected in the list
	Folder:    ['New Folder', '', '~Paste', '', 'Sort By >', '', 'Get Info'],//for the folder itself, from the list's space below its rows
	Table:     ['Contact Sheet', ...{mac: ['Close', 'Quit'], windows: ['Exit'], linux: ['Quit']}[platform()]],//for the light table: back to the contact sheet, and the way out in the platform's words. The mac keeps fuji running after a window closes, so it has both; on windows and linux closing the window ends the program, so one item does both
}
function parse(item) { return {name: item.replace(/^~|\s>$/g, ''), off: item.startsWith('~'), more: item.endsWith(' >')} }

const open = ref(false)
const items = ref([])
const at = ref({x: 0, y: 0})//the menu's top left corner in the window, in css pixels
const menu = ref(null)
let answer = null//resolves what show handed back, with the item chosen or blank

async function show(x, y, name) {//open the menu of this name at this point, then move in from any edge of the window the menu would cross; what it hands back gives the item chosen, or blank when the menu closes without one
	if (!menus[name]) throw new Error(`no menu named ${name}`)
	let chosen = new Promise(resolve => { answer?.(''); answer = resolve })
	items.value = menus[name].map(parse)
	open.value = true
	window.addEventListener('keydown', onMenuKey, true)//on the way down, ahead of the shell's own keys
	at.value = {x, y}
	await nextTick()
	let box = menu.value.getBoundingClientRect()
	at.value = {x: Math.max(0, Math.min(x, innerWidth - box.width)), y: Math.max(0, Math.min(y, innerHeight - box.height))}
	return chosen
}
function hide() { close('') }
function close(word) {
	open.value = false
	window.removeEventListener('keydown', onMenuKey, true)
	let a = answer; answer = null
	a?.(word)
}
onBeforeUnmount(hide)
function onMenuKey(e) {//while a menu is open the keys are its own, as a system menu's are: one closes it and goes no further, so escape on the light table closes the menu rather than the window
	if (['Shift', 'Control', 'Alt', 'Meta'].includes(e.key)) return//a modifier pressed alone, which a system menu stays open through
	e.preventDefault(); e.stopPropagation()
	hide()
}
async function onBackdropMenu(e) {//a right click outside the open menu closes it and goes on to whatever is under the pointer, as a system menu's does, so the menu that opens next is the one for that place
	hide()
	await nextTick()
	document.elementFromPoint(e.clientX, e.clientY)?.dispatchEvent(new MouseEvent('contextmenu', {bubbles: true, cancelable: true, clientX: e.clientX, clientY: e.clientY, button: 2}))
}

defineExpose({show, hide})

</script>
<template>

<div v-if="open" class="fixed inset-0 select-none cursor-default" @pointerdown.stop="hide" @dblclick.stop @wheel.prevent.stop @contextmenu.prevent.stop="onBackdropMenu"><!-- the rest of the window, where a press closes the menu and goes no further, as the wheel and a double-click go no further, so the view beneath never acts on what only closed the menu -->
	<div ref="menu" class="myMenu fixed" :style="{left: `${at.x}px`, top: `${at.y}px`}" @pointerdown.stop>
		<template v-for="(item, i) in items" :key="i">
			<div v-if="item.name" class="myItem" :class="{myOff: item.off}" @click="item.off || close(item.name)">{{item.name}}<span v-if="item.more" class="myMore">›</span></div>
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
