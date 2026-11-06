<script setup>//a place on screen for a list that list.js made and the app keeps: the app makes it with listMake, hands it here, and calls list.js on it from then on

import {ref, onMounted} from 'vue'

const props = defineProps({list: {type: Object, required: true}})
const place = ref(null)//empty in the template, since the list brings its own element and vue never draws inside
onMounted(() => place.value.replaceChildren(props.list.element))

</script>
<template>

<div ref="place" class="flex flex-col mySans text-zed-text select-none"></div>

</template>
<style>

/* not scoped, because the rows are list.js's elements rather than this template's, and the window's state is an attribute the shell keeps on the root, which a scoped rule cannot reach. So every rule here reaches every element of its class anywhere in the window, and each class is one only the list uses: the contact sheet names its own cells myCell, and a bare rule on that name here once padded them and clipped their tiles. Every rule keys on attributes, data-active from list.js and data-window from the shell, and none on :focus, which WebKit stops matching in a window gone behind another without restyling the rows that depended on it */
.myListFrame { flex: 1; min-height: 0; display: flex; flex-direction: column; background-color: var(--color-zed-editor-background) } /* the list's own element, filling the place this component gives it, on a ground of its own as the tree is, so whatever the page puts behind it stays behind it */
.myHeader { flex: none; display: grid; grid-template-columns: var(--list-columns); grid-template-rows: var(--row-height); padding: 0 8px; border-bottom: 1px solid var(--color-zed-border); color: var(--color-zed-text-placeholder); overflow: hidden; scrollbar-gutter: stable } /* above the rows and outside what scrolls, so list.js moves it sideways with them; the gutter matches the one the rows reserve beside them, so where a platform draws a scrollbar, the right-aligned columns still line up */
.myHeader .myListCell { position: relative; overflow: visible; border-right: 1px solid var(--color-zed-border); display: flex; align-items: center } /* the divider at each column's right edge, the header's whole height, with the title centered up and down inside it as a row's text is */
.myHeader .myListCell[data-align="right"] { justify-content: flex-end }
.myHeader .myListCell::before, .myHeader .myListCell::after { white-space: pre } /* keeps the space between an arrow and its title, since in a flex cell the arrow is a box of its own, and a box drops the space it starts or ends with */
.myGrip { position: absolute; top: 0; bottom: 0; right: -4px; width: 8px; cursor: col-resize; z-index: 1 } /* the place to grab it, a few pixels either side of the line */
.myHeader .myListCell:not([data-align="right"])[data-sort="up"]::after { content: " ▲" } /* the arrow on the side away from the column's edge, after a title on the left and before one on the right */
.myHeader .myListCell:not([data-align="right"])[data-sort="down"]::after { content: " ▼" }
.myHeader .myListCell[data-align="right"][data-sort="up"]::before { content: "▲ " }
.myHeader .myListCell[data-align="right"][data-sort="down"]::before { content: "▼ " }
.myList { flex: 1; min-height: 0; overflow: auto; scrollbar-gutter: stable; isolation: isolate } /* down and, when the columns are wider than the list, sideways; a stacking context of its own, so the cut lines raised over the rows stay under whatever the page puts over the list, like a menu */
.myRow { position: relative; display: grid; grid-template-columns: var(--list-columns); grid-template-rows: var(--row-height); padding: 0 8px; width: max-content; min-width: 100% } /* every row on the one grid list.js sets on the frame, which the headers share, as tall as style.css says a row is, each cell the row's full height; as wide as its columns and never narrower than the list, so a selected row's fill, and the place a press selects it, reach past the last column to the list's edge, as Finder's do, on every platform. Positioned, so a cut line can be placed against a column of its grid */
.myListCell { padding: 0 8px; white-space: nowrap; overflow: hidden } /* the space between columns is each cell's own padding, inside the width list.js gives its column, so the dividers fall between them; text too long for its column is cut off at the edge, with no ellipsis */
.myRow .myListCell { display: flex; align-items: center; gap: 4px } /* the icon and the text side by side, each centered up and down in the row, 4 css pixels apart */
.myRow .myListCell[data-align="right"] { justify-content: flex-end } /* a right-aligned value wider than its column keeps its end against the edge and loses its start, so the column's right edge stays one straight line; text-align stops applying once text is wider than its box, and would start it at the left and cut it on the right */
.myCutLine { position: absolute; top: -6px; bottom: -6px; right: -2px; width: 4px; border-radius: 2px; z-index: 1; pointer-events: none; background-color: var(--color-zed-border) } /* the line list.js puts beside a cell whose text is wider than its column, in place of the ellipsis other lists draw, which takes width right where room is shortest and makes the reader wait for text that never comes: on the edge the text is cut at, beside each cut row rather than down the whole column, so it says which rows lost text; in Zed's border gray, lighter than the text, 4 css pixels wide with round ends and 6 past the row above and below, so it reads as an edge rather than an I-beam or a letter l, and a run of cut rows draws one long line. Centered on the edge, 2 css pixels into each column, where the neighbor keeps only padding, so a cut name and a cut size beside it draw the same line rather than two. Raised over the rows so the next row's selected fill never covers its end, and passing every press to the row beneath, since its ends lie over the rows beside its own */
.myCutLine[data-align="right"] { right: auto; left: -2px } /* on the left edge, which a right-aligned value loses its start at */
.myListCell[data-tabular="true"] { font-variant-numeric: tabular-nums } /* every digit one width, in a face that draws them proportional, so sizes and dates line up down the column; the letters keep the face's own widths */
.myListIcon { flex: none; width: 16px; height: 16px; background-color: var(--color-fuji-brand) } /* a placeholder for the icon, 16 css pixels square in fuji's cyan, so where the icons will go shows until they arrive */
.myRow[data-selected="true"] { background-color: var(--color-select-quiet); color: var(--color-on-select-quiet) } /* a selection the keyboard is away from, in another control or another window, as a platform list shows one */
html:not([data-window="back"]) .myList[data-active="true"] .myRow[data-selected="true"] { background-color: var(--color-accent); color: var(--color-on-accent) }
html[data-carry="true"], html[data-carry="true"] * { cursor: var(--list-carry, default) } /* while list.js carries a selection, the pointer everywhere in the window is the one the page that places the list set in --list-carry, and otherwise the plain arrow. On the root and every element under it, so the column dividers' own resize pointer gives way to it */
.myList { outline: none } /* no ring around the whole list when it has the keyboard, since the row the keyboard is on shows that */
html:not([data-window="back"]) .myList[data-active="true"][data-keyboard="true"] .myRow[data-focus="true"] { outline: 1px solid color-mix(in srgb, currentColor 50%, transparent); outline-offset: -1px } /* the row the keyboard is on, once a key has been used, while the list has the keyboard and the window is in front: Windows' focus rectangle, drawn solid in a share of the row's own text color, dark on a plain row and light on a selected one. Windows draws it dotted, but a page's one-pixel dots land between pixels at every other window width, since the engine spaces them to fit the row, and blur into a faint line, and an outline's width is rounded to a whole pixel, so no fraction of one moves them */

</style>
