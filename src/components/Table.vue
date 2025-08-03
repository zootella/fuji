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
	tableRef.value.addEventListener('pointerdown', myPointerDown)
	tableRef.value.addEventListener('dblclick', myDoubleClick)
//	tableRef.value.addEventListener('contextmenu', myContextMenu)
})
onBeforeUnmount(() => {

	window.removeEventListener('keydown', myKey)
	frameRef.value.removeEventListener('wheel', myWheel)

	tableRef.value.removeEventListener('pointermove', myPointerMove)//remove any remaining drag listeners
	tableRef.value.removeEventListener('pointerup', myPointerUp)
	tableRef.value.removeEventListener('pointerdown', myPointerDown)
	tableRef.value.removeEventListener('dblclick', myDoubleClick)
//	tableRef.value.removeEventListener('contextmenu', myContextMenu)
	if (draggingPointer) {
		tableRef.value.releasePointerCapture(draggingPointer)
		draggingPointer = null
	}
})

/*
function myContextMenu(e) {
	e.preventDefault()//don't show a context menu; you think tauri only shows one in development mode; chat says this won't mess things up
}
*/

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

function myDoubleClick(e) {}//not using because detecting with pointer down
function myPointerDown(e) {

  if (e.button == 2) e.preventDefault()//tell the browser not to show the context menu; tauri shows it but probably only for local development

	if (e.button == 0 && e.detail == 2 && e.buttons == 1) {//primary button 0, 2nd quick click, first bit value 1 only button down right now
		console.log('pointer down: double click')
		return
	}

	// 2) Double‐secondary click (right button, two taps)
	if (e.button == 2 && e.detail == 2 && e.buttons == 2) {//secondary button 2, 2nd quick click, second bit value 2 only button down right now
		console.log('pointer down: right double click')
		return
	}

	// 3) Chord: primary held, then secondary pressed
	if (e.button === 2 && (e.buttons & 1) === 1) {
		console.log('pointer down: primary chord')
		return
	}

	// 4) Chord: secondary held, then primary pressed
	if (e.button === 0 && (e.buttons & 2) === 2) {
		console.log('pointer down: secondary chord')
		return
	}

	//otherwise it's a drag
	draggingPointer = e.pointerId

	dragStart.x = e.clientX//remember where the drag started
	dragStart.y = e.clientY

	tableRef.value.setPointerCapture(e.pointerId)//watch the mouse
	tableRef.value.addEventListener('pointermove', myPointerMove)
	tableRef.value.addEventListener('pointerup', myPointerUp)
}
function myStartDrag(e) {

}







function myPointerMove(e) {
	const segment = {//how far did the mouse move in this segment of the drag?
		x: e.clientX - dragStart.x,
		y: e.clientY - dragStart.y,
	}
	dragStart.x = e.clientX//get ready for the next segment if the drag continues
	dragStart.y = e.clientY

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
function myPointerUp(e) {

	tableRef.value.removeEventListener('pointermove', myPointerMove)
	tableRef.value.removeEventListener('pointerup',   myPointerUp)
	tableRef.value.releasePointerCapture(e.pointerId)
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
