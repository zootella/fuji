<script setup>//./.vitepress/theme/components/Download.vue
import { ref, computed, onMounted } from 'vue'
import { useData } from 'vitepress'
import { fetchSidecar, readableDate, saySize, copyText } from '../downloads.js'

/*
One download: the outlined box on the download page that offers a file and says what is known about it. downloads.js has the system it belongs to — where a sidecar comes from, what is in one, and why the fetch happens on mount.

The page places three of these by hand rather than looping over a list, because fuji ships three installers and will for a long time:

	<Download file="fuji.dmg" platform="macOS" system="Apple silicon" />

The attributes are what a sidecar cannot say. A sidecar knows the version, the architecture token, the byte count and the build date, because the machine that built the installer wrote them down; it does not know that aarch64 on a Mac is what everyone calls Apple silicon. So the words come from the page and the numbers come from the build.

The layout lives in the template rather than in strings assembled here, and the template asks one question: did a sidecar arrive. Inside that branch every field reads straight off it with no second guard, so moving a line, or moving a fact from one line to another, is moving markup and nothing else.

The copy message rides the end of the line below the hash rather than taking a line of its own: the home page learned that on a line of its own it changed the block's height as it appeared, moving the hash out from under the pointer, firing mouseleave, clearing the message and giving the height back — a flicker loop you could hold the mouse still inside.
*/

let props = defineProps({
	file: String,     //the published name, like fuji.dmg; its sidecar sits beside it under the same name
	platform: String, //macOS, Windows, Linux
	system: String,   //what it runs on, in the words somebody choosing a download would use
})

let { theme } = useData()//carries origin, the one place the site's own address is written

let sidecar = ref(false)//everything the build wrote down, or false until it arrives — and false for good if this file is not published
let loaded = ref(false)//the fetch has settled, so a missing sidecar now means unpublished rather than unread
let status = ref('')//the hover hint, then the copy confirmation
let fade = 0//the pending clear of a confirmation. every message cancels it first, so a fade already running can never wipe a fresher one

let address = computed(() => theme.value.origin + '/' + props.file)//the link's text and its target are the same string, which keeps the page honest in development, where a relative path would point at a file nothing is serving

onMounted(async () => {
	sidecar.value = await fetchSidecar(props.file)
	loaded.value = true
})

//the hash is clickable but not underlined, so the message beside it carries the affordance
function hintCopy() { clearTimeout(fade); status.value = 'Click to copy' }
function unhintCopy() { if (status.value == 'Click to copy') status.value = '' }//only the hint goes; a confirmation gets its full second however the pointer moves

//a confirmation that stood would be a lie the moment a second file was copied: there is one of these
//boxes per file, each with its own message, and two reading Copied do not say which one you took
async function copyHash() {
	clearTimeout(fade)
	status.value = await copyText(sidecar.value.sha256) ? 'Copied' : 'Could not copy'
	fade = setTimeout(() => { status.value = '' }, 1000)
}
</script>

<template>
<!-- a div per line: a p would bring the theme's 16px between each, and these lines belong tight together.
     ／ is U+FF0F, the fullwidth solidus the Meet Aki page divides a line with, not an ordinary slash -->
<div class="download">
	<div><a :href="address" download>{{ address }}</a></div>

	<template v-if="sidecar">
		<div><code
			tabindex="0"
			@click="copyHash" @keyup.enter="copyHash"
			@mouseenter="hintCopy" @mouseleave="unhintCopy"
		>{{ sidecar.sha256 }}</code></div>
		<div class="detail">{{ saySize(sidecar.bytes) }} ／ SHA-256 ／ {{ readableDate(sidecar.date) }} ／ Fuji {{ sidecar.version }}<span v-if="status"> ／ {{ status }}</span></div>
	</template>
	<div v-else-if="loaded" class="detail">Not yet published</div>

	<div class="detail">{{ platform }}: {{ system }}<span v-if="sidecar"> ({{ sidecar.arch }})</span></div>
</div>
</template>

<style scoped>
/* the outline is --vp-c-divider, exactly what the theme draws an hr and a heading rule with, and the
   radius is the 8px it gives a code block, its nearest neighbour on a documentation page */
.download {
	margin: 16px 0;
	padding: 12px 16px;
	border: 1px solid var(--vp-c-divider);
	border-radius: 8px;
}

.download code {
	cursor: pointer;
	overflow-wrap: anywhere;   /* sixty-four characters has to be free to break anywhere on a phone */
}

/* the face and size code is set in, without the highlighter chip it carries, because a byte count is a
   fact rather than a token to type. weight 500 for the reason style.css gives beside its own code rule:
   plex mono at this size reads lighter than it measures, and regular here would look thinner than the
   hash above it. scoped because one component wants it so far; a second one moves it to style.css */
.detail {
	font-family: var(--vp-font-family-mono);
	font-size: var(--vp-code-font-size);
	font-weight: 500;
	color: var(--vp-c-text-2);
}
</style>
