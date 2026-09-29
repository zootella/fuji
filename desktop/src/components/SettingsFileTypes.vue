<script setup>//the file types section of fuji's settings: every kind of file fuji opens, as a chip under the program that opens it now, with a card for each

import {ref, computed, onMounted, onBeforeUnmount} from 'vue'
import {associateAnswers, associateOpens, associateActive, associateLook, associateDiffers, associateProgram, associateFinish} from '../associate.js'
import {logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {imageTypes, platform} from './library.js'
import SettingsFileTypeCard from './SettingsFileTypeCard.vue'

/*
Every kind of file fuji opens appears once, as a chip under the program the system opens it with right now, headed Opens with and the program's name: fuji's own list first, whether or not it holds anything yet, then the other programs, the one opening the most kinds first, and last Opens with nothing, for the kinds nothing opens. Nothing on screen explains any of this, on purpose. The headings say what opens each list and a card says the rest, in the few words an application uses, and a panel that needed a paragraph to explain its own colors would already have failed. What the paragraph would have said lives here instead: fuji is offered for every kind whatever the answer, Choose Fuji makes it the program for a kind, and Change takes that back, leaving the system to open the kind with something else. A chip shows its answer at a glance, yes bright, no dim and struck through, and dim alone for a kind not answered yet, and turns amber where the answer and the system disagree. What a card holds, and how it places itself and knows when to close, is SettingsFileTypeCard's; this section decides which card is up.

Resting the pointer on a chip shows its card, after a moment so that sweeping across a row flashes nothing, and once one is up the next chip's comes at once; moving off the chip and its card puts it away, after a moment long enough to cross the gap between them. Clicking a chip pins its card, and so does clicking anywhere in a card, so one that appeared on hover cannot vanish between two clicks; hover then leaves it alone. A click on another chip moves it there in one click, the same chip again puts it away, and so do a click anywhere else, esc, and choosing, since the chip then shows the result.

Amber means one thing here, something for the user to attend to, so it appears only where an answer and the system disagree, on those chips and their cards. Every disagreement is settled toward fuji: a no the system does not share by Choose Fuji in the card, and a yes it does not share in the system's own settings, so while there is such a yes the section opens with one amber link naming those kinds, Choose Fuji for .png in Windows Settings, and no sentence around it. A link rather than a button, by the rule this section keeps: a button changes something inside fuji, and a link takes the user somewhere outside it. When nothing needs the system there is no link. One link rather than one per chip, because Windows' link to its Default apps can name an application and nothing finer: Windows 11 opens it at fuji's page, listing every kind fuji offers, and Windows 10 ignores the name and opens Default apps itself.

associate.js is the policy and carries the essay; this only shows it and hands the user's clicks down. Fuji asks the system what opens each kind only while this is showing, so it looks when the section appears, and the panel calls look again whenever the window comes back into focus, which is how it notices the user returning from the system's settings. Until the system has answered, and on a platform where fuji cannot ask it yet, there are no programs to list under, so every kind appears in one list with no heading.
*/

const fileExtensions = Object.keys(imageTypes).sort()//every kind of file fuji opens, alphabetical, which is the order every list below keeps
const hoverShow = 300//milliseconds the pointer rests on a chip before its card appears
const hoverHide = 200//and after it leaves the chip and the card, long enough to cross from one to the other

const inactiveNote = {//why the answers here cannot be changed, on a copy that cannot act on them
	windows: `Only the installed copy of ${brandName} can change these.`,
	mac: `Coming to the Mac. For now, use Get Info in the Finder.`,
	linux: `Not available on Linux yet.`,
}[platform()]

const fileLists = computed(() => {//the chips, as lists of {program, heading, extensions}, in the order the essay above gives
	if (Object.keys(associateOpens.value).length == 0) return [{program: '', heading: '', extensions: fileExtensions}]//the system has not been asked, so one list with nothing true to say above it
	let lists = {[brandName]: []}//fuji's own list, there from the start so it shows even when it holds nothing
	for (let extension of fileExtensions) (lists[associateProgram(extension)] ??= []).push(extension)//a blank program collects the kinds nothing opens
	let ordered = Object.entries(lists).map(([program, extensions]) => ({program, heading: `Opens with ${program || 'nothing'}`, extensions}))
	return ordered.sort((a, b) => listRank(a) - listRank(b) || b.extensions.length - a.extensions.length || a.program.localeCompare(b.program))//fuji first and nothing last, and between them the program opening the most kinds first, ties by name
})
function listRank(list) {//fuji's list first and nothing's last, whatever their sizes
	if (list.program == brandName) return 0
	if (!list.program)             return 2
	return 1
}
const systemMustChange = computed(() => fileExtensions.filter(extension => associateDiffers(extension) && associateAnswers.value[extension] == 'yes'))//the kinds answered yes that the system opens with something else, which only its own settings can settle; a no it does not share is settled in the card

function sayList(items) {//items in a sentence: .png, then .gif and .png, then .bmp, .gif and .png
	if (items.length <= 2) return items.join(' and ')
	return `${items.slice(0, -1).join(', ')} and ${items.at(-1)}`
}

function chipLook(extension) {//the classes that show a chip's answer, and a disagreement over it
	let answer = associateAnswers.value[extension]
	let look = answer == 'no' ? 'line-through ' : ''
	if (associateDiffers(extension)) return look + 'text-amber-400'
	if (answer == 'yes') return look + 'text-white'
	return look + 'text-neutral-500'
}

const shown = ref(null)//the chip whose card is up, {extension, anchor, pinned}, or null when none is
let hoverTimer = 0//a card waiting to appear or to go, which any newer move of the pointer replaces

function chipEnter(extension, e) {
	if (shown.value?.pinned) return//a card the user clicked stays put until they close it
	clearTimeout(hoverTimer)
	let next = {extension, anchor: e.currentTarget, pinned: false}
	if (shown.value) shown.value = next//a card is already up, so the next chip's comes at once
	else hoverTimer = setTimeout(() => { shown.value = next }, hoverShow)
}
function hoverLeave() {//off a chip, or off its card
	if (shown.value?.pinned) return
	clearTimeout(hoverTimer)
	hoverTimer = setTimeout(() => { shown.value = null }, hoverHide)
}
function hoverStay() { clearTimeout(hoverTimer) }//into the card before it went
function chipClick(extension, e) {
	clearTimeout(hoverTimer)
	if (shown.value?.pinned && shown.value.extension == extension) { shown.value = null; return }//the pinned chip again puts its card away
	shown.value = {extension, anchor: e.currentTarget, pinned: true}
}
function cardClick() { if (shown.value && !shown.value.pinned) shown.value = {...shown.value, pinned: true} }//any click in a card that appeared on hover pins it
onBeforeUnmount(() => clearTimeout(hoverTimer))

function finish() { associateFinish().catch(error => logTrouble('settings: opening windows settings', error)) }
function look() { associateLook().catch(error => logTrouble('settings: asking what opens each kind of file', error)) }

onMounted(look)
defineExpose({look})//for the panel, which hears when the window comes back into focus

</script>
<template>

<section class="relative"><!-- relative, so the card below is placed within this section and scrolls with its chip -->
	<h2 class="mb-2 text-white">File types</h2>
	<p v-if="!associateActive" class="text-neutral-500">{{inactiveNote}}</p>
	<p v-if="systemMustChange.length > 0"><a href="#" class="text-amber-400 underline" @click.prevent="finish">Choose {{brandName}} for {{sayList(systemMustChange)}} in Windows Settings</a></p>

	<div v-for="list in fileLists" :key="list.program" class="mt-6">
		<h3 v-if="list.heading" class="text-white">{{list.heading}}</h3>
		<p v-if="list.extensions.length == 0" class="mt-1 text-neutral-600">none yet</p>
		<div class="flex flex-wrap gap-2 mt-1">
			<!-- mousedown.prevent keeps the focus where it is, so a pinned card is not told it lost the focus by a click on a chip, which moves or puts it away by itself -->
			<button v-for="extension in list.extensions" :key="extension" type="button" :class="chipLook(extension)" class="myChip px-2" @mouseenter="chipEnter(extension, $event)" @mouseleave="hoverLeave" @mousedown.prevent @click="chipClick(extension, $event)">{{extension}}</button>
		</div>
	</div>

	<SettingsFileTypeCard v-if="shown" :extension="shown.extension" :anchor="shown.anchor" :pinned="shown.pinned" @close="shown = null" @mouseenter="hoverStay" @mouseleave="hoverLeave" @click="cardClick" />
</section>

</template>
<style scoped>

.myChip {
	border: 1px solid #404040;
}

</style>
