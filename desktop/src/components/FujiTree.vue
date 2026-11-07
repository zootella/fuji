<script setup>//a place on screen for a tree that tree.js made and the app keeps, as FujiList.vue is for a list

import {ref, onMounted} from 'vue'

const props = defineProps({tree: {type: Object, required: true}})
const place = ref(null)//empty in the template, since the tree brings its own element and vue never draws inside
onMounted(() => place.value.replaceChildren(props.tree.element))

</script>
<template>

<div ref="place" class="flex flex-col mySans text-zed-text select-none"></div>

</template>
<style>

/* not scoped, for the reasons FujiList.vue gives: the rows are tree.js's elements rather than this template's, and the window's state is an attribute the shell keeps on the root */
.myTree { flex: 1; min-height: 0; overflow: auto; outline: none; background-color: var(--color-zed-editor-background) } /* both ways, since a deep folder's name runs past the right edge. The row's height from style.css is also each level's indent, so every guide cell is a square */
.myTreeRow { display: flex; height: var(--row-height); width: max-content; min-width: 100% }
.myGuide { position: relative; flex: none; width: var(--row-height) }
.myGuide::before, .myGuide::after { position: absolute; background-color: var(--color-zed-text-placeholder) } /* the lines, --guide-width wide and centered on the cell's middle by a translate rather than by where the box is placed, so that a half pixel can be drawn as one, smoothed across the two pixels it falls between, rather than snapped to a whole one */
.myTree { --guide-width: 1.5px }
.myGuide[data-guide]::before { content: ""; left: 50%; width: var(--guide-width); translate: calc(var(--guide-width) / -2) 0; --guide-top: 0px; top: var(--guide-top); bottom: 0 } /* down through the row, for a line running past and for a tee; its top a property of its own, which Pattern reads to line its stripes up */
.myGuide[data-guide="elbow"]::before { bottom: calc(50% - var(--guide-width) / 2) } /* down to the bottom of the line across and no further, so the two cover the same square at the corner, as they do at a tee */
.myGuide[data-guide="first"]::before { --guide-top: calc(var(--row-height) / 2 - var(--guide-width) / 2) } /* from the top of the line across down */
.myGuide[data-guide="alone"]::before { content: none }
.myGuide[data-guide]:not([data-guide="line"])::after { content: ""; left: 50%; right: 0; top: 50%; height: var(--guide-width); translate: calc(var(--guide-width) / -2) calc(var(--guide-width) / -2) } /* across from the line to the icon, at the row's own level, starting at the line down's left edge */

/* for now, the stroke experiments, each a block of its own turned on by an attribute the bench puts on the tree's place, so each can be seen alone or with the others */
.myTree { --guide-reach: 9px }
[data-ends] .myGuide[data-guide]:not([data-guide="first"])::before { --guide-top: calc(var(--row-height) / 2 - var(--guide-reach)) } /* Ends: every arm of a cell, up, down and across, reaches --guide-reach from the cell's middle and stops, so the arms are one length and the lines break between rows and before the icon, where without Ends they run to the cell's edges and touch. Up, except from the first root, which starts at its corner */
[data-ends] .myGuide[data-guide]:not([data-guide="elbow"])::before { bottom: calc(50% - var(--guide-reach)) } /* down, except from an elbow, which turns at its corner */
[data-ends] .myGuide[data-guide]::after { right: calc(50% - var(--guide-reach) - var(--guide-width) / 2) } /* and across, its end measured from the middle as the others' are, past the translate that moves the line back by half its width */
[data-caps] .myGuide::before, [data-caps] .myGuide::after { border-radius: calc(var(--guide-width) / 2) } /* Caps: every corner of every piece round, as a pen with a round tip draws, whatever Ends says; with Ends off, where pieces meet at a row's edge, the rounding pinches the line there */
.myTree { --pattern-size: 10px }
[data-pattern] .myGuide::before, [data-pattern] .myGuide::after { background-image: linear-gradient(45deg, var(--color-zed-text-placeholder) 25%, var(--color-zed-border) 25% 50%, var(--color-zed-text-placeholder) 50% 75%, var(--color-zed-border) 75%); background-size: var(--pattern-size) var(--pattern-size) } /* Pattern: the lines as cutouts onto stripes behind the tree, two of Zed's grays far enough apart to read as dashes, in a tile whose side has to go into the row's height evenly, so every cell's stripes join its neighbors'. Each line's background starts at its own box, so each kind is shifted back to its cell's corner */
[data-pattern] .myGuide::before { background-position: left calc(var(--guide-width) / 2 - var(--row-height) / 2) top calc(-1 * var(--guide-top)) } /* a line down's stripes shifted back to its cell's corner from wherever its own top is */
[data-pattern] .myGuide::after { background-position: calc(var(--guide-width) / 2 - var(--row-height) / 2) calc(var(--guide-width) / 2 - var(--row-height) / 2) }
[data-magnified] .myTree { zoom: 8 } /* the whole tree magnified, rows, lines, icons and text alike, so the experiments are easier to see and to measure */
.myTwistie { position: absolute; z-index: 1; left: calc(var(--row-height) / 2 - 5px); top: calc(var(--row-height) / 2 - 5px); width: 9px; height: 9px; border: 1px solid var(--color-zed-text-placeholder); background-color: var(--color-zed-editor-background) } /* the classic box, centered on the line it covers, 9 css pixels so its middle is a whole pixel */
.myTwistie::before, .myTwistie::after { content: ""; position: absolute; background-color: var(--color-zed-text) }
.myTwistie::before { left: 1px; top: 3px; width: 5px; height: 1px } /* the minus, across the middle of the 7 pixels inside the border */
.myTwistie:not([data-open])::after { left: 3px; top: 1px; width: 1px; height: 5px } /* and the stroke that makes it a plus while the folder is closed */
.myTwistie { display: none } /* for now, no box, so the lines are the whole show; a press anywhere in the cell still opens and closes the folder */
.myTreeItem { display: flex; align-items: center; gap: 4px; padding: 0 4px 0 2px; white-space: nowrap } /* the icon and the name, which a click selects and the highlight covers, as the classic Windows tree highlights a name rather than its row. 2 css pixels before the icon, as above and below it, so the icon sits on the same 20 pixel squares as the guide cells and the lines of the folders inside come down from its middle */
.myTreeIcon { flex: none; width: 16px; height: 16px; background-color: var(--color-fuji-brand) } /* a placeholder for the icon, the list's cyan square */
.myTreeItem[data-selected="true"] { background-color: var(--color-select-quiet); color: var(--color-on-select-quiet) } /* a selection the keyboard is away from, as the list shows one */
html:not([data-window="back"]) .myTree[data-active="true"] .myTreeItem[data-selected="true"] { background-color: var(--color-accent); color: var(--color-on-accent) }

</style>
