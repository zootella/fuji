<script setup>//a box of thumbnails, all from one folder, arranged by the flow

import SquareFlow from './SquareFlow.vue'

//the card stays simple and the flow does the work: this takes a width from whatever contains it, hands its images to the flow, and takes back whatever height the flow needed. The box exists so the sheet has something it can count — a card totals what its canvases cost, and a limit on cards is what turns that into a real ceiling on the sheet
//there is one flow, so this names it directly; a register and a prop come back when there is a second one to choose between

const props = defineProps({
	paths: {type: Array, required: true},//already in the model's order, and never from two folders
})

</script>
<template>

<!-- no border, no background: a card is a boundary fuji needs and the user is not meant to notice -->
<div>
	<SquareFlow :paths="props.paths" />
</div>

</template>
