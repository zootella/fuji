import {execFile, spawn, spawnSync} from 'node:child_process'
import {appendFileSync, closeSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, openSync, readdirSync, readFileSync, realpathSync, renameSync, rmSync, statSync} from 'node:fs'
import {tmpdir} from 'node:os'
import {basename, join} from 'node:path'
import {setTimeout as sleep} from 'node:timers/promises'
import {fileURLToPath} from 'node:url'
import {promisify} from 'node:util'
import {dmgFile, dmgMake} from '../desktop/dmg.js'
import {buildLog, readSha256, sayDate, sayDuration, sayTime} from '../build-log.js'

/*
The mac release through Apple, one application at a time: build, notarize, hash and upload. The waiting room draws this, pnpm notarize in desktop runs build and notarize straight through, and README.md beside this file is the guide.

An application is a folder in applications/, made the moment a build is signed and named for the day, the version, and a tag, the first eight hex digits of the app's code hash, which is what Apple's ticket is bound to. Everything after the build happens to the copy in that folder, so pnpm installer can sign the bundle with Fuji Desktop again at any time without touching a release on its way through Apple. Once notarized, the dmg goes back to tauri's bundle folder, where pnpm installer leaves its own, so pnpm hash and pnpm upload publish whichever of the two was made last.

Where an application stands is read every time from things that outlast this program: its folder, Apple's history, and the dmg sidecars staged in desktop/release and live on the site. Every submission is named for its application and ends with its tag, so Apple's history says which of an application's two submissions exist and what Apple made of each. So notarize can run again after any interruption: it finds the submissions already made, waits on them or carries on from them, and never submits the same thing twice.

Each step is a line in the build log at the root as it starts and as it ends, which build-log.js describes: the build, the dmg, sending each part and its time at Apple, the checks, and the dmg delivered to the bundle folder. pnpm notarize prints those lines as it goes, the waiting room reads them to see work in any terminal, and log.txt in the folder has them too, with what every program printed in between.
*/

//every path is built from this file's own location, for the reason scripts.js gives at the root
const root          = fileURLToPath(new URL('..', import.meta.url))
const desktop       = join(root, 'desktop')
const bundled       = join(desktop, 'src-tauri/target/release/bundle')//where tauri leaves the app it built, and where hash in scripts.js looks for a dmg
const applications  = join(root, 'apple/applications')//one folder per application, gitignored
const configuration = JSON.parse(readFileSync(join(desktop, 'src-tauri/tauri.conf.json'), 'utf8'))
const brandName     = configuration.productName//the name tauri gives the .app, and the first word of every submission's name
const brandStem     = readFileSync(join(desktop, 'src-tauri/Cargo.toml'), 'utf8').match(/^name\s*=\s*"([^"]+)"/m)[1]//the stem of the published names, fuji.dmg and fuji.dmg.json, read the way scripts.js reads it
const stagedSidecar = join(root, 'desktop/release', `${brandStem}.dmg.json`)//what hash staged
const liveSidecar   = new URL(`${brandStem}.dmg.json`, configuration.bundle.homepage)//what the site serves now

const identity    = 'Developer ID Application'//the kind of certificate rather than its holder's name, which the README explains
const credentials = ['APPLE_API_KEY', 'APPLE_API_ISSUER', 'APPLE_API_KEY_PATH']//the api key's three values in .env, under the names tauri reads
const checkEvery  = 30*1000//how often a wait asks apple again
const folderName  = /^\d{4}-[A-Z][a-z]{2}-\d{1,2} [A-Z][a-z]{2} (\S+) ([0-9a-f]{8})$/ //an application's folder: the day it was built, its version and its tag
const quit        = {success: false, outcome: 'Quit'}//what notarize comes to after q, which starts nothing new and leaves the rest to the next n
const hashes      = new Map()//each notarized dmg's sha256 by its path, hashed once rather than every time the waiting room looks, since a notarized dmg never changes again

// Applications

//every application, newest first
export function listApplications() {
	if (!existsSync(applications)) return []
	let found = readdirSync(applications).filter(folder => folderName.test(folder)).map(readApplication)
	return found.sort((a, b) => b.made - a.made)//by when b made each folder
}

//one application, as its folder has it
export function readApplication(folder) {
	let [, version, tag] = folder.match(folderName)
	let path = join(applications, folder)
	let dmg = join(path, dmgFile(version))
	let log = join(path, 'log.txt')
	let verify = join(path, 'verify.txt')
	let notarized = existsSync(verify)//written only once the dmg carries its ticket and gatekeeper accepts both
	let made = statSync(path).birthtimeMs//when b finished, by making the folder
	return {
		folder, version, tag, path, dmg, log, notarized, made,
		app:      join(path, `${brandName}.app`),
		begun:    existsSync(log) ? statSync(log).birthtimeMs : made,//when b started, since the log was opened then and kept through the rename
		sent:     notarized ? readSent(path) : 0,//when apple received the app
		finished: notarized ? statSync(verify).mtimeMs : 0,//when the last check passed
		sha256:   notarized ? hashOnce(dmg) : '',//the hash hash will publish
	}
}

function hashOnce(dmg) {
	if (!hashes.has(dmg)) hashes.set(dmg, readSha256(dmg))
	return hashes.get(dmg)
}

function readSent(path) {//when apple received the app, from apple's own log of it, or 0 without one
	let notary = join(path, 'app.notary.json')
	return existsSync(notary) ? Date.parse(JSON.parse(readFileSync(notary, 'utf8')).uploadDate) || 0 : 0
}

//where an application stands, as a status the waiting room shows, since, when its clock started, or 0 for no clock, and until, when a finished step's clock stopped. the folder answers once the dmg is notarized, and before that apple's history does
export function standing(application, {apple, staged, live}) {
	if (application.notarized) {
		if (application.sha256 == live)   return {status: 'Uploaded',  since: 0}
		if (application.sha256 == staged) return {status: 'Hashed',    since: 0}
		return                                   {status: 'Notarized', since: application.sent, until: application.finished}
	}
	if (!apple || !apple.success) return {status: 'Unknown', since: 0}//until apple answers

	let {app, dmg} = findSubmissions(application, apple.history)
	let last = dmg || app//the dmg is only ever sent after the app, so whichever exists last is how far it got
	let part = dmg ? '.dmg' : '.app'
	if (!last) return {status: 'Built', since: application.begun, until: application.made}
	if (last.status == 'In Progress') return {status: `Notarizing ${part}...`, since: Date.parse(last.createdDate)}//apple's clock, so a quit and a return show the same time
	if (last.status == 'Accepted') return {status: `Notarized ${part}`, since: 0}//apple said yes, and n carries on from here
	return {status: 'Rejected', since: 0}//invalid or rejected, and a new build is the way on
}

//an application's two submissions in apple's history, each the newest whose name ends with its tag, or false where there is none. the tag rather than the whole name, since the tag is the code apple's ticket is bound to and the rest of the name is for people. history comes newest first
export function findSubmissions(application, history) {
	let find = extension => history.find(submission => submission.name.endsWith(` ${application.tag}${extension}`)) || false
	return {app: find('.zip'), dmg: find('.dmg')}
}

// Build

//b: build fuji signed with the developer id, and keep the signed app as a new application, or hand back the one already kept for the same code
export async function build() {
	let found = findIdentities()
	if (found.length != 1) return {success: false, outcome: `expected one valid "${identity}" identity in the keychain, found ${found.length}, and the README says what to do with two`}
	let version = readVersion()
	mkdirSync(applications, {recursive: true})
	let building = join(applications, 'building.txt')//the build's output, until there is a folder to keep it in
	rmSync(building, {force: true})
	return step(building, 'build', '-', `${version}, signed with the Developer ID`, () => buildSigned(building, version))
}

async function buildSigned(building, version) {
	//none of tauri's APPLE_ variables but the identity: no api key or apple id, so tauri signs, says it skipped notarizing, and stops, leaving that to n, and no certificate to import in place of the one in the keychain
	let env = Object.fromEntries(Object.entries(process.env).filter(([name]) => !name.startsWith('APPLE_')))
	env.APPLE_SIGNING_IDENTITY = identity//which tauri prefers to the Fuji Desktop in tauri.conf.json
	let built = await run('pnpm', ['tauri', 'build'], {log: building, env, cwd: desktop})
	if (!built.success) return {success: false, outcome: `${built.outcome}, and apple/applications/building.txt says why`}

	//name the application for the code just signed, then keep a copy the next tauri build cannot touch
	let app = join(bundled, `macos/${brandName}.app`)
	let codeHash = readCodeHash(app)
	if (!codeHash) return {success: false, outcome: 'codesign shows no code hash for ' + app}
	let tag = codeHash.slice(0, 8)
	let same = listApplications().find(application => application.tag == tag)//built unchanged signs to the same code hash, on any day, and the application already kept is the one to carry on with
	if (same) return {success: true, outcome: `nothing changed: this build is the same code as ${same.folder}`, application: same, subject: tag, value: codeHash}
	let folder = `${sayDate(new Date())} ${version} ${tag}`
	let path = join(applications, folder)
	mkdirSync(path)
	let log = join(path, 'log.txt')
	renameSync(building, log)
	let copied = await run('ditto', [app, join(path, `${brandName}.app`)], {log})//apple's own copy for a bundle, which keeps the signature whole
	if (!copied.success) return {...copied, subject: tag, log}
	return {success: true, outcome: `Built ${folder}`, application: readApplication(folder), subject: tag, value: codeHash, log}
}

function findIdentities() {//the names of the valid code signing identities containing the words in identity, which are the ones codesign would choose among
	let listed = spawnSync('security', ['find-identity', '-v', '-p', 'codesigning'], {encoding: 'utf8'}).stdout
	return [...listed.matchAll(/\d+\) [0-9A-F]{40} "(.*)"/g)].map(match => match[1]).filter(name => name.includes(identity))
}

function readCodeHash(app) {//the app's code directory hash, which stapler looks a ticket up by and whose first eight hex digits are its tag, or false
	let shown = spawnSync('codesign', ['--display', '--verbose=3', app], {encoding: 'utf8'})
	let found = shown.stderr.match(/^CDHash=([0-9a-f]{40})/m)
	return found ? found[1] : false
}

// Notarize

//n: take an application through apple from wherever it stands, finding first what was already sent, and leave its dmg in the bundle folder
export async function notarize(application, {signal}) {
	if (application.notarized) {//nothing to send, and the dmg goes to the bundle folder all the same, which is how pnpm notarize after pnpm installer puts the notarized one back
		await deliver(application)
		return {success: true, outcome: `${application.folder} is already notarized, and its dmg is in the bundle folder`}
	}
	let apple = await readHistory(signal)
	if (signal.aborted) return quit
	if (!apple.success) return apple
	let {app, dmg} = findSubmissions(application, apple.history)
	let dmgId = dmg ? dmg.id : ''

	if (!dmgId) {
		//the app, sent once and then waited on
		let appId = app ? app.id : ''
		if (!appId) {
			let sent = await submit(application, 'app')
			if (!sent.success) return sent
			appId = sent.id
		}
		let accepted = await waitOnApple(appId, {application, part: 'app', signal})
		if (!accepted.success) return accepted
		if (signal.aborted) return quit//apple's answer came in with the quit, and is kept

		//make the dmg and send it, without stopping between. a quit waits for both, so a dmg is either at apple or not yet made
		let made = await step(application.log, 'dmg', basename(application.dmg), `of the app of ${application.folder}, stapled`, () => makeDmg(application))
		if (!made.success) return made
		let sent = await submit(application, 'dmg')
		if (!sent.success) return sent
		dmgId = sent.id
	}
	let accepted = await waitOnApple(dmgId, {application, part: 'dmg', signal})
	if (!accepted.success) return accepted
	if (signal.aborted) return quit
	let checked = await step(application.log, 'check', application.tag, 'stapling the dmg, then asking Gatekeeper', () => check(application))
	if (!checked.success) return checked
	await deliver(application)
	return {success: true, outcome: `Notarized ${application.folder}, and its dmg is in the bundle folder`}
}

//staple the app's ticket to it, so it carries its own inside the dmg, then wrap it in a dmg and sign that
async function makeDmg(application) {
	let log = application.log
	let stapled = await run('xcrun', ['stapler', 'staple', application.app], {log})
	if (!stapled.success) return stapled
	rmSync(application.dmg, {force: true})//one left by an attempt that stopped before apple had it, safe to make again because apple never saw it
	note(log, `$ dmgMake ${application.dmg}`)
	let output = openSync(log, 'a')
	let made = await dmgMake({app: application.app, file: application.dmg, output})
	closeSync(output)
	if (!made.success) return made
	let signed = await run('codesign', ['--sign', identity, '--timestamp', application.dmg], {log})
	if (!signed.success) return signed
	return {success: true, outcome: 'signed with the Developer ID', value: readSha256(application.dmg)}
}

//send one part to apple under the application's name, from a private temporary folder: this mac's send step, the zip and the upload, and then apple's step, which starts once apple has it
async function submit(application, part) {
	let subject = `${application.tag}.${part}`
	let name = `${brandName} ${application.folder}.${part == 'app' ? 'zip' : 'dmg'}`//apple files a submission under the name of the file sent
	let sent = await step(application.log, 'send', subject, `as ${name}`, async () => {
		let scratch = mkdtempSync(join(tmpdir(), 'notarize-'))
		let file = join(scratch, name)
		let packed = {success: true}
		if (part == 'app') packed = await run('ditto', ['-c', '-k', '--keepParent', application.app, file], {log: application.log})//the zip apple's documentation makes, with the bundle whole inside
		else copyFileSync(application.dmg, file)//the same bytes under the submission's name, leaving the one in the folder named the way hash expects
		let asked = packed.success ? await askApple(['submit', file]) : packed
		rmSync(scratch, {recursive: true, force: true})
		if (!asked.success) return asked
		return {success: true, outcome: "Apple's id for it", value: asked.answer.id}
	})
	if (!sent.success) return sent
	record(application.log, 'apple', 'started', subject, sent.value, 'with Apple')
	return {success: true, id: sent.value}
}

//wait for apple's answer on one submission, asking every half minute, and keep apple's log of it. a failed ask is asked again, since a wait can outlast a network outage, and a quit ends the wait at once with nothing lost, since apple keeps going: apple's step stays started until its answer is recorded, by this wait or the next
async function waitOnApple(id, {application, part, signal}) {
	while (!signal.aborted) {
		let asked = await askApple(['info', id], signal)
		if (asked.success && asked.answer.status != 'In Progress') {
			let answer = asked.answer.status
			await askApple(['log', id, join(application.path, `${part}.notary.json`)])//apple's own account of what it examined, warnings included
			let accepted = answer == 'Accepted'
			record(application.log, 'apple', accepted ? 'finished' : 'failed', `${application.tag}.${part}`, answer.replace(/\s+/g, '-'), accepted ? "Apple's answer" : `Apple's answer, and ${part}.notary.json says why`)
			if (!accepted) return {success: false, outcome: `Apple answered ${answer} for the .${part}, and ${part}.notary.json in its folder says why`}
			return {success: true}
		}
		await sleep(checkEvery, null, {signal}).catch(() => {})//a quit ends the sleep early, and the loop's test then ends the wait
	}
	return quit
}

//staple the dmg's ticket to it and ask gatekeeper what a downloaded copy will meet, keeping every verdict. verify.txt existing is what says an application is notarized, so the verdicts are written under another name and take that one only if every check passes
async function check(application) {
	let stapled = await run('xcrun', ['stapler', 'staple', application.dmg], {log: application.log})
	if (!stapled.success) return stapled

	let verifying = join(application.path, 'verifying.txt')
	rmSync(verifying, {force: true})//an earlier attempt's verdicts, replaced by this one's
	let checks = [
		['xcrun', ['stapler', 'validate', application.app]],
		['xcrun', ['stapler', 'validate', application.dmg]],
		['spctl', ['--assess', '-vv', '--type', 'execute', application.app]],//accepted, with source=Notarized Developer ID
		['spctl', ['--assess', '-vv', '--type', 'open', '--context', 'context:primary-signature', application.dmg]],
	]
	let passed = true
	for (let [command, args] of checks) if (!(await run(command, args, {log: verifying})).success) passed = false
	if (!passed) return {success: false, outcome: 'a check failed after stapling, and verifying.txt in its folder has the verdicts'}

	renameSync(verifying, join(application.path, 'verify.txt'))
	return {success: true, outcome: 'the app and the dmg each carry a ticket, and Gatekeeper accepts both', value: readSha256(application.dmg)}
}

//leave the application's dmg in tauri's bundle folder, where pnpm installer leaves its own and hash looks, emptied first as dmg.js empties it, so hash finds exactly this one
function deliver(application) {
	return step(application.log, 'deliver', basename(application.dmg), `the notarized dmg of ${application.folder}`, async () => {
		let folder = join(bundled, 'dmg')
		rmSync(folder, {recursive: true, force: true})
		mkdirSync(folder, {recursive: true})
		copyFileSync(application.dmg, join(folder, basename(application.dmg)))
		return {success: true, outcome: 'in the bundle folder, where pnpm hash finds it', value: application.sha256 || readSha256(application.dmg)}
	})
}

// Hash and upload

//h: put the application's dmg in the bundle folder again, since another may have landed there after n, and run the same hash as pnpm hash in desktop, so staging from either place comes out the same
export async function hash(application) {
	if (!application.notarized) return {success: false, outcome: 'nothing to hash until n has notarized it'}
	let version = readVersion()
	if (application.version != version) return {success: false, outcome: `this is ${application.version} and tauri.conf.json says ${version}, which is the version hash publishes`}
	await deliver(application)
	let hashed = await run(process.execPath, [join(root, 'scripts.js'), 'hash', 'dmg'], {log: application.log})
	if (!hashed.success) return hashed
	return {success: true, outcome: `Hashed ${application.folder}`}
}

//u: send what hash staged, the same upload as pnpm upload in desktop, once what is staged is this application's dmg
export async function upload(application) {
	if (!application.notarized || readStaged() != application.sha256) return {success: false, outcome: 'h first, so the dmg staged in desktop/release is this one'}
	let sent = await run(process.execPath, [join(root, 'scripts.js'), 'upload-installer', 'dmg'], {log: application.log})//the deploy values came in from .env with the waiting room's own
	if (!sent.success) return sent
	return {success: true, outcome: `Uploaded ${application.folder}`}
}

//the sha256 of the dmg hash staged, or blank
export function readStaged() {
	if (!existsSync(stagedSidecar)) return ''
	return JSON.parse(readFileSync(stagedSidecar, 'utf8')).sha256 || ''
}

//the sha256 of the dmg the site serves now, or blank when the site does not say. a bottom gate, so a site that is down is an answer rather than an exception
export async function readLive(signal) {
	try {
		let response = await fetch(liveSidecar, {signal})
		if (!response.ok) return ''
		return (await response.json()).sha256 || ''
	} catch (e) {
		return ''
	}
}

// Apple, and the programs underneath

//apple's list of the team's submissions, newest first
export async function readHistory(signal) {
	let asked = await askApple(['history'], signal)
	if (!asked.success) return asked
	return {success: true, history: asked.answer.history}
}

//one question to apple's notary service, answered in json. a bottom gate: notarytool speaks in exit codes and execFile in exceptions, so both become a result
async function askApple(args, signal) {
	let missing = credentials.filter(name => !process.env[name])
	if (missing.length > 0) return {success: false, outcome: `.env at the monorepo root is missing ${missing.join(', ')}`}
	try {
		let {stdout} = await promisify(execFile)('xcrun', ['notarytool', ...args, '--output-format', 'json',
			'--key', process.env.APPLE_API_KEY_PATH, '--key-id', process.env.APPLE_API_KEY, '--issuer', process.env.APPLE_API_ISSUER,
		], {signal})
		return {success: true, answer: JSON.parse(stdout)}
	} catch (e) {
		return {success: false, outcome: (e.stderr || e.message).trim()}
	}
}

//one of this mac's steps, between its started line and its finished or failed one: work comes to a result, whose outcome is the last line's words, and whose subject, value and log, when it has them, are what the step turned out to be about and where its story now goes
async function step(log, name, subject, words, work) {
	record(log, name, 'started', subject, '-', words)
	let result = await work()
	record(result.log || log, name, result.success ? 'finished' : 'failed', result.subject || subject, result.value || '-', result.outcome || '')
	return result
}

function record(log, ...line) {//a line in the build log, and the same words in the application's own log
	appendFileSync(log, `\n${buildLog(...line)}\n`)
}

//run one program to its end without holding up the waiting room, with its output added to the log at log, and say how it went. a bottom gate like askApple, for the programs whose work is the point rather than their answer
function run(command, args, {log, env = process.env, cwd = root}) {
	note(log, `$ ${command} ${args.join(' ')}`)
	let output = openSync(log, 'a')
	let child = spawn(command, args, {cwd, env, stdio: ['ignore', output, output]})
	closeSync(output)//the child holds its own copy from the moment it starts
	return new Promise(resolve => {
		child.on('error', e => resolve({success: false, outcome: `${command} could not start: ${e.message}`}))
		child.on('close', code => resolve(code == 0 ? {success: true} : {success: false, outcome: `${basename(command)} ${args[0]} exited ${code}`}))
	})
}

function note(log, text) {//a line in the log at log, with the time it was written
	appendFileSync(log, `\n${sayTime(Date.now(), {precise: true})}  ${text}\n`)
}

function readVersion() {//read fresh rather than once, since the waiting room stays open across a version bump
	return JSON.parse(readFileSync(join(desktop, 'src-tauri/tauri.conf.json'), 'utf8')).version
}

//pnpm notarize in desktop: b and then n, for a release that needs nothing from the waiting room, each step's lines printed as it starts and ends. b hands back the application already kept for unchanged code, so running this again after a stop carries on from where the last run got to. the dmg ends in the bundle folder, as pnpm installer's does, and hash and upload stay commands of their own, as the deliberate acts they are
async function main() {
	if (process.platform != 'darwin') throw new Error('notarization is apple\'s, and only a mac submits to it')
	let began = Date.now()
	let finish = result => console.log(`${sayTime(Date.now(), {precise: true})}  ${result.outcome}, took ${sayDuration(Date.now() - began, {precise: true})}`)

	let built = await build()
	if (!built.success) { finish(built); process.exitCode = 1; return }
	let notarized = await notarize(built.application, {signal: new AbortController().signal})//never aborted: ctrl-c ends this whole process, and apple keeps whatever it has
	finish(notarized)
	if (!notarized.success) { process.exitCode = 1; return }
	console.log('pnpm hash and pnpm upload publish it')
}

if (process.argv[1] && realpathSync(process.argv[1]) == fileURLToPath(import.meta.url)) main().catch(e => { console.error('🚧 Error:', e); process.exitCode = 1 })//only when run, so the waiting room can import the rest, and said the way desktop/dmg.js gives the reason for
