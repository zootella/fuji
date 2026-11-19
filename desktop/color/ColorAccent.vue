<script setup>//where fuji's accent paints, on the Mac and everywhere else, each in light and in dark: a list with the keyboard on a selected row, with it on a row that is not selected, and without it, and a row under the pointer in FujiBar's menu, and then at the bottom, apart, the same in MockMenu, the right-click menu, which looks the same on every platform. The list is drawn by FujiList's own rules, which are not scoped and so reach any element of the list's classes, here laid out in the markup list.js makes; the menus are ColorMenu and ColorMockMenu, copies of FujiBar's rules and MockMenu's, which each file says why. Under each menu, the styles that give it its edge, its shadow and any see-through fill, read from the menu's own component as text, since a pane has nothing behind a menu to show them. Every color here is fuji's, painted by its name in style.css and labeled with the value it has there

import {paletteValue} from './palette.js'
import ColorMenu from './ColorMenu.vue'
import ColorMockMenu from './ColorMockMenu.vue'
import FujiList from '../src/components/FujiList.vue'//never drawn here, and imported for its stylesheet alone, so the rows below are drawn by the rules fuji draws its list with rather than a copy of them
import fujiBarText from '../src/components/FujiBar.vue?raw'//the two menus' components as text, so the styles below that make each one's edge, shadow and see-through fill are read from the components themselves
import mockMenuText from '../src/components/MockMenu.vue?raw'

function styled(text, parts) {//each part's declarations of the properties named, as the component's stylesheet writes them, from every rule whose selectors include the part's selector, and the palette's colors they name
	let style = text.slice(text.indexOf('>', text.indexOf('<style')) + 1).replace(/\/\*[\s\S]*?\*\//g, '')//the stylesheet after its opening tag, without its comments
	let lines = [], colors = new Set()
	for (let part of parts) {
		for (let [, selectors, body] of style.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
			if (!selectors.split(',').map(s => s.trim()).includes(part.selector)) continue
			for (let declaration of body.split(';')) {
				let [property, ...value] = declaration.split(':')
				property = property.trim(); value = value.join(':').trim()
				if (!part.properties.includes(property)) continue
				lines.push({part: part.name, property, value})
				for (let [, name] of value.matchAll(/var\(--color-([a-z0-9-]+)\)/g)) colors.add(name)
			}
		}
	}
	return {lines, colors: [...colors]}
}
const fujiBarStyles = styled(fujiBarText, [
	{name: 'the card', selector: '.myBarCard', properties: ['border', 'background-color']},
	{name: 'its gutter', selector: '.myBarCard[data-kind="menu"]', properties: ['background-image']},
	{name: 'its shadow', selector: '.myBarDrop', properties: ['filter']},
])
const mockMenuStyles = styled(mockMenuText, [
	{name: 'the card', selector: '.myMenu', properties: ['border-radius', 'background-color', 'backdrop-filter', 'box-shadow']},
])

const platforms = [//each a row of panes, which names its platform as the shell names the window's, so style.css stands its bars on that platform's ground
	{key: 'mac', title: 'On the Mac'},
	{key: 'windows', title: 'On Windows and Linux'},
]
const modes = ['light', 'dark']
const rows = ['An ordinary row', 'Selected', 'Selected', 'Selected', 'An ordinary row', 'Selected alone', 'An ordinary row']//a run of three selected rows and one alone, as list.js marks them, each joined to a selected neighbor above or below
const selected = [1, 2, 3, 5]
const lists = [//the same rows three times, while the list has the keyboard, after a key has been used, which list.js marks on the list as data-active and data-keyboard, and while the keyboard is elsewhere
	{title: 'With the keyboard on a selected row', active: true, focus: 3, fill: 'accent'},
	{title: 'With the keyboard on a row not selected', active: true, focus: 4, fill: 'accent'},
	{title: 'Without the keyboard', active: false, focus: 3, fill: 'select-quiet'},
]
function marks(i, list) {//the attributes list.js puts on a row
	let s = selected.includes(i)
	return {'data-selected': s || null, 'data-join-up': (s && selected.includes(i - 1)) || null, 'data-join-down': (s && selected.includes(i + 1)) || null, 'data-focus': i == list.focus || null}
}

</script>
<template>

<div>
	<h1>Accent Colors</h1>
	<p>Where the accent paints, the same on every platform. In a list, the selected rows are filled in the accent while the list has the keyboard, and in a quiet gray while the keyboard is away; a run of selected rows is one block with a score between its rows, and once a key has been used the row the keyboard is on is drawn around in black and white dashes, as Windows' classic controls draw their focus rectangle, whether that row is selected or not. The list is drawn by FujiList's own rules, in the markup list.js makes. Below it, FujiBar's menu, the menu a bar opens, with a row under the pointer outlined in the accent over the accent at 30 percent, as Office 2003 lights one. MockMenu, the right-click menu, has a section of its own at the bottom. The accent is the user's, <span class="myFixedWidth">accent.color</span> in <span class="myFixedWidth">fuji.toml</span>, and this is its factory value. The Mac and everywhere else differ today only in the ground a menu stands on, which on Windows is its title bar's white or black; Linux's bars stand on the content's ground for now, as the Mac's do.</p>
	<template v-for="platform in platforms" :key="platform.key">
		<h2>{{platform.title}}</h2>
		<div class="myPanes">
			<div v-for="mode in modes" :key="mode" class="myPane" :data-mode="mode" :data-platform="platform.key">
				<div class="myLists">
					<figure v-for="list in lists" :key="list.title">
						<div class="myListFrame myListBox">
							<div class="myHeader"><span class="myListCell">Name</span></div>
							<div class="myList" :data-active="list.active || null" :data-keyboard="list.active || null">
								<div v-for="(row, i) in rows" :key="i" class="myRow" v-bind="marks(i, list)">
									<span class="myListCell"><span class="myListIcon"></span><span>{{row}}</span></span>
								</div>
							</div>
						</div>
						<figcaption><b>{{list.title}}</b> <span class="myFixedWidth myValue">{{list.fill}} {{paletteValue(list.fill, mode)}}</span></figcaption>
					</figure>
				</div>
				<figure>
					<ColorMenu name="Edit Menu" lit="Copy" />
					<figcaption><b>FujiBar's menu, a row under the pointer</b> <span class="myFixedWidth myValue">accent {{paletteValue('accent', mode)}} at 30 percent over the bar's ground</span></figcaption>
				</figure>
			</div>
		</div>
	</template>

	<h2>How FujiBar's menu is drawn</h2>
	<p>Opaque, on the bar's ground, so nothing behind it shows through: the content's background on the Mac and Linux, and the title bar's white or black on Windows. Its shadow is a filter on the button drawn again and the card together, so the two cast one shadow, falling down and to the right. The styles as <span class="myFixedWidth">FujiBar.vue</span> writes them, read from it as this page loads, and the colors they name:</p>
	<div class="myStyles">
		<div v-for="(line, i) in fujiBarStyles.lines" :key="i" class="myStyleLine"><span>{{line.part}}</span><span class="myFixedWidth">{{line.property}}: {{line.value}}</span></div>
		<div v-for="name in fujiBarStyles.colors" :key="name" class="myStyleLine"><span class="myFixedWidth">{{name}}</span><span class="myFixedWidth myValue">{{paletteValue(name, 'light')}} in light, {{paletteValue(name, 'dark')}} in dark</span></div>
	</div>

	<h2>MockMenu, the right-click menu, on every platform</h2>
	<p>The early menu in the Mac's look that still opens on a right click on the sheet, the light table and the list: translucent and rounded, its row under the pointer filled in the accent with its words in white or black, whichever reads. It looks the same on every platform, since nothing in it stands on the bar's ground.</p>
	<div class="myPanes">
		<div v-for="mode in modes" :key="mode" class="myPane" :data-mode="mode">
			<figure>
				<ColorMockMenu lit="Copy" />
				<figcaption><b>A row under the pointer</b> <span class="myFixedWidth myValue">accent {{paletteValue('accent', mode)}}, its words on-accent {{paletteValue('on-accent', mode)}}</span></figcaption>
			</figure>
		</div>
	</div>
	<p class="myAfter">How it is drawn, which the panes above cannot show, since nothing is behind the menu but the pane: its fill is the menu's gray at a share short of whole, so a little of what is under it shows through, and that is blurred by a backdrop filter. Its edge is three shadows, from the inside out: a light rim just inside its edge, a dark hairline just outside it, and a soft shadow below it in the same dark. The styles as <span class="myFixedWidth">MockMenu.vue</span> writes them, read from it as this page loads, and the colors they name:</p>
	<div class="myStyles">
		<div v-for="(line, i) in mockMenuStyles.lines" :key="i" class="myStyleLine"><span>{{line.part}}</span><span class="myFixedWidth">{{line.property}}: {{line.value}}</span></div>
		<div v-for="name in mockMenuStyles.colors" :key="name" class="myStyleLine"><span class="myFixedWidth">{{name}}</span><span class="myFixedWidth myValue">{{paletteValue(name, 'light')}} in light, {{paletteValue(name, 'dark')}} in dark</span></div>
	</div>
</div>

</template>
<style scoped>

.myLists { display: flex; flex-wrap: wrap; gap: 16px }
.myLists figure { flex: 1 1 180px; min-width: 0 }
figure { margin: 0 }
figcaption { margin-top: 6px; display: flex; flex-wrap: wrap; gap: 0 8px }
.myValue { color: var(--color-text-placeholder) }
.myStyles { display: grid; grid-template-columns: max-content minmax(0, 1fr); gap: 4px 16px; padding: 12px 16px; border: 1px solid var(--color-border) } /* each line a part and its declaration, or a color and its values */
.myStyleLine { display: contents }
.myStyleLine > :last-child { overflow-wrap: anywhere }
.myAfter { margin-top: 16px }
.myListBox { --list-columns: 1fr; border: 1px solid var(--color-border) } /* the list's one column, which list.js would set on the frame from its columns, and a line around it, which the page that places a list draws */

</style>
