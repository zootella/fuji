import {homeDir} from '@tauri-apps/api/path'
import {parse as parseToml} from 'smol-toml'//parseToml, so the name parse stays free for path-browserify below
import parse from 'path-browserify'
import {diskRead, diskWrite} from './disk.js'
import {desktopExitHold} from './desktop.js'
import {forwardize} from './components/library.js'
import {logTrouble, sayTrouble} from './log.js'//for a line after startup; the ones from during the load are handed back to the shell instead, because the file being read is the one that says whether fuji keeps a log at all
import {brandName, brandStem} from './brand.js'
import {fitNames} from './fit.js'

const settingsFileName = `${brandStem}.toml`//fuji.toml, in the user's home folder for now; the per-platform config folders are a later decision, and a portable copy carrying its own is not one, since fuji is always installed
const settingsHeader = `# ${settingsFileName} — ${brandName} reads this file when it starts and writes it when it closes; edit the values freely, but the comments and the layout are regenerated every time, so notes of your own here will not survive`

export const settingsThumbnailBeams = ['Small', 'Medium', 'Large', 'Xl']//the named beam lengths; each names the setting below it, lowercased, and the settings panel offers them in this order
const settingsTextList = value => value.every(item => typeof item == 'string')//a list whose every item is text; what each item means is for the code reading the list to judge, one item at a time, so a typo in one never throws away the rest

//every setting fuji has, and the only place any of them is defined; a check, where the type alone isn't enough, has to accept the factory value or an ordinary file would report a problem against itself
const settingsSchema = [
	{
		section: 'view',
		key: 'table',
		factory: 'Diamond',
		comment: `which table was showing: Diamond sizes an image into an invisible diamond on an infinite plane, Comic runs it full width down a scroll; the tables ${brandName} has are known to the shell rather than here, so a name it does not recognize is reported there and Diamond shown instead`,
	}, {
		section: 'sheet',
		key: 'width',
		factory: 0,
		comment: `the size of the contact sheet window as you last left it, in css pixels, recorded as you resize it; ${brandName} opens the next one at this size, somewhere at random, whenever it fits the desktop less its menu bar, dock or taskbar, and at a portion of that desktop when it does not. A width of 0 means nothing has been recorded yet`,
	}, {
		section: 'sheet',
		key: 'height',
		factory: 0,
	}, {
		section: 'sheet',
		key: 'maximized',
		factory: false,
		comment: 'whether you left the contact sheet maximized, which on a mac is zoomed, so the next one opens that way; the size above stays the one it had before, which is where restoring it goes',
	}, {
		section: 'sheet',
		key: 'buckets',
		factory: 10,
		comment: `how many buckets a page of the contact sheet holds, so this times the images in a bucket is the most thumbnails ${brandName} ever has at once; Previous and Next move the page along every image on the disk, a page at a time`,
		check: value => Number.isInteger(value) && value >= 1,
	}, {
		section: 'sort',
		key: 'order',
		factory: 'Alphabet',
		comment: `which order ${brandName} puts a folder in: Alphabet is the order javascript itself puts an array of names in, capitals before lowercase and page10 before page9, with no locale and no opinion; the sorts ${brandName} has are known to the model rather than here, so a name it does not recognize is reported there and Alphabet used instead`,
	}, {
		section: 'bucket',
		key: 'images',
		factory: 200,
		comment: 'how many images one bucket holds. A bucket never mixes two folders, and the sheet scrolls over buckets rather than over the thumbnails themselves, which is what lets it page through a whole drive in constant memory; a bucket large enough to hold a typical folder whole makes every bucket on a page a different folder',
		check: value => Number.isInteger(value) && value >= 1,
	}, {
		section: 'thumbnail',
		key: 'fit',
		factory: 'SquareFit',
		comment: `how every thumbnail is sized against the beam below: ${fitNames.join(', ')}. SquareFit lays the beam across a picture's longer side, which is what Finder and Explorer do; fit.js says what each of the others does. One choice for the whole sheet, so changing it changes every bucket at once`,
		check: value => fitNames.includes(value),
	}, {
		section: 'thumbnail',
		key: 'beam',
		factory: 'Small',
		comment: 'which of the beam lengths below every thumbnail is measured against; one choice for the whole sheet, so changing it changes every bucket at once',
		check: value => settingsThumbnailBeams.includes(value),
	}, {
		section: 'thumbnail',
		key: 'small',
		factory: 120,
		comment: 'what each beam length means, in css pixels: the width a 4:3 picture, like an 800 by 600 screenshot or a phone photograph, comes out at under every fit but RowFit, ScaleFit and LogFit, which is what every other shape is measured against; under RowFit it is the height of every thumbnail. An image already smaller than its fit is left at its own size rather than blown up. Nothing is tuned to these particular numbers, so raise them all or just the one you live in',
		check: value => Number.isInteger(value) && value >= 1,
	}, {
		section: 'thumbnail',
		key: 'medium',
		factory: 240,
		check: value => Number.isInteger(value) && value >= 1,
	}, {
		section: 'thumbnail',
		key: 'large',
		factory: 360,
		check: value => Number.isInteger(value) && value >= 1,
	}, {
		section: 'thumbnail',
		key: 'xl',
		factory: 480,
		check: value => Number.isInteger(value) && value >= 1,
	}, {
		section: 'flip',
		key: 'back',
		factory: 5,
		comment: `how many images before the one on screen a table keeps decoded, so flipping back to them is instant instead of a fresh read and decode; one is the smallest that works, because a table always holds the image on either side of the one it is showing, and one here with one forward is the behaviour ${brandName} had before it kept a window`,
		check: value => Number.isInteger(value) && value >= 1,
	}, {
		section: 'flip',
		key: 'forward',
		factory: 5,
		comment: 'and how many after it; flipping forward is the common direction, so this is the one to raise first if a folder of large images still makes the user wait',
		check: value => Number.isInteger(value) && value >= 1,
	}, {
		section: 'zoom',
		key: 'drag',
		factory: 100,
		comment: 'how many css pixels a right drag travels to zoom one notch, the same notch the wheel and + or - zoom, a little under one and a half times; up zooms in and down out, six notches are exactly ten times, and dragging back to where the drag began restores what you had. It zooms about the point where the drag began',
		check: value => value > 0,
	}, {
		section: 'gamma',
		key: 'key',
		factory: 1.22,//the ratio of a modern screen's 2.2 to the classic Mac's 1.8, so one press shows a 1990s Mac picture as its maker saw it; the gamma page on fuji's site has the arithmetic. A first 1.2 and then 1.25 were picked by feel before the number had a reason
		comment: `how far one press of g lifts the shadows, in the convention picture viewers use: 1 changes nothing, 1.22 shows a picture made on a Mac before 2009 as its maker saw it, since a screen today is sRGB at about 2.2 and the Mac of that era was 1.8, and 2.2 divided by 1.8 is 1.22; it is also a gentle lift for any other picture. 2 floods the shadows so nothing dark stays hidden; black stays black and white stays white either way. A second press returns to normal, and so does g from any gamma the keys, the wheel or the drag below have set. ${brandName} always starts at normal`,
		check: value => value > 0,
	}, {
		section: 'gamma',
		key: 'step',
		factory: 0.2,
		comment: 'how much one press of shift and plus raises the gamma, or shift and minus lowers it, from wherever it is',
		check: value => value > 0,
	}, {
		section: 'gamma',
		key: 'wheel',
		factory: 0.1,
		comment: 'and how much one notch of the wheel does with shift held on a table, away from you to raise it; smaller than a key press, because a wheel is easy to flick several notches at once',
		check: value => value > 0,
	}, {
		section: 'gamma',
		key: 'drag',
		factory: 5,
		comment: 'how much gamma a shift and right drag on a table adds over the whole height of the frame, up to brighten and down to darken; it counts only the height from where the drag began, like a slider laid up the frame, so at 5 a fifth of the frame takes 1 to 2, the same as five steps of 0.2',
		check: value => value > 0,
	}, {
		section: 'gamma',
		key: 'floor',
		factory: 0.2,
		comment: 'the lowest gamma the keys, the wheel and the drag reach: below 1 darkens, which is a way to check the blacks, and a floor keeps a long drag down from reaching 0, which would turn the whole picture black',
		check: value => value > 0,
	}, {
		section: 'pan',
		key: 'step',
		factory: -0.25,
		comment: 'how far an arrow key pans, as a fraction of the frame\'s shorter side, and which way: negative moves your view the way the arrow points, so the picture slides the other way, as in every viewer with arrow keys; positive moves the picture the way the arrow points, as dragging it would',
		check: value => value != 0,
	}, {
		section: 'hud',
		key: 'help',
		factory: true,
		comment: `show the help panel in the middle of the window, the one [h] toggles in every view; on at the factory so it greets a new user, and ${brandName} writes this back as you turn it on and off, so it comes back the way you left it`,
	}, {
		section: 'hud',
		key: 'information',
		factory: false,
		comment: 'show the information panel along the bottom of the frame, the one [i] toggles; off at the factory, since the help panel names the key, and written back the same way',
	}, {
		section: 'hud',
		key: 'caption',
		factory: true,
		comment: 'show the caption beneath the image at startup',
	}, {
		section: 'font',
		key: 'faces',
		factory: 'system',
		comment: `which fonts ${brandName} sets its text in. ${brandStem} uses the two ${brandName} carries, Inter with IBM Plex Mono for the fixed-width text, so it looks the same on every computer. system uses the computer's own, so ${brandName} looks at home on it: Segoe UI with Consolas on Windows, and San Francisco with SF Mono on a mac. The fixed-width text is every hud and panel, and the caption beneath a picture; the settings panel changes this too`,
		check: value => ['system', brandStem].includes(value),
	}, {
		section: 'appearance',
		key: 'mode',
		factory: 'system',
		comment: `light, dark, or system: light or dark keeps ${brandName} that way whatever the computer is set to, and system matches the computer's own light or dark setting, and changes when it does. The settings panel changes this too`,
		check: value => ['light', 'dark', 'system'].includes(value),
	}, {
		section: 'associations',
		key: 'yes',
		factory: [],
		comment: `which kinds of file ${brandName} opens when you double-click one, as three lists of extensions: yes, no, and ask, which means you have not decided. ${brandName} is offered for every one of them whatever you answer, and yes is what makes it the program that opens them; the system keeps the final say, and ${brandName}\'s settings show where it disagrees and take you to where it can be changed. Every extension belongs in exactly one list: one left out or listed twice counts as ask, and one ${brandName} does not open is dropped. Easiest changed in ${brandName}\'s settings, where ${brandName} does the rest. On a Mac these lists go unused, since the Mac keeps the whole answer itself, which ${brandName}\'s settings and Get Info in the Finder both change`,
		check: settingsTextList,
	}, {
		section: 'associations',
		key: 'no',
		factory: [],
		check: settingsTextList,
	}, {
		section: 'associations',
		key: 'ask',
		factory: [],//empty rather than every extension fuji opens, since one no list names already counts as ask; associate.js fills this in at startup, which is also how an extension a later fuji adds arrives here undecided
		check: settingsTextList,
	}, {
		section: 'log',
		key: 'record',
		factory: false,
		comment: `write the log: every image load, every flip and every thumbnail with what each cost, and any line the code chose to keep, from the page or from rust, saved when ${brandName} closes into ${brandStem}-temp under your home folder, which ${brandName} makes if it is missing; off by default because it is for answering a question rather than for running the app, and the file's own header says what each column means`,
	},
]

export const settings = settingsFactory()//the live settings the rest of fuji reads, filled in at startup and never replaced, so an importer keeps the same object

let settingsFilePath = ''//where the file is, found once at startup
let settingsFileText = ''//what fuji last read from or wrote to the file, to tell when a write would change nothing
let settingsHeldText = ''//the text rust's held copy corresponds to, so an unchanged render doesn't cross to it again

function settingsFactory() {//a settings object with every value at its factory setting
	let s = {}
	for (let entry of settingsSchema) (s[entry.section] ??= {})[entry.key] = Array.isArray(entry.factory) ? [...entry.factory] : entry.factory//a list of its own, so changing a setting never changes the schema's factory list
	return s
}

function settingsSameType(value, factory) {//a value has the shape of its factory value: a list where the factory is a list, and otherwise the same typeof. typeof alone says object for a list and for a toml table alike
	if (Array.isArray(factory)) return Array.isArray(value)
	return typeof value == typeof factory && !Array.isArray(value)
}

function settingsParse(text) {//the settings the given file text describes, plus a list of anything in it fuji had to turn away
	let settings = settingsFactory()//only a usable value in the file replaces one of these
	let problems = []
	let parsed
	try {
		parsed = parseToml(text)
	} catch (error) {
		problems.push(`could not read the file, ${error.message}`)
		return {settings, problems}//nothing in there is usable, so everything stays factory and the next render repairs the file
	}

	for (let entry of settingsSchema) {
		let value = parsed[entry.section]?.[entry.key]
		if (value == undefined) continue//not in the file, which is ordinary; rendering puts the line back
		let name = `${entry.section}.${entry.key}`//only for the two complaints below
		if (!settingsSameType(value, entry.factory)) { problems.push(`${name} has to be ${sayType(entry.factory)}, so ${sayValue(value)} was ignored`); continue }
		if (entry.check && !entry.check(value))      { problems.push(`${name} cannot be ${sayValue(value)}, so it was ignored`);                        continue }
		settings[entry.section][entry.key] = value
	}
	for (let [section, table] of Object.entries(parsed)) {//the file lists every setting fuji has, so a name fuji doesn't know is a typo rather than a default quietly showing through, and worth saying out loud
		if (typeof table != 'object' || Array.isArray(table)) { problems.push(`${section} is not a ${brandName} setting`); continue }
		for (let key of Object.keys(table)) {
			if (!settingsSchema.some(entry => entry.section == section && entry.key == key)) problems.push(`${section}.${key} is not a ${brandName} setting`)
		}
	}
	return {settings, problems}
}

function settingsRender(settings) {//the complete text of the file for these settings, and the only place that text ever comes from
	let lines = [settingsHeader]
	for (let section of new Set(settingsSchema.map(entry => entry.section))) {
		let entries = settingsSchema.filter(entry => entry.section == section)
		let keyWidth   = Math.max(...entries.map(entry => entry.key.length))//pad within the section, so a long name in one doesn't push the others out
		let valueWidth = Math.max(0, ...entries.filter(entry => !Array.isArray(entry.factory)).map(entry => sayValue(settings[section][entry.key]).length))//a list is left out and never padded, since one can run to dozens of items and would push every line beside it out to its width

		lines.push('', `[${section}]`)
		for (let entry of entries) {
			if (entry.comment) lines.push(`# ${entry.comment}`)//above the setting, not trailing it: these run long, and a soft wrapped comment beside a value would fold across the next line
			let value = sayValue(settings[section][entry.key])
			if (!Array.isArray(entry.factory)) value = value.padEnd(valueWidth)
			lines.push(`${entry.key.padEnd(keyWidth)} = ${value} # factory ${sayValue(entry.factory)}`)
		}
	}
	return lines.join('\n')+'\n'
}

function sayValue(value) {//a value as the toml text that means it
	if (typeof value == 'boolean') return value ? 'true' : 'false'
	if (typeof value == 'number')  return String(value)
	if (Array.isArray(value))      return `[${value.map(sayValue).join(', ')}]`//a flat list of the values above, the one shape of list the schema uses
	return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`//a basic string, escaping the backslash that starts an escape and the quote that would end it early
}

function sayType(value) {//the word for a value's type in a complaint, so a list reads as a list rather than as an object
	return Array.isArray(value) ? 'a list' : typeof value
}

export function settingsThumbnailBeam() {//the beam in css pixels, for the length the user chose; the one place the name becomes a number, so a flow asks rather than looks up
	return settings.thumbnail[settings.thumbnail.beam.toLowerCase()]
}

export function settingsSet(section, key, value) {//change one setting from inside fuji, held to the same type and check as a value read from the file; answers whether it took, and a value turned away leaves the setting as it was
	let entry = settingsSchema.find(entry => entry.section == section && entry.key == key)
	if (!entry) throw new Error(`no setting named ${section}.${key}`)//a mistake in the code asking, not something a user typed
	if (!settingsSameType(value, entry.factory)) return false
	if (entry.check && !entry.check(value))      return false
	settings[section][key] = value; settingsChanged()
	return true
}

export async function settingsLoad() {//read the settings file and leave it exactly as fuji would write it, which is what creates a missing one, repairs a bad value, and adds a setting fuji has gained since the last launch; call once, before anything reads a setting. Answers with the lines this load wants remembered, for the shell to log the moment it has started one
	settingsFilePath = parse.join(forwardize(await homeDir()), settingsFileName)
	let notices = []//what happened while reading the file, said here and logged by the shell a few lines later

	let text = ''
	let unreadable = false//a file that is there and will not open, as opposed to one that is not there at all
	try {
		text = new TextDecoder().decode(new Uint8Array(await diskRead(settingsFilePath)))
	} catch (error) {
		unreadable = !String(error).includes('os error 2')//both platforms number a missing file 2; anything else is a lock, a permission, or a disk saying no
		notices.push(`settings: ${unreadable ? 'leaving alone' : 'starting a new file at'} ${settingsFilePath}, because reading one said: ${error}`)
	}
	settingsFileText = text

	let {settings: found, problems} = settingsParse(text)
	for (let entry of settingsSchema) settings[entry.section][entry.key] = found[entry.section][entry.key]//fill the live object rather than replacing it, so importers keep theirs
	for (let problem of problems) notices.push(`settings: ${problem}`)

	let rendered = settingsRender(settings)
	if (unreadable) return notices//a file fuji could not read is one it must not overwrite: the settings in it are the user's and are still there, and writing factory values over them would be losing data to a lock
	if (rendered != settingsFileText) {//the file is missing, or held a value fuji had to repair, or came from a fuji with fewer settings than this one
		try {
			await diskWrite(settingsFilePath, Array.from(new TextEncoder().encode(rendered)))//disk.rs speaks bytes because it mirrors posix, and this is the one place fuji encodes; everywhere else text stays text
			settingsFileText = rendered//only once the write happened
		} catch (error) {
			notices.push(sayTrouble(`settings: writing ${settingsFilePath}`, error))
		}
	}
	settingsHeldText = settingsFileText
	settingsChanged()//a no-op after a write that worked; after one that didn't, this is what leaves the file with rust to try again on the way out
	return notices
}

export function settingsChanged() {//call after changing a value in settings, the way quiver() gets called after moving an arrow; hands the file down to rust, which writes it on the way out — desktop.rs has why only rust can see a quit coming
	if (!settingsFilePath) return//settingsLoad has not run, so there is nowhere to hand anything down to
	let text = settingsRender(settings)
	if (text == settingsHeldText) return//rust's view already matches, which is what a move event reporting the same position produces
	settingsHeldText = text
	desktopExitHold(settingsFilePath, text)//the whole file, and never blank even when it matches the disk: rust holds one text per path for the whole process, so on the mac a blank from one window would erase what another is waiting to write. Handing the whole file down is also what makes settings last-modified-wins with several windows open — the window someone changed something in most recently is the one whose view survives
		.catch(error => logTrouble('settings: handing the file down to rust', error))
}
