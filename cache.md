# Cache

One place that turns a path into pixels, and remembers. The sheet and every table ask it for images; it alone reads the disk. `architecture.md` says why it sits below the views and knows nothing about folders, order, or who asked.

This file is where the cache is being thought through. Some of it is settled and says so. The rest is the problem stated as clearly as it can be stated right now, which is not the same as solved — the decisions that are still open are open because fuji has not yet been dragged through enough real folders to make them honestly.

## What it is for

**Two views wanting the same image is the whole point.** The sheet renders a folder of thumbnails; the user clicks one and a table shows it full size. The table should not read that file again, and should not decode it again either. Later the user clicks away and comes back, and the folder is there at once because nothing was thrown away.

**Getting an image on screen costs twice, and neither cost is small.** Reading the bytes, then turning those bytes into pixels. They are separate costs, they are separately expensive, and they are worth recording separately — because they are worth *keeping* separately, and because which one dominates turns out to vary wildly from file to file.

## The first iteration

Deliberately small, and the point of it is to produce measurements rather than to be right about policy.

- **A caller asks by path and gets back an image ready to put on screen.**
- **The cache alone reads the disk.** Nothing above it calls `disk_read`.
- **Two callers asking for the same path at once do the work once.** The entry holds the promise from the first call, and the second awaits it rather than starting a second read and a second decode of a file already in flight. This is the situation the cache exists for, so it cannot be the situation it handles badly.
- **Requests go through a queue with a small number of workers.** A sheet opening a folder asks for everything it can see at once; running all of it at once means the file the user is looking at finishes last and every file in the folder is briefly in memory. How many workers, and whether a request can be withdrawn or reordered when the user scrolls, are open. That there is a queue is not.
- **A failure is remembered as a failure**, with as much detail about what went wrong as we have. Otherwise one broken file in a folder is retried on every scroll, forever.
- **Nothing is ever expired, evicted, or invalidated.** It grows for as long as fuji runs.
- **The footprint is on the HUD.** Not a limit, not a warning that does anything — a number the user and the developer can watch while using the app, which is the whole instrument this iteration is built to provide.

## What every entry holds

    path         the key, forwardized, as it came from the listing
    blob         the file's bytes, held once
    url          one object URL over that blob, made once and never remade
    img          a decoded <img>, which is the grip on the pixels
    requested    Date.now() when a caller first asked
    loaded       when the bytes arrived
    rendered     when the decode finished
    touched      when it was last handed to a caller
    error        what went wrong, in as much detail as there is
    stat         size and dates from the filesystem

**Three references to one file, each doing something the other two cannot.** The blob is the only way back to the bytes, because an `<img>` has a src string and no path back to what it was made from. The url is what a view puts in its own `img.src`, and what the engine keys the decode on. The img is what keeps that decode wanted.

**Do not also keep the array the bytes arrived in.** `new Blob([bytes.buffer])` copies — a Blob is an immutable snapshot — so holding both is two copies of every file. `blob.size` gives the length, and `blob.arrayBuffer()` gives the bytes back if hashing ever wants them.

**The url is made once and never remade.** This is the rule a broken cache would violate silently; the measurements below say what it costs.

**A view sets its own `src` to the entry's url.** The entry's `<img>` is the grip, not the delivery. Holding it biases the engine toward keeping the decode; it does not guarantee it, and the engine may still discard a decode for something nobody is painting and rebuild it on demand.

**The three timestamps are two durations**, and keeping them apart is the foundation everything later stands on: `loaded - requested` is what getting the bytes cost, `rendered - loaded` is what the decode cost. No eviction policy worth having can be designed without both.

**`touched` is the one that is easy to forget to record** and impossible to reconstruct afterwards.

## What the field is already saying

One file, downloaded from Unsplash at the start of the project, has taught more than any amount of reasoning:

    /Users/…/colors for fuji/1red.jpg
    6240 × 4160, progressive JPEG, 5.95 MiB on disk, 99 MiB decoded
    883ms getting the bytes + 7ms in memory + 960ms to render

**The decode is not something a byte cache can save.** 960ms, on an M2, every single time the image is shown, unless the decoded image itself is kept. That is why the cache holds the rendered image and not only the bytes — and it is exactly what the diamond table's triad already does for three images at a time, generalized to everything.

**Size and time are independent, and that is what makes eviction hard.** This file is 99 MiB decoded because it is 26 megapixels, not because it is progressive. A cheaply compressed image of the same dimensions would be the same 99 MiB and might decode in a tenth of the time. So the interesting question is never "how big is it" — it is "how much would it cost to make again", and the two have almost nothing to do with each other. Sketching the corners:

|  | fast to rebuild | slow to rebuild |
| --- | --- | --- |
| **small** | doesn't matter either way | keep it — cheap to hold, expensive to lose |
| **large** | the obvious thing to let go | the hard case, and the one this file is |

**A gigabyte is about ten of these pictures.** 99 MiB each. Whatever number ends up being the budget, that is the scale it is measured in, and it is nothing like the scale of the encoded bytes — for this file the decoded image is sixteen times the file. Two numbers, counted separately, or the footprint means nothing.

### What the experiment measured

One question could not be reasoned out: whether an `<img>` that is not in the document keeps its decoded pixels. `img.decode()` answers it, resolving at once for an image already decoded. On the subject file, in the tauri webview on macOS:

    1 cold decode, detached element, url1 .......... 925ms
    2 same element decoded again ................... 0ms
    3 new element, same url, first still alive ..... 1ms
    4 new element, same url, others released ....... 0ms
    5 new element, new url over the same blob ...... 911ms
    6 attached element, fresh url .................. 916ms
      detached element, same url as the attached one 0ms

**Being in the document buys nothing.** Lines 2 and 6 settle the question that prompted the test: a detached element keeps its pixels, and attaching one changes nothing.

**The decode belongs to the url, not to any element.** Line 3 — a brand new element pointed at the same url is ready in a millisecond. That is what a cache hit costs.

**And a fresh url over the same blob costs a full decode.** Line 5. Decodes are keyed by url, not by content, so a cache that mints a url per request re-decodes every time while looking exactly like a cache that works. Nothing else in this file matters more.

Line 4 carries a caveat: releasing both elements left the decode alive, but JavaScript cannot force collection, so that may only mean it had not been collected yet. It also cannot say what happens under memory pressure, which is the condition that matters at a gigabyte and the one this test could not create.

**Why fuji did not do this last year.** `library.js` carries the reason where the data URL is made: *"alternatively, URL.createObjectURL saves memory, but creates a resource that could leak."* That was right for code with no cache — an object URL has to be revoked by somebody, and a string the collector handles is safer when nothing is keeping anything. The cache is what inverts it: retaining is the point, and the revoke becomes the eviction lever rather than a leak.

**Releasing is deliberate, but it is release, not reclaim.** Revoking the url unbinds it from the blob, dropping the img removes what holds the decode, dropping the blob makes the bytes collectable — and then the collector decides when, because JavaScript has no free. The only thing in the platform that frees decoded pixels on demand is `ImageBitmap.close()`, and nothing measured here argues for the canvas rewrite it would cost. It maps cleanly onto the eviction tier this document keeps wanting: drop the img, revoke the url, keep the blob, and the next request pays 911ms to decode and skips 885ms of reading.

**The 883ms is probably not the disk.** `disk_read` returns `Vec<u8>`, which crosses the IPC boundary as a JSON array — six million numbers for this file, encoded on one side and parsed on the other. Tauri has a raw-bytes response, `tauri::ipc::Response`, that bypasses JSON and arrives in the page as an ArrayBuffer. If that is where the time is going, most of the read cost is fuji's own encoding and not the SSD's fault at all. Worth measuring before any conclusion about caching bytes is drawn, because it may move the number by an order of magnitude and change which half of the problem matters.

## Open, and why they stay open

**What to keep and what to let go.** The table above is the shape of the question, not an answer. A policy might weigh the decode duration, the size, the time since `touched`, and how far the image is from what the user is looking at — and the honest position is that nobody knows the weights until fuji has been scrolled through real galleries, real folders of saved torrents, real photograph libraries. Recording every input now is what makes that experiment possible later. Deciding the policy now would only mean deciding it wrong with more confidence.

**Whether the two footprints get one budget or two.** Bytes and decoded images differ by more than an order of magnitude and have completely different rebuild costs. Letting a decode go while keeping the bytes it came from is the obvious first move, and it implies two numbers with two thresholds rather than one. It also implies the cache can be in a half-state for a path, which is a thing the entry has to be able to express.

**Whether retention holds under pressure.** The measurements above were taken on an idle machine with one image. At a gigabyte the engine may begin discarding decodes and rebuilding them, and a cache hit that costs a millisecond today could cost 900ms then, with nothing in fuji able to see it happen. Watching for that is most of what the first iteration is for.

**Whether Windows answers the same way.** WebView2 is Chromium and the webview on macOS is WKWebView. Line 5 in particular — decodes keyed by url rather than by content — is exactly the kind of thing two engines could reasonably do differently, and the whole design rests on it.

**What the sheet actually needs.** A table wants the image at full size. A sheet wants three hundred of them small, and 300 × 99 MiB is not a cache, it is a crash. Whether that means the cache holds more than one decode per path, or the sheet renders down to something small of its own and holds that, is a question about the sheet as much as about the cache, and the sheet is centered text today. Chromium can decode a JPEG at reduced scale, which may make this cheaper than the arithmetic suggests — another thing to measure rather than assume.

**Identity.** Paths today, with the assumption that a file at a path never changes, is never replaced, and is never deleted. That is not true of a filesystem and will not survive v1.0. Hashing the bytes with SHA-256 gives a second index, by content, so one copy serves every path naming the same file — get the image on screen first, hash in the background, then move the entry under its hash. It is also the beginning of invalidation, since a changed file is a changed hash and a modification date from the listing is the cheap way to suspect one.

**Whether the entry's `stat` is worth a call per file.** Dates and size are also what a date sort needs, and `disk_readdir` already returns size but no dates. One widened listing might serve both and save the cache a round trip per image; see `sort.md`.

**Whether a cache hit should be synchronous.** A caller that already has the image could paint in the same frame rather than after an await, which matters for a sheet redrawing a folder it has seen before.

## Not in scope, on purpose

Folder listings are not cached — listing is cheap, pixels are not, and a user clicking back to a folder wants a fresh listing with cached images. The cache does not survive a restart; durable storage of derived facts like hashes and dimensions is the metadata idea in `roadmap.hide.md` and a different feature. And the cache never learns what folder an image came from, what order it is in, or which view asked — that ignorance is what lets a sheet and a table share it without either knowing the other exists.
