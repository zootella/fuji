<script setup>//the file types section of fuji's settings: a chip for every kind of file fuji opens, under the program that opens it now, and a card for each

import {ref, computed, onMounted, onBeforeUnmount} from 'vue'
import {associateAnswers, associateOpens, associateActive, associateChoose, associateLook, associateDiffers, associateProgram, associateFinish} from '../associate.js'
import {logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {platform} from './library.js'
import {fileTypes, fileTypesEnabled} from '../fileTypes.js'
import SettingsFileTypeCard from './SettingsFileTypeCard.vue'

/*
Every kind of file fuji opens appears once, as a chip under the program the system opens it with now: Opens with Fuji first, shown even when it holds nothing; then each other program, headed Currently opens with and its name, the one opening the most kinds first; then Currently opens with nothing. Currently, because it is true, and because it says gently that this can change. The panel sets another program's name, or the name of a page in the system's settings, in single quotes and italics everywhere, and leaves Fuji and nothing plain. It is a summary of who has what, not a list of what fuji would like. On the installed copy Currently opens with nothing is all but always empty, since the system opens a kind no other program offers with fuji on the offer alone. Last, Additional formats coming soon to Fuji holds the kinds fuji does not open yet, whose cards describe the format and offer nothing; this is the one place fuji shows them. Nothing on screen explains the panel, on purpose: the headings say what opens each list and a card says the rest, and a panel that needs a paragraph to explain its own colors has already failed. A chip is the panel's ordinary text, since its heading says where it stands, and turns the warning color where the answer and the system disagree. What a card holds, and how it places itself and closes, is SettingsFileTypeCard's; this section decides which card is up.

Resting the pointer on a chip shows its card, after a moment so that sweeping across a row flashes nothing, and once one is up the next chip's comes at once; moving off the chip and its card puts it away, after a moment long enough to cross the gap between them. Clicking a chip pins its card, and so does clicking anywhere in a card, so one that appeared on hover cannot vanish between two clicks; hover then leaves it alone. A click on another chip moves it there, and the same chip again, a click anywhere else, esc, and choosing all put it away, since the chip then shows the result.

The warning color means something to attend to, and it appears in two places only. One is the note that says this copy cannot change anything, whose cards then offer no buttons at all, since a grayed one reads as broken. The other is a disagreement, an answer the system has not carried out, which only its own settings can do, since fuji never writes a saved choice. Its chip turns the warning color, and the section opens with a link in that color for each direction, » In 'Set defaults by app' choose Fuji for .png, and » In 'Set defaults by app' choose another program for .jpg, which each such card carries too. A link rather than a button, by the rule this section keeps: a button changes something inside fuji, and a link goes somewhere outside it. One for each direction rather than one per chip, because Windows' link to Default apps can name an application and nothing finer: Windows 11 opens it at fuji's page, and Windows 10 ignores the name and opens Default apps itself. The no's link has or and a plain Choose Fuji after it, because a square does not remember the path to it: a no just given and about to be finished, one given a day ago and forgotten, and one from long ago with fuji chosen in the system since all look the same. The link finishes the no and the button takes it back, and the user knows which they meant. The button answers every kind its line names in one click, which is safe, since the system already opens them with fuji. A yes gets the link alone, since No longer open with Fuji beside it would read as though fuji opened those kinds now.

associate.js is the policy and carries the essay; this only shows it and hands the user's clicks down. Fuji asks the system what opens each kind only while this is showing, so it looks when the section appears, and the panel calls look again whenever the window comes back into focus, which is how it notices the user returning from the system's settings. Until the system has answered, and on a platform where fuji cannot ask it yet, every kind appears in one list with no heading.
*/

const fileExtensions = Object.keys(fileTypesEnabled).sort()//every kind of file fuji opens, alphabetical, which is the order every list below keeps
const fileExtensionsComing = Object.keys(fileTypes).filter(extension => !fileTypes[extension].enabled).sort()//and every kind it will open, shown under the last heading and nowhere else
const hoverShow = 300//milliseconds the pointer rests on a chip before its card appears
const hoverHide = 200//and after it leaves the chip and the card, long enough to cross from one to the other

const inactiveNote = {//why the answers here cannot be changed, on a copy that cannot act on them
	windows: `Only the installed copy of ${brandName} can change these.`,
	mac: `Coming to the Mac. For now, use Get Info in the Finder.`,
	linux: `Not available on Linux yet.`,
}[platform()]

const fileLists = computed(() => {//the chips as lists of {heading, named, extensions}, in the essay's order; named is another program's name, set after the heading
	let coming = fileExtensionsComing.length ? [{heading: `Additional formats coming soon to ${brandName}`, named: '', extensions: fileExtensionsComing}] : []//last, whatever else there is
	if (Object.keys(associateOpens.value).length == 0) return [{heading: '', named: '', extensions: fileExtensions}, ...coming]//the system has not been asked, so one list with nothing true to say above it
	let lists = {[brandName]: []}//fuji's own list, there from the start so it shows even when it holds nothing
	for (let extension of fileExtensions) (lists[associateProgram(extension)] ??= []).push(extension)//a blank program collects the kinds nothing opens
	let ordered = Object.entries(lists).map(([program, extensions]) => ({...listFor(program), extensions}))
	ordered.sort((a, b) => a.rank - b.rank || b.extensions.length - a.extensions.length || a.named.localeCompare(b.named))//fuji first and nothing last, and between them the program opening the most kinds first, ties by name
	return [...ordered, ...coming]
})
function listFor(program) {//where a program's list goes and the words above it: fuji first and nothing last whatever their sizes, every other program between
	if (program == brandName) return {rank: 0, heading: `Opens with ${brandName}`, named: ''}
	if (!program)             return {rank: 2, heading: 'Currently opens with nothing', named: ''}
	return {rank: 1, heading: 'Currently opens with ', named: program}
}
const pendingYes = computed(() => pending('yes'))//the kinds answered yes that the system opens with something else
const pendingNo  = computed(() => pending('no'))//and the kinds answered no that it still opens with fuji
function pending(answer) { return fileExtensions.filter(extension => associateDiffers(extension) && associateAnswers.value[extension] == answer) }//an answer the system has not carried out, which only its own settings can

function sayList(items) {//items in a sentence: .png, then .gif and .png, then .bmp, .gif, and .png
	if (items.length <= 2) return items.join(' and ')
	return `${items.slice(0, -1).join(', ')}, and ${items.at(-1)}`
}

const shown = ref(null)//the chip whose card is up, {extension, anchor, pinned}, or null when none is
const shownList = computed(() => shown.value && fileLists.value.find(list => list.extensions.includes(shown.value.extension)))//the list that chip sits in, whose heading is also its card's status
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
function settle(extensions, answer) { associateChoose([...extensions], answer).catch(error => logTrouble('settings: carrying out an answer', error)) }//every kind a line names, answered the way the system already opens them; a copy, since the line's own list changes as the answers land
function look() { associateLook().catch(error => logTrouble('settings: asking what opens each kind of file', error)) }

onMounted(look)
defineExpose({look})//for the panel, which hears when the window comes back into focus

</script>
<template>

<section class="relative"><!-- relative, so the card below is placed within this section and scrolls with its chip -->
	<h2 class="mb-2 text-strong">File types</h2>
	<p v-if="!associateActive" class="text-warn italic">{{inactiveNote}}</p><!-- the warning color, in italics and without the underline, being something to know rather than somewhere to go -->
	<!-- each a link in the warning color, led by the chevron that flags something to do; the no's line adds a plain button that agrees with the system -->
	<p v-if="pendingYes.length > 0" class="mt-2"><a href="#" class="text-warn underline" @click.prevent="finish"><b>»</b> In '<i>Set defaults by app</i>' choose {{brandName}} for {{sayList(pendingYes)}}</a></p>
	<p v-if="pendingNo.length > 0" class="mt-2 flex flex-wrap items-center gap-2"><a href="#" class="text-warn underline" @click.prevent="finish"><b>»</b> In '<i>Set defaults by app</i>' choose another program for {{sayList(pendingNo)}}</a> <span>or</span> <button type="button" class="myChoice px-3" @click="settle(pendingNo, 'yes')">Choose {{brandName}}</button></p>

	<div v-for="list in fileLists" :key="list.heading + list.named" class="mt-6"><!-- the whole heading, which no two lists share -->
		<h3 v-if="list.heading" class="text-strong">{{list.heading}}<template v-if="list.named">'<i>{{list.named}}</i>'</template></h3>
		<p v-if="list.extensions.length == 0" class="mt-1 text-fainter">none yet</p>
		<div class="flex flex-wrap gap-2 mt-1">
			<!-- mousedown.prevent keeps the focus where it is, so a click on a chip, which moves or puts away the card itself, does not also tell a pinned card it lost the focus -->
			<button v-for="extension in list.extensions" :key="extension" type="button" :class="{'text-warn': associateDiffers(extension)}" class="myChip px-2" @mouseenter="chipEnter(extension, $event)" @mouseleave="hoverLeave" @mousedown.prevent @click="chipClick(extension, $event)">{{extension}}</button>
		</div>
	</div>

	<SettingsFileTypeCard v-if="shown" :extension="shown.extension" :list="shownList" :anchor="shown.anchor" :pinned="shown.pinned" @close="shown = null" @mouseenter="hoverStay" @mouseleave="hoverLeave" @click="cardClick" />
</section>

</template>
<style scoped>

.myChip {
	border: 1px solid var(--color-line);
}
.myChoice {
	border: 1px solid var(--color-line); /* the card's buttons, which the no's line at the top offers as well */
}

</style>
