<script setup>//./components/Sheet.vue - the contact sheet: one folder, seen whole

/*
A stub, and the shape is the point. Fuji has one sheet and several tables, and the shell hands a window event to whichever is showing, so every view answers the same small set of calls. This one answers them by doing nothing, which is what lets the switching be built and proven before there is anything to switch to.

When it grows a grid, note the rule it will meet immediately: v-show is display none, which destroys the layout box, so a hidden view measures its container as zero. Whatever decides how many thumbnails fit across has to run when this view becomes active, not when it is created.
*/

let started = false//start() is called every time this view becomes active, and one-time setup has to happen once
function start() {
	if (started) return
	started = true
}
function onKey(e)   {}//nothing to do with a key yet
function onResize() {}//and nothing to remeasure

defineExpose({start, onKey, onResize})//the same calls every view answers; onDrop and isFullscreen arrive when this view has a use for them

</script>
<template>

<div class="mySheet w-screen h-screen flex items-center justify-center select-none">contact sheet stub - press [c] to switch</div>

</template>
<style scoped>

.mySheet {
	background-color: black;
	color: #737373;
	font-family: monospace;
	font-size: 0.875rem;
}

</style>
