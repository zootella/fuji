//WindowsTableSort.js's table: Microsoft's published sorting weights for Windows 11, read from the text file Microsoft offers and packed into weight.js beside it, which WindowsTableSort.js unpacks the first time it is asked. pnpm sort-table in the side folder, or node table.js in this one, fetches the file, checks its hash and writes the module; node table.js with a path reads a copy already on this machine. Run it again only to move to a later table, and read the diff
import {createHash} from 'node:crypto'
import {readFileSync, writeFileSync} from 'node:fs'
import {fileURLToPath} from 'node:url'
import {dirname, join} from 'node:path'

/*
Explorer puts names in order with Windows' own linguistic comparison, and that comparison is a table: four small numbers for every character, which Microsoft publishes for each version of Windows as a text file of twenty megabytes, most of it the orders Chinese, Japanese and Korean locales swap in. WindowsTableSort.js follows the order an English Windows gives, which is the file's default table and nothing after it but the expansions, the compatibility pairs and the jamo: sixty thousand characters, four bytes each.

The module this writes is that table as text a page can import: the four weights as four planes of 65,536 bytes, one byte per UTF-16 code unit, each plane written as runs, since a script's characters sit together and count upward; then the expansions, a character that sorts as two others, like æ as a and e; then the jamo, which old Hangul is composed from. A character the file gives no weight is four zeros, which is how Windows treats one too: it is not there.

The file is fetched from Microsoft and never kept in the repository, and its hash is written here, so the packed table beside it can be made again from the same bytes and checked by anyone.
*/

const tableUrl = 'https://download.microsoft.com/download/C/F/7/CF713A5E-9FBC-4FD6-9246-275F65C0E498/Windows%2011%20Sorting%20Weight%20Table.txt'
const tableHash = 'f0bab873ed8901ae17afc4848c7d9e0a7de4310942effb2b54b538411b26d542'//sha256 of the file, the same bytes Wine's build checks for
const here = dirname(fileURLToPath(import.meta.url))

let bytes = process.argv[2] ? readFileSync(process.argv[2]) : Buffer.from(await (await fetch(tableUrl)).arrayBuffer())
let hash = createHash('sha256').update(bytes).digest('hex')
if (hash != tableHash) throw new Error(`the table's hash is ${hash}, expected ${tableHash}`)

let part = '', section = '', version = 0
let weights = new Map()//code unit to [script, alphabetic, diacritic, case], as the file orders them
let expansions = []//pairs of code units, a character that sorts as these two
let jamo = []//[code unit, old, leading, vowel, trailing, weight]
for (let line of bytes.toString('utf8').split(/\r?\n/)) {
	line = line.replace(/\s*;.*$/, '')//a comment runs from a semicolon to the end of its line
	if (!line.trim()) continue
	let m
	if ((m = line.match(/^\s*(SORTKEY|SORTTABLES)\b/))) { part = m[1]; continue }
	if (/^\s*(ENDSORTKEY|ENDSORTTABLES)\b/.test(line)) { part = section = ''; continue }
	if ((m = line.match(/^\s*(DEFAULT|RELEASE|REVERSEDIACRITICS|DOUBLECOMPRESSION|INVERSECASING|MULTIPLEWEIGHTS|EXPANSION|COMPATIBILITY|COMPRESSION|EXCEPTION|JAMOSORT)(\s|$)/))) { section = m[1]; continue }
	if (part == 'SORTKEY' && section == 'DEFAULT') {
		if (!(m = line.match(/^\s*0x([0-9a-f]+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s*$/i))) throw new Error(`a line of the default table that is not a character and four weights: ${line}`)
		weights.set(parseInt(m[1], 16), [Number(m[2]), Number(m[3]), Number(m[4]), Number(m[5])])
	} else if (section == 'RELEASE') {
		if ((m = line.match(/NLSVERSION\s+0x([0-9a-f]+)/i))) version = parseInt(m[1], 16)
	} else if (section == 'EXPANSION') {
		if (!(m = line.match(/^\s*0x([0-9a-f]+)\s+0x([0-9a-f]+)\s+0x([0-9a-f]+)\s*$/i))) throw new Error(`a line of the expansions that is not three characters: ${line}`)
		let at = expansions.length/2
		let c = parseInt(m[1], 16)
		if (!weights.has(c)) weights.set(c, [2, 0, at & 0xff, at >> 8])//script 2 marks an expansion, and the last two weights say which; a character the default table already weighs keeps that weight, as ü does, whose expansion is for the German phone book's order alone
		expansions.push(parseInt(m[2], 16), parseInt(m[3], 16))//pushed even when unused, since a later expansion finds an earlier one by its place
	} else if (section == 'COMPATIBILITY') {
		if (!(m = line.match(/^\s*0x([0-9a-f]+)\s+0x([0-9a-f]+)\s*$/i))) throw new Error(`a line of the compatibility pairs that is not two characters: ${line}`)
		let to = weights.get(parseInt(m[2], 16))
		if (to) weights.set(parseInt(m[1], 16), to); else weights.delete(parseInt(m[1], 16))//the first character sorts exactly as the second
	} else if (section == 'JAMOSORT') {
		if (!(m = line.match(/^\s*0x([0-9a-f]+)\s+0x([0-9a-f]+)\s+0x([0-9a-f]+)\s+0x([0-9a-f]+)\s+0x([0-9a-f]+)\s+0x([0-9a-f]+)\s*$/i))) throw new Error(`a line of the jamo that is not a character and five numbers: ${line}`)
		jamo.push(m.slice(1).map(s => parseInt(s, 16)))
	}//every other section belongs to a locale's own order, which fuji's does not follow
}
if (!version || weights.size < 60000 || expansions.length < 1000 || jamo.length != 357) throw new Error(`read too little of the table: version ${version}, ${weights.size} weights, ${expansions.length/2} expansions, ${jamo.length} jamo`)

let out = []
for (let plane = 0; plane < 4; plane++) {//each weight as a plane of its own, where the runs are long
	let values = new Uint8Array(65536)
	for (let [c, w] of weights) values[c] = w[plane]
	pack(values, out)
}
out.push(expansions.length >> 8, expansions.length & 0xff)
for (let c of expansions) out.push(c >> 8, c & 0xff)
out.push(jamo.length >> 8, jamo.length & 0xff)
for (let [c, old, leading, vowel, trailing, weight] of jamo) out.push(c >> 8, c & 0xff, old, leading, vowel, trailing, weight)

function pack(values, out) {//a plane as runs: 0 a stretch of bytes as they are, 1 one byte repeated, 2 a byte and the ones counting up from it, each followed by its length in two bytes
	let i = 0, plain = []
	let flush = () => { while (plain.length) { let piece = plain.splice(0, 65535); out.push(0, piece.length >> 8, piece.length & 0xff, ...piece) } }
	while (i < values.length) {
		let same = 1, rising = 1
		while (i + same < values.length && same < 65535 && values[i + same] == values[i]) same++
		while (i + rising < values.length && rising < 65535 && values[i + rising] == values[i] + rising) rising++
		let run = Math.max(same, rising)
		if (run < 5) { plain.push(values[i]); i++; continue }//a run shorter than its own four bytes of header saves nothing
		flush()
		out.push(same >= rising ? 1 : 2, run >> 8, run & 0xff, values[i])
		i += run
	}
	flush()
}

let packed = Buffer.from(out).toString('base64')
let lines = [
	`//Microsoft's sorting weights for Windows 11, the default table at NLS version 0x${version.toString(16).padStart(8, '0')}, packed by table.js beside it, which says what is in here and how to make it again; WindowsTableSort.js is the one reader`,
	`export const windowsTableVersion = 0x${version.toString(16).padStart(8, '0')}`,
	`export const windowsTablePacked = '${packed}'`,
]
writeFileSync(join(here, 'weight.js'), lines.join('\n') + '\n')
console.log(`${weights.size} weights, ${expansions.length/2} expansions and ${jamo.length} jamo in ${out.length} bytes, ${packed.length} characters of text`)
