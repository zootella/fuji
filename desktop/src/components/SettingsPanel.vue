<script setup>//the settings panel: what a user changes from inside fuji, shown in the contact sheet's window in place of the sheet

import {ref} from 'vue'
import {settings, settingsSet} from '../settings.js'
import {brandName} from '../brand.js'
import SettingsFileTypes from './SettingsFileTypes.vue'

//the shell trades this with the sheet when either asks with s, and makes it fresh each time, so every box starts from the settings as they are now. A change reaches the settings object the moment the user commits it, and the file when fuji closes, as every setting does; settings.js checks it against the one schema, so nothing here repeats what a valid value is. A section with a life of its own, like the file types, is a component of its own, and a setting that is one box stays here

const cardImages = ref(settings.card.images)//what the box says, which becomes the setting only once the user commits it
function cardImagesCommit() {//enter, or leaving the box
	if (!settingsSet('card', 'images', cardImages.value)) cardImages.value = settings.card.images//a value the setting turns away, like 0, 2.5 or nothing at all, puts the box back to the setting as it stands
}

const faces = ref(settings.font.faces)//which set of fonts, the computer's or the ones fuji carries; the list offers only what the setting takes, so it never needs putting back
function facesCommit() {
	if (settingsSet('font', 'faces', faces.value)) emit('faces')//the root is the shell's, so it puts the new faces there
}

const fileTypes = ref(null)//the file types section, which asks the system again when told
function onFocus(focused) { if (focused) fileTypes.value?.look() }//the shell hands this view the window's focus events while it is showing; coming back from the system's own settings is when a default is most likely to have changed

const emit = defineEmits(['sheet', 'faces'])//s, for the contact sheet back; which view is showing is the shell's, so this only asks. And faces, when the fonts change
function onKey(e) {
	if (e.key == 's' && !e.ctrlKey && !e.metaKey) emit('sheet')//a keystroke in the box never gets here, since the shell leaves a form field its own keys
}

defineExpose({onKey, onFocus})//the calls of the shell's this view has a use for

</script>
<template>

<div class="mySettings myMono w-full h-full overflow-y-auto p-8">
	<h1 class="mb-8 text-white">Settings</h1>

	<label class="flex items-center gap-4">
		<span class="w-48">Images on a card</span>
		<input type="number" min="1" step="1" v-model.number="cardImages" @change="cardImagesCommit" @keydown.enter="$event.target.blur()" class="myBox w-24 px-2 py-1" />
	</label>
	<!-- no hint beneath: for now the sheet shows one card, so this is also how many of a folder's pictures it shows, which is scaffolding to know rather than something to tell a user -->

	<label class="mt-4 flex items-center gap-4">
		<span class="w-48">Fonts</span>
		<select v-model="faces" @change="facesCommit" class="myBox px-2 py-1"><!-- the focus stays once chosen, as any list's does, so a keyboard can keep choosing; escape, or a click away, gives s, h and g back to fuji -->
			<option value="platform">This computer's</option>
			<option value="bundled">{{brandName}}'s own, the same everywhere</option>
		</select>
	</label>

	<SettingsFileTypes ref="fileTypes" class="mt-12" />

	<p class="mt-12 text-neutral-500">Press S to return to the contact sheet</p>

	<!-- a sample of the root's text, apart from the panel's fixed-width type, so the fonts choice above can be seen changing it: first a line whose letters give a typeface away, the pangram for every shape, AVATAR and Wavy Tofu for the spacing between pairs, QGRSJ for the letters faces differ on most, and Il1| O0 rn m for the ones easiest to confuse; then the words of File Explorer's ribbon, to set beside it on Windows. Here until fuji has text of its own in that face -->
	<div class="mySans mt-12">
		<p>Sphinx of black quartz, judge my vow. AVATAR Wavy Tofu QGRSJ 0123456789 Il1| O0 rn m</p>
		<p class="mt-2 flex gap-4"><span>File</span><span>Home</span><span>Share</span><span>View</span><span>Picture Tools</span></p>
	</div>
</div>

</template>
<style scoped>

.mySettings {
	background-color: black; /* the sheet's, since this takes the sheet's place in the same window */
	color: #a3a3a3; /* which every section and popup inside inherits, along with the type myMono gives it */
	color-scheme: dark; /* so the boxes' own parts, the number box's spinner and caret and the list's drop-down, are drawn for a dark page */
}
.myBox {
	color: white;
	background-color: #171717;
	border: 1px solid #404040;
}

</style>
