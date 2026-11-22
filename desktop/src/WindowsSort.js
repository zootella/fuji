/*
The order Windows' File Explorer puts names in, on any platform, so a folder arranged in Explorer reads the same in fuji: exactly for a name in ASCII, and closely for every other.

Explorer sorts by name with StrCmpLogicalW, and on Windows 7 and later that function is one call to Windows' own linguistic comparison, CompareStringEx, with two flags: ignore case, and digits as numbers. It was measured on Windows 11 in November 2026: three hundred thousand random pairs through both, no disagreement, and every other set of flags disagreeing. That comparison is a table of four weights for every character, which Microsoft keeps, and a way of walking two names by it. This file carries no table. It is Windows' way for the characters of ASCII written out as rules, and for every other character the collation the web engine already has.

The rules, for ASCII, each seen in Windows and held against it:

Every symbol comes before every digit, and every digit before every letter. The symbols are in an order of Windows' own, which is windowsSymbols below, the space first.
A run of digits is one number, so 2 is before 10. More zeros in front come first when the numbers are the same, so 01 is before 1, and that is decided on the spot, ahead of every character after it, which is why x01z sorts before x1a. A number is at most 19 digits, and a longer run is a second number after the first.
Letters are in the alphabet's order with case ignored.
The apostrophe and the hyphen are set aside: they are not there when names are first compared, so co-op sorts beside coop, and only then break the tie, the name without one first, and of two with one, the one whose mark comes later. Microsoft calls this a word sort. The dashes typed in place of a hyphen go with them, and are windowsWords below.
Two names the same in all of that, which differ only in case, go by their characters as they are, where Explorer has no order at all.

For every character outside ASCII the order is the web engine's own, Intl.Collator for English, which is CLDR's order for a language with none of its own. It puts an accented letter with its plain one and tells them apart only when nothing else does, as Windows does; it knows every alphabet and its order; and where it puts a character before the digits that character is a symbol here, and where it does not, a letter. A name is read in its composed form, so a letter and its accent written apart, as a Mac writes them, are the letter they make. A character beyond Unicode's first plane, an emoji above all, goes after every letter, by its own number, which is where Windows puts one.

How right it is was measured, not argued, on Windows 11 on 2026-11-21, against StrCmpLogicalW itself. Every pair of the 95 characters of ASCII, alone, before a letter, after one and between two: no disagreement in 17,756 pairs. A hundred thousand random pairs of names in each of eleven mixes: none in ASCII, none in digits with little else, none in emoji; 0.4 percent in accented names written composed against decomposed; 0.7 percent in everything mixed together; 3.1 percent in Latin with accents, the misses among the rare letters of extended Latin, a click letter, an Icelandic thorn; 3.2 percent in kana; 3.5 percent in nine alphabets mixed, where each alphabet is in its own right order and the misses are which alphabet comes before which; 17 percent in Korean, in names whose syllables are written out in parts, as older Macs stored them, where Korean as it is typed had none among the chosen names; 19 percent in the digits of other scripts, fullwidth or Arabic-Indic, which Windows counts as numbers and this does not; and 21 percent in Chinese characters, which come out in one steady order that is not Windows'. Of 95,661 pairs among 438 names chosen to show one thing each, 1.9 percent disagree, none of them in the ten groups of ASCII names, none in a real folder's names, and most where a name of one script meets a name of another.

So a name in ASCII is in Explorer's order, which is nearly every name most people have. A name in another script is in a right and steady order for its language that is close to Explorer's and is not promised to be it. A Swedish user's Explorer puts å after z, and a Japanese user's orders kanji by its own table, and this does neither, since it is the order for a user whose language has none of its own, as English has none.

For a day fuji did carry the table. Microsoft publishes its sorting weights for Windows 11 as a text file, and from 2026-11-20 this file built the very key Windows builds from them, byte for byte, exact on every one of those mixes. The table came to 69 KB of base64 in the source, sixty thousand characters' weights packed by a script, and the user took it out the next day: a block that long cannot be read, and a reader has to trust where it came from, where two short strings and some rules can be checked by eye and against any Windows machine. Exact order for names outside ASCII was the price, weighed and paid. The two strings below were read from that table, which matched Windows on every character, and then held against Windows as the measurements above say.

MacSort.js beside this is built the same way, the platform's own rules for names written out over the engine's collation, and stands on firmer ground. Apple publishes the source of the comparison Finder uses, and Finder's comparison is itself that same collation library, so MacSort.js is a copy of a known algorithm over the very thing it uses, and is exact on a Mac. Microsoft publishes no source, and Windows' order comes from its own table and no library fuji has, so this is an imitation, right where it was made to be and checked from outside. The two share no code: their rules for numbers, for symbols and for ties differ in every particular, and each reads top to bottom on its own.

Since the order outside ASCII is the engine's, it can differ between fuji on Windows, whose engine carries its own collation, and fuji on a Mac, whose engine uses the system's, for the newest and rarest characters. ASCII is the same everywhere.
*/

const windowsSymbols = " !\"#$%&()*,./:;?@[\\]^_`{|}~+<=>"//the symbols of ASCII in the order Windows puts them, the space first, all of them before any digit. The apostrophe and the hyphen are not among them, being set aside
const windowsWords = "'-‐‑‒–—―‧⁃"//the characters Windows sets aside from the first comparison and breaks ties with, in the order it weighs them: the apostrophe, the hyphen, then Unicode's hyphen, its non-breaking hyphen, the figure dash, the en dash, the em dash, the horizontal bar, the hyphenation point and the hyphen bullet
const digitsMost = 19//the most digits in one number
const classSymbol = 0, classOther = 1, classNumber = 2, classLetter = 3, classBeyond = 4//what a piece of a name is, in the order the kinds sort: a symbol of ASCII, any other symbol, a number, a letter, and a character beyond the first plane
const base = new Intl.Collator('en', {usage: 'sort', sensitivity: 'base'})//the engine's collation of English, which is CLDR's root, telling letters apart and not their accents or case
const accented = new Intl.Collator('en', {usage: 'sort', sensitivity: 'accent'})//and the same telling accents apart too
const skipped = /^[\p{M}\p{Cf}\p{Cc}]$/u//a combining mark left over, which the accents are compared by, and the characters with no weight at all, a soft hyphen or a zero width joiner
const cacheMost = 100000//names whose reading is kept before the cache starts over, more than any folder fuji has listed
let cache = new Map()//name to its reading, so a sort reads each name once however many comparisons the name is in

export default function windowsCompare(a, b) { return windowsCompareNames(a.name, b.name) }//which of two entries comes first by name, as Explorer orders them

export function windowsCompareNames(a, b) {//the comparison on two names
	if (a == b) return 0
	let x = _windowsCached(a), y = _windowsCached(b)
	let n = Math.min(x.pieces.length, y.pieces.length)
	for (let i = 0; i < n; i++) {//first by the pieces, one at a time, the first difference deciding
		let p = x.pieces[i], q = y.pieces[i]
		if (p[0] != q[0]) return p[0] - q[0]//different kinds, in the kinds' own order
		if (p[1] === q[1]) continue
		if (p[0] == classSymbol || p[0] == classBeyond) return p[1] - q[1]//by place in windowsSymbols, or by the character's own number
		if (p[0] == classNumber) return p[1] < q[1] ? -1 : 1
		if (p[2] && q[2]) return p[1] < q[1] ? -1 : 1//two letters of ASCII, already lowered, where the collation would answer the same
		let c = base.compare(p[1], q[1])
		if (c) return c
	}
	if (x.pieces.length != y.pieces.length) return x.pieces.length - y.pieces.length//a name before a longer one that starts with it
	let c = accented.compare(x.plain, y.plain)//then by accents, which only names outside ASCII have
	if (c) return c
	if (x.words != y.words) return x.words < y.words ? -1 : 1//then by the hyphens and apostrophes set aside
	let s = a.toUpperCase(), t = b.toUpperCase()//two different names still the same, like ss and ß: by their characters with case ignored, as Explorer was seen to order them
	if (s != t) return s < t ? -1 : 1
	return a < b ? -1 : 1//and names that differ only by case as they are, so no two different names ever tie
}

function _windowsCached(name) {
	let reading = cache.get(name)
	if (!reading) {
		if (cache.size >= cacheMost) cache = new Map()
		reading = _windowsRead(name)
		cache.set(name, reading)
	}
	return reading
}

function _windowsRead(name) {//a name as the comparison wants it: its pieces, each a kind and what to compare it by; the name without the characters set aside, for the accents; and those characters as text that sorts the way Windows sorts them
	let points = [...name.normalize('NFC')], n = points.length//by whole characters, in composed form, so e and a combining acute are é
	let pieces = [], plain = '', words = ''
	for (let i = 0; i < n; i++) {
		let c = points[i], code = c.codePointAt(0)
		let word = windowsWords.indexOf(c)
		if (word >= 0) {//set aside: how far into the name it stood, counted backward so the later one sorts first, and then which it is
			words += String.fromCharCode((-pieces.length - 1) & 0xffff, word)
			continue
		}
		plain += c
		if (code >= 0x30 && code <= 0x39) {//a digit of ASCII, and the digits after it up to the most a number holds
			let length = 1
			while (length < digitsMost && i + length < n && points[i + length] >= '0' && points[i + length] <= '9') { plain += points[i + length]; length++ }
			let leading = 0
			while (leading < length && points[i + leading] == '0') leading++
			pieces.push([classNumber, String.fromCharCode(length - leading) + points.slice(i + leading, i + length).join('') + String.fromCharCode(0xfe - leading)])//how many digits count, so a longer number is a larger one before any digit is looked at; then the digits; then a character that is smaller the more zeros led
			i += length - 1
		} else if (code < 0x80) {
			let symbol = windowsSymbols.indexOf(c)
			if (symbol >= 0) pieces.push([classSymbol, symbol])
			else if ((code >= 0x41 && code <= 0x5a) || (code >= 0x61 && code <= 0x7a)) pieces.push([classLetter, c.toLowerCase(), true])//a letter of ASCII, lowered, and marked so two of them compare without the collation
		} else if (code > 0xffff) pieces.push([classBeyond, code])//an emoji or another character beyond the first plane, after every letter, where Windows puts one
		else if (skipped.test(c)) continue
		else pieces.push([base.compare(c, '0') < 0 ? classOther : classLetter, c, false])//a symbol where the engine's collation puts it before every digit, and with the letters where it does not, as it does a trademark sign, which sorts as TM
	}
	return {pieces, plain, words}
}
