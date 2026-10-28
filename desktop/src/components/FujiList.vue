<script setup>//fuji's own list, in its plainest form: every row in the page, one column of text, a filter that runs on the text the list already holds, and a selection

import {computed, shallowRef, watch} from 'vue'
import {platform} from './library.js'
import {selectionEmpty, selectionClick} from '../selection.js'

const props = defineProps({
	rows:   {type: Array,  default: () => []},//the text of each row; a plain array from a shallowRef, so reading ten thousand of them goes through no proxy
	filter: {type: String, default: ''},//matched anywhere in a row, ignoring case
})
const emit = defineEmits(['select'])//how many rows are selected, after each press on the list

const lower = computed(() => props.rows.map(row => row.toLowerCase()))//once per list rather than once per keystroke
const shown = computed(() => {//the indexes of the rows that match, in order
	let f = props.filter.toLowerCase()
	let indexes = []
	for (let i = 0; i < lower.value.length; i++) if (!f || lower.value[i].includes(f)) indexes.push(i)
	return indexes
})
defineExpose({shown})

const traits = shallowRef(selectionEmpty())//replaced whole on every change, so one value is all the rows watch
watch(() => props.rows, () => { traits.value = selectionEmpty() })//new rows, so nothing in them is chosen yet

function onPointerDown(e) {//on the press rather than the click, as Finder and Explorer select; one listener for every row, which each row's index finds
	if (e.button != 0) return
	let toggle = platform() == 'mac' ? e.metaKey : e.ctrlKey//command on the mac, where control and a click is a right click
	let row = e.target.closest('[data-row]')
	if (!row) {
		if (e.offsetX >= e.currentTarget.clientWidth) return//the scrollbar, which a press on clears nothing
		if (!toggle && !e.shiftKey) traits.value = selectionEmpty()//a press below the last row clears, as it does in Finder and Explorer
	} else {
		traits.value = selectionClick(traits.value, Number(row.dataset.row), shown.value, {range: e.shiftKey, toggle})
	}
	emit('select', traits.value.selected.size)
}

</script>
<template>

<div class="mySans text-strong overflow-y-auto select-none" @pointerdown="onPointerDown">
	<div v-for="i in shown" :key="i" :data-row="i" class="px-4 whitespace-nowrap overflow-hidden text-ellipsis" :class="traits.selected.has(i) && 'bg-select text-on-select'">{{rows[i]}}</div>
</div>

</template>
