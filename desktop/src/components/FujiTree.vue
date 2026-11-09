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
.myTree { flex: 1; min-height: 0; overflow: auto; outline: none; background-color: var(--color-zed-editor-background); isolation: isolate } /* both ways, since a deep folder's name runs past the right edge; a stacking context of its own, so whatever is raised inside it stays under what the page puts over the tree, like a menu. The row's height from style.css is also each level's indent, so every guide cell is a square */
.myTreeRow { display: flex; height: var(--row-height); width: max-content; min-width: 100% }
.myGuide { position: relative; flex: none; width: var(--row-height) }
.myGuide::before, .myGuide::after { position: absolute; background-image: linear-gradient(45deg, var(--color-zed-border) 25%, transparent 25% 50%, var(--color-zed-border) 50% 75%, transparent 75%); background-size: 4px 4px } /* the lines, dashed in Zed's border gray like the other lines fuji draws beside them, 2 css pixels of gray and 2 of the tree's ground along each, the rhythm of the list's dashed focus rectangle. The dashes are cut from stripes at 45 degrees, which cross a line at their width times the square root of 2, in a tile whose side goes into the row's height evenly, so every cell's stripes join its neighbors' and the dashes run on from row to row and around corners. Each line is --guide-width wide, placed at the cell's middle and moved back by --guide-shift, half its width, so it is centered on the middle. Seen crisp on Windows 10 at 100 and 125 percent and on the Mac mini at 2x, 2026-11-06 */
.myTree { --guide-width: 1px; --guide-shift: calc(var(--guide-width) / -2) }
@media (resolution: 1dppx) { .myTree { --guide-shift: round(down, calc(var(--guide-width) / -2), 1px) } } /* except where a css pixel is one backing pixel, which has no half pixel to center a line of whole pixels on: there the shift is rounded to a whole pixel, so the line lies crisp on the pixels just left of the middle and just above it, where smoothed across two it would go soft. On a retina display half a css pixel is a whole backing pixel, so the line stays centered */
.myGuide[data-guide]::before { content: ""; left: 50%; width: var(--guide-width); translate: var(--guide-shift) 0; --guide-top: 0px; top: var(--guide-top); bottom: 0 } /* down through the row, for a line running past and for a tee; its top a property of its own, which its stripes read to line up */
.myGuide[data-guide="elbow"]::before { bottom: calc(50% - var(--guide-shift) - var(--guide-width)) } /* down to the bottom of the line across and no further, so the two cover the same square at the corner, as they do at a tee */
.myGuide[data-guide="first"]::before { --guide-top: calc(var(--row-height) / 2 + var(--guide-shift)) } /* from the top of the line across down */
.myGuide[data-guide="alone"]::before { content: none }
.myGuide[data-guide]:not([data-guide="line"])::after { content: ""; left: 50%; right: 0; top: 50%; height: var(--guide-width); translate: var(--guide-shift) var(--guide-shift) } /* across from the line to the icon, at the row's own level, starting at the line down's left edge */
.myGuide::before { background-position: left calc(0.25px - var(--row-height) / 2 - var(--guide-shift)) top calc(-1 * var(--guide-top)) } /* each line's stripes start at its own box, so each is shifted back to its cell's corner, a line down's from wherever its own top is, and a quarter css pixel on, so no edge between stripes runs through a pixel's center at 1x or 2x, where the engine would have to pick a color for it, as the list's focus rectangle does */
.myGuide::after { background-position: calc(0.25px - var(--row-height) / 2 - var(--guide-shift)) calc(-1 * var(--row-height) / 2 - var(--guide-shift)) }
.myTwistie { --twistie: 11px; position: absolute; z-index: 1; left: calc(50% + var(--guide-shift) + var(--guide-width) / 2 - var(--twistie) / 2); top: calc(50% + var(--guide-shift) + var(--guide-width) / 2 - var(--twistie) / 2); width: var(--twistie); height: var(--twistie); border-radius: 50%; background-color: var(--color-zed-editor-background) } /* the twistie, a circle where a folder's lines meet, shown only on a folder that holds folders, and nothing more: an empty ring once the folder is open and its folders are rows below it, and while it is closed, holding what it holds, the whole circle filled, solid in the gray the lines are dashed in. Centered where the lines' own center is, so on retina on the cell's middle and at 1x on the middle of the pixel the lines lie on; 11 css pixels across, an odd number, so it comes out even on either side of a line a pixel wide at 1x, and 22 backing pixels on retina, even on either side of the middle there. A disc of the tree's own ground, raised over the lines so where they meet is covered, with its ring drawn on it by the rule below, apart from the disc so the ring's masks leave the disc whole */
.myTwistie::before { content: ""; position: absolute; inset: 0; border: var(--guide-width) solid transparent; border-radius: 50%; background-color: var(--color-zed-border); background-origin: border-box; mask-image: linear-gradient(#000 0 0), linear-gradient(#000 0 0); mask-clip: padding-box, border-box; mask-composite: exclude } /* the ring, as thick as a line: its fill reaches under its transparent border, and a mask of the whole circle less its inside leaves only the border's width, so both edges are curves the engine draws itself */
.myTwistie:not([data-open])::before { mask-image: none } /* while the folder is closed, the ring's cutout taken away, so the whole circle is the ring's gray */
.myTreeItem { display: flex; align-items: center; gap: 4px; padding: 0 4px 0 2px; white-space: nowrap } /* the icon and the name, which a click selects and the highlight covers, as the classic Windows tree highlights a name rather than its row. 2 css pixels before the icon, as above and below it, so the icon sits on the same 20 pixel squares as the guide cells and the lines of the folders inside come down from its middle */
.myTreeIcon { flex: none; width: 16px; height: 16px; background-color: var(--color-fuji-brand) } /* the icon's square, as the list's is */
.myTreeItem[data-selected="true"] { background-color: var(--color-select-quiet); color: var(--color-on-select-quiet) } /* a selection the keyboard is away from, as the list shows one */
html:not([data-window="back"]) .myTree[data-active="true"] .myTreeItem[data-selected="true"] { background-color: var(--color-accent); color: var(--color-on-accent) }

</style>
