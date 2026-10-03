# Thumbnails, still open

What is left to decide, build and measure about the thumbnail pipeline, and nothing else. The pipeline as built is documented on the site, in `site/docs/thumbnails.md` — that page is the record of how a path becomes a tile and what was measured to choose each route, and it replaced the planning document that stood in for it while it was being built. This file is the short list of what that page does not get to claim.

It is deliberately small. A question here is one we decided not to answer yet, not one nobody thought of.

## The document that comes next

The site page is about getting one thumbnail right: which decoder, at what size, with which pixels, in which colors. It stops on purpose before every question that is about many of them at once, because those have different answers, fuji's are still moving, and the card they are currently built on is scaffolding that may not survive.

That second subject is a page of its own once the work is done: how many thumbnails to keep in flight on each route and in what order; what the sheet should be holding at any moment and what it should let go of; how the store should behave when the sheet and a table want the same file; and whether finished thumbnails belong on disk, which is how a file manager is instant on a folder it has seen before. `cache.md` holds the store's half of it in the meantime.

Nothing was lost taking it out of the site page. `SquareFlow.vue`'s essay carries the two loops and their widths, and that is the right home for them until the second page exists.

## Later: the look-ahead, and a database of what fuji has seen

**A separate system, and a much later one.** Today fuji reads a file when a visible card needs its thumbnail, and nothing sooner: one render per tile, the first bytes and the header read by the library that decodes it, and nothing kept once the card goes. That is what fuji wants now — fast to first pixels and light on the disk. What is sketched here would read further ahead than any card and remember what it read, and it is written down so the idea is scoped rather than rebuilt piecemeal inside the flow.

**What it would do.** Peek at the tips of files well ahead of the view — the first bytes for the format, the header for the size, a flag or two such as whether a WebP is animated — and keep what it learns, by path, in an SQLite database on disk, with enough beside each row to tell when a file has changed since, its size and modification time at the least. Everything that reads it would then ask the database first and the disk second.

**What would read it.** A card could hold every tile's box before its pixels arrive, so the rows stop reflowing as a card fills. A file saved with no extension could be found and named by its bytes, which is the open case in `security.md`. Animated WebP could be routed to an `<img>` before its tile is chosen. A file the page route cannot size today could be checked against the ceiling. And the thumbnail cache on disk, under decisions not made above, is a store of the same shape over the same paths, so the two may turn out to be one database.

**The one constraint already known.** The look-ahead can never stand between a card and its first thumbnail. Fuji had a version of this inside the flow: a probe of every file on a card, all answered before any render began. The sheet paused and then filled all at once, and taking it out made thumbnails appear immediately. Whatever reads ahead runs beside the view, or ahead of it, and the view uses what it finds when it is there and does without it when it is not.

**Open, all of it.** Where the database lives and what it is called; what a row holds and how stale is too stale; how far ahead to read and in what order, and how to stay light on a spinning disk or a network share; whether it walks a drive in the background or only follows the user; and what it costs a machine that never needs it.

## Tests to run

**A card's cost from inside the running app.** Every thumbnail and every card is already a row in the log, and the rows have now been read once — on Windows, to settle whether *WIC*'s scaled decode engages, which it does. That measurement is on the site, on the thumbnail pipeline page, along with the wrong experiment that nearly said otherwise.

What is still unread is the rest: what a card costs to fill, how that is spread across a real folder rather than a handful of authored files, and whether several native thumbnails in flight is helping or crowding. That last one is a tuning question rather than a correctness one, which is why it has waited. The trap, now known, is that a reading taken while a folder fills carries the other thumbnails' contention in it — a per-image cost has to be taken with one image in the folder.

**The odd-and-even measurement again, after a macOS or Safari update.** The `flowSnap` rule rests on observed WKWebView behaviour — a canvas box rounded to whole CSS pixels — and not on anything a specification guarantees. It needs a Retina Mac, takes about ten minutes, and the answer is a single percentage.

**`flowSnap` on Windows is settled, and is named here only so it is not mistaken for a gap.** It was measured at 125%, 150% and 175%, flooring was tried against rounding and was worse, and rounding stayed, because how far a canvas misses its box matters more than which side it misses on. Some tiles still resample at fractional scaling, and where a tile sits is involved, but that part is not characterized and no model built from one run has survived a fresh set of files — what is certain is that no canvas size can reach it, so it belongs to the sheet's layout. The measurements, the failed alternative and a picture of the defect are on the site, on the thumbnail pipeline page, which is where to go before reopening any of it.

## Decisions not made

**Fault tolerance is its own session.** Today every file fuji cannot show gets the placeholder and nothing is tried twice, which is a policy chosen for being simple rather than for being right. That session decides: a file whose extension lies in either direction, an image saved with no image extension and a non-image saved with one; a file the operating system refuses that the page might still show, which is a second attempt fuji never makes; and what a placeholder should say, if anything, about why. The two routes already disagree about the first of those: the native route refuses a file whose bytes are not the format its extension names, and the page route hands it to the engine, which decodes by the bytes and shows it if it can.

**The size ceiling's shape.** A header can claim a hundred thousand pixels a side so that a PNG decodes to forty gigabytes, and today's ceiling refuses anything whose raster would exceed half the machine's physical memory. A share of the machine rather than a fixed number is deliberate — it never limits capable hardware and never refuses what could have fit — but it has not been tested against the legitimate gigapixel files that exist, and half may be the wrong fraction.

**The size ceiling on the page route.** The ceiling stands in `thumbnail.rs`, so it guards the native route, where the decoder runs in fuji's own process. The page route — everything on Linux, and BMP, WebP and AVIF on Windows — hands the engine the whole file with no ceiling, exactly as a light table does with every picture it shows. The engine decodes inside its sandbox, so a bomb there costs a crashed content process rather than fuji's. If a ceiling is wanted on that side, the place for it is the store, which every page decode passes through, the tables' included, and not the sheet alone.

**Animated WebP**, which is a still on the native route today. Sending it to an `<img>` the way a GIF goes means knowing it is animated before its tile is chosen, which is a reading ahead of the render, and so belongs to the look-ahead below.

**HEIC, and the extension list generally.** ImageIO thumbnails a HEIC in about sixteen milliseconds; Windows without the paid codec cannot show one at all, so listing the extension means placeholders there. TIFF is the mirror case — WIC has it at the factory and Chromium never has. Neither is in `fileTypes` today, and both are a decision about what fuji claims to open rather than a decision about the pipeline.

**A thumbnail cache on disk.** How Finder is instant on a folder it has seen, and what would make the walk through a drive free on the way back. It is a store with an eviction policy, a location, and an invalidation rule, so it belongs to the document above rather than to this list.

**A change of monitor.** Canvases are never remade when a window moves to a different screen, so both `devicePixelRatio` and the gamut go stale until the sheet rebuilds. Read from the code, never exercised, and it needs two screens of different character to exercise at all.

## Owned somewhere else

Listed so this file does not grow to hold them.

**The path scope in Rust**, which would keep every disk and thumbnail command inside the folders the user has actually shown fuji. It is `security.md`'s first wall and the largest single change to fuji's posture available; the render's signature check and its size ceiling are walls two and three and are built.

**What a card is to the user** — whether it is visible at all, how many fill a page, and where Next sits. `card.md` has the walk through a whole drive that the card exists for, and the open questions are there.

**Everything the engines do with a canvas** — acceleration, compositing layers, subsampling rules, and the sources each claim was read from. `canvas.md` is that file and it is not about thumbnails specifically.
