<script setup>//where fuji's accent paints, in light and in dark: a list's selected row with the keyboard and without it, a menu's row under the pointer, and beside them the window buttons fuji draws in a title bar of its own. Every color here is fuji's, painted by its name in style.css and labeled with the value it has there

import {paletteValue} from './palette.js'
import ColorMenu from './ColorMenu.vue'

const modes = ['light', 'dark']
const rows = [//a few rows of a list, two of them selected
	{label: 'An ordinary row'},
	{label: 'Selected, with the keyboard', fill: 'accent', ink: 'on-accent'},
	{label: 'An ordinary row'},
	{label: 'Selected, without the keyboard', fill: 'select-quiet', ink: 'on-select-quiet'},
	{label: 'An ordinary row'},
]
const buttons = [//the window buttons as VideoWindow.vue draws them, once for each look they take: under the pointer, close in a red of its own, and pressed, which is the same cyan for all three
	{glyph: '—', state: 'minimize or maximize, under the pointer', fill: 'color-mix(in srgb, var(--color-fuji-window-button) 10%, transparent)', named: 'fuji-window-button at 10 percent'},
	{glyph: '✕', state: 'close, under the pointer', fill: 'var(--color-fuji-close-red)', ink: 'var(--color-fuji-on-close-red)', named: 'fuji-on-close-red on fuji-close-red'},
	{glyph: '☐', state: 'any of the three, pressed', fill: 'var(--color-fuji-brand)', ink: 'var(--color-fuji-on-brand)', named: 'fuji-on-brand on fuji-brand'},
]

</script>
<template>

<div>
	<h1>Accent Colors</h1>
	<p>Where the accent paints, the same on every platform: a list's selected row while the list has the keyboard, filled in the accent, beside the quiet gray of a selection the keyboard is away from; and a menu's row under the pointer, outlined in the accent over the accent at 30 percent, as FujiBar lights one. The accent is the user's, <span class="myFixedWidth">accent.color</span> in <span class="myFixedWidth">fuji.toml</span>, and this is its factory value. Below them, the window buttons fuji draws in a title bar of its own, whose colors come from Windows 10.</p>
	<div class="myPanes">
		<div v-for="mode in modes" :key="mode" class="myPane" :data-mode="mode">
			<h2>A list and a menu, on every platform</h2>
			<div class="myList">
				<div v-for="(row, i) in rows" :key="i" class="myRow" :style="row.fill ? {backgroundColor: `var(--color-${row.fill})`, color: `var(--color-${row.ink})`} : null">
					<span>{{row.label}}</span>
					<span v-if="row.fill" class="myFixedWidth myValue">{{row.fill}} {{paletteValue(row.fill, mode)}}</span>
				</div>
			</div>
			<div class="myBeside">
				<ColorMenu name="Edit Menu" lit="Copy" />
				<span class="myFixedWidth myValue">accent {{paletteValue('accent', mode)}}, at 30 percent over the bar's ground</span>
			</div>

			<h2>Fuji's window buttons, from Windows 10</h2>
			<div class="myButtons">
				<figure v-for="(b, i) in buttons" :key="i">
					<div :style="{backgroundColor: b.fill, color: b.ink}">{{b.glyph}}</div>
					<figcaption class="myFixedWidth">{{b.state}}<br>{{b.named}}</figcaption>
				</figure>
			</div>
		</div>
	</div>
</div>

</template>
<style scoped>

.myPane h2 { margin: 0 } /* the pane's own gap spaces its parts */
.myList { border: 1px solid var(--color-zed-border); padding: 4px 0 }
.myRow { display: flex; justify-content: space-between; align-items: center; gap: 16px; height: var(--row-height); margin: 0 4px; padding: 0 8px; border-radius: 6px } /* a row as tall as style.css says a list's row is, its fill inset from the list's sides and rounded, as FujiList draws a selected one */
.myValue { opacity: 0.8 }
.myBeside { display: flex; align-items: flex-end; gap: 16px; flex-wrap: wrap }
.myButtons { display: flex; flex-wrap: wrap; gap: 12px 20px }
.myButtons figure { margin: 0 }
.myButtons figure div { width: 46px; height: 32px; display: flex; align-items: center; justify-content: center; font-size: 14px }
.myButtons figcaption { margin-top: 4px; font-size: 11px; line-height: 1.4; white-space: nowrap; color: var(--color-zed-text-placeholder) } /* each name whole on its line, the buttons wrapping instead */

</style>
