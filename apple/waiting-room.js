import {styleText} from 'node:util'
import {createPrompt, useState, useEffect, useKeypress, isUpKey, isDownKey} from '@inquirer/core'
import {buildLogSize, quietBuildLog, readBuildLog, sayDuration, sayTime, stepsUnderWay} from '../build-log.js'
import {listApplications, standing, findSubmissions, readHistory, readLive, readStaged, build, notarize, hash, upload} from './notarize.js'

/*
The waiting room the mac release through Apple is made from, named for what a release spends most of its time here doing. It lists every application, newest first, with where each stands and a clock on anything at Apple, then every step under way on this mac, whichever terminal started it, and four keys act: b builds a new application, and n, h and u notarize, hash and upload the one selected. notarize.js does the work, and README.md is the guide.

@inquirer/core lets one prompt be the whole program: a function of hooks that returns the screen as text, redrawn whenever its state changes, alive until it calls done. A timer, an answer from Apple or a step finishing can change the state as well as a key can, which lets the clocks tick while the keys still answer.

One job runs at a time, and nothing starts without a key. Every step taken on this mac is a line in the build log, which build-log.js describes, as it starts and as it ends, and the waiting room looks at it every second. A new line, from a key here or a pnpm notarize in another terminal, is the news that something happened: the waiting room shows the steps under way from it, and reads the folders, Apple's history and the site again at once, as it does every half minute anyway. Quitting is always safe: a wait on Apple ends at once, since Apple keeps going and n finds the submission again, and work on this mac, a build or a dmg on its way to Apple, finishes before the waiting room closes.
*/

const checkEvery = 30*1000//how often to read the folders, apple and the site again; the clocks between run on the times they gave

let job      = false//the promise of the one job running, or false. outside the screen, so ctrl-c can wait for it after the screen has gone
let quitting = new AbortController()//aborted at q or ctrl-c: a wait on apple ends at once, and work on this mac finishes first
let drawing  = true//false once the screen has closed, after which nothing may set its state
quietBuildLog()//the steps this process takes appear on its screen as under way, rather than printed over it

let screen = createPrompt((configuration, done) => {
	let [now, setNow]                   = useState(Date.now())
	let [applications, setApplications] = useState(listApplications)
	let [chosen, setChosen]             = useState('')//the folder the arrows chose, or blank for the newest, held by name so a row arriving from another terminal never moves the choice
	let [apple, setApple]               = useState(false)//the latest answer from apple's history, or false before the first
	let [staged, setStaged]             = useState(readStaged)//the sha256 of the dmg hash staged, or blank
	let [live, setLive]                 = useState('')//the sha256 of the dmg the site serves, or blank
	let [asks, setAsks]                 = useState(0)//counted up to read the folders, apple and the site again now, rather than at the next half minute
	let [logged, setLogged]             = useState(readBuildLog)//every line of the build log, read again whenever it grows
	let [message, setMessage]           = useState('')//what the last key came to
	let [closing, setClosing]           = useState(false)

	useEffect(() => {
		let size = buildLogSize()
		let timer = setInterval(() => {
			setNow(Date.now())//a new time is a new state, and a new state redraws the screen
			let grown = buildLogSize()
			if (grown == size) return
			size = grown
			setLogged(readBuildLog())
			setAsks(count => count + 1)//a line from any terminal, so look at everything again now
		}, 1000)
		return () => { clearInterval(timer); drawing = false }//cleanups run as the screen closes, before anything else could draw on it
	}, [])

	useEffect(() => {
		let controller = new AbortController()//so quitting cancels an ask still on its way, rather than waiting for it
		let ask = () => {
			setApplications(listApplications())
			setStaged(readStaged())
			readHistory(controller.signal).then(answer => { if (!controller.signal.aborted) setApple(answer) })
			readLive(controller.signal).then(sha256 => { if (!controller.signal.aborted) setLive(sha256) })
		}
		ask()
		let timer = setInterval(ask, checkEvery)
		return () => { clearInterval(timer); controller.abort() }
	}, [asks])

	let selected = Math.max(0, applications.findIndex(a => a.folder == chosen))
	let application = applications[selected]//the one n, h and u act on, or undefined in an empty list

	//run one job, the only one, its steps showing as under way through the build log, and what it came to on the message line
	function start(work) {
		if (job) { setMessage('one thing at a time, and the one under way is still going'); return }
		setMessage('')
		job = work({signal: quitting.signal}).then(result => {
			job = false
			if (!drawing) return
			setMessage(result.outcome)
			if (result.application) setChosen(result.application.folder)//a build lands on its application, the new one or the one with the same code
			setAsks(count => count + 1)//which reads the folders again too
		}).catch(e => { console.error('🚧 Error:', e); process.exit(1) })//a throw from inside a job is a bug, and this is its top gate
	}

	useKeypress(key => {
		if (closing) return
		if (key.name == 'q') {
			quitting.abort()
			setClosing(true)
			if (job) job.then(() => done())
			else done()
			return
		}
		if (isUpKey(key) && application)   setChosen(applications[Math.max(0, selected - 1)].folder)
		if (isDownKey(key) && application) setChosen(applications[Math.min(applications.length - 1, selected + 1)].folder)
		if (key.name == 'b') start(build)
		if (!['n', 'h', 'u'].includes(key.name)) return
		if (!application) { setMessage('b builds the first application'); return }
		if (key.name == 'n') start(options => notarize(application, options))
		if (key.name == 'h') start(() => hash(application))
		if (key.name == 'u') start(() => upload(application))
	})

	if (closing && !job) return 'apple waiting room closed'//the line left in scrollback

	let lines = [styleText('bold', 'apple waiting room'), '']
	let width = Math.max(0, ...applications.map(a => a.folder.length))
	applications.forEach((a, i) => {
		let row = `${a.folder.padEnd(width)}  ${sayStatus(standing(a, {apple, staged, live}), now)}`
		lines.push(i == selected ? styleText('cyan', '❯ ' + row) : '  ' + row)
	})
	if (applications.length == 0) lines.push('  No applications yet.')
	if (application) lines.push('', ...sayDetails(application, apple).map(line => styleText('dim', '    ' + line)))
	let underWay = stepsUnderWay(logged)
	if (underWay.length) lines.push('', 'under way', ...sayUnderWay(underWay, now).map(line => '    ' + line))
	if (!apple) lines.push('', styleText('dim', 'asking Apple...'))
	else if (!apple.success) lines.push('', styleText('red', `couldn't ask Apple: ${apple.outcome}`))
	if (closing) lines.push('', 'Quitting once the work on this mac finishes...')
	else if (message) lines.push('', message)
	lines.push('', styleText('dim', 'b build · n notarize · h hash · u upload · ↑↓ choose · q quit'))
	return lines.join('\n')
})

function sayStatus({status, since, until}, now) {//like Notarizing .app... (12 minutes, 3 seconds), the clock counting while Apple examines a part and stopped once a step has finished
	let said = since > 0 ? `${status} (${sayDuration((until || now) - since)})` : status
	return status == 'Rejected' ? styleText('red', said) : said
}

function sayDetails(application, apple) {//the selected application's two submissions as apple's history has them, then when it was notarized and its hash
	let lines = []
	if (apple && apple.success) {
		let submissions = findSubmissions(application, apple.history)
		for (let part of ['app', 'dmg']) {
			let submission = submissions[part]
			if (submission) lines.push(`.${part}    ${submission.id}`, `        sent ${sayTime(submission.createdDate)} · ${submission.status}`)
		}
	}
	if (application.notarized) lines.push(`        notarized ${sayTime(application.finished)}` + (application.sent ? `, in ${sayDuration(application.finished - application.sent)}` : ''))
	if (application.sha256) lines.push(`sha256  ${application.sha256}`)
	return lines
}

function sayUnderWay(steps, now) {//each step under way on this mac, like dmg  Fuji_0.1.0_aarch64.dmg  of the app of ..., by pnpm notarize (4 seconds)
	let width = Math.max(...steps.map(line => line.subject == '-' ? 0 : line.subject.length))
	return steps.map(line => `${line.step.padEnd(9)}${(line.subject == '-' ? '' : line.subject).padEnd(width + 2)}${line.words} (${sayDuration(now - line.when)})`)
}

//ctrl-c closes the screen by rejecting with ExitPromptError, the library's way of saying the user left rather than a fault, so it ends like q: a job under way finishes first, with no screen left to show it
screen({}).catch(e => {
	if (e.name != 'ExitPromptError') { console.error('🚧 Error:', e); process.exitCode = 1; return }
	quitting.abort()
	if (job) console.log('closing once the work on this mac finishes; ctrl-c again stops it now')
})
