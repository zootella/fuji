# Letter to windows claude

Hello — this letter is from the Claude Code session that worked with the user on fuji's mac side, August 2026. The user has pulled this repo on their windows workstation and pointed you here. Your job is to verify the modernization sprint on windows. The full sprint record is in `update.md`; read it before starting.

## Ground rules (these matter — they're not in your memory yet)

- **Git**: you may run read-only git commands freely (status, log, diff, show). All mutating git commands — add, commit, push, pull, everything — are the user's alone. When it's time to commit, end your response with one line: 📌 followed by the suggested commit message in boldface. Nothing else on that line.
- **Style**: read `./style.md` before writing or editing any code. One rule to know even before you read it: `ttd` comments are the user's alone — never add, reword, or delete one.
- **Private files**: names with "hide" as a dot-separated part (`hide/`, `*.hide`, `hide.*`, `*.hide.*`) are gitignored private planning docs. Respect that boundary in anything you write.

## What changed on the mac (short version)

yarn classic → pnpm 10.28.2 (corepack-enforced via the packageManager field); Tauri npm packages and Rust crates aligned at 2.11; Vue 3.5.41; Tailwind 4.3 (vite plugin only — postcss/autoprefixer/tailwind.config are deliberately gone); Vite 8 + plugin-vue 6; windows crate 0.62.2 and core-graphics 0.25 pins; everything verified on mac through dev runs and a release build.

## Your sequence

1. **Prerequisites**: Node 20.19+/22.12+ with corepack, current stable Rust, the usual Tauri windows toolchain (MSVC, WebView2). `corepack` should activate pnpm 10.28.2 automatically from the packageManager field.
2. **The headline test**: `pnpm install --frozen-lockfile` from the mac-made pnpm-lock.yaml. This exact scenario — installing on windows from a mac-created lockfile — is what broke under yarn classic and motivated the pnpm switch. Do NOT delete or regenerate the lockfile to make problems go away; if install fails, that's a finding — diagnose and report it.
3. **Dev run**: `pnpm local`. Expect no version-mismatch warnings from the Tauri CLI. Manual test loop (the user drives): drag an image in, wheel-flip through the folder, ctrl+wheel zoom, drag-pan, double-click fullscreen and back (pan position should survive), h/i HUDs.
4. **Windows crate proof**: the jump 0.48 → 0.62.2 was only type-checked from the mac. Here it compiles and links for real, and `panel_resolution()` (src-tauri/src/panel.rs, GetSystemMetrics path) must return real hardware pixel counts at runtime.
5. **Release build**: `pnpm build`. Note what bundles windows produces (msi is Tauri's default; NSIS is a planned but not-yet-configured addition — see notes in the repo). Run the built exe, loop again.
6. **The long-standing untested case**: fullscreen pan preservation on a high-resolution windows display — flagged untested since 2025. The user knows the details.
7. **The `app` script is mac-only** (`open` on the .app bundle). Work out a windows equivalent with the user — likely a separate script or a small cross-platform node one-liner — and keep the mac one working.

## Reporting

Record results by editing `update.md` — add a "## Windows session results" section with what passed, what failed, and exact errors. The user commits and pushes; the mac session reads it from there. If the shared-lockfile test passes, say so explicitly — it's the single most important sentence in your report.
