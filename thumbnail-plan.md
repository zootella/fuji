# The thumbnail plan

How fuji makes a thumbnail, from a path to a tile in the sheet. This is the answer. `canvas.md` has the measurements and engine facts behind each choice, `security.md` the walls that go around every decoder, and `card.md` the box the tiles fill.

## In outline

    a path, listed by its extension
      gif or svg                       an img tile: the engine shows the file itself
      anything else                    a canvas tile, and its pixels come from one of two places
        on the platform's native list    rust asks the operating system, and the page puts the pixels on the canvas
        not on it, or linux              the page decodes the file and halves it down into the canvas
      bytes not the format claimed     refused: the placeholder

    native lists    mac: jpeg png webp avif bmp      windows: jpeg png      linux: none

One flow, SquareFlow, does all of this for a card: it probes the card's files in one call, lays out every box at its final size, then fills them.

## The tiles

**A canvas for every raster format, whichever path made the pixels.** A canvas is memory fuji sized and the engine cannot take back, which is what makes a fixed number of cards a real budget, and one kind of tile makes a card's cost one number. Native pixels arrive raw and go on with a single `putImageData`: no decode, no blob, no url, and the store never hears about the file. An img would have been the wrong tool for them, since it needs an encoded file, so Rust would gain an encoder and the page a second decode, and the pixels would belong to the engine again. Page pixels are drawn in by the halving that `CanvasFlow.vue` does today.

**An img for GIF and SVG, all of them, whatever their size.** A GIF because only an img animates, and one that bounces in a browser should bounce in fuji; the engines run an animation only while it is in view, so a card of GIFs costs the few on screen. An SVG because it has no pixels to own: the engine parses it and paints it sharp at whatever size the css gives, and one without intrinsic dimensions cannot be drawn into a canvas at all. Each gets the store's url with no decode and two lines of css to fit the square.

**An SVG in an img is already sandboxed, and that is the whole rule.** Shown through an img, an SVG is an image and not a document: its scripts never run, it loads nothing from outside itself, it gets no events, and it is clipped to the img's box, so it cannot draw over its neighbours however its coordinates are written. It is parsed in the engine's content process, which is sandboxed on both platforms. It must never be shown the other way, inline in fuji's document or in a frame, where its scripts would run as fuji.

## Who makes the pixels

**The operating system, wherever the platform's list allows.** ImageIO on the Mac and WIC on Windows, through `thumbnail.rs`. It is three times faster than the page on ordinary web pictures and up to twenty-five times on camera originals, and it costs the main thread nothing where the page costs it 50 to 220 milliseconds per large photograph. Responsiveness is what the user feels, ahead of the total wait.

**Each list is short, conservative, and not exceeded.** Windows gets JPEG and PNG: WIC's decoders for both have shipped in every Windows since XP and are its most exercised code. The Mac gets the formats this Mac's ImageIO was run against and decoded at the factory. A format off a column's list goes to the page whatever the machine could do; fuji does not probe for codecs. That is `security.md`'s reading: the rarer the format, the more sandboxed its path.

**The page, for everything else.** The store reads and decodes the file, the halving draws it into the canvas, the store releases it, one file at a time. It is the minority path on the Mac and Windows and the only path on Linux, and it stays as it is because it is correct on all three. It could not be img tiles instead: WebKit decodes an img small only on Apple platforms, so on Linux an img of a camera original is the full hundred-megabyte decode the canvas exists to prevent.

## The probe

**One Rust command, one call per card, before any thumbnail is made.** `thumbnail_probe(paths)` reads each file's first bytes and, on the Mac and Windows, its dimensions from the header without decoding. It answers with a format and a size, or a refusal: bytes that are not a format fuji handles, or a header claiming more than the ceiling. Those are walls two and three of `security.md`, and `thumbnail_render` enforces the same two itself, so a caller that skipped the probe cannot hand it a mystery.

**The size is what ends the reflow.** A card that knows every aspect ratio lays out every box at its final size at once, and thumbnails land in boxes already there. On Linux the probe names the format only, since there is no library to ask, so Linux keeps the reflow.

## SquareFlow

**One flow, in one file for now.** It takes a card's paths and the box, probes them as a batch, lays out the boxes, and fills them in the model's order. The table, the probe call, the unpack and the halving all live in `SquareFlow.vue` until a second arrangement exists to share them with.

**Two loops and a list.** The native loop keeps a few thumbnails in flight, four to start, because each is a pool thread that never touches the page and the machine has cores. The page loop keeps one in flight, because each is a full decode held in the store and a draw on the main thread. The img tiles are set at once and the engine loads them as it likes. Both loops stop when the card goes away and wait while the sheet is hidden, so a sheet behind the table does none of its work inside the table's frames.

**What it counts and what it keeps.** Every canvas is width times height times four bytes, totalled per card, so what a card costs is a number the sheet can read, exact for all but the imgs. Nothing is kept between cards or launches; a page Next discards is remade on the way back, and a cache on disk is a separate document. A change of thumbnail size is every card made again.

## The steps, in order

1. **The walls in Rust.** `thumbnail_probe`, the signature check and the size ceiling in `thumbnail.rs`, type-checked on both platforms the way `thumbnail.rs` was.
2. **SquareFlow**, beside TagFlow and CanvasFlow in the register, so the three can be compared on one folder.
3. **The switch and the retirement.** `card.flow` becomes SquareFlow and then goes away as a setting; TagFlow and CanvasFlow are deleted, their essays' lessons already in `canvas.md`. The img control lives outside the app, as the page beside the test images does.
4. **The documents.** `structure.md` says there is one flow; `card.md` drops the experiment and points here; `contents.md`, `CLAUDE.md` and `settings.js` lose the two names; `architecture.md`'s tree names SquareFlow.
5. **The log.** A row per thumbnail, with its path, which path made it, and the milliseconds, so `performance.md` gets numbers from the running app in place of the bench in `canvas.md`.
6. **The path scope**, `security.md`'s first wall, for every disk and thumbnail command. Independent of the rest, and built before fuji goes to anyone else.

## Status

Steps one, two and five are built, 2026-09-08. `thumbnail_probe` and both walls are in `thumbnail.rs`, type-checked on both platforms and run on the Mac against the six test images, a PNG named `.jpg`, a text file named `.jpg`, an empty file, a header claiming ten gigapixels, and SVGs with and without a byte order mark, each answered as the plan says. `SquareFlow.vue` is in the register beside the old two, is chosen by `card.flow`, and passed its first smoke test on the six. Every thumbnail and every card it fills is a row in the log, which `performance.md` describes. The switch and the retirement, steps three and four, have not happened, and neither has the path scope.

## Open

- **Fault tolerance is its own session, and these are for it.** For now every file fuji cannot show gets the placeholder, and nothing is tried twice. The session decides the rest: a file whose extension lies in either direction, an image saved with no image extension and a non-image saved with one; a file the operating system refuses that the page might show; whether imgs are probed at all, since a GIF has its size in its first ten bytes and an SVG has nothing to check but a leading `<`; HEIC, which this Mac thumbnails natively in sixteen milliseconds and Windows without the paid codec cannot show at all, so listing it means placeholders there; and the size ceiling. On the ceiling: it exists only because a header can lie, claiming a hundred thousand pixels a side so that a PNG decodes to forty gigabytes, and a fixed number would also refuse the legitimate gigapixel files Wikimedia keeps. The recommendation is a ceiling relative to the machine's own memory, so it never limits capable hardware, now or later, and only turns away what could not have fit anyway. JPEG never needs it, because both operating systems decode a JPEG scaled and the full raster never exists; PNG, BMP, WebP and AVIF decode whole, and that is where a lie costs memory.
- **Animated WebP**, a still on the native path until the probe reads the animation flag in its header and sends it to an img the way a GIF goes.
- **The native loop's width.** Four is a guess for the log to correct.
- **A thumbnail cache on disk**, which is how Finder is instant on a folder it has seen and would make Next and back free.
- **The Linux size gate**, which would need a header parser of fuji's own.
