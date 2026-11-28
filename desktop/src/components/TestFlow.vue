<script setup>//the tiles of one bucket, flowing like words: flow.js in plastic decides each tile's kind and size and makes its pixels, and this places and draws them

import {reactive, onMounted, onBeforeUnmount} from 'vue'
import {flowMake, flowFill, flowClose, flowCanvas, flowRefuse, flowImgSize, flowDetails} from '../plastic/flow.js'
import {logTrouble} from '../brass/log.js'
import {xy, thinSpace} from '../plastic/library.js'
import {labelPut} from '../fuji-components/label.js'

const props = defineProps({
	files: {type: Array, required: true},//the listing's entries, in the pager's order, and never from two folders
})
const emit = defineEmits(['bytes', 'refused'])//what this bucket's canvases cost so far, sent up as each one is sized, and how many of its files could not be shown, sent up as each is refused, both for the bucket's caption

const flow = flowMake({files: props.files, watched: reactive, onBytes: bytes => emit('bytes', bytes), onRefused: refused => emit('refused', refused)})//reactive, so the template hears a tile change when the flow changes one

onMounted(() => { flowFill(flow).catch(error => logTrouble('TestFlow: filling a bucket', error)) })//the top gate for this bucket: anything that escapes a tile's work lands here, loudly
onBeforeUnmount(() => flowClose(flow))

function flowImgLoad(tile, event) { flowImgSize(flow, tile, xy(event.target.naturalWidth, event.target.naturalHeight)) }//the engine has loaded an img tile, and knows its picture's size

function flowLabel(said) {//a ref that puts a caption line's runs into its element as the flow renders, every space a thin one, since a caption is proportional text with little room under a narrow thumbnail and nothing in it lines up; said is worked out in the render itself rather than in here, so vue hears what it was read from, a tile's size arriving among them, and renders again when that changes
	let thin = said.map(run => typeof run == 'string' ? run.replaceAll(' ', thinSpace) : run)
	return el => { if (el) labelPut(el, thin) }
}

</script>
<template>

<!-- every tile names its path in data-path, on the cell that holds the picture and its caption, which is how the sheet knows which one was double-clicked without a handler on each. items-start so a short cell keeps its own height instead of stretching to the tallest in its row; the spacing is in myFlow, beside the outline it makes room for -->
<div class="myFlow flex flex-wrap items-start" :style="{'--beam': flow.beam + 'px'}">
	<template v-for="tile in flow.tiles" :key="tile.path"><div v-if="tile.kind != 'refused'" class="myCell flex flex-col" :data-path="tile.path" :style="tile.css ? {width: tile.css.x + 'px'} : null"><!-- a refused file has no cell at all, so the rows close over it; the caption above the bucket says how many. The cell takes the img's width as its own rather than from the img, because WebKit does not re-lay out a shrink-to-fit flex item when its img goes from no intrinsic size to an explicit one: an svg with a viewBox and no width or height lays out at 0 by 0 under the cap, the cell at 0 wide, and when the fit's size goes on the img it grows and the cell does not, so the drawing hangs over the next cell and the caption, as wide as the cell, is nothing. Measured 2026-10-06 on the Mac mini in a WKWebView with the flow's own structure: the cell stayed at 0 a frame later with the img sized, and laid out at once with its own width set. A canvas never meets this, since its size changes through its attributes, which WebKit treats as an intrinsic change -->
		<canvas v-if="tile.kind == 'native' || tile.kind == 'page'"
			:ref="el => flowCanvas(flow, tile.path, el)"
			class="myTile" width="0" height="0"
		></canvas>
		<img v-else-if="tile.kind == 'img' && tile.url" class="myTile" :class="{myImg: !tile.css}" :src="tile.url" :style="tile.css ? {width: tile.css.x + 'px', height: tile.css.y + 'px'} : {}" @load="flowImgLoad(tile, $event)" @error="flowRefuse(flow, tile, 'the engine could not show it')" />
		<div class="myCaption mySans"><!-- always the same lines, each its own text rather than one text wrapped, so every caption is the same height and each line is cut on its own -->
			<div class="myCaptionLine" :ref="flowLabel([tile.file.name])"></div>
			<div class="myCaptionLine" :ref="flowLabel(flowDetails(tile))"></div><!-- both lines runs, which label.js puts in as text and small spans; empty in the template, so vue leaves what it puts there alone -->
		</div>
	</div></template>
</div>

</template>
<style scoped>

.myFlow { /* 6 css pixels of paper between two tiles' outlines and between an outline and the bucket's edge, with room for the 2 each outline takes outside its tile; so two buckets' outlines meet 12 apart */
	gap: 10px; /* 6, and an outline on either side */
	padding: 8px; /* 6, and the one outline */
}
.myTile {
	display: block; /* a canvas and an img are inline by default, and their baselines would show as a stripe under every row */
	overflow: clip; /* the picture stays inside its own box, which the engines' stylesheets already say for a canvas and an img and which this says again where the tile is styled, so no rule elsewhere can let an svg paint past it; the outline and the shadow are the tile's own and drawn outside regardless */
	border-radius: 4px; /* a clip the compositor draws, so the pixels inside are blitted exactly as before; it only hides the few in each corner, and a tile shorter than 8px gets a smaller radius from css itself */
	outline: 2px solid var(--color-fuji-frame); /* an outline rather than a border, which would take its width from each side of the box the canvas is sized to and resample every thumbnail into what was left; an outline takes no room and follows the radius */
	outline-offset: 0; /* drawn just outside the edge, so every pixel of the picture shows; the line lands in the gap and the bucket's padding, which leaves 2 css pixels of paper between two neighbors' outlines and 4 between an outline and the bucket's edge */
	box-shadow: 4px 4px 12px var(--color-fuji-shade); /* one wide soft shadow, lit from the top left, so the right and bottom match and so do the left and top, since the outline already draws the crisp edge where the tile meets the sheet; the same shadow the table's card casts. Painted outside the box and taking no room, so the canvas is blitted exactly as before, and shaped by the radius. box-shadow rather than a drop-shadow filter, because filter on a tile is the gamma lens's, in Shell.vue */
}
.myCell:hover .myTile {
	outline-color: var(--color-fuji-brand); /* the tile under the pointer, or under it on its caption, in fuji's cyan, by its frame alone, so nothing about its size or its pixels changes */
}
.myCaption {
	width: 0; min-width: 100%; /* as wide as the tile above it and never wider: no width of its own to give the cell, then stretched to the one the tile gave it, so the fit alone still sizes every cell */
	padding-top: 6px; /* 4 css pixels of paper under the outline, which takes the first 2 */
	color: var(--color-text-placeholder);
	cursor: default; /* the arrow rather than the text beam, as a file manager shows over a file's name */
}
.myCaptionLine {
	min-height: 1lh; /* a line tall even with nothing in it, like the size of a picture not yet loaded or refused, so every caption keeps the same height */
	white-space: pre; overflow: hidden; /* one line, cut off at the tile's right edge where the tile is narrower than the words; pre rather than nowrap, which also never wraps but keeps every space, so a filename shows each of its spaces, thin as every space in a caption is */
	mask-image: linear-gradient(to right, black calc(100% - 2px), transparent); /* and melted over the last 2 css pixels, so a letter cut in half fades out rather than looking broken; on the line's box rather than the overflow, so a short line ending right there fades a little too, which is the price of doing it without script */
}
.myImg { /* an img tile until flowImgLoad has sized it, which is the moment between the url and the load */
	max-width: var(--beam); /* only ever shrinks: a square lands on the beam, a wide one hits the limit on width alone, and an image already smaller keeps its own size */
	max-height: var(--beam);
}

</style>
