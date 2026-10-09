import {execFile} from 'node:child_process'
import {promisify, styleText} from 'node:util'
import {createPrompt, useState, useEffect, useKeypress, isUpKey, isDownKey, isEnterKey, isNumberKey} from '@inquirer/core'

/*
The console a mac release through Apple is watched from. For now it shows what Apple is examining and for how long, above a hello world menu that proves the library can carry the rest.

A prompt library asks one question and waits for the answer, then the next, which is the wrong shape for watching a submission that may take ten minutes or two days. @inquirer/core is the layer under Inquirer's own prompts, and it lets one prompt be the whole program: a function of hooks that returns the screen as text, redrawn whenever its state changes, alive until it calls done. So the menu and the views it leads to are state inside that one prompt, and nothing has to happen in order. A timer or an answer from Apple can change the state as well as a key can, which is what lets the clocks tick while the keys still answer.

The console keeps no record of its own. Apple's history lists every submission with its name, status and the time it arrived, so how long one has been waiting is the time since then, and a submission shows up here whatever made it: Tauri, a release command, or notarytool typed by hand.
*/

const items      = ['Hello', 'World', 'Fuji']//placeholders for the views a release will have
const checkEvery = 30*1000//how often to ask Apple again; the clocks between asks run on the start times it gave

let screen = createPrompt((configuration, done) => {
	let [opened]                = useState(Date.now())//when the console opened
	let [now, setNow]           = useState(Date.now())
	let [apple, setApple]       = useState(false)//the latest answer from readHistory, or false before the first
	let [selected, setSelected] = useState(0)//the menu item the arrow keys are on
	let [view, setView]         = useState('')//blank for the menu, or the item being shown
	let [closed, setClosed]     = useState(false)

	useEffect(() => {
		let timer = setInterval(() => setNow(Date.now()), 1000)//a new time is a new state, and a new state redraws the screen
		return () => clearInterval(timer)
	}, [])

	useEffect(() => {
		let controller = new AbortController()//so quitting cancels an ask still on its way, rather than waiting for it, or drawing after the console has closed
		let ask = () => readHistory(controller.signal).then(answer => { if (!controller.signal.aborted) setApple(answer) })
		ask()
		let timer = setInterval(ask, checkEvery)
		return () => { clearInterval(timer); controller.abort() }
	}, [])

	useKeypress(key => {
		if (key.name == 'q') { setClosed(true); done(); return }
		if (view) {
			if (key.name == 'escape' || key.name == 'left' || isEnterKey(key)) setView('')//any of the three goes back to the menu
			return
		}
		if (isUpKey(key))   setSelected(Math.max(0, selected - 1))
		if (isDownKey(key)) setSelected(Math.min(items.length - 1, selected + 1))
		if (isEnterKey(key) || key.name == 'right') setView(items[selected])
		if (isNumberKey(key) && Number(key.name) >= 1 && Number(key.name) <= items.length) setView(items[Number(key.name) - 1])//a digit opens its item directly
	})

	if (closed) return `apple console, closed after ${sayClock(now - opened)}`//the line left in scrollback

	let lines = [styleText('bold', 'apple console') + '  ' + styleText('dim', `open for ${sayClock(now - opened)}`), '']
	lines.push(...sayApple(apple, now), '')
	if (view) {
		lines.push(`${view}, from the apple workspace.`, '', styleText('dim', '← esc or enter to go back · q to quit'))
	} else {
		items.forEach((item, i) => lines.push(i == selected ? styleText('cyan', `❯ ${i + 1} ${item}`) : `  ${i + 1} ${item}`))
		lines.push('', styleText('dim', '↑↓ to move · enter or a number to open · q to quit'))
	}
	return lines.join('\n')
})

//ask apple for the team's submissions, newest first. this is a bottom gate: notarytool speaks in exit codes and execFile in exceptions, so both become a result the screen can show
async function readHistory(signal) {
	let missing = ['APPLE_API_KEY', 'APPLE_API_ISSUER', 'APPLE_API_KEY_PATH'].filter(name => !process.env[name])
	if (missing.length > 0) return {success: false, outcome: `.env at the monorepo root is missing ${missing.join(', ')}`}
	try {
		let {stdout} = await promisify(execFile)('xcrun', ['notarytool', 'history', '--output-format', 'json',
			'--key', process.env.APPLE_API_KEY_PATH, '--key-id', process.env.APPLE_API_KEY, '--issuer', process.env.APPLE_API_ISSUER,
		], {signal})
		return {success: true, history: JSON.parse(stdout).history, checked: Date.now()}
	} catch (e) {
		return {success: false, outcome: (e.stderr || e.message).trim()}
	}
}

function sayApple(apple, now) {//the lines about apple: each submission still being examined and for how long, or the newest one when none is
	if (!apple) return ['asking Apple…']
	if (!apple.success) return [styleText('red', `couldn't ask Apple: ${apple.outcome}`)]

	let lines = []
	let examining = apple.history.filter(submission => submission.status == 'In Progress')
	for (let submission of examining) {
		let arrived = new Date(submission.createdDate)
		lines.push(`🍎🕵🏻‍♂️ Apple has been examining ${submission.name} for ${sayDuration(now - arrived)}`)
		lines.push(styleText('dim', `   submitted ${arrived.toLocaleString()} · ${submission.id}`))
	}
	if (examining.length == 0) {
		lines.push('Nothing at Apple right now.')
		let newest = apple.history[0]
		if (newest) lines.push(styleText('dim', `   newest: ${newest.name}, ${newest.status}, submitted ${new Date(newest.createdDate).toLocaleString()}`))
	}
	lines.push(styleText('dim', `checked with Apple ${sayDuration(now - apple.checked)} ago`))
	return lines
}

function sayDuration(milliseconds) {//like 36 minutes and 5 seconds, or 2 hours and 22 minutes once it reaches an hour
	let seconds = Math.floor(milliseconds / 1000)
	let hours = Math.floor(seconds / 3600), minutes = Math.floor(seconds / 60) % 60
	if (hours > 0) return `${sayCount(hours, 'hour')} and ${sayCount(minutes, 'minute')}`
	if (minutes > 0) return `${sayCount(minutes, 'minute')} and ${sayCount(seconds % 60, 'second')}`
	return sayCount(seconds, 'second')
}
function sayCount(n, word) { return `${n} ${word}${n == 1 ? '' : 's'}` }

function sayClock(milliseconds) {//like 2:05:09, or 5:09 under an hour
	let seconds = Math.floor(milliseconds / 1000)
	let h = Math.floor(seconds / 3600), m = Math.floor(seconds / 60) % 60, s = seconds % 60
	let pad = n => String(n).padStart(2, '0')
	return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`
}

//ctrl-c closes the prompt by rejecting with ExitPromptError, which is the library's way of saying the user left rather than a fault, so it ends quietly like q
screen({}).catch(e => {
	if (e.name == 'ExitPromptError') return
	console.error('🚧 Error:', e)
	process.exitCode = 1
})
