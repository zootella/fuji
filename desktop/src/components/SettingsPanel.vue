<script setup>//the settings panel: what a user changes from inside fuji, shown in the contact sheet's window in place of the sheet

import {ref} from 'vue'
import {settings, settingsSet, settingsThumbnailBeams} from '../settings.js'
import {fitNames, fitDescriptions} from '../fit.js'
import {brandName, brandVersion, urlHome} from '../brand.js'
import {platform} from './library.js'
import {processOpen} from '../process.js'//the site in the system's browser, rather than in the web view
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
const about = ref(null)//the About section at the end, which the Mac's menu item scrolls into view
const homeLink = `https://${urlHome}`//the site, https://fujidesktop.app, both where the link goes and its words
function showAbout() { about.value?.scrollIntoView({block: 'start'}) }//the panel's root scrolls, so this scrolls it
function onFocus(focused) { if (focused) fileTypes.value?.look() }//the shell hands this view the window's focus events while it is showing; coming back from the system's own settings is when a default is most likely to have changed

const emit = defineEmits(['sheet', 'faces', 'theme'])//s, for the contact sheet back; which view is showing is the shell's, so this only asks. And faces and theme, when the fonts or the appearance change
function onKey(e) {
	if (e.key == 's' && !e.ctrlKey && !e.metaKey) emit('sheet')//a keystroke in the box never gets here, since the shell leaves a form field its own keys
}

defineExpose({onKey, onFocus, showAbout})//the calls of the shell's this view has a use for

</script>
<template>

<div class="mySettings myMono w-full h-full overflow-y-auto p-8">
	<h1 class="mb-8 text-strong">Settings</h1>

	<div class="grid grid-cols-[auto_1fr] gap-4 items-baseline"><!-- every setting but the file types in one grid: the titles in a column as wide as the longest of them and no wider, the boxes and buttons in the rest, and each title on the baseline of its options' first line, which puts a title level with its box and with the first button of its group alike -->
		<label for="bucketImages">Images in a bucket</label>
		<input id="bucketImages" type="number" min="1" step="1" v-model.number="bucketImages" @change="bucketImagesCommit" @keydown.enter="$event.target.blur()" class="myBox w-24 px-2 py-1" />
		<label for="sheetBuckets">Buckets on the sheet</label>
		<input id="sheetBuckets" type="number" min="1" step="1" v-model.number="sheetBuckets" @change="sheetBucketsCommit" @keydown.enter="$event.target.blur()" class="myBox w-24 px-2 py-1" />
		<!-- no hint beneath: for now the two together are also how many of a folder's pictures the sheet shows, which is scaffolding to know rather than something to tell a user -->

		<!-- in each group, a button sits on the baseline of its words rather than centered on them, so a label that wraps in a narrow window keeps its button beside its first line -->
		<span>Thumbnail size</span>
		<div role="radiogroup" aria-label="Thumbnail size" class="grid gap-1">
			<label v-for="name in settingsThumbnailBeams" :key="name" class="flex items-baseline gap-2"><input type="radio" name="beam" :value="name" v-model="beam" @change="beamCommit" /><span>{{name == 'Xl' ? 'XL' : name}}, {{settings.thumbnail[name.toLowerCase()]}} px</span></label><!-- the beam's length beside its name, since it is what every thumbnail is measured against and what a bucket's cost follows -->
		</div>

		<span>Fit</span>
		<div role="radiogroup" aria-label="Fit" class="grid gap-1">
			<label v-for="name in fitNames" :key="name" class="flex items-baseline gap-2"><input type="radio" name="fit" :value="name" v-model="fit" @change="fitCommit" /><span>{{name.replace(/Fit$/, '')}}: {{fitDescriptions[name]}}</span></label><!-- Square rather than SquareFit, as the sheet's toolbar wrote them until the choice moved here, and then what it does, from the table beside the fits themselves -->
		</div>

		<span>Typography</span>
		<div role="radiogroup" aria-label="Typography" class="grid gap-1"><!-- radio buttons rather than a list, so every choice and what it gives are in view without a click; the panel scrolls when it grows -->
			<label class="flex items-baseline gap-2"><input type="radio" name="faces" value="system" v-model="faces" @change="facesCommit" /><span>System fonts<template v-if="systemFaces">: <i>{{systemFaces[0]}}</i>, with <i>{{systemFaces[1]}}</i></template></span></label><!-- the words in one span, so the flex row holds the button and them as two items, rather than putting its gap around every name -->
			<label class="flex items-baseline gap-2"><input type="radio" name="faces" value="bundled" v-model="faces" @change="facesCommit" /><span>{{brandName}} fonts: <i>Inter</i>, with <i>IBM Plex Mono</i></span></label>
			<label v-if="platform() != 'linux'" class="flex items-baseline gap-2"><input type="radio" name="faces" value="retro" v-model="faces" @change="facesCommit" /><span>Retro fonts: <i>Verdana</i>, vibing the 2000s Web</span></label><!-- with IBM Plex Mono for the fixed-width text, left out of the label; Verdana is the system's rather than carried, and Windows and the Mac install it where Linux does not, so there the button is not offered and the setting's check turns the word away -->
		</div>

		<span>Appearance</span>
		<div role="radiogroup" aria-label="Appearance" class="grid gap-1">
			<label class="flex items-baseline gap-2"><input type="radio" name="mode" value="light" v-model="mode" @change="modeCommit" /><span>Light</span></label>
			<label class="flex items-baseline gap-2"><input type="radio" name="mode" value="dark" v-model="mode" @change="modeCommit" /><span>Dark</span></label>
			<label class="flex items-baseline gap-2"><input type="radio" name="mode" value="system" v-model="mode" @change="modeCommit" /><span>System</span></label>
		</div>
	</div>

	<SettingsFileTypes ref="fileTypes" class="mt-12" />

	<div ref="about" class="mt-12"><!-- at the end, where About Fuji in the Mac's application menu brings the panel scrolled to; the system's own About panel is gone, so this is the one place fuji says its version -->
		<h2 class="mb-2 text-strong">About {{brandName}}</h2>
		<p>{{brandName}} {{brandVersion}}</p>
		<p class="mt-2"><a :href="homeLink" class="underline" @click.prevent="processOpen(homeLink)">{{homeLink}}</a></p><!-- the system's browser opens it rather than the web view navigating away from fuji -->
		<svg class="my-8 fill-brand" width="360" height="360" viewBox="0 0 360 360" :aria-label="brandName"><circle cx="180" cy="180" r="180" /></svg><!-- fuji's mark: the disc that is the application icon, the site's favicon and the README's banner, at the README's size, 360 CSS pixels across; in the brand color by its palette name, which is the same in light and dark. Twice the panel's usual space above and below, the disc's alone, so the line after it sets none of its own -->
	</div>

	<p class="text-fainter italic">Press S to return to the contact sheet</p>
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
