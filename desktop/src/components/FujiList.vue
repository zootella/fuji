<script setup>//fuji's own list, in its plainest form: every row in the page, one column of text, and a filter that runs on the text the list already holds

import {computed} from 'vue'

const props = defineProps({
	rows:   {type: Array,  default: () => []},//the text of each row; a plain array from a shallowRef, so reading ten thousand of them goes through no proxy
	filter: {type: String, default: ''},//matched anywhere in a row, ignoring case
})

const lower = computed(() => props.rows.map(row => row.toLowerCase()))//once per list rather than once per keystroke
const shown = computed(() => {//the indexes of the rows that match, in order
	let f = props.filter.toLowerCase()
	let indexes = []
	for (let i = 0; i < lower.value.length; i++) if (!f || lower.value[i].includes(f)) indexes.push(i)
	return indexes
})
defineExpose({shown})

</script>
<template>

<div class="mySans text-strong overflow-y-auto">
	<div v-for="i in shown" :key="i" class="px-4 whitespace-nowrap overflow-hidden text-ellipsis">{{rows[i]}}</div>
</div>

</template>
