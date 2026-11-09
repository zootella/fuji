//breakouts: measures how far a block marked class="breakout" may grow past the prose column, and fades the outline while one is on screen; style.css does the drawing
import { onMounted, onUnmounted } from 'vue'
import { onContentUpdated } from 'vitepress'

export default {
	setup() {
		let sizes = null//watches the page's width, made on mount because setup also runs in the build, where there's no browser
		let observer = null//watches this page's breakouts come on and off the screen
		let showing = new Set()//the breakouts on screen right now

		function measure() {//how far the article's region reaches past the prose column on the right, across the outline's column; 0 while VitePress hides the outline, below 1280px
			let column = document.querySelector('.VPDoc .content-container')//the prose column
			let region = document.querySelector('.VPDoc .container')//the article's region: the box the prose column is centered in, and the outline
			let aside = document.querySelector('.VPDoc .aside')
			let reach = 0
			if (column && region && aside && getComputedStyle(aside).display != 'none') reach = Math.max(0, region.getBoundingClientRect().right - column.getBoundingClientRect().right)
			document.documentElement.style.setProperty('--breakout-reach', `${reach}px`)
		}

		function watchBreakouts() {
			observer?.disconnect()
			showing.clear()
			document.documentElement.classList.remove('breakout-in-view')
			let breakouts = document.querySelectorAll('.vp-doc .breakout')
			if (breakouts.length == 0) return
			measure()
			observer = new IntersectionObserver(entries => {
				for (let entry of entries) {
					if (entry.isIntersecting) showing.add(entry.target)
					else showing.delete(entry.target)
				}
				document.documentElement.classList.toggle('breakout-in-view', showing.size > 0)
			})
			for (let breakout of breakouts) observer.observe(breakout)
		}

		onContentUpdated(watchBreakouts)//VitePress moves between pages without reloading, and calls this as each page's content lands
		onMounted(() => {
			sizes = new ResizeObserver(measure)//the page's width changes with the window, and also when a vertical scrollbar appears and takes its share, which no resize event reports
			sizes.observe(document.documentElement)
			watchBreakouts()
		})
		onUnmounted(() => {
			sizes?.disconnect()
			observer?.disconnect()
		})
		return () => null
	},
}
