


onMounted(() => {

	//register listeners for input devices that are not related to position; those will go on the table in the template
	window.addEventListener('keydown', onKey)
	frameRef.value.addEventListener('wheel', onWheel, {passive: false})

	cardRef.value.style.transformOrigin = '0 0'
	cardRef.value.style.transform = `translate(${spacePosition.x}px, ${spacePosition.y}px)`
})
onBeforeUnmount(() => {

	//remove the listeners; there shouldn't be a danglign pointer but release one if there is
	window.removeEventListener('keydown', onKey)
	frameRef.value.removeEventListener('wheel', onWheel)
	if (drag?.pointer) { cardRef.value.releasePointerCapture(drag.pointer); drag.pointer = null }
})






















/*
[]zoom with mouse wheel
[]hud
[]marble tile
[]shadows on image
[~]continuity with window move, resize, fullscreen; do that last (and, of course, you do it first. and, of course, it doesn't work and takes most of the middle of the day)


*/





