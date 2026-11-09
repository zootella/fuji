// https://vitepress.dev/guide/custom-theme
import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import HomePage from './components/HomePage.vue'
import DownloadLink from './components/DownloadLink.vue'
import DownloadCommand from './components/DownloadCommand.vue'
import CreativeCommons from './components/CreativeCommons.vue'
import Breakouts from './breakout.js'
import './style.css'

/*
VitePress's default theme, extended rather than replaced. style.css changes it through the theme's own --vp-* variables wherever one exists, so the parts no rule of ours ever names change with them, and writes a rule of its own only where no variable reaches. Prose stays Inter, which is VitePress's default; code is IBM Plex Mono, which is not. That file carries the reasoning for all of it.

The layout places CreativeCommons and Breakouts on every article through the theme's doc-after slot, after the previous and next links: CreativeCommons is the license line, the last thing on a page, and Breakouts draws nothing and watches the page for blocks marked to break out of the prose column. The home page carries no layout, so nothing in that slot reaches it. The components registered below are the ones markdown places. HomePage is the whole of index.md, which carries layout: false — VitePress renders no navbar, sidebar or footer there, so that page replaces the theme rather than restyling it, which is why it names its own fonts and paints its own background instead of reading theme variables. DownloadLink is one installer's box, placed among the download page's prose; unlike HomePage it styles itself from the theme's variables rather than against them. DownloadCommand wraps a fenced install command on the download page and writes the current hash into it when the page opens; the fence itself is ordinary markdown, highlighted during the build like every other.
*/

/** @type {import('vitepress').Theme} */
export default {
	extends: DefaultTheme,
	Layout: () => h(DefaultTheme.Layout, null, {
		'doc-after': () => [h(CreativeCommons), h(Breakouts)],// https://vitepress.dev/guide/extending-default-theme#layout-slots
	}),
	enhanceApp({ app }) {
		app.component('HomePage', HomePage)
		app.component('DownloadLink', DownloadLink)
		app.component('DownloadCommand', DownloadCommand)
	},
}
