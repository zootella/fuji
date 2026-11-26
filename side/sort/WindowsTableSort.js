import {windowsTablePacked} from './weight.js'

/*
The order Windows' File Explorer puts names in, built from Microsoft's own table: the exact reference that fuji's WindowsSort.js approximates without a table, which the sort test holds it against on any platform, and which fuji could take up in its place.

Explorer sorts by name with StrCmpLogicalW, and on Windows 7 and later that function is one call to Windows' own linguistic comparison, CompareStringEx, with two flags: ignore case, and digits as numbers. It was measured on Windows 11 in November 2026: three hundred thousand random pairs through both, no disagreement, and every other set of flags disagreeing. So this is not a natural sort with rules of its own to guess at. It is Windows' collation, which is a table of four weights for every character and a way of building a key from them, and this file is that way, with the table beside it in weight.js, packed from the file Microsoft publishes by sort/table.js.

A key is built in levels, and two names compare by the first level that differs:

The first level is each character's script and its place in the script, so every symbol comes before every digit and every digit before every letter, and b follows a whatever accents or capitals either wears. A run of digits is one number here: how many digits it has once its leading zeros are gone, then the digits, then a byte that is smaller the more zeros led it. So 2 is before 10, and 01 is before 1, decided on the spot and ahead of every character after it, which is why x01z sorts before x1a. A number is at most 19 digits, and a longer run is a second number after the first, so a run of 20 digits can sort before a run of 19. Digits of other scripts are numbers too, each run in its own script's digits.
The second level is accents, so e is before é only when nothing in the first level told them apart.
The third is case and width, where ignoring case leaves a and A the same and a fullwidth letter after a plain one.
A fourth, present only for Japanese kana, tells small from full size, hiragana from katakana, and holds the marks that repeat or lengthen the character before them.
The last is for the hyphen and the apostrophe, and the characters like them, which Windows sets aside from the first level entirely: co-op sorts beside coop, and the hyphen breaks the tie, the name without one first and then the one whose hyphen comes later. Microsoft calls this a word sort.

Some characters are worth nothing at all, a soft hyphen or a zero width joiner, and a character the table has never heard of is one of those. Some sort as two, æ exactly as ae and ß exactly as ss. So two names that differ can have the same key, and Explorer was seen to break that tie by the characters' own numbers with case ignored, ss before ß, which is the last thing the comparison here does, with the names as they are after that so no two different names ever tie.

This is the order of Windows 11's table, NLS version 0x00060502, for a user whose language has no order of its own, as English has none. Windows 10's table gives the same order for every name in ASCII and for nearly every other; Microsoft's two files differ in Georgian, in the accents of voiced kana, and in a few hundred characters newer than Windows 10. A Swedish user's Explorer puts å after z, and a Japanese user's orders kanji by its own table, and this does neither.

How right it is was measured rather than argued, by sort/test.js on Windows 11. The key here is the key Windows writes, byte for byte, for every UTF-16 code unit alone and beside a letter and for 2.2 million random names in eleven mixes of scripts. The order is StrCmpLogicalW's for every pair of the 438 names in sort/names.js but those the next paragraph is about, for 1.1 million random pairs in the same mixes, and for the same names as real files asked of the shell's own folder object, which is what Explorer's view sorts by.

Windows compares two names by walking them side by side, and a key can only stand for one name at a time, so there are three corners where the walk and the keys part, and there this follows the keys. A character that weighs as a digit and is not counted with, a superscript, a fraction, a circled number, beside an ordinary number: Windows weighs the number's digits one at a time against it, which puts 10 before ² and ² before 3 while 3 is before 10, so Windows has no order there to match. A combining accent with no letter before it, at the start of a name or after a hyphen. And a single Korean letter from the block decomposed text is written in, beside the same letter as it is typed. A name a person or a program gave a file was not found to reach any of them; Korean and accented names as a Mac decomposes them are in the mixes above.

Wine's CompareStringEx is the one other implementation outside Windows, built from the same published table, and was read to learn how a key is put together where Microsoft's own description stops; none of it is copied here. It gives a kana iteration mark after a character that is not kana that character's accent, where Windows was seen to keep the mark's own, which is what this does.
*/

let table//the unpacked weights, made the first time a key is asked for
const cacheMost = 100000//names whose keys are kept before the cache starts over, more than any folder fuji has listed
let cache = new Map()//name to key, so a sort builds each name's key once however many comparisons the name is in

export default function windowsCompare(a, b) {//which of two entries comes first by name, as Explorer orders them
	let x = _windowsCached(a.name), y = _windowsCached(b.name)
	if (x != y) return x < y ? -1 : 1//a key is text whose every character is one byte of it, so text's own order is the order of the bytes
	if (a.name == b.name) return 0
	x = a.name.toUpperCase(); y = b.name.toUpperCase()//two different names with one key, like ss and ß: by their characters with case ignored, as Explorer was seen to order them
	if (x != y) return x < y ? -1 : 1
	return a.name < b.name ? -1 : 1//and names that differ only by case as they are, where Explorer has no order at all
}
function _windowsCached(name) {
	let key = cache.get(name)
	if (key === undefined) {
		if (cache.size >= cacheMost) cache = new Map()
		key = windowsKey(name)
		cache.set(name, key)
	}
	return key
}

function _windowsUnpack() {//the table from its text: four planes of 65,536 weights, the expansions and the jamo, as sort/table.js packed them
	let text = atob(windowsTablePacked), at = 0
	let byte = () => text.charCodeAt(at++)
	let two = () => { let n = byte() << 8; return n | byte() }
	let planes = []
	for (let p = 0; p < 4; p++) {
		let plane = new Uint8Array(65536), filled = 0
		while (filled < 65536) {
			let kind = byte(), length = two()
			if (kind == 0) for (let i = 0; i < length; i++) plane[filled++] = byte()//a stretch as it is
			else { let from = byte(); for (let i = 0; i < length; i++) plane[filled++] = kind == 1 ? from : from + i }//one byte repeated, or counting up
		}
		planes.push(plane)
	}
	let expansions = new Uint16Array(two())
	for (let i = 0; i < expansions.length; i++) expansions[i] = two()
	let count = two()
	let jamo = {old: new Uint8Array(count), leading: new Uint8Array(count), vowel: new Uint8Array(count), trailing: new Uint8Array(count), weight: new Uint8Array(count)}
	for (let i = 0; i < count; i++) {
		let k = _windowsJamo(two())
		jamo.old[k] = byte(); jamo.leading[k] = byte(); jamo.vowel[k] = byte(); jamo.trailing[k] = byte(); jamo.weight[k] = byte()
	}
	return {script: planes[0], place: planes[1], accent: planes[2], form: planes[3], expansions, jamo}
}
function _windowsJamo(c) {//a jamo's place in the jamo table, whose four blocks of Unicode sit end to end
	if (c >= 0x1100 && c <= 0x11ff) return c - 0x1100
	if (c >= 0xa960 && c <= 0xa97c) return c - 0xa960 + 0x100
	if (c >= 0xd7b0 && c <= 0xd7c6) return c - 0xd7b0 + 0x11d
	return c - 0xd7cb + 0x134
}

const scriptNothing = 0, scriptMark = 1, scriptExpansion = 2, scriptKana = 3, scriptJamo = 4, scriptExtensionA = 5, scriptWord = 6, scriptDigit = 13, scriptKanaWritten = 34, scriptPrivateFirst = 169, scriptPrivateLast = 175//the scripts the key treats specially, as the table numbers them
const formFullWidth = 0x01, formFullSize = 0x02, formKatakana = 0x20//bits of a kana's third weight
const formKept = 0x27//the bits of the third weight left once case is ignored, which drops capital and superscript
const zeros = [0x0030, 0x0660, 0x06f0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0be6, 0x0c66, 0x0ce6, 0x0d66, 0x0e50, 0x0ed0, 0x0f20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x1bb0, 0x1c40, 0x1c50, 0xa620, 0xa8d0, 0xa900, 0xaa50, 0xff10]//the zero of each run of ten digits Windows counts as numbers, from ASCII's to the fullwidth ones
const digitsMost = 19//the most digits in one number

export function windowsKey(name, written = false) {//a name's key, as text whose characters are the key's bytes; written makes it the key Windows writes out for LCMapStringEx, which differs from the one its comparison uses in one small way, and is here so a test can hold this against Windows byte for byte
	if (!table) table = _windowsUnpack()
	let t = table
	let first = [], accents = [], forms = [], words = [], kana = [[], [], [], []]
	let normal = (script, place, accent, form) => {//a character with a place in the first level
		first.push(script, place)
		if (script >= scriptPrivateFirst && script <= scriptPrivateLast) { first.push(accent); forms.push(form); return }//a private use character's second weight is part of its place
		accents.push(accent); forms.push(form)
	}
	let mark = accent => {//a combining mark adds to the accent of the character before it, so e and an acute weigh what é does
		if (!accents.length) return accents.push(accent)
		let sum = accents[accents.length - 1] + accent
		accents[accents.length - 1] = written ? sum & 0xff : sum//a written key is bytes, so a heavy mark on a heavy accent wraps around there, and the comparison was seen not to wrap, which text can follow since its characters hold more than a byte
	}
	let part = (script, place, accent, form) => {//one of the characters an expansion stands for
		if (!written && script == scriptNothing) return
		if (!written && script == scriptMark) return mark(accent)
		normal(script, place, accent, form)
	}
	let n = name.length
	for (let i = 0; i < n; i++) {
		let c = name.charCodeAt(i)//by UTF-16 code unit, as Windows goes, so a character beyond the first plane is its two surrogates, each with a weight
		let script = t.script[c], place = t.place[c], accent = t.accent[c], form = t.form[c] & formKept
		if (script == scriptNothing) continue
		else if (script == scriptMark) mark(accent)
		else if (script == scriptExpansion) {
			let k = accent | ((t.form[c] & 0x3f) << 8)//which expansion, kept in the weights an expansion has no use for
			for (;;) {
				let a = t.expansions[k*2], b = t.expansions[k*2 + 1]
				part(t.script[a], t.place[a], t.accent[a], t.form[a] & formKept)
				if (t.script[b] != scriptExpansion) { part(t.script[b], t.place[b], t.accent[b], t.form[b] & formKept); break }
				k = t.accent[b] | (t.form[b] << 8)//the second of the two is an expansion itself, as a character standing for three is written
			}
		} else if (script == scriptKana) {
			let extra = 3, kanaForm = form, found = true
			if (place <= 1) {//a mark that repeats the kana before it, 0, or lengthens its vowel, 1: it takes that kana's place
				found = false
				for (let j = i - 1; j >= 0; j--) {
					let p = name.charCodeAt(j), before = t.script[p]
					if (before == scriptNothing || before == scriptMark) continue
					if (before == scriptExpansion) break
					if (before != scriptKana) {//after something that is not kana, it takes that character's place and keeps its own accent, measured on Windows 11, where Wine gives it that character's accent too
						found = true; kanaForm = -1
						if (before >= scriptPrivateFirst && before <= scriptPrivateLast) { first.push(before, t.place[p], t.accent[p]); accents.push(accent); forms.push(2); script = scriptNothing }//a private use character's place is three bytes, and the mark's accent still goes to the second level
						else { script = before; place = t.place[p] }
						break
					}
					if (t.place[p] <= 1) continue
					kanaForm = t.form[p] & formKept
					let beforePlace = t.place[p]
					if (place == 1) { beforePlace &= 0x87; kanaForm = (kanaForm & ~formFullWidth) | (form & formFullWidth) }//lengthened, the kana before counts by its vowel alone
					extra = 4 + place
					place = beforePlace
					found = true
					break
				}
			}
			if (!found) { first.push(0xff, 0xff); continue }//a mark with nothing before it to repeat
			if (kanaForm != -1) {
				kana[0].push(0xc4 | (kanaForm & formFullSize)); kana[1].push(extra); kana[2].push(0xc4 | (kanaForm & formKatakana)); kana[3].push(0xc4 | (kanaForm & formFullWidth))
				script = scriptKanaWritten
			}
			if (script != scriptNothing) normal(script, place, accent, 2)
		} else if (script == scriptJamo) {
			i += _windowsHangul(t, first, name, i) - 1
			accents.push(2); forms.push(2)
		} else if (script == scriptExtensionA) {
			first.push(0xfd, 0xff, place, accent)
			accents.push(2); forms.push(2)
		} else if (script == scriptWord) {//a hyphen or an apostrophe: nothing in the first level, and in the last, how far into the first level it stood, counted backward so the later one sorts first
			let far = (-(first.length >> 1) - 1) & 0xffff
			words.push(far >> 8, far & 0xff, place, (form | (accent << 3)) & 0xff)
		} else if (script == scriptDigit && _windowsZero(c)) {
			let zero = _windowsZero(c), length = 1
			while (length < digitsMost && i + length < n) { let d = name.charCodeAt(i + length); if (d < zero || d > zero + 9) break; length++ }
			let leading = 0
			while (leading < length && name.charCodeAt(i + leading) == zero) leading++
			first.push(scriptDigit, 2, 2 + length - leading)//how many digits count, so a longer number is a larger one before any digit is looked at
			let high = 2
			for (let k = leading; k < length; k++) {//the digits two to a byte, each two more than itself so no byte is the 0 or 1 that end a level
				let d = name.charCodeAt(i + k) - zero
				if ((length - k) % 2) first.push((high << 4) + d + 2); else high = d + 2
			}
			first.push(0xfe - leading)//more leading zeros, smaller
			accents.push(accent); forms.push(form)
			i += length - 1
		} else normal(script, place, accent, form)
	}
	while (accents.length && accents[accents.length - 1] <= 2) accents.pop()//a level's trailing plain weights are left off, so a name sorts before a longer one that starts with it
	while (forms.length && forms[forms.length - 1] <= 2) forms.pop()
	let key = String.fromCharCode(...first, 1, ...accents, 1, ...forms, 1)
	if (kana[2].length) {
		let plain = [0xc4 | formFullSize, 3, 0xc4 | formKatakana, 0xc4 | formFullWidth], ends = [0xff, 2, 0xff, 0xff]
		for (let k = 0; k < 4; k++) {
			while (kana[k].length && kana[k][kana[k].length - 1] == plain[k]) kana[k].pop()
			key += String.fromCharCode(...kana[k], ends[k])
		}
	}
	return key + String.fromCharCode(1, ...words)
}

function _windowsZero(c) {//the zero of the run of ten digits this character is in, or 0 for a digit Windows does not count with, like a superscript
	for (let zero of zeros) if (c >= zero && c <= zero + 9) return zero
	return 0
}

function _windowsHangul(t, first, name, at) {//a jamo and the ones after it that make a syllable with it, written as the syllable they would compose and then each jamo's own weight; answers how many characters it took
	let j = t.jamo, n = name.length, i = at
	let leading = 0x5f, vowel = 0x60, trailing = -1//the fillers Unicode has for a syllable missing its first consonant or its vowel
	let c = name.charCodeAt(i)
	if (c >= 0x1100 && c <= 0x115f) { leading = c - 0x1100; i++ }
	else if (c >= 0xa960 && c <= 0xa97c) { leading = c - 0xa960 + 0x100; i++ }
	if (i < n) {
		c = name.charCodeAt(i)
		if (c >= 0x1160 && c <= 0x11a7) { vowel = c - 0x1100; i++ }
		else if (c >= 0xd7b0 && c <= 0xd7c6) { vowel = c - 0xd7b0 + 0x11d; i++ }
	}
	if (i < n) {
		c = name.charCodeAt(i)
		if (c >= 0x11a8 && c <= 0x11ff) { trailing = c - 0x1100; i++ }
		else if (c >= 0xd7cb && c <= 0xd7fb) { trailing = c - 0xd7cb + 0x134; i++ }
	}
	if (!j.old[leading] && !j.old[vowel] && (trailing == -1 || !j.old[trailing])) { i = at + 1; vowel = 0x60; trailing = -1 }//modern jamo go one at a time
	let leadingOff = Math.max(j.leading[leading], j.leading[vowel])
	let vowelOff = Math.max(j.vowel[leading], j.vowel[vowel])
	let trailingOff = Math.max(j.trailing[leading], j.trailing[vowel])
	if (trailing != -1) trailingOff = Math.max(trailingOff, j.trailing[trailing])
	let composed = 0xac00 + (leadingOff*21 + vowelOff)*28 + trailingOff, filler = 0
	if (leading == 0x5f || vowel == 0x60) { filler = 0x80; composed-- }
	if (composed < 0xac00) composed = 0x3260
	first.push(t.script[composed], t.place[composed], 0xff, j.weight[leading] | filler, 0xff, j.weight[vowel], 0xff, trailing != -1 ? j.weight[trailing] : 2)
	return Math.max(i - at, 1)
}
