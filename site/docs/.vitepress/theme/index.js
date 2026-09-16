// https://vitepress.dev/guide/custom-theme
import DefaultTheme from 'vitepress/theme'
import HomePage from './components/HomePage.vue'
import Download from './components/Download.vue'
import './style.css'

/*
The default theme, deliberately unmodified. The documentation pages are stock VitePress — its colors, its Inter — and that is the intent, not an omission.

Two components are registered globally so markdown can place them. HomePage is the whole of index.md, which carries layout: false — that page replaces the theme entirely rather than restyling it, which is why nothing here overrides a theme variable. Download is one installer's row, and the download page places three of them among its prose; it styles itself from the theme's own variables rather than against them.
*/

/** @type {import('vitepress').Theme} */
export default {
	extends: DefaultTheme,
	enhanceApp({ app }) {
		app.component('HomePage', HomePage)
		app.component('Download', Download)
	},
}
