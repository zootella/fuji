<script setup>//the settings panel: what a user changes from inside fuji, shown in the contact sheet's window in place of the sheet

import {ref} from 'vue'
import {settings, settingsSet} from '../settings.js'
import {brandName, brandStem} from '../brand.js'
import {platform} from './library.js'
import SettingsFileTypes from './SettingsFileTypes.vue'

//the shell trades this with the sheet when either asks with s, and makes it fresh each time, so every box starts from the settings as they are now. A change reaches the settings object the moment the user commits it, and the file when fuji closes, as every setting does; settings.js checks it against the one schema, so nothing here repeats what a valid value is. A section with a life of its own, like the file types, is a component of its own, and a setting that is one box stays here

const cardImages = ref(settings.card.images)//what the box says, which becomes the setting only once the user commits it
function cardImagesCommit() {//enter, or leaving the box
	if (!settingsSet('card', 'images', cardImages.value)) cardImages.value = settings.card.images//a value the setting turns away, like 0, 2.5 or nothing at all, puts the box back to the setting as it stands
}
const sheetCards = ref(settings.sheet.cards)//the same, for how many cards the sheet holds
function sheetCardsCommit() {
	if (!settingsSet('sheet', 'cards', sheetCards.value)) sheetCards.value = settings.sheet.cards
}

const faces = ref(settings.font.faces)//which fonts, the ones fuji carries or the system's; the buttons offer only what the setting takes, so it never needs putting back
const systemFaces = {windows: ['Segoe UI', 'Consolas'], mac: ['San Francisco', 'SF Mono']}[platform()]//what the system's fonts are, proportional and fixed-width, named where every computer of that kind has the same ones; a linux desktop chooses its own, so there the choice names none
function facesCommit() {
	if (settingsSet('font', 'faces', faces.value)) emit('faces')//the root is the shell's, so it puts the new faces there
}

const mode = ref(settings.appearance.mode)//light, dark, or the system's, whichever that is
function modeCommit() {
	if (settingsSet('appearance', 'mode', mode.value)) emit('theme')//the window is the shell's, so it sets the theme on it
}

const fileTypes = ref(null)//the file types section, which asks the system again when told
function onFocus(focused) { if (focused) fileTypes.value?.look() }//the shell hands this view the window's focus events while it is showing; coming back from the system's own settings is when a default is most likely to have changed

const emit = defineEmits(['sheet', 'faces', 'theme'])//s, for the contact sheet back; which view is showing is the shell's, so this only asks. And faces and theme, when the fonts or the appearance change
function onKey(e) {
	if (e.key == 's' && !e.ctrlKey && !e.metaKey) emit('sheet')//a keystroke in the box never gets here, since the shell leaves a form field its own keys
}

defineExpose({onKey, onFocus})//the calls of the shell's this view has a use for

</script>
<template>

<div class="mySettings myMono w-full h-full overflow-y-auto p-8">
	<h1 class="mb-8 text-strong">Settings</h1>

	<label class="flex items-center gap-4">
		<span class="w-48">Images on a card</span>
		<input type="number" min="1" step="1" v-model.number="cardImages" @change="cardImagesCommit" @keydown.enter="$event.target.blur()" class="myBox w-24 px-2 py-1" />
	</label>
	<label class="mt-1 flex items-center gap-4">
		<span class="w-48">Cards on the sheet</span>
		<input type="number" min="1" step="1" v-model.number="sheetCards" @change="sheetCardsCommit" @keydown.enter="$event.target.blur()" class="myBox w-24 px-2 py-1" />
	</label>
	<!-- no hint beneath: for now the two together are also how many of a folder's pictures the sheet shows, which is scaffolding to know rather than something to tell a user -->

	<div class="mt-4 flex gap-4">
		<span class="w-48">Typography</span>
		<div role="radiogroup" aria-label="Typography"><!-- radio buttons rather than a list, so every choice and what it gives are in view without a click; the panel scrolls when it grows -->
			<label class="flex items-center gap-2"><input type="radio" name="faces" value="system" v-model="faces" @change="facesCommit" /><span>System fonts<template v-if="systemFaces">: <i>{{systemFaces[0]}}</i>, with <i>{{systemFaces[1]}}</i></template></span></label><!-- the words in one span, so the flex row holds the button and them as two items, rather than putting its gap around every name -->
			<label class="mt-1 flex items-center gap-2"><input type="radio" name="faces" :value="brandStem" v-model="faces" @change="facesCommit" /><span>{{brandName}} fonts: <i>Inter</i>, with <i>IBM Plex Mono</i></span></label>
		</div>
	</div>

	<div class="mt-4 flex gap-4">
		<span class="w-48">Appearance</span>
		<div role="radiogroup" aria-label="Appearance">
			<label class="flex items-center gap-2"><input type="radio" name="mode" value="light" v-model="mode" @change="modeCommit" />Light</label>
			<label class="mt-1 flex items-center gap-2"><input type="radio" name="mode" value="dark" v-model="mode" @change="modeCommit" />Dark</label>
			<label class="mt-1 flex items-center gap-2"><input type="radio" name="mode" value="system" v-model="mode" @change="modeCommit" />System</label>
		</div>
	</div>

	<SettingsFileTypes ref="fileTypes" class="mt-12" />

	<p class="mt-12 text-faint">Press S to return to the contact sheet</p>

	<!-- a sample of the root's text, apart from the panel's fixed-width type, so the fonts choice above can be seen changing it: first a line whose letters give a typeface away, the pangram for every shape, AVATAR and Wavy Tofu for the spacing between pairs, QGRSJ for the letters faces differ on most, and Il1| O0 rn m for the ones easiest to confuse; then the words of File Explorer's ribbon, to set beside it on Windows. Here until fuji has text of its own in that face -->
	<div class="mySans mt-12">
		<p>Sphinx of black quartz, judge my vow. AVATAR Wavy Tofu QGRSJ 0123456789 Il1| O0 rn m</p>
		<p class="mt-2 flex gap-4"><span>File</span><span>Home</span><span>Share</span><span>View</span><span>Picture Tools</span></p>
	</div>
</div>

</template>
<style scoped>

.mySettings {
	background-color: var(--color-paper); /* the sheet's, since this takes the sheet's place in the same window */
	color: var(--color-ink); /* which every section and popup inside inherits, along with the type myMono gives it */
}
.myBox {
	color: var(--color-strong);
	background-color: var(--color-surface);
	border: 1px solid var(--color-line);
}

</style>
