<script setup>//a bucket of thumbnails, all from one folder, arranged by the flow

import {computed} from 'vue'
import parse from 'path-browserify'
import TestFlow from './TestFlow.vue'
import {backize, middleDot} from './library.js'

//the bucket stays simple and the flow does the work: this takes a width from whatever contains it, hands its images to the flow, and takes back whatever height the flow needed. The bucket exists so the sheet has something it can count — a bucket totals what its canvases cost, and a limit on buckets is what turns that into a real ceiling on the sheet. Named bucket rather than card, which is the table's word for the box around its picture
//there is one flow, so this names it directly; a register and a prop come back when there is a second one to choose between

const props = defineProps({
	paths: {type: Array, required: true},//already in the model's order, and never from two folders
	first: {type: Number, required: true},//where this bucket starts in the folder, counting from 1
	total: {type: Number, required: true},//how many images the folder holds, shown or not
})

const bucketCaption = computed(() => {//the folder the bucket's images are in, and which of the folder's images they are, like /Users/name/Pictures · Images 1 through 20 of 35
	let folder = backize(parse.dirname(props.paths[0]))//one folder per bucket, so the first image's says it for all; written as the platform's file manager writes it
	let last = props.first + props.paths.length - 1
	let range = `Images ${props.first} through ${last} of ${props.total}`
	if (props.first == last) range = `Image ${last} of ${props.total}`//a bucket holding one image, which the plural would make read oddly
	return `${folder} ${middleDot} ${range}`
})

</script>
<template>

<!-- no border, no background: a bucket is a boundary fuji needs and the user is not meant to notice -->
<div>
	<div class="myBucketCaption mySans"><span class="myBucketCaptionText">{{bucketCaption}}</span></div>
	<TestFlow :paths="props.paths" />
</div>

</template>
<style scoped>

.myBucketCaption {
	padding: 8px 8px 0; /* the flow's own padding, so the text ends at the same right edge the bucket's widest row can reach, and the flow's top padding puts 6 css pixels of paper between it and the first row's outlines */
	text-align: right; /* in the bucket's upper right, apart from the pictures' own captions, which start at their left edges */
	white-space: pre; overflow: hidden; /* one line, cut off at the bucket's right edge, as a thumbnail's caption is at its picture's */
	color: var(--color-quiet);
	cursor: default;
}
.myBucketCaptionText {
	background-color: orange; /* the text alone, shown while its place in the layout is tried */
}

</style>
