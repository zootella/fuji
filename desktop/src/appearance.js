import {getCurrentWindow} from '@tauri-apps/api/window'
import {settings} from './settings.js'
import {logTrouble} from './log.js'
import {brandName} from './brand.js'
import {platform} from './components/library.js'

//the look the settings name, put on the window and its page: light or dark and the faces, which every kind of window fuji makes puts on as it starts, before its reveal, and a browsing window again when its settings panel changes them; and the fill of a selected row, which a browsing window puts on as it starts

export async function themeShow() {//light or dark, for the window's title bar and the page inside, from the appearance setting. The theme on the window is the one switch, as the essay above the palette in style.css says: null follows the system, and keeps following it as it changes
	await getCurrentWindow().setTheme(settings.appearance.mode == 'system' ? null : settings.appearance.mode)
}

export async function facesShow() {//put the page's text in the fonts the settings name, which style.css reads off the root. The fonts fuji carries are loaded before the switch, so nothing on screen is drawn in the system's and then snaps to these: only the regular faces, which is all a window shows at first, and at any size, since only the face is being fetched. A file that will not load is logged, and its text is drawn in the system's face, which style.css names behind it
	let loads = {bundled: ['16px Inter', '16px "IBM Plex Mono"'], retro: ['16px "IBM Plex Mono"']}[settings.font.faces] || []//both bundled faces; retro borrows Plex alone, beside the Verdana the system installs; and system fetches nothing
	await Promise.all(loads.map(face => document.fonts.load(face))).catch(error => logTrouble(`appearance: loading ${brandName}'s fonts`, error))
	document.documentElement.dataset.faces = settings.font.faces//read again after the wait, so a quick change back is never overwritten by the load it interrupted; the setting's own word, system, bundled or retro, which style.css keys its rules on
}

export function selectionShow() {//the fill of a selected row: blank leaves the platform's defaults in style.css, and a color the user named replaces the fill, with whichever of white or black text reads on it. Fuji carries each platform's colors rather than reading the system's, since every engine fixes the Highlight color a page can ask for against fingerprinting, and Rust could read the real one only on the mac
	document.documentElement.dataset.platform = platform()//mac, windows or linux, which style.css picks the defaults by
	let root = document.documentElement.style
	let color = settings.selection.color
	if (!color) { root.removeProperty('--color-select'); root.removeProperty('--color-on-select'); return }
	root.setProperty('--color-select', color)
	root.setProperty('--color-on-select', _colorPale(color) ? 'var(--color-fuji-on-pale)' : 'var(--color-fuji-on-deep)')
}

function _colorPale(color) {//whether black text contrasts more with this color than white does, by the relative luminance WCAG defines
	let c = document.createElement('canvas').getContext('2d')
	c.fillStyle = color
	c.fillRect(0, 0, 1, 1)//painted and read back as srgb bytes, since fillStyle hands a color outside srgb, like an oklch() or a display-p3 one, back in its own notation rather than as hex
	let [r, g, b] = c.getImageData(0, 0, 1, 1).data
	let linear = v => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }
	let luminance = 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
	return (luminance + 0.05) / 0.05 > 1.05 / (luminance + 0.05)//black's contrast with the color, weighed against white's
}
