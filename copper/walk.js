//the walk of the sandbox process, on whatever machine runs it, through fuji --inventory and fuji --sandbox-check on a built fuji: pnpm walk runs every step, pnpm walk crash runs one, and the linux workspace's pnpm walk runs it in containers

import {spawn, spawnSync} from 'node:child_process'
import {existsSync, mkdirSync, readdirSync, readFileSync, rmSync, symlinkSync, writeFileSync} from 'node:fs'
import {homedir, tmpdir} from 'node:os'
import {dirname, join, resolve} from 'node:path'
import {fileURLToPath} from 'node:url'
import {Time, sayDuration} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in

/*
Each step says what it saw, and the reader judges; nothing here passes or fails but the check's own lines. The steps are the ones a sandbox process has to get right on a new platform: the walls, tried from inside by the check; the same folder in both processes, which must give the same file, and again named through a link, which the kernel judges by the name; Stop, which closes the request's stdin as the panel's Stop kills; a crash, which the request's test option makes the sandbox process do partway, since Windows has no signal to send another process; a hang, which the other test option makes it do a second in, standing in for a read the kernel never finishes, so Stop is seen to kill a process that has stopped listening; and the main process killed, which must take its sandbox process with it. Where the platform keeps crash reports, the crash step counts fuji's before and after, since a crash of the sandbox process should reach the main process and nothing else. On Windows the count means something only because the main process starts the sandbox process with its own error mode: node quiets Windows Error Reporting in every process it starts, and a sandbox process that inherited that passed this step with no wall at all.
*/

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const windows = process.platform == 'win32'
const fuji = process.env.FUJI || join(root, 'desktop/src-tauri/target/debug', windows ? 'fuji.exe' : 'fuji')//cargo build -p fuji makes it; FUJI names another, the one inside an installed app say
const scratch = join(tmpdir(), 'fuji-walk')
const many = resolve(root, process.env.WALK_MANY || 'node_modules/.pnpm')//a tree of many small files; WALK_MANY names another, where there is none
const big = resolve(root, process.env.WALK_BIG || 'desktop/src-tauri/target')//a tree that takes longer than a step waits, after any build; WALK_BIG names another
const video = ['.mp4', '.mkv', '.avi', '.mov', '.ts', '.webm', '.wmv', '.flv', '.mpg', '.vob', '.m4v', '.rm']

function take(name, folder, options, act) {//one inventory through fuji --inventory, and act, if given, two seconds in: stop closes stdin, kill kills the main process
	return new Promise(resolve => {
		let output = join(scratch, `${name}.txt`)
		let request = {folder, output, options: {sha1: false, sha256: false, blake3: false, indent: false, video, sandbox: false, crash: false, hang: false, ...options}}
		let began = Date.now(), acted = 0, out = '', err = ''
		let child = spawn(fuji, ['--inventory'])
		child.stdin.on('error', () => {})//a fuji that ended before reading its request, which the close below reports
		child.stdout.on('data', d => out += d)
		child.stderr.on('data', d => err += d)
		child.stdin.write(JSON.stringify(request) + '\n')//and left open, since its closing is Stop
		let timer = act ? setTimeout(() => { acted = Date.now(); if (act == 'stop') child.stdin.end(); else child.kill('SIGKILL') }, 2*Time.second) : null
		child.on('close', code => {
			clearTimeout(timer)
			let lines = existsSync(output) ? readFileSync(output, 'utf8').split('\n').filter(line => line).length : 0
			let ended = Date.now()
			let reading = ''
			try { reading = JSON.parse(out.trim().split('\n').pop()).path } catch {}//the counts fuji --inventory prints at the end, whose path is the file in hand when it ended
			console.log(`${name}: exit ${code} in ${sayDuration(ended - began, '#.##s')}, ${lines} lines${acted ? `, ended ${sayDuration(ended - acted, '#.##s')} after the ${act}` : ''}${acted && reading ? `, reading ${reading}` : ''}`)
			if (err.trim()) console.log(`   ${err.trim().slice(-300)}`)
			resolve(output)
		})
	})
}

function same(a, b) {//line for line, the accessed times aside, since the first run's reads move them
	if (!existsSync(a) || !existsSync(b)) return console.log('   one wrote no file')
	let load = path => readFileSync(path, 'utf8').split('\n').map(line => line.replace(/,?"accessed":"[^"]*"/, ''))
	let x = load(a), y = load(b)
	let differ = x.filter((line, i) => line != y[i]).length
	console.log(x.length == y.length && differ == 0 ? '   identical' : `   DIFFER: ${differ} lines, ${x.length} against ${y.length}`)
}

function sandboxes() {//the sandbox processes running now, by their switch
	if (windows) return spawnSync('powershell', ['-NoProfile', '-Command', "(Get-CimInstance Win32_Process | Where-Object CommandLine -like '*inventory-sandbox*').Count"], {encoding: 'utf8'}).stdout.trim() || '0'
	return String(spawnSync('pgrep', ['-f', 'inventory-sandbox'], {encoding: 'utf8'}).stdout.split('\n').filter(line => line).length)
}

function reports() {//fuji's crash reports where this platform keeps them, or blank where it keeps none this walk knows of
	let folders = windows ? ['ReportArchive', 'ReportQueue'].flatMap(name => [process.env.LOCALAPPDATA, process.env.ProgramData].filter(base => base).map(base => join(base, 'Microsoft/Windows/WER', name)))//the user's and the machine's, whichever the consent setting sends a report to
		: process.platform == 'darwin' ? [join(homedir(), 'Library/Logs/DiagnosticReports')]
		: ['/var/crash']
	let found = folders.filter(folder => existsSync(folder))
	if (!found.length) return ''
	return found.reduce((count, folder) => count + readdirSync(folder).filter(name => name.toLowerCase().includes('fuji')).length, 0)
}

async function main() {
	if (!existsSync(fuji)) { console.log(`no fuji at ${fuji}; cargo build -p fuji makes one, or FUJI names one`); process.exit(1) }
	mkdirSync(join(scratch, 'lab/inside'), {recursive: true})
	writeFileSync(join(scratch, 'lab/inside/a.txt'), 'inside')
	writeFileSync(join(scratch, 'lab/outside.txt'), 'a secret')
	let step = process.argv[2] || 'all'
	let run = name => step == 'all' || step == name

	if (run('check')) {
		console.log('check:')
		let check = spawnSync(fuji, ['--sandbox-check', join(scratch, 'lab/inside'), join(scratch, 'lab/outside.txt')], {encoding: 'utf8'})
		console.log((check.stdout + check.stderr).trim().split('\n').map(line => `   ${line}`).join('\n'))
	}
	if (run('same')) {
		for (let folder of [join(root, 'steel/samples'), many]) {
			let short = folder.split(/[\\/]/).pop()
			same(await take(`${short}-main`, folder, {sha1: true}), await take(`${short}-sandbox`, folder, {sha1: true, sandbox: true}))
		}
	}
	if (run('link')) {
		let link = join(scratch, 'through-a-link')
		rmSync(link, {force: true})
		symlinkSync(root, link, 'junction')//a junction on windows, which needs no privilege, and a symbolic link elsewhere
		let folder = join(link, 'steel/samples')
		same(await take('link-main', folder, {sha1: true}), await take('link-sandbox', folder, {sha1: true, sandbox: true}))
	}
	if (run('stop')) {
		await take('stop-main', big, {sha256: true}, 'stop')
		await take('stop-sandbox', big, {sha256: true, sandbox: true}, 'stop')
		console.log(`   sandbox processes left: ${sandboxes()}`)
	}
	if (run('crash')) {
		let before = reports()
		await take('crash', big, {sha256: true, sandbox: true, crash: true})
		await new Promise(resolve => setTimeout(resolve, 5*Time.second))//the system's reporter takes a moment to write
		console.log(`   sandbox processes left: ${sandboxes()}${before === '' ? '' : `; fuji's crash reports, ${before} before and ${reports()} after`}`)
	}
	if (run('hang')) {
		await take('hang', big, {sha256: true, sandbox: true, hang: true}, 'stop')//stuck a second in, and Stop a second after that: the lines are the first second's, and the path the file it stuck on
		console.log(`   sandbox processes left: ${sandboxes()}`)
	}
	if (run('kill')) {
		await take('kill', big, {sha256: true, sandbox: true}, 'kill')
		await new Promise(resolve => setTimeout(resolve, 500*Time.millisecond))
		console.log(`   sandbox processes left half a second after: ${sandboxes()}`)
	}
}
main().catch(e => { console.error(e); process.exit(1) })
