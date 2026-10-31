<script setup>//the list that list.js draws, as a component the rest of fuji can place: this layer hands it the rows and the filter and passes its answers up

import {ref, watch, onMounted} from 'vue'
import {listMake, listRows, listFilter, listKey} from '../list.js'
import {logTrouble} from '../log.js'

const props = defineProps({
	columns: {type: Array,  required: true},//what each row shows, left to right: a key into the row, the title its header says, a say function that makes its value text, a sort function that takes rows and hands them back in order, which a column of raw numbers leaves out, an align, and a width in css pixels; read once as the list is made
	rows:    {type: Array,  default: () => []},//a record per row, holding a value for each column's key
	filter:  {type: String, default: ''},//matched anywhere in the first column, ignoring case
})
const emit = defineEmits(['select', 'sort'])//what each press, key or drag did: how many rows are selected, how many changed, the kind of press or key, and the list's own milliseconds; and each sort, by which column and which way

const frame = ref(null)//empty in the template, since list.js fills it and vue never draws inside
let list = null
onMounted(() => {
	list = listMake(frame.value, {columns: props.columns, onSelect: press => emit('select', press), onSort: sort => emit('sort', sort), onTrouble: error => logTrouble('fuji list', error)})
	listRows(list, props.rows)
	listFilter(list, props.filter)
})
watch(() => props.rows, rows => listRows(list, rows))
watch(() => props.filter, text => listFilter(list, text))
defineExpose({
	shown: () => list.shown,//the rows on screen, for the bench to count
	key: e => listKey(list, e),//a key from the view, which hands every key it is given here
})

</script>
<template>

<div ref="frame" class="myListFrame mySans text-zed-text select-none"></div>

</template>
<style>

/* not scoped, because the rows are list.js's elements rather than this template's, and the window's state is an attribute the shell keeps on the root, which a scoped rule cannot reach. So every rule here reaches every element of its class anywhere in the window, and each class is one only the list uses: the contact sheet names its own cells myCell, and a bare rule on that name here once padded them and clipped their tiles. Every rule keys on attributes, data-active from list.js and data-window from the shell, and none on :focus, which WebKit stops matching in a window gone behind another without restyling the rows that depended on it */
.myListFrame { display: flex; flex-direction: column }
.myHeader { flex: none; display: grid; grid-template-columns: var(--list-columns); padding: 0 8px; border-bottom: 1px solid var(--color-zed-border); color: var(--color-zed-text-placeholder); overflow: hidden; scrollbar-gutter: stable } /* above the rows and outside what scrolls, so list.js moves it sideways with them; the gutter matches the one the rows reserve beside them, so where a platform draws a scrollbar, the right-aligned columns still line up */
.myHeader .myListCell { position: relative; overflow: visible; border-right: 1px solid var(--color-zed-border) } /* the divider at each column's right edge */
.myGrip { position: absolute; top: 0; bottom: 0; right: -4px; width: 8px; cursor: col-resize; z-index: 1 } /* the place to grab it, a few pixels either side of the line */
.myHeader .myListCell:not([data-align="right"])[data-sort="up"]::after { content: " ▲" } /* the arrow on the side away from the column's edge, after a title on the left and before one on the right */
.myHeader .myListCell:not([data-align="right"])[data-sort="down"]::after { content: " ▼" }
.myHeader .myListCell[data-align="right"][data-sort="up"]::before { content: "▲ " }
.myHeader .myListCell[data-align="right"][data-sort="down"]::before { content: "▼ " }
.myList { flex: 1; min-height: 0; overflow: auto; scrollbar-gutter: stable } /* down and, when the columns are wider than the list, sideways */
.myRow { display: grid; grid-template-columns: var(--list-columns); padding: 0 8px; width: max-content; min-width: 100% } /* every row on the one grid list.js sets on the frame, which the headers share; as wide as its columns and never narrower than the list, so a selected row's fill, and the place a press selects it, reach past the last column to the list's edge, as Finder's do, on every platform */
.myListCell { padding: 0 8px; white-space: nowrap; overflow: hidden } /* the space between columns is each cell's own padding, inside the width list.js gives its column, so the dividers fall between them; text too long for its column is cut off at the edge, with no ellipsis */
.myListCell[data-align="right"] { text-align: right }
.myRow .myListCell { mask-image: linear-gradient(to right, black calc(100% - 2px), transparent) } /* melted over its last 2 css pixels, as the contact sheet melts its captions, so a letter cut in half fades out rather than looking broken, in less room than an ellipsis and with nothing to stop the eye; the cell's own padding keeps text that fits clear of the fade, and the headers go without it, since it would clip their dividers and arrows */
.myRow .myListCell[data-align="right"] { display: flex; justify-content: flex-end; mask-image: linear-gradient(to left, black calc(100% - 2px), transparent) } /* a right-aligned value wider than its column keeps its end against the edge and loses its start, melted on the left, so the column's right edge stays one straight line; text-align stops applying once text is wider than its box, and would start it at the left and cut it on the right */
.myRow[data-selected="true"] { background-color: var(--color-select-quiet); color: var(--color-on-select-quiet) } /* a selection the keyboard is away from, in another control or another window, as a platform list shows one */
html:not([data-window="back"]) .myList[data-active="true"] .myRow[data-selected="true"] { background-color: var(--color-accent); color: var(--color-on-accent) }
.myList { outline: none } /* no ring around the whole list when it has the keyboard, since the row the keyboard is on shows that */
html:not([data-window="back"]) .myList[data-active="true"][data-keyboard="true"] .myRow[data-focus="true"] { outline: 1px solid color-mix(in srgb, currentColor 50%, transparent); outline-offset: -1px } /* the row the keyboard is on, once a key has been used, while the list has the keyboard and the window is in front: Windows' focus rectangle, drawn solid in a share of the row's own text color, dark on a plain row and light on a selected one. Windows draws it dotted, but a page's one-pixel dots land between pixels at every other window width, since the engine spaces them to fit the row, and blur into a faint line, and an outline's width is rounded to a whole pixel, so no fraction of one moves them */

</style>
