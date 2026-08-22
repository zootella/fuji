# Update Sprint — Modernize Dependencies

The goal: bring fuji's dependencies to where a fresh scaffold with the same tools would put them today — majors and minors both — so this repo matches its sister project **ftorrent**, which will be scaffolded from scratch on the same stack. When ftorrent exists, diffing its manifests against fuji's should show no meaningful drift.

Ground rules for this sprint: Claude edits files and runs builds; the user alone runs mutating git commands. Each step ends at a commit point so any step can stop the train without stranding the others. After every step: build passes, app runs, images behave.

## The manual test loop

Run after each step (the user, at the keyboard, `pnpm local` or the built app):

1. Drag an image file into the window — it loads, siblings list
2. Mouse wheel — flips forward and back through the folder
3. Ctrl/Cmd+wheel and +/- keys — zoom in and out
4. Drag — pans; dots background rides along
5. Double-click — fullscreen; pan position survives the transition; Esc exits
6. [h] and [i] — help and information HUDs toggle
7. Check the information HUD shows path, natural size, displayed size

## Step 0 — Baseline: prove it works before changing anything

Verify the untouched code builds and runs, so any later breakage is attributable to an upgrade, not to rot we inherited.

- [x] `yarn install` + `yarn vite-build` — passed 2026-08-21
- [x] `cargo check` — passed clean, Rust 1.97.1
- [x] Dev run: `yarn local` — built and ran successfully, confirmed 2026-08-21
- [x] Release build: `yarn build` completes — passed 2026-08-21, 1m18s, both bundles produced
- [x] Run `src-tauri/target/release/bundle/macos/Fuji.app` directly — done via the `app` script, loop passed

**Versions before** (locked → latest available, checked 2026-08-21):

| JavaScript | Locked | Latest | | Rust | Locked | Latest |
|---|---|---|---|---|---|---|
| vite | 6.3.5 | 8.2.2 | | tauri | 2.6.2 | 2.11.5 |
| @vitejs/plugin-vue | 5.2.x | 6.0.8 | | tauri-build | 2.3.0 | 2.6.3 |
| vue | 3.5.18 | 3.5.41 | | tauri-plugin-fs | 2.4.0 | 2.5.1 |
| tailwindcss (+vite) | 4.1.11 | 4.3.3 | | tauri-plugin-opener | 2.4.0 | 2.5.4 |
| @tauri-apps/api | 2.7.0 | 2.11.1 | | windows | 0.48.0 | 0.62.2 |
| @tauri-apps/cli | 2.7.1 | 2.11.4 | | core-graphics | 0.22.3 | 0.25.0 |
| plugin-dialog/fs/opener | 2.x | 2.7.2 / 2.5.1 / 2.5.4 | | display-info | 0.5.4 | 0.5.9 |

**Commit point:** baseline confirmed; good moment to also commit the new docs (update.md, roadmap.plan.md, style.md, .gitignore).

## Step 1 — Move from yarn classic to pnpm

Match cold3 (and ftorrent when it's scaffolded): `"packageManager": "pnpm@10.28.2"`, pnpm-lock.yaml checked in. The versions don't change in this step — `pnpm import` converts yarn.lock into pnpm-lock.yaml preserving every resolved version — so the package-manager switch is isolated from the upgrades that follow.

Why pnpm fixes the mac/windows lockfile problem: the native-binary packages (@tauri-apps/cli-darwin-arm64, cli-win32-x64-msvc, esbuild's and rollup's platform binaries) are optionalDependencies selected by os/cpu. Yarn classic records only the set resolvable on the machine that ran install, so a mac-made lockfile strands a windows install — and yarn 1 is frozen software, so it never got fixed. pnpm records every platform variant in the lockfile and selects at install time, so one tracked lockfile serves both machines.

- [x] `pnpm import` — pnpm-lock.yaml generated; spot-check confirmed identical versions (vite 6.3.5, vue 3.5.18, tailwind 4.1.11)
- [x] package.json: `"packageManager": "pnpm@10.28.2"`; upgrade-wash script now removes pnpm-lock.yaml
- [x] yarn.lock and node_modules deleted; `pnpm install --frozen-lockfile` clean in 1.2s
- [x] CLAUDE.md converted to pnpm commands
- [x] tauri.conf.json beforeDevCommand/beforeBuildCommand converted (missed in first sweep — corepack's yarn refusal caught it on first `pnpm local`)
- [x] `pnpm vite-build` passes — JS bundle byte-identical to yarn baseline; CSS +0.25 kB (verify visually in dev run)
- [x] `pnpm local` dev run; manual test loop passed (after the tauri.conf.json fix below)
- [x] Corepack check: `yarn` refuses ("configured to use pnpm"), `pnpm app` launches the built app

Note: pnpm 10 ignores postinstall scripts (esbuild, @tailwindcss/oxide) by default; build works anyway because both ship native binaries as optionalDependencies — the same mechanism that makes the shared lockfile cross-platform.

**Commit point:** "switch to pnpm"

## Step 2 — JavaScript minors (low risk)

Bump everything that stays within its current major: @tauri-apps/api and cli, the three tauri plugins, vue, tailwindcss, @tailwindcss/vite. Edit package.json to current versions, reinstall, rebuild.

- [x] package.json bumped, `pnpm install` clean in 1.4s — api 2.11.1, cli 2.11.4, dialog 2.7.2, fs 2.5.1, opener 2.5.4, vue 3.5.41, tailwind 4.3.3
- [x] `pnpm vite-build` passes — JS 92.4 kB, CSS 13.8 kB, growth consistent with minor feature additions
- [x] Pulled forward from Step 5: full `cargo update` (355 changes) — Tauri's dev guardrail requires npm packages and Rust crates on the same major/minor, so the crate side had to move with the JS side. Now tauri 2.11.5 ↔ api 2.11.1, plugin-fs 2.5.1 ↔ 2.5.1, opener 2.5.4 ↔ 2.5.4; `cargo check` clean in 33s. (A targeted `cargo update -p tauri ...` first produced an incoherent tree that failed to compile — full update was the fix.)
- [x] Manual test loop passes — 2026-08-22: mismatch warning gone; drag-in, flipping, and the fullscreen round-trip with pan preservation all confirmed

**Commit point:** "bump js minors and align rust crates: tauri 2.11, vue 3.5.41, tailwind 4.3"

## Step 3 — Vite 6 → 8, plugin-vue 5 → 6 (the one real migration)

Two whole-number jumps: Vite 7 raised the Node floor (satisfied — Node 22.21 installed vs `^20.19 || >=22.12` required) and changed browser baselines; Vite 8 is the Rolldown-bundler generation. plugin-vue 6.0.8 supports Vite 5 through 8. Our vite.config.js is small and uses nothing exotic, so the expectation is: read both migration guides, bump, and change little or nothing.

- [x] Read Vite 7 and 8 migration guides — v7: Node floor (satisfied) and browser targets (irrelevant for a current-webview app); v8: Rolldown/Oxc bundler, Lightning CSS minification, renamed options we don't use. Both plugins peer-support Vite 8. No config changes needed
- [x] Bump vite ^8.2.2 and @vitejs/plugin-vue ^6.0.8; `pnpm install` clean in 1s
- [x] `pnpm vite-build` passes — JS 90.6 kB (Oxc minifies tighter than esbuild did), CSS 13.5 kB (Lightning CSS); esbuild left the dependency tree entirely
- [x] Dev server smoke test — serves 200 on strict port 1420 with HMR client injected
- [x] Dev run (`pnpm local`) — manual test loop passed; HMR verified live (script-setup edit hot-swapped sub-second, component state reset as expected)
- [x] Release build + run from built — deferred to and completed in Step 5's final verification, so the sprint ends with one full release proof instead of three redundant ones

**Commit point:** "vite 8, plugin-vue 6"

## Step 4 — Delete what a fresh scaffold wouldn't have

Tailwind 4's Vite plugin handles vendor prefixing itself (Lightning CSS) and reads configuration from CSS, so three artifacts are vestigial:

- [x] Delete postcss.config.js; drop autoprefixer and postcss from devDependencies — grep confirmed zero references anywhere first
- [x] Delete tailwind.config.js — nothing references it via @config
- [x] Drop @tauri-apps/plugin-dialog from package.json — vestigial: no matching Rust crate, not registered in lib.rs, never imported in JS (found via fresh-scaffold comparison)
- [x] Bonus parity tweak: removed the legacy `async` wrapper from vite.config.js, matching Tauri's current template (their PR #959)
- [x] Rebuild passes — JS byte-identical 90.59 kB; CSS 13.47 kB (−0.04, autoprefixer's prefixes now handled by Lightning CSS natively)
- [x] Visual check + manual test loop (the user) — styles pixel-identical, loop passed

**Commit point:** "remove vestigial postcss and tailwind config"

## Step 5 — Rust side

- [x] `cargo update` — done early, in Step 2, to satisfy Tauri's npm↔crate minor-match guardrail (see Step 2 notes)
- [x] Bump the windows crate pin 0.48 → 0.62.2 (the version tauri's own tree pulls) — panel.rs needed zero changes; verified by type-checking the module against 0.62 for the x86_64-pc-windows-msvc target from this mac (a full cross cargo check is blocked by tauri-winres wanting llvm-rc; the real windows build happens on the windows machine)
- [x] Bump core-graphics 0.22 → 0.25 and foreign-types-shared 0.1 → 0.3 (they pair) — panel.rs mac path compiled clean, zero changes; both old duplicate versions left the lock tree
- [x] Edition: decided — staying on 2021 for scaffold/ftorrent parity (Tauri's template choice); revisit when Tauri's template moves
- [x] `cargo check` clean, no warnings
- [x] Windows path verified from mac via scratch-crate type-check against windows 0.62 (full cross cargo check blocked by tauri-winres needing llvm-rc — real proof comes on the windows machine)
- [x] Release build passes — cargo 1m14s, both bundles produced 2026-08-22
- [x] Run from built (`pnpm app`) — 2026-08-22, runs quickly, test loop fine

**Commit point:** "modernize rust deps, edition 2024"

## Fresh-scaffold comparison (2026-08-22)

Potted a new plant to sanity-check the repotted one: `pnpm create tauri-app` (vue template, pnpm manager) into the gitignored `hide/fresh/`, then diffed manifests against fuji.

Findings:

- **The template's pins are stale — and we verified it's release lag, not intent** (investigated 2026-08-22 when the vite-8 question was raised). The scaffold ships vite ^6.0.3, but: (a) create-tauri-app's own repo updated all JS templates to **vite 8** on 2026-06-15 (PR #975) — Tauri's current intent is vite 8; (b) the npm channel still serves the older 4.6.2 wrapper with vite-6 pins, which is why fresh scaffolds arrive on 6 today; (c) npm download data shows **vite 8.x is the largest running cohort at 41%** of all vite downloads (7.x 24%, 6.x a shrinking 16%), and Nuxt 4.4 ships vite 8. Decision: **fuji stays on Vite 8** — that's both current and mainstream-standard, and matches where Tauri's templates are headed. When ftorrent's desktop is scaffolded, bump its vite to 8 if the npm wrapper is still behind. Also from that template history: PR #959 removed the unneeded `async` wrapper from vite config templates — optional one-word parity tweak for our vite.config.js in Step 4.
- **vite.config.js**: fresh scaffold's is conceptually identical to ours (ours adds only the tailwind plugin) — nothing vestigial there.
- **Cargo.toml**: identical shape; fuji's extras (plugin-fs, display-info, windows, core-graphics) are all deliberate. Edition still 2021 in the template — see Step 5 decision.
- **tauri.conf.json**: same structure; fuji's window size, dragDropEnabled, and script names are deliberate differences.
- **packageManager field**: the scaffold doesn't add one even with `--manager pnpm`; ours (cold3 pattern, corepack-enforced) is strictly better — keep.
- **Found a stowaway**: @tauri-apps/plugin-dialog in fuji's package.json has no Rust crate, no registration, no JS import — added to Step 4's deletions.
- **.gitignore**: fresh scaffold's matches the first half of ours; our duplicated lines are two scaffold generations concatenated — harmless, optional tidy.

The `hide/fresh/` scaffold stays around as reference until the sprint closes, then can be deleted (or kept for the ftorrent scaffolding session).

## Step 6 — Letter to windows claude

After the mac-side steps land and push: write a letter in the repo (tracked and public — it must ride the push to reach the windows clone) briefing the Claude Code session the user will start on the windows workstation. It should carry: what this sprint changed, how to install (corepack/pnpm, shared lockfile — installing from the mac-made pnpm-lock.yaml **is the test**), what to build and run, the manual test loop, windows-specific attention points (windows crate 0.6x compile, high-res fullscreen pan preservation, NSIS bundling), and where to record results.

- [x] Letter written: `windows.md` — carries the ground rules (git, style, ttd, hide files) that live only in the mac session's memory, the install-is-the-test framing, the verification sequence, and the reporting format
- [x] Committed and pushed 2026-08-22 — the letter is on the remote, awaiting the windows session

## Open decisions

- **Windows verification:** everything here is verified on mac; the windows build (and high-res windows fullscreen panning, untested since last year) needs a session on the windows machine after the sprint lands.
