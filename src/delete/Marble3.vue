<script setup>//./components/Marble3.vue - infinite panning marble background design for the full screen experience

//delete because the comment at the start says this is a disaster



/*
omg, this is a disaster
you're not even going to have a tiled marble background at the start!
nor caption text!!
but the infinite pan idea is good
so here's the new breakdown
- actually start in an empty component, Marble4.vue
- add just a big square div in the center
- copy in drag code to pan it around
then, one by one, and all maybe
- add the hud text at the bottom of the frame
- add caption text beneath the image div
- add a repeating marble background, and confirm that it rides along in the same GPU composition layer

*/









import {getCurrentWindow} from '@tauri-apps/api/window'
import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window or whole screen
const marbleRef = ref(null)//tiled marble background pans on drag holding glass and image riding along
const glassRef = ref(null)//container for centered components that ride along on top of the background
const imageRef = ref(null)//on the glass, an image

let position = {x: 0, y: 0}//where we are in the infinite pan
let drag//an object of positions and ids during a left or right click drag

onMounted(() => {

	//register listeners for input devices that are not related to position; those will go on the table in the template
	window.addEventListener('keydown', onKey)
	frameRef.value.addEventListener('wheel', onWheel, {passive: false})

	glassRef.value.style.transformOrigin = '0 0'
	glassRef.value.style.transform = `translate(${position.x}px, ${position.y}px)`

	marbleRef.value.style.transformOrigin = '0 0'
	marbleRef.value.style.transform = `translate(${position.x}px, ${position.y}px)`
})
onBeforeUnmount(() => {

	//remove the listeners; there shouldn't be a danglign pointer but release one if there is
	window.removeEventListener('keydown', onKey)
	frameRef.value.removeEventListener('wheel', onWheel)
	if (drag?.pointer) { frameRef.value.releasePointerCapture(drag.pointer); drag.pointer = null }
})

async function onKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//ignore keystrokes into a form field

	if (e.key == 'f') {
		console.log('my key F, which goes full screen')
		await setFullscreen(true)

	} else if (e.key == 'q') {
		console.log('my key Q, which quits full screen')
		await setFullscreen(false)

	} else if ((e.ctrlKey || e.metaKey) && e.key == 's') {
		e.preventDefault()//tell the browser not to show the file save dialog box
		console.log('my key Ctrl+S')

	} else if (e.key == 'Escape') {
		console.log('my key Escape, which causes both us and macos to leave full screen')
		await setFullscreen(false)//macos will also exit fullscreen, but this call doesn't mess anything up with that

	} else if (e.key == 'ArrowLeft')  { console.log('my key ArrowLeft')  }
	else if (e.key == 'ArrowRight') { console.log('my key ArrowRight') }
	else if (e.key == 'ArrowUp')    { console.log('my key ArrowUp')    }
	else if (e.key == 'ArrowDown')  { console.log('my key ArrowDown')  }
	else if (e.key == 'PageUp')     { console.log('my key PageUp')     }
	else if (e.key == 'PageDown')   { console.log('my key PageDown')   }
}
async function setFullscreen(set) { let w = getCurrentWindow(); let current = await w.isFullscreen()
	if (set != current) w.setFullscreen(set)
}
async function toggleFullscreen() { let w = getCurrentWindow(); let current = await w.isFullscreen()
	w.setFullscreen(!current)
}

function onWheel(e) {
	e.preventDefault()//tell the browser not to scroll

	if      (e.deltaY < 0) { console.log('my wheel back')    }
	else if (e.deltaY > 0) { console.log('my wheel forward') }
}//ttd august, see how this flips out on a touchpad, though

async function onDoubleClick(e) {
	await toggleFullscreen()
}

function onPointerDown(e) {

	if (e.button == 0 && e.detail == 2 && e.buttons == 1) {//primary button 0, 2nd quick click, first bit value 1 only button down right now
		console.log('pointer down: double click')
	} else if (e.button == 2 && e.detail == 2 && e.buttons == 2) {//secondary button 2, 2nd quick click, second bit value 2 only button down right now
		console.log('pointer down: right double click')
	} else {
		dragStart(e)
	}
}
function dragStart(e) {
	drag = {
		button: e.button,//0 primary or 2 secondary mouse button
		start: {x: e.clientX, y: e.clientY},//record where this drag started
		pointer: e.pointerId,
	}
	frameRef.value.setPointerCapture(e.pointerId)//watch the mouse during the drag; works even when dragged outside the window!
}
function onMove(e) { if (!drag) return

/*
//below is from previous design; we'll review before we let it out into working code!

	const segment = {//how far did the mouse move in this segment of the drag?
		x: e.clientX - drag.start.x,
		y: e.clientY - drag.start.y,
	}
	drag.start.x = e.clientX//get ready for the next segment if the drag continues
	drag.start.y = e.clientY

	const candidate = {//what would our new table position be?
		x: position.x + segment.x,
		y: position.y + segment.y,
	}
	let bound = {//what are the bounds of possible new positions, keeping the frame entirely on top of the table
		minimum: {
			x: frameRef.value.clientWidth  - tableSize,
			y: frameRef.value.clientHeight - tableSize,
		},
		maximum: {x: 0, y: 0},//don't let the table move to the right or below the upper left corner of the frame!
	}
	position.x = Math.min(bound.maximum.x, Math.max(bound.minimum.x, candidate.x))//clamp to those bounds
	position.y = Math.min(bound.maximum.y, Math.max(bound.minimum.y, candidate.y))

	glassRef.value.style.transform = `translate(${position.x}px, ${position.y}px)`//have the GPU move the table and image
*/

}
function onUp(e) {
	frameRef.value.releasePointerCapture(e.pointerId)//should be the same as drag.pointer
	drag = null//discard the drag object, getting things ready for the next drag
}

</script>
<template>

<!-- outermost frame sized to full component, which in practice is full window and full screen; styled with marble tile -->
<div
	ref="frameRef"
	class="frameClass relative w-screen h-screen overflow-hidden touch-none"
	@contextmenu.prevent
	@dblclick.prevent="onDoubleClick"
	@pointerdown="onPointerDown"
	@pointermove="onMove" @pointerup="onUp" @pointercancel="onUp" @lostpointercapture="onUp"
	@wheel.prevent="onWheel"
>

	<!-- infinite tiled marble countertop that moves locked in the same GPU layer, is our goal, to the stuff that rides along contained within it -->
	<div
		ref="marbleRef"
		class="marbleClass will-change-transform"
	>

		<!-- glass sits in the center of the pannable space, holding the image and caption -->
		<div
			ref="glassRef"
			class="glassClass will-change-transform absolute top-0 left-0"
		>

			<!-- child components that ride along on the glass, right now a div acting as a placeholder for an image, and two lines of caption text beneath it -->
			<div
				ref="imageRef"
				class="imageClass border border-cyan-500 bg-gray-300 mx-auto w-[800px] h-[600px]"
			></div>
			<div
				ref="captionRef"
				class="captionClass mt-2 text-sm text-green-800 text-center"
			>
				<p>first line of caption text</p>
				<p>second line of caption text, my hope is we can do the styles above</p>
			</div>

		</div>
	</div>
</div>

<!-- soon but not right now, we'll also want to test out HUD text that sticks to the frame, with the glass and tiled marble panning beneath it -->

</template>
<style scoped>

.frameClass {
	/* checkerboard placeholder until marble.jpg is ready */
	background-image:
	repeating-linear-gradient(45deg, #eee 0 20px, #ccc 20px 40px);
	background-size: 80px 80px;
}
.marbleClass {
	/*check and add stuff here, i think*/
}
.glassClass {
	pointer-events: none; /* let all clicks/drag fall through to the frame */
	will-change: transform; /* ensure smooth, GPU-accelerated transforms */
}

.imageClass {}
.captionClass {/* nothing yet in these two, but here in case we need to add something later */}

</style>
