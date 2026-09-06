# Cache

A store, not a strategy. It holds what the views tell it to hold and lets go when they say to let go. It does not decide, schedule, prioritise, or expire, and it never learns anything about folders, order, or who is asking. Every clever decision in fuji about images lives in the view that is showing them.

This file is where that is being thought through. Some of it is settled and says so; the rest is the problem stated as sharply as it can be right now, which is not the same as solved.

## Why it is dumb on purpose

The cache went through three shapes before this one, and each failed for the same reason: intelligence in the middle needs to know things only the edges know.

**A shared queue needs a priority system.** If one loader serves everybody, a table's urgent image lands behind the sheet's four hundredth thumbnail, and fixing that means teaching the middle whose request matters — which is the view's knowledge, moved somewhere it does not belong. So there is no queue. Every request races every other request, and Rust and the operating system sort it out, exactly as if the view had called the disk itself.

**A shared eviction policy needs to know what the user is looking at.** What to keep, in what size, for how long, and what to throw away when the sheet's thumbnail size changes are all questions with view-shaped answers. So there is no policy. The store frees what it is told to free, when it is told.

**What is left is worth centralising precisely because it is not clever**: one place that knows a path's bytes, its object url, its decoded pixels, and what each of those cost. One owner, so nothing is loaded twice and nothing is freed twice or never.

## The commands

Views do all the thinking and say what they want in two words:

- **need** — a path and who is asking. Taking a reference and asking for the image are the same act, so nothing can be had without saying who wants it, and there is no way to load something and forget to say so. If the store has it, it comes back at once; if not, the store loads it and the caller waits. Two callers arriving together share one load, because that is bookkeeping rather than judgement.
- **release** — the same path and the same holder. When the last reference goes, the store revokes the url and drops what it held.

That is the whole protocol. No hints, no priorities, no promises about what will still be there later.

**References are counted per path, not per size.** Everything the store holds for a path lives while any holder wants that path. This is the dumb answer and it has one visible consequence: if the sheet releases its thumbnails while a table still holds the same file, the thumbnail lingers until the table lets go too. A handful of small bitmaps, not a category of problem — and if it ever becomes one, the fix is another plain command rather than any cleverness.

## What the store holds, per path

    blob         the file's bytes: raw material for any decode at any size, and for a hash later
    url          one object url over that blob, made once and held as long as the entry is
    img          the decoded element, which is the thing a table puts on screen
    references   a map of holder name to count, so a leak has a name
    requested    Date.now() when a caller first asked
    loaded       when the bytes arrived
    rendered     when the decode finished
    touched      when it was last asked for
    error        what went wrong, in as much detail as there is

**Two products over one file, and a name for one of them.** The blob is what anything future is made from — a thumbnail with `createImageBitmap`, a SHA-256 hash, a re-decode after the engine drops one. The img is the finished thing. The url is neither: it is the handle the img was built through, kept because it is the cheap way back if the engine ever drops what it built.

**The blob costs about 6% of the entry**, and the argument for it got weaker the day `disk_read` started returning raw bytes. Re-reading a file is cheap now. The blob stays because it is the raw material for a thumbnail, a hash, and a re-decode, not because reading is expensive.

**A table shows the store's element itself.** It does not point an element of its own at the same picture, because that pays the whole decode again. This is the single most important thing in this file and it was learned the hard way.

## Why the store owns the object url

`library.js` carries the reason fuji avoided object urls for a year, on the line where it makes a data url: *"alternatively, URL.createObjectURL saves memory, but creates a resource that could leak."* That is exactly right, and the answer is ownership rather than avoidance: urls are made in one place in `cache.js` and revoked in one place, `cacheFree`, and no view touches either. The reference count is what makes the rule checkable instead of remembered.

**Revoking at `decode()` was tried, and reverted.** It is a tempting shortcut — a loaded `<img>` does go on displaying after its url is revoked, because revoking removes the name and not the data. The worry is what happens when the data goes: an element the page is not showing can have its decoded frame dropped by the engine, and getting it back means rebuilding from source. A revoked url leaves nothing cheap to rebuild from, where the old data url — a string on the element that nothing could revoke — always did.

**That worry is reasoning, not measurement.** Whether a hidden element actually loses its frame here, and whether the url is what it would rebuild through, is in the open questions below. Keeping the url costs one revoke in a function that already exists, so the store owns it until that is settled.

## What the views do instead

**A table keeps a window around the current image.** It asks for the neighbours of wherever the user is, holds references to them, and releases the ones that fall out of range as the user flips. `flipCache.js` is the diamond table's, and it counts **images**, from `flip.back` and `flip.forward` in `fuji.toml`.

Counting images is known to be the wrong unit and is deliberate for now. A dozen 26-megapixel photographs is 1.2 GB where a dozen web JPEGs is 100 MB, so a count is wrong in both directions and a byte budget would let the window fit the folder. But a count is a number the user can change while fuji is running, which is what makes the window itself measurable — and until the measurements below say what a window is worth, a budget would be a policy written ahead of its evidence. The unit is an open question with an instrument pointed at it, not a settled answer.

This is the triad grown up: same job, no fixed three. The half of the triad that was never about caching is kept — showing an element that already has its pixels, so a flip is a display swap rather than a source assignment — and that is why a table shows the store's own element rather than pointing one of its own at the same picture.

**A view must not start a read before it has painted.** This is the rule the triad kept without anyone noticing, in a single line — *wait for above paint to hit the screen* — and the first version of the window broke it by sliding before the swap. Every flip then blocked on the read of the image entering the window, and a table whose every flip was a cache hit still took 300ms to turn a page. The meter caught it because two numbers matched to the millisecond, again and again: the flip's paint and the next load's read were the same interval seen twice. Show first, then ask.

**The sheet asks for what is visible.** With no queue, a view that asks for five hundred images at once gets five hundred concurrent loads, and that is the view's problem to avoid — ask for what is on screen, ask for more as the user scrolls, release what scrolls away. The sheet knows what is visible and the store never will, which is why the discipline belongs there.

**The sheet also owns its downscale.** A CSS-shrunk `<img>` may or may not be holding a full-size bitmap: Chromium can decode a JPEG at half, quarter, or eighth scale when it knows the paint size, WebKit does some of this, and none of it is observable from JavaScript. For five hundred thumbnails that is the difference between about 180 MB and about 25 GB, and the sheet cannot afford to find out which it got. So it asks the store for a *size*, and the store decodes with `createImageBitmap(blob, {resizeWidth, resizeHeight})`, which can make the small bitmap without ever building the large one — and which returns something with a real `close()`. `Thumbnail.vue` already implements the canvas version of this, coded and never run.

**When the sheet changes thumbnail size, it releases everything and asks again.** No invalidation logic, no store cleverness — a view letting go of one set of references and taking another.

## The safety

**Nothing else can see the total.** A view knows what it asked for; only the store knows what fuji is actually holding, so noticing that something has gone wrong is its one job that is not simply obedience. The point is not to recover — it is to not smile while broken.

**A reference carries a label, not just a count.** `need(path, holder)` and `release(path, holder)`, where the holder is the view and what it is holding for. The cost is a string; the return is that a leak has a name. Three thousand entries and a number tells you fuji is holding 3.2 GB, which is a mystery. The same three thousand with labels tell you the sheet is holding four hundred thumbnails it stopped showing four minutes ago, which is a bug report.

Three signals, and they deserve different reactions:

- **A release that was never needed** — or one more release than there were needs — is an exact programmer error, and it throws, carrying the path and the holder. This is the guard family from `style.md`: a mistake caught at the boundary the moment it happens rather than corrupting quietly below.
- **Referenced but not touched in a long while** is a leak signal, not a certainty — a table legitimately holds its window while the user studies one image for ten minutes. So it warns rather than throws, and the label is what makes the warning worth reading.
- **Past a ceiling** is the HUD saying so plainly, in both units, because a number that only ever grows is not an alarm until somebody looks at it.

**Two calls, because they cost differently.** The footprint is running totals, kept as things are held and let go, so a HUD can read it from the pan path for nothing. Finding trouble means walking every entry, so it is a separate call that a HUD asks rarely and never inside a frame.

**None of these free anything.** The store still only lets go on command. A safety that quietly fixed the problem would be intelligence in the middle again, and would hide the bug it was built to expose.

## What is not stored: bytes as a cache

**The operating system is already the byte cache.** Read a file and macOS or Windows keeps those pages in RAM; read it again and the second read is a copy out of memory. That cache is shared between processes, knows about system-wide memory pressure, and evicts on information fuji does not have. The store holds a blob per live path because the url is made from it — about 6% overhead against a decode — not because re-reading is expensive.

**And the number that made reading look expensive was fuji's own.** 883ms for a 6.2 MB file is not what an M2's SSD does. `disk_read` returned `Vec<u8>`, which crossed the IPC boundary as a JSON array — one decimal number per byte, encoded on one side and parsed on the other. The meter caught it precisely: read time was linear in file size at about **150ms per megabyte**, from 250ms for a 1.6 MB file to 904ms for a 6.2 MB one. `tauri::ipc::Response` carries the same bytes as an ArrayBuffer, and every caller already wrapped the result in `new Uint8Array(...)`, which takes either. That fix was worth more than any byte caching would have been.

## What the library provides

Stateless helpers in `library.js`, holding nothing: read a path into a blob and an object url, decode into an `<img>` at natural size, decode into an `ImageBitmap` at a given size, each recording its own durations. The store calls them; a view could too.

The existing data-url pair stays for now, alongside rather than replaced. A data url is **self-contained** — a string that carries its own bytes and needs no owner — where an object url is a **reference** that means nothing without the blob behind it. That is a real difference, not just convenience, and it may yet find a caller. If it does not, it gets deprecated and then deleted rather than quietly kept.

## What the measurements said

`performance.md` carries everything the running app has measured, with the numbers and the reasoning. What follows is the bench that came before it, kept because two of its lines are still the reason the store holds elements at all — and because one of them was wrong in a way worth remembering.

One question could not be reasoned out: whether an `<img>` that is not in the document keeps its decoded pixels. `img.decode()` answers it, resolving at once for an image already decoded. On a 6240 × 4160 progressive JPEG, 5.95 MiB on disk and 99 MiB decoded, in the tauri webview on macOS:

    1 cold decode, detached element, url1 .......... 925ms
    2 same element decoded again ................... 0ms
    3 new element, same url, first still alive ..... 1ms
    4 new element, same url, others released ....... 0ms
    5 new element, new url over the same blob ...... 911ms
    6 attached element, fresh url .................. 916ms
      detached element, same url as the attached one 0ms

**Line 3 is wrong, and the running app is what proved it.** Wired into fuji, a second element pointed at the same source paid the *whole decode again* — a HUD reading `show` against `render` showed them equal, 960ms against 960ms for the roof photograph. The test elements were detached and never painted, so `decode()` resolved cheaply on a question that had nothing to do with putting pixels on a screen. **Do not trust a rendering measurement taken on something nobody is rendering.**

What lines 2 and 6 do still say is that detachment itself is fine, which is what makes a store of elements viable. Line 5 — a fresh url over the same blob costing a full decode — is why one url per path is made once and kept rather than remade.

**The real finding, from the app rather than the bench: what makes re-display instant is the same element, already decoded.** The old triad was not caching an image, it was caching a rendered element — a stronger and more specific thing than anyone here understood at the time.

Line 4 carries a caveat: releasing both elements left the decode alive, but JavaScript cannot force collection, so that may only mean it had not been collected yet. It says nothing about memory pressure, which is the condition that matters when fuji is holding a lot and the one this test could not create.

## The first iteration

Small on purpose, and meant to produce measurements rather than to be right about policy.

- **Nothing expires on its own.** The store frees only on command. A file at a path is assumed to be the same file forever.
- **Loading is raced.** No queue, no workers, no priority.
- **The footprint is on the HUD** — blobs and pixels counted separately, because they differ by more than an order of magnitude and rebuild at completely different costs. Not a limit; an instrument.
- **Two callers at once share one load**, failures are remembered with their detail, and one url per path is made once. Each of those is a correctness property rather than an optimisation.

## Open

**What a table's window should cost.** Size and rebuild time are independent: a 26-megapixel image is 99 MiB decoded whether it took 60ms or 960ms, and a badly compressed photograph costs the same memory as a beautiful one. So a policy that only counts bytes will throw away the expensive decodes as readily as the cheap ones. Which weights matter cannot be settled before fuji has been dragged through real galleries, real folders of saved torrents, real photograph libraries — recording every input now is what makes that experiment possible later.

**Whether holding eleven images costs anything per flip.** Measured but not yet answered: the HUD reports the flip's wall clock and the frame count beside it. Compare `flip.back` and `flip.forward` at 5 against 1 on a folder of large photographs. If the flip is the same either way, what remains is background loading and the window should ask more gently; if it climbs at 5, the window wants a byte budget rather than a count.

**What a flip costs now.** The first run said: every flip a hit, `store` at 0, and `paint` between 134 and 372ms — all of it the blocked read described above. With the read fixed and the order corrected, the same walk should report `paint` at about one frame. If it does not, the questions below are where to look next.

**Whether a hidden element keeps its pixels.** This is the question the url ownership above is waiting on, and the HUD now splits a flip into `store` and `paint` to answer it: a store hit costs no time at all, so a large `paint` is the engine rebuilding an image the store believes it already has. If that number is large, holding an `<img>` is not holding a decode and the store is keeping the wrong thing.

**Whether that is a difference at all.** An engine dropping the frames of images nobody is showing would do it to the triad exactly as readily as to the store — same elements, same `display: none`, same lack of any way to ask. So it cannot be what makes one design feel faster than the other, and if the two feel different the cause is somewhere fuji controls. Worth knowing before reading anything into a comparison.

**Whether retention holds under pressure.** The measurements were taken on an idle machine with one image. When fuji is holding hundreds, the engine may begin discarding decodes and rebuilding them, and a hit that costs a millisecond today could cost 900ms then, invisibly.

**Whether Windows answers the same way.** WebView2 is Chromium and macOS is WKWebView, and line 5 especially is the sort of thing two engines could reasonably differ on.

**What the sheet's thumbnail sizes are**, and whether a resize really means releasing everything or whether a larger decode can serve a smaller display without the full-size problem coming back.

**Identity.** Paths, with the assumption that a file at a path never changes. Hashing the bytes with SHA-256 would give a second index by content, so one entry serves every path naming the same file, and it is also where invalidation starts, since a changed file is a changed hash.

**Whether a hit should be synchronous**, so a view that already has an image can paint in the same frame instead of after an await.
