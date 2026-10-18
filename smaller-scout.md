# Every reader stays, 794 lines

The scout keeps the width the first pass gave it. No rare pattern costs more than about 20 lines, so the weight is breadth, about 25 short readers of formats each popular in some era, and cutting into them would mean many arbitrary choices for little gain in simplicity.

## The core, 172 lines

**Popular:** every file passes through it.
**Extensions:** `.ape`, `.wv`, `.amr`, `.voc`, `.mid`, `.midi`, `.ac3` and `.swf` by the signature table; `.rmi` by the RIFF walk, as a RIFF it reads no further into; and `.m3u`, `.m3u8`, `.pls`, `.cue`, `.xspf` and `.srt` by the text verdict.

- Lines are the non-blank lines in `scout/src/lib.rs` at `a7f3612`, other than the essay, the section headings and the tests, and every section here adds up to the 794. Reach is the figure `fileTypes.js` gives an extension, 1 to 3. The rest of each Popular line is judged, not measured; the test panel's run over a real collection is what would count it.
- A format cut to naming keeps one row in the signature table, about a line, and its files go on to the web engine if they are pictures and to the scan tier if they are sound or video.
- 35 lines are `Bytes`, which reads every number in the tip through one helper that answers nothing past the end, and keeps how deep the reading went.
- 23 lines are `read_format`, which checks each format's signature where the format puts it, a line a format, and runs that format's reader.
- 23 lines read the RIFF wrapper and walk its chunks, which WebP, AVI and WAV share; WebP alone needs about 6 of them.
- 20 lines are `scout_bytes`, which runs the reader and turns where it stopped into the answer.
- 19 lines are `Tip`, the answer.
- 18 lines decide whether bytes that are no known format read as text, from counts of four kinds of byte.
- 17 lines are `SIGNATURES`, which names fifteen formats from their first bytes and reads no further: APE, WavPack, AMR, VOC, MIDI, AC-3, SWF in two forms, ZIP, RAR, 7z, gzip, PDF, an executable and SQLite.
- 17 lines set the rules every reader follows: the constants, the `Step` type, the checked arithmetic, the imports and the two lint lines.

## JPEG, 54 lines

**Popular:** reach 3, the most common picture of every era.
**Extensions:** `.jpg`, `.jpeg`, `.jpe`, `.jfif`.

- 26 lines step through the segments by their stated lengths to the frame header, which holds the size, for baseline and progressive files alike.
- 27 lines read the EXIF orientation on the way past, so a phone's portrait comes back tall. Phone cameras write it in nearly every photo. Most of these lines are the TIFF directory reader, which TIFF shares.
- 1 line steps over fill bytes before a marker, which the format allows and few writers use.

## PNG, 18 lines

**Popular:** reach 3, since the late 1990s; animated PNG stayed rare.
**Extensions:** `.png`.

- 8 lines read the size from the header chunk, which comes first by rule.
- 10 lines walk the chunks before the picture to find animated PNG's control chunk.

## MP4, MOV, M4A and 3GP, 84 lines

**Popular:** reach 3 for MP4, MOV and M4A, and 1 for 3GP; every phone's video, the web's video, and iTunes' music.
**Extensions:** `.mp4`, `.m4v`, `.f4v`, `.mov`, `.qt`, `.3gp`, `.3g2`, `.m4a`, `.m4p`.

- 36 lines walk the top-level boxes and name the family by the file-type box's brand.
- 16 lines walk the movie box for the movie header's running time. They answer every file whose index comes first, and the second slice brings them an index written last, as every phone writes it.
- 18 lines read the first picture track's size. The track is chosen by whether its header has a size, so a subtitle or chapter track can lend an audiobook a width and height, which is BookDimensions; reading the track's handler box would take about 10 lines, an estimate. The track header holds the shown size rather than the stored pixel count, which is RectanglePixels.
- 6 lines turn the size by the track's matrix, for the quarter turns a phone records.
- 5 lines read a box's 64-bit size, which every file past 4 GB has; a phone's 4K video reaches that in minutes, and the walk steps over its data box by it.
- 3 lines recognize QuickTime written before 2001, with no file-type box.

## MP3, 59 lines

**Popular:** reach 3, every collection since Napster, and the late 2000s' with cover art in the tags.
**Extensions:** `.mp3`, `.mp2`.

- 27 lines decode a frame header, with the bit-rate and sample-rate tables.
- 9 lines step over an ID3 tag, and ask for the second slice where it ends when it runs past the tip, which cover art often makes it do.
- 8 lines find the first frame whose next frame sits where its length says.
- 7 lines read the frame count from a Xing or Info header, which every variable-rate file and every file LAME wrote carries.
- 6 lines run those steps in order.
- 2 lines estimate a constant-rate file's running time from its size, for the constant-rate files of the early collections.

## GIF, 27 lines

**Popular:** reach 3, from CompuServe in 1987 to today's reaction GIFs, many of them animated.
**Extensions:** `.gif`.

- 7 lines read the size from the screen descriptor.
- 20 lines walk the extension blocks to find the looping extension an animated GIF carries.

## WebP, 23 lines

**Popular:** reach 3, common on the web since Safari took it in 2020.
**Extensions:** `.webp`.

- 10 lines read the first chunk's name and check the size.
- 4 lines read the lossy form, which most WebP files are.
- 5 lines read the lossless form.
- 4 lines read the extended form, with its animation flag and canvas size, which animated WebP and WebP with transparency or EXIF use.

## Matroska and WebM, 98 lines

**Popular:** reach 3, fansubs and rips since about 2005, and the web's video downloads.
**Extensions:** `.mkv`, `.mka`, `.webm`.

- 42 lines read EBML's variable-length integers and elements, the generic part every answer needs.
- 23 lines read the first picture track's size from the tracks element. Matroska keeps a display size beside the pixel size, which is RectanglePixels.
- 21 lines walk the header and the segment to the information and tracks elements, and name WebM by its document type.
- 12 lines read the running time from the information element.
- The running time alone, without the size, would be about 75 lines, an estimate.

## HEIC and AVIF, 4 lines

**Popular:** reach 3 for HEIC, every iPhone photo since 2017, and 2 for AVIF.
**Extensions:** `.heic`, `.heif`, `.avif`.

- 4 lines, in the MP4 reader, name them by their brand and read no size, which sits in the item properties three boxes deeper.
- Sizing them would be about 50 lines, an estimate for code not written, through a walk close to MP4's.

## AVI, 25 lines

**Popular:** reach 3, DivX and XviD from about 1999 to 2010, and every capture card's output.
**Extensions:** `.avi`, `.divx`.

- 22 lines find the main header in the header list, for the frame time, the frame count and the size.
- 3 lines take the whole frame count from the OpenDML header, which every AVI past a gigabyte has.
- It also shares the RIFF walk counted in the core.

## FLAC, 10 lines

**Popular:** reach 3, the collector's lossless audio since the early 2000s.
**Extensions:** `.flac`.

- 10 lines read the sample rate and the total samples from the stream information block, which comes first by rule.

## WAV, 11 lines

**Popular:** reach 2, every era.
**Extensions:** `.wav`.

- 11 lines divide the data chunk's size by the format chunk's bytes a second.
- It also shares the RIFF walk counted in the core.

## Windows Media, 33 lines

**Popular:** reach 3 for WMV, 2 for WMA and 1 for ASF, everywhere in the 2000s.
**Extensions:** `.wmv`, `.wma`, `.asf`, `.dvr-ms`.

- 20 lines walk the header's objects, each a GUID and a size, with the five GUIDs the reader knows.
- 10 lines read each stream's kind and a video stream's size.
- 3 lines read the running time from the file properties, less the preroll.

## FLV, 41 lines

**Popular:** reach 3, Flash video, from about 2005 to 2012.
**Extensions:** `.flv`.

- 11 lines read the header's stream flags and find the first tag, which holds the metadata.
- 30 lines step through the metadata's names and typed values for the running time, the width and the height.
- A search bounded by the metadata tag for those three names would be about 15 lines in place of the 30, an estimate. It goes wrong only when a text value holds one of the names in its encoded form.
- Whether FFmpeg's probe answers an FLV's size with its decoders off is open in `scan.md`.

## MPEG, 9 lines

**Popular:** reach 3 for `.mpg` and 2 for `.vob`, Video CDs and DVDs.
**Extensions:** `.mpg`, `.mpeg`, `.mpe`, `.vob`, and `.dat` through the RIFF wrapper a Video CD gives it.

- 9 lines search for the sequence header and read the size, and a program stream's running time is its last pack's clock less its first's, from the second slice. The same header holds an aspect code, which is RectanglePixels.

## BMP, 8 lines

**Popular:** reach 2, the Windows wallpapers and Paint files of the 1990s.
**Extensions:** `.bmp`, and `.dib` when it carries the file header.

- 8 lines read the size from either header, OS/2's of 1990 or Windows'.

## Ogg, 12 lines

**Popular:** reach 3 for `.ogg` and 2 for `.opus`, Vorbis since about 2002 and Opus since 2012.
**Extensions:** `.ogg`, `.ogv`, `.opus`.

- 12 lines name the codec from the first packet, and read Theora's size. The running time is the last page's granule position, which the second slice reads, over the clock the first packet gives.

## TIFF, 14 lines

**Popular:** reach 2, scanners and print.
**Extensions:** `.tif`, `.tiff`, and `.dng`, `.cr2`, `.nef` and `.arw`, which it names as TIFF and refuses.

- 10 lines read the size from the first directory, which many writers put at the end of the file, past the tip, where the second slice reads it.
- 4 lines refuse a camera's raw file, whose first image is a small preview; `.cr2`, `.nef`, `.arw` and `.dng` are each reach 1.
- It also shares the TIFF directory reader counted in JPEG's lines.

## PSD, 7 lines

**Popular:** reach 2, anywhere artists keep their working files.
**Extensions:** `.psd`.

- 7 lines read the size from Photoshop's header.

## SVG, 12 lines

**Popular:** reach 3, the web's drawings.
**Extensions:** `.svg`.

- 12 lines name text whose first element is svg, and read no size.

## AAC, 7 lines

**Popular:** reach 2 for raw AAC, most of which now sits inside MP4 as M4A.
**Extensions:** `.aac`.

- 7 lines name raw AAC frames, behind an ID3 tag or not, by two sync words a frame apart.

## RealMedia, 31 lines

**Popular:** reach 2 for `.rm` and `.rmvb`, RealPlayer's streams of the late 1990s, and much of Asia's video sharing until about 2010.
**Extensions:** `.rm`, `.rmvb`.

- 16 lines walk the chunks.
- 14 lines read each stream's MIME type and a video stream's size, past two strings of their own length.
- 1 line reads the running time from the properties chunk.
- The running time alone would be about 17 lines, an estimate.

## AIFF, 22 lines

**Popular:** reach 2, the Mac's WAV.
**Extensions:** `.aiff`, `.aif`.

- 11 lines walk the chunks, a copy of RIFF's walk in the other byte order.
- 11 lines read the frame count and the 80-bit float sample rate from the common chunk.
- Sharing RIFF's walk with a byte order argument would make it about 15, an estimate.

## Transport stream, 3 lines

**Popular:** reach 1, broadcast and camcorder recordings, and Blu-ray.
**Extensions:** `.ts`, `.mts`, `.m2ts`.

- 3 lines name it by three sync bytes in a row, and read nothing else.

## AU, 10 lines

**Popular:** reach 2, the early web's sounds, never in large numbers.
**Extensions:** `.au`, `.snd`.

- 10 lines divide the data size by the bytes a second the encoding fixes.

## Not named, no lines

**Popular:** reach 1 or 2 each.
**Extensions:** `.tga`, `.pict`, `.pct`, `.dv`, `.ogm`, `.nsv`, `.bik`, `.wtv`, `.ra`, `.dts`, `.pcx`, `.iff`, `.lbm`, `.wmf`, `.emf`, `.mod`, `.s3m`, `.xm`, `.it`, `.669`.

- The scout names nothing for 20 of the media extensions in `fileTypes.js`. Ten are enabled, TGA, PICT and PCT, DV, OGM, NSV, Bink, WTV, RealAudio and DTS, and ten are not, PCX, IFF, LBM, WMF, EMF, MOD, S3M, XM, IT and 669.
- RealAudio, which streamed the late 1990s, would be one row in the signature table.
