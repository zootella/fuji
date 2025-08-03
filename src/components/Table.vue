<script setup>//./components/Table.vue - mvp lighttable for full screen

import {getCurrentWindow} from '@tauri-apps/api/window'

const imageSize = 900
const tableSize = 3000

import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window or whole screen
const tableRef = ref(null)//background yellow on black grid the user will be able to drag to pan around
const imageRef = ref(null)//on top of that, centered, rendered to device pixels, the img tag that holds the image

let tablePosition = {x: 0, y: 0}
let dragStart = {x: 0, y: 0}
let draggingPointer = null

onMounted(() => {

	window.addEventListener('keydown', myKey)
	frameRef.value.addEventListener('wheel', myWheel, {passive: false})

	tableRef.value.style.transformOrigin = '0 0'
	tableRef.value.style.transform = `translate(${tablePosition.x}px, ${tablePosition.y}px)`
	tableRef.value.addEventListener('pointerdown', panDown)//start listening for drags
})
onBeforeUnmount(() => {

	window.removeEventListener('keydown', myKey)
	frameRef.value.removeEventListener('wheel', myWheel)

	tableRef.value.removeEventListener('pointermove', panMove)//remove any remaining drag listeners
	tableRef.value.removeEventListener('pointerup', panUp)
	tableRef.value.removeEventListener('pointerdown', panDown)
	if (draggingPointer) {
		tableRef.value.releasePointerCapture(draggingPointer)
		draggingPointer = null
	}
})

async function myKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//ignore keystrokes into a form field

	if (e.key == 'f') {
		console.log('my key F')
		await myFullscreen(true)

	} else if (e.key == 'q') {
		console.log('my key Q')
		await myFullscreen(false)

	} else if ((e.ctrlKey || e.metaKey) && e.key == 's') {
		e.preventDefault()//tell the browser not to show the file save dialog box
		console.log('my key Ctrl+S')

	} else if (e.key == 'Escape') {
		console.log('my key Escape')
		await myFullscreen(false)//macos will also exit fullscreen, but this call doesn't mess anything up with that

	} else if (e.key == 'ArrowLeft')  { console.log('my key ArrowLeft')  }
	else if (e.key == 'ArrowRight') { console.log('my key ArrowRight') }
	else if (e.key == 'ArrowUp')    { console.log('my key ArrowUp')    }
	else if (e.key == 'ArrowDown')  { console.log('my key ArrowDown')  }
	else if (e.key == 'PageUp')     { console.log('my key PageUp')     }
	else if (e.key == 'PageDown')   { console.log('my key PageDown')   }
}
async function myFullscreen(set) {
	let w = getCurrentWindow()
	let current = await w.isFullscreen()
	if (set != current) {
		console.log(`setting fullscreen ${set}`)
		w.setFullscreen(set)
	}
}
function myWheel(e) {
	e.preventDefault()//tell the browser not to scroll

	if (e.deltaY < 0) {
		console.log('my wheel back')
	} else if (e.deltaY > 0) {
		console.log('my wheel forward')
	}
}//ttd august, see how this flips out on a touchpad, though

function panDown(panEvent) {
	draggingPointer = panEvent.pointerId

	dragStart.x = panEvent.clientX//remember where the drag started
	dragStart.y = panEvent.clientY

	tableRef.value.setPointerCapture(panEvent.pointerId)//watch the mouse
	tableRef.value.addEventListener('pointermove', panMove)
	tableRef.value.addEventListener('pointerup', panUp)
}
function panMove(panEvent) {
	const segment = {//how far did the mouse move in this segment of the drag?
		x: panEvent.clientX - dragStart.x,
		y: panEvent.clientY - dragStart.y,
	}
	dragStart.x = panEvent.clientX//get ready for the next segment if the drag continues
	dragStart.y = panEvent.clientY

	const candidate = {//what would our new table position be?
		x: tablePosition.x + segment.x,
		y: tablePosition.y + segment.y,
	}
	let bound = {//what are the bounds of possible new positions, keeping the frame entirely on top of the table
		minimum: {
			x: frameRef.value.clientWidth  - tableSize,
			y: frameRef.value.clientHeight - tableSize,
		},
		maximum: {x: 0, y: 0},//don't let the table move to the right or below the upper left corner of the frame!
	}
	tablePosition.x = Math.min(bound.maximum.x, Math.max(bound.minimum.x, candidate.x))//clamp to those bounds
	tablePosition.y = Math.min(bound.maximum.y, Math.max(bound.minimum.y, candidate.y))

	tableRef.value.style.transform = `translate(${tablePosition.x}px, ${tablePosition.y}px)`//push to the GPU
}
function panUp(panEvent) {

	tableRef.value.removeEventListener('pointermove', panMove)
	tableRef.value.removeEventListener('pointerup',   panUp)
	tableRef.value.releasePointerCapture(panEvent.pointerId)
	draggingPointer = null
}

</script>
<template>

<!-- outer div frame sized to component, often the tauri window renderer viewport, frequently the full screen -->
<div ref="frameRef" class="relative w-screen h-screen overflow-hidden myFrame">

	<!-- inner div tabletop the user drags around to pan, always larger than the frame -->
	<div
		ref="tableRef"
		class="bg-black myTable"
		:style="{width: tableSize + 'px', height: tableSize + 'px'}"
	>

		<!-- the img tag will go here, where we expect it will "ride along" with the drag to pan tabletop -->

	</div>

	<!-- hud text will go here, inside the frame, alongside the tabletop -->

</div>

</template>
<style scoped>

.myTable {
	will-change: transform; /* pan with the compositor thread; no layout or paint; buttery-smooth 🧈 */
	touch-action: none; /* tell the browser to not try to scroll or zoom behind a drag that we are handling */

	/* temporary grid lines so we can see panning */
	background-image:
		linear-gradient(to right, yellow 1px, transparent 1px),
		linear-gradient(to bottom, yellow 1px, transparent 1px);
	background-size: 500px 500px;
}

.myHud {
	pointer-events: none; /* the HUD is information, only--no buttons, no selectable text */
}

</style>
