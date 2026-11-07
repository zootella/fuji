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
.myGuide[data-guide]::before { content: ""; left: 50%; width: var(--guide-width); translate: calc(var(--guide-width) / -2) 0; top: 0; bottom: 0 } /* down through the row, for a line running past and for a tee */
.myGuide[data-guide="elbow"]::before { bottom: calc(50% - var(--guide-width) / 2) } /* down to the bottom of the line across and no further, so the two cover the same square at the corner, as they do at a tee */
.myGuide[data-guide="first"]::before { top: calc(50% - var(--guide-width) / 2) } /* from the top of the line across down */
.myGuide[data-guide="alone"]::before { content: none }
.myGuide[data-guide]:not([data-guide="line"])::after { content: ""; left: 50%; right: 0; top: 50%; height: var(--guide-width); translate: calc(var(--guide-width) / -2) calc(var(--guide-width) / -2) } /* across from the line to the icon, at the row's own level, starting at the line down's left edge */

/* for now, the stroke experiments, each a block of its own turned on by an attribute the bench puts on the tree's place, so each can be seen alone or with the others */
[data-ends] .myGuide[data-guide]::before { top: 1px } /* Ends: each piece of a line down starts a css pixel below its row's top and runs to the bottom, so the line breaks once at each row, by a whole css pixel, where half a css pixel at either end would fall between pixels at 1x, and every end is round, so a line reads as strokes laid one after another */
[data-ends] .myGuide[data-guide="first"]::before { top: calc(50% - var(--guide-width) / 2) } /* the first root's starts at the corner, as it does without Ends */
[data-ends] .myGuide[data-guide]::after { right: 1px }
[data-ends] .myGuide::before, [data-ends] .myGuide::after { border-radius: calc(var(--guide-width) / 2) }
[data-caps] .myGuide::before, [data-caps] .myGuide::after { border-radius: calc(var(--guide-width) / 2) } /* Caps: every end round, as a pen with a round tip draws, which rounds the outside of a corner and leaves the inside sharp, since each box's inner corner sits under the other's */
[data-caps] .myGuide[data-guide]:not([data-start]):not([data-guide="first"])::before { border-top-left-radius: 0; border-top-right-radius: 0 } /* except where a line down runs on from the row above */
[data-caps] .myGuide[data-guide]:not([data-guide="elbow"])::before { border-bottom-left-radius: 0; border-bottom-right-radius: 0 } /* or on into the row below, so nothing breaks there */
.myTree { --pattern-size: 10px }
[data-pattern] .myGuide::before, [data-pattern] .myGuide::after { background-image: linear-gradient(45deg, var(--color-zed-text-placeholder) 25%, var(--color-zed-border) 25% 50%, var(--color-zed-text-placeholder) 50% 75%, var(--color-zed-border) 75%); background-size: var(--pattern-size) var(--pattern-size) } /* Pattern: the lines as cutouts onto stripes behind the tree, two of Zed's grays far enough apart to read as dashes, in a tile whose side has to go into the row's height evenly, so every cell's stripes join its neighbors'. Each line's background starts at its own box, so each kind is shifted back to its cell's corner */
[data-pattern] .myGuide::before { background-position: left calc(var(--guide-width) / 2 - var(--row-height) / 2) bottom 0 } /* a line down ends at its cell's bottom, whatever its top */
[data-pattern] .myGuide[data-guide="elbow"]::before { background-position: left calc(var(--guide-width) / 2 - var(--row-height) / 2) bottom calc(var(--guide-width) / 2 - var(--row-height) / 2) } /* except an elbow's, which ends at the line across */
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
