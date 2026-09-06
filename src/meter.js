//./src/meter.js

import {homeDir} from '@tauri-apps/api/path'
import parse from 'path-browserify'
import {diskWrite} from './disk.js'
import {desktopExitHold} from './desktop.js'
import {forwardize} from './components/library.js'
import {settings} from './settings.js'

/*
Fuji's performance log. Every image load and every flip, with what each one cost, written to a file. Off unless meter.record says otherwise, and off is the factory setting: this is for answering a question, not for running the app.

It exists because the HUD could not answer the question honestly. The HUD builds a string and paints text in the same frame as the image it describes, so reading it costs some part of what it reports — and a number you can only see by making the thing slower is not a measurement, it is a rumour. So nothing here renders. Rows go into an array, the array becomes text only after a pause in the action, and the file is written away from the frames it is describing. performance.md carries what this instrument has already found, which is most of what fuji now knows about its own speed.

Two things make a row worth having. The first is the split: a flip is a wait on the store and then a wait on the screen, and those two have completely different causes and completely different fixes. The second is chronology — loads and flips are interleaved in one list on purpose, because a decode finishing in the middle of a flip is exactly the sort of thing that explains a slow one, and two separate tables would hide it.

The file is written twice by two mechanisms, deliberately. Each hand-over writes it outright, so a failure is reported to the console while somebody is around to see it and so a crash still leaves everything up to the last pause. Rust also holds a copy to write at exit, which catches the rows added after that last pause. Both write the same text and the later one wins, so there is nothing to reconcile.
*/

const meterQuiet = 1500//milliseconds of nothing happening before the rows become a file; long enough that a burst of flipping writes once rather than once a flip
const meterCeiling = 5000//rows, after which recording stops rather than growing without end; the beginning of a session is the most diagnostic part of it, so this keeps the start and drops the rest

let meterRecording = false//the one thing every entry point below checks, set from settings at the start of a run
let meterLabel = ''//what this run was, in the file name and the header
let meterStamp = ''//when it began, likewise
let meterFilePath = ''//blank until homeDir answers, which is the only asynchronous part of starting
let meterRows = []
let meterFull = false//the ceiling was reached, which the file says plainly so a short log is never mistaken for a quiet session
let meterQuietTimer = null

export function meterStart(label) {//name this run and find somewhere to put it; call once when a view starts
	meterRecording = settings.meter.record
	meterRows = []; meterFull = false; meterFilePath = ''
	if (!meterRecording) return

	meterLabel = label
	meterStamp = sayStamp(new Date())
	homeDir()
		.then(home => { meterFilePath = parse.join(forwardize(home), settings.meter.folder, `fuji-meter-${meterLabel}-${meterStamp}.txt`); meterLater() })
		.catch(error => { meterRecording = false; console.error('meter, finding somewhere to write:', error) })//a recording nobody can write is worse than none, because you find that out at exit
}

export function meterLoad(entry) {//one completed load, however it turned out; the store knows these times whether anything ever flipped to the image or not
	if (!meterRecording) return
	meterPush({
		what: 'load',
		path: entry.path,
		disk: entry.loaded ? entry.loaded - entry.requested : 0,
		render: entry.rendered ? entry.rendered - entry.loaded : 0,
		bytes: entry.blobBytes,
		natural: entry.img ? `${entry.img.naturalWidth}x${entry.img.naturalHeight}` : '',
		note: entry.error ? String(entry.error) : (entry.img ? '' : 'released while loading'),//an entry with no element was let go before its decode finished, which is ordinary and is not a failure of the file
	})
}

export function meterFlip(row) {//one flip, already measured by the view, which is the only place that can see both halves of it
	if (!meterRecording) return
	meterPush({what: 'flip', ...row})
}

function meterPush(row) {
	if (meterRows.length >= meterCeiling) { if (!meterFull) { meterFull = true; meterLater() } return }//one last write when the ceiling is first reached, so the file says it stopped rather than simply ending
	meterRows.push(row)
	meterLater()
}

function meterLater() {//write after things go quiet, so the file never lands on a frame it would be timing
	clearTimeout(meterQuietTimer)
	meterQuietTimer = setTimeout(meterWrite, meterQuiet)
}

function meterWrite() {
	if (!meterRecording || !meterFilePath || !meterRows.length) return
	let text = meterRender()
	diskWrite(meterFilePath, Array.from(new TextEncoder().encode(text)))//written now, so a folder that does not exist is reported while somebody can still read the console
		.catch(error => console.error(`meter, writing ${meterFilePath}:`, error))
	desktopExitHold(meterFilePath, text)//and held by rust, which catches whatever happens after this pause
		.catch(error => console.error('meter, handing down:', error))
}

function meterRender() {//the whole run as text, rebuilt each time rather than appended, because a few thousand rows is nothing to build and one string is all rust holds
	let lines = [
		`# fuji meter, ${meterLabel}, run began ${meterStamp}`,
		`# flip.back ${settings.flip.back}, flip.forward ${settings.flip.forward}`,
		`# every load and every flip in the order they happened; times in milliseconds`,
		`# store is the wait on the cache, and is zero for an image it already had; paint is the swap reaching the screen`,
		`# performance.md says what these numbers have already shown`,
	]
	if (meterFull) lines.push(`# stopped at ${meterCeiling} rows, so this run continued past the end of this file`)
	lines.push('', sayRow({
		what: 'what', sequence: 'seq', index: 'index', direction: 'dir', hit: 'hit',
		store: 'store', paint: 'paint', flip: 'flip', frames: 'frames',
		disk: 'disk', render: 'render', bytes: 'bytes', natural: 'natural', path: 'path',
	}))
	for (let row of meterRows) lines.push(sayRow(row))
	return lines.join('\n') + '\n'
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
