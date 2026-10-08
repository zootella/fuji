<script setup>//the settings panel: what a user changes from inside fuji, shown in the contact sheet's window in place of the sheet

import {ref} from 'vue'
import {settings, settingsSet, settingsThumbnailBeams} from '../settings.js'
import {fitNames, fitDescriptions} from '../fit.js'
import {brandName} from '../brand.js'
import {platform} from './library.js'
import SettingsFileTypes from './SettingsFileTypes.vue'

//the shell trades this with the sheet when either asks with s, and makes it fresh each time, so every box starts from the settings as they are now. A change reaches the settings object the moment the user commits it, and the file when fuji closes, as every setting does; settings.js checks it against the one schema, so nothing here repeats what a valid value is. A section with a life of its own, like the file types, is a component of its own, and a setting that is one box stays here

const bucketImages = ref(settings.bucket.images)//what the box says, which becomes the setting only once the user commits it
function bucketImagesCommit() {//enter, or leaving the box
	if (!settingsSet('bucket', 'images', bucketImages.value)) bucketImages.value = settings.bucket.images//a value the setting turns away, like 0, 2.5 or nothing at all, puts the box back to the setting as it stands
}
const sheetBuckets = ref(settings.sheet.buckets)//the same, for how many buckets the sheet holds
function sheetBucketsCommit() {
	if (!settingsSet('sheet', 'buckets', sheetBuckets.value)) sheetBuckets.value = settings.sheet.buckets
}

const beam = ref(settings.thumbnail.beam)//which beam length every thumbnail is measured against; the buttons offer only the names the setting takes, so it never needs putting back
function beamCommit() { settingsSet('thumbnail', 'beam', beam.value) }//the sheet reads it again when it comes back on screen and remakes every bucket, since a flow reads the beam once, as it is made
const fit = ref(settings.thumbnail.fit)//how every thumbnail is sized against that beam, one of the fits fit.js lists
function fitCommit() { settingsSet('thumbnail', 'fit', fit.value) }//the same way

const faces = ref(settings.font.faces)//which fonts, system, bundled or retro; the buttons offer only what the setting takes on this platform, so it never needs putting back
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
	<!-- a specimen of the caption face, apart from the panel's fixed-width type, at the top so the title bar is right above it, in the strongest text color so a screenshot compares renderers rather than inks: the letters that give a typeface away, the pangram for every shape, AVATAR and Wavy Tofu for the spacing between pairs, QGRSJ for the letters faces differ on most, Il1 O0 rn m for the ones easiest to confuse, and .txt because a caption is a file name; the same characters as the name of the text file in the test folder, for a line of Explorer's to be set against. Then a second line of the kind the contact sheet shows, a path, a date, a size and dimensions, with a middle dot between and a multiplication sign; and a label above each pair, in the same face, saying which it is. First the pair as it is, a bare mySans drawn in whichever face the setting names, which changes when a Typography button is pressed. Then a spike, to be removed: the same pair set directly in the scoped styles below, each isolated from the setting, the system face as the control, then Inter and Verdana each sized two ways, by the x-height ratio and by whole css pixels, to compare before choosing -->
	<div class="mySans mb-8 text-strong">
		<p>1</p>
		<p>Sphinx of black quartz, judge my vow. AVATAR Wavy Tofu QGRSJ 0123456789 Il1 O0 rn m.txt</p>
		<p>C:\Users\Name\Desktop · 2026-Oct-8 · 4589 KB · 464 × 698</p>
		<div class="mySpikeGap"></div>
		<p class="mySpikeSystem">2</p>
		<p class="mySpikeSystem">Sphinx of black quartz, judge my vow. AVATAR Wavy Tofu QGRSJ 0123456789 Il1 O0 rn m.txt</p>
		<p class="mySpikeSystem">C:\Users\Name\Desktop · 2026-Oct-8 · 4589 KB · 464 × 698</p>
		<div class="mySpikeGap"></div>
		<template v-if="false"><!-- hidden for the moment, to compare the pixel pairs alone -->
		<p class="mySpikeInter">3</p>
		<p class="mySpikeInter">Sphinx of black quartz, judge my vow. AVATAR Wavy Tofu QGRSJ 0123456789 Il1 O0 rn m.txt</p>
		<p class="mySpikeInter">C:\Users\Name\Desktop · 2026-Oct-8 · 4589 KB · 464 × 698</p>
		</template>
		<p class="mySpikeInterPx">4</p>
		<p class="mySpikeInterPx">Sphinx of black quartz, judge my vow. AVATAR Wavy Tofu QGRSJ 0123456789 Il1 O0 rn m.txt</p>
		<p class="mySpikeInterPx">C:\Users\Name\Desktop · 2026-Oct-8 · 4589 KB · 464 × 698</p>
		<div class="mySpikeGap"></div>
		<template v-if="false"><!-- hidden for the moment, to compare the pixel pairs alone -->
		<p class="mySpikeVerdana">5</p>
		<p class="mySpikeVerdana">Sphinx of black quartz, judge my vow. AVATAR Wavy Tofu QGRSJ 0123456789 Il1 O0 rn m.txt</p>
		<p class="mySpikeVerdana">C:\Users\Name\Desktop · 2026-Oct-8 · 4589 KB · 464 × 698</p>
		</template>
		<p class="mySpikeVerdanaPx">6</p>
		<p class="mySpikeVerdanaPx">Sphinx of black quartz, judge my vow. AVATAR Wavy Tofu QGRSJ 0123456789 Il1 O0 rn m.txt</p>
		<p class="mySpikeVerdanaPx">C:\Users\Name\Desktop · 2026-Oct-8 · 4589 KB · 464 × 698</p>
	</div>

	<h1 class="mb-8 text-strong">Settings</h1>

	<label class="flex items-center gap-4">
		<span class="w-48">Images in a bucket</span>
		<input type="number" min="1" step="1" v-model.number="bucketImages" @change="bucketImagesCommit" @keydown.enter="$event.target.blur()" class="myBox w-24 px-2 py-1" />
	</label>
	<label class="mt-1 flex items-center gap-4">
		<span class="w-48">Buckets on the sheet</span>
		<input type="number" min="1" step="1" v-model.number="sheetBuckets" @change="sheetBucketsCommit" @keydown.enter="$event.target.blur()" class="myBox w-24 px-2 py-1" />
	</label>
	<!-- no hint beneath: for now the two together are also how many of a folder's pictures the sheet shows, which is scaffolding to know rather than something to tell a user -->

	<div class="mt-4 flex gap-4">
		<span class="w-48">Thumbnail size</span>
		<div role="radiogroup" aria-label="Thumbnail size">
			<label v-for="(name, i) in settingsThumbnailBeams" :key="name" :class="{'mt-1': i > 0}" class="flex items-center gap-2"><input type="radio" name="beam" :value="name" v-model="beam" @change="beamCommit" /><span>{{name == 'Xl' ? 'XL' : name}}, {{settings.thumbnail[name.toLowerCase()]}} px</span></label><!-- the beam's length beside its name, since it is what every thumbnail is measured against and what a bucket's cost follows -->
		</div>
	</div>

	<div class="mt-4 flex gap-4">
		<span class="w-48">Fit</span>
		<div role="radiogroup" aria-label="Fit">
			<label v-for="(name, i) in fitNames" :key="name" :class="{'mt-1': i > 0}" class="flex items-center gap-2"><input type="radio" name="fit" :value="name" v-model="fit" @change="fitCommit" /><span>{{name.replace(/Fit$/, '')}}: {{fitDescriptions[name]}}</span></label><!-- Square rather than SquareFit, as the sheet's toolbar wrote them until the choice moved here, and then what it does, from the table beside the fits themselves -->
		</div>
	</div>

	<div class="mt-4 flex gap-4">
		<span class="w-48">Typography</span>
		<div role="radiogroup" aria-label="Typography"><!-- radio buttons rather than a list, so every choice and what it gives are in view without a click; the panel scrolls when it grows -->
			<label class="flex items-center gap-2"><input type="radio" name="faces" value="system" v-model="faces" @change="facesCommit" /><span>System fonts<template v-if="systemFaces">: <i>{{systemFaces[0]}}</i>, with <i>{{systemFaces[1]}}</i></template></span></label><!-- the words in one span, so the flex row holds the button and them as two items, rather than putting its gap around every name -->
			<label class="mt-1 flex items-center gap-2"><input type="radio" name="faces" value="bundled" v-model="faces" @change="facesCommit" /><span>{{brandName}} fonts: <i>Inter</i>, with <i>IBM Plex Mono</i></span></label>
			<label v-if="platform() != 'linux'" class="mt-1 flex items-center gap-2"><input type="radio" name="faces" value="retro" v-model="faces" @change="facesCommit" /><span>Retro fonts: <i>Verdana</i>, vibing the 2000s Web</span></label><!-- with IBM Plex Mono for the fixed-width text, left out of the label; Verdana is the system's rather than carried, and Windows and the Mac install it where Linux does not, so there the button is not offered and the setting's check turns the word away -->
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

	<p class="mt-12">Press S to return to the contact sheet</p>
</div>

</template>
<style scoped>

.mySettings {
	background-color: var(--color-paper); /* the sheet's, since this takes the sheet's place in the same window */
	color: var(--color-ink); /* which every section and popup inside inherits, along with the type myMono gives it */
}
.mySpikeGap       { height: 1rlh } /* one row of the caption face, empty */
.mySpikeSystem    { font: menu; line-height: 1rlh }
.mySpikeInter     { font: menu; line-height: 1rlh; font-family: Inter, system-ui, sans-serif; font-size-adjust: 0.55 }
.mySpikeInterPx   { font: menu; line-height: 1rlh; font-family: Inter, system-ui, sans-serif; font-size: 12px }
.mySpikeVerdana   { font: menu; line-height: 1rlh; font-family: Verdana, sans-serif;          font-size-adjust: 0.48 }
.mySpikeVerdanaPx { font: menu; line-height: 1rlh; font-family: Verdana, sans-serif;          font-size: 12px }
.myBox {
	color: var(--color-strong);
	background-color: var(--color-surface);
	border: 1px solid var(--color-line);
}

</style>
