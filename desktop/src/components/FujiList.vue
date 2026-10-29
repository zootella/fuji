<script setup>//the list that list.js draws, as a component the rest of fuji can place: this layer hands it the rows and the filter and passes its answers up

import {ref, watch, onMounted} from 'vue'
import {listMake, listRows, listFilter, listKey} from '../list.js'
import {logTrouble} from '../log.js'

const props = defineProps({
	rows:   {type: Array,  default: () => []},//the text of each row
	filter: {type: String, default: ''},//matched anywhere in a row, ignoring case
})
const emit = defineEmits(['select'])//what each press or key did: how many rows are selected, how many changed, the kind of press or key, and the list's own milliseconds

const root = ref(null)//empty in the template, since list.js fills it and vue never draws inside
let list = null
onMounted(() => {
	list = listMake(root.value, {onSelect: press => emit('select', press), onTrouble: error => logTrouble('fuji list', error)})
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

<div ref="root" tabindex="0" class="myList mySans text-strong overflow-y-auto select-none"></div>

</template>
<style>

/* not scoped, because the rows are list.js's elements rather than this template's, and the window's state is an attribute the shell keeps on the root, which a scoped rule cannot reach. Every rule keys on attributes, data-active from list.js and data-window from the shell, and none on :focus, which WebKit stops matching in a window gone behind another without restyling the rows that depended on it */
.myRow { padding: 0 16px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis }
.myRow[data-selected="true"] { background-color: var(--color-select-quiet); color: var(--color-on-select-quiet) } /* a selection the keyboard is away from, in another control or another window, as a platform list shows one */
html:not([data-window="back"]) .myList[data-active="true"] .myRow[data-selected="true"] { background-color: var(--color-select); color: var(--color-on-select) }
.myList { outline: none } /* no ring around the whole list when it has the keyboard, since the row the keyboard is on shows that */
html:not([data-window="back"]) .myList[data-active="true"][data-keyboard="true"] .myRow[data-focus="true"] { outline: 1px dotted currentColor; outline-offset: -1px } /* the row the keyboard is on, once a key has been used, while the list has the keyboard and the window is in front: Windows' dotted focus rectangle, whose dots invert what is under them, here drawn in the row's own text color, which comes out the same, dark on a plain row and light on a selected one */

</style>
