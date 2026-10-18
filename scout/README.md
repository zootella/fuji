# Scout

Fuji's own reader of a file's first bytes. Given the first 64 KiB of a file and the file's size, it says what format the bytes are and what the common formats keep in their header: the size a picture or a video shows at, its turn, the running time of a sound or a video. When the first bytes say where the rest of an answer lies, a phone's video index after its data or an Ogg's last page, it asks for one more 64 KiB slice at that offset and answers again from both: two reads at most, and never a search of the file. It reads no pixels and no samples.

It is a Rust crate of its own, so it builds and tests here with nothing of Fuji around it. Fuji's Rust depends on it by path, and nothing publishes it.

```
pnpm test    run the reader's tests, pinned to the files in samples/
pnpm fuzz    run libFuzzer on the reader, seeded from samples/, until stopped
```

`pnpm fuzz -- -max_total_time=60` runs it for a minute. Fuzzing needs the nightly toolchain and cargo-fuzz, `rustup toolchain install nightly` and `cargo install cargo-fuzz`.

## The files

```
src/lib.rs    the reader, with its essay at the top and its tests at the end
samples/      small files made by real encoders, each pinned by a test to the answer it must give
fuzz/         the crate cargo-fuzz runs, which keeps what it finds in fuzz/corpus and fuzz/artifacts, both gitignored
```

## How Fuji uses it

`desktop/src-tauri/src/scout.rs` is the command the page calls with a path. It reads the start of the file, and the second slice when the reader asks for one, and hands them to the reader here; `desktop/src/scout.js` wraps the command for the page.

## The answer

The reader answers every file with the same fields, and every field is always there: a value the file does not give, or the reader did not reach, is 0, false or blank. The fields fall into two groups, what the file holds and how the reading went.

### Findings from the file

- `format` is the scout's own word for the format the first bytes announce, `jpeg`, `mp4`, `mp3` and so on, or blank when they announce none it knows. It names the container rather than the codec, so a WMV and a WMA are both `asf`, and Vorbis, Opus and Theora are all `ogg`. The scout sets it the moment a signature matches, before it checks anything else, so `stop` says how much the reader confirmed: blank when it walked the format's structure to its answers, `signature only` for a format the scout names and reads no further into, and any other reason when the reading broke off part way. A text file that opens with a signature's letters, `ID3` say, is named as that format.
- `medium` is `image`, `video` or `audio`, or blank for a format that is none of the three, like a zip or a PDF, and for text and unknown bytes. A container's medium comes from its tracks when the reader reaches them, and otherwise from its signature or brand, which can say `video` for a file that holds only sound, or nothing at all.
- `width` and `height` are the size the picture or the video shows at, in pixels, with its turn applied, so a phone's portrait comes back taller than it is wide. For a picture they are the stored pixels, with the sides swapped when `orientation` is 5 through 8. For a video they are the frame's stored pixels in every container but MP4 and MOV, which give the track header's size: the same for square pixels, and the shown shape for pixels that are not square, as a DVD's are. They are 0 for sound, for SVG, for HEIC and AVIF, whose size the scout does not read, and wherever the reader stopped before them.
- `orientation` is the turn, in EXIF's numbering, 1 for upright through 8. The scout reads it from a JPEG's EXIF, a TIFF's first directory, and an MP4 or MOV track's matrix, where only the three quarter turns a phone records count, 3, 6 and 8. Every other file answers 1. `width` and `height` already have it applied, and it says which turn, for whatever draws the pixels.
- `seconds` is the running time of a sound or a video, in seconds and their fractions, to whatever precision the format records, converted from the format's own units: a timescale's ticks, nanoseconds, a 90 kHz clock, samples over a rate. It is 0 for a picture, an animated one included, and where the file does not say or the reader stopped first. Four cases come close rather than exact. An MP3 with no Xing header is timed from its size and its first frame's bit rate, which is exact only at a constant bit rate. An MP3 timed from its frame count includes the encoder's padding, about 45 milliseconds. An MPEG program stream is timed to its last pack's clock, short of its last frame's end. And an FLV with no running time in its metadata is timed by its last tag, which is when its last frame starts.
- `animated` is true when the format's own flag says the picture moves: a GIF's looping extension, an APNG's animation control chunk, an animated WebP's flag, or the sequence brand of an AVIF or a HEIF file. It is false for a still and wherever the reader could not look, and a GIF that plays once, with no looping extension, also reads false.
- `truncated` is true when a header claims more bytes than the file holds, or the file ends inside the structure the reader was walking: an unfinished download, or a file cut short. It stands apart from `stop`, since the reader can finish and still find the file short, and then `width`, `height` and `seconds` are what the header promised rather than what the file holds. False says only that nothing the reader read claimed more than the file has, and not that the file is whole: an MP4 with its index first, cut in half, answers its full size and running time with `truncated` false, since the reader stops at the index.
- `text` is true when no format the scout knows claims the bytes and the first 64 KiB read as text, by a rule over the counts below: a UTF-16 byte order mark, or else no zero byte at all and no more than one control character in a hundred bytes. The rule judges and does not prove, and it sees only the first 64 KiB. False does not mean binary. The scout judges text only when no signature matched, so false is one of three things: a format matched, and `format` names it; the file is empty, and `stop` says `empty`; or the bytes look binary, which is `format` blank, `text` false and `stop` blank together. An SVG is the one answer with both, `text` true and `format` `svg`. UTF-16 with no byte order mark reads as binary, since half its bytes are zero.
- `nul`, `control`, `printable` and `high` count four kinds of byte across the first 64 KiB, whatever the format: zero bytes; control characters other than text's whitespace and escape; printable ASCII with that whitespace and escape; and bytes past ASCII, which UTF-8 text and every binary file have. `text` is judged from them, and since they are in every answer, a caller can judge text by a rule of its own.

### The scout's report on its reading

- `stop` is blank when the reader finished, with every answer it reads for this format in hand, and otherwise the reason it stopped, in the scout's own words. The findings above hold whatever it read before the stop. The reasons fall on both sides of the line between the file and the scout. Some are the scout's own limits: the answer lies past what it read, `not in the slices`, or in a structure or a variant it does not read, `signature only` or `item properties not read`. Others are findings about the file: it ends early, `past the end of the file`, which also sets `truncated`; it breaks its format's rules, or is not the format its signature claims, `no marker where the format puts one`; or it is sound but lacks the answer, `total samples not written`. The walk's ceiling, `more boxes than the walk allows`, can be either, a strange file or a hostile one.
- `depth` is how far into the first 64 KiB the reader read, in bytes, so a run over many files says whether a smaller read would have done.
- `slice` is where the second 64 KiB began, as an offset into the file, and 0 when the scout read only the first. In what the command hands the page it always names a read that happened; an answer from `scout_bytes` alone, the first of the two reads, uses it to ask for that read.
- `slice_depth` is how far into the second slice the reader read, in bytes, and 0 when there was none.
