<script setup>//the brand colors proposed for fuji on 2025-07-08, in a session with Copilot, each as a circle in light and in dark, with a mark under the one fuji took. Each is written here, since each says what one proposal was; fuji's own cyan is read from style.css to find which

import {paletteValue} from './palette.js'

const modes = ['light', 'dark']
const proposals = [
	{name: 'Process Teal', color: '#22d3ee'},
	{name: 'Wasabi Mist',  color: '#9fffe0'},
	{name: 'Fuji Cyan',    color: '#00bcd4'},
	{name: 'Fuji Green',   color: '#00a651'},
]
const taken = paletteValue('fuji-brand', 'light').toLowerCase()//fuji's cyan, the same in both modes

</script>
<template>

<div>
	<h1>Brand Colors</h1>
	<p>Four brand colors proposed for fuji on 2025-07-08, in a session with Copilot. Wasabi Mist became fuji's cyan, <span class="myFixedWidth">fuji-brand</span> in <span class="myFixedWidth">style.css</span>: the icon's own color, the frame of the thumbnail under the pointer, a window button fuji draws while it is pressed, and for now the square that holds a row's place for its icon in the list and the tree.</p>
	<div class="myPanes">
		<div v-for="mode in modes" :key="mode" class="myPane myCircles" :data-mode="mode">
			<div v-for="p in proposals" :key="p.name" class="mySwatch">
				<div class="myCircle" :style="{backgroundColor: p.color}"></div>
				<div>{{p.name}}</div>
				<div class="myFixedWidth">{{p.color}}</div>
				<div v-if="p.color == taken" class="myFixedWidth myTaken">fuji-brand</div>
			</div>
		</div>
	</div>
</div>

</template>
<style scoped>

.myCircles { flex-direction: row; flex-wrap: wrap; justify-content: space-around; gap: 24px }
.mySwatch { display: flex; flex-direction: column; align-items: center; gap: 2px }
.myCircle { width: 120px; height: 120px; margin-bottom: 6px; border-radius: 50% }
.myTaken { color: var(--color-zed-text-placeholder) }

</style>
