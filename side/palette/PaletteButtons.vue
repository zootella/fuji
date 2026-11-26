<script setup>//a survey of the palettes other people made, for coloring the bar's lucide drawings in light, where fuji draws them in color: each laid out as a box of crayons, its hues in order around the color wheel and each hue's shades stacked, each crayon with a drawing in it, and then the toolbar and the edit menu in the palette's colors, at the sizes fuji draws them, 24 css pixels on a toolbar's big button and 16 in a menu's row. Above each box, who made the palette, when and what for, what people take it for, and where its values were read. In dark every drawing is in the text's own color, which the monochrome palette shows first, so the page draws every palette in light alone. The toolbar is FujiBar itself, from bars.txt, and the menu is PaletteMenu, which PaletteMenu.vue says why. The palettes and their colors are upstream.js's, which says what each is; everything else on the page is fuji's

import {upstreams, upstreamGroups, upstreamLeftOut} from './upstream.js'
import {barIcons} from '../../desktop/src/plastic/bar.js'
import FujiBar from '../../desktop/src/components/FujiBar.vue'
import PaletteMenu from './PaletteMenu.vue'

const groups = upstreamGroups.map(g => ({...g, palettes: upstreams.filter(p => p.group == g.title)}))//each group with its palettes, in the order upstream.js lists them
const headings = []//each group's heading on the page, by its place in groups, which the buttons at the top scroll to
function crayons(palette) { return palette.colors.reduce((n, [, ...shades]) => n + shades.length, 0) }//how many crayons are in a palette's box
function said(name, shade) { return shade ? `${name} ${shade}` : name }//a crayon's name, its hue and then its maker's word for the shade, where the hue has more than one
function drawingColors(palette) {//the colors the named drawings take in this palette, as properties the rules below read; none for monochrome, where every drawing keeps the text's color
	let style = {}
	for (let [role, [name, shade]] of Object.entries(palette.role)) {
		let found = palette.colors.find(c => c[0] == name)?.slice(1).find(s => s[0] == shade)
		if (!found) throw new Error(`${palette.name} has no ${said(name, shade)} for its ${role}`)
		style[`--draw-${role}`] = found[1]
	}
	return style
}
function taken(palette) {//the crayons the toolbar's four drawings take, in words, for the line under the box
	return Object.entries(palette.role).map(([role, [name, shade]]) => `${role} ${said(name, shade)}`).join(', ')
}

</script>
<template>

<div>
	<h1>Button Color</h1>
	<p>A survey of the palettes other people made, {{upstreams.length - 1}} of them, before fuji takes one for the bar's drawings. A good many teams have spent years choosing families of colors that are distinct, usable and belong together, and fuji would rather inherit that work than repeat it. Each palette is a box of crayons: its hues in order around the color wheel from red, and each hue's shades stacked, lightest at the top where the palette orders them so, each crayon with a drawing in it at 24 and 16 css pixels. Above each box, who made it, when and what for, what people take it for, and in smaller words its license and where its values were read. Below it, the toolbar and the edit menu from <span class="myFixedWidth">bars.txt</span>, where Back, Forward and Up take the palette's blue, Folders its yellow, Delete its red and Undo its green, the four named so far, and every other drawing stays in the text's own color. Everything is drawn in light alone, since in dark every drawing is in the text's own color, as the monochrome one first shows, and each pane stands on the face in light, as the bar does. Point at a button or a row to see it lit and lifted; the toolbar is FujiBar itself.</p>
	<div class="myContents">
		<button v-for="(group, i) in groups.slice(1)" :key="group.title" class="myButton" @click="headings[i + 1].scrollIntoView()">{{group.title}}, {{group.palettes.length}}</button><!-- to a group's heading, within the tab, with nothing put in the address, which the tabs keep -->
	</div>
	<template v-for="(group, i) in groups" :key="group.title">
		<h2 :ref="el => headings[i] = el" class="myGroup">{{group.title}}</h2>
		<p v-if="group.about">{{group.about}}</p>
		<template v-for="palette in group.palettes" :key="palette.name">
			<h3>{{palette.name}}</h3>
			<p>{{palette.made}}</p>
			<p v-if="palette.liked">{{palette.liked}}</p>
			<p v-if="palette.read" class="myRead">{{palette.read}}</p>
			<div class="myPanes">
				<div class="myPane myOnBar" data-mode="light" :style="drawingColors(palette)">
					<div v-if="palette.colors.length" class="myBox">
						<div v-for="[name, ...shades] in palette.colors" :key="name" class="myColumn"><!-- one hue, its shades stacked, as crayons stand in a box -->
							<div v-for="[shade, color] in shades" :key="shade" class="myCrayon">
								<span class="myChip" :style="{background: color}"></span>
								<span class="mySample" :style="{color}"><!-- the same drawing at a toolbar's size and a menu's -->
									<span class="myIcon myBig" v-html="barIcons['folder-tree']"></span>
									<span class="myIcon" v-html="barIcons['folder-tree']"></span>
								</span>
								<span class="myName">{{said(name, shade)}}</span>
								<span class="myFixedWidth">{{color}}</span>
							</div>
						</div>
					</div>
					<div class="mySideBySide myDrawn">
						<FujiBar name="Toolbar">
							<template #panel><p class="myPanelNote">The bench draws this panel.</p></template>
						</FujiBar>
						<PaletteMenu name="Edit Menu" />
					</div>
					<div v-if="palette.colors.length" class="myTaken">{{crayons(palette)}} crayons. On the toolbar: {{taken(palette)}}.</div>
				</div>
			</div>
		</template>
	</template>
	<h2 class="myGroup">Left out</h2>
	<p>{{upstreamLeftOut}}</p>
</div>

</template>
<style scoped>

.myContents { display: flex; flex-wrap: wrap; gap: 4px; margin-bottom: 8px } /* a button for each group, which scrolls to it */
.myGroup { padding-top: 16px; border-top: 1px solid var(--color-border) } /* a group's heading under a line across the page, so the long page reads as its few parts */
h3 { margin: 28px 0 4px; font-weight: 600; color: var(--color-text) } /* a palette's name, in the root's own size, under its group's larger heading */
.myRead { font-size: 12px; color: var(--color-text-placeholder) } /* the license and the source, quieter than the words about the palette */
.myOnBar { background-color: var(--color-face) } /* the face, which the bar stands on, so the drawings are seen on what they will stand on */
.myBox { display: flex; flex-wrap: wrap; gap: 16px 6px }
.myColumn { display: flex; flex-direction: column; gap: 8px } /* one hue's shades, one above the next */
.myCrayon { display: flex; flex-direction: column; align-items: center; gap: 3px; width: 58px; font-size: 10px; line-height: 1.3; text-align: center; overflow-wrap: anywhere }
.myName { min-height: 2lh } /* room for a name of two lines under every crayon, so a long name in one column leaves the shades level across the box */
.myChip { width: 40px; height: 14px }
.mySample { display: flex; gap: 2px }
.myIcon { display: flex }
.myIcon :deep(svg) { width: 16px; height: 16px; stroke-width: 1.5 } /* lucide's drawing at a menu's size, its stroke a css pixel */
.myIcon.myBig :deep(svg) { width: 24px; height: 24px } /* and at a toolbar's, its stroke 1.5 in the drawing's own 24 */
.mySideBySide { display: flex; gap: 32px; align-items: flex-start; flex-wrap: wrap }
.myDrawn :deep(.lucide-arrow-left), .myDrawn :deep(.lucide-arrow-right), .myDrawn :deep(.lucide-arrow-up) { color: var(--draw-blue) } /* Back, Forward and Up, by the lucide drawings bars.txt gives them, which carry their names as classes; with no color of the palette's, as in monochrome, each keeps the text's */
.myDrawn :deep(.lucide-folder-tree) { color: var(--draw-yellow) } /* Folders */
.myDrawn :deep(.lucide-trash-2) { color: var(--draw-red) } /* Delete */
.myDrawn :deep(.lucide-undo-2) { color: var(--draw-green) } /* Undo */
.myTaken { font-size: 12px; color: var(--color-text-placeholder) } /* which crayons the toolbar's drawings took */
.myPanelNote { padding: 8px 12px } /* the words in the views panel, which the bench fills with controls in fuji */

</style>
