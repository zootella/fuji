#![forbid(unsafe_code)]//the whole crate is safe rust, checked by the compiler and refused if a line ever says otherwise; the essay says what that promise covers, and the rules below add what it does not
#![deny(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects)]//the rules the essay states, held by clippy wherever it runs: no index that could be out of range, no unwrap, no panic, no arithmetic that could overflow; rustc leaves lints in clippy's namespace alone, so this costs an ordinary build nothing

use serde::Serialize;
use std::cell::Cell;

/*
Scout: fuji's own reader of the first bytes of a file, a crate of its own so it builds, tests and fuzzes with nothing of fuji around it. Given the tip of a file, its first 64 KiB or the whole of a smaller one, and the file's size, scout_bytes says what the bytes are and what the common formats keep in their first bytes: the kind of medium, the stored size of a picture or a video and the size to show it at, both with a phone's quarter turn applied, the running time of a sound or a video, whether a picture is animated, whether the file breaks its format's rules or ends before the part its structure points to, and, for every file, whether its bytes read as text or as binary. When the tip shows where the rest of an answer lies, scout_bytes hands back the offset of one more slice, 64 KiB there, beside its answer, and scout_slices answers again from both. It reads no frame, no pixel and no sample. The file's name and fuji's table of types play no part: the bytes name themselves, which is also why no name can steer the reader.

Why fuji writes this itself. Every parser that reads a collection's files reads bytes nobody vouched for, and the history fuji's security page tells is of decoders written in C and C++ made to run code by a file: GDI+'s JPEG, the Windows Metafile engine, ImageIO's JBIG2, libwebp, GStreamer's music decoders, FFmpeg's MagicYUV. A scan asks a question of every file on a drive, so it reaches more hostile files than any click, and the code it runs on them has to be code a file cannot subvert. Safe Rust is that code: the compiler refuses a read or a write outside the memory the program owns, which is the whole class of mistake those exploits were, and the first line of this file forbids unsafe code, so the compiler holds that promise for every line below.

The compiler does not promise the rest, and five rules do. The tip and the second slice are buffers fuji sized, one constant for both, and no number from the file sizes an allocation; the reader allocates nothing at all. Every read goes through one helper, Bytes, which holds the tip and the second slice as two windows onto the file and answers nothing outside them, so a length or an offset in the file can point anywhere and the worst it does is end the reading, or ask once for the slice that holds it. Every sum of offsets is a checked operation, since a release build wraps silently and a wrapped sum can look valid. Every loop moves forward and runs under a ceiling of steps as well as the slices' ends, so the work is bounded by 128 KiB whatever the file holds. And nothing here can panic: no indexing, no unwrap, no division before a check for zero; the lints on the second line hold those where clippy runs, and the tests below and the fuzzer in the fuzz folder hold them where it does not. Under those rules a hostile file gets one of two outcomes, no answer or a wrong one, and a wrong size or running time lasts until a decoder reads the file behind its own wall, the memory ceiling in thumbnail.rs among them, which stands whatever this reader said.

Why 64 KiB, and why a second slice. Most formats keep their answers in the first few hundred bytes. The exception that sets the size is JPEG, whose frame header comes after every application segment, and a phone's EXIF segment carries an embedded thumbnail of five to thirty kilobytes, a camera's up to sixty. Reading 64 KiB after a seek costs a fraction of a millisecond on a spinning disk against the ten the seek costs, so a night's work is the same at 16 KiB, and the tip is sized for the answer. Some answers are not in the tip at all, and the tip says where they are: a phone's MP4 keeps its index after the data box, whose size the box states; an MP3 behind a tag with cover art keeps its first frame where the tag's header says the tag ends; a TIFF's header points at its first directory, which libtiff writes last; and an Ogg, a Flash video without metadata and a DVD's program stream keep their running time in their last page, tag or pack, so in the file's last 64 KiB. For those the first answer comes with an offset, the command reads one more 64 KiB slice there, and the reader runs again over both. That is the last read: never a third, and never a read whose place would have to be found by searching the file, so a file costs one seek or two. The answer's reading says how far into the tip a file's answers reached and whether the second slice was needed, so a run over a collection says what a smaller tip would have missed.

What the reader does not try is chosen as carefully as what it does, and each reader's comment says where it stops. A HEIF picture's size sits behind three more boxes; a transport stream could be timed only by reading its packets; an MP4 whose sound track and its tables come before the picture track can keep that track past the second slice. Each of those is a limit the answer's reading names in a few words, and the limits are the map of the horizon: what a decoder would add. Every stop is one of four kinds, and the answer says which beside the words: a format named by its signature and read no further, the scout's own limit, the file corrupted, and the file chopped. The answer is numbers and this file's own constants, so no text from the file ever reaches the page.

Where it runs. Today the command in fuji's scout.rs makes the two reads in fuji's own process on the blocking pool, which is safe because the reader is; the overnight scan will run the same function in a sandboxed process of fuji's own, where a hang on a dead disk can be killed, and nothing about the reader changes for that.
*/

pub const TIP_BYTES: usize = 64 << 10;//how much of a file fuji's command reads, and the most scout_bytes is ever handed: 64 KiB, the one constant the essay argues
const STEPS: usize = 256;//the most steps any walk below takes through a file's segments, chunks, boxes or elements before it gives up; more than any real file has before its answers, and the ceiling that bounds a loop a hostile file would otherwise keep going

const PAST: &str = "not in the slices";//the limit every reader can meet: a read of bytes the file holds and the slices in hand do not; after the first read it is also the request for the second slice, at the offset read asks for
const SHORT: &str = "past the end of the file";//the other: a read past the file's size, which is a structure claiming more than the file holds, and the file chopped
const SIGNATURE_ONLY: &str = "signature only";//the details of a format the reader names and reads no further into
const IMAGE: &str = "image";//the three media, as the answer's medium says them
const VIDEO: &str = "video";
const AUDIO: &str = "audio";

type Step<T> = Result<T, Stop>;//what every read and every reader answers: the value, or why the reading stopped
#[derive(Debug)]
enum Stop {//why a reader stopped before it had every answer it reads for its format, in a few words of this file's own, which become the reading's details; the kind decides what else the answer says, in scout below
	Named(&'static str),//a format named by its signature and read no further into: not parsed, and no trouble
	Limit(&'static str),//the scout's own limit, in a structure or a variant it does not read, or an answer the file does not keep: trouble, and no finding about the file. A walk that reaches its ceiling of steps is one, since a ceiling is no evidence that the file breaks a rule
	Ask(&'static str, u64),//the scout's limit at an offset it knows its answer lies at and the bytes in hand do not reach: after the first read the request for the second slice there, and after the second a limit like any other
	Corrupted(&'static str),//the file breaks its format's rules where the reader needed them: trouble, and that finding
	Chopped(&'static str),//the file ends before the part its structure points to, or inside the structure the reader was walking: trouble, and that finding
}
impl Stop { fn words(&self) -> &'static str { match self { Stop::Named(w) | Stop::Limit(w) | Stop::Ask(w, _) | Stop::Corrupted(w) | Stop::Chopped(w) => w } } }

#[derive(Serialize, Debug, PartialEq)]
pub struct Tip {//what the first bytes of a file say: what the file holds at the top, and how the reading went in reading. Every field is set as far as the bytes allow and left at its 0, false or blank when they do not, and every key is one word, so the answer is spelled the same in rust and on the page
	pub format: &'static str,//the format the first bytes announce, as this file names it: jpeg, png, mp4, matroska, mp3 and so on; blank when they announce nothing the reader knows
	pub medium: &'static str,//image, video or audio; blank for a format that is none of the three, and for a container whose streams the reader did not reach
	pub text: bool,//the scout is very certain the tip is characters, judged from the counts in the reading alone and apart from every format's reader and signature, so a text file that opens with a signature's letters answers its format and this both; text_verdict says the rule
	pub binary: bool,//the scout is very certain the tip is not characters: it holds a zero byte and opens with no byte order mark. Never true with text, and false beside it for an empty file
	pub raster: Size,//the stored pixel count of a picture or a video, 0 when the slices do not say, with its turn already applied
	pub aspect: Size,//the size to show a picture or a video at, in square pixels: raster grown on one side where a header the scout reads says the pixels are not square, so a DVD stored at 720 by 480 and flagged 16:9 answers 853 by 480, the same as raster everywhere else, and turned with it; wherever the scout read the shape, mpv, fuji's player, draws the file at this size unzoomed
	pub orientation: u32,//the turn or flip the file asks for, in EXIF's numbering, 1 for none through 8, as a finding only: raster and aspect already have it applied, so a caller uses them as they are and never turns them again, which would swap them back. It says the file is one an older program, which ignores the instruction, would show turned
	pub seconds: f64,//the running time of a sound or a video; 0 when the slices do not say, and an estimate where the comment on the reader says so
	pub animated: bool,//true when the format's own flag says the picture is animated; false where it says still, and where the reader could not look
	pub corrupted: bool,//the reader found a format, then found the file breaking that format's rules where it needed them; a broken part it steps past without needing, a JPEG's EXIF or an AVI's OpenDML header, costs only what it held and is not reported. Like chopped, a reason an answer is missing or may be wrong, and never a verdict on the file
	pub chopped: bool,//the reader found a format, then found the file's end cut off: a length or an offset it met on the way to its answers points past the end, or the file ends inside the structure it was walking, which is most often why an answer is missing. It can come with a finished reading, where the reader meets the file's stated total on its way, as for a WAV whose data chunk claims more than the file holds, and then seconds is what the header promised. The scout never goes looking for a length its answers do not need, so false on this and on corrupted says only that it met no such evidence: the two say why an answer is missing or may be wrong, and never sort whole files from broken ones
	pub reading: Reading,//how the reading went, apart from what the file holds
	#[serde(skip)]
	pixel: (u64, u64),//the shape of one stored pixel as the file states it, width to height and unturned, which a reader records and sizes makes aspect from; 0 on either side when the file states none, which counts as square. Kept out of the answer, which says it as aspect
}
#[derive(Serialize, Debug, PartialEq, Clone, Copy)]
pub struct Size { pub width: u32, pub height: u32 }//a width and a height in pixels: as stored in raster, and square in aspect; 0 and 0 when the file did not say
#[derive(Serialize, Debug, PartialEq)]
pub struct Reading {//the scout's report on its own reading
	pub parsed: bool,//the scout has a reader for this format and ran it past the signature; false for a format it names by its signature alone, for bytes it does not recognize, and for an empty file. Says nothing about trouble: a JPEG cut off at 100 bytes is parsed, with trouble
	pub trouble: bool,//the reader stopped before it had every answer it reads for this format: its own limit, past its slices or in a structure it does not read, or the file at fault, which corrupted or chopped then names. Corrupted always comes with trouble; chopped can come without it
	pub details: &'static str,//a few words on the condition, why the reader stopped or why it did not parse, as this file's own constants; blank when it parsed with no trouble
	pub reads: u32,//how many reads the answer is from, 1 or 2
	pub depth: u32,//the farthest byte of the tip the reader looked at, signature probes included, for every file; with reads, what a run over a collection reads to learn whether a smaller first read would do and how often a second is needed
	pub counts: Counts,//the evidence behind text and binary
}
#[derive(Serialize, Debug, PartialEq)]
pub struct Counts {//how many of the tip's bytes are of each of four kinds, whatever the format; text_verdict reads them, and a run over a collection sets its thresholds by where the two humps fall
	pub nul: u32,//zero bytes
	pub control: u32,//control characters other than the whitespace and escape of text
	pub printable: u32,//printable ascii, with that whitespace and escape
	pub high: u32,//bytes past ascii, which utf-8 text and every binary file have
}

/// What the first bytes of a file say, from the tip alone and the file's size in bytes, and beside it the one request the reader makes: the offset of one more 64 KiB slice when the tip says where the rest of an answer lies, which scout_slices takes with the tip, or nothing. Pure: no path, no open, no allocation, and the same answer for the same bytes on every platform; the essay says what is answered and what is promised
pub fn scout_bytes(tip: &[u8], size: u64) -> (Tip, Option<u64>) { scout(tip, None, size) }

/// The answer from the tip and the second slice the first answer asked for, at the offset it named, which is where the slice begins in the file. The second read is the last: a reader that still cannot reach its answer stops and says why in the reading, and asks for nothing
pub fn scout_slices(tip: &[u8], slice: &[u8], at: u64, size: u64) -> Tip { scout(tip, Some((at, slice)), size).0 }

fn scout(tip: &[u8], second: Option<(u64, &[u8])>, size: u64) -> (Tip, Option<u64>) {//both entries: judge the bytes as text or binary, run the reader over them, turn where it stopped into the reading, and after the first read turn a stop at bytes it did not have into the request for the second, which comes back beside the answer and never in it
	let (slice_at, slice): (usize, &[u8]) = match second { Some((at, s)) => (usize::try_from(at).unwrap_or(usize::MAX), s), None => (0, &[]) };
	let b = Bytes { tip, slice, slice_at, size, depth: Cell::new(0) };
	let counts = text_counts(tip);
	let (text, binary) = text_verdict(tip, &counts);
	let none = Size { width: 0, height: 0 };
	let mut t = Tip { format: "", medium: "", text, binary, raster: none, aspect: none, orientation: 1, seconds: 0.0, animated: false, corrupted: false, chopped: false, reading: Reading { parsed: false, trouble: false, details: "", reads: if second.is_some() { 2 } else { 1 }, depth: 0, counts }, pixel: (0, 0) };
	let mut wish = None;
	if tip.is_empty() { t.reading.details = "empty" }
	else {
		match read_format(&b, size, &mut t) {
			None => if text && svg(&b) { t.format = "svg"; t.medium = IMAGE; t.reading.parsed = true },//no format: text that draws a picture is an svg, and the rest is text or binary as judged above
			Some(Ok(())) => t.reading.parsed = true,
			Some(Err(stop)) => {//the reader stopped: its words, and what else its kind says
				t.reading.details = stop.words();
				if !matches!(stop, Stop::Named(_)) { t.reading.parsed = true; t.reading.trouble = true }//every kind but a format named by its signature and read no further
				match stop {
					Stop::Ask(_, at) if at >= size => t.chopped = true,//a place at or past the file's end is no request: the file ends before the part its structure points to
					Stop::Ask(_, at) if second.is_none() => wish = Some(at),//the request for the second slice; after the second read, a limit like any other
					Stop::Corrupted(_) => t.corrupted = true,
					Stop::Chopped(_) => t.chopped = true,
					_ => {}
				}
			}
		}
	}
	sizes(&mut t);
	t.reading.depth = u32::try_from(b.depth.get()).unwrap_or(u32::MAX);//exact, since the tip is at most 64 KiB
	(t, wish)
}

fn sizes(t: &mut Tip) {//the one step that makes the answer's two sizes from what the readers recorded, stored and unturned: aspect is raster grown by the pixel's shape, and then both swap their sides together when the orientation turns the picture a quarter turn, EXIF's 5 through 8, the same four values thumbnail.rs swaps for, so the two can never disagree about the turn
	t.aspect = grow(t.raster, t.pixel);
	if (5..=8).contains(&t.orientation) {
		t.raster = Size { width: t.raster.height, height: t.raster.width };
		t.aspect = Size { width: t.aspect.height, height: t.aspect.width };
	}
}

fn grow(stored: Size, pixel: (u64, u64)) -> Size {//the size to show stored pixels of this shape at, by mpv's rule in mp_image_params_get_dsize: the side the pixel is long on grows by its shape, so neither side shrinks below the stored count, in whole numbers with the fraction dropped, which keeps the answer to the pixel of what fuji's player draws at full size. A shape the file did not state, a square one, or one that would grow a side past what the answer holds leaves the size as stored
	let (across, down) = pixel;
	let scale = |side: u32, long: u64, short: u64| -> Option<u32> { u32::try_from(u128::from(side).checked_mul(u128::from(long))?.checked_div(u128::from(short))?).ok() };
	if across > down && down > 0 { if let Some(width) = scale(stored.width, across, down) { return Size { width, height: stored.height } } }
	if down > across && across > 0 { if let Some(height) = scale(stored.height, down, across) { return Size { width: stored.width, height } } }
	stored
}

fn read_format(b: &Bytes, size: u64, t: &mut Tip) -> Option<Step<()>> {//the format the first bytes announce, and that format's reader run on them: none when no signature matches, and otherwise the reader's own step, which is Ok when it finished and a stop when it did not. A signature is checked where its format puts it, most at the first byte; the order matters only where two could match, and none here do
	Some(if b.has(0, &[0xFF, 0xD8, 0xFF]) { jpeg(b, t) }
	else if b.has(0, b"\x89PNG\r\n\x1A\n") { png(b, t) }
	else if b.has(0, b"GIF87a") || b.has(0, b"GIF89a") { gif(b, t) }
	else if b.has(0, b"BM") && matches!(b.le32(14), Ok(12 | 40 | 52 | 56 | 64 | 108 | 124)) { bmp(b, size, t) }//two letters are a weak signature, so the header's size has to be one the format defines too
	else if b.has(0, b"RIFF") { riff(b, size, t) }
	else if b.has(0, b"II*\0") || b.has(0, b"MM\0*") { tiff(b, t) }
	else if b.has(0, b"8BPS") { psd(b, t) }
	else if b.has(4, b"ftyp") || iso_bare(b, size) { iso(b, size, t) }
	else if b.has(0, &[0x1A, 0x45, 0xDF, 0xA3]) { ebml(b, t) }
	else if b.has(0, b"FLV\x01") { flv(b, t) }
	else if b.has(0, &ASF_HEADER) { asf(b, t) }
	else if b.has(0, b".RMF") { realmedia(b, t) }
	else if b.has(0, b"OggS") { ogg(b, t) }
	else if b.has(0, b"fLaC") { flac(b, 0, t) }
	else if b.has(0, b"ID3") || adts(b, 0) || mp3_header(b, 0).is_some_and(|head| mp3_header(b, head.length).is_some()) { mp3(b, size, t) }//a frame header is eleven bits of sync and a few fields, which two bytes of anything can pass, a UTF-16 text's byte order mark among them, so a file that starts with one is MPEG audio only when a second header sits where the first says its frame ends
	else if b.has(0, b".snd") { au(b, t) }
	else if b.has(0, b"FORM") && (b.has(8, b"AIFF") || b.has(8, b"AIFC")) { aiff(b, t) }
	else if b.has(0, &[0, 0, 1, 0xBA]) || b.has(0, &[0, 0, 1, 0xB3]) { mpeg(b, 0, t) }
	else if mpegts(b) { t.format = "mpegts"; t.medium = VIDEO; Err(Stop::Named(SIGNATURE_ONLY)) }
	else if let Some(&(_, format, medium)) = SIGNATURES.iter().find(|(bytes, _, _)| b.has(0, bytes)) { t.format = format; t.medium = medium; Err(Stop::Named(SIGNATURE_ONLY)) }
	else { return None })
}

const SIGNATURES: &[(&[u8], &str, &str)] = &[//formats the reader names from their first bytes and reads no further into: sound a collection holds whose running time is not in its header, and common files that are not media, so a scan can say so without handing them to any decoder
	(b"MAC ", "ape", AUDIO),
	(b"wvpk", "wavpack", AUDIO),
	(b"#!AMR", "amr", AUDIO),
	(b"Creative Voice File", "voc", AUDIO),
	(b"MThd", "midi", AUDIO),
	(&[0x0B, 0x77], "ac3", AUDIO),//two bytes, the weakest signature here: a binary file of another kind can open with them, and nothing in the answer says otherwise, which is acceptable because a caller reads the answer beside the file's name
	(b"FWS", "swf", ""),
	(b"CWS", "swf", ""),
	(b"PK\x03\x04", "zip", ""),
	(b"Rar!", "rar", ""),
	(b"7z\xBC\xAF\x27\x1C", "7z", ""),
	(&[0x1F, 0x8B], "gzip", ""),
	(b"%PDF", "pdf", ""),
	(b"MZ", "exe", ""),
	(b"SQLite format 3", "sqlite", ""),
];

//the bytes in hand, and the one way anything below reads them: the tip, and the second slice once it is read, each a window onto the file at its own offset, so every reader speaks in the file's own offsets and never learns which window answered
struct Bytes<'a> {
	tip: &'a [u8],//the file's first bytes
	slice: &'a [u8],//the second slice, beginning at slice_at; empty until the second read
	slice_at: usize,
	size: u64,//the file's size, which tells a read past the end of the file from one the slices merely do not hold
	depth: Cell<usize>,//the end of the farthest read the tip answered, for the reading's depth; a cell so the readers can share one Bytes without carrying it as mutable through every call
}
impl<'a> Bytes<'a> {
	fn held(&self, at: usize, n: usize) -> Option<&'a [u8]> {//n bytes at an offset when one window holds them whole, the tip first, counting a read of the tip toward the depth; none otherwise, which read turns into its stop and has into false
		let end = at.checked_add(n)?;
		if let Some(s) = self.tip.get(at..end) { self.depth.set(self.depth.get().max(end)); return Some(s) }
		let from = at.checked_sub(self.slice_at)?;
		self.slice.get(from..from.checked_add(n)?)
	}
	fn read(&self, at: usize, n: usize) -> Step<&'a [u8]> {//n bytes at an offset, or why there are none: the one read everything below is built on, and the line that makes the rule that nothing indexes past the end true of the whole file. Bytes the file holds and the windows do not are the request for the second slice, and bytes past the file's end are the file chopped
		if let Some(s) = self.held(at, n) { return Ok(s) }
		if add(at, n)? as u64 > self.size { return Err(Stop::Chopped(SHORT)) }
		Err(Stop::Ask(PAST, at as u64))
	}
	fn u8(&self, at: usize) -> Step<u8> { self.read(at, 1)?.first().copied().ok_or(Stop::Limit(PAST)) }
	fn be16(&self, at: usize) -> Step<u16> { Ok(u16::from_be_bytes(array(self.read(at, 2)?)?)) }
	fn le16(&self, at: usize) -> Step<u16> { Ok(u16::from_le_bytes(array(self.read(at, 2)?)?)) }
	fn be24(&self, at: usize) -> Step<u32> { let s = array::<3>(self.read(at, 3)?)?; Ok(u32::from_be_bytes([0, s[0], s[1], s[2]])) }
	fn le24(&self, at: usize) -> Step<u32> { let s = array::<3>(self.read(at, 3)?)?; Ok(u32::from_le_bytes([s[0], s[1], s[2], 0])) }
	fn be32(&self, at: usize) -> Step<u32> { Ok(u32::from_be_bytes(array(self.read(at, 4)?)?)) }
	fn le32(&self, at: usize) -> Step<u32> { Ok(u32::from_le_bytes(array(self.read(at, 4)?)?)) }
	fn be64(&self, at: usize) -> Step<u64> { Ok(u64::from_be_bytes(array(self.read(at, 8)?)?)) }
	fn le64(&self, at: usize) -> Step<u64> { Ok(u64::from_le_bytes(array(self.read(at, 8)?)?)) }
	fn four(&self, at: usize) -> Step<[u8; 4]> { array(self.read(at, 4)?) }//a four-letter name, as chunks and boxes carry
	fn has(&self, at: usize, bytes: &[u8]) -> bool { self.held(at, bytes.len()) == Some(bytes) }//whether these bytes sit at this offset; a probe, so it is never a reason to stop and never asks for a slice
	fn holds(&self, at: usize) -> bool { self.held(at, 1).is_some() }//whether a window holds the byte at an offset: what a reader checks before it asks for a slice there
	fn holds_end(&self) -> bool { self.tip.len() as u64 >= self.size || (self.slice_at as u64).saturating_add(self.slice.len() as u64) >= self.size }//whether a window holds the file's last byte, for the readers whose answer is at the end
	fn end_of(&self, at: usize) -> usize {//the end of the window that holds an offset, or the offset itself when none does: where a scan of the bytes from there stops
		if at < self.tip.len() { self.tip.len() } else if at >= self.slice_at && at.saturating_sub(self.slice_at) < self.slice.len() { self.slice_at.saturating_add(self.slice.len()) } else { at }
	}
	fn find(&self, from: usize, pattern: &[u8]) -> Step<Option<usize>> {//where a pattern first occurs at or after an offset in the bytes in hand, the tip and then the slice, or none: the bounded search of the formats that define a search as the way to find their header. Never a reason to stop and never a request, since a read whose place is found by searching is the read the scout does not make; a pattern is never empty, which windows would refuse
		for (window, start) in [(self.tip, 0), (self.slice, self.slice_at)] {
			let skip = from.saturating_sub(start).min(window.len());
			let Some(rest) = window.get(skip..) else { continue };
			if let Some(p) = rest.windows(pattern.len()).position(|w| w == pattern) {
				let at = add(start, add(skip, p)?)?;
				self.read(at, pattern.len())?;//for the depth
				return Ok(Some(at))
			}
		}
		Ok(None)
	}
	fn rfind(&self, before: usize, pattern: &[u8]) -> Step<Option<usize>> {//where a pattern last occurs ending at or before an offset, the slice and then the tip, or none: the backward search of the formats that keep their answer at the end of the file
		for (window, start) in [(self.slice, self.slice_at), (self.tip, 0)] {
			let limit = before.saturating_sub(start).min(window.len());
			let Some(head) = window.get(..limit) else { continue };
			if let Some(p) = head.windows(pattern.len()).rposition(|w| w == pattern) {
				let at = add(start, p)?;
				self.read(at, pattern.len())?;
				return Ok(Some(at))
			}
		}
		Ok(None)
	}
}
fn array<const N: usize>(s: &[u8]) -> Step<[u8; N]> { s.try_into().map_err(|_| Stop::Limit(PAST)) }//a slice as a fixed array, for the from_bytes constructors; never fails after read gave exactly N bytes
fn add(a: usize, b: usize) -> Step<usize> { a.checked_add(b).ok_or(Stop::Corrupted("an offset sum that overflows")) }//every sum of offsets below, checked; a wrapped sum is the arithmetic a hostile length aims for, and no file that keeps its format's rules has one
fn sub(a: usize, b: usize) -> Step<usize> { a.checked_sub(b).ok_or(Stop::Corrupted("an offset difference below zero")) }
fn mul(a: usize, b: usize) -> Step<usize> { a.checked_mul(b).ok_or(Stop::Corrupted("an offset product that overflows")) }
fn to_usize(n: u64) -> Step<usize> { usize::try_from(n).map_err(|_| Stop::Corrupted("a size larger than memory")) }//a 64-bit size from a file as an offset, refused rather than wrapped where it does not fit
fn seconds(value: f64) -> f64 { if value.is_finite() && value > 0.0 { value } else { 0.0 } }//every running time passes through here on its way into the answer, so a division by a zero the reader missed, or a float a file wrote, lands as unknown rather than as infinity or a NaN
fn ask(b: &Bytes, at: usize, reason: &'static str) -> Step<()> {//a reader's request for the second slice, where it knows the offset its answer lies at and the bytes in hand do not reach it: after the first read scout turns the stop into the request, with the reason as the details, and after the second read the same line finds the bytes in hand and the reader goes on. A reader that walks into bytes it does not have asks the same way through read, with the plain reason
	if b.holds(at) { Ok(()) } else { Err(Stop::Ask(reason, at as u64)) }
}
fn ask_end(b: &Bytes, reason: &'static str) -> Step<()> {//the same request for the last 64 KiB of the file, for the formats that keep their running time on their last page, tag or pack; the file's start when the file is shorter than that, which only a test or the fuzzer hands over, since the command never hands over a tip that falls short of such a file's end
	if b.holds_end() { Ok(()) } else { Err(Stop::Ask(reason, b.size.saturating_sub(TIP_BYTES as u64))) }
}

//text or binary, from the whole tip

fn text_counts(tip: &[u8]) -> Counts {//how many bytes are nul, control, printable and high, over the whole tip: the raw material of text_verdict, and of a run over a collection that sets its thresholds by where the two humps fall
	let mut c = Counts { nul: 0, control: 0, printable: 0, high: 0 };
	for &byte in tip {
		match byte {
			0 => c.nul = c.nul.saturating_add(1),
			0x09..=0x0D | 0x1B | 0x20..=0x7E => c.printable = c.printable.saturating_add(1),//tab, newline, vertical tab, form feed and return; escape, since ansi art is text; and printable ascii
			0x80..=0xFF => c.high = c.high.saturating_add(1),//utf-8 past ascii, or any byte of a binary file
			_ => c.control = c.control.saturating_add(1),//the other control characters, which text has none of
		}
	}
	c
}

fn text_verdict(tip: &[u8], c: &Counts) -> (bool, bool) {//whether the tip is very likely text, and whether it is very likely not, from the counts alone and apart from every format's reader and signature, so no format's pattern reaches either. A utf-16 byte order mark says text outright, since every other byte of such text is a nul; otherwise no nul at all, which is git's rule for a binary file, with control characters in no more than one byte in a hundred, is text, and a nul is binary. Never both, and neither for an empty tip or for bytes with no nul but too many control characters, which real files rarely are. The one in a hundred is a factory value; a run over a collection, with the four counts in the answer, is what sets it where the two humps part
	if tip.is_empty() { return (false, false) }
	if tip.starts_with(&[0xFF, 0xFE]) || tip.starts_with(&[0xFE, 0xFF]) { return (true, false) }
	if c.nul > 0 { return (false, true) }
	let total = c.control.saturating_add(c.printable).saturating_add(c.high);
	(c.control.saturating_mul(100) <= total, false)
}

fn svg(b: &Bytes) -> bool {//whether text is an svg: markup from its first byte, as xml begins, and svg the first element, after whatever declaration, comments and doctype come before it; a web page with a drawing inside it is a page, and prose that mentions the tag is prose
	let start = if b.has(0, &[0xEF, 0xBB, 0xBF]) { 3 } else { 0 };//past a utf-8 byte order mark
	let Ok(Some(mut at)) = b.find(start, b"<") else { return false };
	if !b.read(start, at.saturating_sub(start)).is_ok_and(|s| s.iter().all(u8::is_ascii_whitespace)) { return false }
	for _ in 0..STEPS {
		if b.has(at, b"<svg") { return true }
		let skip = if b.has(at, b"<!--") { b.find(at, b"-->") } else if b.has(at, b"<?") || b.has(at, b"<!") { add(at, 2).map(Some) } else { return false };//a comment whole, since it can hold anything; a declaration only past its opening, since a doctype can hold declarations of its own, as Illustrator's do
		let Ok(Some(next)) = skip.and_then(|from| match from { Some(from) => b.find(from, b"<"), None => Ok(None) }) else { return false };
		at = next;
	}
	false
}

//pictures

fn jpeg(b: &Bytes, t: &mut Tip) -> Step<()> {//a JPEG is segments, each a marker and a length, stepped through by their stated lengths to the start-of-frame segment that holds the size; never searched for, because the EXIF segment can hold a whole small JPEG, the thumbnail, with a frame header of its own. The orientation is a tag in that EXIF segment, read on the way past. A segment that runs past the tip, as a color profile's or a camera's can, is where the second slice begins
	t.format = "jpeg"; t.medium = IMAGE;
	let mut at = 2;//past the start-of-image marker
	for _ in 0..STEPS {
		if b.u8(at)? != 0xFF { return Err(Stop::Corrupted("no marker where the format puts one")) }
		at = add(at, 1)?;
		while b.u8(at)? == 0xFF { at = add(at, 1)? }//a run of 0xFF before a marker's code is fill the format allows; each step moves forward, so the tip's end stops it
		let code = b.u8(at)?;
		at = add(at, 1)?;
		match code {
			0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => {//a start-of-frame segment, one of the thirteen kinds, baseline and progressive among them: its length, then the sample precision, the height, the width and the component count
				let (height, width) = (b.be16(add(at, 3)?)?, b.be16(add(at, 5)?)?);
				if height == 0 { return Err(Stop::Limit("a height the file defines after its first scan")) }//the format allows a zero here, filled in by a marker after the scan, which no common writer uses
				if width == 0 { return Err(Stop::Corrupted("a zero width")) }
				t.raster = Size { width: width.into(), height: height.into() };
				return Ok(())
			}
			0x01 | 0xD0..=0xDA => return Err(Stop::Corrupted("no frame header before the picture's data")),//the markers with no segment, the start and end of the image, and the start of the scan: none comes before the frame header in a picture the reader understands
			_ => {
				let length = usize::from(b.be16(at)?);//the segment's length, which counts these two bytes
				if length < 2 { return Err(Stop::Corrupted("a segment shorter than its own length field")) }
				if code == 0xE1 && b.has(add(at, 2)?, b"Exif\0\0") {//the EXIF segment: a TIFF header and its first directory, where the orientation tag lives; a malformed one costs the orientation and nothing else
					if let Ok(ifd) = ifd_read(b, add(at, 8)?, add(at, length)?) { t.orientation = ifd.orientation }
				}
				at = add(at, length)?;
			}
		}
	}
	Err(Stop::Limit("more segments before the frame header than any picture has"))
}

struct Ifd { width: u32, height: u32, orientation: u32, reduced: bool, dng: bool }//what the reader takes from a TIFF structure's first directory: the stored size, the orientation, whether the first image is a reduced-size copy of another, and whether the file is a DNG
fn ifd_read(b: &Bytes, tiff: usize, end: usize) -> Step<Ifd> {//the first image file directory of the TIFF structure at tiff, read no further than end: a byte order, the number 42, the directory's offset, then an entry count and twelve-byte entries of tag, type, count and value. A JPEG's EXIF segment and a TIFF file are both this
	let little = match b.read(tiff, 2)? { b"II" => true, b"MM" => false, _ => return Err(Stop::Corrupted("not a TIFF header")) };
	if num16(b, add(tiff, 2)?, little)? != 42 { return Err(Stop::Corrupted("not a TIFF header")) }
	let directory = add(tiff, to_usize(num32(b, add(tiff, 4)?, little)?.into())?)?;
	let count = usize::from(num16(b, directory, little)?);
	let mut ifd = Ifd { width: 0, height: 0, orientation: 1, reduced: false, dng: false };
	for i in 0..count.min(STEPS) {
		let entry = add(add(directory, 2)?, mul(i, 12)?)?;
		if add(entry, 12)? > end { break }//a count that reaches past the structure
		let tag = num16(b, entry, little)?;
		if tag == 0xC612 { ifd.dng = true; continue }//DNGVersion, whose four bytes say which DNG and are not read
		let value = match num16(b, add(entry, 2)?, little)? { 3 => u32::from(num16(b, add(entry, 8)?, little)?), 4 => num32(b, add(entry, 8)?, little)?, _ => continue };//a short or a long, which is all these tags are; a value of either sits in the first bytes of the value field
		match tag {
			0x00FE => ifd.reduced = value & 1 == 1,//NewSubfileType's first bit: a reduced-resolution version of another image in the file
			0x0100 => ifd.width = value,
			0x0101 => ifd.height = value,
			0x0112 if (1..=8).contains(&value) => ifd.orientation = value,
			_ => {}
		}
	}
	Ok(ifd)
}
fn num16(b: &Bytes, at: usize, little: bool) -> Step<u16> { if little { b.le16(at) } else { b.be16(at) } }//a TIFF number in the structure's own byte order
fn num32(b: &Bytes, at: usize, little: bool) -> Step<u32> { if little { b.le32(at) } else { b.be32(at) } }

fn tiff(b: &Bytes, t: &mut Tip) -> Step<()> {//a TIFF's size is in its first directory, which libtiff and many writers keep at the end of the file, past the tip, so the header's pointer to it is where the second slice begins; and a camera's raw file is a TIFF whose first directory describes a small preview, which would be a wrong answer that looks right, so those are refused by the marks they carry
	t.format = "tiff"; t.medium = IMAGE;
	if b.has(8, b"CR") { return Err(Stop::Limit("a camera's raw file, whose first image is a preview")) }//Canon's CR2 puts these two letters after the header
	ask(b, to_usize(num32(b, 4, b.has(0, b"II"))?.into())?, "first directory past the tip")?;
	let ifd = ifd_read(b, 0, usize::MAX)?;//bounded by the reads alone, since the directory may sit in either slice
	if ifd.dng || ifd.reduced { return Err(Stop::Limit("a first image that is a reduced-size copy")) }//a DNG, or any TIFF whose first directory is a thumbnail of a later one, as camera raw files are
	if ifd.width == 0 || ifd.height == 0 { return Err(Stop::Corrupted("no size in the first directory")) }
	t.orientation = ifd.orientation;
	t.raster = Size { width: ifd.width, height: ifd.height };
	Ok(())
}

fn png(b: &Bytes, t: &mut Tip) -> Step<()> {//the header chunk is first by rule, with the size in its first eight bytes; the chunks after it are walked only to see whether an animation control chunk comes before the picture's data, which is what makes an APNG
	t.format = "png"; t.medium = IMAGE;
	if !b.has(12, b"IHDR") { return Err(Stop::Corrupted("no header chunk first")) }
	let (width, height) = (b.be32(16)?, b.be32(20)?);
	if width == 0 || height == 0 || width > 0x7FFF_FFFF || height > 0x7FFF_FFFF { return Err(Stop::Corrupted("a size outside the format's range")) }
	t.raster = Size { width, height };
	let mut at = 33;//past the signature and the header chunk: its length, name, thirteen bytes and checksum
	for _ in 0..STEPS {
		let length = to_usize(b.be32(at)?.into())?;
		match &b.four(add(at, 4)?)? {
			b"acTL" => t.animated = true,
			b"IDAT" | b"IEND" => return Ok(()),//the picture's data, or the end: whether it animates is settled
			_ => {}
		}
		at = add(add(at, 12)?, length)?;//the length and name, the data, and the checksum
	}
	Err(Stop::Limit("more chunks before the picture than any file has"))
}

fn gif(b: &Bytes, t: &mut Tip) -> Step<()> {//the size is in the screen descriptor after the six-byte signature. Whether it animates is in the blocks after the color table: the looping extension every animated GIF of the web carries says so, and the first image descriptor ends the walk, because what follows it is the first frame's data and a second frame would be past the tip
	t.format = "gif"; t.medium = IMAGE;
	let (width, height) = (u32::from(b.le16(6)?), u32::from(b.le16(8)?));
	if width == 0 || height == 0 { return Err(Stop::Corrupted("a zero size in the screen descriptor")) }
	t.raster = Size { width, height };
	let flags = b.u8(10)?;
	let mut at = 13;//past the header and the screen descriptor
	if flags & 0x80 != 0 { at = add(at, 6usize << (flags & 7))? }//the global color table: three bytes a color, and two to the power of one more than the low three bits of colors
	for _ in 0..STEPS {
		match b.u8(at)? {
			0x2C | 0x3B => return Ok(()),//the first image descriptor, or the trailer
			0x21 => {//an extension: a label, then sub-blocks of a length byte and that many bytes, ended by a zero
				let label = b.u8(add(at, 1)?)?;
				at = add(at, 2)?;
				if label == 0xFF && b.has(at, b"\x0BNETSCAPE2.0") { t.animated = true }//the application extension that sets a loop count, which a still picture never carries
				for _ in 0..STEPS {
					let n = usize::from(b.u8(at)?);
					at = add(at, 1)?;
					if n == 0 { break }
					at = add(at, n)?;
				}
			}
			_ => return Err(Stop::Corrupted("a block the format does not define")),
		}
	}
	Err(Stop::Limit("more extension blocks than any picture has"))
}

fn bmp(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//the file header says the file's size and where the pixels start; the information header after it has a size of its own, twelve for the 1990 OS/2 form with 16-bit sides and forty or more for Windows', with 32-bit sides, the height negative when the rows run top down
	t.format = "bmp"; t.medium = IMAGE;
	if u64::from(b.le32(2)?) > size { t.chopped = true }
	let (width, height) = if b.le32(14)? == 12 { (u32::from(b.le16(18)?), u32::from(b.le16(20)?)) } else { ((b.le32(18)? as i32).unsigned_abs(), (b.le32(22)? as i32).unsigned_abs()) };
	if width == 0 || height == 0 { return Err(Stop::Corrupted("a zero size in the header")) }
	t.raster = Size { width, height };
	Ok(())
}

fn webp(b: &Bytes, t: &mut Tip) -> Step<()> {//a RIFF whose first chunk is one of three: the lossy bitstream, whose size follows a start code; the lossless one, whose size is two fourteen-bit fields after its signature byte; or the extended header, which carries flags for animation and transparency and the canvas size as two 24-bit fields, less one
	t.format = "webp"; t.medium = IMAGE;
	let (width, height) = match &b.four(12)? {
		b"VP8 " => {
			if !b.has(23, &[0x9D, 0x01, 0x2A]) { return Err(Stop::Corrupted("no start code in the lossy bitstream")) }
			(u32::from(b.le16(26)? & 0x3FFF), u32::from(b.le16(28)? & 0x3FFF))
		}
		b"VP8L" => {
			if b.u8(20)? != 0x2F { return Err(Stop::Corrupted("no signature in the lossless bitstream")) }
			let bits = b.le32(21)?;
			(add_one(bits & 0x3FFF), add_one((bits >> 14) & 0x3FFF))
		}
		b"VP8X" => {
			t.animated = b.u8(20)? & 0x02 != 0;
			(add_one(b.le24(24)?), add_one(b.le24(27)?))
		}
		_ => return Err(Stop::Corrupted("a first chunk the format does not define")),
	};
	if width == 0 || height == 0 { return Err(Stop::Corrupted("a zero size")) }
	t.raster = Size { width, height };
	Ok(())
}
fn add_one(n: u32) -> u32 { n.saturating_add(1) }//a size a format stores less one

fn psd(b: &Bytes, t: &mut Tip) -> Step<()> {//Photoshop's header: the signature, a version, six reserved bytes, the channel count, then the height and the width as 32-bit numbers
	t.format = "psd"; t.medium = IMAGE;
	let (height, width) = (b.be32(14)?, b.be32(18)?);
	if width == 0 || height == 0 { return Err(Stop::Corrupted("a zero size in the header")) }
	t.raster = Size { width, height };
	Ok(())
}

//RIFF, the chunked form of the AVI, the WAV, the WebP and the Video CD's track

fn riff(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//a RIFF file is one chunk holding the rest, and the four letters after its size say which kind of file it is
	let n = b.le32(4)?;
	if n != u32::MAX && u64::from(n).saturating_add(8) > size { t.chopped = true }//the file's size as the header states it, against the file; all ones is a writer that never filled it in
	match &b.four(8)? {
		b"WEBP" => webp(b, t),
		b"AVI " => avi(b, t),
		b"WAVE" => wav(b, size, t),
		b"CDXA" => mpeg(b, 12, t),//a Video CD's track as its .dat file holds it: MPEG in sectors inside a RIFF wrapper, whose sequence header the search finds the same way
		_ => { t.format = "riff"; Err(Stop::Named(SIGNATURE_ONLY)) }
	}
}

fn riff_chunk(b: &Bytes, from: usize, end: usize, want: &[u8; 4]) -> Step<(usize, usize)> {//the first chunk named want between from and end, as the start of its data and the size it states; a chunk is a four-letter name, a little-endian size and the data, padded to an even length. The top level has no end but the tip's
	let mut at = from;
	for _ in 0..STEPS {
		if at >= end { return Err(Stop::Corrupted("no such chunk")) }
		let name = b.four(at)?;
		let n = to_usize(b.le32(add(at, 4)?)?.into())?;
		let data = add(at, 8)?;
		if &name == want { return Ok((data, n)) }
		at = add(add(data, n)?, n & 1)?;
	}
	Err(Stop::Limit("more chunks than the walk allows"))
}
fn riff_list(b: &Bytes, from: usize, end: usize, kind: &[u8; 4]) -> Step<(usize, usize)> {//the children of the first LIST chunk of this kind between from and end, as where they start and where they end
	let mut at = from;
	for _ in 0..STEPS {
		let (data, n) = riff_chunk(b, at, end, b"LIST")?;
		let list_end = add(data, n)?;
		if &b.four(data)? == kind { return Ok((add(data, 4)?, list_end)) }
		at = add(list_end, n & 1)?;
	}
	Err(Stop::Limit("more lists than the walk allows"))
}

fn avi(b: &Bytes, t: &mut Tip) -> Step<()> {//the main header is the first chunk of the header list: the time a frame lasts in microseconds, the frame count, and the size, at fixed offsets. A file past a gigabyte continues in further RIFF chunks, and its whole frame count is in an OpenDML header in the same list, taken when it is there. The pixel's shape is not read: AVI's place for it, the OpenDML video properties chunk, is one few writers fill in, and the shape an AVI carries is usually inside its codec's own stream
	t.format = "avi"; t.medium = VIDEO;
	let (from, end) = riff_list(b, 12, usize::MAX, b"hdrl")?;
	let (main, _) = riff_chunk(b, from, end, b"avih")?;
	let micros = b.le32(main)?;
	let mut frames = b.le32(add(main, 16)?)?;
	let (width, height) = (b.le32(add(main, 32)?)?, b.le32(add(main, 36)?)?);
	if let Ok((odml, odml_end)) = riff_list(b, from, end, b"odml") {
		if let Ok((dmlh, _)) = riff_chunk(b, odml, odml_end, b"dmlh") { if let Ok(total) = b.le32(dmlh) { if total > 0 { frames = total } } }
	}
	if micros > 0 && frames > 0 { t.seconds = seconds(f64::from(frames) * f64::from(micros) / 1e6) }
	if width == 0 || height == 0 { return Err(Stop::Corrupted("no size in the main header")) }
	t.raster = Size { width, height };
	Ok(())
}

fn wav(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//the format chunk says the bytes a second and the data chunk's stated size is the sound, so the running time is one divided by the other; a data chunk past the tip, which a long list of tags or a picture before the sound can cause, is where the second slice begins
	t.format = "wav"; t.medium = AUDIO;
	let (format, _) = riff_chunk(b, 12, usize::MAX, b"fmt ")?;
	let rate = b.le32(add(format, 8)?)?;//bytes a second
	let (data, n) = riff_chunk(b, 12, usize::MAX, b"data")?;
	if n == 0 || n == u32::MAX as usize { return Err(Stop::Limit("a data size the writer never filled in")) }//before the chopped check, since all ones is a size nobody wrote rather than one the file falls short of
	if (data as u64).saturating_add(n as u64) > size { t.chopped = true }
	if rate == 0 { return Err(Stop::Corrupted("no byte rate")) }
	t.seconds = seconds(n as f64 / f64::from(rate));
	Ok(())
}

//the ISO base media file: MP4, MOV, M4A, 3GP, and the HEIF pictures

fn iso_bare(b: &Bytes, size: u64) -> bool {//a QuickTime file with no file-type box, which the format allowed before 2001: the first box is one a movie has at its top, with a size that is 0, 1, or at least a header's and no more than the file's; four letters at the fifth byte are a signature text can pass, a line that opens "Get free" among them, and text's first four bytes read as a size of over 150 MB
	matches!(b.four(4), Ok(ref kind) if matches!(kind, b"moov" | b"mdat" | b"wide" | b"free" | b"skip" | b"pnot")) && matches!(b.be32(0), Ok(n) if n <= 1 || (n >= 8 && u64::from(n) <= size))
}

fn box_head(b: &Bytes, at: usize) -> Step<([u8; 4], usize, usize)> {//a box's four-letter type, where its data starts and where the box ends: a 32-bit size and the type, where a size of 1 means a 64-bit size follows and 0 means the box runs to the end of the file, which is answered as usize::MAX
	let n = b.be32(at)?;
	let kind = b.four(add(at, 4)?)?;
	match n {
		0 => Ok((kind, add(at, 8)?, usize::MAX)),
		1 => {
			let large = to_usize(b.be64(add(at, 8)?)?)?;
			if large < 16 { return Err(Stop::Corrupted("a box shorter than its header")) }
			Ok((kind, add(at, 16)?, add(at, large)?))
		}
		2..=7 => Err(Stop::Corrupted("a box shorter than its header")),
		_ => Ok((kind, add(at, 8)?, add(at, to_usize(n.into())?)?)),
	}
}

fn iso(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//the top-level boxes in order: the file-type box names the family, the movie box holds every answer, and the data box before the movie box is the layout every phone records and a web encoder undoes, stepped over to the movie box behind it, which the second slice holds when the data runs past the tip
	t.format = "mov"; t.medium = VIDEO;//until the file-type box says otherwise, which the first box nearly always is
	let mut at = 0;
	for _ in 0..STEPS {
		let (kind, data, end) = box_head(b, at)?;
		if end != usize::MAX && end as u64 > size { t.chopped = true }
		match &kind {
			b"ftyp" => iso_brand(b, data, t)?,
			b"moov" => return iso_moov(b, data, end, t),
			b"mdat" | b"moof" if end != usize::MAX => ask(b, end, "index at the end")?,//the movie's data before its index, or a fragmented file: the box's own size says where the next begins, in hand for a small file and the second slice for a phone's
			_ => {}//free space, wide, skip, uuid, pnot and whatever else sits at the top of a file: stepped over
		}
		if end == usize::MAX { return Err(Stop::Limit("a box that runs to the end of the file")) }
		at = end;
	}
	Err(Stop::Limit("more boxes than the walk allows"))
}

fn iso_brand(b: &Bytes, data: usize, t: &mut Tip) -> Step<()> {//the file's major brand, which says which family of file this is; the box lists compatible brands after it, which the reader does not need. A picture's brand ends the reading, since a HEIF picture's size sits in its item properties, a walk through three more boxes the reader does not make
	let brand = b.four(data)?;
	(t.format, t.medium) = match &brand {
		b"avif" | b"avis" => ("avif", IMAGE),
		b"heic" | b"heix" | b"hevc" | b"hevx" | b"heim" | b"heis" | b"mif1" | b"msf1" => ("heic", IMAGE),
		b"qt  " => ("mov", VIDEO),
		b"M4A " | b"M4B " | b"M4P " => ("m4a", AUDIO),
		_ if brand.starts_with(b"3g") => ("3gp", VIDEO),
		_ => ("mp4", VIDEO),
	};
	if matches!(&brand, b"avis" | b"msf1") { t.animated = true }//the sequence brands of the two picture families
	if t.medium == IMAGE { return Err(Stop::Limit("item properties not read")) }
	Ok(())
}

fn iso_moov(b: &Bytes, from: usize, end: usize, t: &mut Tip) -> Step<()> {//the movie box: the movie header with the running time, and a box per track with the handler that says what the track is, the matrix that turns it and its sizes
	let (mut at, mut tracks, mut visual) = (from, 0u32, false);
	for _ in 0..STEPS {
		if at >= end { break }
		let (kind, data, box_end) = box_head(b, at)?;
		let box_end = box_end.min(end);
		match &kind {
			b"mvhd" => {//version 0 keeps its times in 32 bits and version 1 in 64, which moves the timescale and the duration along
				let (scale, duration) = if b.u8(data)? == 1 { (b.be32(add(data, 20)?)?, b.be64(add(data, 24)?)?) } else { (b.be32(add(data, 12)?)?, u64::from(b.be32(add(data, 16)?)?)) };
				if scale > 0 { t.seconds = seconds(duration as f64 / f64::from(scale)) }
			}
			b"trak" => { tracks = tracks.saturating_add(1); if !visual { visual = iso_track(b, data, box_end, t)? } }
			_ => {}
		}
		at = box_end;
	}
	if tracks > 0 && !visual && t.medium == VIDEO { t.medium = AUDIO }//a movie with no picture is a sound file, whatever its brand said
	Ok(())
}

fn iso_track(b: &Bytes, from: usize, end: usize, t: &mut Tip) -> Step<bool> {//one track: true when its handler says it is a picture, with its stored size, its pixel's shape and its turn read. The track header comes first, with the matrix and the size the track shows at; the media box after it holds the handler and, three boxes further in, the sample description
	let (mut shown, mut orientation) = (Size { width: 0, height: 0 }, 1);
	let mut at = from;
	for _ in 0..STEPS {
		if at >= end { return Ok(false) }
		let (kind, data, box_end) = box_head(b, at)?;
		let box_end = box_end.min(end);
		match &kind {
			b"tkhd" => {
				let matrix = if b.u8(data)? == 1 { 52 } else { 40 };//where the matrix starts: after the version and flags, the times, the id, the duration, and the layer, group, volume and reserved fields, which version 1's 64-bit times push along by twelve
				shown = Size { width: b.be32(add(data, add(matrix, 36)?)?)? >> 16, height: b.be32(add(data, add(matrix, 40)?)?)? >> 16 };//the size after the matrix, as 16.16 fixed point, of which FFmpeg keeps the integer part, and so does this
				let number = |offset: usize| -> Step<i32> { Ok(b.be32(add(data, add(matrix, offset)?)?)? as i32) };
				let (a, bb, c, d) = (number(0)?, number(4)?, number(12)?, number(16)?);//the matrix's a, b, c and d, 16.16 fixed; u, v and the translation are not read
				const ONE: i32 = 1 << 16;
				const MINUS_ONE: i32 = -ONE;
				orientation = match (a, bb, c, d) { (0, ONE, MINUS_ONE, 0) => 6, (MINUS_ONE, 0, 0, MINUS_ONE) => 3, (0, MINUS_ONE, ONE, 0) => 8, _ => 1 };//the three quarter turns a phone records, in EXIF's numbering, checked against ffmpeg's rotate tags of 90, 180 and 270 in the samples; anything else, the identity included, shows as stored, and a matrix that stretches one side, which FFmpeg turns into a pixel's shape, is too rare to read as one
			}
			b"mdia" => {
				let Some((stored, pixel)) = iso_picture(b, data, box_end, shown)? else { return Ok(false) };
				(t.raster, t.pixel, t.orientation) = (stored, pixel, orientation);
				return Ok(true)
			}
			_ => {}
		}
		at = box_end;
	}
	Err(Stop::Limit("more boxes in a track than the walk allows"))
}

fn iso_picture(b: &Bytes, from: usize, end: usize, shown: Size) -> Step<Option<(Size, (u64, u64))>> {//a track's media box, when its handler says vide: the stored size from the first entry of the sample description, and the pixel's shape, from that entry's pasp box when it has one and otherwise from the shown size against the stored, which is the order FFmpeg's demuxer takes them in. None for every other handler, sound, a subtitle or a chapter's text, whose track header can carry a size of its own that would otherwise lend an audiobook a picture
	let Some((handler, _)) = iso_child(b, from, end, b"hdlr")? else { return Ok(None) };
	if &b.four(add(handler, 8)?)? != b"vide" { return Ok(None) }//after the version and flags, and four bytes that ISO leaves zero and QuickTime fills with its component type
	let Some((information, information_end)) = iso_child(b, from, end, b"minf")? else { return Err(Stop::Corrupted("a picture track with no sample description")) };
	let Some((table, table_end)) = iso_child(b, information, information_end, b"stbl")? else { return Err(Stop::Corrupted("a picture track with no sample description")) };
	let Some((description, description_end)) = iso_child(b, table, table_end, b"stsd")? else { return Err(Stop::Corrupted("a picture track with no sample description")) };
	let first = add(description, 8)?;//the first sample entry, past the version, the flags and the count; its type names the codec
	if first >= description_end { return Err(Stop::Corrupted("a sample description with no entry")) }
	let (_, entry, entry_end) = box_head(b, first)?;
	let entry_end = entry_end.min(description_end);
	if add(entry, 78)? > entry_end { return Err(Stop::Corrupted("a sample entry shorter than a picture's fields")) }//every visual entry, ISO's and QuickTime's, has these seventy-eight bytes before its boxes
	let stored = Size { width: b.be16(add(entry, 24)?)?.into(), height: b.be16(add(entry, 26)?)?.into() };//after six reserved bytes, the data reference and sixteen bytes ISO reserves and QuickTime spends on a version, a vendor and two qualities
	if stored.width == 0 || stored.height == 0 { return Err(Stop::Corrupted("a zero size in the sample description")) }
	let spacing = match iso_child(b, add(entry, 78)?, entry_end, b"pasp") {//the boxes after the entry's fields
		Ok(Some((pasp, _))) => (b.be32(pasp)?, b.be32(add(pasp, 4)?)?),
		Ok(None) | Err(Stop::Corrupted(_) | Stop::Limit(_)) => (0, 0),//a box the walk cannot step, like the color table an old QuickTime entry can keep there, costs the shape and nothing else
		Err(stop) => return Err(stop),//while bytes the slices do not hold are asked for, and bytes past the file's end are the file chopped, as for any read
	};
	let pixel = if spacing.0 > 0 && spacing.1 > 0 { (u64::from(spacing.0), u64::from(spacing.1)) }//the pixel's shape as two spacings, 32 and 27 for a DVD at 16:9
		else if shown.width > 0 && shown.height > 0 && shown != stored { (u64::from(shown.width).saturating_mul(stored.height.into()), u64::from(shown.height).saturating_mul(stored.width.into())) }//or the shown size over the stored, as a pixel's: 853 by 480 over 720 by 480 is 853 to 720
		else { (0, 0) };
	Ok(Some((stored, pixel)))
}

fn iso_child(b: &Bytes, from: usize, end: usize, want: &[u8; 4]) -> Step<Option<(usize, usize)>> {//the first box of a kind between two offsets, as where its data starts and where it ends; none when there is no such box
	let mut at = from;
	for _ in 0..STEPS {
		if at >= end { return Ok(None) }
		let (kind, data, box_end) = box_head(b, at)?;
		if &kind == want { return Ok(Some((data, box_end.min(end)))) }
		at = box_end.min(end);
	}
	Err(Stop::Limit("more boxes than the walk allows"))
}

//Matroska and WebM, whose elements are EBML: an id and a size as variable-length integers, then the data

fn ebml(b: &Bytes, t: &mut Tip) -> Step<()> {//the header names the document type, matroska or webm, and the segment after it holds the information element with the running time and the tracks element with each track's kind and size; both come before the first cluster of frames in every file a common muxer writes
	t.format = "matroska"; t.medium = VIDEO;
	let (_, header, header_end) = ebml_head(b, 0)?;
	if let Some((data, n)) = ebml_child(b, header, header_end, 0x4282)? { if b.read(data, n)? == b"webm" { t.format = "webm" } }
	let (id, segment, segment_end) = ebml_head(b, header_end)?;
	if id != 0x1853_8067 { return Err(Stop::Corrupted("no segment after the header")) }
	let (mut at, mut info, mut tracks) = (segment, false, false);
	for _ in 0..STEPS {
		if at >= segment_end || (info && tracks) { break }//both answers in hand: the attachments and tags that can follow run to megabytes, and walking them would send the reader after a slice it does not need
		let (id, data, end) = ebml_head(b, at)?;
		if id == 0x1F43_B675 { break }//the first cluster: the headers are behind it, and a live recording's clusters are of unknown size, so this comes before the check below
		if end == usize::MAX { return Err(Stop::Limit("an element of unknown size")) }//a header element written before its size was known, which leaves nothing to step over it by
		match id {
			0x1549_A966 => { ebml_info(b, data, end, t)?; info = true }
			0x1654_AE6B => { ebml_tracks(b, data, end, t)?; tracks = true }//in either order with the information element, since the format fixes neither
			_ => {}//the seek head, void space, tags, attachments, chapters, and cues when a muxer puts them first
		}
		at = end;
	}
	if tracks { Ok(()) } else { Err(Stop::Corrupted("no tracks before the first cluster")) }
}

fn ebml_size(b: &Bytes, at: usize) -> Step<(u64, usize, bool)> {//a variable-length integer as a size: the leading zeros of its first byte say its length, one to eight, the rest of its bits are the value, and every value bit set means the size is unknown
	let first = b.u8(at)?;
	let length = add(first.leading_zeros() as usize, 1)?;
	if length > 8 { return Err(Stop::Corrupted("a size longer than the format allows")) }
	let mask = (0xFFu32 >> length) as u8;
	let mut value = u64::from(first & mask);
	let mut unknown = first & mask == mask;
	for i in 1..length {
		let c = b.u8(add(at, i)?)?;
		value = (value << 8) | u64::from(c);
		unknown &= c == 0xFF;
	}
	Ok((value, length, unknown))
}
fn ebml_head(b: &Bytes, at: usize) -> Step<(u32, usize, usize)> {//an element's id, with its length marker kept as the format's documents write ids, where its data starts and where it ends, usize::MAX when its size is unknown
	let first = b.u8(at)?;
	let id_length = add(first.leading_zeros() as usize, 1)?;
	if id_length > 4 { return Err(Stop::Corrupted("an element id longer than the format allows")) }
	let mut id = 0u32;
	for i in 0..id_length { id = (id << 8) | u32::from(b.u8(add(at, i)?)?) }
	let (size, size_length, unknown) = ebml_size(b, add(at, id_length)?)?;
	let data = add(at, add(id_length, size_length)?)?;
	let end = if unknown { usize::MAX } else { add(data, to_usize(size)?)? };
	Ok((id, data, end))
}
fn ebml_child(b: &Bytes, from: usize, end: usize, want: u32) -> Step<Option<(usize, usize)>> {//the first child element with this id between from and end, as its data's start and size; none when there is no such child
	let mut at = from;
	for _ in 0..STEPS {
		if at >= end { return Ok(None) }
		let (id, data, element_end) = ebml_head(b, at)?;
		if element_end == usize::MAX { return Err(Stop::Limit("an element of unknown size")) }
		if id == want { return Ok(Some((data, element_end.checked_sub(data).ok_or(Stop::Corrupted("an element that ends before its data"))?))) }
		at = element_end;
	}
	Err(Stop::Limit("more elements than the walk allows"))
}
fn ebml_uint(b: &Bytes, data: usize, n: usize) -> Step<u64> {//an unsigned integer element, one to eight bytes, big-endian
	if n == 0 || n > 8 { return Err(Stop::Corrupted("an integer of a size the format does not allow")) }
	let mut value = 0u64;
	for i in 0..n { value = (value << 8) | u64::from(b.u8(add(data, i)?)?) }
	Ok(value)
}
fn ebml_float(b: &Bytes, data: usize, n: usize) -> Step<f64> {//a float element, four or eight bytes, big-endian
	match n {
		4 => Ok(f64::from(f32::from_be_bytes(array(b.read(data, 4)?)?))),
		8 => Ok(f64::from_be_bytes(array(b.read(data, 8)?)?)),
		_ => Err(Stop::Corrupted("a float of a size the format does not allow")),
	}
}
fn ebml_info(b: &Bytes, from: usize, end: usize, t: &mut Tip) -> Step<()> {//the running time is a float in timecode-scale units, nanoseconds by default and a millisecond in nearly every file
	let scale = match ebml_child(b, from, end, 0x2A_D7B1)? { Some((data, n)) => ebml_uint(b, data, n)?, None => 1_000_000 };
	if let Some((data, n)) = ebml_child(b, from, end, 0x4489)? { t.seconds = seconds(ebml_float(b, data, n)? * scale as f64 / 1e9) }
	Ok(())
}
fn ebml_tracks(b: &Bytes, from: usize, end: usize, t: &mut Tip) -> Step<()> {//each track entry says its type, 1 for a picture, and a picture track carries a video element with its size in pixels and, where the pixels are not square, its display size; the first picture track's sizes are the answer, and a file with none is a sound file
	let (mut at, mut visual) = (from, false);
	for _ in 0..STEPS {
		if at >= end { break }
		let (id, data, entry_end) = ebml_head(b, at)?;
		if entry_end == usize::MAX { return Err(Stop::Limit("an element of unknown size")) }
		if id == 0xAE && !visual {
			let kind = match ebml_child(b, data, entry_end, 0x83)? { Some((d, n)) => ebml_uint(b, d, n)?, None => 0 };
			if kind == 1 {
				visual = true;
				if let Some((video, n)) = ebml_child(b, data, entry_end, 0xE0)? {
					let video_end = add(video, n)?;
					let side = |want: u32| -> Step<u32> { match ebml_child(b, video, video_end, want)? { Some((d, n)) => Ok(u32::try_from(ebml_uint(b, d, n)?).unwrap_or(0)), None => Ok(0) } };
					let (width, height) = (side(0xB0)?, side(0xBA)?);
					if width > 0 && height > 0 {
						t.raster = Size { width, height };
						let display = |side: u32, pixels: u32| u64::from(if side > 0 { side } else { pixels });//a display side the file leaves out is its pixel count
						t.pixel = (display(side(0x54B0)?, width).saturating_mul(height.into()), display(side(0x54BA)?, height).saturating_mul(width.into()));//the display size over the pixel size, as a pixel's shape: 853 and 480 over 720 by 480, or 16 and 9 the way a Matroska file can write the same ratio, are both 32 to 27. The display unit is not read, and mpv's own Matroska reader reads it no more than this does; a crop is not read, which mpv also keeps out of the pixel's shape under its demuxer-mkv-crop-compat option, on by default, though it then draws the cropped rectangle, so the few files that carry one show in mpv at aspect less the crop
					}
				}
			}
		}
		at = entry_end;
	}
	if !visual { t.medium = AUDIO }
	Ok(())
}

//the other containers of video

fn mpeg(b: &Bytes, from: usize, t: &mut Tip) -> Step<()> {//an MPEG-1 or MPEG-2 stream, a program stream of packs or the bare video: the size is twelve bits each in the sequence header, found by its start code, which the format defines as the way to find it, and the pixel's shape is in the four bits after them. A program stream's running time is the last pack's clock less the first's, which is how FFmpeg estimates it, and a Video CD or a DVD writes a pack every two kilobytes, so the last is in the last 64 KiB, which the second slice reads; the bare video has no clock
	t.format = "mpeg"; t.medium = VIDEO;
	let header = b.find(from, &[0, 0, 1, 0xB3])?;
	if let Some(at) = header {
		let (b0, b1, b2) = (u32::from(b.u8(add(at, 4)?)?), u32::from(b.u8(add(at, 5)?)?), u32::from(b.u8(add(at, 6)?)?));
		let (width, height) = ((b0 << 4) | (b1 >> 4), ((b1 & 0x0F) << 8) | b2);
		if width == 0 || height == 0 { return Err(Stop::Corrupted("no size in the sequence header")) }
		t.raster = Size { width, height };
		t.pixel = mpeg_pixel(b, at, t.raster)?;
	}
	if let Some(first) = b.find(from, &[0, 0, 1, 0xBA])? {
		let start = pack_clock(b, first)?;
		ask_end(b, "clock reference at the end")?;
		let mut before = to_usize(b.size)?;
		for _ in 0..STEPS {//back from the end to the last pack header, which its four bytes alone do not prove, since four bytes of sound can spell them; the clock's marker bits do
			let Some(pack) = b.rfind(before, &[0, 0, 1, 0xBA])? else { break };
			if let Ok(clock) = pack_clock(b, pack) {
				if let Some(ticks) = clock.checked_sub(start) { t.seconds = seconds(ticks as f64 / 90000.0) }
				break
			}
			before = pack;
		}
	}
	if header.is_none() { return Err(Stop::Limit("no sequence header in the slices")) }
	Ok(())
}
const MPEG1_PIXELS: [(u64, u64); 14] = [(1, 1), (49, 33), (64, 45), (239, 182), (36, 29), (32, 27), (169, 151), (178, 163), (54, 53), (196, 201), (187, 200), (200, 219), (127, 147), (134, 161)];//MPEG-1's fourteen pixel shapes, codes 1 to 14, width to height. The format writes each as a pixel's height over its width to four places, and these are the fractions FFmpeg's decoder makes of them, av_d2q under 255 and turned over, which ffprobe reported for every code on the mac mini on 2026-10-18; a test pins them
fn mpeg_pixel(b: &Bytes, header: usize, raster: Size) -> Step<(u64, u64)> {//the pixel's shape from the four bits after the size in the sequence header at an offset, which the two versions spend differently: MPEG-1 on a pixel's shape from its table, and MPEG-2 on the whole picture's, square, 4:3, 16:9 or 2.21:1, which the stored size divides into a pixel's. A sequence extension at the next start code is what makes a stream MPEG-2, as it is to FFmpeg's decoder; a header whose next start code the slices do not hold, or is a pack's or a packet's because a packetizer split the two, reads as MPEG-1, which no sample and no muxer seen does. MPEG-2's ratio is taken over the whole frame and a display extension is not read, which is what FFmpeg does too unless the extension names a window whose shape under the code comes out at exactly 4:3 or 16:9; none when the code is one the format reserves
	let code = b.u8(add(header, 7)?)? >> 4;
	let next = b.find(add(header, 12)?, &[0, 0, 1])?;//the next start code, past the twelve bytes of the header and any of its two tables of quantizer weights, whose values the format forbids to be zero, so a search cannot stop inside them
	let mpeg2 = match next { Some(at) => matches!(b.held(add(at, 3)?, 2), Some(&[0xB5, id]) if id >> 4 == 1), None => false };//the extension start code, and 1 for the sequence extension in its first four bits
	if !mpeg2 { return Ok(usize::from(code).checked_sub(1).and_then(|i| MPEG1_PIXELS.get(i)).copied().unwrap_or((0, 0))) }
	let (across, down): (u64, u64) = match code { 1 => return Ok((1, 1)), 2 => (4, 3), 3 => (16, 9), 4 => (221, 100), _ => return Ok((0, 0)) };
	Ok((across.saturating_mul(raster.height.into()), down.saturating_mul(raster.width.into())))//the picture's ratio over the frame's, as a pixel's: 16:9 over 720 by 480 is 16 times 480 to 9 times 720, which is 32 to 27
}

fn pack_clock(b: &Bytes, at: usize) -> Step<u64> {//the system clock reference of the pack header at an offset, in ticks of a 90 kHz clock: 33 bits that MPEG-1 writes in five bytes behind a 0010 nibble and MPEG-2 in six behind 01, each with marker bits set between the fields, which a run of data that spells the start code fails
	let c = array::<6>(b.read(add(at, 4)?, 6)?)?;
	if c[0] & 0xF0 == 0x20 {
		if c[0] & 1 == 0 || c[2] & 1 == 0 || c[4] & 1 == 0 { return Err(Stop::Corrupted("not a pack header")) }
		Ok((u64::from((c[0] >> 1) & 7) << 30) | (u64::from(u16::from_be_bytes([c[1], c[2]]) >> 1) << 15) | u64::from(u16::from_be_bytes([c[3], c[4]]) >> 1))
	} else if c[0] & 0xC0 == 0x40 {
		if c[0] & 4 == 0 || c[2] & 4 == 0 || c[4] & 4 == 0 || c[5] & 1 == 0 { return Err(Stop::Corrupted("not a pack header")) }
		Ok((u64::from((c[0] >> 3) & 7) << 30) | (u64::from(c[0] & 3) << 28) | (u64::from(c[1]) << 20) | (u64::from((c[2] >> 3) & 0x1F) << 15) | (u64::from(c[2] & 3) << 13) | (u64::from(c[3]) << 5) | u64::from((c[4] >> 3) & 0x1F))
	} else { Err(Stop::Corrupted("not a pack header")) }
}

fn mpegts(b: &Bytes) -> bool {//a transport stream: the sync byte at the head of three packets in a row, of 188 bytes, or of 192 with a timestamp before each, as a Blu-ray's are
	(b.has(0, &[0x47]) && b.has(188, &[0x47]) && b.has(376, &[0x47])) || (b.has(4, &[0x47]) && b.has(196, &[0x47]) && b.has(388, &[0x47]))
}

fn flv(b: &Bytes, t: &mut Tip) -> Step<()> {//Flash video: a header with flags for its streams, then tags, the first of which is the metadata every common writer puts there, with the running time and the size. A stream captured without it keeps its running time only in its last tag's timestamp, at the end of the file, which the second slice reads
	t.format = "flv"; t.medium = VIDEO;
	let flags = b.u8(4)?;
	if flags & 1 == 0 && flags & 4 != 0 { t.medium = AUDIO }//bit 0 for a video stream and bit 2 for a sound one
	let metadata = flv_metadata(b, t);
	if t.seconds <= 0.0 {
		ask_end(b, "running time at the end")?;
		flv_last_tag(b, t)?;
	}
	metadata
}
fn flv_metadata(b: &Bytes, t: &mut Tip) -> Step<()> {//the first tag: a string naming it, then an array of names and typed values in AMF0, among them the running time and the size as doubles
	let tag = add(to_usize(b.be32(5)?.into())?, 4)?;//past the header and the size of the tag before it, which is zero for the first
	if b.u8(tag)? != 18 { return Err(Stop::Limit("the first tag is not metadata")) }
	let n = to_usize(b.be24(add(tag, 1)?)?.into())?;
	let data = add(tag, 11)?;
	let end = add(data, n)?;
	if !b.has(data, b"\x02\x00\x0AonMetaData") { return Err(Stop::Limit("no metadata tag first")) }//an AMF0 string: its type, its length and its text
	let mut at = add(data, 13)?;
	let count = match b.u8(at)? {//an ECMA array with its count, or a plain object, which ends at its end marker
		8 => { let c = to_usize(b.be32(add(at, 1)?)?.into())?; at = add(at, 5)?; c }
		3 => { at = add(at, 1)?; STEPS }
		_ => return Err(Stop::Limit("metadata in a shape the reader does not know")),
	};
	let (mut width, mut height) = (0.0f64, 0.0f64);
	for _ in 0..count.min(STEPS) {
		if at >= end { break }
		let length = usize::from(b.be16(at)?);
		let key = b.read(add(at, 2)?, length)?;
		at = add(add(at, 2)?, length)?;
		let kind = b.u8(at)?;
		at = add(at, 1)?;
		if length == 0 && kind == 9 { break }//the object end marker: an empty name and a type of 9
		match kind {
			0 => {//a number, as a double
				let value = f64::from_be_bytes(array(b.read(at, 8)?)?);
				at = add(at, 8)?;
				match key { b"duration" => t.seconds = seconds(value), b"width" => width = value, b"height" => height = value, _ => {} }
				if width >= 1.0 && height >= 1.0 && width <= 1e6 && height <= 1e6 { t.raster = Size { width: width as u32, height: height as u32 } }//as soon as both are read, so a value later in the metadata the reader cannot step over costs nothing already found
			}
			1 => at = add(at, 1)?,//a boolean
			2 => { let l = usize::from(b.be16(at)?); at = add(add(at, 2)?, l)? }//a string
			5 | 6 => {}//null and undefined
			11 => at = add(at, 10)?,//a date
			_ => return Err(Stop::Limit("a metadata value the reader does not step over")),//an object or an array inside the metadata, like the keyframe index flvtool2 and yamdi add
		}
	}
	Ok(())
}
fn flv_last_tag(b: &Bytes, t: &mut Tip) -> Step<()> {//the running time from the end: the file's last four bytes are the size of the tag before them, which begins that many bytes back, and its timestamp is when the last frame or sample begins, in milliseconds, three bytes and a fourth above them
	let last = sub(to_usize(b.size)?, 4)?;
	let n = to_usize(b.be32(last)?.into())?;
	let tag = sub(last, n)?;
	if !matches!(b.u8(tag)?, 8 | 9 | 18) || add(to_usize(b.be24(add(tag, 1)?)?.into())?, 11)? != n { return Err(Stop::Chopped("no whole tag at the end")) }//a sound, picture or script tag whose stated size agrees with the size written after it; a file cut short fails here
	let ms = u64::from(b.be24(add(tag, 4)?)?) | (u64::from(b.u8(add(tag, 7)?)?) << 24);
	t.seconds = seconds(ms as f64 / 1e3);
	Ok(())
}

const ASF_HEADER: [u8; 16] = [0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C];//the GUIDs of Windows Media's Advanced Systems Format as the file writes them, the first three fields little-endian: the header object, the file properties object, the stream properties object, and the two stream types
const ASF_FILE: [u8; 16] = [0xA1, 0xDC, 0xAB, 0x8C, 0x47, 0xA9, 0xCF, 0x11, 0x8E, 0xE4, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65];
const ASF_STREAM: [u8; 16] = [0x91, 0x07, 0xDC, 0xB7, 0xB7, 0xA9, 0xCF, 0x11, 0x8E, 0xE6, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65];
const ASF_AUDIO: [u8; 16] = [0x40, 0x9E, 0x69, 0xF8, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44, 0x2B];
const ASF_VIDEO: [u8; 16] = [0xC0, 0xEF, 0x19, 0xBC, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44, 0x2B];

fn asf(b: &Bytes, t: &mut Tip) -> Step<()> {//Windows Media, video and audio alike: the header object holds smaller objects, each a GUID and a 64-bit size; the file properties object has the play duration in hundreds of nanoseconds and the preroll in milliseconds, which the duration includes; a stream properties object says whether its stream is sound or picture, and a picture's size is the first two numbers of its type-specific data. The pixel's shape is not read: Windows Media keeps it as two attributes in the header extension, which few files carry
	t.format = "asf";
	let end = to_usize(b.le64(16)?)?;
	let (mut at, mut visual, mut sound) = (30, false, false);//past the header object's GUID, size, count and two reserved bytes
	for _ in 0..STEPS {
		if at >= end { break }
		let guid = b.read(at, 16)?;
		let n = to_usize(b.le64(add(at, 16)?)?)?;
		if n < 24 { return Err(Stop::Corrupted("an object shorter than its header")) }
		let data = add(at, 24)?;
		if guid == ASF_FILE {
			let (play, preroll) = (b.le64(add(data, 40)?)?, b.le64(add(data, 56)?)?);
			t.seconds = seconds(play as f64 / 1e7 - preroll as f64 / 1e3);
		} else if guid == ASF_STREAM {
			let kind = b.read(data, 16)?;
			if kind == ASF_VIDEO {
				visual = true;
				if t.raster.width == 0 {
					let (width, height) = (b.le32(add(data, 54)?)?, b.le32(add(data, 58)?)?);//the encoded image width and height, after the type GUIDs, the time offset, two lengths, the flags and a reserved field
					if width > 0 && height > 0 { t.raster = Size { width, height } }
				}
			} else if kind == ASF_AUDIO { sound = true }
		}
		at = add(at, n)?;
	}
	t.medium = if visual { VIDEO } else if sound { AUDIO } else { "" };
	Ok(())
}

fn realmedia(b: &Bytes, t: &mut Tip) -> Step<()> {//RealMedia: chunks of a four-letter name, a big-endian size and a version, where the properties chunk has the running time in milliseconds and each media properties chunk names its stream's MIME type, a video stream's type-specific data holding the size after its own size and VIDO mark
	t.format = "realmedia";
	let (mut at, mut visual, mut sound) = (0, false, false);
	for _ in 0..STEPS {
		let name = b.four(at)?;
		let n = to_usize(b.be32(add(at, 4)?)?.into())?;
		if n < 10 { return Err(Stop::Corrupted("a chunk shorter than its header")) }
		match &name {
			b"PROP" => t.seconds = seconds(f64::from(b.be32(add(at, 30)?)?) / 1e3),//after the version, the bit rates, the packet sizes and the packet count
			b"MDPR" => {
				let description = usize::from(b.u8(add(at, 40)?)?);//a length byte and the stream's description
				let mime_at = add(at, add(41, description)?)?;
				let mime_length = usize::from(b.u8(mime_at)?);
				let mime = b.read(add(mime_at, 1)?, mime_length)?;
				let specific = add(add(mime_at, 1)?, add(mime_length, 4)?)?;//past the MIME type and the type-specific data's length
				if mime.starts_with(b"video/") {
					visual = true;
					if t.raster.width == 0 && b.has(add(specific, 4)?, b"VIDO") {
						let (width, height) = (u32::from(b.be16(add(specific, 12)?)?), u32::from(b.be16(add(specific, 14)?)?));//after the size, the VIDO mark and the codec's four letters
						if width > 0 && height > 0 { t.raster = Size { width, height } }
					}
				} else if mime.starts_with(b"audio/") { sound = true }
			}
			b"DATA" | b"INDX" => break,//the packets, or the index: the headers are behind
			_ => {}
		}
		at = add(at, n)?;
	}
	t.medium = if visual { VIDEO } else if sound { AUDIO } else { "" };
	Ok(())
}

fn ogg(b: &Bytes, t: &mut Tip) -> Step<()> {//an Ogg stream's first page carries the first packet, which names the codec and the clock its granule positions count in; its running time is the last page's granule position, at the end of the file, which the second slice reads, since a page is at most 65,307 bytes and the last one begins inside the last 64 KiB
	t.format = "ogg";
	let payload = add(27, usize::from(b.u8(26)?))?;//the page header, then a lacing table of one byte a segment
	let serial = b.le32(14)?;//the first stream's serial number, which every page of it carries, so the page read at the end is this stream's and not that of a second stream muxed in or chained on after
	let (rate, skip, shift) = if b.has(payload, b"\x01vorbis") { t.medium = AUDIO; (f64::from(b.le32(add(payload, 12)?)?), 0, 0) }//the sample rate after the version and the channel count
		else if b.has(payload, b"OpusHead") { t.medium = AUDIO; (48000.0, u64::from(b.le16(add(payload, 10)?)?), 0) }//Opus counts its granule at 48 kHz whatever the sound's own rate, less the pre-skip after the version and the channel count
		else if b.has(payload, b"\x7FFLAC") { t.medium = AUDIO; (f64::from((b.be64(add(payload, 27)?)? >> 44) as u32), 0, 0) }//FLAC's stream information block inside the first packet, with its sample rate in the top twenty bits of the same eight bytes the bare file packs
		else if b.has(payload, b"Speex   ") { t.medium = AUDIO; (f64::from(b.le32(add(payload, 36)?)?), 0, 0) }//the rate after the version string, its id and the header's size
		else if b.has(payload, b"\x80theora") {//the picture size after the identification header's version and its frame size in macroblocks, then the frame rate as a fraction and the pixel's shape. A granule position is a keyframe's number and the count of frames since it, split at a shift the header gives, so the frame count is the two added
			t.medium = VIDEO;
			let (width, height) = (b.be24(add(payload, 14)?)?, b.be24(add(payload, 17)?)?);
			if width > 0 && height > 0 { t.raster = Size { width, height } }
			let (numerator, denominator) = (b.be32(add(payload, 22)?)?, b.be32(add(payload, 26)?)?);
			if denominator == 0 { return Err(Stop::Corrupted("no frame rate")) }
			t.pixel = (b.be24(add(payload, 30)?)?.into(), b.be24(add(payload, 33)?)?.into());//a numerator and a denominator, 32 and 27 for a DVD at 16:9, and 0 in either when the encoder did not know, as FFmpeg's Ogg reader takes them
			(f64::from(numerator) / f64::from(denominator), 0, u32::from((b.be16(add(payload, 40)?)? >> 5) & 0x1F))
		}
		else { return Err(Stop::Limit("a first packet the reader does not know")) };
	ask_end(b, "granule position at the end")?;
	let mut before = to_usize(b.size)?;
	for _ in 0..STEPS {//back from the end to the last page of the first stream, past the pages of any other, under the ceiling
		let Some(page) = b.rfind(before, b"OggS")? else { break };
		if b.u8(add(page, 4)?)? == 0 && b.le32(add(page, 14)?)? == serial {//the version byte, always 0, tells a page from the four letters inside a packet's data
			let granule = b.le64(add(page, 6)?)?;
			if granule == u64::MAX { return Err(Stop::Limit("a last page with no packet ending on it")) }
			let count = if shift > 0 { (granule >> shift).saturating_add(granule & (1u64 << shift).saturating_sub(1)) } else { granule.saturating_sub(skip) };
			if rate > 0.0 { t.seconds = seconds(count as f64 / rate) }
			return Ok(())
		}
		before = page;
	}
	Err(Stop::Limit("no page of the first stream at the end"))
}

//sound

fn flac(b: &Bytes, from: usize, t: &mut Tip) -> Step<()> {//the stream information block is first by rule, and eight of its bytes pack the sample rate in twenty bits, the channels and bits per sample, and the total samples in thirty-six; from is where the signature sits, the file's start or the end of an ID3 tag
	t.format = "flac"; t.medium = AUDIO;
	if b.u8(add(from, 4)?)? & 0x7F != 0 { return Err(Stop::Corrupted("the first metadata block is not the stream information")) }
	let packed = b.be64(add(from, 18)?)?;//after the block header and the block and frame sizes
	let (rate, samples) = ((packed >> 44) as u32, packed & 0xF_FFFF_FFFF);
	if rate == 0 { return Err(Stop::Corrupted("no sample rate")) }
	if samples == 0 { return Err(Stop::Limit("total samples not written")) }
	t.seconds = seconds(samples as f64 / f64::from(rate));
	Ok(())
}

const BITRATES_1_I: [u32; 14] = [32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448];//MPEG audio's bit rate tables in kilobits a second, by version and layer, indexed by the header's four bits less one
const BITRATES_1_II: [u32; 14] = [32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384];
const BITRATES_1_III: [u32; 14] = [32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
const BITRATES_2_I: [u32; 14] = [32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256];
const BITRATES_2_II_III: [u32; 14] = [8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];
const RATES_1: [u32; 3] = [44100, 48000, 32000];//and the sample rates, by version
const RATES_2: [u32; 3] = [22050, 24000, 16000];
const RATES_25: [u32; 3] = [11025, 12000, 8000];

struct Frame { layer: u8, kbps: u32, rate: u32, samples: u32, side: usize, length: usize }//one MPEG audio frame header, decoded: the layer, the bit rate, the sample rate, the samples a frame holds, the side information's size, after which a Xing header would sit, and the frame's length in bytes
fn mp3_header(b: &Bytes, at: usize) -> Option<Frame> {//the four bytes at an offset as a frame header, or none when they are not a valid one: a sync word and then fields none of which may hold a reserved value
	let h = array::<4>(b.read(at, 4).ok()?).ok()?;
	if h[0] != 0xFF || h[1] & 0xE0 != 0xE0 { return None }
	let version = (h[1] >> 3) & 3;//3 is MPEG-1, 2 is MPEG-2, 0 is MPEG-2.5, and 1 is reserved
	let layer = (h[1] >> 1) & 3;//3 is layer I, 2 is layer II, 1 is layer III, and 0 is reserved
	let bitrate_index = usize::from(h[2] >> 4);
	let rate_index = usize::from((h[2] >> 2) & 3);
	if version == 1 || layer == 0 || bitrate_index == 0 || bitrate_index == 15 || rate_index == 3 { return None }
	let padding = usize::from((h[2] >> 1) & 1);
	let mono = h[3] >> 6 == 3;
	let kbps = *match (version, layer) { (3, 3) => &BITRATES_1_I, (3, 2) => &BITRATES_1_II, (3, 1) => &BITRATES_1_III, (_, 3) => &BITRATES_2_I, _ => &BITRATES_2_II_III }.get(bitrate_index.checked_sub(1)?)?;
	let rate = *match version { 3 => &RATES_1, 2 => &RATES_2, _ => &RATES_25 }.get(rate_index)?;
	let samples: u32 = match layer { 3 => 384, 2 => 1152, _ if version == 3 => 1152, _ => 576 };
	let side = match (version == 3, mono) { (true, true) => 17, (true, false) => 32, (false, true) => 9, (false, false) => 17 };//layer III's side information, by version and channels
	let bits = (kbps as usize).checked_mul(1000)?;
	let length = if layer == 3 { bits.checked_mul(12)?.checked_div(rate as usize)?.checked_add(padding)?.checked_mul(4)? } else { (samples as usize).checked_div(8)?.checked_mul(bits)?.checked_div(rate as usize)?.checked_add(padding)? };//layer I counts in slots of four bytes; the others in bytes
	Some(Frame { layer, kbps, rate, samples, side, length })
}
fn mp3_find(b: &Bytes, from: usize) -> Step<(usize, Frame)> {//the first frame header at or after an offset whose next frame is where its length says, since a sync word alone is two bytes any file can hold, searched to the end of the slice the offset is in; accepted unchecked only where the tag said the frames begin and the slice has no room for the next
	let end = b.end_of(from);
	for at in from..end {
		let Some(head) = mp3_header(b, at) else { continue };
		let next = add(at, head.length)?;
		if mp3_header(b, next).is_some() || (at == from && next >= end) { return Ok((at, head)) }
	}
	Err(Stop::Limit("no frame header"))
}
fn adts(b: &Bytes, at: usize) -> bool {//raw AAC frames at an offset, which an ID3 tag can also wrap: the sync word of one, and a second sync word where the first frame's length field says the next begins, since the sync word alone is twelve bits any file can hold
	let Ok(h) = b.read(at, 6).and_then(array::<6>) else { return false };
	if h[0] != 0xFF || h[1] & 0xF6 != 0xF0 { return false }
	let length = (usize::from(h[3] & 3) << 11) | (usize::from(h[4]) << 3) | usize::from(h[5] >> 5);//thirteen bits across three bytes
	length >= 7 && matches!(add(at, length).and_then(|next| b.read(next, 2)), Ok(&[0xFF, second]) if second & 0xF6 == 0xF0)
}

fn mp3(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//MPEG audio, which is to say MP3: an ID3 tag first if there is one, then frames. The running time is the frame count in the Xing or Info header a modern encoder writes in a first silent frame, times the samples a frame holds, over the sample rate; without one the file is taken as constant bit rate and timed from its size, which is right for the constant-rate files of the early collections and wrong by its variation for a variable-rate file with no header
	t.format = "mp3"; t.medium = AUDIO;
	let mut at = 0;
	if b.has(0, b"ID3") {//an ID3v2 tag: a ten-byte header whose size is four seven-bit bytes, and a footer of ten more when its flag says so
		let flags = b.u8(5)?;
		let s = array::<4>(b.read(6, 4)?)?;
		let n = (usize::from(s[0] & 0x7F) << 21) | (usize::from(s[1] & 0x7F) << 14) | (usize::from(s[2] & 0x7F) << 7) | usize::from(s[3] & 0x7F);
		at = add(10, n)?;
		if flags & 0x10 != 0 { at = add(at, 10)? }
		ask(b, at, "a tag longer than the tip")?;//a cover picture in the tag can outgrow the tip, and the frames, which hold the running time, begin where the tag ends: the second slice
	}
	if b.has(at, b"fLaC") { return flac(b, at, t) }//a FLAC behind an ID3 tag, which some taggers write though the format keeps tags of its own
	if adts(b, at) { t.format = "aac"; return Err(Stop::Named(SIGNATURE_ONLY)) }//raw AAC frames; a frame count would mean reading every frame
	let (frame, head) = mp3_find(b, at)?;
	if head.layer == 1 {
		let xing = add(frame, add(4, head.side)?)?;
		if b.has(xing, b"Xing") || b.has(xing, b"Info") {//Xing from a variable-rate encoder, Info from LAME at a constant rate; either way the frame count follows when the first flag is set
			let flags = b.be32(add(xing, 4)?)?;
			if flags & 1 != 0 { t.seconds = seconds(f64::from(b.be32(add(xing, 8)?)?) * f64::from(head.samples) / f64::from(head.rate)); return Ok(()) }
		}
	}
	let sound = size.saturating_sub(frame as u64);//the frames, taken as the rest of the file; a tag at the end, 128 bytes of ID3v1, is counted as sound, which costs a hundredth of a second at 128 kbps
	t.seconds = seconds(sound as f64 * 8.0 / (f64::from(head.kbps) * 1000.0));
	Ok(())
}

fn au(b: &Bytes, t: &mut Tip) -> Step<()> {//Sun's sound file: after the signature, the data's offset, its size, the encoding, the sample rate and the channel count, each four bytes; the running time is the size over the bytes a second, which the encoding fixes
	t.format = "au"; t.medium = AUDIO;
	let (n, encoding, rate, channels) = (b.be32(8)?, b.be32(12)?, b.be32(16)?, b.be32(20)?);
	let bytes: u32 = match encoding { 1 | 2 | 27 => 1, 3 => 2, 4 => 3, 5 | 6 => 4, 7 => 8, _ => return Err(Stop::Limit("an encoding the reader does not know")) };//µ-law, 8-bit linear and A-law are a byte a sample; then 16, 24 and 32-bit linear, 32-bit float and 64-bit float
	if n == u32::MAX { return Err(Stop::Limit("a data size the writer never filled in")) }
	let per_second = f64::from(rate) * f64::from(channels) * f64::from(bytes);
	if per_second <= 0.0 { return Err(Stop::Corrupted("no sample rate")) }
	t.seconds = seconds(f64::from(n) / per_second);
	Ok(())
}

fn aiff(b: &Bytes, t: &mut Tip) -> Step<()> {//Apple's interchange format, chunked like a RIFF with big-endian sizes: the common chunk holds the channel count, the frame count, the bits a sample and the sample rate as an 80-bit extended float, which is the one place the reader meets one
	t.format = "aiff"; t.medium = AUDIO;
	let (common, _) = iff_chunk(b, 12, b"COMM")?;
	let frames = b.be32(add(common, 2)?)?;
	let exponent = i32::from(b.be16(add(common, 8)?)? & 0x7FFF);//a sign bit and fifteen bits of exponent, biased by 16383
	let mantissa = b.be64(add(common, 10)?)?;//sixty-four bits of mantissa with its leading one written out, so the value is the mantissa times two to the exponent less 63
	let rate = mantissa as f64 * 2f64.powi(exponent.saturating_sub(16_446));
	if !(rate.is_finite() && rate > 0.0) { return Err(Stop::Corrupted("no sample rate")) }
	t.seconds = seconds(f64::from(frames) / rate);
	Ok(())
}
fn iff_chunk(b: &Bytes, from: usize, want: &[u8; 4]) -> Step<(usize, usize)> {//the first chunk named want from an offset, as its data's start and stated size: a four-letter name, a big-endian size and the data, padded to an even length
	let mut at = from;
	for _ in 0..STEPS {
		let name = b.four(at)?;
		let n = to_usize(b.be32(add(at, 4)?)?.into())?;
		let data = add(at, 8)?;
		if &name == want { return Ok((data, n)) }
		at = add(add(data, n)?, n & 1)?;
	}
	Err(Stop::Limit("more chunks than the walk allows"))
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects, clippy::panic)]//a test may index, unwrap and add: a mistake in one fails the test, which is the point of it
mod tests {
	use super::*;

	fn scout(bytes: &[u8]) -> Tip {//the whole file in hand, as fuji's command hands a file shorter than the tip; with nothing of the file outside the tip, the reader never asks for a second slice
		let (t, wish) = scout_bytes(bytes, bytes.len() as u64);
		assert_eq!(wish, None);
		t
	}
	fn scout_file(bytes: &[u8], tip: usize) -> Tip {//the command's two reads over a file in memory: the first tip bytes, then, when the answer comes with a request, 64 KiB at the offset it names; so a test can hand the reader a tip shorter than the file and watch it ask
		let first = bytes.get(..tip).unwrap_or(bytes);
		let (t, wish) = scout_bytes(first, bytes.len() as u64);
		let Some(at) = wish else { return t };
		let slice = bytes.get(at as usize..(at as usize + TIP_BYTES).min(bytes.len())).unwrap_or(&[]);
		scout_slices(first, slice, at, bytes.len() as u64)
	}
	fn size(width: u32, height: u32) -> Size { Size { width, height } }
	fn be32(n: u32) -> [u8; 4] { n.to_be_bytes() }

	//hand-built headers, each the few bytes a format puts its answer in, so the test says exactly what the reader read

	#[test]
	fn nothing() {
		let t = scout(&[]);
		assert_eq!((t.format, t.text, t.binary, t.reading.parsed, t.reading.trouble, t.reading.details), ("", false, false, false, false, "empty"));
	}

	#[test]
	fn text_and_binary() {
		let t = scout(b"A line of text\nand another, with a tab\tand an escape \x1b[0m\n");
		assert!(t.text && !t.binary && t.format.is_empty() && t.reading.counts.nul == 0 && t.reading.counts.control == 0);
		let t = scout(b"\xFF\xFEt\0e\0x\0t\0");//utf-16 behind its byte order mark
		assert!(t.text && !t.binary);
		let t = scout(&[0x00, 0x01, 0x02, 0xFF, 0x80, 0x41]);
		assert!(!t.text && t.binary && t.format.is_empty() && t.reading.counts.nul == 1);
		let t = scout(b"\x01\x02\x03 control characters, and no zero byte among them");//neither: too many control characters for text, and no zero byte for binary
		assert!(!t.text && !t.binary);
		let t = scout(b"<?xml version=\"1.0\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\"/>");
		assert_eq!((t.text, t.format, t.medium, t.reading.parsed), (true, "svg", IMAGE, true));
		let t = scout(b"<?xml version=\"1.0\"?>\n<!-- Generator: Adobe Illustrator 24.0.0 -->\n<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\" [\n\t<!ENTITY ns_flows \"http://ns.adobe.com/Flows/1.0/\">\n]>\n<svg version=\"1.1\"/>\n");//a doctype holding declarations of its own
		assert_eq!(t.format, "svg");
		let t = scout(b"<!doctype html>\n<html><body><p>A page with a drawing.</p><svg width=\"8\" height=\"8\"></svg></body></html>\n");//a page that holds a drawing is a page
		assert_eq!((t.text, t.format), (true, ""));
		let t = scout(b"Draw it with an <svg> element.\n");
		assert_eq!((t.text, t.format), (true, ""));
		let t = scout(b"Get free shipping on every order\n");//a fifth byte that starts one of QuickTime's top boxes
		assert_eq!((t.text, t.format), (true, ""));
		let t = scout(b"PK\x03\x04and the rest of a zip");
		assert_eq!((t.format, t.medium, t.reading.details, t.reading.parsed, t.reading.trouble, t.text, t.binary), ("zip", "", SIGNATURE_ONLY, false, false, false, false));
		let t = scout(b"ID3 tags, as this text file opens with, name mp3 to the reader\n");//a signature's letters at the head of prose: the format and the text verdict both stand, and the caller sees both. The four letters after the tag's flags read as a size that points past the end
		assert_eq!((t.format, t.text, t.binary, t.chopped, t.reading.details), ("mp3", true, false, true, "a tag longer than the tip"));
	}

	#[test]
	fn the_shape_grows_the_short_edge_and_the_turn_swaps_both() {//the step at the end of scout, handed what a reader would have recorded: a stored size, a pixel's shape and a turn. The first three are the samples' numbers, with the sizes mpv 0.41 drew them at on the mac mini on 2026-10-18, read from its dwidth and dheight
		let shown = |raster: Size, pixel: (u64, u64), orientation: u32| { let mut t = scout(&[]); t.raster = raster; t.pixel = pixel; t.orientation = orientation; sizes(&mut t); (t.raster, t.aspect) };
		assert_eq!(shown(size(720, 480), (32, 27), 1), (size(720, 480), size(853, 480)));//a DVD at 16:9: wide pixels, so the width grows, and 853.33 drops its fraction
		assert_eq!(shown(size(720, 480), (8, 9), 1), (size(720, 480), size(720, 540)));//at 4:3: tall pixels, so the height grows, and the width never shrinks to 640
		assert_eq!(shown(size(352, 240), (200, 219), 1), (size(352, 240), size(352, 262)));//a Video CD's MPEG-1 code 12
		assert_eq!(shown(size(720, 576), (64, 45), 1), (size(720, 576), size(1024, 576)));//PAL at 16:9, and at 4:3
		assert_eq!(shown(size(720, 576), (16, 15), 1), (size(720, 576), size(768, 576)));
		assert_eq!(shown(size(720, 480), (32, 27), 6), (size(480, 720), size(480, 853)));//a quarter turn swaps both sizes together
		assert_eq!(shown(size(32, 20), (0, 0), 8), (size(20, 32), size(20, 32)));//no shape stated: the stored size, turned
		assert_eq!(shown(size(32, 20), (1, 1), 3), (size(32, 20), size(32, 20)));//a half turn swaps nothing
		assert_eq!(shown(size(u32::MAX, 1), (2, 1), 1), (size(u32::MAX, 1), size(u32::MAX, 1)));//a shape that would grow a side past what the answer holds leaves it as stored
	}

	#[test]
	fn mpeg_codes_give_the_shapes_ffmpeg_gives() {//every value of the four bits written into every sequence header of the two MPEG samples, against the sample aspect ratio ffprobe reported for the same bytes on the mac mini on 2026-10-18, FFmpeg 9.0.2; 0 and 0 where it reported none
		fn coded(name: &str, code: u8) -> (u64, u64) {
			let mut v = sample(name);
			let mut at = 0;
			while let Some(p) = v[at..].windows(4).position(|w| w == [0, 0, 1, 0xB3]) { at += p; v[at + 7] = (code << 4) | (v[at + 7] & 0x0F); at += 4 }
			scout(&v).pixel
		}
		let same = |(a, b): (u64, u64), (c, d): (u64, u64)| a * d == b * c && (a == 0) == (c == 0);//one shape, however the pair is scaled
		let mpeg1 = [(0, 0), (1, 1), (49, 33), (64, 45), (239, 182), (36, 29), (32, 27), (169, 151), (178, 163), (54, 53), (196, 201), (187, 200), (200, 219), (127, 147), (134, 161), (0, 0)];
		for (code, want) in mpeg1.into_iter().enumerate() { assert!(same(coded("video.mpg", code as u8), want), "MPEG-1 code {code}: {:?} against {want:?}", coded("video.mpg", code as u8)) }
		let mpeg2 = [(0, 0), (1, 1), (5, 6), (10, 9), (221, 160), (0, 0)];//over the sample's 32 by 20
		for (code, want) in mpeg2.into_iter().enumerate() { assert!(same(coded("video.vob", code as u8), want), "MPEG-2 code {code}: {:?} against {want:?}", coded("video.vob", code as u8)) }
	}

	fn png_header(width: u32, height: u32) -> Vec<u8> {
		let mut v = b"\x89PNG\r\n\x1A\n".to_vec();
		v.extend(be32(13)); v.extend(b"IHDR"); v.extend(be32(width)); v.extend(be32(height)); v.extend([8, 2, 0, 0, 0]); v.extend(be32(0));
		v
	}
	#[test]
	fn png_sizes_from_the_header_chunk() {
		let mut v = png_header(640, 480);
		v.extend(be32(0)); v.extend(b"IDAT"); v.extend(be32(0));
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.raster, t.aspect, t.animated, t.reading.parsed, t.reading.trouble, t.reading.details), ("png", IMAGE, size(640, 480), size(640, 480), false, true, false, ""));
		let mut v = png_header(2, 3);
		v.extend(be32(8)); v.extend(b"acTL"); v.extend([0; 12]);
		v.extend(be32(0)); v.extend(b"IDAT"); v.extend(be32(0));
		assert!(scout(&v).animated);
		let t = scout(&png_header(640, 480));//the whole file, ending before any picture data
		assert_eq!((t.raster.width, t.reading.details, t.reading.trouble, t.chopped, t.corrupted), (640, SHORT, true, true, false));
		let (t, wish) = scout_bytes(&png_header(640, 480), 1 << 20);//a tip cut from a longer file, which asks for the second slice where the chunks ran out
		assert_eq!((t.raster.width, t.reading.details, t.chopped, wish), (640, PAST, false, Some(33)));
	}

	fn jpeg_segment(code: u8, data: &[u8]) -> Vec<u8> {//a marker, a length that counts itself, and the data
		let mut v = vec![0xFF, code];
		v.extend((data.len() as u16 + 2).to_be_bytes()); v.extend(data);
		v
	}
	fn jpeg_frame(width: u16, height: u16) -> Vec<u8> {
		let mut data = vec![8]; data.extend(height.to_be_bytes()); data.extend(width.to_be_bytes()); data.extend([3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1]);
		jpeg_segment(0xC0, &data)
	}
	fn exif(orientation: u16) -> Vec<u8> {//an EXIF segment holding a big-endian TIFF with one directory of one entry
		let mut data = b"Exif\0\0MM\0\x2A".to_vec();
		data.extend(be32(8)); data.extend(1u16.to_be_bytes());
		data.extend(0x0112u16.to_be_bytes()); data.extend(3u16.to_be_bytes()); data.extend(be32(1)); data.extend(orientation.to_be_bytes()); data.extend([0, 0]);
		data.extend(be32(0));
		jpeg_segment(0xE1, &data)
	}
	#[test]
	fn jpeg_sizes_from_the_frame_header_and_turns_by_exif() {
		let mut v = vec![0xFF, 0xD8];
		v.extend(jpeg_segment(0xE0, b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0"));
		v.extend(jpeg_frame(640, 480));
		let t = scout(&v);
		assert_eq!((t.format, t.raster, t.orientation, t.reading.details), ("jpeg", size(640, 480), 1, ""));
		let mut v = vec![0xFF, 0xD8];
		v.extend(exif(6)); v.extend(jpeg_frame(640, 480));
		let t = scout(&v);
		assert_eq!((t.raster, t.aspect, t.orientation), (size(480, 640), size(480, 640), 6));
	}
	#[test]
	fn jpeg_steps_over_a_thumbnail() {//an EXIF segment whose directory is followed by a whole small JPEG, frame header and all, which a search would find first
		let mut inner = vec![0xFF, 0xD8]; inner.extend(jpeg_frame(160, 120)); inner.extend([0xFF, 0xD9]);
		let mut data = b"Exif\0\0MM\0\x2A".to_vec(); data.extend(be32(8)); data.extend(0u16.to_be_bytes()); data.extend(be32(0)); data.extend(inner);
		let mut v = vec![0xFF, 0xD8]; v.extend(jpeg_segment(0xE1, &data)); v.extend(jpeg_frame(4000, 3000));
		let t = scout(&v);
		assert_eq!(t.raster, size(4000, 3000));
	}
	#[test]
	fn jpeg_stops_where_the_bytes_run_out_or_lie() {
		let mut v = vec![0xFF, 0xD8, 0xFF, 0xE1]; v.extend(60000u16.to_be_bytes()); v.extend([0; 100]);//a segment that claims to run far past the end
		let t = scout(&v);
		assert_eq!((t.format, t.raster.width, t.reading.details, t.chopped, t.corrupted), ("jpeg", 0, SHORT, true, false));
		let t = scout(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x01]);//a length shorter than its own field
		assert_eq!((t.reading.details, t.reading.trouble, t.corrupted, t.chopped), ("a segment shorter than its own length field", true, true, false));
		let t = scout(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0x00, 0x00, 0x12]);//a segment followed by something other than a marker
		assert_eq!((t.reading.details, t.corrupted), ("no marker where the format puts one", true));
	}

	#[test]
	fn gif_bmp_webp_and_psd() {
		let mut v = b"GIF89a".to_vec(); v.extend(320u16.to_le_bytes()); v.extend(200u16.to_le_bytes()); v.extend([0x80, 0, 0]); v.extend([0; 6]);//a two-color global table
		v.extend(b"\x21\xFF\x0BNETSCAPE2.0\x03\x01\0\0\0"); v.push(0x2C);
		let t = scout(&v);
		assert_eq!((t.format, t.raster, t.animated, t.reading.details), ("gif", size(320, 200), true, ""));
		let mut v = b"BM".to_vec(); v.extend(1_000_000u32.to_le_bytes()); v.extend([0; 8]); v.extend(40u32.to_le_bytes()); v.extend(1024u32.to_le_bytes()); v.extend((-768i32).to_le_bytes()); v.extend([0; 28]);
		let t = scout(&v);
		assert_eq!((t.format, t.raster, t.chopped, t.reading.trouble), ("bmp", size(1024, 768), true, false));//a header that claims more than the file holds, and a reading that finished all the same
		let mut v = b"RIFF".to_vec(); v.extend(22u32.to_le_bytes()); v.extend(b"WEBPVP8X"); v.extend(10u32.to_le_bytes()); v.extend([0x02, 0, 0, 0]); v.extend([0x1F, 0x03, 0x00, 0x57, 0x02, 0x00]);
		let t = scout(&v);
		assert_eq!((t.format, t.raster, t.animated, t.chopped), ("webp", size(800, 600), true, false));
		let mut v = b"8BPS\0\x01\0\0\0\0\0\0\0\x03".to_vec(); v.extend(be32(600)); v.extend(be32(800)); v.extend([0, 8, 0, 3]);
		let t = scout(&v);
		assert_eq!((t.format, t.raster), ("psd", size(800, 600)));
	}

	fn iso_box(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
		let mut v = be32(data.len() as u32 + 8).to_vec(); v.extend(kind); v.extend(data);
		v
	}
	fn tkhd(width: u32, height: u32, matrix: [i32; 4]) -> Vec<u8> {//version 0, with the matrix's a, b, c and d given and the rest of it the identity
		let mut data = vec![0; 40];
		for n in [matrix[0], matrix[1], 0, matrix[2], matrix[3], 0, 0, 0, 1 << 30] { data.extend(n.to_be_bytes()) }
		data.extend(be32(width << 16)); data.extend(be32(height << 16));
		iso_box(b"tkhd", &data)
	}
	fn media(handler: &[u8; 4], width: u16, height: u16) -> Vec<u8> {//a track's media box down to its sample description, whose one entry has the stored size where a visual entry keeps it
		let mut hdlr = vec![0; 8]; hdlr.extend(handler); hdlr.extend([0; 13]);
		let mut entry = vec![0; 24]; entry.extend(width.to_be_bytes()); entry.extend(height.to_be_bytes()); entry.extend([0; 50]);
		let mut stsd = vec![0, 0, 0, 0]; stsd.extend(be32(1)); stsd.extend(iso_box(b"avc1", &entry));
		let mut v = iso_box(b"hdlr", &hdlr); v.extend(iso_box(b"minf", &iso_box(b"stbl", &iso_box(b"stsd", &stsd))));
		iso_box(b"mdia", &v)
	}
	#[test]
	fn mp4_reads_its_movie_box_when_it_comes_first() {
		let mut mvhd = vec![0; 12]; mvhd.extend(be32(1000)); mvhd.extend(be32(5500)); mvhd.extend([0; 80]);
		let one = 1 << 16;
		let track = |tkhd: Vec<u8>, media: Vec<u8>| { let mut v = tkhd; v.extend(media); iso_box(b"trak", &v) };
		let mut moov = iso_box(b"mvhd", &mvhd);
		moov.extend(track(tkhd(0, 0, [one, 0, 0, one]), media(b"soun", 0, 0)));//a sound track first
		moov.extend(track(tkhd(320, 60, [one, 0, 0, one]), media(b"sbtl", 0, 0)));//a subtitle track whose header has a size, which is no picture
		moov.extend(track(tkhd(1920, 1080, [0, one, -one, 0]), media(b"vide", 1920, 1080)));//then the picture, turned a quarter turn as a phone records it
		let mut v = iso_box(b"ftyp", b"mp42\0\0\0\0isommp42"); v.extend(iso_box(b"moov", &moov)); v.extend(iso_box(b"mdat", &[0; 16]));
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.raster, t.aspect, t.orientation, t.seconds, t.reading.details), ("mp4", VIDEO, size(1080, 1920), size(1080, 1920), 6, 5.5, ""));
		let mut sound = iso_box(b"mvhd", &mvhd); sound.extend(track(tkhd(0, 0, [one, 0, 0, one]), media(b"soun", 0, 0))); sound.extend(track(tkhd(320, 60, [one, 0, 0, one]), media(b"sbtl", 0, 0)));
		let mut v = iso_box(b"ftyp", b"isom\0\0\0\0"); v.extend(iso_box(b"moov", &sound));
		let t = scout(&v);
		assert_eq!((t.medium, t.raster), (AUDIO, size(0, 0)));//sound and a subtitle with a size: a sound file, and no size
	}
	#[test]
	fn mp4_with_its_index_at_the_end_names_the_stop() {
		let mut v = iso_box(b"ftyp", b"isom\0\0\0\0"); v.extend(be32(100_000)); v.extend(b"mdat"); v.extend([0; 16]);
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.seconds, t.reading.details, t.reading.trouble, t.chopped, t.corrupted), ("mp4", VIDEO, 0.0, "index at the end", true, true, false));//a data box that claims past the file's end: nothing to ask for, and the file is chopped where its index should be
		let (t, wish) = scout_bytes(&v, 1 << 30);
		assert_eq!((t.chopped, wish), (false, Some(100_016)));//in a file that long, the index is asked for where the data box ends
		let v = iso_box(b"ftyp", b"heic\0\0\0\0mif1heic");
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.reading.parsed, t.reading.trouble, t.reading.details, t.corrupted, t.chopped), ("heic", IMAGE, true, true, "item properties not read", false, false));
	}

	#[test]
	fn wav_and_flac_time_themselves() {
		let mut v = b"RIFF".to_vec(); v.extend(36u32.to_le_bytes()); v.extend(b"WAVEfmt "); v.extend(16u32.to_le_bytes()); v.extend([1, 0, 1, 0]); v.extend(8000u32.to_le_bytes()); v.extend(16000u32.to_le_bytes()); v.extend([2, 0, 16, 0]); v.extend(b"data"); v.extend(32000u32.to_le_bytes());
		let (t, wish) = scout_bytes(&v, 32044);
		assert_eq!((t.format, t.medium, t.seconds, t.chopped, wish), ("wav", AUDIO, 2.0, false, None));
		let t = scout(&v);//the data chunk claims more than the file holds
		assert_eq!((t.seconds, t.chopped, t.reading.trouble), (2.0, true, false));
		let mut unfilled = v.clone(); let n = unfilled.len(); unfilled[n - 4..].copy_from_slice(&u32::MAX.to_le_bytes());//a writer that never came back to fill in the size
		let t = scout(&unfilled);
		assert_eq!((t.reading.details, t.reading.trouble, t.chopped, t.corrupted), ("a data size the writer never filled in", true, false, false));
		let mut v = b"fLaC\0\0\0\x22".to_vec(); v.extend([0x10, 0x00, 0x10, 0x00, 0, 0, 0, 0, 0, 0]);
		let packed: u64 = (44100u64 << 44) | (1 << 41) | (15 << 36) | 88200;
		v.extend(packed.to_be_bytes()); v.extend([0; 16]);
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.seconds), ("flac", AUDIO, 2.0));
		let mut tagged = b"ID3\x04\0\0\0\0\0\x0A".to_vec(); tagged.extend([0; 10]); tagged.extend(&v);//the same FLAC behind an ID3 tag, as some taggers leave one
		let t = scout(&tagged);
		assert_eq!((t.format, t.medium, t.seconds), ("flac", AUDIO, 2.0));
	}

	#[test]
	fn mp3_counts_frames_from_a_xing_header_or_estimates_from_the_size() {
		let header = [0xFF, 0xFB, 0x90, 0x00];//MPEG-1 layer III at 128 kbps and 44100 Hz, stereo, 417 bytes a frame
		let mut v = header.to_vec(); v.extend([0; 32]); v.extend(b"Xing"); v.extend(be32(1)); v.extend(be32(3828)); v.resize(417, 0); v.extend(header); v.resize(417 * 2, 0);
		let t = scout(&v);
		assert_eq!((t.format, t.medium), ("mp3", AUDIO));
		assert!((t.seconds - 100.0).abs() < 0.01);
		let mut v = b"ID3\x04\0\0\0\0\0\x0A".to_vec(); v.extend([0; 10]); v.extend(header); v.resize(20 + 417, 0); v.extend(header); v.resize(20 + 417 * 2, 0);
		let (t, wish) = scout_bytes(&v, 20 + 160_000);
		assert!((t.seconds - 10.0).abs() < 0.01 && wish.is_none());
		let mut v = b"ID3\x04\0\0\0\0\0\x0A\0\0\0\0\0\0\0\0\0\0".to_vec();//a tag, then two raw AAC frames of 16 bytes, the second where the first's length says
		v.extend([0xFF, 0xF1, 0x50, 0x80, 0x02, 0x1F, 0xFC]); v.resize(20 + 16, 0); v.extend([0xFF, 0xF1, 0x50, 0x80, 0x02, 0x1F, 0xFC]); v.resize(20 + 32, 0);
		let t = scout(&v);
		assert_eq!((t.format, t.reading.details, t.reading.parsed, t.reading.trouble), ("aac", SIGNATURE_ONLY, false, false));
		let t = scout(b"\xFF\xFEt\0e\0x\0t\0 \0i\0n\0 \0u\0t\0f\0-\x001\x006\0");//a byte order mark that is also a valid frame header, with no second frame behind it
		assert_eq!((t.format, t.text), ("", true));
		let mut v = b"ID3\x04\0\0\0\x08\0\0".to_vec(); v.resize(TIP_BYTES, 0);//a tag of 128 KiB, the size a cover picture makes one
		let (t, wish) = scout_bytes(&v, 1 << 20);
		assert_eq!((t.format, t.reading.details, t.chopped, wish), ("mp3", "a tag longer than the tip", false, Some(131_082)));
		let t = scout(&v);//and longer than the file
		assert_eq!((t.reading.details, t.chopped), ("a tag longer than the tip", true));
	}

	//the sample files in the samples folder, made by real encoders on the mac mini on 2026-10-17 with ImageMagick 7.1, exiftool 13 and ffmpeg 8: every picture a 32 by 20 red-to-blue gradient, the eight orientations written by exiftool onto one JPEG and a 16 by 10 thumbnail embedded in another, every video a one-second test pattern at ten frames a second, the turned ones with ffmpeg's -noautorotate -display_rotation, which counts counter-clockwise, and every sound a one-second tone, at 8 kHz for the uncompressed kinds and 44.1 for the rest. On 2026-10-18 with ffmpeg 9 came video.vob, a DVD's program stream at the same size, and nine that carry a pixel's shape or a track that is no picture: the dvd ones at a DVD's 720 by 480, vcd.mpg at a Video CD's 352 by 240, the three MPEG streams among them thirty frames at NTSC's 29.97 a second, and audiobook.m4b, a tone beside a subtitle track; their rows say what each was flagged. Each row is what the maker was told to make, checked against ffprobe, exiftool and the files' own bytes, and the test pins the reader to it

	fn sample(name: &str) -> Vec<u8> { std::fs::read(format!("{}/samples/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap_or_else(|e| panic!("sample {name}: {e}")) }
	fn scout_sample(name: &str) -> Tip { scout_file(&sample(name), TIP_BYTES) }

	#[test]
	fn samples_answer_as_their_makers_made_them() {
		type Row = (&'static str, &'static str, &'static str, (u32, u32), (u32, u32), u32, f64, bool, (bool, bool), (bool, bool), &'static str);//the file; its format and medium; its stored size and the size to show it at, both turned; its orientation; its running time, matched within a twentieth of a second and 0 for none; whether it animates; text and binary; parsed and trouble; and the details. No sample is corrupted or chopped, which the loop checks for every row
		let rows: &[Row] = &[
			("picture.png", "png", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("picture.gif", "gif", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("animated.gif", "gif", IMAGE, (32, 20), (32, 20), 1, 0.0, true, (false, true), (true, false), ""),
			("picture.bmp", "bmp", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("picture.jpg", "jpeg", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("progressive.jpg", "jpeg", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("orient-1.jpg", "jpeg", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("orient-2.jpg", "jpeg", IMAGE, (32, 20), (32, 20), 2, 0.0, false, (false, true), (true, false), ""),
			("orient-3.jpg", "jpeg", IMAGE, (32, 20), (32, 20), 3, 0.0, false, (false, true), (true, false), ""),
			("orient-4.jpg", "jpeg", IMAGE, (32, 20), (32, 20), 4, 0.0, false, (false, true), (true, false), ""),
			("orient-5.jpg", "jpeg", IMAGE, (20, 32), (20, 32), 5, 0.0, false, (false, true), (true, false), ""),
			("orient-6.jpg", "jpeg", IMAGE, (20, 32), (20, 32), 6, 0.0, false, (false, true), (true, false), ""),
			("orient-7.jpg", "jpeg", IMAGE, (20, 32), (20, 32), 7, 0.0, false, (false, true), (true, false), ""),
			("orient-8.jpg", "jpeg", IMAGE, (20, 32), (20, 32), 8, 0.0, false, (false, true), (true, false), ""),
			("thumbnail.jpg", "jpeg", IMAGE, (20, 32), (20, 32), 6, 0.0, false, (false, true), (true, false), ""),//turned, with a 16 by 10 picture inside its EXIF segment that a search would have sized it by
			("picture.webp", "webp", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("lossless.webp", "webp", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("animated.webp", "webp", IMAGE, (32, 20), (32, 20), 1, 0.0, true, (false, true), (true, false), ""),
			("picture.tif", "tiff", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("picture.psd", "psd", IMAGE, (32, 20), (32, 20), 1, 0.0, false, (false, true), (true, false), ""),
			("picture.heic", "heic", IMAGE, (0, 0), (0, 0), 1, 0.0, false, (false, true), (true, true), "item properties not read"),
			("picture.avif", "avif", IMAGE, (0, 0), (0, 0), 1, 0.0, false, (false, true), (true, true), "item properties not read"),
			("picture.svg", "svg", IMAGE, (0, 0), (0, 0), 1, 0.0, false, (true, false), (true, false), ""),
			("moov-first.mp4", "mp4", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),
			("moov-last.mp4", "mp4", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),//the index after the data, as a phone writes it, in hand since the file is small; the second slice reads it in the test below
			("ccw-90.mp4", "mp4", VIDEO, (20, 32), (20, 32), 8, 1.0, false, (false, true), (true, false), ""),//a quarter turn counter-clockwise, which is EXIF's 8
			("ccw-180.mp4", "mp4", VIDEO, (32, 20), (32, 20), 3, 1.0, false, (false, true), (true, false), ""),
			("ccw-270.mp4", "mp4", VIDEO, (20, 32), (20, 32), 6, 1.0, false, (false, true), (true, false), ""),//and three quarters counter-clockwise is a quarter clockwise, EXIF's 6, the turn a phone held upright records
			("video.mov", "mov", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),
			("dvd-16x9.mp4", "mp4", VIDEO, (720, 480), (853, 480), 1, 1.0, false, (false, true), (true, false), ""),//720 by 480 flagged 16:9, which FFmpeg writes as a pasp box of 32 to 27 and a track header of 853.33 by 480, and mpv draws at 853 by 480
			("dvd-16x9.mov", "mov", VIDEO, (720, 480), (853, 480), 1, 1.0, false, (false, true), (true, false), ""),//the same as QuickTime, whose track header FFmpeg leaves at 720 by 480, so the pasp box alone holds the shape
			("audiobook.m4b", "m4a", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),//a tone and a subtitle track whose header has a size, 320 by 60, which is no picture
			("audio.m4a", "m4a", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),
			("video.avi", "avi", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),
			("video.mpg", "mpeg", VIDEO, (32, 20), (32, 20), 1, 0.58, false, (false, true), (true, false), ""),//an MPEG-1 program stream of three packs, the last clocked at 52,201 ticks of 90 kHz: the running time to the last pack, which is what the reader has, short of the last frame's end
			("video.vob", "mpeg", VIDEO, (32, 20), (32, 20), 1, 0.5, false, (false, true), (true, false), ""),//MPEG-2, whose pack writes its clock the other way, two packs and the last at 45,001
			("dvd-16x9.vob", "mpeg", VIDEO, (720, 480), (853, 480), 1, 0.7, false, (false, true), (true, false), ""),//a DVD's MPEG-2 at 720 by 480 flagged 16:9, which mpv draws at 853 by 480; eight packs, the last at 63,019 ticks
			("dvd-4x3.vob", "mpeg", VIDEO, (720, 480), (720, 540), 1, 0.7, false, (false, true), (true, false), ""),//and flagged 4:3, which mpv draws at 720 by 540
			("vcd.mpg", "mpeg", VIDEO, (352, 240), (352, 262), 1, 0.5, false, (false, true), (true, false), ""),//MPEG-1 at a Video CD's NTSC size flagged 4:3, which FFmpeg's encoder writes as code 12, and mpv draws at 352 by 262
			("video.ts", "mpegts", VIDEO, (0, 0), (0, 0), 1, 0.0, false, (false, true), (false, false), SIGNATURE_ONLY),
			("video.mkv", "matroska", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),
			("video.webm", "webm", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),
			("dvd-16x9.mkv", "matroska", VIDEO, (720, 480), (853, 480), 1, 1.0, false, (false, true), (true, false), ""),//720 by 480 flagged 16:9, which FFmpeg writes as a display size of 16 by 9 in display unit 3, a bare ratio, and mpv draws at 853 by 480
			("dvd-16x9.webm", "webm", VIDEO, (720, 480), (853, 480), 1, 1.0, false, (false, true), (true, false), ""),//the same flag, which WebM, allowing no such unit, writes as 853 by 480
			("dvd-4x3.webm", "webm", VIDEO, (720, 480), (720, 540), 1, 1.0, false, (false, true), (true, false), ""),//and 4:3, written as 640 by 480, which mpv draws at 720 by 540 rather than shrink the width
			("video.flv", "flv", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),
			("video.wmv", "asf", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),
			("video.rm", "realmedia", VIDEO, (32, 20), (32, 20), 1, 1.0, false, (false, true), (true, false), ""),
			("audio.wav", "wav", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),
			("audio.aiff", "aiff", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),
			("audio.au", "au", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),
			("audio.flac", "flac", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),
			("cbr.mp3", "mp3", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),
			("plain.mp3", "mp3", AUDIO, (0, 0), (0, 0), 1, 1.04, false, (false, true), (true, false), ""),//no Xing header and no tag, so the running time is the size over the bit rate, which ffprobe also answers 1.044875 for
			("vbr.mp3", "mp3", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),
			("audio.aac", "aac", AUDIO, (0, 0), (0, 0), 1, 0.0, false, (false, true), (false, false), SIGNATURE_ONLY),
			("audio.ogg", "ogg", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),//the last page's granule position over the rate, 44,160 samples at 44.1 kHz
			("audio.opus", "ogg", AUDIO, (0, 0), (0, 0), 1, 1.0, false, (false, true), (true, false), ""),//48,312 at 48 kHz, less a pre-skip of 312
			("audio.wma", "asf", AUDIO, (0, 0), (0, 0), 1, 1.02, false, (false, true), (true, false), ""),
			("text.txt", "", "", (0, 0), (0, 0), 1, 0.0, false, (true, false), (false, false), ""),
			("page.html", "", "", (0, 0), (0, 0), 1, 0.0, false, (true, false), (false, false), ""),
			("binary.bin", "", "", (0, 0), (0, 0), 1, 0.0, false, (false, true), (false, false), ""),
			("archive.zip", "zip", "", (0, 0), (0, 0), 1, 0.0, false, (false, true), (false, false), SIGNATURE_ONLY),
			("empty", "", "", (0, 0), (0, 0), 1, 0.0, false, (false, false), (false, false), "empty"),
		];
		let mut wrong = vec![];//every row that disagrees, so one run shows them all
		for &(name, format, medium, raster, aspect, orientation, secs, animated, (text, binary), (parsed, trouble), details) in rows {
			let t = scout_sample(name);
			let got = (t.format, t.medium, (t.raster.width, t.raster.height), (t.aspect.width, t.aspect.height), t.orientation, t.animated, (t.text, t.binary), (t.reading.parsed, t.reading.trouble), t.reading.details);
			let want = (format, medium, raster, aspect, orientation, animated, (text, binary), (parsed, trouble), details);
			if got != want { wrong.push(format!("{name}: {got:?} against {want:?}")) }
			if (t.seconds - secs).abs() >= 0.05 { wrong.push(format!("{name}: {} seconds against {secs}", t.seconds)) }
			if t.corrupted || t.chopped { wrong.push(format!("{name}: read as corrupted or chopped")) }
		}
		assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
	}

	#[test]
	fn samples_changed_the_way_other_writers_leave_them() {//the matroska sample with its segment and clusters of unknown size, as a live recording leaves them, the flash video's metadata with a value the reader cannot step over after the size, and the MP4 with its shape in the track header alone
		let mut v = sample("video.mkv");
		let (segment, cluster) = {
			let b = Bytes { tip: &v, slice: &[], slice_at: 0, size: v.len() as u64, depth: Cell::new(0) };
			let (_, _, header_end) = ebml_head(&b, 0).unwrap();
			let (_, mut at, _) = ebml_head(&b, header_end).unwrap();
			while ebml_head(&b, at).unwrap().0 != 0x1F43_B675 { at = ebml_head(&b, at).unwrap().2 }
			(header_end, at)
		};
		for element in [segment, cluster] {//a four-byte id, then the size, whose first byte says its length; every value bit set is unknown
			let size = element + 4;
			let length = v[size].leading_zeros() as usize + 1;
			v[size] = 0xFF >> (length - 1);
			for c in &mut v[size + 1..size + length] { *c = 0xFF }
		}
		let t = scout(&v);
		assert_eq!((t.format, t.raster, t.seconds, t.reading.details), ("matroska", size(32, 20), 1.0, ""));
		let mut v = sample("video.flv");
		let at = v.windows(7).position(|w| w == b"encoder").unwrap() + 7;//the value's type, after its name
		v[at] = 3;//an object, as a keyframe index is
		let t = scout(&v);
		assert_eq!((t.format, t.raster, t.reading.details, t.reading.trouble, t.corrupted, t.chopped), ("flv", size(32, 20), "a metadata value the reader does not step over", true, false, false));
		let mut v = sample("dvd-16x9.mp4");
		let at = v.windows(4).position(|w| w == b"pasp").unwrap();
		v[at] = b'x';//the pixel's spacing renamed away, as a writer that keeps the shape only in the track header leaves it: 853 by 480 there over 720 by 480 stored, which FFmpeg and mpv turn into the same 853 by 480
		let t = scout(&v);
		assert_eq!((t.raster, t.aspect, t.reading.details), (size(720, 480), size(853, 480), ""));
	}

	#[test]
	fn ogg_theora_counts_frames_from_its_granule() {//hand-built, since the mac mini's ffmpeg has no Theora encoder: an identification header at ten frames a second with a granule shift of 6, and a last page whose granule says keyframe 8 and two frames since, which is ten frames. The second is 720 by 480 with pixels of 32 to 27, which FFmpeg 9.0.2's own listing of the same bytes read as 720 by 480, SAR 32:27 and DAR 16:9, on the mac mini on 2026-10-18
		fn page(kind: u8, granule: u64, payload: &[u8]) -> Vec<u8> { let mut v = b"OggS\0".to_vec(); v.push(kind); v.extend(granule.to_le_bytes()); v.extend(7u32.to_le_bytes()); v.extend([0; 8]); v.push(1); v.push(payload.len() as u8); v.extend(payload); v }
		let theora = |width: u32, height: u32, pixel: (u32, u32)| {
			let mut header = b"\x80theora\x03\x02\x01".to_vec(); header.extend(((width / 16) as u16).to_be_bytes()); header.extend(((height / 16) as u16).to_be_bytes());
			header.extend(&width.to_be_bytes()[1..]); header.extend(&height.to_be_bytes()[1..]); header.extend([0, 0]); header.extend(be32(10)); header.extend(be32(1));
			header.extend(&pixel.0.to_be_bytes()[1..]); header.extend(&pixel.1.to_be_bytes()[1..]); header.extend([0; 4]); header.extend((6u16 << 5).to_be_bytes());
			let mut v = page(2, 0, &header); v.extend(page(4, (8 << 6) | 2, &[0]));
			scout(&v)
		};
		let t = theora(32, 20, (0, 0));//a shape the encoder did not know, which counts as square
		assert_eq!((t.format, t.medium, t.raster, t.aspect, t.seconds, t.reading.details), ("ogg", VIDEO, size(32, 20), size(32, 20), 1.0, ""));
		let t = theora(720, 480, (32, 27));
		assert_eq!((t.raster, t.aspect, t.seconds, t.reading.details), (size(720, 480), size(853, 480), 1.0, ""));
	}

	#[test]
	fn the_second_slice_reaches_what_the_tip_did_not() {//files larger than the tip, built around the samples, one for each way a reader asks: the first answer comes with the request and names the limit, and the second answer has what the sample's has
		let pad = 200_000;//past the tip and past twice it, so the last 64 KiB never overlap the tip
		let first = |v: &[u8]| scout_bytes(&v[..TIP_BYTES], v.len() as u64);
		let mut v = vec![0xFF, 0xD8]; v.extend(jpeg_segment(0xE2, &vec![0; 65533])); v.extend(&sample("picture.jpg")[2..]);//a segment as long as the format allows, a color profile say, before the picture's own
		let (t, wish) = first(&v);
		assert_eq!((t.format, t.raster.width, t.reading.details, t.reading.reads, wish), ("jpeg", 0, PAST, 1, Some(65_539)));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.raster, t.reading.details, t.reading.reads), (size(32, 20), "", 2));
		let mut v = vec![0xFF, 0xD8]; v.extend(jpeg_segment(0xE2, &vec![0; 65533])); v.extend(jpeg_segment(0xE2, &vec![0; 65533])); v.extend(&sample("picture.jpg")[2..]);//two of them: the second slice runs out too, and there is no third
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.raster.width, t.reading.details, t.reading.trouble, t.chopped, t.reading.reads), (0, PAST, true, false, 2));
		let m = sample("moov-last.mp4");
		let boxes: Vec<&[u8]> = { let mut v = vec![]; let mut at = 0; while at < m.len() { let n = u32::from_be_bytes(m[at..at + 4].try_into().unwrap()) as usize; v.push(&m[at..at + n]); at += n } v };
		let find = |kind: &[u8]| boxes.iter().find(|b| &b[4..8] == kind).unwrap().to_vec();
		let mut v = find(b"ftyp"); v.extend(be32(pad as u32 + 8)); v.extend(b"mdat"); v.resize(v.len() + pad, 0); v.extend(find(b"moov"));//the sample's data box grown past the tip, its movie box still behind it
		let end = (find(b"ftyp").len() + 8 + pad) as u64;
		let (t, wish) = first(&v);
		assert_eq!((t.format, t.seconds, t.reading.details, t.chopped, wish), ("mp4", 0.0, "index at the end", false, Some(end)));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.raster, t.seconds, t.reading.details, t.reading.reads), (size(32, 20), 1.0, "", 2));
		let mut v = b"ID3\x04\0\0".to_vec(); v.extend([(pad >> 21) as u8 & 0x7F, (pad >> 14) as u8 & 0x7F, (pad >> 7) as u8 & 0x7F, pad as u8 & 0x7F]); v.resize(10 + pad, 0); v.extend(sample("plain.mp3"));//a tag the size of a cover picture, then the frames
		let (t, wish) = first(&v);
		assert_eq!((t.format, t.seconds, t.reading.details, wish), ("mp3", 0.0, "a tag longer than the tip", Some(10 + pad as u64)));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.reading.details, t.reading.reads), ("", 2));
		assert!((t.seconds - 1.04).abs() < 0.05, "{}", t.seconds);
		let mut v = b"RIFF\0\0\0\0WAVEfmt ".to_vec(); v.extend(16u32.to_le_bytes()); v.extend([1, 0, 1, 0]); v.extend(8000u32.to_le_bytes()); v.extend(16000u32.to_le_bytes()); v.extend([2, 0, 16, 0]);
		v.extend(b"LIST"); v.extend((pad as u32).to_le_bytes()); v.resize(v.len() + pad, 0); let data = v.len() as u64; v.extend(b"data"); v.extend(32000u32.to_le_bytes()); v.resize(v.len() + 32000, 0);//a chunk the size of a picture before the sound
		let n = v.len() as u32 - 8; v[4..8].copy_from_slice(&n.to_le_bytes());
		let (t, wish) = first(&v);
		assert_eq!((t.format, t.seconds, t.reading.details, wish), ("wav", 0.0, PAST, Some(data)));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.seconds, t.reading.details, t.chopped), (2.0, "", false));
		let mut v = b"II*\0".to_vec(); v.extend((pad as u32).to_le_bytes()); v.resize(pad, 0);//the directory at the end of the file, as libtiff writes it
		v.extend(2u16.to_le_bytes()); for (tag, value) in [(0x0100u16, 32u32), (0x0101, 20)] { v.extend(tag.to_le_bytes()); v.extend(3u16.to_le_bytes()); v.extend(1u32.to_le_bytes()); v.extend(value.to_le_bytes()) } v.extend(0u32.to_le_bytes());
		let (t, wish) = first(&v);
		assert_eq!((t.format, t.raster.width, t.reading.details, wish), ("tiff", 0, "first directory past the tip", Some(pad as u64)));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.raster, t.reading.details), (size(32, 20), ""));
		let o = sample("audio.ogg"); let last = o.windows(4).rposition(|w| w == b"OggS").unwrap();
		let mut v = o.clone(); v.resize(o.len() + pad, 0); v.extend(&o[last..]);//the sample, then silence, then its last page again at the end
		let (t, wish) = first(&v);
		assert_eq!((t.format, t.medium, t.seconds, t.reading.details, wish), ("ogg", AUDIO, 0.0, "granule position at the end", Some((v.len() - TIP_BYTES) as u64)));
		let t = scout_file(&v, TIP_BYTES);
		assert!((t.seconds - 1.0).abs() < 0.05 && t.reading.details.is_empty(), "{} {}", t.seconds, t.reading.details);
		let mut f = sample("video.flv"); let key = f.windows(8).position(|w| w == b"duration").unwrap(); f[key] = b'x';//the running time in the metadata renamed, so the reader has to go to the end for it
		let t = scout(&f);
		assert!((t.seconds - 0.9).abs() < 0.01 && t.reading.details.is_empty(), "{} {}", t.seconds, t.reading.details);//the last tag's timestamp, in hand since the file is small
		let last = f.len() - 4 - u32::from_be_bytes(f[f.len() - 4..].try_into().unwrap()) as usize;
		let mut v = f[..last].to_vec(); v.resize(last + pad, 0); v.extend(&f[last..]);
		let (t, wish) = first(&v);
		assert_eq!((t.format, t.raster.width, t.seconds, t.reading.details, wish), ("flv", 32, 0.0, "running time at the end", Some((v.len() - TIP_BYTES) as u64)));
		let t = scout_file(&v, TIP_BYTES);
		assert!((t.seconds - 0.9).abs() < 0.01 && t.reading.details.is_empty(), "{} {}", t.seconds, t.reading.details);
		let p = sample("video.mpg"); let last = p.windows(4).rposition(|w| w == [0, 0, 1, 0xBA]).unwrap();
		let mut v = p.clone(); v.resize(p.len() + pad, 0); v.extend(&p[last..]);//the sample, then a gap, then its last pack again at the end
		let (t, wish) = first(&v);
		assert_eq!((t.format, t.raster.width, t.seconds, t.reading.details, wish), ("mpeg", 32, 0.0, "clock reference at the end", Some((v.len() - TIP_BYTES) as u64)));
		let t = scout_file(&v, TIP_BYTES);
		assert!((t.seconds - 0.58).abs() < 0.01 && t.reading.details.is_empty(), "{} {}", t.seconds, t.reading.details);
	}

	#[test]
	fn a_short_tip_asks_and_answers_the_same() {//every sample through tips shorter than the file: whatever the cut, the second slice brings the reader to the answer the whole file gives
		let folder = format!("{}/samples", env!("CARGO_MANIFEST_DIR"));
		for entry in std::fs::read_dir(&folder).unwrap() {
			let path = entry.unwrap().path();
			if !path.is_file() { continue }
			let name = path.file_name().unwrap().to_string_lossy().to_string();
			let bytes = std::fs::read(&path).unwrap();
			let whole = scout_file(&bytes, TIP_BYTES);
			for cut in [512, 1024, 2048, 4096] {
				if cut >= bytes.len() { continue }
				let t = scout_file(&bytes, cut);
				assert_eq!((&name, cut, t.format, t.medium, t.raster, t.aspect, t.orientation, t.animated, (t.corrupted, t.chopped), (t.reading.parsed, t.reading.trouble), t.reading.details), (&name, cut, whole.format, whole.medium, whole.raster, whole.aspect, whole.orientation, whole.animated, (whole.corrupted, whole.chopped), (whole.reading.parsed, whole.reading.trouble), whole.reading.details));
				assert!((t.seconds - whole.seconds).abs() < 0.001, "{name} cut at {cut}: {} seconds against {}", t.seconds, whole.seconds);
			}
		}
	}

	//the sweep: every input above and every sample file, cut at every length and with its bytes changed at random, must come back with an answer. A panic anywhere in the reader fails this test; the fuzzer in the fuzz folder does the same for longer, from the same samples

	struct Random(u64);//xorshift, enough to pick bytes to change
	impl Random { fn next(&mut self) -> u64 { let mut x = self.0; x ^= x << 13; x ^= x >> 7; x ^= x << 17; self.0 = x; x } }

	#[test]
	fn every_cut_and_every_mutation_returns() {
		let mut inputs: Vec<Vec<u8>> = vec![png_header(640, 480)];
		let mut v = vec![0xFF, 0xD8]; v.extend(exif(6)); v.extend(jpeg_frame(640, 480)); inputs.push(v);
		let folder = format!("{}/samples", env!("CARGO_MANIFEST_DIR"));
		for entry in std::fs::read_dir(&folder).unwrap() { let path = entry.unwrap().path(); if path.is_file() { inputs.push(std::fs::read(path).unwrap()) } }
		let mut random = Random(0x9E37_79B9_7F4A_7C15);
		for input in &inputs {
			let input = input.get(..TIP_BYTES).unwrap_or(input);
			for n in 0..=input.len().min(2048) { scout_file(input, n); }
			for _ in 0..400 {
				let mut changed = input.to_vec();
				if changed.is_empty() { break }
				for _ in 0..=(random.next() % 4) { let at = (random.next() as usize) % changed.len(); changed[at] = random.next() as u8 }
				let cut = (random.next() as usize) % (changed.len() + 1);
				scout_file(&changed, cut);
				scout_bytes(&changed[..cut], random.next() % (1 << 40));
				let second = (random.next() as usize) % (changed.len() + 1);
				scout_slices(&changed[..cut], &changed[second..], random.next() % (1 << 40), random.next() % (1 << 40));//the second read with a slice and offsets that agree with nothing, which must still return
			}
		}
	}
}
