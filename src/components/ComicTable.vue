<script setup>//./components/ComicTable.vue - one image full width, read down a vertical scroll

/*
A stub, here so that DiamondTable has a sister in the repository and the shape of a second table is visible before one is written. Nothing switches to it yet.

What will make it a different table is what it keeps: DiamondTable keeps an invisible diamond and sizes every image into it, so pan and zoom act on that. This one would keep a width, run the image down an ordinary scroll, and have no pan at all. Both show one image out of the same folder in the same order, because that lives below them in the model rather than in either.
*/

let started = false//start() is called every time this view becomes active, and one-time setup has to happen once
function start() {
	if (started) return
	started = true
}
function onKey(e)   {}
function onResize() {}
function isFullscreen() { return false }//no way to enter fullscreen from here yet

defineExpose({start, onKey, onResize, isFullscreen})

</script>
<template>

<div class="myComic w-screen h-screen flex items-center justify-center select-none">comic table stub</div>

</template>
<style scoped>

.myComic {
	background-color: #171717;
	color: #737373;
	font-family: monospace;
	font-size: 0.875rem;
}

</style>
