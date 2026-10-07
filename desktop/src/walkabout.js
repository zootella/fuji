import {watch} from 'vue'
import {homeDir} from '@tauri-apps/api/path'
import parse from 'path-browserify'
import {diskRead, diskWrite, diskStat} from './disk.js'
import {forwardize} from './components/library.js'
import {walkPage, walkListed, walkNextState, walkPreviousState, walkStart, walkNext} from './walk.js'
import {memoryReport} from './memory.js'
import {logTrouble} from './log.js'

/*
A temporary harness that walks the sheet by itself and writes what each page cost, so a session can measure the walk over a real disk without a hand on the button. Not a feature: it runs only when a start file exists, it names folders in full in its own file by the user's leave for this measurement, and it comes out of the tree when the measuring is done.

The start file is fuji-temp/walkabout.start under the home folder: the folder to walk from on its first line, and on its second, optionally, the most pages to walk before stopping, which is what makes one difficult step a test of its own. The harness starts the walk there and presses Next the moment Next is ready, until Next says there is nothing after or the limit is reached, so thumbnails load and are discarded as under a user pressing briskly. After every page it rewrites fuji-temp/walkabout.log whole, one row per page, so a crash loses nothing but the step in flight: the step number, the page's first folder, its buckets and thumbnails, the glances and their milliseconds, the seconds from the press to Next being ready again, which is when a user could press, the seconds from the press to both directions settled, which is when the walk is idle and its count final, the seconds since the start, and fuji's processes' memory in megabytes from the memory report.
*/

export async function walkaboutStart() {//the shell calls this once the sheet is on screen; it returns at once when there is no start file
	let temp = parse.join(forwardize(await homeDir()), 'fuji-temp')
	let startFile = parse.join(temp, 'walkabout.start'), logFile = parse.join(temp, 'walkabout.log')
	try { await diskStat(startFile) } catch { return }//no start file, no walkabout
	let lines = new TextDecoder().decode(new Uint8Array(await diskRead(startFile))).split('\n').map(line => line.trim()).filter(line => line)
	let folder = lines[0], limit = Number(lines[1]) || Infinity
	let rows = [`walkabout from ${folder}, at most ${limit} pages`, ['step', 'first folder', 'buckets', 'thumbnails', 'glances', 'glance ms', 'next s', 'settled s', 'elapsed s', 'fuji MB'].join('\t')]
	let began = performance.now()
	async function write() { await diskWrite(logFile, Array.from(new TextEncoder().encode(rows.join('\n') + '\n'))) }//whole, every time; a few hundred rows is nothing
	await write()
	let step = 0, pressed = performance.now()
	await walkStart(folder)
	while (true) {
		let nextAt = 0
		await walkaboutSettled(() => { nextAt = performance.now() })
		let page = walkPage.value
		let next = (nextAt - pressed) / 1000, settled = (performance.now() - pressed) / 1000
		let megabytes = await memoryReport().then(report => Math.round(report.processes.reduce((sum, p) => sum + p.bytes, 0) / 1048576)).catch(() => 0)
		step++
		rows.push([step, page[0]?.folder || '', page.length, page.reduce((count, bucket) => count + bucket.files.length, 0), walkListed.value.count, walkListed.value.milliseconds, next.toFixed(2), settled.toFixed(2), ((performance.now() - began) / 1000).toFixed(1), megabytes].join('\t'))
		await write()
		if (walkNextState.value != 'ready' || step >= limit) { rows.push(walkNextState.value == 'none' ? 'the end of the volume' : 'stopped at the limit'); await write(); return }
		pressed = performance.now()
		await walkNext()
	}
}

function walkaboutSettled(onNext) {//resolves once the look-ahead has settled both directions, which is when the walk is idle and its count is final; calls onNext the moment Next alone settles, which is when a user could press
	return new Promise(resolve => {
		let nextSeen = false
		let stop = watch([walkNextState, walkPreviousState], check)
		check()//both may have settled already
		function check() {
			if (!nextSeen && walkNextState.value != 'looking') { nextSeen = true; onNext() }
			if (walkNextState.value != 'looking' && walkPreviousState.value != 'looking') { stop(); resolve() }
		}
	})
}
