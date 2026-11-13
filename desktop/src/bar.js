import barsText from './bars.txt?raw'//the text as it is, which vite hands over for any import marked raw
import arrowDownUp from 'lucide-static/icons/arrow-down-up.svg?raw'//the roster of lucide's drawings the bars name, below
import arrowLeft from 'lucide-static/icons/arrow-left.svg?raw'
import arrowRight from 'lucide-static/icons/arrow-right.svg?raw'
import arrowUp from 'lucide-static/icons/arrow-up.svg?raw'
import clipboardPaste from 'lucide-static/icons/clipboard-paste.svg?raw'
import copy from 'lucide-static/icons/copy.svg?raw'
import folderTree from 'lucide-static/icons/folder-tree.svg?raw'
import info from 'lucide-static/icons/info.svg?raw'
import layoutGrid from 'lucide-static/icons/layout-grid.svg?raw'
import scissors from 'lucide-static/icons/scissors.svg?raw'
import trash2 from 'lucide-static/icons/trash-2.svg?raw'
import undo2 from 'lucide-static/icons/undo-2.svg?raw'
import wrench from 'lucide-static/icons/wrench.svg?raw'

/*
Every bar fuji draws is written in one plain text, bars.txt, and no JavaScript assembles one. A component asks for a bar by its name and hears back the word of the item the user chose, so adding an item, moving one or changing its wording is an edit to that text and nothing else.

Each bar starts at a markdown heading, which is its name, a nickname for the code and the text that the user never sees. Its first line says which of four kinds it is, and the kind fixes which way it runs and how it looks, so a bar cannot be written running the wrong way:

menubar  the main menu, words across, each opening a menu
toolbar  buttons across, each big with its drawing above its words, or small with them side by side
menu     narrow rows down, the menu a title or a button opens and the menu a right click opens
panel    controls the page draws itself in Vue, checkboxes and radio buttons and whatever else, which a button opens as it opens a menu; it has no lines of its own in the text, which names it so a button can point at it

Every line after the kind is an item, or --- for a separator between groups. An item is its label, with & before the letter a key will choose it by, and then, in any order, big for a toolbar button with its drawing above its words, and the name of a lucide drawing, lowercase with dashes. Those words are told apart by their kind rather than by where they stand, so lining them up in columns is for the eye alone. Last on the line, the name of another bar in square brackets, as markdown writes a link to a heading, makes the item open that bar, a menu or a panel. Lines before the first heading are the file's own words to whoever opens it, and blank lines are skipped.

The text is read once, as this module loads, so a mistake in it, like a drawing the roster lacks, two bars of one name, or a bracket naming no menu or panel, throws as fuji starts and names the line, rather than the first time someone opens that bar.

The word an item sends is its label without the &. A label that differs by platform, Exit on windows and Quit elsewhere, would send two words for one command, and is where an item gets a word of its own, when one first needs it. The & letter is kept with the item, and nothing draws or hears it yet.
*/

export const barIcons = {//every drawing a bar names, imported one by one so only these reach the build, since a bundler cannot see which names a string holds; each is lucide's own markup, bundled, never anything from outside
	'arrow-down-up': arrowDownUp,
	'arrow-left': arrowLeft,
	'arrow-right': arrowRight,
	'arrow-up': arrowUp,
	'clipboard-paste': clipboardPaste,
	'copy': copy,
	'folder-tree': folderTree,
	'info': info,
	'layout-grid': layoutGrid,
	'scissors': scissors,
	'trash-2': trash2,
	'undo-2': undo2,
	'wrench': wrench,
}

const barKinds = ['menubar', 'toolbar', 'menu', 'panel']
export function barParse(text) {//the text in, each bar by its name out, or a throw that names the line
	let bars = {}, bar = null
	for (let [n, line] of text.split(/\r?\n/).entries()) {
		line = line.trim()
		let where = `line ${n + 1}: ${line}`
		if (line.startsWith('#')) {
			let name = line.replace(/^#+\s*/, '')
			if (!name || bars[name]) throw new Error(`expected a heading naming a bar not named before, ${where}`)
			bar = bars[name] = {name, kind: '', items: []}
		}
		else if (!bar || !line) continue//before the first heading, or blank
		else if (!bar.kind) {
			if (!barKinds.includes(line)) throw new Error(`expected ${barKinds.join(', ')} to start ${bar.name}, ${where}`)
			bar.kind = line
		}
		else if (bar.kind == 'panel') throw new Error(`expected nothing under a panel, which the page draws, ${where}`)
		else if (line == '---') bar.items.push({separator: true})
		else bar.items.push(_barItem(line, bar.kind, where))
	}
	for (let bar of Object.values(bars)) {//after every bar is read, since a menu may be written below the button that opens it
		if (!bar.kind) throw new Error(`expected ${barKinds.join(', ')} to start ${bar.name}`)
		for (let item of bar.items) {
			if (item.separator) continue
			let opens = bars[item.opens]
			if (item.opens && opens?.kind != 'menu' && opens?.kind != 'panel') throw new Error(`expected a menu or a panel named ${item.opens}, ${item.where}`)
			if (bar.kind == 'menubar' && opens?.kind != 'menu') throw new Error(`expected a menu for each title of a menubar to open, ${item.where}`)
		}
	}
	return bars
}

function _barItem(line, kind, where) {//one item's line: the name in brackets at its end of the bar it opens, then the words at its end that are big or a drawing's name, taken from the right, and the label all that is left, so a label of one word is never read as one of them
	let item = {big: false, icon: '', opens: '', where}
	let link = line.match(/\s*\[([^\]]+)\]$/)
	if (link) { item.opens = link[1].trim(); line = line.slice(0, link.index) }
	let words = line.split(/\s+/)
	while (words.length > 1) {
		let last = words.at(-1)
		if (last == 'big') {
			if (item.big) throw new Error(`expected big once, ${where}`)
			if (kind != 'toolbar') throw new Error(`expected big only on a toolbar's button, ${where}`)
			item.big = true
		} else if (/^[a-z0-9]+(-[a-z0-9]+)*$/.test(last)) {//lowercase with dashes, as lucide names its drawings
			if (item.icon) throw new Error(`expected one drawing, ${where}`)
			if (!barIcons[last]) throw new Error(`expected a drawing in the roster in bar.js, ${where}`)
			item.icon = last
		} else break
		words.pop()
	}
	if (item.big && !item.icon) throw new Error(`expected a drawing for a big button to show above its words, ${where}`)
	let label = words.join(' ')
	let at = label.indexOf('&')
	if (at != label.lastIndexOf('&') || at == label.length - 1) throw new Error(`expected one & before a letter, ${where}`)
	item.key = at == -1 ? '' : label[at + 1].toLowerCase()
	item.label = label.replace('&', '')
	item.word = item.label
	return item
}

const bars = barParse(barsText)
export function barGet(name) {//the bar of this name, as bars.txt says it
	if (!bars[name]) throw new Error(`no bar named ${name} in bars.txt`)
	return bars[name]
}
