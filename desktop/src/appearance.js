import {getCurrentWindow} from '@tauri-apps/api/window'
import {settings} from './settings.js'
import {logTrouble} from './log.js'
import {brandName} from './brand.js'

//the look the settings name, put on the window and its page: light or dark and the faces, which every kind of window fuji makes puts on as it starts, before its reveal, and a browsing window again when its settings panel changes them. The accent was a setting too, until 2026-11-19; it is style.css's alone now, with no setting, as every color of a selection is

export async function themeShow() {//light or dark, for the window's title bar and the page inside, from the appearance setting. The theme on the window is the one switch, as the essay above the palette in style.css says: null follows the system, and keeps following it as it changes
	await getCurrentWindow().setTheme(settings.appearance.mode == 'system' ? null : settings.appearance.mode)
}

export async function facesShow() {//put the page's text in the fonts the settings name, which style.css reads off the root. The fonts fuji carries are loaded before the switch, so nothing on screen is drawn in the system's and then snaps to these: only the regular faces, which is all a window shows at first, and at any size, since only the face is being fetched. A file that will not load is logged, and its text is drawn in the system's face, which style.css names behind it
	let loads = {bundled: ['16px Inter', '16px "IBM Plex Mono"'], retro: ['16px "IBM Plex Mono"']}[settings.font.faces] || []//both bundled faces; retro borrows Plex alone, beside the Verdana the system installs; and system fetches nothing
	await Promise.all(loads.map(face => document.fonts.load(face))).catch(error => logTrouble(`appearance: loading ${brandName}'s fonts`, error))
	document.documentElement.dataset.faces = settings.font.faces//read again after the wait, so a quick change back is never overwritten by the load it interrupted; the setting's own word, system, bundled or retro, which style.css keys its rules on
}

