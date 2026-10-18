#![forbid(unsafe_code)]//the whole crate is safe rust, checked by the compiler and refused if a line ever says otherwise; the essay says what that promise covers, and the rules below add what it does not
#![deny(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects)]//the rules the essay states, held by clippy wherever it runs: no index that could be out of range, no unwrap, no panic, no arithmetic that could overflow; rustc leaves lints in clippy's namespace alone, so this costs an ordinary build nothing

use serde::Serialize;
use std::cell::Cell;

/*
Scout: fuji's own reader of the first bytes of a file, a crate of its own so it builds, tests and fuzzes with nothing of fuji around it. Given the tip of a file, its first 64 KiB or the whole of a smaller one, and the file's size, scout_bytes says what the bytes are and what the common formats keep in their first bytes: the kind of medium, the size a picture or a video shows at with a phone's quarter turn applied, the running time of a sound or a video, whether a picture is animated, whether a header claims more than the file holds, and, for bytes that are no format it knows, whether they read as text or as binary. When the tip shows where the rest of an answer lies, the answer asks for one more slice, 64 KiB at an offset it names, and scout_slices answers again from both. It reads no frame, no pixel and no sample. The file's name and fuji's table of types play no part: the bytes name themselves, which is also why no name can steer the reader.

Why fuji writes this itself. Every parser that reads a collection's files reads bytes nobody vouched for, and the history fuji's security page tells is of decoders written in C and C++ made to run code by a file: GDI+'s JPEG, the Windows Metafile engine, ImageIO's JBIG2, libwebp, GStreamer's music decoders, FFmpeg's MagicYUV. A scan asks a question of every file on a drive, so it reaches more hostile files than any click, and the code it runs on them has to be code a file cannot subvert. Safe Rust is that code: the compiler refuses a read or a write outside the memory the program owns, which is the whole class of mistake those exploits were, and the first line of this file forbids unsafe code, so the compiler holds that promise for every line below.

The compiler does not promise the rest, and five rules do. The tip and the second slice are buffers fuji sized, one constant for both, and no number from the file sizes an allocation; the reader allocates nothing at all. Every read goes through one helper, Bytes, which holds the tip and the second slice as two windows onto the file and answers nothing outside them, so a length or an offset in the file can point anywhere and the worst it does is end the reading, or ask once for the slice that holds it. Every sum of offsets is a checked operation, since a release build wraps silently and a wrapped sum can look valid. Every loop moves forward and runs under a ceiling of steps as well as the slices' ends, so the work is bounded by 128 KiB whatever the file holds. And nothing here can panic: no indexing, no unwrap, no division before a check for zero; the lints on the second line hold those where clippy runs, and the tests below and the fuzzer in the fuzz folder hold them where it does not. Under those rules a hostile file gets one of two outcomes, no answer or a wrong one, and a wrong size or running time lasts until a decoder reads the file behind its own wall, the memory ceiling in thumbnail.rs among them, which stands whatever this reader said.

Why 64 KiB, and why a second slice. Most formats keep their answers in the first few hundred bytes. The exception that sets the size is JPEG, whose frame header comes after every application segment, and a phone's EXIF segment carries an embedded thumbnail of five to thirty kilobytes, a camera's up to sixty. Reading 64 KiB after a seek costs a fraction of a millisecond on a spinning disk against the ten the seek costs, so a night's work is the same at 16 KiB, and the tip is sized for the answer. Some answers are not in the tip at all, and the tip says where they are: a phone's MP4 keeps its index after the data box, whose size the box states; an MP3 behind a tag with cover art keeps its first frame where the tag's header says the tag ends; a TIFF's header points at its first directory, which libtiff writes last; and an Ogg, a Flash video without metadata and a DVD's program stream keep their running time in their last page, tag or pack, so in the file's last 64 KiB. For those the answer names an offset, the command reads one more 64 KiB slice there, and the reader runs again over both. That is the last read: never a third, and never a read whose place would have to be found by searching the file, so a file costs one seek or two and the work is bounded by 128 KiB whatever the file holds. The depths in the answer say how far into each slice a file's answers reached, so a run over a collection says what smaller slices would have missed.

What the reader does not try is chosen as carefully as what it does, and each reader's comment says where it stops. A HEIF picture's size sits behind three more boxes; a transport stream could be timed only by reading its packets; an MP4 whose sound track and its tables come before the picture track can keep that track past the second slice. Each of those is a named stop in the answer, and the stops are the map of the horizon: what a decoder would add. The answer is numbers and this file's own constants, so no text from the file ever reaches the page.

Where it runs. Today the command in fuji's scout.rs makes the two reads in fuji's own process on the blocking pool, which is safe because the reader is; the overnight scan will run the same function in a sandboxed process of fuji's own, where a hang on a dead disk can be killed, and nothing about the reader changes for that.
*/

pub const TIP_BYTES: usize = 64 << 10;//how much of a file fuji's command reads, and the most scout_bytes is ever handed: 64 KiB, the one constant the essay argues
const STEPS: usize = 256;//the most steps any walk below takes through a file's segments, chunks, boxes or elements before it gives up; more than any real file has before its answers, and the ceiling that bounds a loop a hostile file would otherwise keep going

const PAST: &str = "not in the slices";//the stop every reader can meet: a read of bytes the file holds and the slices in hand do not; after the first read it is also the request for the second slice, at the offset Bytes remembers
const SHORT: &str = "past the end of the file";//the other: a read past the file's size, which is a header claiming more than the file holds, and sets truncated
const SIGNATURE_ONLY: &str = "signature only";//the stop of a format the reader names and reads no further into
const IMAGE: &str = "image";//the three media, as the answer's medium says them
const VIDEO: &str = "video";
const AUDIO: &str = "audio";

type Step<T> = Result<T, &'static str>;//what every read and every reader answers: the value, or the reason the reading stopped, which becomes the answer's stop

#[derive(Serialize, Debug, PartialEq)]
pub struct Tip {//what the first bytes of a file say, every field set as far as the bytes allow and left at its blank when they do not
	pub text: bool,//the bytes read as text rather than binary, judged only when no format below claims them; text_verdict says the rule
	pub format: &'static str,//the format the first bytes announce, as this file names it: jpeg, png, mp4, matroska, mp3 and so on; blank when they announce nothing the reader knows
	pub medium: &'static str,//image, video or audio; blank for a format that is none of the three, and for a container whose streams the reader did not reach
	pub width: u32,//the size a picture or a video shows at, in its own pixels, with its turn applied; 0 when the tip does not say
	pub height: u32,
	pub orientation: u32,//the turn a picture or a video is shown with, in EXIF's numbering, 1 upright through 8; 1 when the file says nothing
	pub seconds: f64,//the running time of a sound or a video; 0 when the tip does not say, and an estimate where the comment on the reader says so
	pub animated: bool,//true when the format's own flag says the picture is animated; false where it says still, and where the reader could not look
	pub truncated: bool,//true when a header claims more bytes than the file holds, or the file ends inside the structure the reader was walking: an unfinished download, or a file cut short
	pub depth: u32,//how far into the tip the reader read to answer, in bytes; the measurement that says whether a smaller tip would have done
	pub stop: &'static str,//why the reader stopped before it had every answer this format can carry, as this file's own words; blank when it finished
	pub slice: u64,//where the second slice begins: after the first read, the offset the reader asks for, 0 when it asks for none; after the second, where the slice it was given began. The one request a reader makes, and never a third
	pub slice_depth: u32,//how far into the second slice the reader read, in bytes from its start; 0 before the second read
	pub nul: u32,//how many of the tip's bytes are zero, control characters other than the whitespace and escape of text, printable ascii and whitespace, and bytes past ascii; text_verdict reads them, and a run over a collection sets its thresholds
	pub control: u32,
	pub printable: u32,
	pub high: u32,
}

/// What the first bytes of a file say, from the tip alone and the file's size in bytes; the essay says what is answered and what is promised. Pure: no path, no open, no allocation, and the same answer for the same bytes on every platform. When the answer's slice is not 0 the reader is asking for the second slice, 64 KiB at that offset, and scout_slices takes both
pub fn scout_bytes(tip: &[u8], size: u64) -> Tip { scout(tip, None, size) }

/// The answer from the tip and the second slice the first answer asked for, at the offset it named, which is where the slice begins in the file. The second read is the last: a reader that still cannot reach its answer stops and says why, and the answer's slice says where the second read was
pub fn scout_slices(tip: &[u8], slice: &[u8], at: u64, size: u64) -> Tip { scout(tip, Some((at, slice)), size) }

fn scout(tip: &[u8], second: Option<(u64, &[u8])>, size: u64) -> Tip {//both entries: run the reader over the bytes in hand, turn where it stopped into the answer, and after the first read turn a stop at bytes it did not have into the request for the second
	let (slice_at, slice): (usize, &[u8]) = match second { Some((at, s)) => (usize::try_from(at).unwrap_or(usize::MAX), s), None => (0, &[]) };
	let b = Bytes { tip, slice, slice_at, size, depth: Cell::new(0), slice_depth: Cell::new(0), missed: Cell::new(0) };
	let (nul, control, printable, high) = text_counts(tip);
	let mut t = Tip { text: false, format: "", medium: "", width: 0, height: 0, orientation: 1, seconds: 0.0, animated: false, truncated: false, depth: 0, stop: "", slice: 0, slice_depth: 0, nul, control, printable, high };
	if tip.is_empty() { t.stop = "empty"; return t }
	match read_format(&b, size, &mut t) {
		Some(Ok(())) => {}
		Some(Err(reason)) => {
			t.stop = reason;
			if reason == SHORT { t.truncated = true }
			if second.is_none() {//the first read: the reader's own request, or else the offset of the read that fell outside the tip, is where the second slice begins. Past the file's end there is nothing to read, and a structure that points there claims more than the file holds
				let wish = if t.slice > 0 { t.slice } else if reason == PAST { b.missed.get() as u64 } else { 0 };
				if wish > size { t.truncated = true }
				t.slice = if wish < size { wish } else { 0 };
			}
		}
		None => {//no format: the bytes are text or they are not, and text that draws a picture is an svg
			b.depth.set(0);//the probes that found no signature are not reading, so the depth is what the search below needs and nothing else
			t.text = text_verdict(&b, &t);
			if t.text && svg(&b) { t.format = "svg"; t.medium = IMAGE }
		}
	}
	if let Some((at, _)) = second { t.slice = at }//the second read happened, here, whatever the reader wished for after it
	t.depth = u32::try_from(b.depth.get()).unwrap_or(u32::MAX);//exact, since a slice is at most 64 KiB
	t.slice_depth = u32::try_from(b.slice_depth.get()).unwrap_or(u32::MAX);
	t
}

fn read_format(b: &Bytes, size: u64, t: &mut Tip) -> Option<Step<()>> {//the format the first bytes announce, and that format's reader run on them: none when no signature matches, and otherwise the reader's own step, which is Ok when it finished and a reason when it stopped. A signature is checked where its format puts it, most at the first byte; the order matters only where two could match, and none here do
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
	else if mpegts(b) { t.format = "mpegts"; t.medium = VIDEO; Err(SIGNATURE_ONLY) }
	else if let Some(&(_, format, medium)) = SIGNATURES.iter().find(|(bytes, _, _)| b.has(0, bytes)) { t.format = format; t.medium = medium; Err(SIGNATURE_ONLY) }
	else { return None })
}

const SIGNATURES: &[(&[u8], &str, &str)] = &[//formats the reader names from their first bytes and reads no further into: sound a collection holds whose running time is not in its header, and common files that are not media, so a scan can say so without handing them to any decoder
	(b"MAC ", "ape", AUDIO),
	(b"wvpk", "wavpack", AUDIO),
	(b"#!AMR", "amr", AUDIO),
	(b"Creative Voice File", "voc", AUDIO),
	(b"MThd", "midi", AUDIO),
	(&[0x0B, 0x77], "ac3", AUDIO),
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
	depth: Cell<usize>,//the end of the farthest read the tip answered, for the answer's depth; a cell so the readers can share one Bytes without carrying it as mutable through every call
	slice_depth: Cell<usize>,//the same for the slice, counted from its start
	missed: Cell<usize>,//the offset of the last read that fell in the file but outside the windows: where the second slice begins when the reader stops there
}
impl<'a> Bytes<'a> {
	fn held(&self, at: usize, n: usize) -> Option<&'a [u8]> {//n bytes at an offset when one window holds them whole, the tip first, counting the read toward that window's depth; none otherwise, which read turns into its reason and has into false
		let end = at.checked_add(n)?;
		if let Some(s) = self.tip.get(at..end) { self.depth.set(self.depth.get().max(end)); return Some(s) }
		let from = at.checked_sub(self.slice_at)?;
		let to = from.checked_add(n)?;
		let s = self.slice.get(from..to)?;
		self.slice_depth.set(self.slice_depth.get().max(to));
		Some(s)
	}
	fn read(&self, at: usize, n: usize) -> Step<&'a [u8]> {//n bytes at an offset, or the reason there are none: the one read everything below is built on, and the line that makes the rule that nothing indexes past the end true of the whole file
		if let Some(s) = self.held(at, n) { return Ok(s) }
		if add(at, n)? as u64 > self.size { return Err(SHORT) }
		self.missed.set(at);
		Err(PAST)
	}
	fn u8(&self, at: usize) -> Step<u8> { self.read(at, 1)?.first().copied().ok_or(PAST) }
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
fn array<const N: usize>(s: &[u8]) -> Step<[u8; N]> { s.try_into().map_err(|_| PAST) }//a slice as a fixed array, for the from_bytes constructors; never fails after read gave exactly N bytes
fn add(a: usize, b: usize) -> Step<usize> { a.checked_add(b).ok_or("an offset sum that overflows") }//every sum of offsets below, checked; a wrapped sum is the arithmetic a hostile length aims for
fn sub(a: usize, b: usize) -> Step<usize> { a.checked_sub(b).ok_or("an offset difference below zero") }
fn mul(a: usize, b: usize) -> Step<usize> { a.checked_mul(b).ok_or("an offset product that overflows") }
fn to_usize(n: u64) -> Step<usize> { usize::try_from(n).map_err(|_| "a size larger than memory") }//a 64-bit size from a file as an offset, refused rather than wrapped where it does not fit
fn seconds(value: f64) -> f64 { if value.is_finite() && value > 0.0 { value } else { 0.0 } }//every running time passes through here on its way into the answer, so a division by a zero the reader missed, or a float a file wrote, lands as unknown rather than as infinity or a NaN
fn ask(b: &Bytes, t: &mut Tip, at: usize, reason: &'static str) -> Step<()> {//a reader's request for the second slice, where it knows the offset its answer lies at and the bytes in hand do not reach it: the offset goes in the answer and the reason is the stop, and after the second read the same line finds the bytes in hand and the reader goes on. A reader that walks into bytes it does not have asks the same way through read, which remembers the offset, with the plain stop for its reason
	if b.holds(at) { return Ok(()) }
	t.slice = at as u64;
	Err(reason)
}
fn ask_end(b: &Bytes, t: &mut Tip, reason: &'static str) -> Step<()> {//the same request for the last 64 KiB of the file, for the formats that keep their running time on their last page, tag or pack. At least 1, since 0 is where the tip begins and means no request; the command never hands over a tip shorter than 64 KiB that does not reach the file's end, so only a test can ask that
	if b.holds_end() { return Ok(()) }
	t.slice = b.size.saturating_sub(TIP_BYTES as u64).max(1);
	Err(reason)
}

//text or binary, from the whole tip

fn text_counts(tip: &[u8]) -> (u32, u32, u32, u32) {//how many bytes are nul, control, printable and high, over the whole tip: the raw material of text_verdict, and of a run over a collection that sets its thresholds by where the two humps fall
	let (mut nul, mut control, mut printable, mut high) = (0u32, 0u32, 0u32, 0u32);
	for &c in tip {
		match c {
			0 => nul = nul.saturating_add(1),
			0x09..=0x0D | 0x1B | 0x20..=0x7E => printable = printable.saturating_add(1),//tab, newline, vertical tab, form feed and return; escape, since ansi art is text; and printable ascii
			0x80..=0xFF => high = high.saturating_add(1),//utf-8 past ascii, or any byte of a binary file
			_ => control = control.saturating_add(1),//the other control characters, which text has none of
		}
	}
	(nul, control, printable, high)
}

fn text_verdict(b: &Bytes, t: &Tip) -> bool {//whether bytes that are no known format read as text: a utf-16 byte order mark says so outright, since every other byte of such text is a nul; otherwise no nul at all, which is git's rule for a binary file, and control characters in no more than one byte in a hundred. The one in a hundred is a factory value; a run over a collection, with the four counts in the answer, is what sets it where the two humps part
	if b.has(0, &[0xFF, 0xFE]) || b.has(0, &[0xFE, 0xFF]) { return true }
	if t.nul > 0 { return false }
	let total = t.control.saturating_add(t.printable).saturating_add(t.high);
	t.control.saturating_mul(100) <= total
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

fn shown(t: &mut Tip, width: u32, height: u32) {//the size a picture shows at: the stored size, with the sides swapped when the orientation turns it a quarter turn, EXIF's 5 through 8, the same four values thumbnail.rs swaps for
	if (5..=8).contains(&t.orientation) { t.width = height; t.height = width } else { t.width = width; t.height = height }
}

fn jpeg(b: &Bytes, t: &mut Tip) -> Step<()> {//a JPEG is segments, each a marker and a length, stepped through by their stated lengths to the start-of-frame segment that holds the size; never searched for, because the EXIF segment can hold a whole small JPEG, the thumbnail, with a frame header of its own. The orientation is a tag in that EXIF segment, read on the way past. A segment that runs past the tip, as a color profile's or a camera's can, is where the second slice begins
	t.format = "jpeg"; t.medium = IMAGE;
	let mut at = 2;//past the start-of-image marker
	for _ in 0..STEPS {
		if b.u8(at)? != 0xFF { return Err("no marker where the format puts one") }
		at = add(at, 1)?;
		while b.u8(at)? == 0xFF { at = add(at, 1)? }//a run of 0xFF before a marker's code is fill the format allows; each step moves forward, so the tip's end stops it
		let code = b.u8(at)?;
		at = add(at, 1)?;
		match code {
			0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => {//a start-of-frame segment, one of the thirteen kinds, baseline and progressive among them: its length, then the sample precision, the height, the width and the component count
				let (height, width) = (b.be16(add(at, 3)?)?, b.be16(add(at, 5)?)?);
				if height == 0 { return Err("a height the file defines after its first scan") }//the format allows a zero here, filled in by a marker after the scan, which no common writer uses
				if width == 0 { return Err("a zero width") }
				shown(t, width.into(), height.into());
				return Ok(())
			}
			0x01 | 0xD0..=0xDA => return Err("no frame header before the picture's data"),//the markers with no segment, the start and end of the image, and the start of the scan: none comes before the frame header in a picture the reader understands
			_ => {
				let length = usize::from(b.be16(at)?);//the segment's length, which counts these two bytes
				if length < 2 { return Err("a segment shorter than its own length field") }
				if code == 0xE1 && b.has(add(at, 2)?, b"Exif\0\0") {//the EXIF segment: a TIFF header and its first directory, where the orientation tag lives; a malformed one costs the orientation and nothing else
					if let Ok(ifd) = ifd_read(b, add(at, 8)?, add(at, length)?) { t.orientation = ifd.orientation }
				}
				at = add(at, length)?;
			}
		}
	}
	Err("more segments before the frame header than any picture has")
}

struct Ifd { width: u32, height: u32, orientation: u32, reduced: bool, dng: bool }//what the reader takes from a TIFF structure's first directory: the stored size, the orientation, whether the first image is a reduced-size copy of another, and whether the file is a DNG
fn ifd_read(b: &Bytes, tiff: usize, end: usize) -> Step<Ifd> {//the first image file directory of the TIFF structure at tiff, read no further than end: a byte order, the number 42, the directory's offset, then an entry count and twelve-byte entries of tag, type, count and value. A JPEG's EXIF segment and a TIFF file are both this
	let little = match b.read(tiff, 2)? { b"II" => true, b"MM" => false, _ => return Err("not a TIFF header") };
	if num16(b, add(tiff, 2)?, little)? != 42 { return Err("not a TIFF header") }
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
	if b.has(8, b"CR") { return Err("a camera's raw file, whose first image is a preview") }//Canon's CR2 puts these two letters after the header
	ask(b, t, to_usize(num32(b, 4, b.has(0, b"II"))?.into())?, "first directory past the tip")?;
	let ifd = ifd_read(b, 0, usize::MAX)?;//bounded by the reads alone, since the directory may sit in either slice
	if ifd.dng || ifd.reduced { return Err("a first image that is a reduced-size copy") }//a DNG, or any TIFF whose first directory is a thumbnail of a later one, as camera raw files are
	if ifd.width == 0 || ifd.height == 0 { return Err("no size in the first directory") }
	t.orientation = ifd.orientation;
	shown(t, ifd.width, ifd.height);
	Ok(())
}

fn png(b: &Bytes, t: &mut Tip) -> Step<()> {//the header chunk is first by rule, with the size in its first eight bytes; the chunks after it are walked only to see whether an animation control chunk comes before the picture's data, which is what makes an APNG
	t.format = "png"; t.medium = IMAGE;
	if !b.has(12, b"IHDR") { return Err("no header chunk first") }
	let (width, height) = (b.be32(16)?, b.be32(20)?);
	if width == 0 || height == 0 || width > 0x7FFF_FFFF || height > 0x7FFF_FFFF { return Err("a size outside the format's range") }
	t.width = width; t.height = height;
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
	Err("more chunks before the picture than any file has")
}

fn gif(b: &Bytes, t: &mut Tip) -> Step<()> {//the size is in the screen descriptor after the six-byte signature. Whether it animates is in the blocks after the color table: the looping extension every animated GIF of the web carries says so, and the first image descriptor ends the walk, because what follows it is the first frame's data and a second frame would be past the tip
	t.format = "gif"; t.medium = IMAGE;
	let (width, height) = (u32::from(b.le16(6)?), u32::from(b.le16(8)?));
	if width == 0 || height == 0 { return Err("a zero size in the screen descriptor") }
	t.width = width; t.height = height;
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
			_ => return Err("a block the format does not define"),
		}
	}
	Err("more extension blocks than any picture has")
}

fn bmp(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//the file header says the file's size and where the pixels start; the information header after it has a size of its own, twelve for the 1990 OS/2 form with 16-bit sides and forty or more for Windows', with 32-bit sides, the height negative when the rows run top down
	t.format = "bmp"; t.medium = IMAGE;
	if u64::from(b.le32(2)?) > size { t.truncated = true }
	let (width, height) = if b.le32(14)? == 12 { (u32::from(b.le16(18)?), u32::from(b.le16(20)?)) } else { ((b.le32(18)? as i32).unsigned_abs(), (b.le32(22)? as i32).unsigned_abs()) };
	if width == 0 || height == 0 { return Err("a zero size in the header") }
	t.width = width; t.height = height;
	Ok(())
}

fn webp(b: &Bytes, t: &mut Tip) -> Step<()> {//a RIFF whose first chunk is one of three: the lossy bitstream, whose size follows a start code; the lossless one, whose size is two fourteen-bit fields after its signature byte; or the extended header, which carries flags for animation and transparency and the canvas size as two 24-bit fields, less one
	t.format = "webp"; t.medium = IMAGE;
	let (width, height) = match &b.four(12)? {
		b"VP8 " => {
			if !b.has(23, &[0x9D, 0x01, 0x2A]) { return Err("no start code in the lossy bitstream") }
			(u32::from(b.le16(26)? & 0x3FFF), u32::from(b.le16(28)? & 0x3FFF))
		}
		b"VP8L" => {
			if b.u8(20)? != 0x2F { return Err("no signature in the lossless bitstream") }
			let bits = b.le32(21)?;
			(add_one(bits & 0x3FFF), add_one((bits >> 14) & 0x3FFF))
		}
		b"VP8X" => {
			t.animated = b.u8(20)? & 0x02 != 0;
			(add_one(b.le24(24)?), add_one(b.le24(27)?))
		}
		_ => return Err("a first chunk the format does not define"),
	};
	if width == 0 || height == 0 { return Err("a zero size") }
	t.width = width; t.height = height;
	Ok(())
}
fn add_one(n: u32) -> u32 { n.saturating_add(1) }//a size a format stores less one

fn psd(b: &Bytes, t: &mut Tip) -> Step<()> {//Photoshop's header: the signature, a version, six reserved bytes, the channel count, then the height and the width as 32-bit numbers
	t.format = "psd"; t.medium = IMAGE;
	let (height, width) = (b.be32(14)?, b.be32(18)?);
	if width == 0 || height == 0 { return Err("a zero size in the header") }
	t.width = width; t.height = height;
	Ok(())
}

//RIFF, the chunked form of the AVI, the WAV, the WebP and the Video CD's track

fn riff(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//a RIFF file is one chunk holding the rest, and the four letters after its size say which kind of file it is
	let n = b.le32(4)?;
	if n != u32::MAX && u64::from(n).saturating_add(8) > size { t.truncated = true }//the file's size as the header states it, against the file; all ones is a writer that never filled it in
	match &b.four(8)? {
		b"WEBP" => webp(b, t),
		b"AVI " => avi(b, t),
		b"WAVE" => wav(b, size, t),
		b"CDXA" => mpeg(b, 12, t),//a Video CD's track as its .dat file holds it: MPEG in sectors inside a RIFF wrapper, whose sequence header the search finds the same way
		_ => { t.format = "riff"; Err(SIGNATURE_ONLY) }
	}
}

fn riff_chunk(b: &Bytes, from: usize, end: usize, want: &[u8; 4]) -> Step<(usize, usize)> {//the first chunk named want between from and end, as the start of its data and the size it states; a chunk is a four-letter name, a little-endian size and the data, padded to an even length. The top level has no end but the tip's
	let mut at = from;
	for _ in 0..STEPS {
		if at >= end { return Err("no such chunk") }
		let name = b.four(at)?;
		let n = to_usize(b.le32(add(at, 4)?)?.into())?;
		let data = add(at, 8)?;
		if &name == want { return Ok((data, n)) }
		at = add(add(data, n)?, n & 1)?;
	}
	Err("more chunks than the walk allows")
}
fn riff_list(b: &Bytes, from: usize, end: usize, kind: &[u8; 4]) -> Step<(usize, usize)> {//the children of the first LIST chunk of this kind between from and end, as where they start and where they end
	let mut at = from;
	for _ in 0..STEPS {
		let (data, n) = riff_chunk(b, at, end, b"LIST")?;
		let list_end = add(data, n)?;
		if &b.four(data)? == kind { return Ok((add(data, 4)?, list_end)) }
		at = add(list_end, n & 1)?;
	}
	Err("more lists than the walk allows")
}

fn avi(b: &Bytes, t: &mut Tip) -> Step<()> {//the main header is the first chunk of the header list: the time a frame lasts in microseconds, the frame count, and the size, at fixed offsets. A file past a gigabyte continues in further RIFF chunks, and its whole frame count is in an OpenDML header in the same list, taken when it is there
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
	if width == 0 || height == 0 { return Err("no size in the main header") }
	t.width = width; t.height = height;
	Ok(())
}

fn wav(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//the format chunk says the bytes a second and the data chunk's stated size is the sound, so the running time is one divided by the other; a data chunk past the tip, which a long list of tags or a picture before the sound can cause, is where the second slice begins
	t.format = "wav"; t.medium = AUDIO;
	let (format, _) = riff_chunk(b, 12, usize::MAX, b"fmt ")?;
	let rate = b.le32(add(format, 8)?)?;//bytes a second
	let (data, n) = riff_chunk(b, 12, usize::MAX, b"data")?;
	if n == 0 || n == u32::MAX as usize { return Err("a data size the writer never filled in") }//before the truncation check, since all ones is a size nobody wrote rather than one the file falls short of
	if (data as u64).saturating_add(n as u64) > size { t.truncated = true }
	if rate == 0 { return Err("no byte rate") }
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
			if large < 16 { return Err("a box shorter than its header") }
			Ok((kind, add(at, 16)?, add(at, large)?))
		}
		2..=7 => Err("a box shorter than its header"),
		_ => Ok((kind, add(at, 8)?, add(at, to_usize(n.into())?)?)),
	}
}

fn iso(b: &Bytes, size: u64, t: &mut Tip) -> Step<()> {//the top-level boxes in order: the file-type box names the family, the movie box holds every answer, and the data box before the movie box is the shape every phone records and a web encoder undoes, stepped over to the movie box behind it, which the second slice holds when the data runs past the tip
	t.format = "mov"; t.medium = VIDEO;//until the file-type box says otherwise, which the first box nearly always is
	let mut at = 0;
	for _ in 0..STEPS {
		let (kind, data, end) = box_head(b, at)?;
		if end != usize::MAX && end as u64 > size { t.truncated = true }
		match &kind {
			b"ftyp" => iso_brand(b, data, t)?,
			b"moov" => return iso_moov(b, data, end, t),
			b"mdat" | b"moof" if end != usize::MAX => ask(b, t, end, "index at the end")?,//the movie's data before its index, or a fragmented file: the box's own size says where the next begins, in hand for a small file and the second slice for a phone's
			_ => {}//free space, wide, skip, uuid, pnot and whatever else sits at the top of a file: stepped over
		}
		if end == usize::MAX { return Err("a box that runs to the end of the file") }
		at = end;
	}
	Err("more boxes than the walk allows")
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
	if t.medium == IMAGE { return Err("item properties not read") }
	Ok(())
}

fn iso_moov(b: &Bytes, from: usize, end: usize, t: &mut Tip) -> Step<()> {//the movie box: the movie header with the running time, and a box per track with its size and the matrix that turns it
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

fn iso_track(b: &Bytes, from: usize, end: usize, t: &mut Tip) -> Step<bool> {//one track, from its header: true when it is a picture, with the size and the turn read; a sound track has a zero size there
	let mut at = from;
	for _ in 0..STEPS {
		if at >= end { return Ok(false) }
		let (kind, data, box_end) = box_head(b, at)?;
		if &kind == b"tkhd" {
			let matrix = if b.u8(data)? == 1 { 52 } else { 40 };//where the matrix starts: after the version and flags, the times, the id, the duration, and the layer, group, volume and reserved fields, which version 1's 64-bit times push along by twelve
			let (width, height) = (b.be32(add(data, add(matrix, 36)?)?)? >> 16, b.be32(add(data, add(matrix, 40)?)?)? >> 16);//the size after the matrix, as 16.16 fixed point, so the integer part is the high half
			if width == 0 || height == 0 { return Ok(false) }
			let number = |offset: usize| -> Step<i32> { Ok(b.be32(add(data, add(matrix, offset)?)?)? as i32) };
			let (a, bb, c, d) = (number(0)?, number(4)?, number(12)?, number(16)?);//the matrix's a, b, c and d, 16.16 fixed; u, v and the translation are not read
			const ONE: i32 = 1 << 16;
			const MINUS_ONE: i32 = -ONE;
			t.orientation = match (a, bb, c, d) { (0, ONE, MINUS_ONE, 0) => 6, (MINUS_ONE, 0, 0, MINUS_ONE) => 3, (0, MINUS_ONE, ONE, 0) => 8, _ => 1 };//the three quarter turns a phone records, in EXIF's numbering, checked against ffmpeg's rotate tags of 90, 180 and 270 in the samples; anything else, the identity included, shows as stored
			shown(t, width, height);
			return Ok(true)
		}
		at = box_end.min(end);
	}
	Err("more boxes in a track than the walk allows")
}

//Matroska and WebM, whose elements are EBML: an id and a size as variable-length integers, then the data

fn ebml(b: &Bytes, t: &mut Tip) -> Step<()> {//the header names the document type, matroska or webm, and the segment after it holds the information element with the running time and the tracks element with each track's kind and size; both come before the first cluster of frames in every file a common muxer writes
	t.format = "matroska"; t.medium = VIDEO;
	let (_, header, header_end) = ebml_head(b, 0)?;
	if let Some((data, n)) = ebml_child(b, header, header_end, 0x4282)? { if b.read(data, n)? == b"webm" { t.format = "webm" } }
	let (id, segment, segment_end) = ebml_head(b, header_end)?;
	if id != 0x1853_8067 { return Err("no segment after the header") }
	let (mut at, mut info, mut tracks) = (segment, false, false);
	for _ in 0..STEPS {
		if at >= segment_end || (info && tracks) { break }//both answers in hand: the attachments and tags that can follow run to megabytes, and walking them would send the reader after a slice it does not need
		let (id, data, end) = ebml_head(b, at)?;
		if id == 0x1F43_B675 { break }//the first cluster: the headers are behind it, and a live recording's clusters are of unknown size, so this comes before the check below
		if end == usize::MAX { return Err("an element of unknown size") }//a header element written before its size was known, which leaves nothing to step over it by
		match id {
			0x1549_A966 => { ebml_info(b, data, end, t)?; info = true }
			0x1654_AE6B => { ebml_tracks(b, data, end, t)?; tracks = true }//in either order with the information element, since the format fixes neither
			_ => {}//the seek head, void space, tags, attachments, chapters, and cues when a muxer puts them first
		}
		at = end;
	}
	if tracks { Ok(()) } else { Err("no tracks before the first cluster") }
}

fn ebml_size(b: &Bytes, at: usize) -> Step<(u64, usize, bool)> {//a variable-length integer as a size: the leading zeros of its first byte say its length, one to eight, the rest of its bits are the value, and every value bit set means the size is unknown
	let first = b.u8(at)?;
	let length = add(first.leading_zeros() as usize, 1)?;
	if length > 8 { return Err("a size longer than the format allows") }
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
	if id_length > 4 { return Err("an element id longer than the format allows") }
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
		if element_end == usize::MAX { return Err("an element of unknown size") }
		if id == want { return Ok(Some((data, element_end.checked_sub(data).ok_or("an element that ends before its data")?))) }
		at = element_end;
	}
	Err("more elements than the walk allows")
}
fn ebml_uint(b: &Bytes, data: usize, n: usize) -> Step<u64> {//an unsigned integer element, one to eight bytes, big-endian
	if n == 0 || n > 8 { return Err("an integer of a size the format does not allow") }
	let mut value = 0u64;
	for i in 0..n { value = (value << 8) | u64::from(b.u8(add(data, i)?)?) }
	Ok(value)
}
fn ebml_float(b: &Bytes, data: usize, n: usize) -> Step<f64> {//a float element, four or eight bytes, big-endian
	match n {
		4 => Ok(f64::from(f32::from_be_bytes(array(b.read(data, 4)?)?))),
		8 => Ok(f64::from_be_bytes(array(b.read(data, 8)?)?)),
		_ => Err("a float of a size the format does not allow"),
	}
}
fn ebml_info(b: &Bytes, from: usize, end: usize, t: &mut Tip) -> Step<()> {//the running time is a float in timecode-scale units, nanoseconds by default and a millisecond in nearly every file
	let scale = match ebml_child(b, from, end, 0x2A_D7B1)? { Some((data, n)) => ebml_uint(b, data, n)?, None => 1_000_000 };
	if let Some((data, n)) = ebml_child(b, from, end, 0x4489)? { t.seconds = seconds(ebml_float(b, data, n)? * scale as f64 / 1e9) }
	Ok(())
}
fn ebml_tracks(b: &Bytes, from: usize, end: usize, t: &mut Tip) -> Step<()> {//each track entry says its type, 1 for a picture, and a picture track carries a video element with its size in pixels; the first picture track's size is the answer, and a file with none is a sound file
	let (mut at, mut visual) = (from, false);
	for _ in 0..STEPS {
		if at >= end { break }
		let (id, data, entry_end) = ebml_head(b, at)?;
		if entry_end == usize::MAX { return Err("an element of unknown size") }
		if id == 0xAE && !visual {
			let kind = match ebml_child(b, data, entry_end, 0x83)? { Some((d, n)) => ebml_uint(b, d, n)?, None => 0 };
			if kind == 1 {
				visual = true;
				if let Some((video, n)) = ebml_child(b, data, entry_end, 0xE0)? {
					let video_end = add(video, n)?;
					let side = |want: u32| -> Step<u32> { match ebml_child(b, video, video_end, want)? { Some((d, n)) => Ok(u32::try_from(ebml_uint(b, d, n)?).unwrap_or(0)), None => Ok(0) } };
					let (width, height) = (side(0xB0)?, side(0xBA)?);
					if width > 0 && height > 0 { t.width = width; t.height = height }
				}
			}
		}
		at = entry_end;
	}
	if !visual { t.medium = AUDIO }
	Ok(())
}

//the other containers of video

fn mpeg(b: &Bytes, from: usize, t: &mut Tip) -> Step<()> {//an MPEG-1 or MPEG-2 stream, a program stream of packs or the bare video: the size is twelve bits each in the sequence header, found by its start code, which the format defines as the way to find it. A program stream's running time is the last pack's clock less the first's, which is how FFmpeg estimates it, and a Video CD or a DVD writes a pack every two kilobytes, so the last is in the last 64 KiB, which the second slice reads; the bare video has no clock
	t.format = "mpeg"; t.medium = VIDEO;
	let header = b.find(from, &[0, 0, 1, 0xB3])?;
	if let Some(at) = header {
		let (b0, b1, b2) = (u32::from(b.u8(add(at, 4)?)?), u32::from(b.u8(add(at, 5)?)?), u32::from(b.u8(add(at, 6)?)?));
		let (width, height) = ((b0 << 4) | (b1 >> 4), ((b1 & 0x0F) << 8) | b2);
		if width == 0 || height == 0 { return Err("no size in the sequence header") }
		t.width = width; t.height = height;
	}
	if let Some(first) = b.find(from, &[0, 0, 1, 0xBA])? {
		let start = pack_clock(b, first)?;
		ask_end(b, t, "clock reference at the end")?;
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
	if header.is_none() { return Err("no sequence header in the slices") }
	Ok(())
}
fn pack_clock(b: &Bytes, at: usize) -> Step<u64> {//the system clock reference of the pack header at an offset, in ticks of a 90 kHz clock: 33 bits that MPEG-1 writes in five bytes behind a 0010 nibble and MPEG-2 in six behind 01, each with marker bits set between the fields, which a run of data that spells the start code fails
	let c = array::<6>(b.read(add(at, 4)?, 6)?)?;
	if c[0] & 0xF0 == 0x20 {
		if c[0] & 1 == 0 || c[2] & 1 == 0 || c[4] & 1 == 0 { return Err("not a pack header") }
		Ok((u64::from((c[0] >> 1) & 7) << 30) | (u64::from(u16::from_be_bytes([c[1], c[2]]) >> 1) << 15) | u64::from(u16::from_be_bytes([c[3], c[4]]) >> 1))
	} else if c[0] & 0xC0 == 0x40 {
		if c[0] & 4 == 0 || c[2] & 4 == 0 || c[4] & 4 == 0 || c[5] & 1 == 0 { return Err("not a pack header") }
		Ok((u64::from((c[0] >> 3) & 7) << 30) | (u64::from(c[0] & 3) << 28) | (u64::from(c[1]) << 20) | (u64::from((c[2] >> 3) & 0x1F) << 15) | (u64::from(c[2] & 3) << 13) | (u64::from(c[3]) << 5) | u64::from((c[4] >> 3) & 0x1F))
	} else { Err("not a pack header") }
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
		ask_end(b, t, "running time at the end")?;
		flv_last_tag(b, t)?;
	}
	metadata
}
fn flv_metadata(b: &Bytes, t: &mut Tip) -> Step<()> {//the first tag: a string naming it, then an array of names and typed values in AMF0, among them the running time and the size as doubles
	let tag = add(to_usize(b.be32(5)?.into())?, 4)?;//past the header and the size of the tag before it, which is zero for the first
	if b.u8(tag)? != 18 { return Err("the first tag is not metadata") }
	let n = to_usize(b.be24(add(tag, 1)?)?.into())?;
	let data = add(tag, 11)?;
	let end = add(data, n)?;
	if !b.has(data, b"\x02\x00\x0AonMetaData") { return Err("no metadata tag first") }//an AMF0 string: its type, its length and its text
	let mut at = add(data, 13)?;
	let count = match b.u8(at)? {//an ECMA array with its count, or a plain object, which ends at its end marker
		8 => { let c = to_usize(b.be32(add(at, 1)?)?.into())?; at = add(at, 5)?; c }
		3 => { at = add(at, 1)?; STEPS }
		_ => return Err("metadata in a shape the reader does not know"),
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
				if width >= 1.0 && height >= 1.0 && width <= 1e6 && height <= 1e6 { t.width = width as u32; t.height = height as u32 }//as soon as both are read, so a value later in the metadata the reader cannot step over costs nothing already found
			}
			1 => at = add(at, 1)?,//a boolean
			2 => { let l = usize::from(b.be16(at)?); at = add(add(at, 2)?, l)? }//a string
			5 | 6 => {}//null and undefined
			11 => at = add(at, 10)?,//a date
			_ => return Err("a metadata value the reader does not step over"),//an object or an array inside the metadata, like the keyframe index flvtool2 and yamdi add
		}
	}
	Ok(())
}
fn flv_last_tag(b: &Bytes, t: &mut Tip) -> Step<()> {//the running time from the end: the file's last four bytes are the size of the tag before them, which begins that many bytes back, and its timestamp is when the last frame or sample begins, in milliseconds, three bytes and a fourth above them
	let last = sub(to_usize(b.size)?, 4)?;
	let n = to_usize(b.be32(last)?.into())?;
	let tag = sub(last, n)?;
	if !matches!(b.u8(tag)?, 8 | 9 | 18) || add(to_usize(b.be24(add(tag, 1)?)?.into())?, 11)? != n { return Err("no whole tag at the end") }//a sound, picture or script tag whose stated size agrees with the size written after it; a file cut short fails here
	let ms = u64::from(b.be24(add(tag, 4)?)?) | (u64::from(b.u8(add(tag, 7)?)?) << 24);
	t.seconds = seconds(ms as f64 / 1e3);
	Ok(())
}

const ASF_HEADER: [u8; 16] = [0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C];//the GUIDs of Windows Media's Advanced Systems Format as the file writes them, the first three fields little-endian: the header object, the file properties object, the stream properties object, and the two stream types
const ASF_FILE: [u8; 16] = [0xA1, 0xDC, 0xAB, 0x8C, 0x47, 0xA9, 0xCF, 0x11, 0x8E, 0xE4, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65];
const ASF_STREAM: [u8; 16] = [0x91, 0x07, 0xDC, 0xB7, 0xB7, 0xA9, 0xCF, 0x11, 0x8E, 0xE6, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65];
const ASF_AUDIO: [u8; 16] = [0x40, 0x9E, 0x69, 0xF8, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44, 0x2B];
const ASF_VIDEO: [u8; 16] = [0xC0, 0xEF, 0x19, 0xBC, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44, 0x2B];

fn asf(b: &Bytes, t: &mut Tip) -> Step<()> {//Windows Media, video and audio alike: the header object holds smaller objects, each a GUID and a 64-bit size; the file properties object has the play duration in hundreds of nanoseconds and the preroll in milliseconds, which the duration includes; a stream properties object says whether its stream is sound or picture, and a picture's size is the first two numbers of its type-specific data
	t.format = "asf";
	let end = to_usize(b.le64(16)?)?;
	let (mut at, mut visual, mut sound) = (30, false, false);//past the header object's GUID, size, count and two reserved bytes
	for _ in 0..STEPS {
		if at >= end { break }
		let guid = b.read(at, 16)?;
		let n = to_usize(b.le64(add(at, 16)?)?)?;
		if n < 24 { return Err("an object shorter than its header") }
		let data = add(at, 24)?;
		if guid == ASF_FILE {
			let (play, preroll) = (b.le64(add(data, 40)?)?, b.le64(add(data, 56)?)?);
			t.seconds = seconds(play as f64 / 1e7 - preroll as f64 / 1e3);
		} else if guid == ASF_STREAM {
			let kind = b.read(data, 16)?;
			if kind == ASF_VIDEO {
				visual = true;
				if t.width == 0 {
					let (width, height) = (b.le32(add(data, 54)?)?, b.le32(add(data, 58)?)?);//the encoded image width and height, after the type GUIDs, the time offset, two lengths, the flags and a reserved field
					if width > 0 && height > 0 { t.width = width; t.height = height }
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
		if n < 10 { return Err("a chunk shorter than its header") }
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
					if t.width == 0 && b.has(add(specific, 4)?, b"VIDO") {
						let (width, height) = (u32::from(b.be16(add(specific, 12)?)?), u32::from(b.be16(add(specific, 14)?)?));//after the size, the VIDO mark and the codec's four letters
						if width > 0 && height > 0 { t.width = width; t.height = height }
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
		else if b.has(payload, b"\x80theora") {//the picture size after the identification header's version and its frame size in macroblocks, then the frame rate as a fraction. A granule position is a keyframe's number and the count of frames since it, split at a shift the header gives, so the frame count is the two added
			t.medium = VIDEO;
			let (width, height) = (b.be24(add(payload, 14)?)?, b.be24(add(payload, 17)?)?);
			if width > 0 && height > 0 { t.width = width; t.height = height }
			let (numerator, denominator) = (b.be32(add(payload, 22)?)?, b.be32(add(payload, 26)?)?);
			if denominator == 0 { return Err("no frame rate") }
			(f64::from(numerator) / f64::from(denominator), 0, u32::from((b.be16(add(payload, 40)?)? >> 5) & 0x1F))
		}
		else { return Err("a first packet the reader does not know") };
	ask_end(b, t, "granule position at the end")?;
	let mut before = to_usize(b.size)?;
	for _ in 0..STEPS {//back from the end to the last page of the first stream, past the pages of any other, under the ceiling
		let Some(page) = b.rfind(before, b"OggS")? else { break };
		if b.u8(add(page, 4)?)? == 0 && b.le32(add(page, 14)?)? == serial {//the version byte, always 0, tells a page from the four letters inside a packet's data
			let granule = b.le64(add(page, 6)?)?;
			if granule == u64::MAX { return Err("a last page with no packet ending on it") }
			let count = if shift > 0 { (granule >> shift).saturating_add(granule & (1u64 << shift).saturating_sub(1)) } else { granule.saturating_sub(skip) };
			if rate > 0.0 { t.seconds = seconds(count as f64 / rate) }
			return Ok(())
		}
		before = page;
	}
	Err("no page of the first stream at the end")
}

//sound

fn flac(b: &Bytes, from: usize, t: &mut Tip) -> Step<()> {//the stream information block is first by rule, and eight of its bytes pack the sample rate in twenty bits, the channels and bits per sample, and the total samples in thirty-six; from is where the signature sits, the file's start or the end of an ID3 tag
	t.format = "flac"; t.medium = AUDIO;
	if b.u8(add(from, 4)?)? & 0x7F != 0 { return Err("the first metadata block is not the stream information") }
	let packed = b.be64(add(from, 18)?)?;//after the block header and the block and frame sizes
	let (rate, samples) = ((packed >> 44) as u32, packed & 0xF_FFFF_FFFF);
	if rate == 0 { return Err("no sample rate") }
	if samples == 0 { return Err("total samples not written") }
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
	Err("no frame header")
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
		ask(b, t, at, "a tag longer than the tip")?;//a cover picture in the tag can outgrow the tip, and the frames, which hold the running time, begin where the tag ends: the second slice
	}
	if b.has(at, b"fLaC") { return flac(b, at, t) }//a FLAC behind an ID3 tag, which some taggers write though the format keeps tags of its own
	if adts(b, at) { t.format = "aac"; return Err(SIGNATURE_ONLY) }//raw AAC frames; a frame count would mean reading every frame
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
	let bytes: u32 = match encoding { 1 | 2 | 27 => 1, 3 => 2, 4 => 3, 5 | 6 => 4, 7 => 8, _ => return Err("an encoding the reader does not know") };//µ-law, 8-bit linear and A-law are a byte a sample; then 16, 24 and 32-bit linear, 32-bit float and 64-bit float
	if n == u32::MAX { return Err("a data size the writer never filled in") }
	let per_second = f64::from(rate) * f64::from(channels) * f64::from(bytes);
	if per_second <= 0.0 { return Err("no sample rate") }
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
	if !(rate.is_finite() && rate > 0.0) { return Err("no sample rate") }
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
	Err("more chunks than the walk allows")
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects, clippy::panic)]//a test may index, unwrap and add: a mistake in one fails the test, which is the point of it
mod tests {
	use super::*;

	fn scout(bytes: &[u8]) -> Tip { scout_bytes(bytes, bytes.len() as u64) }//the whole file in hand, as fuji's command hands a file shorter than the tip
	fn scout_file(bytes: &[u8], tip: usize) -> Tip {//the command's two reads over a file in memory: the first tip bytes, then, when the answer asks, 64 KiB at the offset it names; so a test can hand the reader a tip shorter than the file and watch it ask
		let first = bytes.get(..tip).unwrap_or(bytes);
		let t = scout_bytes(first, bytes.len() as u64);
		if t.slice == 0 { return t }
		let at = t.slice as usize;
		let slice = bytes.get(at..(at + TIP_BYTES).min(bytes.len())).unwrap_or(&[]);
		scout_slices(first, slice, t.slice, bytes.len() as u64)
	}
	fn be32(n: u32) -> [u8; 4] { n.to_be_bytes() }

	//hand-built headers, each the few bytes a format puts its answer in, so the test says exactly what the reader read

	#[test]
	fn nothing() {
		let t = scout(&[]);
		assert_eq!((t.format, t.stop, t.text), ("", "empty", false));
	}

	#[test]
	fn text_and_binary() {
		let t = scout(b"A line of text\nand another, with a tab\tand an escape \x1b[0m\n");
		assert!(t.text && t.format.is_empty() && t.nul == 0 && t.control == 0);
		let t = scout(b"\xFF\xFEt\0e\0x\0t\0");//utf-16 behind its byte order mark
		assert!(t.text);
		let t = scout(&[0x00, 0x01, 0x02, 0xFF, 0x80, 0x41]);
		assert!(!t.text && t.format.is_empty() && t.nul == 1);
		let t = scout(b"<?xml version=\"1.0\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\"/>");
		assert_eq!((t.text, t.format, t.medium), (true, "svg", IMAGE));
		let t = scout(b"<?xml version=\"1.0\"?>\n<!-- Generator: Adobe Illustrator 24.0.0 -->\n<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\" [\n\t<!ENTITY ns_flows \"http://ns.adobe.com/Flows/1.0/\">\n]>\n<svg version=\"1.1\"/>\n");//a doctype holding declarations of its own
		assert_eq!(t.format, "svg");
		let t = scout(b"<!doctype html>\n<html><body><p>A page with a drawing.</p><svg width=\"8\" height=\"8\"></svg></body></html>\n");//a page that holds a drawing is a page
		assert_eq!((t.text, t.format), (true, ""));
		let t = scout(b"Draw it with an <svg> element.\n");
		assert_eq!((t.text, t.format), (true, ""));
		let t = scout(b"Get free shipping on every order\n");//a fifth byte that starts one of QuickTime's top boxes
		assert_eq!((t.text, t.format), (true, ""));
		let t = scout(b"PK\x03\x04and the rest of a zip");
		assert_eq!((t.format, t.medium, t.stop, t.text), ("zip", "", SIGNATURE_ONLY, false));
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
		assert_eq!((t.format, t.medium, t.width, t.height, t.animated, t.stop), ("png", IMAGE, 640, 480, false, ""));
		let mut v = png_header(2, 3);
		v.extend(be32(8)); v.extend(b"acTL"); v.extend([0; 12]);
		v.extend(be32(0)); v.extend(b"IDAT"); v.extend(be32(0));
		assert!(scout(&v).animated);
		let t = scout(&png_header(640, 480));//the whole file, ending before any picture data
		assert_eq!((t.width, t.stop, t.truncated, t.slice), (640, SHORT, true, 0));
		let t = scout_bytes(&png_header(640, 480), 1 << 20);//a tip cut from a longer file, which asks for the second slice where the chunks ran out
		assert_eq!((t.width, t.stop, t.truncated, t.slice), (640, PAST, false, 33));
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
		assert_eq!((t.format, t.width, t.height, t.orientation, t.stop), ("jpeg", 640, 480, 1, ""));
		let mut v = vec![0xFF, 0xD8];
		v.extend(exif(6)); v.extend(jpeg_frame(640, 480));
		let t = scout(&v);
		assert_eq!((t.width, t.height, t.orientation), (480, 640, 6));
	}
	#[test]
	fn jpeg_steps_over_a_thumbnail() {//an EXIF segment whose directory is followed by a whole small JPEG, frame header and all, which a search would find first
		let mut inner = vec![0xFF, 0xD8]; inner.extend(jpeg_frame(160, 120)); inner.extend([0xFF, 0xD9]);
		let mut data = b"Exif\0\0MM\0\x2A".to_vec(); data.extend(be32(8)); data.extend(0u16.to_be_bytes()); data.extend(be32(0)); data.extend(inner);
		let mut v = vec![0xFF, 0xD8]; v.extend(jpeg_segment(0xE1, &data)); v.extend(jpeg_frame(4000, 3000));
		let t = scout(&v);
		assert_eq!((t.width, t.height), (4000, 3000));
	}
	#[test]
	fn jpeg_stops_where_the_bytes_run_out_or_lie() {
		let mut v = vec![0xFF, 0xD8, 0xFF, 0xE1]; v.extend(60000u16.to_be_bytes()); v.extend([0; 100]);//a segment that claims to run far past the end
		let t = scout(&v);
		assert_eq!((t.format, t.width, t.stop, t.truncated), ("jpeg", 0, SHORT, true));
		let t = scout(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x01]);//a length shorter than its own field
		assert_eq!(t.stop, "a segment shorter than its own length field");
		let t = scout(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0x00, 0x00, 0x12]);//a segment followed by something other than a marker
		assert_eq!(t.stop, "no marker where the format puts one");
	}

	#[test]
	fn gif_bmp_webp_and_psd() {
		let mut v = b"GIF89a".to_vec(); v.extend(320u16.to_le_bytes()); v.extend(200u16.to_le_bytes()); v.extend([0x80, 0, 0]); v.extend([0; 6]);//a two-color global table
		v.extend(b"\x21\xFF\x0BNETSCAPE2.0\x03\x01\0\0\0"); v.push(0x2C);
		let t = scout(&v);
		assert_eq!((t.format, t.width, t.height, t.animated, t.stop), ("gif", 320, 200, true, ""));
		let mut v = b"BM".to_vec(); v.extend(1_000_000u32.to_le_bytes()); v.extend([0; 8]); v.extend(40u32.to_le_bytes()); v.extend(1024u32.to_le_bytes()); v.extend((-768i32).to_le_bytes()); v.extend([0; 28]);
		let t = scout(&v);
		assert_eq!((t.format, t.width, t.height, t.truncated), ("bmp", 1024, 768, true));
		let mut v = b"RIFF".to_vec(); v.extend(22u32.to_le_bytes()); v.extend(b"WEBPVP8X"); v.extend(10u32.to_le_bytes()); v.extend([0x02, 0, 0, 0]); v.extend([0x1F, 0x03, 0x00, 0x57, 0x02, 0x00]);
		let t = scout(&v);
		assert_eq!((t.format, t.width, t.height, t.animated, t.truncated), ("webp", 800, 600, true, false));
		let mut v = b"8BPS\0\x01\0\0\0\0\0\0\0\x03".to_vec(); v.extend(be32(600)); v.extend(be32(800)); v.extend([0, 8, 0, 3]);
		let t = scout(&v);
		assert_eq!((t.format, t.width, t.height), ("psd", 800, 600));
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
	#[test]
	fn mp4_reads_its_movie_box_when_it_comes_first() {
		let mut mvhd = vec![0; 12]; mvhd.extend(be32(1000)); mvhd.extend(be32(5500)); mvhd.extend([0; 80]);
		let one = 1 << 16;
		let mut moov = iso_box(b"mvhd", &mvhd);
		moov.extend(iso_box(b"trak", &tkhd(0, 0, [one, 0, 0, one])));//a sound track first
		moov.extend(iso_box(b"trak", &tkhd(1920, 1080, [0, one, -one, 0])));//then the picture, turned a quarter turn as a phone records it
		let mut v = iso_box(b"ftyp", b"mp42\0\0\0\0isommp42"); v.extend(iso_box(b"moov", &moov)); v.extend(iso_box(b"mdat", &[0; 16]));
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.width, t.height, t.orientation, t.seconds, t.stop), ("mp4", VIDEO, 1080, 1920, 6, 5.5, ""));
		let mut sound = iso_box(b"mvhd", &mvhd); sound.extend(iso_box(b"trak", &tkhd(0, 0, [one, 0, 0, one])));
		let mut v = iso_box(b"ftyp", b"isom\0\0\0\0"); v.extend(iso_box(b"moov", &sound));
		assert_eq!(scout(&v).medium, AUDIO);
	}
	#[test]
	fn mp4_with_its_index_at_the_end_names_the_stop() {
		let mut v = iso_box(b"ftyp", b"isom\0\0\0\0"); v.extend(be32(100_000)); v.extend(b"mdat"); v.extend([0; 16]);
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.seconds, t.stop, t.truncated, t.slice), ("mp4", VIDEO, 0.0, "index at the end", true, 0));//a data box that claims past the file's end: nothing to ask for
		let t = scout_bytes(&v, 1 << 30);
		assert_eq!((t.truncated, t.slice), (false, 100_016));//in a file that long, the index is asked for where the data box ends
		let v = iso_box(b"ftyp", b"heic\0\0\0\0mif1heic");
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.stop), ("heic", IMAGE, "item properties not read"));
	}

	#[test]
	fn wav_and_flac_time_themselves() {
		let mut v = b"RIFF".to_vec(); v.extend(36u32.to_le_bytes()); v.extend(b"WAVEfmt "); v.extend(16u32.to_le_bytes()); v.extend([1, 0, 1, 0]); v.extend(8000u32.to_le_bytes()); v.extend(16000u32.to_le_bytes()); v.extend([2, 0, 16, 0]); v.extend(b"data"); v.extend(32000u32.to_le_bytes());
		let t = scout_bytes(&v, 32044);
		assert_eq!((t.format, t.medium, t.seconds, t.truncated), ("wav", AUDIO, 2.0, false));
		assert!(scout(&v).truncated);//the data chunk claims more than the file holds
		let mut unfilled = v.clone(); let n = unfilled.len(); unfilled[n - 4..].copy_from_slice(&u32::MAX.to_le_bytes());//a writer that never came back to fill in the size
		let t = scout(&unfilled);
		assert_eq!((t.stop, t.truncated), ("a data size the writer never filled in", false));
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
		let t = scout_bytes(&v, 20 + 160_000);
		assert!((t.seconds - 10.0).abs() < 0.01);
		let mut v = b"ID3\x04\0\0\0\0\0\x0A\0\0\0\0\0\0\0\0\0\0".to_vec();//a tag, then two raw AAC frames of 16 bytes, the second where the first's length says
		v.extend([0xFF, 0xF1, 0x50, 0x80, 0x02, 0x1F, 0xFC]); v.resize(20 + 16, 0); v.extend([0xFF, 0xF1, 0x50, 0x80, 0x02, 0x1F, 0xFC]); v.resize(20 + 32, 0);
		let t = scout(&v);
		assert_eq!((t.format, t.stop), ("aac", SIGNATURE_ONLY));
		let t = scout(b"\xFF\xFEt\0e\0x\0t\0 \0i\0n\0 \0u\0t\0f\0-\x001\x006\0");//a byte order mark that is also a valid frame header, with no second frame behind it
		assert_eq!((t.format, t.text), ("", true));
		let mut v = b"ID3\x04\0\0\0\x08\0\0".to_vec(); v.resize(TIP_BYTES, 0);//a tag of 128 KiB, the size a cover picture makes one
		let t = scout_bytes(&v, 1 << 20);
		assert_eq!((t.format, t.stop, t.truncated, t.slice), ("mp3", "a tag longer than the tip", false, 131_082));
		assert!(scout(&v).truncated);//and longer than the file
	}

	//the sample files in the samples folder, made by real encoders on the mac mini on 2026-10-17 with ImageMagick 7.1, exiftool 13 and ffmpeg 8: every picture a 32 by 20 red-to-blue gradient, the eight orientations written by exiftool onto one JPEG and a 16 by 10 thumbnail embedded in another, every video a one-second test pattern at ten frames a second, the turned ones with ffmpeg's -noautorotate -display_rotation, which counts counter-clockwise, and every sound a one-second tone, at 8 kHz for the uncompressed kinds and 44.1 for the rest; video.vob, a DVD's program stream, on 2026-10-18 with ffmpeg 9. Each row is what the maker was told to make, checked against ffprobe, exiftool and the files' own bytes, and the test pins the reader to it

	fn sample(name: &str) -> Vec<u8> { std::fs::read(format!("{}/samples/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap_or_else(|e| panic!("sample {name}: {e}")) }
	fn scout_sample(name: &str) -> Tip { scout_file(&sample(name), TIP_BYTES) }

	#[test]
	fn samples_answer_as_their_makers_made_them() {
		type Row = (&'static str, &'static str, &'static str, (u32, u32), u32, f64, bool, &'static str);//the file; its format and medium; the size it shows at; its orientation; its running time, matched within a twentieth of a second and 0 for none; whether it animates; and the stop
		let rows: &[Row] = &[
			("picture.png", "png", IMAGE, (32, 20), 1, 0.0, false, ""),
			("picture.gif", "gif", IMAGE, (32, 20), 1, 0.0, false, ""),
			("animated.gif", "gif", IMAGE, (32, 20), 1, 0.0, true, ""),
			("picture.bmp", "bmp", IMAGE, (32, 20), 1, 0.0, false, ""),
			("picture.jpg", "jpeg", IMAGE, (32, 20), 1, 0.0, false, ""),
			("progressive.jpg", "jpeg", IMAGE, (32, 20), 1, 0.0, false, ""),
			("orient-1.jpg", "jpeg", IMAGE, (32, 20), 1, 0.0, false, ""),
			("orient-2.jpg", "jpeg", IMAGE, (32, 20), 2, 0.0, false, ""),
			("orient-3.jpg", "jpeg", IMAGE, (32, 20), 3, 0.0, false, ""),
			("orient-4.jpg", "jpeg", IMAGE, (32, 20), 4, 0.0, false, ""),
			("orient-5.jpg", "jpeg", IMAGE, (20, 32), 5, 0.0, false, ""),
			("orient-6.jpg", "jpeg", IMAGE, (20, 32), 6, 0.0, false, ""),
			("orient-7.jpg", "jpeg", IMAGE, (20, 32), 7, 0.0, false, ""),
			("orient-8.jpg", "jpeg", IMAGE, (20, 32), 8, 0.0, false, ""),
			("thumbnail.jpg", "jpeg", IMAGE, (20, 32), 6, 0.0, false, ""),//turned, with a 16 by 10 picture inside its EXIF segment that a search would have sized it by
			("picture.webp", "webp", IMAGE, (32, 20), 1, 0.0, false, ""),
			("lossless.webp", "webp", IMAGE, (32, 20), 1, 0.0, false, ""),
			("animated.webp", "webp", IMAGE, (32, 20), 1, 0.0, true, ""),
			("picture.tif", "tiff", IMAGE, (32, 20), 1, 0.0, false, ""),
			("picture.psd", "psd", IMAGE, (32, 20), 1, 0.0, false, ""),
			("picture.heic", "heic", IMAGE, (0, 0), 1, 0.0, false, "item properties not read"),
			("picture.avif", "avif", IMAGE, (0, 0), 1, 0.0, false, "item properties not read"),
			("picture.svg", "svg", IMAGE, (0, 0), 1, 0.0, false, ""),
			("moov-first.mp4", "mp4", VIDEO, (32, 20), 1, 1.0, false, ""),
			("moov-last.mp4", "mp4", VIDEO, (32, 20), 1, 1.0, false, ""),//the index after the data, as a phone writes it, in hand since the file is small; the second slice reads it in the test below
			("ccw-90.mp4", "mp4", VIDEO, (20, 32), 8, 1.0, false, ""),//a quarter turn counter-clockwise, which is EXIF's 8
			("ccw-180.mp4", "mp4", VIDEO, (32, 20), 3, 1.0, false, ""),
			("ccw-270.mp4", "mp4", VIDEO, (20, 32), 6, 1.0, false, ""),//and three quarters counter-clockwise is a quarter clockwise, EXIF's 6, the turn a phone held upright records
			("video.mov", "mov", VIDEO, (32, 20), 1, 1.0, false, ""),
			("audio.m4a", "m4a", AUDIO, (0, 0), 1, 1.0, false, ""),
			("video.avi", "avi", VIDEO, (32, 20), 1, 1.0, false, ""),
			("video.mpg", "mpeg", VIDEO, (32, 20), 1, 0.58, false, ""),//an MPEG-1 program stream of three packs, the last clocked at 52,201 ticks of 90 kHz: the running time to the last pack, which is what the reader has, short of the last frame's end
			("video.vob", "mpeg", VIDEO, (32, 20), 1, 0.5, false, ""),//MPEG-2, whose pack writes its clock the other way, two packs and the last at 45,001
			("video.ts", "mpegts", VIDEO, (0, 0), 1, 0.0, false, SIGNATURE_ONLY),
			("video.mkv", "matroska", VIDEO, (32, 20), 1, 1.0, false, ""),
			("video.webm", "webm", VIDEO, (32, 20), 1, 1.0, false, ""),
			("video.flv", "flv", VIDEO, (32, 20), 1, 1.0, false, ""),
			("video.wmv", "asf", VIDEO, (32, 20), 1, 1.0, false, ""),
			("video.rm", "realmedia", VIDEO, (32, 20), 1, 1.0, false, ""),
			("audio.wav", "wav", AUDIO, (0, 0), 1, 1.0, false, ""),
			("audio.aiff", "aiff", AUDIO, (0, 0), 1, 1.0, false, ""),
			("audio.au", "au", AUDIO, (0, 0), 1, 1.0, false, ""),
			("audio.flac", "flac", AUDIO, (0, 0), 1, 1.0, false, ""),
			("cbr.mp3", "mp3", AUDIO, (0, 0), 1, 1.0, false, ""),
			("plain.mp3", "mp3", AUDIO, (0, 0), 1, 1.04, false, ""),//no Xing header and no tag, so the running time is the size over the bit rate, which ffprobe also answers 1.044875 for
			("vbr.mp3", "mp3", AUDIO, (0, 0), 1, 1.0, false, ""),
			("audio.aac", "aac", AUDIO, (0, 0), 1, 0.0, false, SIGNATURE_ONLY),
			("audio.ogg", "ogg", AUDIO, (0, 0), 1, 1.0, false, ""),//the last page's granule position over the rate, 44,160 samples at 44.1 kHz
			("audio.opus", "ogg", AUDIO, (0, 0), 1, 1.0, false, ""),//48,312 at 48 kHz, less a pre-skip of 312
			("audio.wma", "asf", AUDIO, (0, 0), 1, 1.02, false, ""),
			("text.txt", "", "", (0, 0), 1, 0.0, false, ""),
			("page.html", "", "", (0, 0), 1, 0.0, false, ""),
			("binary.bin", "", "", (0, 0), 1, 0.0, false, ""),
			("archive.zip", "zip", "", (0, 0), 1, 0.0, false, SIGNATURE_ONLY),
			("empty", "", "", (0, 0), 1, 0.0, false, "empty"),
		];
		for &(name, format, medium, (width, height), orientation, secs, animated, stop) in rows {
			let t = scout_sample(name);
			assert_eq!((name, t.format, t.medium, t.width, t.height, t.orientation, t.animated, t.stop), (name, format, medium, width, height, orientation, animated, stop));
			assert!((t.seconds - secs).abs() < 0.05, "{name}: {} seconds against {secs}", t.seconds);
			assert!(!t.truncated, "{name} read as truncated");
		}
		for name in ["text.txt", "page.html", "picture.svg"] { assert!(scout_sample(name).text, "{name} should read as text") }
		for name in ["binary.bin", "archive.zip", "picture.png", "cbr.mp3", "empty"] { assert!(!scout_sample(name).text, "{name} should not read as text") }
	}

	#[test]
	fn samples_changed_the_way_other_writers_leave_them() {//the matroska sample with its segment and clusters of unknown size, as a live recording leaves them, and the flash video's metadata with a value the reader cannot step over after the size
		let mut v = sample("video.mkv");
		let (segment, cluster) = {
			let b = Bytes { tip: &v, slice: &[], slice_at: 0, size: v.len() as u64, depth: Cell::new(0), slice_depth: Cell::new(0), missed: Cell::new(0) };
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
		assert_eq!((t.format, t.width, t.height, t.seconds, t.stop), ("matroska", 32, 20, 1.0, ""));
		let mut v = sample("video.flv");
		let at = v.windows(7).position(|w| w == b"encoder").unwrap() + 7;//the value's type, after its name
		v[at] = 3;//an object, as a keyframe index is
		let t = scout(&v);
		assert_eq!((t.format, t.width, t.height, t.stop), ("flv", 32, 20, "a metadata value the reader does not step over"));
	}

	#[test]
	fn ogg_theora_counts_frames_from_its_granule() {//hand-built, since the mac mini's ffmpeg has no Theora encoder: an identification header for a 32 by 20 picture at ten frames a second with a granule shift of 6, and a last page whose granule says keyframe 8 and two frames since, which is ten frames
		fn page(kind: u8, granule: u64, payload: &[u8]) -> Vec<u8> { let mut v = b"OggS\0".to_vec(); v.push(kind); v.extend(granule.to_le_bytes()); v.extend(7u32.to_le_bytes()); v.extend([0; 8]); v.push(1); v.push(payload.len() as u8); v.extend(payload); v }
		let mut header = b"\x80theora\x03\x02\x01".to_vec(); header.extend([0, 2, 0, 2]); header.extend([0, 0, 32, 0, 0, 20, 0, 0]); header.extend(be32(10)); header.extend(be32(1)); header.extend([0; 10]); header.extend((6u16 << 5).to_be_bytes());
		let mut v = page(2, 0, &header); v.extend(page(4, (8 << 6) | 2, &[0]));
		let t = scout(&v);
		assert_eq!((t.format, t.medium, t.width, t.height, t.seconds, t.stop), ("ogg", VIDEO, 32, 20, 1.0, ""));
	}

	#[test]
	fn the_second_slice_reaches_what_the_tip_did_not() {//files larger than the tip, built around the samples, one for each way a reader asks: the first answer names the slice and the stop, and the second answer has what the sample's has
		let pad = 200_000;//past the tip and past twice it, so the last 64 KiB never overlap the tip
		let first = |v: &[u8]| scout_bytes(&v[..TIP_BYTES], v.len() as u64);
		let mut v = vec![0xFF, 0xD8]; v.extend(jpeg_segment(0xE2, &vec![0; 65533])); v.extend(&sample("picture.jpg")[2..]);//a segment as long as the format allows, a color profile say, before the picture's own
		let t = first(&v);
		assert_eq!((t.format, t.width, t.stop, t.slice), ("jpeg", 0, PAST, 65_539));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.width, t.height, t.stop, t.slice), (32, 20, "", 65_539));
		assert!(t.slice_depth > 0);
		let mut v = vec![0xFF, 0xD8]; v.extend(jpeg_segment(0xE2, &vec![0; 65533])); v.extend(jpeg_segment(0xE2, &vec![0; 65533])); v.extend(&sample("picture.jpg")[2..]);//two of them: the second slice runs out too, and there is no third
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.width, t.stop, t.slice), (0, PAST, 65_539));
		let m = sample("moov-last.mp4");
		let boxes: Vec<&[u8]> = { let mut v = vec![]; let mut at = 0; while at < m.len() { let n = u32::from_be_bytes(m[at..at + 4].try_into().unwrap()) as usize; v.push(&m[at..at + n]); at += n } v };
		let find = |kind: &[u8]| boxes.iter().find(|b| &b[4..8] == kind).unwrap().to_vec();
		let mut v = find(b"ftyp"); v.extend(be32(pad as u32 + 8)); v.extend(b"mdat"); v.resize(v.len() + pad, 0); v.extend(find(b"moov"));//the sample's data box grown past the tip, its movie box still behind it
		let end = (find(b"ftyp").len() + 8 + pad) as u64;
		let t = first(&v);
		assert_eq!((t.format, t.seconds, t.stop, t.slice), ("mp4", 0.0, "index at the end", end));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.width, t.height, t.seconds, t.stop, t.slice), (32, 20, 1.0, "", end));
		let mut v = b"ID3\x04\0\0".to_vec(); v.extend([(pad >> 21) as u8 & 0x7F, (pad >> 14) as u8 & 0x7F, (pad >> 7) as u8 & 0x7F, pad as u8 & 0x7F]); v.resize(10 + pad, 0); v.extend(sample("plain.mp3"));//a tag the size of a cover picture, then the frames
		let t = first(&v);
		assert_eq!((t.format, t.seconds, t.stop, t.slice), ("mp3", 0.0, "a tag longer than the tip", 10 + pad as u64));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.stop, t.slice), ("", 10 + pad as u64));
		assert!((t.seconds - 1.04).abs() < 0.05, "{}", t.seconds);
		let mut v = b"RIFF\0\0\0\0WAVEfmt ".to_vec(); v.extend(16u32.to_le_bytes()); v.extend([1, 0, 1, 0]); v.extend(8000u32.to_le_bytes()); v.extend(16000u32.to_le_bytes()); v.extend([2, 0, 16, 0]);
		v.extend(b"LIST"); v.extend((pad as u32).to_le_bytes()); v.resize(v.len() + pad, 0); let data = v.len() as u64; v.extend(b"data"); v.extend(32000u32.to_le_bytes()); v.resize(v.len() + 32000, 0);//a chunk the size of a picture before the sound
		let n = v.len() as u32 - 8; v[4..8].copy_from_slice(&n.to_le_bytes());
		let t = first(&v);
		assert_eq!((t.format, t.seconds, t.stop, t.slice), ("wav", 0.0, PAST, data));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.seconds, t.stop, t.truncated), (2.0, "", false));
		let mut v = b"II*\0".to_vec(); v.extend((pad as u32).to_le_bytes()); v.resize(pad, 0);//the directory at the end of the file, as libtiff writes it
		v.extend(2u16.to_le_bytes()); for (tag, value) in [(0x0100u16, 32u32), (0x0101, 20)] { v.extend(tag.to_le_bytes()); v.extend(3u16.to_le_bytes()); v.extend(1u32.to_le_bytes()); v.extend(value.to_le_bytes()) } v.extend(0u32.to_le_bytes());
		let t = first(&v);
		assert_eq!((t.format, t.width, t.stop, t.slice), ("tiff", 0, "first directory past the tip", pad as u64));
		let t = scout_file(&v, TIP_BYTES);
		assert_eq!((t.width, t.height, t.stop), (32, 20, ""));
		let o = sample("audio.ogg"); let last = o.windows(4).rposition(|w| w == b"OggS").unwrap();
		let mut v = o.clone(); v.resize(o.len() + pad, 0); v.extend(&o[last..]);//the sample, then silence, then its last page again at the end
		let t = first(&v);
		assert_eq!((t.format, t.medium, t.seconds, t.stop, t.slice), ("ogg", AUDIO, 0.0, "granule position at the end", (v.len() - TIP_BYTES) as u64));
		let t = scout_file(&v, TIP_BYTES);
		assert!((t.seconds - 1.0).abs() < 0.05 && t.stop.is_empty(), "{} {}", t.seconds, t.stop);
		let mut f = sample("video.flv"); let key = f.windows(8).position(|w| w == b"duration").unwrap(); f[key] = b'x';//the running time in the metadata renamed, so the reader has to go to the end for it
		let t = scout(&f);
		assert!((t.seconds - 0.9).abs() < 0.01 && t.stop.is_empty(), "{} {}", t.seconds, t.stop);//the last tag's timestamp, in hand since the file is small
		let last = f.len() - 4 - u32::from_be_bytes(f[f.len() - 4..].try_into().unwrap()) as usize;
		let mut v = f[..last].to_vec(); v.resize(last + pad, 0); v.extend(&f[last..]);
		let t = first(&v);
		assert_eq!((t.format, t.width, t.seconds, t.stop, t.slice), ("flv", 32, 0.0, "running time at the end", (v.len() - TIP_BYTES) as u64));
		let t = scout_file(&v, TIP_BYTES);
		assert!((t.seconds - 0.9).abs() < 0.01 && t.stop.is_empty(), "{} {}", t.seconds, t.stop);
		let p = sample("video.mpg"); let last = p.windows(4).rposition(|w| w == [0, 0, 1, 0xBA]).unwrap();
		let mut v = p.clone(); v.resize(p.len() + pad, 0); v.extend(&p[last..]);//the sample, then a gap, then its last pack again at the end
		let t = first(&v);
		assert_eq!((t.format, t.width, t.seconds, t.stop, t.slice), ("mpeg", 32, 0.0, "clock reference at the end", (v.len() - TIP_BYTES) as u64));
		let t = scout_file(&v, TIP_BYTES);
		assert!((t.seconds - 0.58).abs() < 0.01 && t.stop.is_empty(), "{} {}", t.seconds, t.stop);
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
				assert_eq!((&name, cut, t.format, t.medium, t.width, t.height, t.orientation, t.animated, t.truncated, t.stop), (&name, cut, whole.format, whole.medium, whole.width, whole.height, whole.orientation, whole.animated, whole.truncated, whole.stop));
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
