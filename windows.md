# To the Windows box: installer, hash and upload, now with a build log

Written on the Mac mini, 2026-10-11, alongside the new apple workspace. Nothing here asks for work. It says what changed and what the first run will look like, so that none of it is a surprise.

## What changed

**The Mac release now goes through Apple, from a fourth workspace, `apple`, and none of it runs here.** `pnpm install` at the root installs its one dependency, which nothing on this machine uses, and `pnpm notarize` in desktop is the Mac's alone.

**`pnpm installer`, `pnpm hash` and `pnpm upload` publish the setup program as they always have, and now also log their steps.** Each step writes a line to `build-log.txt` at the repository root as it starts and as it ends. The file is gitignored, so each machine keeps its own. The code is `build-log.js`, beside `scripts.js`, and its essay describes the line format. `win-setup.js` and `scripts.js` both import it. It runs nothing when imported, so `scripts.js` still runs its verb exactly as before.

**Times people read are now written like `2026-Oct-11 Sun 02:45:07.123 PM`** across the pipeline, while the sidecar's date and the log's first field stay ISO, in UTC.

## The first run

`pnpm installer` runs Tauri's build as before. Then `node dmg.js` returns at once without a word, since a dmg is the Mac's alone. Then `win-setup.js` prints its usual lines between two new ones:

```
2026-Oct-11 Sun 02:45:07.123 PM  exe started  of the program tauri just built, by pnpm installer
2026-Oct-11 Sun 02:45:13.456 PM  exe finished  Fuji_0.1.0_x64-setup.exe  <sha256>  in the bundle folder, where pnpm hash finds it, took 6.3 seconds
```

`pnpm hash` prints the line it always printed, between `hash started fuji.exe` and `hash finished fuji.exe <sha256> took …`. `pnpm upload` prints scp's output, between `upload started fuji.exe` and `upload finished fuji.exe <sha256> and fuji.exe.json, took …`.

`build-log.txt` then holds those six lines. Each carries the commit the checkout was at, followed by a + when tracked files had changed since. Hash changes the sidecar, so the upload lines will carry the + until the sidecar is committed. Without git on the path, the commit reads `-` and nothing else changes.

If anything in those three commands mentions a dmg, Apple or notarization, that is new, so tell the user. Otherwise, retire this letter, as its predecessors were.
