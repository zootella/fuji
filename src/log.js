import {homeDir} from '@tauri-apps/api/path'
import {invoke} from '@tauri-apps/api/core'
import parse from 'path-browserify'
import {forwardize} from './components/library.js'
import {settings} from './settings.js'

/*
Fuji's log: every line the page or Rust chose to keep, in one text file per run, written when fuji exits. Off unless log.record in fuji.toml says otherwise, and off at the factory: this is for answering a question about a run, not for running the app. Turn it on, use fuji, quit, and the file is in fuji-temp under the home folder, named for the moment the run began.

It exists because console.log cannot get over any of five fences, and a file gets over all of them.

The first fence is the platform. A console is a different tool on the Mac and on Windows, reached through a different browser's inspector, and a note left in one is not readable from the other; a file is the same file on both. The second is who is looking. The user cannot see a console without opening an inspector, and a Claude session cannot see one at all; a file can be opened by either and handed from one to the other, which is the whole reason to keep a note. The third is development against production. A console exists only in a development build with the inspector open; a production build of fuji has no console, and a line written to it goes nowhere; a file is written by the same code in both. The fourth is where fuji is installed, in an application folder, a portable folder, or a checkout being run with pnpm local; the file lands under the user's home folder whichever it is. The fifth, and the one that made this a redesign rather than a rename, is the language. console.log belongs to the page, and Rust has only stderr, which goes somewhere else again; a line that matters can be born on either side, and this log takes both into one file. So the rule is: a line meant to be read later goes here, and console is for a programmer error at a top gate, during development, and nothing else.

Two functions called log, one here and one in log.rs, each taking a string the way console.log does. This one gathers lines and hands them down after things go quiet, so even that small crossing never lands beside a flip being timed; performance.md has why nothing else may touch the disk during a session. Rust's appends in place. Lines from the two sides keep no exact order against each other, and console.log never promised one either. The cost of writing at exit is that a crash loses the log, which is the right trade for an instrument, since a crash mid-run invalidates the measurement anyway.

The typed rows — a load, a flip, a thumbnail, a card — are helpers over log() that write aligned columns, because reading a hundred flips means reading down a column rather than along a row. Loads and flips are interleaved on purpose, since a decode landing in the middle of a flip is what explains a slow one. performance.md says what the rows mean and what they have already shown.
*/

const logFolder = 'fuji-temp'//under the user's home folder, on every platform; rust makes it on the way out if it is missing
const logQuiet = 1500//milliseconds of nothing happening before pending lines go down to rust; long enough that a burst of flipping sends once rather than once a flip
const logCeiling = 5000//lines, after which recording stops rather than growing without end; the beginning of a session is the most diagnostic part of it, so this keeps the start and drops the rest

let logRecording = false//the one thing every entry point below checks, set from settings at the start of a run
let logStarted = false//rust has the path, so lines can go down
let logPending = []//lines recorded but not yet handed down; rust holds everything already sent, so this stays small
let logTotal = 0//lines ever recorded, which is what the ceiling counts
let logFull = false//the ceiling was reached, which the file says plainly so a short log is never mistaken for a quiet session
let logQuietTimer = null

export function logStart(label) {//name this run and hand rust the file; the shell calls this once, after settings are read
	logRecording = settings.log.record
	logPending = []; logTotal = 0; logFull = false; logStarted = false
	if (!logRecording) return

	let stamp = sayStamp(new Date())
	homeDir()
		.then(home => invoke('log_start', {path: parse.join(forwardize(home), logFolder, `fuji-log-${stamp}.txt`)}))
		.then(() => { logStarted = true; logHeader(label, stamp); logLater() })//the header first, then whatever lines gathered while the path was crossing
		.catch(error => { logRecording = false; console.error('log, starting:', error) })//a recording nobody can write is worse than none
}

export function log(text) {//one line, handed to rust when things go quiet
	if (!logRecording) return
	if (logTotal >= logCeiling) {
		if (!logFull) { logFull = true; logPending.push(`# stopped at ${logCeiling} lines, so this run continued past the end of this file`); logLater() }
		return
	}
	logPending.push(text); logTotal++
	logLater()
}

export function logLoad(entry, note) {//one completed load, however it turned out. The store says in the note which kind: blank for a read and decode together, bytes only for a read nobody asked to decode, decoded later for a decode that followed such a read (its disk column repeats that row's, being the same read), or released while loading
	if (!logRecording) return
	if (entry.error && note != 'released while loading') note = String(entry.error)//a file that would not read or decode; a release mid-decode aborts the decode, which looks like an error and is not one
	log(sayRow({
		what: 'load',
		path: entry.path,
		disk: entry.loaded ? Math.round(entry.loaded - entry.requested) : 0,//whole milliseconds: performance.now is fractional, and the fraction is noise in a column meant to be read down
		render: entry.rendered ? Math.round(entry.rendered - entry.loaded) : 0,
		bytes: entry.blobBytes,
		natural: entry.img ? `${entry.img.naturalWidth}x${entry.img.naturalHeight}` : '',
		note,
	}))
}
export function logFlip(row)      { log(sayRow({what: 'flip',  ...row})) }//one flip, already measured by the view, which is the only place that can see both halves of it
export function logThumbnail(row) { log(sayRow({what: 'thumb', ...row})) }//one thumbnail the sheet made or refused: hit says by which path, render the milliseconds, bytes its canvas, natural its pixels, and the note why it was refused
export function logCard(row)      { log(sayRow({what: 'card',  ...row})) }//one card filled: index is how many images it holds, render the milliseconds to fill it, bytes what its canvases cost, and the note the count by path

function logLater() {//hand lines down after things go quiet, so even this small crossing never lands beside a flip
	clearTimeout(logQuietTimer)
	logQuietTimer = setTimeout(logSend, logQuiet)
}
function logSend() {
	if (!logRecording || !logStarted || !logPending.length) return
	let text = logPending.join('\n') + '\n'
	logPending = []//rust has them now, so nothing here holds the log
	invoke('log_append', {text}).catch(error => console.error('log, handing down:', error))
}

function logHeader(label, stamp) {//once, ahead of the first lines
	logPending.unshift(
		`# fuji log, ${label}, run began ${stamp} utc`,
		`# flip.back ${settings.flip.back}, flip.forward ${settings.flip.forward}`,
		`# every line the page and rust chose to keep, roughly in the order they happened; times in milliseconds`,
		`# a load row: disk is the read, render the decode. A flip row: store is the wait on the cache, zero for an image it already had, and paint is the swap reaching the screen`,
		`# a thumb row is one thumbnail the sheet made: hit is its path, native, page or img, or refused with the reason in the note; render is the milliseconds to make it; bytes its canvas; natural its pixels`,
		`# a card row is one card filled: index is how many images, render the milliseconds, bytes the canvases, and the note the count by path`,
		`# performance.md says what these numbers have already shown`,
		'',
		sayRow({
			what: 'what', sequence: 'seq', index: 'index', direction: 'dir', hit: 'hit',
			store: 'store', paint: 'paint', flip: 'flip', frames: 'frames',
			disk: 'disk', render: 'render', bytes: 'bytes', natural: 'natural', path: 'path',
		}),
	)
}

function sayRow(r) {//one line of aligned columns, because reading a hundred flips means reading down a column rather than along a row
	return [
		pad(r.what, 5),
		pad(r.sequence, 5), pad(r.index, 6), pad(r.direction, 4), pad(r.hit, 8),
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

function sayStamp(d) {//utc to the millisecond, sortable, like 2026-09-08T18-29-27-123Z: the colons and the dot of the standard form become dashes, because windows forbids a colon in a file name
	return d.toISOString().replace(/[:.]/g, '-')
}
