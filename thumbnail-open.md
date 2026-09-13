# Thumbnails, still open

What is left to decide, build and measure about the thumbnail pipeline, and nothing else. The pipeline as built is documented on the site, in `site/docs/thumbnail-pipeline.md` — that page is the record of how a path becomes a tile and what was measured to choose each route, and it replaced the planning document that stood in for it while it was being built. This file is the short list of what that page does not get to claim.

It is deliberately small. A question here is one we decided not to answer yet, not one nobody thought of.

## The document that comes next

The site page is about getting one thumbnail right: which decoder, at what size, with which pixels, in which colors. It stops on purpose before every question that is about many of them at once, because those have different answers, fuji's are still moving, and the card they are currently built on is scaffolding that may not survive.

That second subject is a page of its own once the work is done: how many thumbnails to keep in flight on each route and in what order; what the sheet should be holding at any moment and what it should let go of; how the store should behave when the sheet and a table want the same file; and whether finished thumbnails belong on disk, which is how a file manager is instant on a folder it has seen before. `cache.md` holds the store's half of it in the meantime.

Nothing was lost taking it out of the site page. `SquareFlow.vue`'s essay carries the two loops and their widths, and that is the right home for them until the second page exists.

## Tests to run

**A card's cost from inside the running app.** Every thumbnail and every card is already a row in the log, and the rows have now been read once — on Windows, to settle whether *WIC*'s scaled decode engages, which it does. That measurement is on the site, on the thumbnail pipeline page, along with the wrong experiment that nearly said otherwise.

What is still unread is the rest: what a card costs to fill, how that is spread across a real folder rather than a handful of authored files, and whether several native thumbnails in flight is helping or crowding. That last one is a tuning question rather than a correctness one, which is why it has waited. The trap, now known, is that a reading taken while a folder fills carries the other thumbnails' contention in it — a per-image cost has to be taken with one image in the folder.

**The odd-and-even measurement again, after a macOS or Safari update.** The `flowSnap` rule rests on observed WKWebView behaviour — a canvas box rounded to whole CSS pixels — and not on anything a specification guarantees. It needs a Retina Mac, takes about ten minutes, and the answer is a single percentage.

**`flowSnap` on Windows is settled, and is named here only so it is not mistaken for a gap.** It was measured at 125%, 150% and 175%, flooring was tried against rounding and was worse, and rounding stayed, because how far a canvas misses its box matters more than which side it misses on. Some tiles still resample at fractional scaling, and where a tile sits is involved, but that part is not characterized and no model built from one run has survived a fresh set of files — what is certain is that no canvas size can reach it, so it belongs to the sheet's layout. The measurements, the failed alternative and a picture of the defect are on the site, on the thumbnail pipeline page, which is where to go before reopening any of it.

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
