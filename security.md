# Security

How fuji stays safe, correct and responsive when it is pointed at files nobody vouches for, on storage nobody promised would answer. The three ways any call can end, what fuji's job makes of each, where fuji stands on each today, and what to build. `disk.rs` carries the contract for the disk commands, `lib.rs` argues the walls around the page, and `canvas.md` says which path decodes which format and why.

The state of the code described here was read from the source on 2026-10-03, on the Mac mini, and none of the gaps below has been reproduced on a running machine unless it says so.

## Three outcomes

**Every call that reaches past the code we control ends one of three ways: it succeeds, it fails, or it never ends.** It does not matter whether the far side is a web service across the world or a disk across a cable. The moment code asks something it does not own, it has given up knowing what comes back, and the code that asked is the code responsible for every answer it might get.

**Success is the one everybody writes and everybody tests.** The happy path is short, it is the path the author had in mind, and on the author's machine it is almost the only path that ever runs. Most software that breaks was correct on the happy path.

**Failure is larger than success, and it has layers.** The first is the failure the far side admits to: an error code, a result marked as failed, a message saying why. The second is the failure that arrives by another channel: an exception thrown instead of a value returned, a panic in a library, a crash in another process — the far side failing to fail properly. The third is the wrong answer that looks right: a file named `.jpg` that is a PNG, a header claiming a size the pixels do not have, a listing that leaves an entry out. And the fourth is malice: input built on purpose to make the far side do something its authors never meant, which is where a sad path becomes a security hole. The far side's failures are not ours to enumerate, and the next version of it will add some. So code anticipates them by shape rather than by list: every call down crosses a gate where anything that comes back, however it comes back, becomes a value the code around it can reason about. `style.md` calls that the walled city, and its bottom gates are this.

**The third outcome is that nothing happens.** The call goes out and no answer comes back — no value, no error, no exception. The spark of execution that asked is still holding the line. Nothing has failed, so nothing reports a failure, and every test written on a fast local disk passed. What makes it dangerous is not the call itself but what is waiting behind it: the loop slot it occupies, the queue it heads, the thread it holds. One call that never ends can stop everything that was waiting its turn, and the program looks frozen while every part of it is working exactly as written.

A careful program has an answer for all three at every place it calls out. An answer to the third is rarely "wait forever", and it is never "it doesn't happen here".

## The mission: collections nobody vouches for

**Fuji will be pointed at drives holding decades of collecting, and no file on them has a trusted origin.** Pictures pulled down from a BBS at 2400 baud, binaries decoded out of Usenet, clips off a CD-ROM, disks copied from disks copied from floppies; then forums, imageboards, Reddit and Discord; then exports from encrypted chats with strangers. Each one passed through hands and programs nobody can name now. Its name may lie, because people renamed files and programs wrote whatever extension they liked. Its bytes may be damaged, by a transfer that dropped a packet in 1994 or a disk that has been dying for a decade. And some of them may be hostile, made to break whatever opens them.

**A file with an image extension is not necessarily an image, and an image is not necessarily honest.** Fuji's whole job is to open thousands of these and hand each one to a decoder that has to parse it. Image decoders are among the most attacked code on any machine, because they run on untrusted input by design: GDI+ in 2004, WebP's decoder in 2023, which was in every browser and every operating system at once, and ImageIO in 2025, exploited in the wild through a camera raw format. The user has lost a machine this way. Fuji has to assume it will be handed such a file eventually, by a user who has thousands of them and no way to know.

**The storage is no more trustworthy than the files.** A collection like this lives on a USB hard drive that spins down after ten minutes, a NAS across the house, a network share that drops when the laptop sleeps, a CD or a floppy in a reader that retries a bad sector for minutes before it gives up, or a cloud folder whose files are placeholders that download when something reads them. For fuji, "the distant thing" is rarely a server. It is the disk, and the disk is where the third outcome lives.

**Privacy and precision ask the same of fuji here.** A file manager made for privacy cannot be the program that hands a stranger's file a way into the machine, and one made for precision cannot freeze on a scratched disc or quietly show the wrong picture. Every read and every decode in fuji is a call to a distant thing, and each has to have an answer for all three outcomes.

## Success

Fuji's happy path is the one that has been built and measured: the listing, the store, the contact sheet's two routes and the tables. `site/docs/thumbnails.md` has what it costs and how it was measured.

## Failure

### What fuji does with an error

**Most of fuji's calls down already turn failure into a value, and the views already carry on past one.**

- **Rust answers with a value.** Every command returns a `Result`, and an error crosses to the page as a rejected promise carrying a sentence. `disk_readdir` skips an entry it cannot read or stat rather than failing the folder over it.
- **Two Mac bodies catch a panic.** `thumbnail.rs` and `panel.rs` call crates that assert rather than return when CoreGraphics or CoreFoundation will not cooperate, and both wrap that work in `catch_unwind`, so a panic there comes back as an error.
- **The store remembers a failure.** `cache.js` records a failed read or decode on the entry, so one broken file is not read again on every pass, and every caller is answered with the entry and its error rather than with a throw.
- **The views refuse a file and move on.** The contact sheet shows a placeholder and logs which file and why. The diamond table's queue catches a failed flip or drop, logs it, and goes on with the next one. The shell routes a drop and an open through gates that log what escaped.

**Three gaps, all read from the code.**

- **A panic outside `catch_unwind` may become a call that never ends.** The Windows thumbnail body, the disk commands and the rest have no such wrapper. Tauri runs an async command as a task on its pool, and a panic ends that task; the belief is that the page's promise is then never answered, which would turn the second layer of failure into the third outcome. That is assumed from how Tauri is built and has not been watched happening. A panic in any of those bodies should be rare, since they call `std::fs` and WIC through `Result`s, but the place to settle it is one command that panics on purpose.
- **A rejection that escapes every gate goes nowhere anyone looks.** The page has no `unhandledrejection` or `error` listener on the window, so a promise rejected outside the gates above lands in the web inspector's console, which neither the user nor a session ever reads. One listener writing to the log is the page's top gate, and fuji does not have it yet.
- **A read is as large as the file.** `disk_read` reads a whole file into memory, and the page holds it more than once on the way to a blob. The size ceiling guards a decode by the raster its header claims; nothing guards a read by the size of the file. A twelve-gigabyte disk image renamed `.jpg` would be read whole the moment a table or the page route asked for it. The listing already knows every file's size, so a limit costs nothing to check.

### What fuji thinks a file is

**Fuji answers "what is this file" twice, in two places, by two different means, and they compare notes at only one seam.**

*By name.* `fileTypes` in `fileTypes.js` maps ten extensions to formats. It decides what `listFolder` keeps out of a folder, how `SquareFlow` routes a tile, what the store types its blobs as, and what fuji declares to each operating system.

*By bytes.* `thumbnail_render` reads a file's first bytes before any decoder sees it, and refuses a file whose bytes are not the format it was told to expect.

**They meet at one seam, and disagreement is treated as an error.** `SquareFlow` works the format out from the name and hands it to the render, and a mismatch refuses the tile — *the bytes say png and the name says jpeg*. That is wall two below, and it should stay exactly as it is: a decoder in fuji's own process handed bytes it did not expect is the case the wall exists for. The seam is on the native route only. The page route and every table hand the file to the web engine, which chooses its decoder by the bytes rather than the name, inside its sandbox.

**Four cases follow, and the third is open.**

1. **The name is right.** Everything works, which is almost always.
2. **The name is wrong.** On the native route the render catches it and the tile is refused. Safe — and a picture fuji could have shown perfectly is not shown. That trade is deliberate and, for now, correct. On the page route and on a table, the engine decodes by the bytes and shows the picture if it can.
3. **There is no name to go on.** `listFolder` keeps only files whose extension is in `fileTypes`, so a file with no extension is dropped before anything asks what it is. Fuji never reaches the machinery that would have identified it correctly in a millisecond. Pictures arrive without extensions from downloads, exports and messaging apps all the time, and from the oldest collections, written before extensions meant much, more often still.
4. **The name is right and the bytes are damaged.** A truncated or corrupted JPEG: the decoder fails, the tile shows the placeholder, and the log says why. That works and needs nothing. What nobody has decided is whether a partial decode should show the part that arrived, the way a browser does, or keep refusing whole — which matters more for a collection whose files have been decaying for thirty years.

**Case three is not a small change, which is why it is written down rather than done.** Naming a file by its bytes means opening it, and `listFolder` runs every time a folder opens and the user waits on it, so reading every unknown file puts a disk read on that path — free in a folder of pictures, where there are no unknown files, and wasteful in a Downloads folder full of archives and installers. The shapes worth weighing: read only files with no extension at all, since a wrong extension is already handled; or read lazily, only when a folder yields no pictures; or leave fuji extension-driven and say so. The most natural home for an answer is the look-ahead that `thumbnail-open.md` scopes as a later system, which would read the first bytes of files ahead of the view and keep what it learns in a database, so a file with no extension would be named once and remembered. Identification and safe decoding are the same subject seen from two ends, so whichever pass takes this takes the walls below with it.

**One visible consequence exists today.** File → Open… deliberately lists every file rather than only the ten fuji knows, so a user can choose a picture saved without an extension — and land on some other picture in that folder, because the file is not in fuji's listing at all. `menu.md` records why the picker is unfiltered; this document owns why the file cannot be opened.

**Names have a smaller version of the same problem.** `disk_readdir` hands names to the page as text, and a name that is not valid UTF-8, which Linux allows and old collections produce, is replaced character by character on the way and can no longer be opened by what came back. And a dropped path is forwardized, which turns a backslash into a slash — correct on Windows, and on Linux or a Mac, where a backslash is an ordinary character in a name, a different path. Both are read from the code. Neither is dangerous, since the read that follows fails into a placeholder, but both are precision fuji does not yet have.

### Malice: where the decoding runs

**Three paths, and only one of them is sandboxed by anyone but fuji.**

    the table, an img in the page          the web engine's content process, sandboxed on the mac and on windows
    the sheet's page route, drawImage      the same process
    thumbnail.rs                           fuji's own process, which nothing sandboxes

On the Mac the decoder is the same library either way — WebKit decodes through ImageIO — so the bugs are identical and only the process differs. On Windows they are different code, Chromium's own decoders, among the most fuzzed in the world, against WIC's, patched by Windows Update. Both are patched without fuji shipping anything, which is the one thing fuji gets right by never carrying a decoder of its own.

**The sandbox is not protecting fuji as things stand.** A process that has taken over the page can invoke every command `lib.rs` registers, and `disk_read`, `disk_write` and `disk_copy` take any path and hold no guard — `disk.rs` says so, deliberately, resting safety on paths that arrive from user gestures. A compromised renderer needs no gesture. So the sandboxed path and the unsandboxed one end in the same place today, the user's files, and the difference between them is one step. That is a reason to narrow the commands. It is not a reason to keep decoding in the page, which is why `canvas.md` leans on the operating system.

**What else stands today.** The content security policy in `tauri.conf.json` allows only fuji's own scripts and its IPC, and blob and data URLs for images. Untrusted text — a file's name, a path — reaches the page only through Vue's escaping interpolation, so it never becomes script. An SVG is shown only in an `<img>`, where its scripts never run.

### The walls to build, in order

1. **A path scope in Rust, for every command that touches the disk.** A registry of the folders the user has dragged in or chosen, kept on the Rust side, with read, copy, write and thumbnail refusing anything outside it. `disk.rs` planned this for the delete family; it belongs to all of them, and it is the largest single change to fuji's posture available. With it, a compromised renderer reaches only what the user already showed fuji. Not built.
2. **The first bytes decide the format, before any decoder sees the file.** Built, 2026-09-08: `thumbnail_render` refuses a file whose bytes are not the format it was told to expect, whatever the extension. This turns "any file ImageIO or WIC will parse", sixty formats on the Mac, into the handful fuji actually shows. `thumbnail.rs` has the signatures.
3. **A size ceiling from the header, before any decode allocates.** Built, the same day: the render refuses a header claiming a raster over half the machine's physical memory. The decompression bomb is the attack that needs no bug, and the ceiling is a share of the machine rather than a number, so it never limits capable hardware and refuses only what could not have fit. The page route and the tables decode in the web engine without it, which is the sandbox's job there rather than fuji's; if it is wanted on that side, the store is the place, since every page decode passes through it.
4. **The rarer the format, the more sandboxed its path.** The routing table sends the mainstream formats native and keeps the odd ones in the page or refuses them. The exotic decoders are where the bugs live, and the page is the process that can afford them. A policy rather than code, and it holds today because both native lists are short.
5. **On the Mac, when isolation matters more than the module's simplicity, QuickLook's thumbnail generator.** It decodes in a sandboxed system agent, handles every format QuickLook does, and keeps a cache on disk. It is the way to go native and keep a sandbox, at the price of Objective-C from Rust. Not built.

## Forever

**No call in fuji has a timeout.** Not one invoke, read, or decode is ever abandoned for taking too long: each waits until it is answered. On the local disks fuji has been developed on, everything is answered in milliseconds, which is exactly why this has never shown.

**Where a call that never ends does its damage, read from the code:**

- **The diamond table stops.** Every flip and every drop goes through one queue, one at a time, which is right — `DiamondTable.vue` says why a drop must never land inside a flip. But the queue moves only when the work at its head settles. A load that never settles stops every flip and every drop after it, and the table is frozen until fuji is restarted. The queue already survives a failure; it does not survive a silence.
- **A governor's line stops.** Every governor in `governor.js` lets four calls through at a time, so four calls that never settle hold all four places, and everything behind them in that line waits for good. The store reads under one name, `disk`, for every view, so four reads stuck on a dead share stop the contact sheet and the table's own reads together.
- **Rust's pool fills up.** An async command runs on Tauri's pool, one worker per core, and a body blocked in `std::fs` holds its worker until the read returns. A handful of reads stuck on a dead share can hold every worker, and then every command waits, including ones that have nothing to do with that disk. `lib.rs` names the next step: `spawn_blocking`, onto Tokio's larger pool meant for blocking work.

**Where forever comes from, for fuji.** A network share that dropped mid-read. A disk spinning up after sleeping, which is seconds, and is not forever but feels like it. A cloud placeholder file, which reads by downloading. A failing CD or floppy, where the drive retries a bad sector for minutes. And a decoder stuck in a loop on a malformed file, which is the place where the third outcome meets malice: a file that does not crash a decoder but keeps it busy forever is a denial of service, and needs no bug that leaks anything.

**A timeout in the page does not end the work.** A read blocked in the kernel cannot be cancelled from Rust, let alone from the page, and a decoder running on a pool thread runs until it returns. So the answer to forever is containment rather than cancellation: the view stops waiting, shows what it has, and moves on, while the stuck call is left to finish or not on a thread that nothing else needs. That shapes what to build — the page decides how long it will wait for each kind of call, and Rust makes sure one stuck call cannot take the threads the others need.

## What to build next, by outcome

Candidates rather than decisions, gathered here so the next pass can take them together.

- **Failure.**
  - A top gate for the page, writing every escaped rejection and error to the log.
  - Every command turning a panic into an error, so a panic can never become a silence; first, one command that panics on purpose, to settle what Tauri does today.
  - A size limit on a read, checked against the listing's size before `disk_read` is asked.
  - Names that survive the round trip: paths carried as the operating system's bytes rather than as text, and forwardizing only on Windows.
- **Malice.** The walls above, starting with the path scope.
- **Forever.**
  - A deadline in the governor, after which it frees the place and fails the caller's promise, so the view treats the call as failed and moves on: a refused tile, a placeholder in the table, the line free for the next call. The table's own queue needs the same, since a flip waits on a read.
  - `spawn_blocking` for every command that waits on a disk or a decoder, so stuck calls cannot starve the rest.
  - A way to reproduce it on demand — a folder on a share that can be pulled out from under fuji, or a file system that answers slowly on purpose — because forever cannot be tested on the disks fuji is developed on.
