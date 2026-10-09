import {ref} from 'vue'

/*
Whether the light table shows a picture's own pixels as blocks, or smoothed, which is how it shows every picture unless asked. Off, the table draws exactly as it always has: the engine resamples the decoded image into the card with its own filter, which is right for a photograph at any size. On, the card's image asks the engine for nearest neighbor through one CSS property, image-rendering: pixelated, so at a whole number of backing pixels per image pixel every image pixel becomes a block of its own color, a palette stays a palette and a dither stays a dither; a GIF from 1996 at the 2 key on an ordinary monitor is four backing pixels per image pixel, each the pixel's own color. At a ratio that is not whole, Chromium and the Mac's WebKit both still draw plain nearest neighbor, every backing pixel the color of one image pixel, so the blocks stay pure but are not all one size: on an ordinary monitor one notch past the 2 key is squares of three with a square of two every sixteenth. Below one to one the same filter drops image pixels rather than averaging them. The Ben Day page on fuji's site works through each ratio.

The r key turns it on and off, on the table, where the i key is, rather than in the shell where g is: gamma is a way of looking at every view at once, and this is a way of looking at the card. Nothing on the contact sheet reads it, and nothing in the thumbnail pipeline can: the one rule that applies it is scoped to the card in DiamondTable.vue. Like gamma it is written nowhere, so fuji starts with it off on every launch. Unlike gamma it needs only one rule: the flip changes the property's own value, which every engine redraws for, where the gamma lens changes a filter underneath a value that stays the same, and so takes turns between two in Shell.vue.

It lives in a module of its own, like gamma.js, so the table that draws it and anything that one day steps it need not reach into each other for it.
*/

export const raster = ref(false)//false is the engine's own smoothing, exactly as the table has always drawn

export function rasterToggle() {//the r key: blocks from smooth, and smooth from blocks
	raster.value = !raster.value
}
