# Security

Where untrusted bytes are parsed in fuji, what stands between them and the user's files, and what to build, in order. `disk.rs` carries the contract for the disk commands and is the other half of this; `canvas.md` says which path decodes which format and why.

## The threat

**A file with an image extension is not necessarily an image, and an image is not necessarily honest.** Fuji's whole job is to open thousands of files the user did not make and hand each one to a decoder that has to parse it. Image decoders are among the most attacked code on any machine, because they run on untrusted input by design: GDI+ in 2004, WebP's decoder in 2023, which was in every browser and every operating system at once, and ImageIO in 2025, exploited in the wild through a camera raw format. The user has lost a machine this way. Fuji has to assume it will be handed such a file eventually, by a user who has thousands of them and no way to know.

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

The page's paths are sandboxed and the IPC is not scoped. `thumbnail.rs` is called by `SquareFlow` for the formats on each platform's allow list, is unsandboxed, and holds walls two and three: it refuses bytes it does not know and headers it could not fit. The content security policy in `tauri.conf.json` allows only fuji's own scripts and its ipc, and blob and data urls for images. Walls one, four and five are not built, and the first is the one that matters most.
