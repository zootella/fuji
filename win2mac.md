# From the Windows box to the Mac: the two names, through the builds only the Mac runs

Written on the Windows box, 2026-10-03, alongside the change that gives each of the product's two names one source. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one.

## What changed

The product has two names, and the code now keeps them apart by two words. **`brandName`** is `Fuji`, `productName` in `tauri.conf.json`, the name people read. **`brandStem`** is `fuji`, the crate's name in `Cargo.toml`, the stem of the executable's name. Neither is made from the other. The README's new section, The two names, says where each is set, and the essay atop `desktop/src/brand.js` says where each goes. `style.md` has the rule.

Before this, the build scripts typed both names out: `scripts.js` spelled `Fuji_` and the six `fuji.*` published names, `linux/build.js` looked for a capital `Fuji`, and `inside-flatpak.sh` spelled `Fuji`, `fuji` and a second copy of the identifier. Now they read the two files instead. `dmg.js` already read `productName`, and only its variable is renamed. `linux/build.js` hands the flatpak container both names and the identifier as arguments, since the container sees neither file. Two source files lost the name from their own filenames: `src-tauri/fuji.desktop.hbs` is now `src-tauri/desktop-entry.hbs`, and `icons/tile/fuji.VisualElementsManifest.xml` is now `icons/tile/VisualElementsManifest.xml`, still copied beside the executable as `fuji.VisualElementsManifest.xml`.

It is meant to be a pure refactor: every file a build makes keeps the name and the contents it had.

**To see all of it**, diff from `3b1b8b1`, the linux release, which is the last commit before the change. The change is the commit after it, whose subject begins "brandStem reads the crate's name". It had not been made when this was written, so `git log --oneline 3b1b8b1..` finds its hash, and `git diff 3b1b8b1 <that hash>` shows all of it.

## What was checked on Windows

- `pnpm vite-build`, and Vite's dev pipeline loading `brand.js`, both give `Fuji` and `fuji`.
- `cargo check` passes, and its build script copied the renamed tile manifest to `target/debug/fuji.VisualElementsManifest.xml`.
- `scripts.js` and `linux/build.js` were loaded beside their versions at `3b1b8b1`, without running them. Both versions give the same six targets and published names, the same three image tags, and the same build found in each folder.
- `inside-flatpak.sh` was run, old and new, with stand-in commands that log their arguments. Both run the same commands with the same arguments, except that the deb is now unpacked under its own name rather than as a copy called `fuji.deb`.

## What only the Mac can check

Three of the changed pieces run only on the Mac, so none of them has done a real build yet.

1. **`dmg.js`.** Run `pnpm installer` in `desktop`. Expect `bundle/dmg/Fuji_0.1.0_aarch64.dmg` and a volume titled Fuji, holding Fuji.app and the Applications link, as at `dded8fd`. If the dmg is missing or named otherwise, look at `brandName` in `dmg.js`. Leave `pnpm hash` for a real release: it would test the prefix `scripts.js` now builds from `brandName`, but it also rewrites the committed `fuji.dmg.json` with this build's hash, and that same prefix was already tested on Windows, where it found the NSIS installer.

2. **The two Docker steps.** Run `pnpm build` in `linux`. The image tags are now spelled from `brandStem` and should come out as before, `fuji-tauri:arm64`, `fuji-tauri:amd64` and `fuji-flatpak:amd64`. Docker's cache would hide a changed tag, since it reuses layers by their contents whatever they are called, so afterwards `docker images` should list those three and no new ones. Expect the same four files the linux release made: `Fuji_0.1.0_arm64.deb`, `Fuji_0.1.0_amd64.deb`, `Fuji-0.1.0-1.x86_64.rpm` and `Fuji_0.1.0_x86_64.flatpak`. The flatpak step's log should show `usr/bin/fuji`, `Fuji.desktop` and the icons laid out under `app.fujidesktop.Fuji`, exactly as before. If it stops with "say the …", an argument from `build.js` did not arrive.

3. **The renamed desktop template.** The debs and the rpm are now built from `desktop-entry.hbs`, whose bytes did not change, so the desktop entry inside a deb should be byte for byte the one the last release shipped: `usr/share/applications/Fuji.desktop`, with `Name=Fuji`, `Exec=fuji` and `Icon=fuji`. `ar x` and then `tar -xf data.tar.gz` pull it out of a deb on the Mac. The staged `linux/release/fuji.amd64.deb`, if it is still there, is the last release's build, and stays that until `pnpm hash` copies a new one over it, so it is what to compare the new `Fuji_0.1.0_amd64.deb` against. Leave `pnpm hash` here for a real release too, since it rewrites the committed sidecars.

These are test builds. Whether any of them becomes a release is the user's decision.

If all three come out the same, there is nothing to write down: delete this letter and its line in `contents.md`. If one differs, the place to look is named in its step, and what you find belongs in the comment beside that code.
