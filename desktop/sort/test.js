//the test of the two platform sorts, WindowsSort.js and MacSort.js in src, against the platforms themselves, which pnpm sort-test runs from the desktop folder with the same words after it. node test.js checks both against what each platform was recorded saying, on any machine, and then asks the platform it is running on about random names by the hundred thousand; node test.js record asks this platform about every pair of the names in names.js and writes what it said beside this file; node test.js folder and a path makes the names into a folder of empty files, a subfolder a group, to look at in Explorer or Finder beside the order fuji gives, written there as text; node test.js shell and that path, on Windows, asks the shell's own folder object about the files in it, pair by pair; node test.js explorer on Windows, or finder on a Mac, and that path opens each folder in a real window of the file manager and reads back the order it shows
import {writeFileSync, readFileSync, readdirSync, existsSync, mkdirSync, mkdtempSync, rmSync} from 'node:fs'
import {execFileSync} from 'node:child_process'
import {fileURLToPath} from 'node:url'
import {dirname, join, resolve} from 'node:path'
import {tmpdir} from 'node:os'
import windowsCompare from '../src/WindowsSort.js'
import macCompare from '../src/MacSort.js'
import {sortGroups, sortNames, sortLoose} from './names.js'

/*
Each sort claims to be another program's order, so the only test worth having is that program. On Windows the oracle is oracle.ps1, which calls StrCmpLogicalW, the function Explorer sorts with, and LCMapStringEx for the keys beneath it; on the Mac it is oracle.swift, which calls localizedStandardCompare. Neither is there on the other platform, so what each said about the names in names.js is recorded in a file, windows-sort.txt and mac-sort.txt, and those travel: the Mac checks the windows sort against what Windows said, and Windows the mac sort against what the Mac said. A record is every pair of the names, so it does not depend on either side's sort being a proper order, which Windows' is not everywhere.

The random names are what find the mistakes nobody thought to write down, and they only run where the platform is. Windows' sort is exact for a name in ASCII and close for any other, by the engine's collation, so only the pools of ASCII, and of emoji, which it puts where Windows does, decide whether it passes; the other pools are counted and said, to show how close. The Mac's sort is exact, and every pool the Mac is asked about decides.
*/

const here = dirname(fileURLToPath(import.meta.url))
const platform = process.platform == 'win32' ? 'windows' : process.platform == 'darwin' ? 'mac' : ''
const sorts = {windows: (a, b) => windowsCompare({name: a}, {name: b}), mac: (a, b) => macCompare({name: a}, {name: b})}
const show = s => [...s].map(c => { let n = c.codePointAt(0); return n > 0x20 && n < 0x7f ? c : `\\u{${n.toString(16)}}` }).join('')//a name with everything outside plain ASCII as its number, so a report can be read and pasted

let scratch = ''
function ask(mode, names, folder) {//what this platform says about these names: for compare, two at a time, -1, 0 or 1 a pair; for keys, on Windows alone, each name's sort key as text of bytes
	if (!platform) throw new Error('neither Windows nor a Mac, so there is nobody to ask')
	if (!scratch) scratch = mkdtempSync(join(tmpdir(), 'fuji-sort-'))
	let parts = []
	for (let s of names) {
		let b = Buffer.alloc(2 + s.length*2)
		b.writeUInt16LE(s.length, 0)
		for (let i = 0; i < s.length; i++) b.writeUInt16LE(s.charCodeAt(i), 2 + i*2)
		parts.push(b)
	}
	let from = join(scratch, 'names'), to = join(scratch, 'said')
	writeFileSync(from, Buffer.concat(parts))
	if (platform == 'windows') execFileSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', join(here, 'oracle.ps1'), '-Mode', mode, '-In', from, '-Out', to, ...(folder ? ['-Folder', folder] : [])])
	else execFileSync('swift', [join(here, 'oracle.swift'), mode, from, to])
	let said = readFileSync(to)
	if (mode != 'keys') return [...said].map(n => n - 1)
	let keys = []
	for (let i = 0; i < said.length;) { let n = said.readUInt16LE(i); i += 2; keys.push(said.subarray(i, i + n).toString('latin1')); i += n }
	return keys
}
function who() {//the platform's line about itself: its version, and the table or the language its order follows
	if (platform == 'windows') return execFileSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', join(here, 'oracle.ps1')]).toString().trim()
	return execFileSync('swift', [join(here, 'oracle.swift')]).toString().trim()
}

function record() {//every pair of the names through this platform's comparison, written as a line a name: the name as JSON, a tab, and a character for each name after it, < = or >
	let names = sortNames(), pairs = []
	for (let i = 0; i < names.length; i++) for (let j = i + 1; j < names.length; j++) pairs.push(names[i], names[j])
	let said = ask('compare', pairs), at = 0
	let lines = [`# what ${platform == 'windows' ? "StrCmpLogicalW, Explorer's comparison," : "localizedStandardCompare, Finder's comparison,"} said about every pair of the names in names.js, written by node test.js record on ${who()}`, '# a line is a name as JSON, a tab, and for each name on a later line whether this one is before it, the same, or after: < = >']
	for (let i = 0; i < names.length; i++) {
		let row = ''
		for (let j = i + 1; j < names.length; j++) row += '<=>'[said[at++] + 1]
		lines.push(JSON.stringify(names[i]) + '\t' + row)
	}
	writeFileSync(join(here, `${platform}-sort.txt`), lines.join('\n') + '\n')
	console.log(`${names.length} names, ${pairs.length/2} pairs, written to ${platform}-sort.txt`)
}

function recorded(which) {//a sort against what its platform was recorded saying; true when every pair agrees outside the groups marked loose
	let path = join(here, `${which}-sort.txt`)
	if (!existsSync(path)) { console.log(`${which}: no ${which}-sort.txt yet, which pnpm sort-test record writes on ${which == 'windows' ? 'Windows' : 'a Mac'}`); return true }
	let rows = readFileSync(path, 'utf8').split('\n').filter(line => line && !line.startsWith('#')).map(line => { let tab = line.lastIndexOf('\t'); return [JSON.parse(line.slice(0, tab)), line.slice(tab + 1)] })
	let loose = sortLoose(), wrong = [], looseWrong = 0, pairs = 0, beyond = 0
	let ascii = name => /^[\x20-\x7e]*$/.test(name)
	for (let i = 0; i < rows.length; i++) for (let j = i + 1; j < rows.length; j++) {
		let said = rows[i][1][j - i - 1], a = rows[i][0], b = rows[j][0]
		pairs++
		if (said == '=') continue//the platform has no order for these two, so any is its order
		let mine = sorts[which](a, b)
		if ((mine < 0) == (said == '<')) continue
		if (which == 'windows' && !(ascii(a) && ascii(b))) beyond++//a name outside ASCII, where the windows sort is close and not exact
		else if (loose.has(a) || loose.has(b)) looseWrong++; else wrong.push(`  [${show(a)}] [${show(b)}] ${which} said ${said}`)
	}
	console.log(`${which}: ${wrong.length} of ${pairs} recorded pairs disagree${looseWrong ? `, and ${looseWrong} more over the names ${which == 'windows' ? 'Windows' : 'the Mac'} contradicts itself about` : ''}${beyond ? `, and ${beyond} more with a name outside ASCII, where the windows sort is close and not exact` : ''}`)
	for (let line of wrong.slice(0, 40)) console.log(line)
	return !wrong.length
}

let seed = 20261120
function random(n) {//a whole number below n, the same run after run, so a failure comes back; mulberry32
	seed = (seed + 0x6D2B79F5) | 0
	let t = Math.imul(seed ^ (seed >>> 15), 1 | seed)
	t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
	return Math.floor(((t ^ (t >>> 14)) >>> 0)/4294967296*n)
}
const range = (from, to) => { let all = []; for (let c = from; c <= to; c++) all.push(String.fromCharCode(c)); return all }
const u = (...codes) => String.fromCharCode(...codes)
const digitLike = new Set([0xb2, 0xb3, 0xb9, 0xbc, 0xbd, 0xbe, 0x1bb, 0x1bc, 0x1bd])//the characters of Latin that weigh as digits and are not counted with
const latin = range(0xa0, 0x24f).filter(c => !digitLike.has(c.charCodeAt(0)))
const accented = [...'aeiounAEIOUcyz'].flatMap(base => range(0x300, 0x30c).map(mark => base + mark))//a letter and a combining accent, as a Mac writes a name
const pools = {//pieces a random name is made of, each a character or a few that belong together
	'ASCII': range(0x20, 0x7e),
	'digits and little else': [...'0001239', ...'abAB-_. \''],
	'Latin with accents': [...range(0x20, 0x7e), ...latin, ...accented, 'e' + u(0x301, 0x302), 'o' + u(0x323, 0x302)],
	'composed and decomposed': [...[...range(0xc0, 0x17f), ...range(0x1e00, 0x1eff), ...range(0x386, 0x3ce), ...range(0x400, 0x45f), ...range(0x304c, 0x3094), ...range(0x30ac, 0x30fa)].flatMap(c => c.normalize('NFD') == c ? [c] : [c, c.normalize('NFD')]), ...'abeo1 -.'],
	'digits of several scripts': [...'0129', u(0xff10), u(0xff11), u(0xff19), u(0x660), u(0x661), u(0x669), u(0x966), u(0x967), u(0xe50), u(0xe51), 'a', '.', '-'],
	'kana': [...range(0x3041, 0x3096), ...range(0x30a1, 0x30fa), ...range(0xff66, 0xff9d), u(0x30fc), u(0xff70), u(0x309d), u(0x309e), u(0x30fd), u(0x30fe), u(0x304b, 0x3099), u(0x306f, 0x309a), u(0xff76, 0xff9e), u(0xff8a, 0xff9f), 'a', '1', '-', ' ', u(0x4e00), u(0x65e5)],
	'Hangul': [...range(0xac00, 0xac60).flatMap(c => [c, c.normalize('NFD')]), ...[0xd55c, 0xae00, 0xd30c, 0xc77c, 0xd7a3, 0xb9c8].flatMap(c => [u(c), u(c).normalize('NFD')]), 'a', '1', ' ', '-', '.'],
	'Chinese characters': [...range(0x4e00, 0x4e80), ...range(0x3400, 0x3440), ...range(0x9fa0, 0x9fff), ...range(0xf900, 0xf920), ...range(0xff21, 0xff5a), u(0x3005), u(0x3000), 'a', '1', '-'],
	'emoji': [u(0xd83d, 0xde00), u(0xd83d, 0xde01), u(0xd83c, 0xdf89), u(0xd83d, 0xdc68, 0x200d, 0xd83d, 0xdc69), u(0x2764, 0xfe0f), u(0x2764), u(0xd840, 0xdc00), u(0xd835, 0xdc9c), u(0xd83c, 0xddfa, 0xd83c, 0xddf8), u(0xa9), u(0x2122), u(0x263a), ...'ab1- _.'],
	'alphabets': [...range(0x391, 0x3c9), ...range(0x410, 0x44f), ...range(0x531, 0x556), ...range(0x561, 0x586), ...range(0x10d0, 0x10fa), ...range(0x5d0, 0x5ea), ...range(0x621, 0x64a), ...range(0xe01, 0xe2e), ...range(0x905, 0x939), 'a', '1', '-', ' '],
	'everything together': [...range(0x20, 0x7e), ...latin.slice(0, 200), ...range(0x391, 0x3c9), ...range(0x410, 0x44f), ...range(0x2010, 0x2015), ...range(0x3041, 0x3096), ...range(0x30a1, 0x30fa), ...range(0x4e00, 0x4e20), ...range(0x3400, 0x3410), u(0xd83d, 0xde00), u(0xd83d, 0xde0a), ...range(0xe000, 0xe005), u(0xad), u(0x200b), u(0x200d), u(0xfeff), u(0xdf), u(0xe6), u(0xfb01), u(0x2122), 'e' + u(0x301), 'u' + u(0x308)],
}
const windowsExact = ['ASCII', 'digits and little else', 'emoji']//the pools the windows sort is exact on, which decide whether it passes; on the others it follows the engine's collation and is only counted
const macPools = ['ASCII', 'digits and little else', 'Latin with accents', 'composed and decomposed']//the pools the Mac is asked about from node, whose ICU is its own and not the system's that Finder and fuji's page share, so only the scripts every ICU agrees on are a test of MacSort.js here
function pairsFrom(pool, count, longest) {//pairs of random names, two in three of which start the same, since two names with nothing in common are settled by their first characters and test little
	let names = []
	let make = n => { let pieces = []; for (let k = 0; k < n; k++) pieces.push(pool[random(pool.length)]); return pieces }
	for (let i = 0; i < count; i++) {
		let pieces = make(1 + random(longest))
		let kind = random(3), other
		if (kind == 0) other = [...pieces.slice(0, random(pieces.length + 1)), ...make(1 + random(3))]//the same start and a different end
		else if (kind == 1) { other = [...pieces]; other.splice(random(other.length + 1), random(2), ...make(random(2))); if (!other.length) other = make(1) }//a piece added, taken or changed
		else other = make(1 + random(longest))
		names.push(pieces.join(''), other.join(''))
	}
	return names
}
function fuzz() {//random names through this platform's comparison and through fuji's; true when they agree on every pair
	let good = true, count = Number(process.env.SORT_PAIRS || 100000)
	for (let [title, pool] of Object.entries(pools)) {
		if (platform == 'mac' && !macPools.includes(title)) continue
		let names = pairsFrom(pool, count, title.startsWith('digits') ? 24 : 8)
		let wrong = []
		let said = ask('compare', names)
		for (let i = 0; i < names.length; i += 2) {
			if (said[i/2] == 0) continue
			let mine = sorts[platform](names[i], names[i + 1])
			if ((mine < 0) != (said[i/2] < 0)) wrong.push([names[i].length + names[i + 1].length, `  [${show(names[i])}] [${show(names[i + 1])}] ${platform} said ${'<=>'[said[i/2] + 1]}`])
		}
		let exact = platform != 'windows' || windowsExact.includes(title)
		console.log(`${title}: ${wrong.length} of ${count} random pairs disagree${exact ? '' : `, ${(100*wrong.length/count).toFixed(1)} percent, where the windows sort is close and not exact`}`)
		wrong.sort((a, b) => a[0] - b[0])//the shortest first, which say the most
		if (exact) for (let w of wrong.slice(0, 12)) console.log(w[1])
		if (wrong.length && exact) good = false
	}
	return good
}

function folder(root) {//the names as empty files, a subfolder a group, with fuji's order for each written beside them
	if (!root) throw new Error('node test.js folder and the path of a folder to make')
	if (existsSync(root)) throw new Error(`${root} is already there, and this makes a folder rather than adding to one`)
	mkdirSync(root, {recursive: true})
	let illegal = platform == 'windows' ? name => /[<>:"/\\|?*\u0000-\u001f]/.test(name) || /[. ]$/.test(name) : name => /[/:\u0000]/.test(name)//what this platform will not have in a file's name
	let lines = [`What ${platform == 'windows' ? 'Explorer' : 'Finder'} should show in each folder here, sorted by name, when fuji's ${platform} sort is right.`, `Made by node test.js folder on ${platform ? who() : process.platform}.`, '']
	sortGroups.forEach((group, n) => {
		let title = `${String(n + 1).padStart(2, '0')} ${group.title}`
		mkdirSync(join(root, title))
		let made = [], skipped = []
		for (let name of group.names) {
			if (illegal(name)) { skipped.push(name); continue }
			try { writeFileSync(join(root, title, name), '', {flag: 'wx'}); made.push(name) } catch (e) { skipped.push(name) }//wx so a name the volume takes for one already there is skipped rather than written over
		}
		lines.push(`${title}: ${group.shows}`)
		if (group.loose) lines.push('  (no order here can be called right or wrong on Windows)')
		for (let which of platform == 'mac' ? ['mac', 'windows'] : ['windows', 'mac']) {
			lines.push(`  as ${which == 'windows' ? 'Explorer' : 'Finder'} orders them:`)
			for (let name of [...made].sort(sorts[which])) lines.push(`    ${name}${name == show(name) ? '' : `    ${show(name)}`}`)
		}
		if (skipped.length) lines.push(`  not made here: ${skipped.map(show).join('  ')}`)
		lines.push('')
	})
	writeFileSync(join(root, 'fuji-sort.txt'), '﻿' + lines.join('\r\n'))//with a mark and Windows' line ends, so Notepad reads the names right
	console.log(`${sortGroups.length} folders in ${root}, and fuji-sort.txt beside them`)
}

function shell(root) {//the files of a folder this made, every pair in each subfolder, through the shell's folder object, which breaks the ties StrCmpLogicalW leaves; true when fuji's order agrees on every pair the shell has an order for
	if (platform != 'windows') throw new Error("the shell is Windows' own, so this runs there")
	let good = true
	for (let entry of readdirSync(root, {withFileTypes: true})) {
		if (!entry.isDirectory()) continue
		let names = readdirSync(join(root, entry.name)), pairs = []
		for (let i = 0; i < names.length; i++) for (let j = i + 1; j < names.length; j++) pairs.push(names[i], names[j])
		let said = ask('shell', pairs, resolve(root, entry.name)), wrong = [], none = 0
		for (let i = 0; i < pairs.length; i += 2) {
			if (said[i/2] == 0) { none++; continue }
			if ((sorts.windows(pairs[i], pairs[i + 1]) < 0) != (said[i/2] < 0)) wrong.push(`  [${show(pairs[i])}] [${show(pairs[i + 1])}] the shell said ${'<=>'[said[i/2] + 1]}`)
		}
		let loose = sortGroups.find(group => entry.name.endsWith(group.title))?.loose
		console.log(`${entry.name}: ${wrong.length} of ${pairs.length/2} pairs disagree${none ? `, ${none} the shell has no order for` : ''}${loose && wrong.length ? ', in the group where Windows contradicts itself' : ''}`)
		for (let line of wrong.slice(0, 12)) console.log(line)
		if (wrong.length && !loose) good = false
	}
	return good
}

const manager = {windows: 'Explorer', mac: 'Finder'}[platform]//the file manager whose windows the order shown is read from
function shown(root) {//the folder this made, shown in real windows of the platform's file manager, one a group, each sorted by name and read back top to bottom: the order a person would see, held against fuji's whole; true when every folder but the one Windows contradicts itself about comes out the same
	if (!scratch) scratch = mkdtempSync(join(tmpdir(), 'fuji-sort-'))
	let to = join(scratch, 'shown')
	if (platform == 'windows') execFileSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', join(here, 'oracle.ps1'), '-Mode', 'explorer', '-Folder', resolve(root), '-Out', to])
	else execFileSync('swift', [join(here, 'oracle.swift'), 'finder', resolve(root), to])
	let seen = new Map()//a folder's name to the names the window showed in it, in order
	for (let line of readFileSync(to, 'utf16le').replace(/^﻿/, '').split(/\r?\n/)) {
		if (!line) continue
		let tab = line.indexOf('\t')
		if (!seen.has(line.slice(0, tab))) seen.set(line.slice(0, tab), [])
		seen.get(line.slice(0, tab)).push(line.slice(tab + 1))
	}
	let good = true
	for (let [title, names] of seen) {
		let mine = [...names].sort(sorts[platform])
		let at = names.findIndex((name, i) => name != mine[i])
		let loose = platform == 'windows' && sortGroups.find(group => title.endsWith(group.title))?.loose
		if (at == -1) { console.log(`${title}: all ${names.length} in fuji's order`); continue }
		console.log(`${title}: differs at row ${at + 1} of ${names.length}, where ${manager} shows [${show(names[at])}] and fuji has [${show(mine[at])}]${loose ? ', in the group where Windows contradicts itself' : ''}`)
		if (!loose) good = false
	}
	return good
}

let verb = process.argv[2]
try {
	if (verb == 'record') record()
	else if (verb == 'folder') folder(process.argv[3])
	else if (verb == 'shell') { if (!shell(process.argv[3])) process.exitCode = 1 }
	else if (verb == 'explorer' || verb == 'finder') {
		if ({explorer: 'windows', finder: 'mac'}[verb] != platform) throw new Error(`${verb} reads the windows of ${verb == 'explorer' ? "Windows' Explorer" : "the Mac's Finder"}, so it runs there`)
		if (!shown(process.argv[3])) process.exitCode = 1
	}
	else {
		let good = recorded('windows')
		good = recorded('mac') && good
		if (platform && verb != 'recorded') { console.log(who()); good = fuzz() && good }
		if (!good) process.exitCode = 1
	}
} finally {
	if (scratch) rmSync(scratch, {recursive: true, force: true})
}
