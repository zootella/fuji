# Security

How fuji safely decodes pictures it did not make, when an extension may be missing or wrong and the bytes may be corrupt or hostile. Where those bytes are parsed, what stands between them and the user's files, and what to build, in order. `disk.rs` carries the contract for the disk commands and is the other half of this; `canvas.md` says which path decodes which format and why.

## The threat

**A file with an image extension is not necessarily an image, and an image is not necessarily honest.** Fuji's whole job is to open thousands of files the user did not make and hand each one to a decoder that has to parse it. Image decoders are among the most attacked code on any machine, because they run on untrusted input by design: GDI+ in 2004, WebP's decoder in 2023, which was in every browser and every operating system at once, and ImageIO in 2025, exploited in the wild through a camera raw format. The user has lost a machine this way. Fuji has to assume it will be handed such a file eventually, by a user who has thousands of them and no way to know.

## What fuji thinks a file is, which is a subject of its own

**Fuji answers "what is this file" twice, in two places, by two different means, and they never compare notes except at one seam.**

*By name.* `imageTypes` in `library.js` maps ten extensions to formats. It decides what `listFolder` keeps out of a folder, how `SquareFlow` routes a tile, what the store types its blobs as, and what fuji declares to each operating system.

*By bytes.* `thumbnail_probe` reads a file's first bytes and says what it actually is, without decoding it, and `thumbnail_render` refuses a file whose bytes are not the format it was told to expect.

**They meet at one seam, and disagreement is already treated as an error.** `SquareFlow` works the format out from the name, hands it to the probe, and a mismatch refuses the tile — *the bytes say png and the name says jpeg*. That is wall two below and it should stay exactly as it is: a decoder handed bytes it did not expect is the case the wall exists for.

**Three cases follow, and only two of them are handled.**

1. **The name is right.** Everything works, which is almost always.
2. **The name is wrong.** The probe catches it and the tile is refused. Safe — and a picture fuji could have shown perfectly is not shown. That trade is deliberate and, for now, correct.
3. **There is no name to go on.** `listFolder` keeps only files whose extension is in `imageTypes`, so a file with no extension is dropped before anything asks what it is. Fuji never reaches the machinery that would have identified it correctly in a millisecond. **This is the open case**, and it is not exotic: pictures arrive without extensions from downloads, exports and messaging apps all the time.

**A fourth case is about robustness rather than safety.** A file that genuinely is a JPEG but is truncated or damaged: the decoder fails, the tile shows the error placeholder, and the log says why. That works and needs nothing. What nobody has decided is whether a partial decode should show the part that arrived, the way a browser does, or keep refusing whole.

**Case three is not a small change, which is why it is written down rather than done.** The probe already takes a list of paths and answers per path, so the mechanism exists. The cost is policy. `listFolder` runs every time a folder opens and the user waits on it, so asking Rust about every unknown file puts a round trip on that path — free in a folder of pictures, where there are no unknown files, and wasteful in a Downloads folder full of archives and installers.

The shapes worth weighing: probe only files with no extension at all, since a wrong extension is already handled; or probe lazily, only when a folder yields no pictures; or leave fuji extension-driven and say so.

**One visible consequence exists today.** File → Open… deliberately lists every file rather than only the ten fuji knows, so a user can choose a picture saved without an extension — and land on some other picture in that folder, because the file is not in fuji's listing at all. `menu.md` records why the picker is unfiltered; this document owns why the file cannot be opened.

**Where the work belongs.** This changes what a folder *contains*, so it is `listFolder` and the model rather than the decoding path or desktop integration. A future pass should take it together with the walls below, because identification and safe decoding are the same subject seen from two ends.

## Where the decoding runs

**Three paths, and only one of them is sandboxed by anyone but fuji.**

    the table, an img in the page          the web engine's content process, sandboxed on the mac and on windows
    the sheet's page route, drawImage      the same process
    thumbnail.rs                           fuji's own process, which nothing sandboxes

On the Mac the decoder is the same library either way — WebKit decodes through ImageIO — so the bugs are identical and only the process differs. On Windows they are different code, Chromium's own decoders, among the most fuzzed in the world, against WIC's, patched by Windows Update. Both are patched without fuji shipping anything, which is the one thing fuji gets right by never carrying a decoder of its own.

**The sandbox is not protecting fuji as things stand.** A process that has taken over the page can invoke every command `lib.rs` registers, and `disk_read`, `disk_write` and `disk_copy` take any path and hold no guard — `disk.rs` says so, deliberately, resting safety on paths that arrive from user gestures. A compromised renderer needs no gesture. So the sandboxed path and the unsandboxed one end in the same place today, the user's files, and the difference between them is one step. That is a reason to narrow the commands. It is not a reason to keep decoding in the page, which is why `canvas.md` leans on the operating system.

## The walls to build, in order

1. **A path scope in Rust, for every command that touches the disk.** A registry of the folders the user has dragged in or chosen, kept on the Rust side, with read, copy, write and thumbnail refusing anything outside it. `disk.rs` planned this for the delete family; it belongs to all of them, and it is the largest single change to fuji's posture available. With it, a compromised renderer reaches only what the user already showed fuji.
2. **The first bytes decide the format, before any decoder sees the file.** Built, 2026-09-08: `thumbnail_probe` names every file from its first bytes and `thumbnail_render` refuses a file whose bytes are not the format it was told to expect, whatever the extension. This turns "any file ImageIO or WIC will parse", sixty formats on the Mac, into the handful fuji actually shows. `thumbnail.rs` has the signatures.
3. **A size ceiling from the header, before any decode allocates.** Built, the same day: a header claiming a raster over half the machine's physical memory is refused by both commands. The decompression bomb is the attack that needs no bug, and the ceiling is a share of the machine rather than a number, so it never limits capable hardware and refuses only what could not have fit.
4. **The rarer the format, the more sandboxed its path.** The routing table sends the mainstream formats native and keeps the odd ones in the page or refuses them. The exotic decoders are where the bugs live, and the page is the process that can afford them.
5. **On the Mac, when isolation matters more than the module's simplicity, QuickLook's thumbnail generator.** It decodes in a sandboxed system agent, handles every format QuickLook does, and keeps a cache on disk. It is the way to go native and keep a sandbox, at the price of Objective-C from Rust.

## What is true today

The page's paths are sandboxed and the IPC is not scoped. `thumbnail.rs` is called by `SquareFlow` for the formats on each platform's allow list, is unsandboxed, and holds walls two and three: it refuses bytes it does not know and headers it could not fit. The content security policy in `tauri.conf.json` allows only fuji's own scripts and its ipc, and blob and data urls for images. Walls one, four and five are not built, and the first is the one that matters most. A picture that arrives with no extension is invisible to fuji, which is the open case above and belongs to the same pass.
