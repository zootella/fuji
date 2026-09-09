// https://vitepress.dev/guide/custom-theme
import DefaultTheme from 'vitepress/theme'
import HomePage from './components/HomePage.vue'
import './style.css'

/*
The default theme, deliberately unmodified. The documentation pages are stock VitePress — its colors, its Inter — and that is the intent, not an omission.

The one addition is HomePage, registered globally so index.md can place it with layout: false. That page replaces the theme entirely rather than restyling it, which is why nothing here overrides a theme variable.
*/

/** @type {import('vitepress').Theme} */
export default {
	extends: DefaultTheme,
	enhanceApp({ app }) {
		app.component('HomePage', HomePage)
	},
}
