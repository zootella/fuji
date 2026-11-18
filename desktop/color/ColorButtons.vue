<script setup>//candidate palettes for coloring the bar's lucide drawings, each laid out as a box of crayons, every hue at one step in order around the color wheel with a drawing in it, and then the toolbar and the edit menu in the palette's colors, in light and in dark, at the sizes fuji draws them: 24 css pixels on a toolbar's big button and 16 in a menu's row. The toolbar is FujiBar itself, from bars.txt, and the menu is ColorMenu, which ColorMenu.vue says why. A palette's colors are written here, since each says what one of that palette's colors is, apart from Tailwind's, which are read from the version installed; everything else on the page is fuji's

import tailwindText from 'tailwindcss/theme.css?raw'
import {barIcons} from '../src/bar.js'
import FujiBar from '../src/components/FujiBar.vue'
import ColorMenu from './ColorMenu.vue'

const modes = ['light', 'dark']
function tailwindHue(name, step) { return tailwindText.match(new RegExp(`--color-${name}-${step}: (oklch\\([^)]*\\))`))[1] }//one of tailwind's hues at one step, as its theme writes it
const palettes = [//each hue as name, light, dark, in order around the color wheel; role says which hue the named drawings take
	{
		name: 'Monochrome, as the bar is now',
		about: "Zed's text color, zed-text in style.css.",
		hues: [],
		role: {},
	},
	{
		name: 'Radix Colors, step 11',
		about: "From @radix-ui/colors 3.0.0, MIT: every one of its 25 colorful scales, at step 11, its color for text and drawings, with a value of its own in light and in dark, each scale tuned to the same contrast so they mix. The oranges alone are tomato, orange, amber and yellow, with bronze, gold and brown beside them.",
		hues: [
			['tomato', '#d13415', '#ff977d'], ['red', '#ce2c31', '#ff9592'], ['ruby', '#ca244d', '#ff949d'], ['crimson', '#cb1d63', '#ff92ad'],
			['pink', '#c2298a', '#ff8dcc'], ['plum', '#953ea3', '#e796f3'], ['purple', '#8145b5', '#d19dff'], ['violet', '#6550b9', '#baa7ff'],
			['iris', '#5753c6', '#b1a9ff'], ['indigo', '#3a5bc7', '#9eb1ff'], ['blue', '#0d74ce', '#70b8ff'], ['sky', '#00749e', '#75c7f0'],
			['cyan', '#107d98', '#4ccce6'], ['teal', '#008573', '#0bd8b6'], ['jade', '#208368', '#1fd8a4'], ['green', '#218358', '#3dd68c'],
			['grass', '#2a7e3b', '#71d083'], ['mint', '#027864', '#58d5ba'], ['lime', '#5c7c2f', '#bde56c'], ['yellow', '#9e6c00', '#f5e147'],
			['amber', '#ab6400', '#ffca16'], ['orange', '#cc4e00', '#ffa057'], ['bronze', '#7d5e54', '#d4b3a5'], ['gold', '#71624b', '#cbb99f'],
			['brown', '#815e46', '#dbb594'],
		],
		role: {red: 'red', green: 'grass', blue: 'blue', yellow: 'amber'},
	},
	{
		name: 'Tailwind, 600 in light and 400 in dark',
		about: "From the tailwindcss installed here, MIT: every one of its 17 colorful hues, on the same eleven-step ladder of lightness fuji's numbered grays follow. Tailwind gives one ladder for both modes, so the steps here, 600 on white and 400 on black, are a pick for this page rather than Tailwind's.",
		hues: ['red', 'orange', 'amber', 'yellow', 'lime', 'green', 'emerald', 'teal', 'cyan', 'sky', 'blue', 'indigo', 'violet', 'purple', 'fuchsia', 'pink', 'rose'].map(name => [name, tailwindHue(name, 600), tailwindHue(name, 400)]),//read from tailwind's own theme as the theme tab reads its grays, rather than written here, since the version is installed beside this page
		role: {red: 'red', green: 'green', blue: 'blue', yellow: 'amber'},
	},
	{
		name: "Zed's One Light and One Dark",
		about: "From assets/themes/one/one.json in Zed's repository, the file fuji's grays come from: its status colors, its terminal's yellow and its syntax colors, one of each hue, tuned for each mode against Zed's grays.",
		hues: [
			['red, error', '#d36151', '#d07277'], ['orange, number', '#ad6e25', '#bf956a'], ['amber, constant', '#c18401', '#dfc184'], ['yellow, terminal', '#d2b67c', '#e5c07b'],
			['green, success', '#669f59', '#a1c181'], ['cyan, type', '#3882b7', '#6eb4bf'], ['blue, info', '#5c78e2', '#74ade8'], ['purple, keyword', '#a449ab', '#b477cf'],
		],
		role: {red: 'red, error', green: 'green, success', blue: 'blue, info', yellow: 'yellow, terminal'},
	},
	{
		name: "GNOME's palette, shade 3",
		about: "From the GNOME Human Interface Guidelines, made for desktop icons: five shades of each hue and one set for both modes, here its middle shade.",
		hues: [['red', '#e01b24', '#e01b24'], ['orange', '#ff7800', '#ff7800'], ['yellow', '#f6d32d', '#f6d32d'], ['green', '#33d17a', '#33d17a'], ['blue', '#3584e4', '#3584e4'], ['purple', '#9141ac', '#9141ac'], ['brown', '#986a44', '#986a44']],
		role: {red: 'red', green: 'green', blue: 'blue', yellow: 'yellow'},
	},
	{
		name: 'Tango, the middle shade',
		about: "The Tango Desktop Project's palette of 2005, three shades of each hue and one set for both modes, here its middle shade. Butter, Chameleon, Sky Blue and Scarlet Red as several sources give them; Orange, Plum and Chocolate from memory.",
		hues: [['scarlet red', '#cc0000', '#cc0000'], ['orange', '#f57900', '#f57900'], ['butter', '#edd400', '#edd400'], ['chameleon', '#73d216', '#73d216'], ['sky blue', '#3465a4', '#3465a4'], ['plum', '#75507b', '#75507b'], ['chocolate', '#c17d11', '#c17d11']],
		role: {red: 'scarlet red', green: 'chameleon', blue: 'sky blue', yellow: 'butter'},
	},
	{
		name: "Okabe and Ito's palette",
		about: "Color Universal Design, chosen so that people with each kind of colorblindness can tell the colors apart; one set for both modes. The red is their vermilion and the green their bluish green.",
		hues: [['vermilion', '#d55e00', '#d55e00'], ['orange', '#e69f00', '#e69f00'], ['yellow', '#f0e442', '#f0e442'], ['bluish green', '#009e73', '#009e73'], ['sky blue', '#56b4e9', '#56b4e9'], ['blue', '#0072b2', '#0072b2'], ['reddish purple', '#cc79a7', '#cc79a7']],
		role: {red: 'vermilion', green: 'bluish green', blue: 'blue', yellow: 'yellow'},
	},
	{
		name: "Apple's system colors",
		about: "The colors macOS and iOS give their own controls, a value for light and one for dark. Red as Apple's table gives it; the others from memory, and Apple says they change from release to release.",
		hues: [['red', '#ff3b30', '#ff453a'], ['orange', '#ff9500', '#ff9f0a'], ['yellow', '#ffcc00', '#ffd60a'], ['green', '#34c759', '#30d158'], ['cyan', '#32ade6', '#64d2ff'], ['blue', '#007aff', '#0a84ff'], ['purple', '#af52de', '#bf5af2']],
		role: {red: 'red', green: 'green', blue: 'blue', yellow: 'yellow'},
	},
]

function hueIn(color, mode) { return mode == 'light' ? color[1] : color[2] }
function drawingColors(palette, mode) {//the colors the named drawings take in this palette and mode, as properties the rules below read; none for monochrome, where every drawing keeps the text's color
	let style = {}
	for (let [role, name] of Object.entries(palette.role)) style[`--draw-${role}`] = hueIn(palette.hues.find(h => h[0] == name), mode)
	return style
}

</script>
<template>

<div>
	<h1>Button Color</h1>
	<p>Candidate palettes for the bar's drawings, each laid out as a box of crayons: every hue the palette has at one step, in order around the color wheel, each with a drawing in it at 24 and 16 css pixels. Below each box, the toolbar and the edit menu from <span class="myFixedWidth">bars.txt</span>, where Back, Forward and Up take the palette's blue, Folders its yellow, Delete its red and Undo its green, the four named so far, and every other drawing stays in the text's own color. Each pane stands on the bar's own ground as fuji draws it on this machine: on Windows the title bar's, white in light and black in dark, and elsewhere the content's. Point at a button or a row to see it lit and lifted; the toolbar is FujiBar itself.</p>
	<template v-for="palette in palettes" :key="palette.name">
		<h2>{{palette.name}}</h2>
		<p class="myAbout">{{palette.about}}</p>
		<div class="myPanes">
			<div v-for="mode in modes" :key="mode" class="myPane myOnBar" :data-mode="mode" :style="drawingColors(palette, mode)">
				<div v-if="palette.hues.length" class="myBox">
					<div v-for="hue in palette.hues" :key="hue[0]" class="myCrayon">
						<span class="myChip" :style="{background: hueIn(hue, mode)}"></span>
						<span class="mySample" :style="{color: hueIn(hue, mode)}"><!-- the same drawing at a toolbar's size and a menu's -->
							<span class="myIcon myBig" v-html="barIcons['folder-tree']"></span>
							<span class="myIcon" v-html="barIcons['folder-tree']"></span>
						</span>
						<span>{{hue[0]}}</span>
						<span class="myFixedWidth">{{hueIn(hue, mode)}}</span>
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
.myBox { display: flex; flex-wrap: wrap; gap: 12px 6px }
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
