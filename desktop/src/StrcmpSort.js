/*
The plainest order fuji has, and the only one a person can work out by hand: names compared a character at a time by the characters' own numbers, the lower first, and a name before a longer one that starts with it. Computing calls it lexicographic order, and C's strcmp is its oldest famous name, which this takes; the settings panel calls it Simple lexicographic.

Two standards decide it, one inside the other. The comparison is ECMAScript's: the language says < on two strings compares them as sequences of UTF-16 code units, with no language, no locale and nothing to choose. The numbers are Unicode's, and for the characters on an English keyboard they are ASCII's, the table of the 1960s that put the digits at 48, the capitals at 65 and the small letters at 97. So every digit comes before every capital and every capital before every small letter, Zebra before apple; a digit is only a character, page10 before page2; and an accented letter, which Unicode numbered after all of ASCII, comes after z.

It has no opinion, which is what it is for: the order to reach for when a cleverer one has done something surprising and the user wants to see the folder plainly. The other sorts are collations, tables of weights made for people and laid over these numbers.

One thing keeps it from being exactly the order of Unicode's numbers. A character past the first 65,536, most emoji among them, is two code units in UTF-16, the first between 55,296 and 56,319, so it sorts before the few characters numbered above that, like a fullwidth letter. Rust compares by the bytes of UTF-8, which has no such fold, so the walk in steel and this can order those two kinds of name differently.
*/

export default function strcmpCompare(a, b) {//which of two entries comes first by name
	return a.name < b.name ? -1 : a.name > b.name ? 1 : 0//< compares text by its UTF-16 code units, which is the order a bare sort() gives
}
