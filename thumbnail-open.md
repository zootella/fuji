# Thumbnails, still open

What is left to decide, build and measure about the thumbnail pipeline, and nothing else. The pipeline as built is documented on the site, in `site/docs/thumbnail-pipeline.md` — that page is the record of how a path becomes a tile and what was measured to choose each route, and it replaced the planning document that stood in for it while it was being built. This file is the short list of what that page does not get to claim.

It is deliberately small. A question here is one we decided not to answer yet, not one nobody thought of.

## The document that comes next

The site page is about getting one thumbnail right: which decoder, at what size, with which pixels, in which colors. It stops on purpose before every question that is about many of them at once, because those have different answers, fuji's are still moving, and the card they are currently built on is scaffolding that may not survive.

That second subject is a page of its own once the work is done: how many thumbnails to keep in flight on each route and in what order; what the sheet should be holding at any moment and what it should let go of; how the store should behave when the sheet and a table want the same file; and whether finished thumbnails belong on disk, which is how a file manager is instant on a folder it has seen before. `cache.md` holds the store's half of it in the meantime.

Nothing was lost taking it out of the site page. `SquareFlow.vue`'s essay carries the two loops and their widths, and that is the right home for them until the second page exists.

## Tests to run

**A card's cost from inside the running app.** Every thumbnail and every card is already a row in the log — `thumb` rows carry which route made the pixels, the milliseconds, the canvas bytes and the natural size, and a `card` row closes each card with the count by route. Nothing has been read off them yet. `performance.md` still measures only the table, and its own open list says so. Set `log.record = true`, open a folder of a few hundred on the sheet, and the first real numbers for the sheet are there. Two things to look for: whether four native thumbnails in flight is the right width, and whether the codec's scaled decode actually engages on Windows or a full decode is being thrown away, which the `render` column would show as a flat cost per megapixel.

**`flowSnap` at a fractional device pixel ratio.** This is the one measurement the Windows box owes. macOS scale factors are only ever 1 or 2, so a whole-CSS box times the ratio is always a whole number of device pixels; Windows offers 125%, 150% and 175%, and at 1.5 a 135-pixel box is 202.5 device pixels, which no canvas can be. The method is the one that settled it on the Mac and is written up in `fidelity.md`: author test images at exactly the size the thumbnailer returns, carrying one-pixel rows, and count the pixels that come back neither black nor white. Until this runs, the most that can be claimed for Windows is "no worse than before".

**WIC's color behaviour.** Read from documentation and never run against a file. It reports sRGB whatever is asked and the twelve-byte header says so, which the page trusts; what has not been checked is whether a file carrying a profile actually arrives converted. A photograph with an embedded Adobe RGB profile is the test, on the Windows box.

**EXIF orientation on Windows**, for the same reason and on the same machine. WIC leaves orientation to the caller and `thumbnail.rs` applies it by hand, reading the tag and mapping EXIF's eight cases onto WIC's rotate-then-flip order. A portrait photograph straight off a phone settles the six cases a camera writes; cases 5 and 7 are the mirrored diagonals nothing writes, and they stay untested.

**The odd-and-even measurement again, after a macOS or Safari update.** The `flowSnap` fix rests on observed WKWebView behaviour — a canvas box rounded to whole CSS pixels — and not on anything a specification guarantees. It needs a Retina Mac, takes about ten minutes, and the answer is a single percentage.

## Decisions not made

**Fault tolerance is its own session.** Today every file fuji cannot show gets the placeholder and nothing is tried twice, which is a policy chosen for being simple rather than for being right. That session decides: a file whose extension lies in either direction, an image saved with no image extension and a non-image saved with one; a file the operating system refuses that the page might still show, which is a second attempt fuji never makes; whether img tiles are probed at all, since a GIF has its size in its first ten bytes and an SVG has nothing to check but a leading `<`; and what a placeholder should say, if anything, about why.

**The size ceiling's shape.** A header can claim a hundred thousand pixels a side so that a PNG decodes to forty gigabytes, and today's ceiling refuses anything whose raster would exceed half the machine's physical memory. A share of the machine rather than a fixed number is deliberate — it never limits capable hardware and never refuses what could have fit — but it has not been tested against the legitimate gigapixel files that exist, and half may be the wrong fraction.

**Linux's size gate, for WebP and AVIF only.** PNG, GIF and BMP are sized by our own Rust on every platform, so Linux already refuses an oversized one. JPEG never needs the gate, because both operating systems decode a JPEG scaled and the full raster never exists. That leaves WebP and AVIF unsized on Linux, and closing it means a header parser of fuji's own for two formats.

**Animated WebP**, which is a still on the native route today. Sending it to an `<img>` the way a GIF goes means reading the animation flag out of its header in the probe.

**HEIC, and the extension list generally.** ImageIO thumbnails a HEIC in about sixteen milliseconds; Windows without the paid codec cannot show one at all, so listing the extension means placeholders there. TIFF is the mirror case — WIC has it at the factory and Chromium never has. Neither is in `imageTypes` today, and both are a decision about what fuji claims to open rather than a decision about the pipeline.

**A thumbnail cache on disk.** How Finder is instant on a folder it has seen, and what would make the walk through a drive free on the way back. It is a store with an eviction policy, a location, and an invalidation rule, so it belongs to the document above rather than to this list.

**A change of monitor.** Canvases are never remade when a window moves to a different screen, so both the device pixel ratio and the gamut go stale until the sheet rebuilds. Read from the code, never exercised, and it needs two screens of different character to exercise at all.

## Owned somewhere else

Listed so this file does not grow to hold them.

**The path scope in Rust**, which would keep every disk and thumbnail command inside the folders the user has actually shown fuji. It is `security.md`'s first wall and the largest single change to fuji's posture available; the probe's signature check and the size ceiling are walls two and three and are built.

**What a card is to the user** — whether it is visible at all, how many fill a page, and where Next sits. `card.md` has the walk through a whole drive that the card exists for, and the open questions are there.

**Everything the engines do with a canvas** — acceleration, compositing layers, subsampling rules, and the sources each claim was read from. `canvas.md` is that file and it is not about thumbnails specifically.
