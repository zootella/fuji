<script setup>//./components/Table.vue - mvp lighttable for full screen

//delete because no longer using finite table

const imageSize = 900
const tableSize = 3000

import {getCurrentWindow} from '@tauri-apps/api/window'
import {ref, onMounted, onBeforeUnmount} from 'vue'

const frameRef = ref(null)//frame around boundaries of this component, likely the whole window or whole screen
const tableRef = ref(null)//background yellow on black grid the user will be able to drag to pan around
const imageRef = ref(null)//on top of that, centered, rendered to device pixels, the img tag that holds the image

let tablePosition = {x: 0, y: 0}//where the top left corner of the table is in CSS page coordinates
let drag//an object of positions and ids during a left or right click drag

onMounted(() => {

	window.addEventListener('keydown', onKey)
	frameRef.value.addEventListener('wheel', onWheel, {passive: false})

	tableRef.value.style.transformOrigin = '0 0'
	tableRef.value.style.transform = `translate(${tablePosition.x}px, ${tablePosition.y}px)`
	tableRef.value.addEventListener('pointerdown', onPointerDown)
	tableRef.value.addEventListener('dblclick', onDoubleClick)

	tableRef.value.addEventListener('contextmenu', e => e.preventDefault())//don't show a context menu; you think tauri only shows one in development mode; chat says this won't mess things up
})
onBeforeUnmount(() => {

	window.removeEventListener('keydown', onKey)
	frameRef.value.removeEventListener('wheel', onWheel)
	['pointermove', 'pointerup', 'pointerdown', 'dblclick', 'contextmenu', 'pointercancel', 'lostpointercapture'].forEach(e => tableRef.value.removeEventListener(e, listenersMap[e]))
	if (drag?.pointer) {
		tableRef.value.releasePointerCapture(drag.pointer)
		drag.pointer = null
	}
})

async function onKey(e) {
	if (e.target.tagName == 'INPUT' || e.target.tagName == 'TEXTAREA' || e.target.isContentEditable) return//ignore keystrokes into a form field

	if (e.key == 'f') {
		console.log('my key F')
		await fullscreen(true)

	} else if (e.key == 'q') {
		console.log('my key Q')
		await fullscreen(false)

	} else if ((e.ctrlKey || e.metaKey) && e.key == 's') {
		e.preventDefault()//tell the browser not to show the file save dialog box
		console.log('my key Ctrl+S')

	} else if (e.key == 'Escape') {
		console.log('my key Escape')
		await fullscreen(false)//macos will also exit fullscreen, but this call doesn't mess anything up with that

	} else if (e.key == 'ArrowLeft')  { console.log('my key ArrowLeft')  }
	else if (e.key == 'ArrowRight') { console.log('my key ArrowRight') }
	else if (e.key == 'ArrowUp')    { console.log('my key ArrowUp')    }
	else if (e.key == 'ArrowDown')  { console.log('my key ArrowDown')  }
	else if (e.key == 'PageUp')     { console.log('my key PageUp')     }
	else if (e.key == 'PageDown')   { console.log('my key PageDown')   }
}
async function fullscreen(set) {
	let w = getCurrentWindow()
	let current = await w.isFullscreen()
	if (set != current) {
		console.log(`setting fullscreen ${set}`)
		w.setFullscreen(set)
	}
}

function onWheel(e) {
	e.preventDefault()//tell the browser not to scroll

	if (e.deltaY < 0) {
		console.log('my wheel back')
	} else if (e.deltaY > 0) {
		console.log('my wheel forward')
	}
}//ttd august, see how this flips out on a touchpad, though

function onDoubleClick(e) {
	console.log('double click event')
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
		start: {x: e.clientX, y: e.clientY},//record where this drag started
		pointer: e.pointerId,
		button: e.button,//0 primary or 2 secondary mouse button
	}

	tableRef.value.setPointerCapture(e.pointerId)//watch the mouse
	tableRef.value.addEventListener('pointermove',        onMove)
	tableRef.value.addEventListener('pointerup',          onUp)
	tableRef.value.addEventListener('pointercancel',      onUp)
	tableRef.value.addEventListener('lostpointercapture', onUp)
}
function onMove(e) {
	if (!drag) return

	const segment = {//how far did the mouse move in this segment of the drag?
		x: e.clientX - drag.start.x,
		y: e.clientY - drag.start.y,
	}
	drag.start.x = e.clientX//get ready for the next segment if the drag continues
	drag.start.y = e.clientY

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
function onUp(e) {
	if (!drag) return

	tableRef.value.removeEventListener('pointermove',        onMove)
	tableRef.value.removeEventListener('pointerup',          onUp)
	tableRef.value.removeEventListener('pointercancel',      onUp)
	tableRef.value.removeEventListener('lostpointercapture', onUp)
	tableRef.value.releasePointerCapture(e.pointerId)//should be the same as drag.pointer

	drag = null//discard the drag object
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
