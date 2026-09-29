<script setup>//the settings panel: what a user changes from inside fuji, shown in the contact sheet's window in place of the sheet

import {ref, computed, onMounted} from 'vue'
import {settings, settingsSet} from '../settings.js'
import {associateAnswers, associateOpens, associateActive, associateChoose, associateLook, associateOurs, associateDiffers, associateFinish} from '../associate.js'
import {logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {imageTypes, platform} from './library.js'

//the shell trades this with the sheet when either asks with s, and makes it fresh each time, so every box starts from the settings as they are now. A change reaches the settings object the moment the user commits it, and the file when fuji closes, as every setting does; settings.js checks it against the one schema, so nothing here repeats what a valid value is

const cardImages = ref(settings.card.images)//what the box says, which becomes the setting only once the user commits it
function cardImagesCommit() {//enter, or leaving the box
	if (!settingsSet('card', 'images', cardImages.value)) cardImages.value = settings.card.images//a value the setting turns away, like 0, 2.5 or nothing at all, puts the box back to the setting as it stands
}

//the file types: an answer per extension, what the system opens each with now, and where the two disagree. associate.js is the policy and carries the essay; this only shows it and hands the user's clicks down. Fuji asks the system only while this is showing, so it looks when the panel appears and again whenever the window comes back into focus, which is how it notices the user returning from windows' own settings
//deliberately plain for now: the whole stack under it works, and the interface over it is still to be designed

const fileGroups = {Pictures: Object.keys(imageTypes)}//every kind of file fuji opens, under the heading a person would look for it by; video and sound join as groups of their own
const answerLabels = {yes: 'Yes', no: 'No', ask: 'Ask'}
const anyDiffers = computed(() => Object.values(fileGroups).flat().some(associateDiffers))

const inactiveNote = {//why the answers here cannot be changed, on a copy that cannot act on them
	windows: `Only the copy of ${brandName} the installer put there acts on these, so this one shows them and changes nothing.`,
	mac: `Answering here comes to the Mac in a later version. Until then, choose ${brandName} for a kind of file with Get Info, Open with, and Change All.`,
	linux: `${brandName} cannot be handed a file on Linux yet, so there is nothing to answer here.`,
}[platform()]

function opensNow(extension) {//what the system would open this with, in words
	let opener = associateOpens.value[extension]
	if (!opener) return ''//not asked yet, or a platform with nothing to ask
	if (associateOurs(extension)) return brandName
	return opener.name || 'nothing'//a type nothing is registered for answers blank
}
function look() { associateLook().catch(error => logTrouble('settings: asking what opens each kind of file', error)) }
function choose(extension, answer) {
	if (associateAnswers.value[extension] == answer) return
	associateChoose(extension, answer).catch(error => logTrouble('settings: carrying out an answer', error))
}
function finish() { associateFinish().catch(error => logTrouble('settings: opening windows settings', error)) }

onMounted(look)
function onFocus(focused) { if (focused) look() }//the shell hands this view the window's focus events while it is showing

const emit = defineEmits(['sheet'])//s, for the contact sheet back; which view is showing is the shell's, so this only asks
function onKey(e) {
	if (e.key == 's' && !e.ctrlKey && !e.metaKey) emit('sheet')//a keystroke in the box never gets here, since the shell leaves a form field its own keys
}

defineExpose({onKey, onFocus})//the calls of the shell's this view has a use for

</script>
<template>

<div class="mySettings w-full h-full overflow-y-auto p-8">
	<h1 class="mb-8 text-white">Settings</h1>

	<label class="flex items-center gap-4">
		<span class="w-48">Images on a card</span>
		<input type="number" min="1" step="1" v-model.number="cardImages" @change="cardImagesCommit" @keydown.enter="$event.target.blur()" class="myBox w-24 px-2 py-1" />
	</label>
	<p class="mt-2 ml-52 max-w-xl text-neutral-500">For now the contact sheet shows one card, so this is also how many of a folder's pictures it shows, from the first in the current order.</p>

	<h2 class="mt-12 mb-2 text-white">File types</h2>
	<p class="max-w-2xl text-neutral-500">Which kinds of file {{brandName}} opens when you double-click one. {{brandName}} is offered for all of them whatever you answer. Yes makes it the program that opens that kind, No leaves it to whatever opens it now, and Ask means you have not decided.</p>
	<p v-if="!associateActive" class="mt-2 max-w-2xl text-amber-400">{{inactiveNote}}</p>

	<table class="mt-4">
		<template v-for="(extensions, group) in fileGroups" :key="group">
			<tr><td colspan="3" class="pt-2 pb-1 text-white">{{group}}</td></tr>
			<tr v-for="extension in extensions" :key="extension">
				<td class="w-24">{{extension}}</td>
				<td class="pr-8">
					<button v-for="(label, answer) in answerLabels" :key="answer" type="button" :disabled="!associateActive" :class="associateAnswers[extension] == answer ? 'myChosen' : 'myChoice'" class="px-3 mr-1" @click="choose(extension, answer)">{{label}}</button>
				</td>
				<td :class="associateDiffers(extension) ? 'text-amber-400' : ''">{{opensNow(extension)}}</td>
			</tr>
		</template>
	</table>
	<p v-if="anyDiffers" class="mt-4 max-w-2xl">
		<span class="text-amber-400">Where a row is amber, your answer and Windows disagree, and Windows has the last word.</span>
		<button type="button" class="myChoice px-3 ml-2" @click="finish">Change in Windows Settings</button>
	</p>

	<p class="mt-12 text-neutral-500">S goes back to the contact sheet</p>
</div>

</template>
<style scoped>

.mySettings {
	background-color: black; /* the sheet's, since this takes the sheet's place in the same window */
	color: #a3a3a3;
	font-family: monospace;
	font-size: 0.875rem;
	color-scheme: dark; /* so the number box's own parts, its spinner and caret, are drawn for a dark page */
}
.myBox {
	color: white;
	background-color: #171717;
	border: 1px solid #404040;
}
.myChoice, .myChosen {
	border: 1px solid #404040;
}
.myChosen {
	color: white;
	background-color: #404040;
}
.myChoice:disabled, .myChosen:disabled {
	opacity: 0.5;
}

</style>
