<script setup>//candidate palettes for coloring the bar's lucide drawings in light, where fuji draws them in color, each laid out as a box of crayons, three shades of each of an upstream's colors in order around the color wheel, each with a drawing in it, and then the toolbar and the edit menu in the palette's colors, at the sizes fuji draws them: 24 css pixels on a toolbar's big button and 16 in a menu's row. In dark every drawing is in the text's own color, which the monochrome palette shows first, so the page draws every palette in light alone. The toolbar is FujiBar itself, from bars.txt, and the menu is ColorMenu, which ColorMenu.vue says why. A palette's colors are written here, since each says what one of that palette's colors is, apart from Tailwind's, which are read from the version installed; everything else on the page is fuji's

import tailwindText from 'tailwindcss/theme.css?raw'
import {barIcons} from '../src/bar.js'
import FujiBar from '../src/components/FujiBar.vue'
import ColorMenu from './ColorMenu.vue'

function tailwindHue(name, step) { return tailwindText.match(new RegExp(`--color-${name}-${step}: (oklch\\([^)]*\\))`))[1] }//one of tailwind's hues at one step, as its theme writes it
const palettes = [//each a box of crayons: three shades of each of an upstream's colors, in order around the color wheel, as name and color in light, so the boxes hold about as many crayons each, and after them the upstreams with one shade of each color, smaller boxes; role says which crayon each named drawing takes, the middle shade of its color
	{
		name: 'Monochrome, as the bar is now',
		about: "Fuji's text color, text in style.css: every drawing in dark, and in light until a palette is chosen.",
		hues: [],
		role: {},
	},
	{
		name: 'Radix Colors, steps 9, 10 and 11',
		about: "From @radix-ui/colors 3.0.0, MIT, read from the package on 2026-11-19: eight of its 25 scales, one for each color, at step 9, its solid swatch and the brightest, step 10, a little deeper, and step 11, its color for text. Each scale is tuned so the same step stands out alike across colors; its yellow at 9 and 10 is made to carry dark text, and is pale for a drawing on white.",
		hues: [
			['red 9', '#e5484d'], ['red 10', '#dc3e42'], ['red 11', '#ce2c31'],
			['orange 9', '#f76b15'], ['orange 10', '#ef5f00'], ['orange 11', '#cc4e00'],
			['amber 9', '#ffc53d'], ['amber 10', '#ffba18'], ['amber 11', '#ab6400'],
			['grass 9', '#46a758'], ['grass 10', '#3e9b4f'], ['grass 11', '#2a7e3b'],
			['cyan 9', '#00a2c7'], ['cyan 10', '#0797b9'], ['cyan 11', '#107d98'],
			['blue 9', '#0090ff'], ['blue 10', '#0588f0'], ['blue 11', '#0d74ce'],
			['violet 9', '#6e56cf'], ['violet 10', '#654dc4'], ['violet 11', '#6550b9'],
			['pink 9', '#d6409f'], ['pink 10', '#cf3897'], ['pink 11', '#c2298a'],
		],
		role: {red: 'red 10', green: 'grass 10', blue: 'blue 10', yellow: 'amber 10'},
	},
	{
		name: 'Tailwind, steps 500, 600 and 700',
		about: "From the tailwindcss installed here, MIT: eight of its 17 hues, one for each color, at 500, its middle, and 600 and 700, on the ladder of lightness fuji's numbered grays take their numbers from.",
		hues: ['red', 'orange', 'amber', 'green', 'cyan', 'blue', 'violet', 'pink'].flatMap(name => [500, 600, 700].map(step => [`${name} ${step}`, tailwindHue(name, step)])),//read from tailwind's own theme rather than written here, since the version is installed beside this page
		role: {red: 'red 600', green: 'green 600', blue: 'blue 600', yellow: 'amber 600'},
	},
	{
		name: "GNOME's palette, shades 3, 4 and 5",
		about: "From the GNOME Human Interface Guidelines, made for desktop icons, read from libadwaita's css-variables.md on 2026-11-19: all seven of its colors at its middle shade and the two deeper, since its two lightest are too pale to draw on white. It has a brown, and no cyan or pink.",
		hues: [
			['red 3', '#e01b24'], ['red 4', '#c01c28'], ['red 5', '#a51d2d'],
			['orange 3', '#ff7800'], ['orange 4', '#e66100'], ['orange 5', '#c64600'],
			['brown 3', '#986a44'], ['brown 4', '#865e3c'], ['brown 5', '#63452c'],
			['yellow 3', '#f6d32d'], ['yellow 4', '#f5c211'], ['yellow 5', '#e5a50a'],
			['green 3', '#33d17a'], ['green 4', '#2ec27e'], ['green 5', '#26a269'],
			['blue 3', '#3584e4'], ['blue 4', '#1c71d8'], ['blue 5', '#1a5fb4'],
			['purple 3', '#9141ac'], ['purple 4', '#813d9c'], ['purple 5', '#613583'],
		],
		role: {red: 'red 4', green: 'green 4', blue: 'blue 4', yellow: 'yellow 4'},
	},
	{
		name: 'Tango, all three shades',
		about: "The Tango Desktop Project's palette of 2005, read from Inkscape's Tango-Palette.gpl on 2026-11-19: all seven of its colors at all three of its shades. It has a brown, Chocolate, and no cyan or pink.",
		hues: [
			['scarlet red 1', '#ef2929'], ['scarlet red 2', '#cc0000'], ['scarlet red 3', '#a40000'],
			['orange 1', '#fcaf3e'], ['orange 2', '#f57900'], ['orange 3', '#ce5c00'],
			['chocolate 1', '#e9b96e'], ['chocolate 2', '#c17d11'], ['chocolate 3', '#8f5902'],
			['butter 1', '#fce94f'], ['butter 2', '#edd400'], ['butter 3', '#c4a000'],
			['chameleon 1', '#8ae234'], ['chameleon 2', '#73d216'], ['chameleon 3', '#4e9a06'],
			['sky blue 1', '#729fcf'], ['sky blue 2', '#3465a4'], ['sky blue 3', '#204a87'],
			['plum 1', '#ad7fa8'], ['plum 2', '#75507b'], ['plum 3', '#5c3566'],
		],
		role: {red: 'scarlet red 2', green: 'chameleon 2', blue: 'sky blue 2', yellow: 'butter 2'},
	},
	{
		name: "Zed's One Light",
		about: "From assets/themes/one/one.json in Zed's repository, the file fuji's grays take their lightness from: its status colors, its terminal's yellow and its syntax colors, one of each hue, tuned against One Light's grays. One shade of each color, which is all it has, so a smaller box than the ones above.",
		hues: [['red, error', '#d36151'], ['orange, number', '#ad6e25'], ['amber, constant', '#c18401'], ['yellow, terminal', '#d2b67c'], ['green, success', '#669f59'], ['cyan, type', '#3882b7'], ['blue, info', '#5c78e2'], ['purple, keyword', '#a449ab']],
		role: {red: 'red, error', green: 'green, success', blue: 'blue, info', yellow: 'yellow, terminal'},
	},
	{
		name: "Okabe and Ito's palette",
		about: "Color Universal Design, chosen so that people with each kind of colorblindness can tell the colors apart. The red is their vermilion and the green their bluish green. One shade of each color, which is all it has, so a smaller box than the ones above.",
		hues: [['vermilion', '#d55e00'], ['orange', '#e69f00'], ['yellow', '#f0e442'], ['bluish green', '#009e73'], ['sky blue', '#56b4e9'], ['blue', '#0072b2'], ['reddish purple', '#cc79a7']],
		role: {red: 'vermilion', green: 'bluish green', blue: 'blue', yellow: 'yellow'},
	},
	{
		name: "Apple's system colors",
		about: "The colors macOS and iOS give their own controls, here their values for light. Red as Apple's table gives it; the others from memory, and Apple says they change from release to release. One shade of each color, which is all it has, so a smaller box than the ones above.",
		hues: [['red', '#ff3b30'], ['orange', '#ff9500'], ['yellow', '#ffcc00'], ['green', '#34c759'], ['cyan', '#32ade6'], ['blue', '#007aff'], ['purple', '#af52de']],
		role: {red: 'red', green: 'green', blue: 'blue', yellow: 'yellow'},
	},
]

function columns(palette) {//the palette's crayons by color, each color's shades together in the order the palette lists them, a color being a crayon's name without the number of its shade
	let byColor = []
	for (let hue of palette.hues) {
		let color = hue[0].replace(/ \d+$/, '')
		if (byColor.at(-1)?.color == color) byColor.at(-1).hues.push(hue)
		else byColor.push({color, hues: [hue]})
	}
	return byColor.map(c => c.hues)
}
function drawingColors(palette) {//the colors the named drawings take in this palette, as properties the rules below read; none for monochrome, where every drawing keeps the text's color
	let style = {}
	for (let [role, name] of Object.entries(palette.role)) style[`--draw-${role}`] = palette.hues.find(h => h[0] == name)[1]
	return style
}

</script>
<template>

<div>
	<h1>Button Color</h1>
	<p>Candidate palettes for the bar's drawings in light, where fuji draws them in color, drawn here in light alone, since in dark every drawing is in the text's own color, as the monochrome one first shows. The colors fuji takes may come from one box or several. Each is laid out as a box of crayons from a trusted upstream, three shades of each of its colors, about eight colors, in order around the color wheel, each with a drawing in it at 24 and 16 css pixels, so the boxes hold about as many crayons each and compare fairly, and the toolbar takes the middle shade; after them, the upstreams that have one shade of each color, in smaller boxes. Below each box, the toolbar and the edit menu from <span class="myFixedWidth">bars.txt</span>, where Back, Forward and Up take the palette's blue, Folders its yellow, Delete its red and Undo its green, the four named so far, and every other drawing stays in the text's own color. Each pane stands on the bar's own ground in light as fuji draws it on this machine: on Windows the title bar's white, and elsewhere the content's. Point at a button or a row to see it lit and lifted; the toolbar is FujiBar itself.</p>
	<template v-for="palette in palettes" :key="palette.name">
		<h2>{{palette.name}}</h2>
		<p class="myAbout">{{palette.about}}</p>
		<div class="myPanes">
			<div class="myPane myOnBar" data-mode="light" :style="drawingColors(palette)">
				<div v-if="palette.hues.length" class="myBox">
					<div v-for="column in columns(palette)" :key="column[0][0]" class="myColumn"><!-- one color, its shades stacked, as crayons stand in a box -->
					<div v-for="hue in column" :key="hue[0]" class="myCrayon">
						<span class="myChip" :style="{background: hue[1]}"></span>
						<span class="mySample" :style="{color: hue[1]}"><!-- the same drawing at a toolbar's size and a menu's -->
							<span class="myIcon myBig" v-html="barIcons['folder-tree']"></span>
							<span class="myIcon" v-html="barIcons['folder-tree']"></span>
						</span>
						<span>{{hue[0]}}</span>
						<span class="myFixedWidth">{{hue[1]}}</span>
					</div>
					</div>
				</div>
				<div class="mySideBySide myDrawn">
					<FujiBar name="Toolbar">
						<template #panel><p class="myPanelNote">The bench draws this panel.</p></template>
					</FujiBar>
					<ColorMenu name="Edit Menu" />
				</div>
			</div>
		</div>
	</template>
</div>

</template>
<style scoped>

.myAbout { font-size: 12px }
.myOnBar { background-color: var(--bar-ground) } /* the bar's own ground, which style.css sets by platform, so the drawings are seen on what they will stand on */
.myBox { display: flex; flex-wrap: wrap; gap: 16px 6px }
.myColumn { display: flex; flex-direction: column; gap: 8px } /* one color's shades, one above the next */
.myCrayon { display: flex; flex-direction: column; align-items: center; gap: 3px; width: 58px; font-size: 10px; line-height: 1.3; text-align: center; overflow-wrap: anywhere }
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
.myPanelNote { padding: 8px 12px } /* the words in the views panel, which the bench fills with controls in fuji */

</style>
