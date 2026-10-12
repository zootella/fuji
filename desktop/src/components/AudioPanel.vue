<script setup>//fuji audio: the view that plays sound and never a picture, from audio files and video files alike. For now it is where the media library is reached from, and it shows that the library is linked and answers

import {ref} from 'vue'
import {mpvVersion} from '../mpv.js'
import {logTrouble} from '../log.js'
import {brandName} from '../brand.js'

//the shell trades this with the sheet when either asks with a, and brings it from the settings panel the same way; which view is showing is the shell's, so this only asks. A view in the sheet's window rather than a window of its own, so the library and the builds can be worked on before fuji's windows are

const version = ref('')//what the library said, or blank until it has answered
const trouble = ref('')//or why it could not

function start() {//the shell calls this each time this view comes on screen
	mpvVersion()
		.then(v => { version.value = `${v.mpv}, client api ${v.api}, FFmpeg ${v.ffmpeg}`; trouble.value = '' })
		.catch(error => { trouble.value = String(error); logTrouble('audio: asking the library its version', error) })
}

const emit = defineEmits(['sheet'])//a, for the contact sheet back
function onKey(e) {
	if (e.key == 'a' && !e.ctrlKey && !e.metaKey) emit('sheet')
}

defineExpose({start, onKey})//the calls of the shell's this view has a use for

</script>
<template>

<div class="myMono bg-paper w-full h-full overflow-y-auto p-8">
	<h1 class="mb-8 text-strong">{{brandName}} Audio</h1>
	<p v-if="version">{{version}}</p><!-- the library names itself at the start of the line -->
	<p v-else-if="trouble" class="text-warn">{{trouble}}</p>
	<p v-else class="text-fainter italic">Asking the library…</p>
</div>

</template>
