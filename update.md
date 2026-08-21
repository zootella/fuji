# Update Sprint — Modernize Dependencies

The goal: bring fuji's dependencies to where a fresh scaffold with the same tools would put them today — majors and minors both — so this repo matches its sister project **ftorrent**, which will be scaffolded from scratch on the same stack. When ftorrent exists, diffing its manifests against fuji's should show no meaningful drift.

Ground rules for this sprint: Claude edits files and runs builds; Kevin alone runs mutating git commands. Each step ends at a commit point so any step can stop the train without stranding the others. After every step: build passes, app runs, images behave.

## The manual test loop

Run after each step (Kevin, at the keyboard, `yarn local` or the built app):

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
- [ ] Run `src-tauri/target/release/bundle/macos/Fuji.app` directly; full manual test loop

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

- [ ] `pnpm import` — generate pnpm-lock.yaml from yarn.lock at identical versions
- [ ] package.json: `"packageManager": "pnpm@10.28.2"`; update wash/upgrade-wash scripts (yarn.lock → pnpm-lock.yaml, and pnpm's node_modules layout)
- [ ] Delete yarn.lock; `pnpm install` clean
- [ ] Update CLAUDE.md (says "Package Manager: Yarn") and any yarn references in docs
- [ ] `pnpm vite-build` passes; `pnpm local` dev run; manual test loop passes
- [ ] Corepack check: `yarn` now refuses in this project, `pnpm` accepted

**Commit point:** "switch to pnpm"

## Step 2 — JavaScript minors (low risk)

Bump everything that stays within its current major: @tauri-apps/api and cli, the three tauri plugins, vue, tailwindcss, @tailwindcss/vite. Edit package.json to current versions, reinstall, rebuild.

- [ ] package.json bumped, `yarn install` clean
- [ ] `yarn vite-build` passes
- [ ] Manual test loop passes

**Commit point:** "bump js minors: tauri 2.11, vue 3.5.41, tailwind 4.3"

## Step 3 — Vite 6 → 8, plugin-vue 5 → 6 (the one real migration)

Two whole-number jumps: Vite 7 raised the Node floor (satisfied — Node 22.21 installed vs `^20.19 || >=22.12` required) and changed browser baselines; Vite 8 is the Rolldown-bundler generation. plugin-vue 6.0.8 supports Vite 5 through 8. Our vite.config.js is small and uses nothing exotic, so the expectation is: read both migration guides, bump, and change little or nothing.

- [ ] Read Vite 7 and 8 migration guides; note anything touching our config
- [ ] Bump vite and @vitejs/plugin-vue; `yarn install` clean
- [ ] `yarn vite-build` passes; skim dist output for sanity
- [ ] Dev run (`yarn local`) — HMR still works; manual test loop passes
- [ ] Release build + run from built

**Commit point:** "vite 8, plugin-vue 6"

## Step 4 — Delete what a fresh scaffold wouldn't have

Tailwind 4's Vite plugin handles vendor prefixing itself (Lightning CSS) and reads configuration from CSS, so three artifacts are vestigial:

- [ ] Delete postcss.config.js; drop autoprefixer and postcss from devDependencies
- [ ] Delete tailwind.config.js (v4 auto-detects content; confirm nothing references it via @config)
- [ ] Rebuild; visually confirm styles unchanged — dots background, HUD smoke, caption emboss
- [ ] Manual test loop passes

**Commit point:** "remove vestigial postcss and tailwind config"

## Step 5 — Rust side

- [ ] `cargo update` — floats tauri, tauri-build, plugins, serde, display-info within their ranges
- [ ] Bump the windows crate pin 0.48 → current (match the version tauri's own tree pulls, cutting duplicate compiles of a big crate); absorb API churn in panel.rs — surface is one feature, Win32_UI_WindowsAndMessaging
- [ ] Bump core-graphics 0.22 → current; absorb any churn in panel.rs mac path
- [ ] Edition 2021 → 2024 (`cargo fix --edition`, then set edition in Cargo.toml)
- [ ] `cargo check` clean, no warnings
- [ ] Optional: `rustup target add x86_64-pc-windows-msvc` + `cargo check --target x86_64-pc-windows-msvc` to compile-check the windows path from this mac
- [ ] Dev run — panelResolution() still returns real numbers on mac (check [i] HUD / console)
- [ ] Release build + run from built; full manual test loop

**Commit point:** "modernize rust deps, edition 2024"

## Open decisions

- **Windows verification:** everything here is verified on mac; the windows build (and high-res windows fullscreen panning, untested since last year) needs a session on the windows machine after the sprint lands.
