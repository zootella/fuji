<script setup>//the card for one kind of file in fuji's settings: what the extension is and where it came from, where it stands, and the button that changes it, shown beside its chip

import {ref, computed, watch, onMounted, nextTick} from 'vue'
import {associateAnswers, associateOpens, associateActive, associateChoose, associateOurs, associateDiffers, associateProgram} from '../associate.js'
import {logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {imageTypes} from './library.js'

/*
The section decides which card is up and whether it is pinned; the card is everything about one kind of file, and how it sits on the page.

It is titled with the extension and the kind of file it commonly holds, naming what the format can do rather than what it cannot, then a few sentences of its history, then one line holding where the kind stands and what the user can do about it, side by side and worded so neither reads as the other. The status is the system's: a check and Opens with Fuji, or Opens with the program that does. A disagreement turns it into an amber caution, and every caution leads toward fuji. After a yes the system opens with something else, which only its own settings can change, so the caution says Select in Windows Settings, pointing at the one link at the top of the section. After a no the system still opens with fuji, and the easy way to settle that is here: the caution stands alone beside an amber Choose Fuji. Leading one way is still polite, because the other way stays one clear path: a user moving a kind from fuji to another program chooses that program in the system's settings and answers no here, and nothing disagrees. The button follows the answer. A kind answered yes offers Change, and only that click offers No longer open with Fuji beside Cancel, so taking a kind away from fuji is a deliberate second step; any other kind offers Choose Fuji. Ask is the third answer under the hood, where every kind starts and what lets fuji follow a choice made in the system, and it is never a button, since it would only be a softer no that turns into yes wherever the system already opens that kind with fuji.

The card is positioned inside the page rather than over the window, absolute within the section, so it scrolls with its chip, and nothing has to cover the window to catch a click elsewhere, which would take the first click on another chip for itself. It sits under its chip where the window has room and above where it does not, and never past the window's right edge; it measures itself once it is on the page, and vue redraws it at the corrected place before the browser paints. Pinned, it takes the focus, which is how it hears esc with fuji's one window listener left in the shell, and how it knows about a click anywhere else: focus leaves it, and it asks to close. A press inside keeps the focus where it is, since the Mac's engine gives a clicked button no focus of its own and a press on one would otherwise read as leaving.
*/

const props = defineProps({
	extension: {type: String, required: true},//the kind of file this card is about, with its dot
	anchor: {type: Object, required: true},//its chip, which the card sits beside
	pinned: {type: Boolean, default: false},//clicked rather than hovered: the card holds the focus, and losing it closes the card
})
const emit = defineEmits(['close'])

const caution = '⚠︎'//a warning sign, then an invisible variation selector asking for its text form, so it takes the amber around it rather than an emoji font's own yellow; keep the selector when editing this line
const standing = computed(() => {//where the kind stands, in the words of the status; blank before the system has been asked
	let extension = props.extension
	if (!associateOpens.value[extension]) return ''
	if (associateDiffers(extension) && associateAnswers.value[extension] == 'yes') return `${caution} Select in Windows Settings`//only the system can settle this one
	if (associateDiffers(extension)) return caution//a no the system does not share: the sign alone, beside the Choose Fuji button that settles it here
	let name = associateProgram(extension)
	if (associateOurs(extension)) return `✓ Opens with ${name}`//a plain check mark, U+2713, drawn in the card's own type rather than as a colored emoji
	if (name) return `Opens with ${name}`
	return 'Opens with nothing'
})

const changing = ref(false)//the user clicked Change, which offers the no and a way back
watch(() => props.extension, () => { changing.value = false })//moved to another chip, so it starts over

function choose(answer) {//yes or no, after which the card goes, since the chip shows the result
	emit('close')
	associateChoose(props.extension, answer).catch(error => logTrouble('settings: carrying out an answer', error))
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
	<p><span class="text-white">{{extension}}</span> <span class="ml-1 text-neutral-400">{{imageTypes[extension].kind}}</span></p><!-- the kind rather than the name windows prints, which is there for explorer's type column and says less -->
	<p class="mt-2">{{imageTypes[extension].about}}</p>
	<div class="mt-3 flex flex-wrap items-center gap-2">
		<span v-if="standing" :class="associateDiffers(extension) ? 'text-amber-400' : 'text-white'" class="mr-2">{{standing}}</span><!-- amber on a disagreement, like the chip and the one link that settles it -->
		<button v-if="associateAnswers[extension] != 'yes'" type="button" :disabled="!associateActive" :class="associateDiffers(extension) ? 'myAttend' : 'myChoice'" class="px-3" @click="choose('yes')">Choose {{brandName}}</button><!-- amber when it is the fix: a no the system does not share, settled here in fuji's favor rather than sent to the system's settings -->
		<button v-else-if="!changing" type="button" :disabled="!associateActive" class="myChoice px-3" @click="changing = true">Change</button>
		<template v-else>
			<button type="button" class="myChoice px-3" @click="choose('no')">No longer open with {{brandName}}</button>
			<button type="button" class="myChoice px-3" @click="changing = false">Cancel</button>
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
.myAttend {
	color: var(--color-amber-400); /* the amber of the chip it is about */
	border: 1px solid var(--color-amber-400);
}
.myChoice:disabled, .myAttend:disabled {
	opacity: 0.5; /* a copy that cannot act still opens every card to read, and offers no answer */
}

</style>
