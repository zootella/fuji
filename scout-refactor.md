# Scout refactor

## How it works now

- **Two entries.** `scout_bytes(tip, size)` answers from a file's first 64 KiB and its size. `scout_slices(tip, slice, at, size)` answers again from the tip and one more 64 KiB slice that begins at `at`. Fuji's command `scout_tip` in `desktop/src-tauri/src/scout.rs` reads the tip and calls `scout_bytes`. When the answer's `slice` is above 0, it reads 64 KiB there and calls `scout_slices`, and that is the last read.
- **One flat answer, with snake_case keys.** `text`, `format`, `medium`, `width`, `height`, `orientation`, `seconds`, `animated`, `truncated`, `depth`, `stop`, `slice`, `slice_depth`, `nul`, `control`, `printable` and `high` are in every answer, at 0, false or blank where the file does not say. What the file holds and how the reading went sit side by side, so `depth` sits beside `width`.
- **`slice` is two things.** After the first read it is the request for the second, and 0 asks for none. After the second read it is where that read began.
- **`stop` is one of about 75 reasons in prose**, blank when the reader finished. Some reasons are the scout's own limits: "not in the slices", "signature only", "item properties not read". Others are findings about the file: "past the end of the file", "no marker where the format puts one", "total samples not written". A caller can tell them apart only by the words. When the place a reader asked for lies at or past the file's end, the request's words stay in the final answer, so an MP4 with no index after its data answers "index at the end".
- **`truncated`** is true when a length or an offset in the file points past its end, or when the file ends inside the structure the reader was walking. It can be true when the reader finished, as for a WAV whose data chunk claims more than the file holds.
- **`text`** is judged only when no signature matched. So false covers a known format, an empty file and binary bytes alike, and a text file that opens with a signature's letters is named as that format.
- **`width` and `height`** are the size a picture or a video shows at, with its turn applied. For pictures, and for every video container but MP4 and MOV, they are the stored pixels. MP4 and MOV give the track header's size, which is the shown size. The scout reads no pixel shape anywhere.
- **`depth`** is the farthest byte of the tip the reader looked at. For a file that matched a signature it includes the probes for other formats' signatures, so a zip answers 19, from the probe for VOC's 19-byte signature. For a file that matched none, `scout()` sets it back to 0 before the text verdict, so it counts only the search for an SVG's first element.
- **The readers write into the answer as they go.** Each reader sets fields on the answer it is handed, and `ask` and `ask_end` put their request into its `slice`. Every stop is a string, and `scout()` turns the last one into `stop` and, after the first read, into the request.
- **Three readers turn the size.** JPEG, TIFF and MP4's track reader each call `shown()` after reading the orientation. `shown()` swaps the sides for orientations 5 through 8, by whatever orientation is set at the moment it runs.

A phone's video, stored at 32 by 20 with a quarter turn, `samples/ccw-270.mp4`:

```json
{
  "text": false,
  "format": "mp4",
  "medium": "video",
  "width": 20,
  "height": 32,
  "orientation": 6,
  "seconds": 1.0,
  "animated": false,
  "truncated": false,
  "depth": 830,
  "stop": "",
  "slice": 0,
  "slice_depth": 0,
  "nul": 601,
  "control": 376,
  "printable": 1118,
  "high": 1191
}
```

## How it works after the refactor

- **Three kinds of information, kept apart.** What the file holds sits at the top of the answer. How the reading went sits in `reading`. What the scout needs next is no part of the answer at all.
- **The request comes back beside the answer.** `scout_bytes` returns the answer and, separately, either the offset of the one more 64 KiB slice it wants or nothing. `scout_slices` returns the answer alone, since there is never a third read. The command acts on the request, so the page never sees one.
- **Readers record what they find, stored and unturned.** No reader turns a size, and no reader writes a request into the answer.
- **Every stop is one of four kinds, beside its few words.** A format named by its signature and read no further, which is not parsed and no trouble. The scout's own limit, which is trouble. The file corrupted, and the file chopped, each of which is trouble with that finding.
- **A walk that reaches its ceiling of steps is the scout's own limit**, since a ceiling is no evidence that the file breaks a rule. An offset sum that overflows is the file corrupted.
- **A request for a place at or past the file's end is no request.** The file ends before the part its structure points to, so the answer is chopped, with the request's words as its details. An MP4 whose data box ends the file, with no index after it, is one.
- **One step at the end turns both sizes.** At the end of `scout()`, one function copies `raster` into `aspect` when the file gave no shape, then swaps both together for orientations 5 through 8. So the two can never disagree about the turn.
- **The readers' safety is untouched.** No walk, read, offset sum, ceiling or slice size changes, and the lints, the ban on unsafe code and the fuzzer stand as they are. What changes is where an answer is written and how a stop is labeled.
- **`format` and `medium`** stay as they are.
- **`text` and `binary`.** `text` says the scout is very certain the first 64 KiB are characters, and `binary` that it is very certain they are not. Both are judged on every file from the four counts alone, apart from every format's reader and signature, so no format's pattern ever reaches them. `text` is true when the bytes open with a UTF-16 byte order mark, whose text is half zero bytes, or else hold no zero byte and no more than one control character in a hundred bytes. `binary` is true when they hold a zero byte and open with no such mark. They are never both true. Both are false for an empty file, and for bytes with no zero byte but too many control characters to be text, which real files rarely are. A format and these two can disagree: a text file that opens with `ID3` answers `format` mp3 and `text` true, and the caller sees both.
- **`raster`** is the stored pixel counts, `{width, height}`, turned. MP4 and MOV give the track header's shown size, which equals the stored count for square pixels.
- **`aspect`** is the picture's shape as the file writes it, `{width, height}`, turned with `raster`. Its numbers are whatever the file wrote, 4 and 3 in one format and 853 and 480 in another, since fuji above uses them only in arithmetic and never as a size. When the file gives no shape, `aspect` copies `raster`, so its two numbers are nonzero whenever `raster`'s are. In this refactor it copies `raster` for every file. Fuji above works out the shown size from the two, growing the short edge and rounding down, as mpv's `mp_image_params_get_dsize` does.
- **`orientation`** is a finding: the file stores an instruction to turn or flip its picture, in EXIF's numbering, 1 for none through 8. The scout has already carried that instruction out on `raster` and `aspect`, so a caller ignores `orientation` and uses those two as they are; turning them again would swap them back. It is there only for a caller that wants to know the file is one an older program, one that ignores the instruction, would show turned. The comment on the field, the README and `scout.js` all say this loudly.
- **`seconds` and `animated`** stay as they are.
- **`corrupted`**: the scout found positive evidence of a format, then evidence that the file breaks that format's rules where the reader needed them. A broken part the reader steps past without needing it, like a JPEG's EXIF or an AVI's OpenDML header, costs only what it held and is not reported.
- **`chopped`**: the scout found positive evidence of a format, then evidence that the file's end is cut off. A length or an offset points past the end, or the file ends inside the structure the reader was walking. Downloads are often chopped, especially from the years when a browser wrote to the final name as it went. `chopped` can come with a finished reading.
- **False on both** means no such evidence: an unknown file, a file the scout did not parse, or one damaged in a way the scout cannot see. It never says the file is good.
- **`reading.parsed`**: the scout has a reader for this format and ran it past the signature. It is false for a format the scout names by its signature alone, for bytes it does not recognize, and for an empty file. It says nothing about trouble: a JPEG cut off at 100 bytes is parsed, with trouble.
- **`reading.trouble`**: the reader stopped before it had every answer it reads for this format. Either it reached its own limit, past its slices or in a structure it does not read, or the file was at fault, which `corrupted` or `chopped` then names. `corrupted` always comes with `trouble`; `chopped` can come without it.
- **`reading.details`**: a few words on the condition, why the reader stopped or why it did not parse, blank when it parsed with no trouble.
- **`reading.reads`**: 1 or 2.
- **`reading.depth`**: the farthest byte of the first 64 KiB the reader looked at, signature probes included, for every file, with a format or without. With `reads`, it is what a run over a real collection reads to learn whether a smaller first read would do, and how often a second read is needed.
- **`reading.counts`**: `nul`, `control`, `printable` and `high` over the first 64 KiB, the evidence behind `text` and `binary`.
- **Every key is a single word**, so the answer is spelled the same in Rust and on the page.
- **A caller reads three things.** Good as far as the scout can tell: parsed, no trouble, and neither corrupted nor chopped. Hand it on to the next tier: trouble with neither, or a format named but not parsed. Damaged: corrupted or chopped.
- **Tests.** The samples table pins `raster`, `aspect`, `text`, `binary`, `corrupted`, `chopped`, `parsed`, `trouble` and `details` for every sample, beside what it pins today. A hand-built test hands the turning step a raster, an aspect that differs from it and orientation 6, and checks that both swap.
- **Callers.** The fuzz target and the tests' two-read helper take the request from beside the answer. `scout_tip` reads the second slice when the request asks for it, and `scoutTip` in `desktop/src/scout.js` describes the new answer. The README's section on the answer and the comments on the answer's fields are rewritten once the code works.

The same video after the refactor:

```json
{
  "format": "mp4",
  "medium": "video",
  "text": false,
  "binary": true,
  "raster": {
    "width": 20,
    "height": 32
  },
  "aspect": {
    "width": 20,
    "height": 32
  },
  "orientation": 6,
  "seconds": 1.0,
  "animated": false,
  "corrupted": false,
  "chopped": false,
  "reading": {
    "parsed": true,
    "trouble": false,
    "details": "",
    "reads": 1,
    "depth": 830,
    "counts": {
      "nul": 601,
      "control": 376,
      "printable": 1118,
      "high": 1191
    }
  }
}
```

A JPEG cut off at 100 bytes, `samples/picture.jpg` shortened:

```json
{
  "format": "jpeg",
  "medium": "image",
  "text": false,
  "binary": true,
  "raster": {
    "width": 0,
    "height": 0
  },
  "aspect": {
    "width": 0,
    "height": 0
  },
  "orientation": 1,
  "seconds": 0.0,
  "animated": false,
  "corrupted": false,
  "chopped": true,
  "reading": {
    "parsed": true,
    "trouble": true,
    "details": "past the end of the file",
    "reads": 1,
    "depth": 93,
    "counts": {
      "nul": 10,
      "control": 46,
      "printable": 36,
      "high": 8
    }
  }
}
```

## The shape, a later step (nicknamed RectanglePixels)

- **MPEG-2's aspect code**, the four bits after the size. 4:3, 16:9 and 2.21:1 are copied, and its code for square pixels copies `raster`.
- **Matroska's display width and height**, in the same video element as the pixel size, copied whatever their display unit.
- **Theora's and MPEG-1's pixel shape**, multiplied once into the picture's: the raster width times the pixel's width, against the raster height times the pixel's height. MPEG-1's table of fourteen decimals, which FFmpeg keeps as `ff_mpeg1_aspect`, becomes whole-number pairs.
- **MP4's stored count**, in the sample description deeper in the track, which makes its `raster` the stored count rather than the shown size.
- **AVI, Windows Media, FLV and RealMedia** keep no shape in anything the scout reads, so their `aspect` copies `raster`.
- **Each format gets sample files and tests of its own.**
