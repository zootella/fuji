<script setup>//a bucket of thumbnails, all from one folder, arranged by the flow

import {ref, computed} from 'vue'
import TestFlow from './TestFlow.vue'
import {backize, middleDot, saySize4} from './library.js'

//the bucket stays simple and the flow does the work: this takes a width from whatever contains it, hands its images to the flow, and takes back whatever height the flow needed. The bucket exists so the sheet has something it can count — a bucket totals what its canvases cost, and a limit on buckets is what turns that into a real ceiling on the sheet. Named bucket rather than card, which is the table's word for the box around its picture
//there is one flow, so this names it directly; a register and a prop come back when there is a second one to choose between

const props = defineProps({
	folder: {type: String, required: true},//the one folder every file here is in
	files: {type: Array, required: true},//the listing's entries for the images this bucket shows, in the walk's order, each with its path, name, size and modified time
	first: {type: Number, required: true},//where this bucket starts in the folder, counting from 1
	total: {type: Number, required: true},//how many images the folder holds, shown or not
})
const bucketBytes = ref(0)//what the flow's canvases cost so far, in bytes, which it reports as each one is sized; exact, since a canvas is the bytes fuji asked for, and the imgs are the engine's and not counted
const bucketRefused = ref(0)//how many of the bucket's files could not be shown, which the flow reports as each one is refused; they keep their places in the range below, since the numbers are the listing's, and the caption says how many are missing
const emit = defineEmits(['bytes'])//the bytes, passed up for the sheet's total beneath every bucket
function bucketBytesSet(bytes) { bucketBytes.value = bytes; emit('bytes', bytes) }//kept for the caption and passed up

const bucketCaption = computed(() => {//the folder the bucket's images are in, which of the folder's images they are, how many of those could not be shown, and what its thumbnails cost, like /Users/name/Pictures · Images 1 through 20 of 35 (2 did not load) · 11 MB
	let folder = backize(props.folder)//written as the platform's file manager writes it
	let last = props.first + props.files.length - 1
	let range = `Images ${props.first} through ${last} of ${props.total}`
	if (props.first == last) range = `Image ${last} of ${props.total}`//a bucket holding one image, which the plural would make read oddly
	if (bucketRefused.value > 0) range += ` (${bucketRefused.value} did not load)`//so a bucket showing fewer thumbnails than its range, or none, says why
	return `${folder} ${middleDot} ${range} ${middleDot} ${saySize4(bucketBytes.value)}`//the cost grows as the bucket fills, and stays at 0 bytes for a bucket of imgs alone
})

</script>
<template>

<!-- no border, no background: a bucket is a boundary fuji needs and the user is not meant to notice -->
<div>
	<div class="myBucketCaption mySans">{{bucketCaption}}</div>
	<TestFlow :files="props.files" @bytes="bucketBytesSet" @refused="bucketRefused = $event" />
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

</style>
