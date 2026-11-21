/*
The order the Mac's Finder puts names in, on any platform, so a folder arranged in Finder reads the same in fuji.

Finder sorts as Foundation's localizedStandardCompare does, as Apple says and as Finder's own window shows on the test's folders. That method is CoreFoundation's string comparison under four options: ignore case, numbers by value, ignore width, and never call two different names equal. CoreFoundation is open source, and this file follows what it does there, in CFStringCompareWithOptionsAndLocale, rather than the summary Apple gives of it. It is two things in turn:

A walk of its own down both names together, for as long as they are the same. Where both reach a run of ASCII digits it reads the two numbers itself: different values decide at once, and the same value written at different lengths, 1 and 01, is remembered with the shorter first, to be used only if nothing after tells the names apart. The values are kept in 64 bits, so a number past 18,446,744,073,709,551,615 wraps around, which is followed here because it is what the Mac does. Two characters that differ and yet fold to the same thing, a capital and its small letter outside ASCII, a fullwidth letter and a plain one, é as one character and as e with a combining accent, are remembered the same way and walked past.

Then, at the first true difference, the Unicode collation ICU carries for the user's language, over what is left of both names: symbols in the order CLDR gives them and ahead of digits, digits ahead of letters, accents second, small letters ahead of capitals third, and width and the like last, since CoreFoundation turns ICU's case level on. The page's own Intl.Collator is the same ICU on the Mac, the system's own, so there the order is Finder's for every character. Away from the Mac it is the web engine's ICU, which agrees for ASCII and for Latin with accents in every version looked at, 66 through 78, and can differ for Chinese characters and for characters newer than one side knows.

Where this cannot match what the Mac does. The language is English here, where Finder follows the user's first language: Swedish puts å after z, and this does not. And the folding here uses the language's own case mapping and normal forms in place of CoreFoundation's tables, which agree nearly everywhere; where they part, the walk stops at a different place, and two names can come out the other way round.
*/

const collators = ['accent', 'case', 'variant'].map(sensitivity => new Intl.Collator('en', {usage: 'sort', sensitivity, numeric: true, ignorePunctuation: false}))//the collation of English, which is CLDR's root, with a run of digits one number, asked in turn about letters and accents, then case, then the rest, which is ICU's order with the case level on

export default function macCompare(a, b) { return macCompareNames(a.name, b.name) }//which of two entries comes first by name, as Finder orders them

export function macCompareNames(a, b) {//the comparison on two names, following CFStringCompareWithOptionsAndLocale under the options localizedStandardCompare passes
	let i = 0, j = 0//where the walk is in each name
	let startI = 0, startJ = 0//where the characters being compared began, which is behind i or j only while one of them is being read from its folded form
	let folded1 = '', at1 = 0, used1 = 1, folded2 = '', at2 = 0, used2 = 1//a character's folded form while the walk reads from it, how far in, and how much of the name it stands for
	let result = 0, numeric = false, forcedI = 0, forcedJ = 0//the first difference that was not a true one, whether it was a number's length, and where it was
	while (i < a.length && j < b.length) {
		let c, d
		if (!folded1) { c = a.charCodeAt(i); if (result && c >= 65 && c <= 90) c += 32; used1 = 1 } else c = folded1.charCodeAt(at1++)//once something is remembered, ASCII's capitals are read as small letters, and until then a capital against its small letter is a difference that goes to ICU
		if (!folded2) { d = b.charCodeAt(j); if (result && d >= 65 && d <= 90) d += 32; used2 = 1 } else d = folded2.charCodeAt(at2++)

		if (!folded1 && !folded2 && c >= 48 && c <= 57 && d >= 48 && d <= 57) {//both at an ASCII digit: two numbers
			let fromI = i, fromJ = j
			do i++; while (_macDigit(a, i))
			do j++; while (_macDigit(b, j))
			let v = _macNumber(a, fromI, i), w = _macNumber(b, fromJ, j)
			if (v != w) return v < w ? -1 : 1
			if (!result && i - fromI != j - fromJ) { result = i - fromI < j - fromJ ? -1 : 1; numeric = true; forcedI = fromI; forcedJ = fromJ }//the same number at two lengths, the shorter first, if it comes to that
			continue
		}

		if (c != d) {
			if (!result) { result = c < d ? -1 : 1; forcedI = startI; forcedJ = startJ }
			if (c < 0x80 && d < 0x80) return _macRest(a, i, b, j)//two different ASCII characters
			if (!folded1) {
				let f = _macFold(a, i, c)
				if (f) { folded1 = f.text; c = folded1.charCodeAt(0); at1 = 1; used1 = f.used }
			}
			if (!folded1 && folded2) return _macRest(a, startI, b, startJ)
			if (!folded2 && (!folded1 || c != d)) {
				let f = _macFold(b, j, d)
				if (f) { folded2 = f.text; d = folded2.charCodeAt(0); at2 = 1; used2 = f.used }
				if (!folded2 || c != d) return _macRest(a, startI, b, startJ)
			}
			if (folded1 && folded2) {
				while (at1 < folded1.length && at2 < folded2.length && folded1.charCodeAt(at1) == folded2.charCodeAt(at2)) { at1++; at2++ }
				if (at1 < folded1.length && at2 < folded2.length) return _macRest(a, startI, b, startJ)
			}
		}

		if (folded1 && at1 == folded1.length) folded1 = ''
		if (folded2 && at2 == folded2.length) folded2 = ''
		if (!folded1) i += used1
		if (!folded2) j += used2
		if (!folded1 && !folded2) { startI = i; startJ = j }
	}
	if (!numeric && result && i == a.length && j == b.length) result = _macRest(a, forcedI, b, forcedJ)//the names fold to the same thing, so ICU orders them from where they first differed
	return i < a.length ? 1 : j < b.length ? -1 : result//a name sorts after the one it starts with
}

function _macDigit(s, at) { let c = s.charCodeAt(at); return c >= 48 && c <= 57 }
function _macNumber(s, from, to) {//the value of a run of digits as CoreFoundation holds it, in 64 bits, so a longer number wraps around
	if (to - from <= 15) return Number(s.slice(from, to))//short enough to be exact as a plain number
	return BigInt.asUintN(64, BigInt(s.slice(from, to)))
}

function _macFold(s, at, read) {//the character at a place and the marks combined onto it, folded so that case, width and the two ways of writing an accent all come out the same; false when it is one character that folding leaves alone. An ASCII character is folded as the walk read it, already small if it was a capital and something is remembered, which is what CoreFoundation folds
	let point = s.codePointAt(at)
	let length = point > 0xffff ? 2 : 1, end = at + length
	if (point < 0x80) point = read
	let head = String.fromCodePoint(point)
	while (end < s.length) {//the marks after it
		let next = String.fromCodePoint(s.codePointAt(end))
		if (!/\p{Grapheme_Extend}/u.test(next)) break
		end += next.length
	}
	let marks = end > at + length
	let text = point >= 0xff00 && point <= 0xffef ? head.normalize('NFKD') : head//a fullwidth or halfwidth form to the plain one
	text = (text + s.slice(at + length, end)).normalize('NFD')
	text = point < 0x80 || point == 0x131 ? text.toLowerCase() : text.toUpperCase().toLowerCase()//through capitals and back, so ß comes out ss as a full case folding has it; but not the dotless ı, which folding leaves alone and the trip would make an i
	if (!marks && text == head) return false
	return {text, used: end - at}
}

function _macBack(s, at) {//where ICU's comparison starts for a difference found at a place: back over the marks and punctuation before it to the letter or digit they follow, since a collation reads a character in the company it keeps
	if (at <= 0) return 0
	let p = at - 1
	while (p > 0) {
		let c = s.charCodeAt(p)
		if (c >= 0xdc00 && c <= 0xdfff && s.charCodeAt(p - 1) >= 0xd800 && s.charCodeAt(p - 1) <= 0xdbff) {
			let point = s.codePointAt(p - 1)
			if (point >= 0x20000 || !/[\p{M}\p{P}]/u.test(String.fromCodePoint(point))) break
			p -= 2
		} else {
			if (!/[\p{M}\p{P}]/u.test(s[p]) || (c >= 0x2e80 && c < 0xac00)) break
			p--
		}
	}
	return Math.max(p, 0)
}

function _macRest(a, i, b, j) {//ICU's order for what is left of both names, and the characters' own numbers when ICU calls them the same
	let x = a.slice(_macBack(a, i)), y = b.slice(_macBack(b, j))
	return collators[0].compare(x, y) || collators[1].compare(x, y) || collators[2].compare(x, y) || (x < y ? -1 : x > y ? 1 : 0)
}
