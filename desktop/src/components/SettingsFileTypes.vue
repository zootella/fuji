<script setup>//the file types section of fuji's settings: every kind of file fuji opens, as a chip under the program that opens it now, and every kind it will open under a heading of its own, with a card for each

import {ref, computed, onMounted, onBeforeUnmount} from 'vue'
import {associateAnswers, associateOpens, associateActive, associateChoose, associateLook, associateDiffers, associateProgram, associateFinish} from '../associate.js'
import {logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {platform} from './library.js'
import {fileTypes, fileTypesEnabled} from '../fileTypes.js'
import SettingsFileTypeCard from './SettingsFileTypeCard.vue'

/*
Every kind of file fuji opens appears once, as a chip under the program the system opens it with right now: fuji's own list first, headed Opens with Fuji whether or not it holds anything yet, then the other programs, each headed Currently opens with and its name, the one opening the most kinds first, and last Currently opens with nothing, for the kinds nothing opens. Currently, because it is true and because it says, gently, that this can change. Anything of a third party's, another program's name or the name of a page in the system's settings, is set in single quotes and italics, everywhere in the panel; Fuji and nothing are not. That order is the point of the section. It is a summary of who has what, not a list of what fuji would like, so a user who sees a kind under nothing, or under a program they no longer use, can click it and give it to fuji from the card, and a user who does not know a kind can read its card first. Below all of those, Additional formats coming soon to Fuji holds every kind in the table fuji does not open yet, and it is the only place fuji does anything with those: they are not listed in a folder, not offered to the system, not in fuji's settings file, and their cards say what the format is and where it came from, and stop there. Nothing on screen explains any of this, on purpose. The headings say what opens each list and a card says the rest, in the few words an application uses, and a panel that needed a paragraph to explain its own colors would already have failed. What the paragraph would have said lives here instead: fuji is offered for every kind whatever the answer, Choose Fuji makes it the program for a kind, and Change takes that back, leaving the system to open the kind with something else, and each is finished in the system's own settings wherever the user has saved a choice there. Every chip is the panel's ordinary text, since the heading it sits under already says where the kind stands, and a brighter or dimmer chip would only say it twice; a chip turns amber where the answer and the system disagree, the amber of the link that settles it, and that is the one look a chip has. What a card holds, and how it places itself and knows when to close, is SettingsFileTypeCard's; this section decides which card is up.

Resting the pointer on a chip shows its card, after a moment so that sweeping across a row flashes nothing, and once one is up the next chip's comes at once; moving off the chip and its card puts it away, after a moment long enough to cross the gap between them. Clicking a chip pins its card, and so does clicking anywhere in a card, so one that appeared on hover cannot vanish between two clicks; hover then leaves it alone. A click on another chip moves it there in one click, the same chip again puts it away, and so do a click anywhere else, esc, and choosing, since the chip then shows the result.

Amber means one thing here, something for the user to attend to, so it appears only where an answer and the system disagree, on those chips and their cards, and on the note that says a copy cannot change any of this, whose cards offer no buttons at all rather than buttons that do nothing. Every disagreement is an answer the system has not carried out, and only the system's own settings can carry it out, since fuji never writes the choice a user saved there. So while there is a yes it does not share, the section opens with an amber link naming those kinds, » In 'Set defaults by app' choose Fuji for .png, and while there is such a no, a line with another, » In 'Set defaults by app' choose another program for .jpg, then or and a plain Choose Fuji, with no sentence around either; the card of each such kind carries the same, so it is at hand wherever the user meets the amber. The no has the button because the path to a square is not in the square: the same disagreement can be a no just given and about to be finished in the system, one given a day ago and forgotten, or one from long ago with fuji chosen in the system since. The link finishes the no and the button takes it back, one way out for each path, and the user knows which they meant. The button answers every kind its line names in one click, and can, because it changes nothing that opens: the system already opens those kinds with fuji, so it only brings fuji's answers into line. A yes has the link alone, since fuji is what the user asked for, and a way back beside it, No longer open with Fuji, would read as though fuji opened those kinds now, which it does not. A link rather than a button, by the rule this section keeps: a button changes something inside fuji, and a link takes the user somewhere outside it. When nothing needs the system there is no link. One link for each direction rather than one per chip, because Windows' link to its Default apps can name an application and nothing finer: Windows 11 opens it at fuji's page, listing every kind fuji offers, and Windows 10 ignores the name and opens Default apps itself.

associate.js is the policy and carries the essay; this only shows it and hands the user's clicks down. Fuji asks the system what opens each kind only while this is showing, so it looks when the section appears, and the panel calls look again whenever the window comes back into focus, which is how it notices the user returning from the system's settings. Until the system has answered, and on a platform where fuji cannot ask it yet, there are no programs to list under, so every kind appears in one list with no heading.
*/

const fileExtensions = Object.keys(fileTypesEnabled).sort()//every kind of file fuji opens, alphabetical, which is the order every list below keeps
const fileExtensionsComing = Object.keys(fileTypes).filter(extension => !fileTypes[extension].enabled).sort()//and every kind it will open, which appear under the last heading and nowhere else
const hoverShow = 300//milliseconds the pointer rests on a chip before its card appears
const hoverHide = 200//and after it leaves the chip and the card, long enough to cross from one to the other

const inactiveNote = {//why the answers here cannot be changed, on a copy that cannot act on them
	windows: `Only the installed copy of ${brandName} can change these.`,
	mac: `Coming to the Mac. For now, use Get Info in the Finder.`,
	linux: `Not available on Linux yet.`,
}[platform()]

const fileLists = computed(() => {//the chips, as lists of {program, heading, named, extensions}, in the order the essay above gives; named is another program's name, set in quotes and italics after the heading, and blank where the heading says it all
	let coming = fileExtensionsComing.length ? [{program: '', heading: `Additional formats coming soon to ${brandName}`, named: '', extensions: fileExtensionsComing}] : []//last, whatever else there is
	if (Object.keys(associateOpens.value).length == 0) return [{program: '', heading: '', named: '', extensions: fileExtensions}, ...coming]//the system has not been asked, so one list with nothing true to say above it
	let lists = {[brandName]: []}//fuji's own list, there from the start so it shows even when it holds nothing
	for (let extension of fileExtensions) (lists[associateProgram(extension)] ??= []).push(extension)//a blank program collects the kinds nothing opens
	let ordered = Object.entries(lists).map(([program, extensions]) => ({program, ...listHeading(program), extensions}))
	ordered.sort((a, b) => listRank(a) - listRank(b) || b.extensions.length - a.extensions.length || a.program.localeCompare(b.program))//fuji first and nothing last, and between them the program opening the most kinds first, ties by name
	return [...ordered, ...coming]
})
function listHeading(program) {//the words above a list, and the program's name to set in quotes and italics after them, blank for fuji and for nothing
	if (program == brandName) return {heading: `Opens with ${program}`, named: ''}
	if (!program)             return {heading: 'Currently opens with nothing', named: ''}
	return {heading: 'Currently opens with ', named: program}
}
function listRank(list) {//fuji's list first and nothing's last, whatever their sizes
	if (list.program == brandName) return 0
	if (!list.program)             return 2
	return 1
}
const pendingYes = computed(() => pending('yes'))//the kinds answered yes that the system opens with something else
const pendingNo  = computed(() => pending('no'))//and the kinds answered no that it still opens with fuji
function pending(answer) { return fileExtensions.filter(extension => associateDiffers(extension) && associateAnswers.value[extension] == answer) }//an answer the system has not carried out, which only its own settings can

function sayList(items) {//items in a sentence: .png, then .gif and .png, then .bmp, .gif, and .png
	if (items.length <= 2) return items.join(' and ')
	return `${items.slice(0, -1).join(', ')}, and ${items.at(-1)}`
}

function chipLook(extension) {//the class that marks a disagreement over a chip, and nothing otherwise: its answer is said by the heading it sits under
	return associateDiffers(extension) ? 'text-amber-400' : ''
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
function settle(extensions, answer) { associateChoose([...extensions], answer).catch(error => logTrouble('settings: carrying out an answer', error)) }//every kind a line names, answered the way the system already opens them; a copy of the list, since the line's own changes as the answers land
function look() { associateLook().catch(error => logTrouble('settings: asking what opens each kind of file', error)) }

onMounted(look)
defineExpose({look})//for the panel, which hears when the window comes back into focus

</script>
<template>

<section class="relative"><!-- relative, so the card below is placed within this section and scrolls with its chip -->
	<h2 class="mb-2 text-white">File types</h2>
	<p v-if="!associateActive" class="text-amber-400 italic">{{inactiveNote}}</p><!-- amber, in italics and without the underline, being something to know rather than somewhere to go -->
	<!-- each line an amber link led by a heavy double chevron, the flag for something the user has to do; the no's then offers or and a plain button that agrees with the system for every kind the line names -->
	<p v-if="pendingYes.length > 0" class="mt-2"><a href="#" class="text-amber-400 underline" @click.prevent="finish"><b>»</b> In '<i>Set defaults by app</i>' choose {{brandName}} for {{sayList(pendingYes)}}</a></p>
	<p v-if="pendingNo.length > 0" class="mt-2 flex flex-wrap items-center gap-2"><a href="#" class="text-amber-400 underline" @click.prevent="finish"><b>»</b> In '<i>Set defaults by app</i>' choose another program for {{sayList(pendingNo)}}</a> <span>or</span> <button type="button" class="myChoice px-3" @click="settle(pendingNo, 'yes')">Choose {{brandName}}</button></p>

	<div v-for="list in fileLists" :key="list.heading + list.named" class="mt-6"><!-- the whole heading rather than the program, since the last list and Currently opens with nothing share a blank program -->
		<h3 v-if="list.heading" class="text-white">{{list.heading}}<template v-if="list.named">'<i>{{list.named}}</i>'</template></h3>
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
.myChoice {
	border: 1px solid #404040; /* the card's buttons, which the no's line at the top offers as well */
}

</style>
