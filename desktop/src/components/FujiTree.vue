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
.myTree { flex: 1; min-height: 0; overflow: auto; outline: none; background-color: var(--color-zed-editor-background) } /* both ways, since a deep folder's name runs past the right edge. The row's height from style.css is also each level's indent, so every guide cell is a square, and being a multiple of 4 it is even, so every measure below taken from half of it lands on a whole pixel */
.myTreeRow { display: flex; height: var(--row-height); width: max-content; min-width: 100% }
.myGuide { position: relative; flex: none; width: var(--row-height) }
.myGuide::before, .myGuide::after { position: absolute; background-color: var(--color-zed-text-placeholder) } /* the lines, solid until they can be dotted */
.myGuide[data-guide]::before { content: ""; left: calc(var(--row-height) / 2 - 1px); width: 1px; top: 0; bottom: 0 } /* down through the row, for a line running past and for a tee */
.myGuide[data-guide="elbow"]::before { bottom: calc(var(--row-height) / 2) } /* down to the middle and no further */
.myGuide[data-guide="first"]::before { top: calc(var(--row-height) / 2 - 1px) } /* from the middle down */
.myGuide[data-guide="alone"]::before { content: none }
.myGuide[data-guide]:not([data-guide="line"])::after { content: ""; left: calc(var(--row-height) / 2 - 1px); right: 0; top: calc(var(--row-height) / 2 - 1px); height: 1px } /* across from the line to the icon, at the row's own level. The middle of an even square falls between two pixels, so the line down and the line across both take the pixel before it, and the twistie centers on that */
.myTwistie { position: absolute; z-index: 1; left: calc(var(--row-height) / 2 - 5px); top: calc(var(--row-height) / 2 - 5px); width: 9px; height: 9px; border: 1px solid var(--color-zed-text-placeholder); background-color: var(--color-zed-editor-background) } /* the classic box, centered on the line it covers, 9 css pixels so its middle is a whole pixel */
.myTwistie::before, .myTwistie::after { content: ""; position: absolute; background-color: var(--color-zed-text) }
.myTwistie::before { left: 1px; top: 3px; width: 5px; height: 1px } /* the minus, across the middle of the 7 pixels inside the border */
.myTwistie:not([data-open])::after { left: 3px; top: 1px; width: 1px; height: 5px } /* and the stroke that makes it a plus while the folder is closed */
.myTreeItem { display: flex; align-items: center; gap: 4px; padding: 0 4px; white-space: nowrap } /* the icon and the name, which a click selects and the highlight covers, as the classic Windows tree highlights a name rather than its row */
.myTreeIcon { color: var(--color-zed-text-muted) }
.myTreeItem[data-selected="true"] { background-color: var(--color-select-quiet); color: var(--color-on-select-quiet) } /* a selection the keyboard is away from, as the list shows one */
.myTreeItem[data-selected="true"] .myTreeIcon { color: inherit }
html:not([data-window="back"]) .myTree[data-active="true"] .myTreeItem[data-selected="true"] { background-color: var(--color-accent); color: var(--color-on-accent) }

</style>
