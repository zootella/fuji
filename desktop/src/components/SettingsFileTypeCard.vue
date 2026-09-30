<script setup>//the card for one kind of file in fuji's settings, beside its chip: what it is and where it came from, where it stands, and the button that changes it

import {ref, computed, watch, onMounted, nextTick} from 'vue'
import {associateAnswers, associateActive, associateChoose, associateDiffers, associateFinish} from '../associate.js'
import {logTrouble} from '../log.js'
import {brandName} from '../brand.js'
import {fileTypes} from '../fileTypes.js'

/*
The section decides which card is up and whether it is pinned; the card is everything about one kind of file, and how it sits on the page.

It is headed with the extension and its title, then a few sentences of history, both from fileTypes.js, then one line holding where the kind stands and the button that changes it, worded so neither reads as the other. A kind coming soon has no such line. The status is the heading of the list the chip sits in, handed down so the words are the section's own. Where the system has not carried out an answer, the status becomes the link to its settings: » In 'Set defaults by app' choose Fuji after a yes, and after a no only » Choose another program, since the kind already sits under Opens with Fuji; a no's link has or and a plain Choose Fuji after it, for the reasons the section gives. Otherwise the button follows the answer. A yes offers Change, and only that click offers No longer open with Fuji beside Cancel, so taking a kind from fuji is a deliberate second step; anything else offers Choose Fuji. Ask is never a button, since it would only be a softer no that turns into yes wherever the system already opens the kind with fuji.

The card sits inside the page rather than over the window, absolute within the section, so it scrolls with its chip, and nothing has to cover the window to catch a click elsewhere, which would take the first click on another chip for itself. It goes under its chip where the window has room and above where it does not, never past the right edge; it measures itself once it is on the page, and vue redraws it in place before the browser paints. Pinned, it takes the focus, which is how it hears esc with fuji's one window listener left in the shell, and how it knows about a click anywhere else: focus leaves, and it asks to close. A press inside keeps the focus where it is, since the Mac's engine gives a clicked button no focus of its own, and a press on one would otherwise read as leaving.
*/

const props = defineProps({
	extension: {type: String, required: true},//the kind of file this card is about, with its dot
	list: {type: Object, required: true},//the section's list its chip sits in, whose heading and named are the card's status, blank before the system has been asked
	anchor: {type: Object, required: true},//its chip, which the card sits beside
	pinned: {type: Boolean, default: false},//clicked rather than hovered: the card holds the focus, and losing it closes the card
})
const emit = defineEmits(['close'])

const coming = computed(() => !fileTypes[props.extension].enabled)//a kind fuji does not open yet, whose card is its heading and history and nothing more
const pending = computed(() => associateDiffers(props.extension) ? associateAnswers.value[props.extension] : '')//yes or no where the system has not carried the answer out, which turns the status into a link to its settings; blank where they agree

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

<div ref="box" tabindex="-1" class="myCard absolute w-112 p-3 outline-none" :style="place" @focusout="onFocusOut" @keydown.esc="emit('close')" @mousedown.prevent>
	<p><span class="text-strong">{{extension}}</span> - <i>{{fileTypes[extension].title}}</i></p><!-- the title, not the type windows prints in explorer's type column, which says less -->
	<p class="mt-2">{{fileTypes[extension].about}}</p>
	<div v-if="!coming && (associateActive || list.heading)" class="mt-3 flex flex-wrap items-center gap-2"><!-- no row for a kind coming soon, nor on a copy that cannot act before the system has been asked -->
		<!-- the status is the link where only the system can carry the answer out, led by the chevron that leads it at the top of the section -->
		<a v-if="pending == 'yes'" href="#" class="mr-2 text-warn underline" @click.prevent="finish"><b>»</b> In '<i>Set defaults by app</i>' choose {{brandName}}</a>
		<a v-else-if="pending == 'no'" href="#" class="text-warn underline" @click.prevent="finish"><b>»</b> Choose another program</a><!-- fewer words than the yes, since the kind sits under Opens with Fuji and the card is about it alone -->
		<span v-else-if="list.heading" class="mr-2 text-strong">{{list.heading}}<template v-if="list.named">'<i>{{list.named}}</i>'</template></span>
		<template v-if="associateActive"><!-- a copy that cannot act offers no buttons, since a grayed one reads as broken, and the section's amber note says why -->
			<span v-if="pending == 'no'">or</span><!-- the link finishes the no and Choose Fuji takes it back, since the card cannot know which the user means -->
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
	background-color: var(--color-surface);
	border: 1px solid var(--color-edge);
	box-shadow: 0 4px 16px var(--color-shade);
}
.myChoice {
	border: 1px solid var(--color-line);
}

</style>
