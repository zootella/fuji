// https://vitepress.dev/guide/custom-theme
import DefaultTheme from 'vitepress/theme'
import HomePage from './components/HomePage.vue'
import DownloadLink from './components/DownloadLink.vue'
import DownloadCommand from './components/DownloadCommand.vue'
import './style.css'

/*
VitePress's default theme, extended rather than replaced. Every layout, every component and every behaviour on a documentation page is stock, and that is the intent rather than an omission. What style.css changes is the color system, and it changes it through the theme's own --vp-* variables wherever one exists, so the parts no rule of ours ever names recolor themselves. Prose stays Inter, which is VitePress's default; code is IBM Plex Mono, which is not. That file carries the reasoning for all of it.

Three components are registered globally so markdown can place them. HomePage is the whole of index.md, which carries layout: false — VitePress renders no navbar, sidebar or footer there, so that page replaces the theme rather than restyling it, which is why it names its own fonts and paints its own background instead of reading theme variables. DownloadLink is one installer's box, and the download page places six of them among its prose; unlike HomePage it styles itself from the theme's variables rather than against them. DownloadCommand wraps a fenced install command on the download page, which places two, and writes the current hash into it when the page opens; the fence itself is ordinary markdown, highlighted during the build like every other.
*/

/** @type {import('vitepress').Theme} */
export default {
	extends: DefaultTheme,
	enhanceApp({ app }) {
		app.component('HomePage', HomePage)
		app.component('DownloadLink', DownloadLink)
		app.component('DownloadCommand', DownloadCommand)
	},
}
