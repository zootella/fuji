<script setup>//a selection and a menu's row under the pointer, in each platform's own look, in four quadrants, A and B the Mac in light and dark and C and D everywhere else: a list with the keyboard on a selected row, with it on a row that is not selected, and without it, and a row under the pointer in FujiBar's menu, and then at the bottom, apart, the same in MockMenu, the right-click menu, which looks the same on every platform. The list is drawn by FujiList's own rules, which are not scoped and so reach any element of the list's classes, here laid out in the markup list.js makes; the menus are ColorMenu and ColorMockMenu, copies of FujiBar's rules and MockMenu's, which each file says why. Under each menu, the styles that give it its edge, its shadow and any see-through fill, read from the menu's own component as text, since a pane has nothing behind a menu to show them. Every color here is fuji's, painted by its name in style.css and captioned with the value the engine paints in that pane, and the numbered gray it is written as, where it is one

import {paletteValue, paletteNotch} from './palette.js'
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
	{name: 'a row under the pointer', selector: '.myBarItem:hover', properties: ['border-color', 'background-color', 'color']},
])
const mockMenuStyles = styled(mockMenuText, [
	{name: 'the card', selector: '.myMenu', properties: ['border-radius', 'background-color', 'backdrop-filter', 'box-shadow']},
])

const platforms = [//each a row of panes, which names its platform as the shell names the window's, so style.css picks that platform's look for everything in it; and the letter each pane goes by
	{key: 'mac', title: 'On the Mac', quadrant: {light: 'A', dark: 'B'}},
	{key: 'windows', title: 'On Windows and Linux', quadrant: {light: 'C', dark: 'D'}},
]
const modes = ['light', 'dark']
const rows = ['An ordinary row', 'Selected', 'Selected', 'Selected', 'An ordinary row', 'Selected alone', 'An ordinary row']//a run of three selected rows and one alone, as list.js marks them, each joined to a selected neighbor above or below
const selected = [1, 2, 3, 5]
const lists = [//the same rows three times, while the list has the keyboard, after a key has been used, which list.js marks on the list as data-active and data-keyboard, and while the keyboard is elsewhere
	{title: 'With the keyboard on a selected row', active: true, focus: 3, fill: 'select'},
	{title: 'With the keyboard on a row not selected', active: true, focus: 4, fill: 'select'},
	{title: 'Without the keyboard', active: false, focus: 3, fill: 'select-quiet'},
]
function painted(name, platform, mode) {//the color a name comes to where an element names this platform and is in this mode, as the engine resolves it, so a caption says what is painted there rather than what this page supposes
	let probe = document.createElement('div')
	probe.dataset.platform = platform
	probe.style.colorScheme = mode
	probe.style.backgroundColor = `var(--color-${name})`
	document.body.append(probe)
	let [r, g, b, a] = getComputedStyle(probe).backgroundColor.match(/[\d.]+/g).map(Number)
	probe.remove()
	return '#' + [r, g, b].map(v => Math.round(v).toString(16).padStart(2, '0')).join('') + (a !== undefined && a < 1 ? ` at ${Math.round(a * 100)} percent` : '')
}
function said(name, platform, mode) {//a color as a caption gives it: the value painted, and the numbered gray it is written as, where it is one
	let notch = paletteNotch(name, platform, mode)
	return painted(name, platform, mode) + (notch ? `, ${notch}` : '')
}
function marks(i, list) {//the attributes list.js puts on a row
	let s = selected.includes(i)
	return {'data-selected': s || null, 'data-join-up': (s && selected.includes(i - 1)) || null, 'data-join-down': (s && selected.includes(i + 1)) || null, 'data-focus': i == list.focus || null}
}

</script>
<template>

<div>
	<h1>Accent Colors</h1>
	<p>A selection and a menu's row under the pointer, each platform in its own look, in four quadrants: A, the Mac in light; B, the Mac in dark; C, Windows and Linux in light; and D, Windows and Linux in dark. The Mac's strong blue shows only in A and B, the same in both; Windows' pale blue only in C; and D has no color at all. Every value comes from a platform, its grays set on the ladder, as the captions' notches show. On the Mac, in A and B, a selection with the keyboard and a menu's row under the pointer are filled solid in the accent, the Mac's own blue, with white words. On Windows and Linux, a selection with the keyboard is Explorer's pale blue in C, and in D a gray two notches darker than Windows 11's, so its text reads, each with the text's own color; a menu's row under the pointer is white in C and black in D, across the menu's whole width with a gray line around it, so a menu there has no blue in it. Without the keyboard a selection is a quiet gray, the Mac's own, on every platform. A run of selected rows is one block with a score between its rows, and once a key has been used the row the keyboard is on has a solid line around it, black in light and white in dark, whether it is selected or not. The list is drawn by FujiList's own rules, in the markup list.js makes, and the menu by a copy of FujiBar's; each caption gives a color as the engine paints it in that pane. A menu's face is the same everywhere, the face the bar and the list stand on, so on Windows and Linux its row under the pointer is a purer white or black than the menu around it. MockMenu, the right-click menu, is apart at the bottom, as it was.</p>
	<template v-for="platform in platforms" :key="platform.key">
		<h2>{{platform.title}}</h2>
		<div class="myPanes">
			<div v-for="mode in modes" :key="mode" class="myPane" :data-mode="mode" :data-platform="platform.key">
				<div class="myQuadrant"><b>{{platform.quadrant[mode]}}</b> {{platform.title.replace('On ', 'on ')}}, in {{mode}} <span class="myFixedWidth myValue">face {{said('face', platform.key, mode)}}</span></div><!-- the pane's letter, and the face it stands on, the same on every platform, as the value the engine paints here and the numbered gray it is written as -->

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
						<figcaption><b>{{list.title}}</b> <span class="myFixedWidth myValue">{{list.fill}} {{said(list.fill, platform.key, mode)}}</span></figcaption>
					</figure>
				</div>
				<figure>
					<ColorMenu name="Edit Menu" lit="Copy" />
					<figcaption><b>FujiBar's menu, a row under the pointer</b> <span class="myFixedWidth myValue">menu-hot {{said('menu-hot', platform.key, mode)}}; its edge {{said('menu-hot-edge', platform.key, mode)}}</span></figcaption>
				</figure>
			</div>
		</div>
	</template>

	<h2>How FujiBar's menu is drawn</h2>
	<p>Opaque, so nothing behind it shows through, on the face the bar it drops from stands on, the same on every platform, with a gutter down its left in the gray named for a menu's material. Its shadow is a filter on the button drawn again and the card together, so the two cast one shadow, falling down and to the right. The styles as <span class="myFixedWidth">FujiBar.vue</span> writes them, read from it as this page loads, and the colors they name:</p>
	<div class="myStyles">
		<div v-for="(line, i) in fujiBarStyles.lines" :key="i" class="myStyleLine"><span>{{line.part}}</span><span class="myFixedWidth">{{line.property}}: {{line.value}}</span></div>
		<div v-for="name in fujiBarStyles.colors" :key="name" class="myStyleLine"><span class="myFixedWidth">{{name}}</span><span class="myFixedWidth myValue">{{paletteValue(name, 'light')}} in light, {{paletteValue(name, 'dark')}} in dark</span></div>
	</div>

	<h2>MockMenu, the right-click menu, on every platform</h2>
	<p>The early menu in the Mac's look that still opens on a right click on the sheet, the light table and the list: translucent and rounded, its row under the pointer filled in the accent with white words. It looks the same on every platform, since nothing in it stands on the bar's ground, so it is the one place the accent still shows on Windows and Linux, until it becomes one of FujiBar's menus.</p>
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
.myQuadrant { color: var(--color-text-muted) } /* the pane's letter and what it shows, which the talk about these choices goes by */
.myQuadrant b { color: var(--color-text); font-size: 15px }
.myStyles { display: grid; grid-template-columns: max-content minmax(0, 1fr); gap: 4px 16px; padding: 12px 16px; border: 1px solid var(--color-border) } /* each line a part and its declaration, or a color and its values */
.myStyleLine { display: contents }
.myStyleLine > :last-child { overflow-wrap: anywhere }
.myAfter { margin-top: 16px }
.myListBox { --list-columns: 1fr; border: 1px solid var(--color-border) } /* the list's one column, which list.js would set on the frame from its columns, and a line around it, which the page that places a list draws */

</style>
