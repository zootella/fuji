# Performance

What fuji's speed has actually been measured to be, and what those measurements mean. Everything here comes from the log, `log.js` and `log.rs`, writing files during real use on 2026-09-06; nothing in it is reasoned from first principles, and where a cause is still a theory it says so.

`architecture.md` says where things live, `cache.md` says what the store is and why it is dumb, and this file says what any of it costs.

## The instrument

The log is two functions named `log`, one in `log.js` for the page and one in `log.rs` for Rust, each taking a string and adding it to one text file per run. The page's lines go down to Rust; Rust's are already there; when fuji exits, if the setting says so, Rust writes the file. Every load, every flip and every thumbnail is a row of aligned columns written through the page's `log`, and any other line either side wants kept goes through the same call. It never renders, because the HUD it replaced could not answer honestly — building a string and painting text in the same frame as the image being timed makes the reading part of what it reports. `log.js` carries the essay on why a file and not a console: a console is trapped behind the platform, behind who is looking, behind a development build, behind where fuji is installed, and behind the language, and a file crosses all five.

Turn it on with `log.record` in `fuji.toml`; it is off at the factory. Files land in `fuji-temp` under your home folder, on every platform, and Rust makes that folder on the way out if it is missing. Each is named for the moment its run began, in UTC to the millisecond — `fuji-log-2026-09-08T18-29-27-123Z.txt` — so runs sort by time and never share a name, and the first line inside says which table and which flip window the run used. One file per run of fuji: the shell starts the log once, and no view does.

**Nothing touches the disk while fuji is running.** The page's lines go down to Rust a few at a time after things go quiet, Rust holds them in memory, and the whole file is written from `RunEvent::Exit` — a file write on the main thread in the middle of the flips being timed would be the HUD's mistake wearing different clothes. The cost is that a crash loses the log, which is the right trade for an instrument, since a crash mid-run invalidates the measurement anyway.

**A flip is recorded as two numbers, not one.** `store` is the wait on the cache and `paint` is the swap reaching the screen. They have unrelated causes and unrelated fixes, and every finding below came from seeing them apart. A third column says `hit` or `miss` — whether the image was decoded *before* the flip asked for it, which is the only thing that makes a window worth keeping.

**Loads and flips share one list, in the order they happened.** A decode landing in the middle of a flip is exactly what explains a slow flip, and two separate tables would have hidden the first finding entirely.

**The sheet's thumbnails are in the same list, as `thumb` and `card` rows.** A `thumb` row is one thumbnail: `hit` says which path made it — `native` through the operating system, `page` through the engine, `img` for a GIF or SVG the engine shows itself — or `refused`, with the reason in the note; `render` is the milliseconds it took, `bytes` what its canvas costs, `natural` its pixels. A `card` row closes each card, with how many images it held in `index`, the milliseconds to fill it in `render`, its canvases' bytes, and the count by path in the note. So a folder opened on the sheet reads as its thumb rows, one card row, and whatever the table loaded beside them.

**Getting a log out on Windows takes one trick, and it is not obvious.** The file is written from `RunEvent::Exit`, so fuji has to be *closed* rather than killed. Launch `target/release/fuji.exe`, wait past the page's 1500 ms quiet timer so its lines have gone down to Rust, then `taskkill /IM fuji.exe` **without** `/F`: a plain close reaches the exit event and writes both the log and the settings, where `/F` terminates the process and neither is written. Use the built exe rather than `pnpm local` for anything that needs a log. Found on the Windows 10 box, 2026-09-11.

## The subject

One folder, six images, 15.1 MB on disk and about 316 MB decoded:

    1red      6240 x 4160   25.9 MP   6,241,142 bytes   ~104 MB decoded   progressive jpeg
    2orange   4096 x 4096   16.8 MP     372,089 bytes    ~67 MB
    6purple   4105 x 3079   12.6 MP   2,285,126 bytes    ~51 MB
    4green    3629 x 2419    8.8 MP   2,497,662 bytes    ~35 MB
    5blue     2455 x 3068    7.5 MP   2,165,436 bytes    ~30 MB
    3yellow   3299 x 2199    7.3 MP   1,575,813 bytes    ~29 MB

On an M2, macOS, WKWebView through Tauri. `1red` earns its place: a 26-megapixel progressive JPEG is the worst case fuji is likely to meet, and every effect below is largest on it.

## One: reading a file cost 150ms per megabyte, and none of it was the disk

`disk_read` returned `Vec<u8>`, which crosses the IPC boundary as a JSON array — one decimal number per byte, written on the Rust side and parsed on the JS side. Measured read times, before:

    1,575,813 bytes    251ms      159 ms/MB
    2,285,126 bytes    345ms      151 ms/MB
    2,497,662 bytes    371ms      149 ms/MB
    6,241,142 bytes    904ms      145 ms/MB

Flat and linear in file size, which no SSD is. `cache.md` had predicted this in prose a while before anything measured it.

**`tauri::ipc::Response` carries the same bytes as an ArrayBuffer.** After, the same files read in 2–15ms — roughly a hundredfold, and no longer meaningfully related to size. Every JS caller already wrapped the result in `new Uint8Array(...)`, which accepts either form, so nothing above `disk.rs` changed.

**The lesson worth keeping:** a number that is suspiciously linear in size is a serialization cost, not an I/O cost.

## Two: a read started before the paint blocks the paint

The first window implementation slid the window at the top of `_flip`, before showing anything. Every flip then began a read and tried to paint through it. The result was the worst shape a performance bug can take — the cache reported perfect behaviour, every flip a `hit` with `store` at 0, and the app was slower than the triad it replaced.

The log caught it by accident. Each flip's `paint` and the next load's `disk` came back as the same number:

    flip 2   paint 372      load 4green    disk 371
    flip 6   paint 366      load 4green    disk 366
    flip 8   paint 345      load 6purple   disk 345
    flip 9   paint 251      load 3yellow   disk 251
    flip 11  paint 249      load 3yellow   disk 248

Two clocks timing one interval, which is what a blocked frame looks like from outside.

**Showing moved before asking**, which is what the triad had been doing all along in one line of comment — *wait for above paint to hit the screen* — and what the rewrite kept the words of without the meaning. Flips went from 247–374ms to 12–138ms. `DiamondTable.vue` carries the full essay above `_flip`, and the rule is: nothing that can occupy the main thread goes before the paint.

**No test would have found this.** Nothing threw, no wrong picture appeared, no assertion could have failed. The only symptom was a frame that took twenty times too long.

## Three: the engine drops decoded frames, and fuji cannot see it happen

With `flip.back` and `flip.forward` at 10 in a six-image folder, the window covers the whole folder from every index. The log confirms the store went completely inert: six loads at the top, then **44 flips with no load row anywhere among them**, every one `hit` with `store` at 0. Nothing was fetched, nothing was released, no `src` was cleared, no element was removed. `cardShow` only toggled `display`.

And returning to `1red` still cost a quarter of a second, but only sometimes. Grouping every flip back to it by what had been displayed since it was last on screen:

    shown in between            paint
    orange                      19, 16, 16, 5, 9, 14, 6          (5-19ms)
    orange and yellow           254, 260, 251, 230, 262          (230-262ms)

Twelve cases, no overlap, roughly twenty times apart. `3yellow` shows no such split — it lands at 73–103ms whatever precedes it.

**Two candidate causes are ruled out by the same run.** Flips 26–32 are `red, orange, red, orange, red, orange, red` — six flips between red showings, every one 4–16ms. So it is not an idle timer, and it is not the number of flips. Against that, flips 33–36 are `orange, yellow, orange, red` — fewer flips, and red comes back at 230ms. **What costs you red is that a third image was displayed.**

**The theory, and it is a theory.** Three layers sit under an `<img>`, and only the first is fuji's: the encoded bytes, which the blob and url hold; the decoded frame, which the engine caches and evicts on its own policy; and the rasterized layer, which `display: none` destroys outright. Layer three is rebuilt on every re-show and that is the 5–19ms floor. The 230–262ms is layer two having been evicted as well, so the frame is rebuilt before it can be rasterized. That it costs 250ms rather than `1red`'s original 1076ms is consistent — the original was measured while all six images decoded at once.

**The next day it was gone, and that is the most useful thing in this file.** The same folder, the same window, a longer walk, and every return to `1red` came back in 11 to 29ms — including returns after three intervening images, and after a full walk out to `6purple` and back past five of them. The split that had been twelve cases with no overlap simply did not appear.

    shown in between                    paint, the following day
    orange                              13, 16, 15
    orange and yellow                   15, 29
    orange, yellow, green               17
    five others, out to purple and back  19, 11

What changed between the runs was `experiment.js`. It ran on every launch and left a one-pixel, fully transparent `<img>` attached to `document.body` holding a decode of `1red` — about **104 MB pinned for the life of the process**, rooted in the DOM so nothing could collect it, plus three object urls never revoked. Every measurement in this file above that line was taken while fuji carried an invisible second copy of the largest image in the folder. Deleting the file removed it.

**So it is a byte budget, not a count.** Five intervening images with no penalty rules the count out. Six decoded images come to roughly 316 MB and now survive; with the experiment's extra copy the working set was about 420 MB and did not. The threshold sits between, higher than the 200 MB guessed above.

**And the lesson is larger than the number.** The instrument was honest and the folder was controlled, and the finding was still wrong — because something outside the experiment was consuming the resource being measured, and nothing in the log could show it. It took deleting an unrelated file for an unrelated reason to reveal it. When a measurement depends on a shared, invisible budget, the question is never only what the code under test is doing.

## What the window is worth

    flip.back / flip.forward     a fast flip     loads during flipping
    1 / 1                        12-138ms        one on every flip
    10 / 10                      3-25ms          none at all

Holding the whole folder is clearly better than holding three, and the reason is finding two rather than finding three: at 1/1 a decode is always running somewhere near the flip, and at 10/10 nothing is.

**And the window works.** An earlier reading of finding three concluded that holding more than about two large images bought nothing, because the engine discarded the rest regardless. That was measured against a handicap and is now retracted: with the leak gone, a six-image window holds six usable decodes and every return is a hit.

**What fuji does not own is the budget, not the window.** The engine still decides when a hidden image's frame goes, and fuji can neither see it happen nor prevent it. What fuji can do is stay under whatever the threshold is, which is an argument for a window measured in bytes rather than in images — not so fuji can evict, but so it can avoid asking for more than the engine will keep.

## What this means for the design

`architecture.md` wrote the conclusion before there was evidence for it:

> **Prefer pixels fuji owns.** An image handed to the page as a data URL on an `img.src` is decoded by the browser, and fuji can neither measure that memory nor free it except by clearing the source. An `ImageBitmap` is an object with a size fuji can account for and a `close()` that releases it. A cache with a real byte budget needs the second kind.

Finding three is that paragraph, measured. The store's `pixelBytes` is an estimate of memory the store does not control, and its retention is a request rather than a guarantee. The sheet took the road that paragraph points down on 2026-09-08, differently than it imagined: its thumbnails are canvases, with pixels from the operating system where the platform allows, and `canvas.md` has the measurements, taken outside the app. The table keeps its `img`, where showing beats drawing.

## Open

- **Where the byte threshold actually is.** Somewhere between 316 MB, which survives, and about 420 MB, which did not. A folder of progressively more or larger images would find it.
- **Whether Windows behaves the same.** WebView2 is Chromium and this was all WKWebView. Finding three especially is the sort of thing two engines could differ on.
- **What a folder of hundreds does.** Everything here is six images. Retention under real pressure is untested.
- **Whether the first decode can be made honest.** `img.decode()` resolves on a detached element that has never been in a render tree, so the store's `rendered` timestamp records something weaker than "ready to show."
- **What the sheet costs, from inside the app.** Its thumbnail paths were measured outside it, in `canvas.md`, from a scratch binary and a headless webview. Rows from the running app, one per thumbnail, are `thumbnail-plan.md`'s fifth step, and until then nothing here says what a card costs to fill.

## Reproducing any of this

Set `log.record = true` in `fuji.toml`, restart fuji, drag in a folder, and flip. Pause a second or two before quitting so the last rows are written. Then read down the `paint` column, not along the rows — every finding above came from one column disagreeing with itself.
