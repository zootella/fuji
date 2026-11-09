import {iconRender} from './icon.js'
import {fileTypesIcon} from './fileTypes.js'
import {platform} from './components/library.js'
import {logTrouble} from './log.js'

/*
The system's icons for the list and the tree, each asked for once and drawn into every row that shows it by one rule.

An icon says what kind a file is and never shows what is inside it. Fuji asks for an icon by kind and never by path, so the system opens no file to answer and hands back no thumbnail, and a file given an icon of its own shows its kind's icon anyway; the contact sheet is where fuji shows what a picture holds.

A kind fuji opens shows its medium's icon, a picture, a video or a sound, rather than the icon of whatever app is installed to open it elsewhere, since a double-click on it in fuji opens it in fuji: an mkv on a Mac with VLC would otherwise wear VLC's. Every other kind shows the icon for its extension, which is the icon of the app a double-click hands it to, so a spreadsheet wears Excel's. fileTypesIcon in fileTypes.js says which kinds those are on each platform, and copper's icon.rs where each platform finds a medium's icon.

The icon arrives as a png at the square's size in css pixels times devicePixelRatio, read when the icon is first asked for: 32 backing pixels on a Retina Mac, 16 on Windows at 100 percent and on a Mac's 1x monitor, 20 at 125 percent. A window that moves to a display with another ratio keeps the icons it first got until fuji starts again, as thumbnails do.

A list can hold ten thousand rows, so no row holds an image of its own. Each icon gets a number, the folder's, each medium's, and each other extension's, a row carries its icon's number on its square, and when the icon arrives one css rule shows it on every square with that number, rows made before it arrived included, from one decode the engine shares. The rule names the number rather than the extension, which is text nobody vouched for and could break out of a selector. An icon the system cannot give leaves the square's cyan, and so does every icon on linux, where fuji asks no system for icons yet.

The square goes where the layout puts it. At a fractional ratio it can start on half a backing pixel, 2.5 down its row at 125 percent, and the engine snaps plain layout to whole backing pixels where it would leave a transform between them, so the square takes no transform and nothing nudges it: a nudge would push one way at one ratio and the other way at the next, and nobody could check it on every screen users run.

Each icon is asked for once per window and kept while the window lives. A change between light and dark asks for nothing again, since the system's icons are the same pixels in both, measured on the Mac mini, and only what is behind them repaints. An icon the system changes while fuji runs, as when an upgrade of Excel brings a new one for its files, shows after fuji starts again, since noticing that and redrawing would add a great deal for a corner that narrow. A few dozen icons asked for once each wait in no governor's line, where they would sit behind the contact sheet's thumbnails for nothing.
*/

const iconCss = 16//the square the list and the tree draw an icon in, css pixels on a side, as their styles size it
const iconNumbers = new Map()//an icon to the number its squares carry: folder, image, video or audio, or an extension with its dot, so no extension is mistaken for the others
let iconSheet = null//where each icon's rule goes, made when the first icon arrives. A stylesheet made in script and adopted by the document rather than a style element the page makes, because an installed fuji on the Windows box gave such an element no sheet at all on 2026-11-09, every icon failing on a sheet of null after copper had answered, where the Mac's gave it one. An element gets no sheet when a content security policy refuses it, and fuji's names no style-src, so its default-src 'self' covers styles; a stylesheet made in script is the page's own object, which the policy does not govern. The document places adopted sheets after every sheet of its own, so these rules win over the square's cyan

export function iconFor({folder = false, extension = ''}) {//the number a row carries on its icon square for a folder or a file of this extension, answered at once; the first time an icon is asked for, it is asked of the system, and every square with the number shows it when it comes
	let lower = extension.toLowerCase()
	let kind = iconKind(folder, lower)
	let key = kind == 'file' ? `.${lower}` : kind
	let number = iconNumbers.get(key)
	if (number) return number
	number = iconNumbers.size + 1
	iconNumbers.set(key, number)
	if (platform() != 'linux') iconAsk(number, kind, kind == 'file' ? lower : '')//not awaited: the row is made now, and its square fills when the icon comes
	return number
}

function iconKind(folder, extension) {//the icon a folder or a file of this extension shows: the folder's, its medium's as fileTypesIcon answers, or file, the icon for its extension
	if (folder) return 'folder'
	return fileTypesIcon(`.${extension}`, platform()) || 'file'
}

async function iconAsk(number, kind, extension) {
	try {
		let png = await iconRender({kind, extension, css: iconCss, backingPerCss: devicePixelRatio})
		let url = URL.createObjectURL(new Blob([png], {type: 'image/png'}))//never revoked, since the rule shows it for as long as the window lives
		if (!iconSheet) { iconSheet = new CSSStyleSheet(); document.adoptedStyleSheets = [...document.adoptedStyleSheets, iconSheet] }
		iconSheet.insertRule(`[data-icon="${number}"] { background: url("${url}") center / 100% 100% no-repeat }`, iconSheet.cssRules.length)//the shorthand clears the cyan as it sets the picture
	} catch (error) {
		logTrouble(`icon: asking for ${kind == 'file' ? `.${extension}` : kind}`, error)//and the square keeps its cyan
	}
}
