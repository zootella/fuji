import styleText from '../src/style.css?raw'//the stylesheet as text, which vite hands over for any import marked raw

/*
Every color style.css names, read from its text as the color page loads.

The page loads the stylesheet as the app does, and that alone would leave colors out: Tailwind writes into the page only the colors something under src names, so a numbered gray nothing paints yet would be missing. So this module sets every color on the root again, exactly as the stylesheet declares it, light-dark() and all. A light-dark() pair resolves by the color-scheme of the element that paints it, which is how the page draws light and dark side by side: a pane that sets color-scheme light or dark draws every one of fuji's colors in that mode, and a bar or a menu inside it with them.

It holds no color of its own, so the page shows whatever the palette says today.
*/

export const paletteColors = [...styleText.matchAll(/^\s*--color-([a-z0-9-]+):\s*(.+?);\s*(?:\/\*\s*(.*?)\s*\*\/)?\s*$/gm)].map(m => ({name: m[1], value: m[2], note: m[3] || ''}))//each as its name without --color-, its value as written, and the comment beside it
for (let c of paletteColors) document.documentElement.style.setProperty(`--color-${c.name}`, c.value)

export function paletteHalf(value, mode) {//the light or the dark half of a light-dark() pair, or the whole value where it is one color in both
	if (!value.startsWith('light-dark(')) return value
	let inner = value.slice('light-dark('.length, -1), depth = 0
	for (let i = 0; i < inner.length; i++) {
		if (inner[i] == '(') depth++
		if (inner[i] == ')') depth--
		if (inner[i] == ',' && depth == 0) return (mode == 'light' ? inner.slice(0, i) : inner.slice(i + 1)).trim()
	}
	return value
}

export const palettePlatforms = {}//the colors each platform's rule in style.css points elsewhere, by the platform it names, and under a blank for the rule every platform has, read from the rules' one-line text
for (let [, platform, body] of styleText.matchAll(/^\s*\[data-platform(?:="([a-z]+)")?\]\s*\{(.*)\}\s*$/gm)) {
	let names = palettePlatforms[platform || ''] = {}
	for (let [, name, value] of body.matchAll(/--color-([a-z0-9-]+):\s*([^;]+?)\s*(?:;|$)/g)) names[name] = value
}

export function paletteNotch(name, platform, mode) {//the numbered gray a color comes to on this platform in this mode, following the names it is written as, or blank where it ends at a value written out, so a page says gray-250 only where the stylesheet does
	let value = palettePlatforms[platform]?.[name] ?? palettePlatforms['']?.[name] ?? paletteColors.find(c => c.name == name)?.value
	if (!value) return ''
	let named = paletteHalf(value, mode).match(/^var\(--color-([a-z0-9-]+)\)$/)
	if (!named) return ''
	return /^gray-\d+$/.test(named[1]) ? named[1] : paletteNotch(named[1], platform, mode)
}

export function paletteValue(name, mode) {//what the color of this name is in light or in dark, as the stylesheet writes it, following a color that names another to the value it ends at
	let c = paletteColors.find(c => c.name == name)
	if (!c) throw new Error(`no color named ${name} in style.css`)
	let value = paletteHalf(c.value, mode)
	let named = value.match(/^var\(--color-([a-z0-9-]+)\)$/)
	return named ? paletteValue(named[1], mode) : value
}
