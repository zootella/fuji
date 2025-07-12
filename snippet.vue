





<script setup>

/*
Fuji, a image viewer and file manager made with Tauri, Vue with Composition API, and Tailwind CSS

design sketch for how the viewer gets the desired change
- without delay, because we pre-rendered it, and
- without a black flicker, because we do all parts of the swap in the same animation frame
*/

//our reference to the one canvas which is visible right now
const canvasActive = ref(null)

 //references to the 6 possible canvases the user could press a key to switch to
const canvasRollBack = ref(null)
const canvasRollForward = ref(null)
const canvasZoomIn = ref(null)
const canvasZoomOut = ref(null)
const canvasGammaMore = ref(null)
const canvasGammaLess = ref(null)

//outside this discussion is this render() function
//it always makes a new canvas of the correct size based on the image, zoom, and gamma we tell it
async function render(/*...*/) {
	let canvas
	//...read bytes from a file, make a new canvas object, render the image into canvas pixels

	//default styles for our new canvas
	canvas.style.position = 'absolute'
	canvas.style.left = '50%'
	canvas.style.top = '50%'
	canvas.style.transform = 'translate(-50%, -50%)'
	canvas.style.opacity = '0'
	canvas.style.zIndex = '10'
	canvas.style.pointerEvents = 'none'
	canvas.style.width = `${canvas.width / devicePixelRatio}px`//match CSS size to physical canvas resolution
	canvas.style.height = `${canvas.height / devicePixelRatio}px`
	return canvas
}

let preparing = false//true while we are prerendering and caching all the next possible images

//our code calls this prepare() function right away as the user is looking at tiles.active
//its job is to get the other 6 references ready so whatever the user clicks next, there's no delay
//to do this, it might just move references to existing canvases around, or call render() to make new ones we don't have yet
async function prepare() {//(imagine this function runs right away will the user is looking at tles.active)
	preparing = true//make a note that we're getting things ready, so we should delay the user's flip
	//this should probably be await and a promise or something

	let active = tiles.active//save the current active reference

	//example to move forward
	canvasRollBack.value = active
	canvasActive.value = tiles.rollForward
	//wait as soon as i do this, won't the page change?!

	canvas.rollForward = await createCanvas(/*...*/)//load the next image into a new canvas
	//here, we also need to position the new canvas on the page. we can get the canvas' dimensions

const cssWidth = canvas.width / devicePixelRatio
const cssHeight = canvas.height / devicePixelRatio

//and then set is location
	tiles.rollForward.style.left = 500
	tiles.rollForward.style.

	//is this correct? clean this up, please


	//if the user switches to it, we'll swap the visibility, without needing to move or size anything
	//we are careful to never stretch a canvas--they're always 1:1 on the page

	//example to zoom in
	tiles.zoomOut = active
	tiles.active = tiles.zoomIn
	tiles.zoomIn = await createCanvas(/*...*/)

	//in here, we also need to move the hidden canvases around, so that when we switch, we're just making them visible or invisible

	preparing = false//everythign is ready for all 6 of the user's next possible moves, so a flip can be instantaneous now
}

async function move(to) {//to is like "RollBack", "RollForward", "ZoomIn", "ZoomOut", "GammaMore", or "GammaLess"
	/*
	before flipping, we need to wait for a current prepare task to finish

	after flipping, we need to start a new prepare task to get ready for the user's next move
	*/

	//the idea here is that since we hide the active canvas and set one of the 6 preloaded ones visible, there will be no black blink between the images at all!
  requestAnimationFrame(() => {
    const next = tiles[role]
    const current = tiles.active

    // Hide the current canvas
    current.style.visibility = 'hidden'

    // Show the target canvas
    next.style.visibility = 'visible'

    // Update reference
    tiles.active = next
  })
}


</script>
<template>
  <div id="canvas-stack" class="relative w-full h-full overflow-hidden">
    <!-- Visible active canvas -->
    <canvas
      ref="canvasActive"
      class="absolute inset-0 z-50 pointer-events-none opacity-100"
    />

    <!-- Hidden background canvases for preloading -->
    <canvas ref="canvasRollBack"     class="invisible absolute" />
    <canvas ref="canvasRollForward"  class="invisible absolute" />
    <canvas ref="canvasZoomIn"       class="invisible absolute" />
    <canvas ref="canvasZoomOut"      class="invisible absolute" />
    <canvas ref="canvasGammaMore"    class="invisible absolute" />
    <canvas ref="canvasGammaLess"    class="invisible absolute" />
  </div>
</template>
<style scoped>


</style>














async function doPreload() {
    ...
    //while the user is looking at active, code in the background makes the other 6 to be ready
    tiles.rollForward = createCanvas(/*arguments to render the next image*/)
    //note that createCanvas always makes a new canvas, exactly one time for each call
    ...
}

async function doSwap(direction) {//pass in a direction like "Back", "Forward", "In", "Out", and so on

    //ok, show me this

}





























