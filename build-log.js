import {spawnSync} from 'node:child_process'
import {appendFileSync, readFileSync} from 'node:fs'
import {join} from 'node:path'
import {fileURLToPath} from 'node:url'
import {Data, Time, sayDuration, sayTick, sha256, tickToText} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in

/*
The build log, and the way the pipeline says a time, for every workspace's build and publish on every machine. scripts.js at the root runs the commands, and this file beside it is what the scripts that build share with it: dmg.js, win-setup.js and notarize.js in desktop, and build.js in linux. It runs nothing when imported, having nothing to run, and like scripts.js it builds every path from its own location.
*/

const root = fileURLToPath(new URL('.', import.meta.url))

/*
The build log, build-log.txt at the root: a line for every step any workspace takes on this machine, as it starts and as it ends, so what was built, notarized, hashed and uploaded here can be read back long after the scripts that did it have changed. It is gitignored, so each machine keeps its own.

A line is seven fields and then words. The fields are when, in utc to the millisecond, in ergo's short form of a tick, like 2026oct22.2347.32148, which a person reads and a program reads back exactly; the process that wrote it; the commit the checkout was at, its first eight hex digits followed by + when tracked files had changed since, which is how a hash in the log is traced back to the source it came from; the step, like dmg, notarize or hash; the mark, started, finished or failed; the subject, a tag or a file's name; and the value, a sha256 or a code hash. None of them holds a space, a field with nothing in it is -, and the columns are padded to read as a table, so splitting a line on runs of spaces gives the seven fields and then the words, which are for people. Expect the + on lines between a hash and its commit, since hash changes the sidecar git tracks.

A step starts and ends in one process, so one left started by a process that has gone was cut short, by ctrl-c or a crash.

Every line is also said in the terminal of the process that wrote it, with the time as a person reads it and, at a step's end, how long it took.
*/
const buildLogFile = join(root, 'build-log.txt')
let begun = new Map()//when each step this process has under way started, so its end can say how long it took

//write one line to the build log and say it in the terminal
export function buildLog(step, mark, subject = '-', value = '-', words = '') {
	let now = Date.now()//a tick, which the line's first field writes in ergo's short form
	let also = []
	if (mark == 'started') {
		begun.set(step, now)
		if (process.env.npm_lifecycle_event) also.push('by pnpm ' + process.env.npm_lifecycle_event)//the script pnpm ran, which is the command a person typed
	} else if (begun.has(step)) {
		also.push('took ' + sayDuration(now - begun.get(step), durationTenths))
		begun.delete(step)
	}
	words = [words.replace(/\s+/g, ' ').trim(), ...also].filter(Boolean).join(', ')//one line, whatever an error message held
	let fields = [sayWhen(now), String(process.pid).padEnd(7), readCommit().padEnd(9), step.padEnd(8), mark.padEnd(8), subject.padEnd(24), value.padEnd(64)]
	appendFileSync(buildLogFile, `${fields.join('  ')}  ${words}`.trimEnd() + '\n')
	let said = [`${step} ${mark}`, ...[subject, value].filter(field => field != '-'), words].filter(Boolean).join('  ')
	console.log(`${sayTime(now, {precise: true})}  ${said}`)//the terminal's in local time, for the person watching it
}

//the commit this checkout is at, its first eight hex digits, followed by + when tracked files have changed since, or - without git. read for every line rather than once, since a commit can land while a long build runs
function readCommit() {
	let head = spawnSync('git', ['rev-parse', 'HEAD'], {cwd: root, encoding: 'utf8'})
	if (head.status != 0) return '-'
	let changed = spawnSync('git', ['--no-optional-locks', 'diff', '--quiet', 'HEAD'], {cwd: root}).status != 0//without the index lock git would otherwise take to refresh, so a line written mid-commit never makes the commit fail
	return head.stdout.trim().slice(0, 8) + (changed ? '+' : '')
}

//one step of this machine's around work that throws, the way this file and the build scripts beside it work: started, then finished with the value and words work returns, or failed with what was thrown, which goes on up. work may answer with a promise, as the linux driver's does since its containers run together, and then the step ends when the promise does
export function buildStep(step, subject, words, work) {
	buildLog(step, 'started', subject, '-', words)
	let finished = result => { buildLog(step, 'finished', subject, result.value, result.words); return result }
	let failed = e => { buildLog(step, 'failed', subject, '-', e.message); throw e }
	try {
		let result = work()
		if (result && typeof result.then == 'function') return result.then(finished, failed)
		return finished(result)
	} catch (e) {
		return failed(e)
	}
}

export async function hashSha256(bytes) { return (await sha256(Data({array: bytes}))).base16() }//the sha-256 of bytes in base16, from ergo over web crypto, which is native in node and answers with a promise; all at once, since a package is tens of megabytes at most
export async function readSha256(path) { return hashSha256(readFileSync(path)) }

/*
Times as people read them, the same in every workspace and whatever the machine's language: a moment like 2026-Oct-9 Fri 2:45 PM, or to the millisecond like 2026-Oct-9 Fri 02:45:07.123 PM for a line in a terminal, and a duration like 1 hour, 2 minutes, 1.5 seconds. Each is a template of ergo's, written by its sayTick for a moment and its sayDuration for a span, so the shapes are the templates and nothing here builds text. A moment is in this machine's own time zone, since a person watching reads it, and sayTick writes the fields of a tick as they fall in utc, so a moment is shifted by the zone's offset first. A time that's kept is utc instead, in one of ergo's forms that read back, like the build log's first field.
*/
const durationTenths = '# days, # hours, # minutes, #.# seconds'//for ergo's sayDuration: every unit from the largest one reached, with the seconds to a tenth, rounded down, for a step that has ended

function sayTime(time, {precise = false} = {}) {//like 2026-Oct-9 Fri 2:45 PM, or precise, 2026-Oct-9 Fri 02:45:07.123 PM, padded so a log's lines line up. time is milliseconds, a Date, or iso text
	return sayTick(_local(time), precise ? 'YYYY-Mmm-D Ddd hh12:mm:ss.ttt AA' : 'YYYY-Mmm-D Ddd h12:mm AA')
}

//the moment that starts a line in the build log: ergo's short form of the tick, in utc, padded to the longest it can be so the log's columns line up
function sayWhen(tick) { return tickToText(tick).padEnd(20) }

function _local(time) {//the tick whose utc fields are this machine's clock at that moment, for sayTick to write; the zone's offset then, daylight saving included, is the one thing only javascript's Date knows
	let tick = new Date(time).getTime()
	return tick - new Date(tick).getTimezoneOffset() * Time.minute
}
