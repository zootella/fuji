<script setup>//the list that list.js draws, as a component the rest of fuji can place: this layer hands it the rows and the filter and passes its answers up

import {ref, watch, onMounted} from 'vue'
import {listMake, listRows, listFilter} from '../list.js'
import {logTrouble} from '../log.js'

const props = defineProps({
	rows:   {type: Array,  default: () => []},//the text of each row
	filter: {type: String, default: ''},//matched anywhere in a row, ignoring case
})
const emit = defineEmits(['select'])//what each press on the list did: how many rows are selected, how many changed, the kind of press, and the list's own milliseconds

const root = ref(null)//empty in the template, since list.js fills it and vue never draws inside
let list = null
onMounted(() => {
	list = listMake(root.value, {onSelect: press => emit('select', press), onTrouble: error => logTrouble('fuji list', error)})
	listRows(list, props.rows)
	listFilter(list, props.filter)
})
watch(() => props.rows, rows => listRows(list, rows))
watch(() => props.filter, text => listFilter(list, text))
defineExpose({shown: () => list.shown})//the rows on screen, for the bench to count

</script>
<template>

<div ref="root" class="myList mySans text-strong overflow-y-auto select-none"></div>

</template>
<style scoped>

.myList :deep(.myRow) { padding: 0 16px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis }
.myList :deep(.myRow[data-selected="true"]) { background-color: var(--color-select); color: var(--color-on-select) }

</style>
