# Letter to windows claude

Hello — this is from the Claude Code session that worked with the user on fuji's icon, on macOS, 2026-09-07. The user has pulled this repo onto their Windows 10 workstation and pointed you here. The mac half of the icon work is done and committed; the Windows half is yours. Read `icon.md` first — it is the whole research record, and this letter only says what is left.

## Ground rules (these matter — they are not in your memory yet)

- **Git**: read-only git commands are free (status, log, diff, show). Every mutating command — add, commit, push, pull — is the user's alone. When it is time to commit, end your response with one line: 📌 followed by the suggested commit message in boldface, nothing else on that line.
- **Style**: read `./style.md` before writing or editing any code. One rule to know even before you read it: `ttd` comments are the user's alone — never add, reword, or delete one.
- **Private files**: names with "hide" as a dot-separated part (`hide/`, `*.hide`, `hide.*`, `*.hide.*`) are gitignored private planning docs. Respect that boundary.
- **Documents say "the user"**, never a name.

## What happened on the mac

Two separate defects, both in the icons, neither of them the user's fault:

1. **A grey fringe on every curved edge**, on every platform including yours. A real Tauri bug ([#14351](https://github.com/tauri-apps/tauri/issues/14351)), fixed upstream in `@tauri-apps/cli` 2.9.3 in November 2025. Fuji's icons were generated in July 2025 and never regenerated, so upgrading the CLI never reached them.
2. **The macOS icon was 100% of its canvas** where Apple's grid asks for 824 of 1024. `tauri icon` has no macOS padding option and never will — the maintainers tried and abandoned it.

The fixes: everything was regenerated on the current CLI, which cleared the fringe everywhere, and macOS got a second source, `app-icon-mac.svg`, identical to the shared one but `r="412"` instead of `r="512"`. It generates into `src-tauri/icons/mac/icon.icns`, which `tauri icon` never writes to, and `bundle.icon` points at that file. `pnpm icons` does both runs.

**The icons in the repository are fresh as of 2026-09-07**, so they do not need rebuilding for age. What is worth testing is whether they *reproduce* on your machine — see below.

## Your sequence

1. **Check `pnpm icons` is genuinely cross-platform.** It should be — `tauri icon` is pure Rust and writes `.icns` on any host, and nothing in the script shells out to a mac-only tool. Run it and then `git status`. **Byte-identical output means no diff at all.** Any diff is a finding worth reporting rather than committing, because it would mean the icons churn every time a different machine touches them.

2. **Confirm the fringe is actually gone on Windows.** The `.ico` was the worst of the six-layer set. Look at the taskbar, the desktop shortcut, Explorer at several view sizes, alt-tab, and the title bar, against both light and dark backgrounds. The old symptom was a dirty grey outline on the circle, most visible against white.

3. **Judge the `.ico` sizing, which is the open question `icon.md` leaves you.** Fuji's Windows icon is full bleed — the disc touches all four edges — and that is deliberate, not an oversight: Microsoft's guidance puts artwork at roughly 90%+ of its canvas, and an app at 78.9% has been [reported as a bug](https://github.com/opensquilla/opensquilla/issues/982) for looking too small. So Windows wants close to the opposite of macOS. But "close to full bleed" is not the same as "full bleed", and nobody has looked at fuji's on a real Windows 10 taskbar. If it wants a small margin, the fix is the same shape as the mac one: a third source, `app-icon-win.svg`, at whatever radius looks right. Measure before deciding — `icon.md` describes the alpha-bounding-box method and the numbers for fourteen shipping applications.

4. **The Start menu medium tile, which is the real work.** Today fuji's tile will be the default: the right size, with the icon sitting small and lonely in the middle of it. The user wants what Firefox has — artwork filling the tile properly, on a background colour. This is a Windows 10 feature and the user believes Windows 11 does not benefit; confirm that rather than assuming it.

   The mechanism to research is a **`VisualElementsManifest.xml` named for the executable** — `fuji.VisualElementsManifest.xml` — sitting **directly beside `fuji.exe`** in the install directory. It carries `Square150x150Logo` for the medium tile, `Square70x70Logo` for the small one, `BackgroundColor`, `ForegroundText`, and `ShowNameOnSquare150x150Logo`. Firefox ships exactly this; its file and Mozilla's bug history are the best worked examples.

   Two things to verify rather than trust:
   - **Whether `BackgroundColor` still works on Windows 10.** Sources disagree. You have the actual machine, so settle it empirically.
   - **How to get the file beside the exe through Tauri's bundlers.** `bundle.resources` exists but places files in a `resources/` subfolder, and the manifest must be a sibling of the exe, not a child directory. That may mean a custom WiX or NSIS template (`bundle.windows.wix.template`, `bundle.windows.nsis.template`) — both are supported config, and both are heavier than fuji has needed so far. **If it turns out to need a lot of machinery, say so and stop rather than building it**; the user values simplicity over the feature, and would rather hear the honest cost than get a large diff.

   Note that fuji already ships the `Square*Logo.png` set that `tauri icon` generates. Those are Microsoft Store / MSIX assets and are probably *not* what the Win32 tile reads — check before reusing them, and if they are unrelated, say so, because that is worth writing down.

## Reporting

**Edit `icon.md` directly** — it is built to grow this way and already has a Windows section marked as researched only as far as the current files. Replace that section with what you find. Keep its conventions: measured numbers rather than impressions, and say plainly where something is still unresearched instead of rounding it up to a conclusion.

If you change how icons are built, update the *arrangement* section too, and the `pnpm icons` description in `CLAUDE.md`, so the two cannot drift apart.

The user commits and pushes; the mac session reads it from there.
