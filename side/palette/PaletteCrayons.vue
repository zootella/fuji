<script setup>//a sample box of crayons picked from each of the palettes other people made: a flat set, several of a family where the palette has room for them, any few of which can be drawn beside each other and told apart. crayon.js picks each box by the same measurements and says how; this tab draws them, in light alone, on the face. At the top, every palette's box by size, with how many crayons of each family it came to. Then each box: its crayons in a column for each family, lightest at the top, each with a drawing in it at 24 and 16 css pixels, its contrast with the ground and the crayon nearest it in the box; and under them every crayon of the box in a row, each on a different drawing, so the near ones are seen side by side, as lines alone and then, as a trial, with a pale fill of the crayon's own color. It holds no color of its own: the crayons are upstream.js's, and the ground and the text are style.css's

import {upstreams} from './upstream.js'
import {crayonBox, crayonLimits, crayonFamilies} from './crayon.js'
import {barIcons} from '../../desktop/src/fuji-components/bar.js'

const pixel = Object.assign(document.createElement('canvas'), {width: 1, height: 1}).getContext('2d', {willReadFrequently: true})//one pixel to paint a color on and read back
function rgb(color) {//a color's red, green and blue as the engine paints it, whatever way css writes it, hex, hsl() or tailwind's oklch(); one outside sRGB comes back as the nearest color inside it, which is what a screen that is not wide shows
	pixel.fillStyle = color
	pixel.fillRect(0, 0, 1, 1)
	return [...pixel.getImageData(0, 0, 1, 1).data.slice(0, 3)]
}
function painted(value) {//one of fuji's own colors in light, where this element names the platform the page is on, as the engine resolves it
	let probe = document.createElement('div')
	probe.dataset.platform = document.documentElement.dataset.platform
	probe.style.colorScheme = 'light'
	probe.style.backgroundColor = value
	document.body.append(probe)
	let color = getComputedStyle(probe).backgroundColor
	probe.remove()
	return rgb(color)
}
const ground = painted('var(--color-face)'), text = painted('var(--color-text)')//what a crayon is drawn on and what it must not be taken for: the face, which the bar stands on, and the text's color, which a drawing with no color of its own is in

const boxes = upstreams.filter(p => p.colors.length).map(p => {//every palette's box, in the order upstream.js lists the palettes
	let crayons = p.colors.flatMap(([name, ...shades]) => shades.map(([shade, color]) => ({name: shade ? `${name} ${shade}` : name, color, rgb: rgb(color)})))
	let picked = crayonBox(crayons, ground, text)
	let closest = picked.box.filter(c => c.nearest).sort((x, y) => x.nearest.apart - y.nearest.apart)[0]
	return {
		palette: p,
		...picked,
		had: crayons.length,
		families: crayonFamilies.map(family => ({family, crayons: picked.box.filter(c => c.family == family)})),
		closest: closest ? `${closest.name} and ${closest.nearest.name}, ${closest.nearest.apart.toFixed(3)} apart` : '',
	}
})
const bySize = [...boxes].sort((x, y) => y.box.length - x.box.length)//the same boxes, fullest first, for the table at the top
const drawings = Object.keys(barIcons)//every drawing the bar has, which the row under a box goes through in turn
const rows = [['myBig'], [], ['myBig', 'myFilled'], ['myFilled']]//the rows under a box: the line alone at a toolbar's size and a menu's, and then each with a fill
const headings = {}//each box's heading on the page, by its palette's name, which the table scrolls to

</script>
<template>

<div>
	<h1>Crayon Boxes</h1>
	<p>A sample box of crayons picked from each palette on the Button Color tab, for the bar's drawings in light. A palette with a ladder under every hue is a grid, and what fuji wants is a box: a flat set, with several reds or blues or browns where there is room for them, uneven if the colors fall that way, from which any few crayons can be taken out and drawn beside each other, each one seen and told from the others. So the choosing is done once, here, and after it coloring a drawing is picking the second blue crayon.</p>
	<p>Every box is picked the same way, by measurement, which <span class="myFixedWidth">crayon.js</span> describes. A crayon has to stand off the face by at least {{crayonLimits.floor}} to 1, be a color and not a gray, and not be so dark that a thin line in it reads as the text's own color. From those, the box takes the crayon farthest in OKLab from the ground, the text and everything already in the box, preferring a strong color to a dull one, again and again, until every one left is nearer than {{crayonLimits.apart}} to a crayon in the box, about what separates Tailwind's blue 500 from its sky 500, or the box has {{crayonLimits.most}}. These are samples for a person to change, since a measurement has no taste: it will pass over a palette's best known red for a neighbor a hair farther from everything else. A crayon's family is this page's own rough word for it, from its hue and its lightness, a brown being an orange or a yellow gone dark. A ratio under 3 to 1, which the accessibility guidelines ask of a drawing, is marked pale.</p>
	<div class="myTable">
		<div class="myHead">Palette</div><div class="myHead myNumber">Box</div><div class="myHead myNumber">From</div>
		<div v-for="family in crayonFamilies" :key="family" class="myHead myNumber">{{family}}</div>
		<template v-for="b in bySize" :key="b.palette.name">
			<div><button class="myLink" @click="headings[b.palette.name].scrollIntoView()">{{b.palette.name}}</button></div><!-- to that palette's box, within the tab, with nothing put in the address, which the tabs keep -->
			<div class="myNumber">{{b.box.length}}</div>
			<div class="myNumber myQuiet">{{b.had}}</div>
			<div v-for="f in b.families" :key="f.family" class="myNumber myTally">
				<span v-for="c in f.crayons" :key="c.name" class="myDot" :style="{background: c.color}" :title="c.name"></span><!-- a dot for each crayon of the family, in its own color -->
			</div>
		</template>
	</div>
	<template v-for="b in boxes" :key="b.palette.name">
		<h2 :ref="el => headings[b.palette.name] = el">{{b.palette.name}}</h2>
		<p>A box of {{b.box.length}}, picked from the {{b.from}} of this palette's {{b.had}} crayons that can be drawn on this ground<template v-if="b.pale || b.gray || b.dark">; left out before the picking, {{[b.pale && `${b.pale} too pale`, b.gray && `${b.gray} too gray`, b.dark && `${b.dark} too dark`].filter(Boolean).join(', ')}}</template>. <template v-if="b.closest">Its closest two are {{b.closest}}.</template></p>
		<div class="myPanes">
			<div class="myPane myOnBar" data-mode="light">
				<div class="myBox">
					<template v-for="f in b.families" :key="f.family">
						<div v-if="f.crayons.length" class="myColumn"><!-- one family, its crayons stacked from light to dark -->
							<div class="myFamily">{{f.family}}</div>
							<div v-for="c in f.crayons" :key="c.name" class="myCrayon">
								<span class="myChip" :style="{background: c.color}"></span>
								<span class="mySample" :style="{color: c.color}"><!-- the same drawing at a toolbar's size and a menu's -->
									<span class="myIcon myBig" v-html="barIcons['folder-tree']"></span>
									<span class="myIcon" v-html="barIcons['folder-tree']"></span>
								</span>
								<span class="myName">{{c.name}}</span>
								<span class="myFixedWidth">{{c.color}}</span>
								<span :class="{myPale: c.contrast < 3}">{{c.contrast.toFixed(1)}} to 1<template v-if="c.contrast < 3">, pale</template></span>
								<span v-if="c.nearest" class="myQuiet">{{c.nearest.apart.toFixed(2)}} from {{c.nearest.name}}</span>
							</div>
						</div>
					</template>
				</div>
				<div v-for="row in rows" :key="row.join()" class="myTogether"><!-- every crayon of the box in a row, family by family, each on a different drawing, at a toolbar's size and then a menu's, so a crayon is seen beside the ones nearest it; and then both again with a pale fill of the crayon's own color under the line -->
					<span v-for="(c, i) in b.box" :key="c.name" class="myIcon" :class="row" :style="{color: c.color}" :title="c.name" v-html="barIcons[drawings[i % drawings.length]]"></span>
				</div>
			</div>
		</div>
	</template>
</div>

</template>
<style scoped>

.myTable { display: grid; grid-template-columns: max-content repeat(11, max-content); gap: 0 16px; margin-bottom: 8px; width: max-content; max-width: 100% }
.myTable > * { padding: 2px 0; border-bottom: 1px solid var(--color-border-quiet) }
.myHead { color: var(--color-text-muted); border-bottom-color: var(--color-border) }
.myNumber { text-align: right }
.myTally { display: flex; justify-content: flex-end; align-items: center; gap: 2px; min-width: 44px }
.myDot { width: 8px; height: 8px; border-radius: 50% }
.myLink { cursor: pointer; text-align: left } /* a palette's name, which scrolls to its box */
.myLink:hover { text-decoration: underline }
.myQuiet { color: var(--color-text-placeholder) }
.myOnBar { background-color: var(--color-face) } /* the face, which the bar stands on, so the crayons are seen on what they will draw on */
.myBox { display: flex; flex-wrap: wrap; gap: 16px 10px }
.myColumn { display: flex; flex-direction: column; gap: 10px } /* one family's crayons, one above the next */
.myFamily { font-weight: 600; text-align: center }
.myCrayon { display: flex; flex-direction: column; align-items: center; gap: 3px; width: 76px; font-size: 10px; line-height: 1.3; text-align: center; overflow-wrap: anywhere }
.myName { min-height: 2lh } /* room for a name of two lines under every crayon */
.myChip { width: 48px; height: 14px }
.mySample { display: flex; gap: 2px }
.myPale { color: var(--color-fuji-warning) } /* a crayon under the guidelines' 3 to 1 */
.myTogether { display: flex; flex-wrap: wrap; gap: 6px }
.myIcon { display: flex }
.myIcon :deep(svg) { width: 16px; height: 16px; stroke-width: 1.5 } /* lucide's drawing at a menu's size, its stroke a css pixel */
.myIcon.myBig :deep(svg) { width: 24px; height: 24px } /* and at a toolbar's, its stroke 1.5 in the drawing's own 24 */
.myIcon.myFilled :deep(svg) { fill: color-mix(in srgb, currentColor 25%, transparent) } /* a trial: a quarter of the crayon's color inside the drawing's shapes, as JetBrains draws a strong line over a pale fill of its hue, to see how much more color a small drawing carries with one. Lucide's drawings are lines with no fill of their own, so a shape left open, like an arrow's head, fills across its opening */

</style>
