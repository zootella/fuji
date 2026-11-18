<script setup>//every color style.css names, in light and in dark, each on its mode's own ground with the stylesheet's own comment beside it, and the numbered grays beside tailwind's, step by step. It holds no color of its own: fuji's come from the stylesheet through palette.js, and tailwind's from the version installed here, read as text

import tailwindText from 'tailwindcss/theme.css?raw'
import {paletteColors, paletteHalf} from './palette.js'

const modes = ['light', 'dark']
const groups = [
	{title: "Zed's", test: name => name.startsWith('zed-')},
	{title: 'The numbered grays', test: name => name.startsWith('gray-')},
	{title: "Fuji's own", test: name => name.startsWith('fuji-')},
	{title: 'Named for their job', test: name => !/^(zed|gray|fuji)-/.test(name)},
]
const steps = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950]
const tailwindGrays = ['slate', 'gray', 'zinc', 'neutral', 'stone']

function tailwindStep(gray, step) {//one of tailwind's grays at one step, as its theme writes it, with its lightness, chroma and hue
	let m = tailwindText.match(new RegExp(`--color-${gray}-${step}: (oklch\\(([\\d.]+)% ([\\d.]+) ([\\d.]+|none)\\))`))
	return {background: m[1], l: +m[2], c: +m[3], h: +m[4]}
}
function fujiStep(step, mode) {//one of fuji's numbered grays at one step in one mode, painted by its name, with its lightness, chroma and hue worked out from the hex the stylesheet gives
	let value = paletteColors.find(c => c.name == `gray-${step}`).value
	return {background: `var(--color-gray-${step})`, scheme: mode, ...oklchOfHex(paletteHalf(value, mode))}
}
function oklchOfHex(hex) {//lightness in percent, chroma, and hue in degrees, through oklab
	let linear = v => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }
	let [red, green, blue] = [1, 3, 5].map(i => linear(parseInt(hex.slice(i, i + 2), 16)))
	let l = Math.cbrt(0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue), m = Math.cbrt(0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue), s = Math.cbrt(0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue)
	let a = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s, b = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s
	return {l: 100 * (0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s), c: Math.hypot(a, b), h: (Math.atan2(b, a) * 180 / Math.PI + 360) % 360}
}
function says({l, c, h}) { return `${l.toFixed(1)} / ${c.toFixed(3)} / ${c < 0.002 ? 'none' : Math.round(h)}` }//no hue where the chroma is too little to lean any way

</script>
<template>

<div>
	<h1>Theme</h1>
	<p>Every color <span class="myFixedWidth">src/style.css</span> names, read from the stylesheet as this page loads, drawn in light and in dark, each on its mode's own ground. The grays are Zed's, from its One Light and One Dark themes, apart from the numbered grays, which carry Tailwind's lightness in Zed's tints. The notes are the stylesheet's own comments.</p>
	<div class="myTable">
		<template v-for="group in groups" :key="group.title">
			<div class="myGroup">{{group.title}}</div>
			<template v-for="c in paletteColors.filter(c => group.test(c.name))" :key="c.name">
				<div class="myFixedWidth">{{c.name}}</div>
				<div v-for="mode in modes" :key="mode" class="myMode" :style="{colorScheme: mode}"><!-- a color in one mode, on that mode's own ground, which the cell's color-scheme picks -->
					<div class="mySwatch" :style="{background: `var(--color-${c.name})`}"></div>
					<span class="myFixedWidth">{{paletteHalf(c.value, mode)}}</span>
				</div>
				<div class="myNote">{{c.note}}</div>
			</template>
		</template>
	</div>

	<h2>The numbered grays beside Tailwind's</h2>
	<p>Each row is one step at one lightness: Tailwind's grays from the version installed here, then fuji's numbered grays as light and dark draw them. Each cell reads its OKLCH lightness, chroma and hue, with no hue where the chroma is under 0.002, too little to lean any way.</p>
	<div class="myLadder">
		<div></div>
		<div v-for="g in tailwindGrays" :key="g" class="myLadderHead">Tailwind {{g}}</div>
		<div class="myLadderHead">fuji in light</div>
		<div class="myLadderHead">fuji in dark</div>
		<template v-for="step in steps" :key="step">
			<div class="myLadderHead">{{step}}</div>
			<div v-for="(cell, i) in [...tailwindGrays.map(g => tailwindStep(g, step)), ...modes.map(mode => fujiStep(step, mode))]" :key="i" class="myFixedWidth myStep" :data-ink="cell.l > 62 ? 'dark' : 'light'" :style="{background: cell.background, colorScheme: cell.scheme}">{{says(cell)}}</div>
		</template>
	</div>
</div>

</template>
<style scoped>

.myTable { display: grid; grid-template-columns: max-content max-content max-content minmax(0, 1fr); border-top: 1px solid var(--color-zed-border) }
.myTable > * { padding: 6px 12px; border-bottom: 1px solid var(--color-zed-border) }
.myGroup { grid-column: 1 / -1; padding-top: 20px; font-weight: 600 }
.myMode { display: flex; align-items: center; gap: 10px; background-color: var(--color-zed-editor-background); color: var(--color-zed-text) }
.mySwatch { flex: none; width: 48px; height: 24px; border-radius: 4px; outline: 1px solid color-mix(in srgb, currentColor 25%, transparent) } /* outlined, so a color the same as the ground still shows where it is */
.myNote { color: var(--color-zed-text-muted) }
.myLadder { display: grid; grid-template-columns: repeat(8, max-content) }
.myLadder > * { padding: 12px 14px; white-space: nowrap }
.myLadderHead { align-self: end; font-weight: 600 }
.myStep[data-ink="dark"] { color: black } /* the cell's reading in whichever of black or white shows on its gray; neither is one of fuji's colors, just ink that reads */
.myStep[data-ink="light"] { color: white }

</style>
