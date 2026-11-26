//the names the platform sorts are tested with: small groups, each showing one thing Explorer or Finder does with a name, which test.js makes into a folder of empty files to look at in the shell itself and runs through the platform's own comparison to keep what it said. A group is a title, a line on what to look for, and its names; twins are names the group also holds that most volumes will not keep beside another of its names, since they differ from it only by case or by how an accent is written, so they are compared and never made into files; loose marks a group where Windows' own comparison contradicts itself, so no order of it can be called Explorer's, and names its characters that do it

const u = (...codes) => String.fromCodePoint(...codes)//a character by its number, for the ones an editor shows as nothing or as something else

export const sortGroups = [
	{
		title: 'numbers',
		shows: 'a run of digits is a number, and the two shells part ways on leading zeros: Explorer puts 001 before 01 before 1, Finder 1 before 01 before 001',
		names: ['0', '00', '1', '01', '001', '2', '02', '9', '10', '010', '20', '100'],
	},
	{
		title: 'numbers in a name',
		shows: 'the leading zeros of file01 outrank the letter after them in Explorer, file01b before file1a, and only break a tie in Finder, file1a before file01b',
		names: ['file1.txt', 'file01.txt', 'file001.txt', 'file2.txt', 'file02.txt', 'file9.txt', 'file10.txt', 'file010.txt', 'file1a.txt', 'file01a.txt', 'file1b.txt', 'file01b.txt', 'file.txt', 'file100.txt'],
	},
	{
		title: 'two numbers',
		shows: 'which run of zeros decides when a name has two: the first, in both shells',
		names: ['a1b1', 'a1b01', 'a01b1', 'a01b01', 'a1b2', 'a01b2', 'a1b02', 'a2b1', 'a02b1', 'a1c1'],
	},
	{
		title: 'long numbers',
		shows: 'Explorer reads a number 19 digits at a time and Finder keeps one in 64 bits, so past those the order stops being by value, in different places',
		names: ['x1', 'x2', 'x999999999999999999', 'x1000000000000000000', 'x9999999999999999999', 'x10000000000000000000', 'x18446744073709551615', 'x18446744073709551616', 'x18446744073709551617', 'x99999999999999999999', 'x100000000000000000000', 'x00000000000000000001', 'x0000000000000000000000002'],
	},
	{
		title: 'versions and dates',
		shows: 'a period, a comma and a hyphen end a number, so 1.10 is after 1.9 and 1,000 is before 999; nothing is a decimal or a minus sign',
		names: ['v1.9', 'v1.10', 'v1.2.9', 'v1.2.10', 'v1.02.3', '1.5', '1.10', '1.05', '999', '1,000', '-1', '-2', '-10', '2026-1-5', '2026-01-05', '2026-10-01', '2026-1-10', '0x9', '0x10', '1e5', '1e10'],
	},
	{
		title: 'case',
		shows: 'case is ignored until nothing else differs',
		names: ['apple', 'Banana', 'cherry', 'Date', 'eel', 'Fig', 'IMG5.jpg', 'img5.png', 'img12.jpg', 'Img100.jpg', 'ab1', 'AB2', 'Ab3', 'aB4'],
		twins: ['Apple', 'APPLE', 'banana', 'ab', 'Ab', 'aB', 'AB', 'a', 'A', 'a1', 'A1', 'a01', 'A01', 'a1B', 'a01b'],
	},
	{
		title: 'symbols first',
		shows: 'where each symbol sorts at the start of a name: all before digits and digits before letters in both shells, in two different orders among themselves, and Explorer sets the hyphen and the apostrophe aside as if they were not there',
		names: [' x', '!x', '#x', '$x', '%x', '&x', "'x", '(x', ')x', '+x', ',x', '-x', '.x', ';x', '=x', '@x', '[x', ']x', '^x', '_x', '`x', '{x', '}x', '~x', '1x', 'x', 'xx', 'ax'],
	},
	{
		title: 'symbols inside',
		shows: 'the same inside a name',
		names: ['a b', 'a!b', 'a#b', 'a$b', 'a%b', 'a&b', "a'b", 'a(b', 'a)b', 'a+b', 'a,b', 'a-b', 'a.b', 'a;b', 'a=b', 'a@b', 'a[b', 'a]b', 'a^b', 'a_b', 'a`b', 'a{b', 'a}b', 'a~b', 'a1b', 'ab', 'aab', 'a'],
	},
	{
		title: 'hyphen and apostrophe',
		shows: "Explorer's word sort: co-op sorts beside coop rather than with co op, and the hyphen only breaks a tie; Finder treats both as ordinary symbols",
		names: ['coop', 'co-op', "co'op", 'co_op', 'co op', 'cooq', 'co-oq', 'coop1', 'co-op1', 'coo', 'ab', 'a-b', 'ab-', '-ab', 'abc', 'a-c', 'a--b', "it's", 'its', 'itsy', 'a1', 'a-1', 'a2', 'a-2', 'a10', 'a-10', '1-2', '1-10', '2-1', '12', '1 2'],
	},
	{
		title: 'copies and extensions',
		shows: 'the period before an extension is one more symbol, so File.txt lands among its own copies; look at this one with extensions hidden too',
		names: ['File', 'File.txt', 'File (1).txt', 'File (2).txt', 'File (10).txt', 'File 1.txt', 'File_1.txt', 'File-1.txt', 'File1.txt', 'File2.txt', 'File.tx', 'File.txt.bak', 'File copy.txt', 'File copy 2.txt', 'File - Copy.txt', 'File - Copy (2).txt', 'Filea.txt', 'File..txt', 'a.tar', 'a.tar.gz', 'a.txt', 'a.txt2', 'a.txt10', 'a 1.txt', 'a.1.txt'],
	},
	{
		title: 'accents',
		shows: 'an accent counts only after the letters themselves, so éa is before ez; Explorer calls ß and ss the same and æ and ae the same, Finder puts the single character after',
		names: ['e', 'é', 'è', 'ê', 'ë', 'ē', 'f', 'ea', 'éa', 'ez', 'éz', 'resume', 'résumé', 'resumé', 'resumes', 'o', 'ö', 'ø', 'oe', 'œ', 'of', 'a', 'å', 'ä', 'ae', 'æ', 'af', 'ss', 'ß', 'st', 'sz', 'n', 'ñ', 'na', 'ña', 'nz', 'th', 'þ', 'ti', 'z', 'ž', 'd', 'đ', 'ð', 'l', 'ł', 'lz'],
		twins: ['e' + u(0x301), 'e' + u(0x301) + 'a', 'a' + u(0x30a), 'n' + u(0x303) + 'a', 'É', 'Résumé'],
	},
	{
		title: 'width',
		shows: 'a fullwidth letter is its plain letter until nothing else differs, and a fullwidth digit is a digit of the number',
		names: ['a', u(0xff41), 'b', u(0xff42), 'ab', u(0xff41, 0xff42), '1', u(0xff11), '2', '9', u(0xff19), '10', u(0xff11, 0xff10), '11', 'x9', 'x' + u(0xff11, 0xff10), 'x11'],
	},
	{
		title: 'other digits',
		shows: "digits of other scripts are numbers too, and a superscript or a fraction is not; in this group Explorer's own comparison contradicts itself, 10 before ² before 3 while 3 is before 10, so its order here is whatever its sort happens to leave",
		loose: [u(0xb9), u(0xb2), u(0xb3), u(0xbd), u(0x2460), u(0x2461)],
		names: ['1', '2', '3', '9', '10', u(0x661), u(0x662), u(0x669), u(0x661, 0x660), u(0xb9), u(0xb2), u(0xb3), u(0xbd), u(0x2460), u(0x2461), u(0x967), u(0x968), u(0x967, 0x966)],
	},
	{
		title: 'kana',
		shows: 'hiragana, katakana and halfwidth katakana are one letter three ways, told apart last, in a different order in each shell; a voiced kana follows its plain one, and the long mark takes the vowel before it',
		names: [u(0x3042), u(0x30a2), u(0xff71), u(0x3044), u(0x30a4), u(0x304b), u(0x304c), u(0x30ab), u(0x30ac), u(0xff76), u(0xff76, 0xff9e), u(0x304d, 0x3083), u(0x304d, 0x3084), u(0x30a2, 0x30fc), u(0x30a2, 0x30a2), u(0x3042, 0x3042), u(0x30ab, 0x30fc), u(0x30ab, 0x30a2), u(0x3053, 0x3053), u(0x3053, 0x309d), u(0x3093)],
		twins: [u(0x304b, 0x3099), u(0x30ab, 0x3099)],
	},
	{
		title: 'scripts',
		shows: 'the order of whole scripts: symbols, digits, Latin, Greek, Cyrillic and on, where the shells agree at first and then go their own ways, most of all on where emoji and Chinese characters fall',
		names: ['$', u(0x20ac), u(0xa9), u(0x2122), u(0x2665), '1', 'a', 'z', u(0x3b1), u(0x3c9), u(0x430), u(0x44f), u(0x5d0), u(0x627), u(0x915), u(0xe01), u(0x10d0), u(0x3042), u(0x30a2), u(0x4e00), u(0x4e2d), u(0xac00), u(0xd55c), u(0x1f600), u(0x1f389), u(0x20000)],
	},
	{
		title: 'spaces and what cannot be seen',
		shows: 'kinds of space, and characters with no width: Explorer ignores a soft hyphen and a zero width joiner entirely and not a zero width space, and Finder ignores all three',
		names: ['ab', 'a b', 'a  b', 'a' + u(0xa0) + 'b', 'a' + u(0x3000) + 'b', 'a' + u(0x200b) + 'b', 'a' + u(0xad) + 'b', 'a' + u(0x200d) + 'b', 'a' + u(0x2009) + 'b', 'aa', 'ac', 'a'],
	},
	{
		title: 'dashes',
		shows: 'the hyphen on the keyboard against the dashes that look like it: Explorer sets the hyphen aside and the Unicode hyphens with it, and Finder gives each a place of its own',
		names: ['ab', 'a-b', 'a' + u(0x2010) + 'b', 'a' + u(0x2011) + 'b', 'a' + u(0x2013) + 'b', 'a' + u(0x2014) + 'b', 'a' + u(0x2212) + 'b', 'a_b', 'a b', 'ac', 'aa', 'a' + u(0x2019) + 'b', "a'b"],
	},
	{
		title: 'emoji',
		shows: 'emoji and the characters that modify them; Finder files them with symbols, ahead of digits, and Explorer after every alphabet',
		names: [u(0x1f600), u(0x1f601), u(0x1f44d), u(0x1f44d, 0x1f3fd), u(0x1f468, 0x200d, 0x1f469, 0x200d, 0x1f467), u(0x2764), u(0x2764, 0xfe0f), u(0x1f1fa, 0x1f1f8), u(0x1f1ef, 0x1f1f5), u(0x263a), 'a', 'z', '1', '_', u(0x1f600) + '1', u(0x1f600) + '2', u(0x1f600) + '10', 'a' + u(0x1f600), 'ab'],
	},
	{
		title: 'korean',
		shows: 'Hangul syllables in their own order',
		names: [u(0xac00), u(0xac01), u(0xac1c), u(0xb098), u(0xb2e4), u(0xd558), u(0xd55c), u(0xd7a3), u(0xd55c, 0xae00), u(0xd55c, 0xad6d), u(0x3131), u(0x314f), 'a', '1'],
		twins: [u(0xd55c).normalize('NFD'), u(0xac00).normalize('NFD'), u(0xd55c, 0xae00).normalize('NFD')],
	},
	{
		title: 'chinese characters',
		shows: "Explorer orders them by a table of Microsoft's own and Finder by radical and stroke, and off the Mac fuji's Finder order falls back to the characters' numbers",
		names: [u(0x4e00), u(0x4e01), u(0x4e03), u(0x4e07), u(0x4e09), u(0x4e0a), u(0x4e0b), u(0x4e2d), u(0x4eba), u(0x65e5), u(0x672c), u(0x8a9e), u(0x3400), u(0x3401), u(0x9fa5), u(0x3005), u(0x4e00, 0x3005), u(0x20000), u(0x2a6d6), u(0x65e5, 0x672c), u(0x65e5, 0x672c, 0x8a9e), 'a', '1'],
	},
	{
		title: 'a real folder',
		shows: 'the names a collection actually holds',
		names: ['IMG_0001.JPG', 'IMG_0002.JPG', 'IMG_0010.JPG', 'IMG_0100.JPG', 'IMG_1.JPG', 'DSC00012.jpg', 'DSC00102.jpg', 'Screenshot 2026-11-20 at 9.41.05 AM.png', 'Screenshot 2026-11-20 at 10.02.11 AM.png', 'Screenshot 2026-11-20 at 9.41.05 PM.png', 'Screenshot (1).png', 'Screenshot (2).png', 'Screenshot (10).png', 'Screenshot.png', 'page1.png', 'page01.png', 'page02.png', 'page2.png', 'page10.png', 'page 1.png', 'page-1.png', 'page_1.png', 'Chapter 1 - Intro.cbz', 'Chapter 2 - Middle.cbz', 'Chapter 10 - End.cbz', 'Chapter 1.5 - Aside.cbz', '001.jpg', '01.jpg', '1.jpg', '0001.jpg', '2.jpg', '10.jpg', 'cover.jpg', 'Cover (back).jpg', '_notes.txt', '.hidden', 'Thumbs.db', 'Zoë.jpg', 'zoe.jpg', 'Zoe 2.jpg', 'café.jpg', 'cafe.jpg', 'cafes.jpg', 'naïve.txt', 'naive.txt', 'Ångström.txt', 'angstrom.txt', 'Österreich.jpg', 'osterreich.jpg', 'Übung.txt', 'ubung.txt'],
	},
]

export function sortNames() {//every name in every group, once each, which the platform's comparison is asked about pair by pair
	let all = new Set()
	for (let group of sortGroups) for (let name of [...group.names, ...(group.twins || [])]) all.add(name)
	return [...all]
}
export function sortLoose() {//the names Windows contradicts itself about, so a disagreement over one of them is counted apart
	let loose = new Set()
	for (let group of sortGroups) for (let name of group.loose || []) loose.add(name)
	return loose
}
