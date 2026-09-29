<script setup>//the card for one kind of file in fuji's settings: what the extension is and where it came from, where it stands, and the button that changes it, shown beside its chip

import {ref, computed, watch, onMounted, nextTick} from 'vue'
import {associateAnswers, associateOpens, associateActive, associateChoose, associateDiffers, associateProgram, associateFinish} from '../associate.js'
import {logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {fileTypes} from '../fileTypes.js'

/*
The section decides which card is up and whether it is pinned; the card is everything about one kind of file, and how it sits on the page.

It is headed with the extension and its title, the kind of file it commonly holds, naming what the format can do rather than what it cannot, then a few sentences of its history, then one line holding where the kind stands and what the user can do about it, side by side and worded so neither reads as the other. A kind fuji does not open yet has no such line, since its card is there to be read and nothing about the kind can change, and the heading its chip sits under already says it is coming. The status is the system's: Opens with Fuji, or Currently opens with the program that does, its name in quotes and italics as in the section's headings, worded so the button beside it reads as the way to change that. A disagreement is an answer the system has not carried out, and only its own settings can change a choice the user saved there, so the status turns into the amber link that opens them: after a yes the system opens with something else, and it says » In 'Set defaults by app' choose Fuji, the words the section shows at its top; after a no it still opens with fuji, and it says only » Choose another program, since the kind already sits under Opens with Fuji. The no's link is followed by or and a plain Choose Fuji that takes the no back, because the card cannot know how the user reached it: a no given a minute ago and about to be finished, one given a day ago and forgotten, or one from long ago with fuji chosen in the system since. Offering the way forward and the way back is right for all three, the user knows which they meant, and Choose Fuji changes nothing that opens, since the system already opens the kind with fuji. The yes's link has no such pair, since fuji is what the user asked for, and No longer open with Fuji beside it would read as though fuji opened the kind now; Change sits beside it as it does anywhere else. The button follows the answer. A kind answered yes offers Change, and only that click offers No longer open with Fuji beside Cancel, so taking a kind away from fuji is a deliberate second step; any other kind offers Choose Fuji. Ask is the third answer under the hood, where every kind starts and what lets fuji follow a choice made in the system, and it is never a button, since it would only be a softer no that turns into yes wherever the system already opens that kind with fuji.

The card is positioned inside the page rather than over the window, absolute within the section, so it scrolls with its chip, and nothing has to cover the window to catch a click elsewhere, which would take the first click on another chip for itself. It sits under its chip where the window has room and above where it does not, and never past the window's right edge; it measures itself once it is on the page, and vue redraws it at the corrected place before the browser paints. Pinned, it takes the focus, which is how it hears esc with fuji's one window listener left in the shell, and how it knows about a click anywhere else: focus leaves it, and it asks to close. A press inside keeps the focus where it is, since the Mac's engine gives a clicked button no focus of its own and a press on one would otherwise read as leaving.
*/

const props = defineProps({
	extension: {type: String, required: true},//the kind of file this card is about, with its dot
	anchor: {type: Object, required: true},//its chip, which the card sits beside
	pinned: {type: Boolean, default: false},//clicked rather than hovered: the card holds the focus, and losing it closes the card
})
const emit = defineEmits(['close'])

const coming = computed(() => !fileTypes[props.extension].enabled)//a kind fuji does not open yet, whose card is its heading and history and nothing more
const pending = computed(() => associateDiffers(props.extension) ? associateAnswers.value[props.extension] : '')//yes or no when the system has not carried the answer out, which only its own settings can, so the status is the link there rather than words; blank where the two agree
const standing = computed(() => {//where the kind stands, as {text, named}: the words of the status, and another program's name to set in quotes and italics after them, blank for fuji and for nothing; text blank before the system has been asked
	let extension = props.extension
	if (!associateOpens.value[extension]) return {text: '', named: ''}
	let name = associateProgram(extension)
	if (name == brandName) return {text: `Opens with ${name}`, named: ''}//fuji, said plainly as the section's heading says it, whichever copy of fuji it is: seen from a development build, the installed copy is fuji too
	if (!name) return {text: 'Currently opens with nothing', named: ''}
	return {text: 'Currently opens with ', named: name}//currently, so the Choose Fuji beside it reads as what changes that
})

const changing = ref(false)//the user clicked Change, which offers the no and a way back
watch(() => props.extension, () => { changing.value = false })//moved to another chip, so it starts over

function finish() { associateFinish().catch(error => logTrouble('settings: opening windows settings', error)) }//the same link the section shows at its top
function choose(answer) {//yes or no, after which the card goes, since the chip shows the result
	emit('close')
	associateChoose([props.extension], answer).catch(error => logTrouble('settings: carrying out an answer', error))
}

const box = ref(null)
const place = ref({})//left and top, in css pixels from the corner of the section the card is positioned within
function measure() {
	let a = props.anchor.getBoundingClientRect()//this and the next are from the window's corner, and place is from the section's, hence the difference below
	let within = box.value.offsetParent.getBoundingClientRect()
	let left = Math.max(8, Math.min(a.left, window.innerWidth - box.value.offsetWidth - 8))
	let top = a.bottom + 4
	if (top + box.value.offsetHeight > window.innerHeight - 8) top = a.top - 4 - box.value.offsetHeight//no room below, so above
	place.value = {left: `${left - within.left}px`, top: `${top - within.top}px`}
}
function hold() { if (props.pinned) box.value.focus() }
function onFocusOut(e) { if (props.pinned && !box.value.contains(e.relatedTarget)) emit('close') }

onMounted(() => { measure(); hold() })
watch(() => props.anchor, () => nextTick(measure))//moved to another chip, once its contents are on the page
watch(() => props.pinned, hold)//a hovered card, clicked and now held

</script>
<template>

<div ref="box" tabindex="-1" class="myCard absolute w-md p-3 outline-none" :style="place" @focusout="onFocusOut" @keydown.esc="emit('close')" @mousedown.prevent>
	<p><span class="text-white">{{extension}}</span> - <i>{{fileTypes[extension].title}}</i></p><!-- the hyphen and the title in the card's own gray; the title rather than the type windows prints, which is there for explorer's type column and says less -->
	<p class="mt-2">{{fileTypes[extension].about}}</p>
	<div v-if="!coming && (associateActive || standing.text)" class="mt-3 flex flex-wrap items-center gap-2"><!-- a kind coming soon has no status and no button, so the card ends with its history, and so does a card on a copy that cannot act before the system has been asked -->
		<!-- the status is the link where only the system can carry the answer out, led by the chevron that leads it at the top of the section -->
		<a v-if="pending == 'yes'" href="#" class="mr-2 text-amber-400 underline" @click.prevent="finish"><b>»</b> In '<i>Set defaults by app</i>' choose {{brandName}}</a>
		<a v-else-if="pending == 'no'" href="#" class="text-amber-400 underline" @click.prevent="finish"><b>»</b> Choose another program</a><!-- fewer words than the yes, since the kind sits under Opens with Fuji and the card is about it alone -->
		<span v-else-if="standing.text" class="mr-2 text-white">{{standing.text}}<template v-if="standing.named">'<i>{{standing.named}}</i>'</template></span>
		<template v-if="associateActive"><!-- a copy that cannot act shows where the kind stands and offers nothing, since a grayed button reads as broken rather than unavailable, and the section's amber note says why -->
			<span v-if="pending == 'no'">or</span><!-- the link goes forward and the plain Choose Fuji after it takes the no back, both offered because the card cannot know which the user means -->
			<button v-if="associateAnswers[extension] != 'yes'" type="button" class="myChoice px-3" @click="choose('yes')">Choose {{brandName}}</button>
			<button v-else-if="!changing" type="button" class="myChoice px-3" @click="changing = true">Change</button>
			<template v-else>
				<button type="button" class="myChoice px-3" @click="choose('no')">No longer open with {{brandName}}</button>
				<button type="button" class="myChoice px-3" @click="changing = false">Cancel</button>
			</template>
		</template>
	</div>
</div>

</template>
<style scoped>

.myCard {
	background-color: #171717;
	border: 1px solid #525252;
	box-shadow: 0 4px 16px rgba(0, 0, 0, 0.6);
}
.myChoice {
	border: 1px solid #404040;
}

</style>
