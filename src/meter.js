import {homeDir} from '@tauri-apps/api/path'
import parse from 'path-browserify'
import {desktopExitAppend} from './desktop.js'
import {forwardize} from './components/library.js'
import {settings} from './settings.js'

/*
Fuji's performance log. Every image load and every flip, with what each one cost, written to a file. Off unless meter.record says otherwise, and off is the factory setting: this is for answering a question, not for running the app.

It exists because the HUD could not answer the question honestly. The HUD builds a string and paints text in the same frame as the image it describes, so reading it costs some part of what it reports — and a number you can only see by making the thing slower is not a measurement, it is a rumour. performance.md carries what this instrument has already found, which is most of what fuji now knows about its own speed.

The same reasoning decides how it reaches the disk, and it is the whole reason for desktop_exit_append. Rows go down to rust a few at a time and rust holds them in memory; nothing touches the disk until fuji exits. Writing during the session would put a file write on the main thread in the middle of the flips being timed, which is the HUD's mistake wearing different clothes. The cost of that choice is that a crash loses the log, which is the right trade for an instrument, since a crash mid-run invalidates the measurement anyway.

Two things make a row worth having. The first is the split: a flip is a wait on the store and then a wait on the screen, and those two have completely different causes and completely different fixes. The second is chronology — loads and flips are interleaved in one list on purpose, because a decode finishing in the middle of a flip is exactly the sort of thing that explains a slow one, and two separate tables would hide it.
*/

//one file per run of fuji, so meterStart is called once, by the shell, and never by a view: a second call would point at a new file while the rows already sent sat in rust under the old path, and rust would write both

const meterQuiet = 1500//milliseconds of nothing happening before pending rows go down to rust; long enough that a burst of flipping sends once rather than once a flip
const meterCeiling = 5000//rows, after which recording stops rather than growing without end; the beginning of a session is the most diagnostic part of it, so this keeps the start and drops the rest

let meterRecording = false//the one thing every entry point below checks, set from settings at the start of a run
let meterLabel = ''//what this run was, in the file name and the header
let meterStamp = ''//when it began, likewise
let meterFilePath = ''//blank until homeDir answers, which is the only asynchronous part of starting
let meterPending = []//rows recorded but not yet handed down; rust holds everything already sent, so this stays small
let meterTotal = 0//rows ever recorded, which is what the ceiling counts
let meterHeaderSent = false
let meterFull = false//the ceiling was reached, which the file says plainly so a short log is never mistaken for a quiet session
let meterQuietTimer = null

export function meterStart(label) {//name this run and find somewhere to put it; the shell calls this once, after settings are read
	meterRecording = settings.meter.record
	meterPending = []; meterTotal = 0; meterHeaderSent = false; meterFull = false; meterFilePath = ''
	if (!meterRecording) return

	meterLabel = label
	meterStamp = sayStamp(new Date())
	homeDir()
		.then(home => { meterFilePath = parse.join(forwardize(home), settings.meter.folder, `fuji-meter-${meterLabel}-${meterStamp}.txt`); meterLater() })
		.catch(error => { meterRecording = false; console.error('meter, finding somewhere to write:', error) })//a recording nobody can write is worse than none
}

export function meterLoad(entry, note) {//one completed load, however it turned out; the store knows these times whether anything ever flipped to the image or not, and says in the note which kind of row this is: blank for a read and decode together, bytes only for a read nobody asked to decode, decoded later for a decode that followed such a read (its disk column repeats the earlier row, because it is the same read), or released while loading
	if (!meterRecording) return
	if (entry.error && note != 'released while loading') note = String(entry.error)//a file that would not read or decode; a release mid-decode aborts the decode, which looks like an error and is not one
	meterPush({
		what: 'load',
		path: entry.path,
		disk: entry.loaded ? entry.loaded - entry.requested : 0,
		render: entry.rendered ? entry.rendered - entry.loaded : 0,
		bytes: entry.blobBytes,
		natural: entry.img ? `${entry.img.naturalWidth}x${entry.img.naturalHeight}` : '',
		note,
	})
}

export function meterFlip(row) {//one flip, already measured by the view, which is the only place that can see both halves of it
	if (!meterRecording) return
	meterPush({what: 'flip', ...row})
}

function meterPush(row) {
	if (meterTotal >= meterCeiling) {
		if (!meterFull) { meterFull = true; meterPending.push({what: `# stopped at ${meterCeiling} rows, so this run continued past the end of this file`}); meterLater() }
		return
	}
	meterPending.push(row); meterTotal++
	meterLater()
}

function meterLater() {//hand rows down after things go quiet, so even this small crossing never lands beside a flip
	clearTimeout(meterQuietTimer)
	meterQuietTimer = setTimeout(meterSend, meterQuiet)
}

function meterSend() {
	if (!meterRecording || !meterFilePath || !meterPending.length) return
	let lines = []
	if (!meterHeaderSent) { lines.push(meterHeader()); meterHeaderSent = true }
	for (let row of meterPending) lines.push(row.what.startsWith('#') ? row.what : sayRow(row))
	meterPending = []//rust has them now, so nothing here holds the log

	desktopExitAppend(meterFilePath, lines.join('\n') + '\n')//rust concatenates and writes the whole thing when fuji exits
		.catch(error => console.error('meter, handing down:', error))
}

function meterHeader() {//sent once, ahead of the first rows
	return [
		`# fuji meter, ${meterLabel}, run began ${meterStamp}`,
		`# flip.back ${settings.flip.back}, flip.forward ${settings.flip.forward}`,
		`# every load and every flip in the order they happened; times in milliseconds`,
		`# store is the wait on the cache, and is zero for an image it already had; paint is the swap reaching the screen`,
		`# performance.md says what these numbers have already shown`,
		'',
		sayRow({
			what: 'what', sequence: 'seq', index: 'index', direction: 'dir', hit: 'hit',
			store: 'store', paint: 'paint', flip: 'flip', frames: 'frames',
			disk: 'disk', render: 'render', bytes: 'bytes', natural: 'natural', path: 'path',
		}),
	].join('\n')
}

function sayRow(r) {//one line of aligned columns, because reading a hundred flips means reading down a column rather than along a row
	return [
		pad(r.what, 5),
		pad(r.sequence, 5), pad(r.index, 6), pad(r.direction, 4), pad(r.hit, 5),
		pad(r.store, 7), pad(r.paint, 7), pad(r.flip, 7), pad(r.frames, 7),
		pad(r.disk, 7), pad(r.render, 7),
		pad(r.bytes, 11), pad(r.natural, 12),
		r.path, r.note ? ' ' + r.note : '',
	].join(' ').trimEnd()
}

function pad(value, width) {
	let s = (value == undefined || value === '') ? '-' : String(value)
	return s.length >= width ? s : s + ' '.repeat(width - s.length)
}

function sayStamp(d) {//a sortable local timestamp down to the second, because two runs a minute apart must not land on one file name
	let two = n => String(n).padStart(2, '0')
	return `${d.getFullYear()}-${two(d.getMonth() + 1)}-${two(d.getDate())}-${two(d.getHours())}${two(d.getMinutes())}${two(d.getSeconds())}`
}
