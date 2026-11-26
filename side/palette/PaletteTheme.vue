<script setup>//every color style.css names, in light and in dark, each on its mode's own ground with the stylesheet's own comment beside it, and beside every neutral gray that is not itself a numbered one, the number its lightness falls at on the numbered grays. It holds no color of its own: every one comes from the stylesheet through palette.js

import {paletteColors, paletteHalf, paletteValue} from './palette.js'

const modes = ['light', 'dark']
const groups = [
	{title: 'Named for their job', test: name => !/^(gray|fuji)-/.test(name)},
	{title: 'The numbered grays', test: name => name.startsWith('gray-')},
	{title: "Fuji's own", test: name => name.startsWith('fuji-')},
]

function lightness(hex) {//the OKLab lightness of a neutral gray, in percent: with red, green and blue one number, OKLab's three cone responses are equal, so the lightness is the cube root of that one channel made linear
	let v = parseInt(hex.slice(1, 3), 16) / 255
	return 100 * Math.cbrt(v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)
}
const ladder = paletteColors.filter(c => /^gray-\d+$/.test(c.name)).map(c => ({n: +c.name.slice(5), l: lightness(c.value)})).sort((a, b) => a.n - b.n)//the numbered grays, lightest first
function ladderAt(c, mode) {//the number on the ladder this color's lightness falls at, for a neutral gray named for anything but its number, and none past either end of the ladder or for a color with a tint
	if (/^gray-\d+$/.test(c.name)) return ''
	let hex = paletteValue(c.name, mode).toLowerCase()
	if (!/^#([0-9a-f]{2})\1\1$/.test(hex)) return ''
	let l = lightness(hex)
	for (let i = 1; i < ladder.length; i++) {
		let above = ladder[i - 1], below = ladder[i]
		if (l <= above.l + 0.05 && l >= below.l - 0.05) return Math.round(above.n + (below.n - above.n) * Math.min(1, Math.max(0, (above.l - l) / (above.l - below.l))))//a twentieth of a point of slack at each end, the rounding a hex value already has
	}
	return ''
}

</script>
<template>

<div>
	<h1>Theme</h1>
	<p>Every color <span class="myFixedWidth">desktop/src/style.css</span> names, read from the stylesheet as this page loads, drawn in light and in dark, each on its mode's own ground, with the stylesheet's own comment beside it, which says where the color came from. Beside every neutral gray that is not itself a numbered one is the number its lightness falls at on the numbered grays, so each can be read against the same ladder.</p>
	<div class="myTable">
		<template v-for="group in groups" :key="group.title">
			<div class="myGroup">{{group.title}}</div>
			<template v-for="c in paletteColors.filter(c => group.test(c.name))" :key="c.name">
				<div class="myFixedWidth">{{c.name}}</div>
				<div v-for="mode in modes" :key="mode" class="myMode" :style="{colorScheme: mode}"><!-- a color in one mode, on that mode's own ground, which the cell's color-scheme picks -->
					<div class="mySwatch" :style="{background: `var(--color-${c.name})`}"></div>
					<span class="myFixedWidth">{{paletteHalf(c.value, mode)}}</span>
					<span v-if="ladderAt(c, mode)" class="myFixedWidth myAt">{{ladderAt(c, mode)}}</span>
				</div>
				<div class="myNote">{{c.note}}</div>
			</template>
		</template>
	</div>
</div>

</template>
<style scoped>

.myTable { display: grid; grid-template-columns: max-content max-content max-content minmax(0, 1fr); border-top: 1px solid var(--color-border) }
.myTable > * { padding: 6px 12px; border-bottom: 1px solid var(--color-border) }
.myGroup { grid-column: 1 / -1; padding-top: 20px; font-weight: 600 }
.myMode { display: flex; align-items: center; gap: 10px; background-color: var(--color-face); color: var(--color-text) }
.mySwatch { flex: none; width: 48px; height: 24px; border-radius: 4px; outline: 1px solid color-mix(in srgb, currentColor 25%, transparent) } /* outlined, so a color the same as the ground still shows where it is */
.myAt { margin-left: auto; color: var(--color-text-placeholder) } /* the number at the cell's right end, quiet, since the hex is the value and this is where it falls */
.myNote { color: var(--color-text-muted) }

</style>
