//./src/settings.js

import {homeDir} from '@tauri-apps/api/path'
import {parse as parseToml} from 'smol-toml'//parseToml, so the name parse stays free for path-browserify below
import parse from 'path-browserify'
import {diskRead, diskWrite} from './disk.js'
import {desktopExitHold} from './desktop.js'
import {forwardize} from './components/library.js'

const settingsFileName = 'fuji.toml'//in the user's home folder for now; portable installs and the per-platform config folders are a later decision
const settingsHeader = `# fuji.toml — fuji reads this file when it starts and writes it when it closes; edit the values freely, but the comments and the layout are regenerated every time, so notes of your own here will not survive`

//every setting fuji has, and the only place any of them is defined; a check, where the type alone isn't enough, has to accept the factory value or an ordinary file would report a problem against itself
const settingsSchema = [
	{section: 'view',       key: 'showing',     factory: 'Table',   comment: 'which kind of view fuji was showing when it last closed, so it opens there again: Sheet for the contact sheet, Table for whichever table', check: value => value == 'Sheet' || value == 'Table'},
	{section: 'view',       key: 'table',       factory: 'Diamond', comment: 'which table was showing: Diamond sizes an image into an invisible diamond on an infinite plane, Comic runs it full width down a scroll; the tables fuji has are known to the shell rather than here, so a name it does not recognize is reported there and Diamond shown instead'},
	{section: 'window',     key: 'remember',    factory: true, comment: 'remember the size and position of the window from launch to launch; when false, fuji sizes its window to a fraction of the desktop and lets the operating system place it'},
	{section: 'window',     key: 'x',           factory: 0,    comment: 'the window fuji last recorded, in physical pixels, and read only when remember is true; a width or height that is not positive means fuji has not recorded a window yet, and it sizes itself to the desktop instead'},
	{section: 'window',     key: 'y',           factory: 0},
	{section: 'window',     key: 'width',       factory: 0},
	{section: 'window',     key: 'height',      factory: 0},
	{section: 'zoom',       key: 'step',        factory: 1.25, comment: 'how much one press of + or - grows or shrinks the image', check: n => n > 1},
	{section: 'fullscreen', key: 'curtain',     factory: true, comment: 'black out the frame through a fullscreen transition, which hides an occasional one-frame shear at the cost of a blink; the user chose true by feel'},
	{section: 'hud',        key: 'information', factory: true, comment: 'show the information panel along the bottom of the frame, the one [i] toggles; fuji writes this back as you turn it on and off, so it comes back the way you left it'},
	{section: 'hud',        key: 'caption',     factory: true, comment: 'show the caption beneath the image at startup'},
]

export const settings = settingsFactory()//the live settings the rest of fuji reads, filled in at startup and never replaced, so an importer keeps the same object

let settingsFilePath = ''//where the file is, found once at startup
let settingsFileText = ''//what fuji last read from or wrote to the file, to tell when a write would change nothing
let settingsHeldText = ''//the text rust's held copy corresponds to, so an unchanged render doesn't cross to it again

function settingsFactory() {//a settings object with every value at its factory setting
	let s = {}
	for (let entry of settingsSchema) (s[entry.section] ??= {})[entry.key] = entry.factory
	return s
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
		if (typeof value != typeof entry.factory) { problems.push(`${name} has to be ${typeof entry.factory}, so ${sayValue(value)} was ignored`); continue }
		if (entry.check && !entry.check(value))   { problems.push(`${name} cannot be ${sayValue(value)}, so it was ignored`);                     continue }
		settings[entry.section][entry.key] = value
	}
	for (let [section, table] of Object.entries(parsed)) {//the file lists every setting fuji has, so a name fuji doesn't know is a typo rather than a default quietly showing through, and worth saying out loud
		if (typeof table != 'object') { problems.push(`${section} is not a fuji setting`); continue }
		for (let key of Object.keys(table)) {
			if (!settingsSchema.some(entry => entry.section == section && entry.key == key)) problems.push(`${section}.${key} is not a fuji setting`)
		}
	}
	return {settings, problems}
}

function settingsRender(settings) {//the complete text of the file for these settings, and the only place that text ever comes from
	let lines = [settingsHeader]
	for (let section of new Set(settingsSchema.map(entry => entry.section))) {
		let entries = settingsSchema.filter(entry => entry.section == section)
		let keyWidth   = Math.max(...entries.map(entry => entry.key.length))//pad within the section, so a long name in one doesn't push the others out
		let valueWidth = Math.max(...entries.map(entry => sayValue(settings[section][entry.key]).length))

		lines.push('', `[${section}]`)
		for (let entry of entries) {
			if (entry.comment) lines.push(`# ${entry.comment}`)//above the setting, not trailing it: these run long, and a soft wrapped comment beside a value would fold across the next line
			lines.push(`${entry.key.padEnd(keyWidth)} = ${sayValue(settings[section][entry.key]).padEnd(valueWidth)} # factory ${sayValue(entry.factory)}`)
		}
	}
	return lines.join('\n')+'\n'
}

function sayValue(value) {//a value as the toml text that means it
	if (typeof value == 'boolean') return value ? 'true' : 'false'
	if (typeof value == 'number')  return String(value)
	return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`//a basic string, escaping the backslash that starts an escape and the quote that would end it early
}

export function settingsWindowRect() {//the window fuji recorded and should return to, or false to size itself to the desktop the way it always has
	let w = settings.window
	if (!w.remember || !(w.width > 0 && w.height > 0)) return false//not remembering, or nothing recorded yet, or something in the file that can't describe a window
	return {x: w.x, y: w.y, width: w.width, height: w.height}
}

export async function settingsLoad() {//read the settings file and leave it exactly as fuji would write it, which is what creates a missing one, repairs a bad value, and adds a setting fuji has gained since the last launch; call once, before anything reads a setting
	settingsFilePath = parse.join(forwardize(await homeDir()), settingsFileName)

	let text = ''
	try {
		text = new TextDecoder().decode(new Uint8Array(await diskRead(settingsFilePath)))
	} catch (error) {
		console.log(`⭕ settings: starting a new file at ${settingsFilePath}, because reading one said: ${error}`)//no file is how fuji starts on a new machine, but a file that exists and can't be read lands here too, so say which
	}
	settingsFileText = text

	let {settings: found, problems} = settingsParse(text)
	for (let entry of settingsSchema) settings[entry.section][entry.key] = found[entry.section][entry.key]//fill the live object rather than replacing it, so importers keep theirs
	for (let problem of problems) console.log(`⭕ settings: ${problem}`)

	let rendered = settingsRender(settings)
	if (rendered != settingsFileText) {//the file is missing, or held a value fuji had to repair, or came from a fuji with fewer settings than this one
		try {
			await diskWrite(settingsFilePath, Array.from(new TextEncoder().encode(rendered)))//disk.rs speaks bytes because it mirrors posix, and this is the one place fuji encodes; everywhere else text stays text
			settingsFileText = rendered//only once the write happened
		} catch (error) {
			console.error('writing settings:', error)
		}
	}
	settingsHeldText = settingsFileText
	settingsChanged()//a no-op after a write that worked; after one that didn't, this is what leaves the file with rust to try again on the way out
}

export function settingsChanged() {//call after changing a value in settings, the way quiver() gets called after moving an arrow; hands the file down to rust, which writes it on the way out — desktop.rs has why only rust can see a quit coming
	let text = settingsRender(settings)
	if (text == settingsHeldText) return//rust's view already matches, which is what a move event reporting the same position produces
	settingsHeldText = text
	desktopExitHold(settingsFilePath, text == settingsFileText ? '' : text)//blank when the settings are back to what is on the disk, so an undone change writes nothing at all
		.catch(error => console.error('handing settings down:', error))
}
