




// Track our hide timer and delay
let hideTimer = null
const HIDE_DELAY = 200  // milliseconds
// Show the cursor and restart the hide timer
function showCursor() {
	frameRef.value.classList.remove('cursor-hidden')
	clearTimeout(hideTimer)
	hideTimer = setTimeout(() => {
		frameRef.value.classList.add('cursor-hidden')
	}, HIDE_DELAY)
}
// Hide the cursor immediately
function hideCursor() {
	clearTimeout(hideTimer)
	frameRef.value.classList.add('cursor-hidden')
}









/*
factor this into one tight little area
one function
one object of state

command:

mount
unmount - component mounted and on before unmounted

enable
disable - turn on the disappearing behavior

move
up
down - things we've seen the mouse do in this component

*/

const pointerState = {//single object of factory settings and current state
	delay: 200,
	moved: 0,//tick when we last saw the pointer move
	dragging: false,
	visible: true,

}
function pointerReport(o) {//single control function for one line calls elsewhere in this component

	if        (o.command == 'mount') {
	} else if (o.command == 'unmount') {

	} else if (o.command == 'enable') {
	} else if (o.command == 'disable') {

	} else if (o.command == 'down') {
	} else if (o.command == 'up') {
	} else if (o.command == 'move') {
	}
}
function pointerFrame() {//single function that, with behavior enabled, runs quickly on every animation frame

}



const HIDE_DELAY = 200  // ms threshold

//–– State for our loop
let lastMoveTime    = performance.now()
let isDragging      = false
let isCursorHidden  = false
let rafId           = null

function showCursor(makeVisible, givenLastMoveTime, givenDetails) {
	if (givenLastMoveTime) lastMoveTime = givenLastMoveTime
	if (givenDetails) isDragging = givenDetails.dragging

	if (makeVisible && isCursorHidden) {//show
		isCursorHidden = false
		frameRef.value.classList.remove('cursor-hidden')
	} else if (!makeVisible && !isCursorHidden) {//hide
		isCursorHidden = true
		frameRef.value.classList.add('cursor-hidden')
	}
}

function trackPointerMove() {
  lastMoveTime = performance.now()
}

function panDown(e) {
  showCursor(false, performance.now(), {dragging: true})
}

function panMove(e) {
  lastMoveTime = performance.now()
}

function panUp(e) {
  showCursor(true, performance.now(), {dragging: false})
}

function cursorLoop(now) {
  if (isDragging) {
    showCursor(false)
  } else {
    if (HIDE_DELAY + lastMoveTime < now) {
      showCursor(false)
    } else {
      showCursor(true)
    }
  }
  rafId = requestAnimationFrame(cursorLoop)
}

onMounted(() => {
  const frame = frameRef.value
  // start tracking moves
  frame.addEventListener('pointermove', trackPointerMove)
  // start the loop
  rafId = requestAnimationFrame(cursorLoop)
})

onBeforeUnmount(() => {
  const frame = frameRef.value
  frame.removeEventListener('pointermove', trackPointerMove)
  cancelAnimationFrame(rafId)
})
</script>



























