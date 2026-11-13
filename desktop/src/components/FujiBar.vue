<script setup>//a menubar or a toolbar from bars.txt, by its name, which sends the word of the item chosen to the one command handler of the page that places it, from a button or from the menu a button opens; bar.js reads the text and says how it is written

import {ref, provide, inject} from 'vue'
import {barGet, barIcons} from '../bar.js'

const props = defineProps({
	name: {type: String, required: true},
	pressed: {type: Array, default: () => []},//the words whose buttons show pressed in, which the page hands over and changes as its state does, since a toolbar is on screen all the time and has no moment before it shows to ask; one of the ways fuji-bar-dsl.md keeps open, taken for now
})
const emit = defineEmits(['command'])
const bar = barGet(props.name)//once, as the bar is made, since the text never changes while fuji runs
if (bar.kind != 'menubar' && bar.kind != 'toolbar') throw new Error(`expected a menubar or a toolbar to place, and ${bar.name} is a ${bar.kind}, which a button opens`)

/*
A button whose line in bars.txt ends with the name of a menu or a panel in brackets opens it, under the button and joined to it, the look of Office 2003's menus: the button goes forward with what it opens, drawn again over itself as the top of one card, so the open title and the card below read as one shape, a house on a skyscraper, outlined together and casting one shadow. Unlike Office's, the open button keeps the bar's color rather than darkening. A menubar's titles open menus and are words alone; a button with a drawing that opens something shows a small arrow beside its words, as toolbar buttons with menus have in Explorer and Office, and a menubar's titles show none, since every one of them opens a menu.

A menu's rows are this component's own. A panel's insides are the page's, handed over in the slot named panel, which hears the panel's name as panel, so a panel is whatever Vue the page writes, radio buttons, checkboxes, and a small toolbar of its own whose button opens a menu of its own, drawn over the panel.

What a button opens opens on the press, as Windows opens its menus, and a menu's row chooses on the release, so a press on the button dragged down the menu and let go on a row chooses it, as a click and a click do. While one is open, the pointer onto another button that opens something opens that instead, so a user can run along the bar reading menus. A press anywhere else closes it and goes no further, so the view beneath never acts on what only closed it, and a press outside a menu opened from inside a panel that is outside the panel too closes both.
*/
const open = ref(null)//what is showing: the item whose button opened it, the menu or panel as bars.txt has it, and that button's box in the window, in css pixels; or null
function show(item, button) {//open what this item opens, against its button, in place of whatever was open
	let box = button.getBoundingClientRect()
	open.value = {item, bar: barGet(item.opens), left: box.left, top: box.top, width: box.width, height: box.height}
}
function hide() { open.value = null }
defineExpose({hide})//for the page, which closes what is open as its window goes behind another, as the system's own menus close

const parent = inject('fujiBar', null)//the bar whose panel this bar is drawn in, if it is
const depth = parent ? parent.depth + 1 : 0
const z = 10 + 3 * depth//the backdrop's layer, with this bar's buttons that open something one above it and the card two above, so a menu opened from inside a panel stacks over the panel, its backdrop included
const drop = ref(null)//what this bar has open, the button drawn again and the card under it
function within(x, y) {//whether a point in the window is on what this bar has open, which a bar drawn inside its panel asks only while that panel is open
	for (let part of drop.value.children) {
		let box = part.getBoundingClientRect()
		if (x >= box.left && x < box.right && y >= box.top && y < box.bottom) return true
	}
	return false
}
function dismiss(x, y) {//a press outside what this bar has open closes it, and the panel it sits in too, and so on out, for each the press is outside of
	hide()
	if (parent && !parent.within(x, y)) parent.dismiss(x, y)
}
provide('fujiBar', {depth, within, dismiss})

function onPress(item, e) { if (e.button == 0 && item.opens) show(item, e.currentTarget) }//a button that opens something opens it on the press
function onEnter(item, e) { if (open.value && item.opens && open.value.item != item) show(item, e.currentTarget) }//and the pointer onto it while something else is open opens it instead
function onClick(item) { if (!item.opens) emit('command', item.word) }//a button that opens nothing sends its word on the click
function onChoose(item, e) {//a menu's row sends its word on the release, wherever the press began

	if (e.button != 0) return
	hide()
	emit('command', item.word)
}

</script>
<template>

<div class="myBar mySans" :data-kind="bar.kind" :data-open="open ? true : null" :style="{'--bar-z': z}" @mousedown.prevent><!-- the press's default cancelled, so a click on a button leaves the keyboard where it was, with the list or the tree the word may act on, as a toolbar button takes no focus on windows or the mac -->
	<template v-for="(item, i) in bar.items" :key="i">
		<div v-if="item.separator" class="myBarLine"></div>
		<button v-else class="myBarButton" :data-big="item.big || null" :data-opens="item.opens || null" :aria-pressed="props.pressed.includes(item.word) || null" tabindex="-1" @pointerdown="onPress(item, $event)" @pointerenter="onEnter(item, $event)" @click="onClick(item)">
			<span v-if="item.icon" class="myBarIcon" v-html="barIcons[item.icon]"></span><!-- a drawing fuji bundles, never anything from outside, which is what lets it go in as markup -->
			<span>{{item.label}}<span v-if="item.opens && item.icon" class="myBarArrow"></span></span>
		</button>
	</template>

<Teleport to="body"><!-- what a button opens, at the end of the page, so nothing the bar sits inside can clip it or paint over it; written inside the bar so the bar stays one element, which takes the classes the page gives it -->
	<template v-if="open">
		<div class="myBarBackdrop" :style="{'--bar-z': z}" @pointerdown="e => dismiss(e.clientX, e.clientY)" @mousedown.prevent @contextmenu.prevent></div><!-- the rest of the window, where a press of any button closes what is open and goes no further, the right button's menu included; under the bar's own buttons that open something, which stay where the pointer can reach them -->
		<div ref="drop" class="myBarDrop mySans" :style="{'--bar-z': z, left: `${open.left}px`, top: `${open.top}px`}" @mousedown.prevent @contextmenu.prevent>
			<div class="myBarButton myBarOpen" :data-big="open.item.big || null" :style="{width: `${open.width}px`, height: `${open.height}px`}" @pointerdown="hide"><!-- the button again, over itself, as the top of the card; a press on it closes what it opened, as a press on an open menu's title does -->
				<span v-if="open.item.icon" class="myBarIcon" v-html="barIcons[open.item.icon]"></span>
				<span>{{open.item.label}}<span v-if="open.item.icon" class="myBarArrow"></span></span>
			</div>
			<div class="myBarCard" :data-kind="open.bar.kind" :style="{top: `${open.height - 1}px`, minWidth: `${open.width}px`}"><!-- its top line under the button's last row, which the button drawn again covers, so the two join with no seam -->
				<slot v-if="open.bar.kind == 'panel'" name="panel" :panel="open.bar.name"></slot><!-- the page's own Vue, told which panel by its name -->
				<template v-else>
					<template v-for="(item, i) in open.bar.items" :key="i">
						<div v-if="item.separator" class="myBarCardLine"></div>
						<div v-else class="myBarItem" @pointerup="onChoose(item, $event)">
							<span class="myBarGutter"><span v-if="item.icon" class="myBarIcon" v-html="barIcons[item.icon]"></span></span>
							<span>{{item.label}}</span>
						</div>
					</template>
				</template>
			</div>
		</div>
	</template>
</Teleport>
</div>

</template>
<style scoped>

.myBar { display: flex; padding: 2px 4px; user-select: none; color: var(--color-zed-text) }
.myBar, .myBarOpen, .myBarCard { background-color: var(--bar-ground) } /* the bar's own ground, which style.css sets by platform: on windows the color of the title bar windows draws above the bars, so the title bar, the bars and an open menu read as one surface, as the menu bars of windows 10's own programs do, and elsewhere the content's own color, as a toolbar in Zed is. A button drawn again over itself as the top of an open menu keeps it, so it does not darken as it opens. A border between the bar and the content is the page's to draw, since only the page knows what is below */
.myBarButton { display: flex; align-items: center; gap: 4px; padding: 2px 8px; border: 1px solid transparent } /* flat at rest, and square, as everything in the bar is while it is built: a square edge is simpler and more exact to work against, and rounding corners is easy to add at the end. The list and the tree keep their rounded selections */
.myBarButton:hover { border-color: var(--color-zed-border) } /* and outlined under the pointer, the coolbar's hover from Internet Explorer 3, so the outline says where the pointer is */
.myBarButton[aria-pressed="true"] { border-color: var(--color-zed-border); background-color: var(--color-select-quiet) } /* pressed in, a toggle that is on: outlined as under the pointer, and filled in the quiet gray of a selected row, which is One Dark's own color for a selected element */
.myBarButton[data-big] { flex-direction: column; gap: 2px; padding: 4px 8px; min-width: calc(1rlh + 36px) } /* the drawing above its words, as Explorer's big buttons have it. As wide as its drawing or its words, whichever is wider, and never narrower than it is tall, which is the line the words are set on plus 36 css pixels, so a button with a short word like Up or Cut is a square, and a longer word widens its button past that */
.myBarIcon { display: flex }
.myBarIcon :deep(svg) { width: 24px; height: 24px; stroke-width: 1.5 } /* lucide's drawing, which the page holds as its own markup, beyond a scoped rule without deep: 24 css pixels square, its stroke 1.5 in the drawing's own 24 */
.myBarArrow { display: inline-block; width: 7px; height: 4px; margin-left: 4px; vertical-align: middle; background-color: currentColor; clip-path: polygon(0 0, 100% 0, 50% 100%) } /* the small arrow that says a button opens something, pointing down to where it opens; an odd width, so its point lands on a pixel's center */
.myBarLine { width: 1px; margin: 4px; background-color: var(--color-zed-border) } /* a separator between groups of buttons */

.myBarBackdrop { position: fixed; inset: 0; z-index: var(--bar-z) }
.myBar[data-open] .myBarButton[data-opens] { position: relative; z-index: calc(var(--bar-z) + 1) } /* the bar's buttons that open something, over the backdrop while one is open, so the pointer reaches them to run along the bar; its other buttons stay under it, where a press only closes what is open */
.myBarDrop { position: fixed; z-index: calc(var(--bar-z) + 2); user-select: none; color: var(--color-zed-text); filter: drop-shadow(2px 2px 2px var(--color-fuji-shade)) } /* the button drawn again and the card under it, at the button's place in the window. One shadow for both, cast by the outline of the two together, which a box-shadow on each could not do: the button's would fall across the card */
.myBarOpen, .myBarOpen:hover { position: absolute; z-index: 1; left: 0; top: 0; border-color: var(--color-gray-500); border-bottom: none; padding-bottom: 3px } /* outlined on three sides, in the card's line, and open along the bottom, where it covers the card's top line, raised over the card so no line runs between the title and its menu; a pixel more padding there for the border it does not have, so its words sit exactly where the button's do */
.myBarOpen[data-big] { padding-bottom: 5px }
.myBarCard { position: absolute; left: 0; width: max-content; padding: 1px; border: 1px solid var(--color-gray-500) } /* the card: a line in a gray strong enough to stand forward in light and dark, and a pixel of room inside it so an item's outline never touches it. Never narrower than the button above it, so the house never overhangs the skyscraper */
.myBarCard[data-kind="menu"] { background-image: linear-gradient(to right, var(--color-zed-elevated-surface-background) 26px, transparent 26px) } /* a menu's card, with a gutter down its left for the rows' drawings on the bar's own ground, as Office 2003 shades one, in Zed's own color for a menu: the card's pixel of room, a row's border and the 24 css pixels of its place for a drawing. A panel's card is the page's to lay out, and has none */
.myBarItem { display: flex; align-items: center; height: calc(1rlh + 8px); padding-right: 24px; border: 1px solid transparent; white-space: nowrap } /* one row: the line its words are set on plus 8 css pixels tall, its border among them */
.myBarItem:hover { border-color: var(--color-accent); background-color: color-mix(in srgb, var(--color-accent) 30%, var(--bar-ground)) } /* the row under the pointer, as Office 2003 lights one: outlined in the accent, and filled with the accent at 30 percent over the bar's ground, the same mix Office's pale blue is of its navy, with the words keeping their own color */
.myBarGutter { flex: none; width: 24px; margin-right: 8px; display: flex; justify-content: center } /* the row's place for a drawing, a column down the card's left, its drawing centered there, and the words 8 css pixels past it, so every row's words start in one line whether it has a drawing or not */
.myBarGutter .myBarIcon :deep(svg) { width: 16px; height: 16px } /* a drawing in a menu, 16 css pixels square, its stroke a css pixel */
.myBarCardLine { margin: 3px 0 3px 33px; border-top: 1px solid var(--color-zed-border) } /* a separator, starting where the words start, past the gutter, as Office's do */

</style>
