<script>//the register, in a normal script block so the sheet can check a flow name against it before handing one down

import TagFlow from './TagFlow.vue'
import CanvasFlow from './CanvasFlow.vue'

export const cardFlows = {//adding a flow is one entry here
	TagFlow,//plain img tags, everything left to the engine
	CanvasFlow,//fuji paints its own thumbnails and owns the memory
}

</script>
<script setup>//a box of thumbnails, all from one folder, and the flow that arranges them

//the card stays simple and the flow does the work: this takes a width from whatever contains it, hands its images to a flow, and takes back whatever height that flow needed. One flow governs every card at once, which is why the name arrives from the sheet rather than being read here. card.md says why the sheet scrolls over cards at all

const props = defineProps({
	paths: {type: Array,  required: true},//already in the model's order, and never from two folders
	flow:  {type: String, required: true},//the same one every other card was given
})

</script>
<template>

<!-- no border, no background: a card is a boundary fuji needs and the user is not meant to notice -->
<div>
	<component :is="cardFlows[props.flow]" :paths="props.paths" />
</div>

</template>
