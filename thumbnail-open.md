# Thumbnails, still open

What is left to decide, build and measure about the thumbnail pipeline. What is built is documented where it lives: the site's thumbnail pipeline page for how one file becomes one tile and what was measured to choose each route, the essays in `TestFlow.vue`, `governor.js` and `thumbnail.rs` for the code, and the essay above `disk_readdir` in `disk.rs` for where Rust's waiting commands run. This file holds only what none of those can claim yet.

Boiled down on 2026-10-04, at the end of a pass that set out to evaluate, simplify and correct the pipeline before adding to it. That pass removed the probe that read every file on a bucket before any render, put every flow's reads and renders behind governors four wide, and moved every waiting Rust command onto the blocking pool so a panic comes back as an error.

## Next, in the order they would likely come

Each says where it came from, since some are the user's and some are a session's suggestions the user has not yet adopted.

**A deadline in the governor.** The answer to a call that never ends: past it, the governor frees the place and fails the caller's promise, while the stuck work finishes or does not on a thread nothing waits for. It would also free the diamond table, whose queue freezes behind one load that never settles. `governor.js` has the shape. Raised in conversation with the user during the 2026-10-04 pass, as the answer to the third way any call can end: never.

**Fuzzy Logic.** Each governor timing its calls and widening or narrowing itself from four to fit the resource, so a fast internal drive is driven harder than a conference thumb drive. The user's idea; `governor.js` sketches it.

**Priority in the governor**, letting a tile the user can see go ahead of one they cannot. A session's suggestion, written into `governor.js`'s essay and not yet adopted.

**More buckets, pages, and paging through a whole drive.** The sheet shows `sheet.buckets` buckets from the start of one folder today, and nothing past them.

**The look-ahead, and a database of what fuji has seen**, below. The user's idea, and deliberately much later.

**Two corrections**, a session's suggestions and not yet adopted: a top gate in the page that writes every escaped rejection to the log, and a limit on a read checked against the listing's size, so a disk image renamed `.jpg` is not read whole.

## Later: the look-ahead, and a database of what fuji has seen

**A separate system, and a much later one.** Today fuji reads a file when a visible bucket needs its thumbnail and nothing sooner, and keeps nothing once the bucket goes: fast to first pixels and light on the disk, which is what fuji wants now.

**What it would do.** Peek at the tips of files well ahead of the view — the first bytes for the format, the header for the size, a flag or two such as whether a WebP is animated — and keep what it learns by path in an SQLite database, with each file's size and modification time beside it to tell when it has changed. A bucket could then hold every tile's box before its pixels arrive; a file with no extension could be named by its bytes; animated WebP could go to an `<img>`; and a page-route file could be checked against the size ceiling. A thumbnail cache on disk, which is how Finder is instant on a folder it has seen, is a store of the same shape over the same paths, so the two may be one database.

**The one constraint already known.** It can never stand between a bucket and its first thumbnail. The probe did exactly that — every file on a bucket read before any render began — and the sheet paused and then filled all at once; taking it out made thumbnails appear immediately. Whatever reads ahead runs beside the view, and the view uses what it finds when it is there and does without it when it is not.

**Open, all of it:** where the database lives, what a row holds and how stale is too stale, how far ahead to read and how to stay light on a spinning disk or a share, whether it walks a drive in the background or follows the user, and what it costs a machine that never needs it.

## Tests to run

**A bucket's cost from inside the running app.** Every thumbnail and every bucket is a row in the log, and apart from one Windows question the rows have never been read: what a bucket costs to fill across a real folder, and whether four native thumbnails in flight helps or crowds. It is the data Fuzzy Logic would need. A per-image cost has to be taken with one image in the folder, since a reading taken while a folder fills carries the others' contention. The bytes, as against the milliseconds, are on screen now: each bucket's caption ends with its canvases' cost, and the memory report beneath the buckets sums them and sets them beside what fuji's processes and the machine are using.

**The odd-and-even measurement again, after a macOS or Safari update.** `flowSnap` rests on observed WKWebView behavior, a canvas box rounded to whole CSS pixels, not on anything a specification guarantees. It needs a Retina Mac, takes about ten minutes, and the answer is a single percentage.

## Decisions not made

**Fault tolerance is its own session.** Every file fuji cannot show is left out of its bucket and counted in its caption, and nothing is tried twice, a policy chosen for being simple rather than right. That session decides what to do with an extension that lies in either direction, with a file the operating system refuses that the page might still show, and what a bucket should say about them beyond a count. Both routes decode by the bytes whatever the extension says, so a file with the wrong extension shows wherever a decoder takes it.

**The size ceiling.** It refuses a raster over half the machine's physical memory, which has never been tested against the legitimate gigapixel files that exist, and half may be the wrong fraction. It also guards only the native route, where the decoder runs in fuji's own process; the page route hands the engine the whole file inside its sandbox, as a light table does. If the page side wants one, the store is the place, since every page decode passes through it.

**HEIC and TIFF**, a decision about what fuji claims to open rather than about the pipeline. ImageIO thumbnails a HEIC in about sixteen milliseconds and Windows without the paid codec cannot show one; TIFF is the mirror case, in WIC at the factory and never in Chromium.

**A change of monitor.** Canvases are never remade when a window moves to another screen, so `devicePixelRatio` and the gamut go stale until the sheet rebuilds. Read from the code and never exercised; it needs two screens of different character.
