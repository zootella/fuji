# Scaffolding a sister desktop client

This guide is for a Claude Code session scaffolding a new Tauri desktop client that matches this repository's stack — same tools, same or functionally equivalent versions. Fuji (this repo) is the reference implementation: **when this guide's version numbers disagree with fuji's actual manifests, the manifests win.** The numbers written here are a snapshot from 2026-08-22; the repo is alive. Diff against fuji's package.json, Cargo.toml, vite.config.js, and tauri.conf.json as you go.

The target stack: Tauri 2.11.x (npm packages and Rust crates on the same minor — the Tauri CLI enforces this), Vue 3.5.x in plain JavaScript, Vite 8.2.x with @vitejs/plugin-vue 6, Tailwind 4.3.x via the vite plugin only, pnpm 10.28.2 through corepack, Rust edition 2021.

## Ground rules (read these first)

- **Git**: run read-only git commands freely; all mutating git commands — add, commit, push, pull — are the user's alone. When it's time to commit, end your response with one line: 📌 followed by the suggested commit message in boldface.
- **Style**: the user's style guide governs all code. Fuji carries a copy at style.md — get a copy into the new repo early and read it before writing code. One rule matters even before you read it: `ttd` comments are the user's alone — never add, reword, or delete one.
- **Private files**: names with "hide" as a dot-separated part (`hide/`, `*.hide`, `hide.*`, `*.hide.*`) are gitignored private planning docs, in fuji and in any repo adopting the same convention.
- **Public docs say "the user"**, never the user's name.

## 1. Scaffold

```
pnpm create tauri-app <name> --manager pnpm --template vue --identifier <identifier> --yes
```

Channel warning: as of August 2026 the npm-published create-tauri-app (4.6.2) is stale and hands out vite ^6.0.3 pins, even though Tauri's own templates moved to vite 8 in June 2026. Don't fight it — accept whatever the scaffold produces; step 3 aligns everything to fuji regardless.

If scaffolding inside an existing pnpm workspace: confirm the workspace's pnpm-workspace.yaml covers the new package, and merge the scaffold's package.json with any existing placeholder (keep the placeholder's name, description, homepage) rather than clobbering it.

## 2. Package manager

- Add `"packageManager": "pnpm@10.28.2"` (match fuji's current value) to package.json — the scaffold does not add this field. Corepack then pins the exact pnpm everywhere, and refuses other package managers. (If pnpm isn't on PATH at all, `corepack enable` once — corepack ships with Node.)
- The lockfile is tracked and shared across mac and windows — pnpm records every platform's native binaries and selects at install time. Never delete or regenerate a lockfile to make a cross-platform problem go away; that's a finding to report.

## 3. Align versions to fuji

Set package.json to fuji's current versions (snapshot: vite ^8.2.2, @vitejs/plugin-vue ^6.0.8, vue ^3.5.41, @tauri-apps/api ^2.11.1, @tauri-apps/cli ^2.11.4, @tauri-apps/plugin-opener ^2.5.4; add @tauri-apps/plugin-fs ^2.5.1 when disk work begins). Then `pnpm install`.

On the Rust side, keep the scaffold's Cargo.toml shape and `edition = "2021"` (deliberate — matches Tauri's template; fuji stayed there too), and run a full `cargo update`. Two lessons fuji learned the hard way:

- The Tauri CLI requires npm packages and Rust crates on the **same major/minor** — bump both sides together or the dev server complains.
- Use a full `cargo update`, not targeted `-p` updates — the targeted form can leave an incoherent tree that fails to compile (fuji hit exactly this with tauri-runtime-wry).

Vite 8 needs Node 20.19+/22.12+. Vite 6→8 migration, if the scaffold started at 6: for a config this small there are no changes — just bump the two version numbers. Fuji verified this.

## 4. Tailwind 4, the modern way

```
pnpm add -D tailwindcss @tailwindcss/vite
```

- vite.config.js: import and add the `tailwindcss()` plugin (see fuji's — the scaffold's config plus one line, no `async` wrapper).
- The main CSS file: `@import "tailwindcss";`
- That's all. **No** postcss.config.js, **no** tailwind.config.js, **no** autoprefixer — Tailwind 4's vite plugin does prefixing and configuration natively. If a tutorial tells you to create those files, it's describing Tailwind 3.

## 5. Config conventions

- tauri.conf.json: `beforeDevCommand: "pnpm dev"`, `beforeBuildCommand` pointing at the vite build script — pnpm, never another package manager.
- Plugins and capabilities: register only plugins the app actually uses, and grant granular permissions, never the blanket `:default` sets. The house pattern (see fuji's lib.rs and capabilities/default.json): plugin-dialog with `dialog:allow-open` and `dialog:allow-save` only (no message/ask/confirm — fake native dialogs are a social-engineering primitive, and the app's own UI lives in the page), and plugin-opener with `opener:allow-reveal-item-in-dir` only (URL opening arrives later, with a scoped allowlist, attached to the feature that needs it). Tauri validates permission identifiers at build time, so a typo fails loudly.
- Replace the scaffold's `"csp": null` with fuji's tested policy: `"default-src 'self'; connect-src 'self' ipc: http://ipc.localhost; img-src 'self' data: blob:"`. Rationale: these apps' webviews load only the bundled frontend and never navigate; all networking lives in the Rust core. CSP is the second wall behind Vue's template escaping — add nothing to it without a reason written next to it. Leave `devCsp` unset (dev stays unrestricted for HMR; the policy guards what ships). Verify on the built app: Tauri injects the policy into the binary's embedded HTML, and failure is loud — blocked IPC means the UI can't reach Rust, blocked img-src means images don't render.
- Scripts, following fuji's package.json: `local` (tauri dev); the build trail `build-binary` (--no-bundle, quickest compile-and-link proof) → `build-app` (--bundles app, runnable without dmg theatrics) → `build-dmg`/`build` (everything); `app`/`win` (launch the built app per platform); `vite-build`. A script cannot be named `run` — pnpm's builtin shadows it. Deliberately no cleanup scripts: those were yarn-classic-era crutches, pnpm doesn't need clean reinstalls — and no script ever deletes the tracked lockfiles.
- Router: decide by the app's nature, and revisit as the app grows. A many-screened app — lists, detail views, settings, the shape of a management console — earns vue-router: the route table is its table of contents. A single-space immersive app switches modes with plain Vue (`v-if` or `<component :is>` on a mode ref) — two or three modes sharing live state don't need URL serialization, history, or guards, and routers unmount components by default, which fights kept-alive state. If a router is adopted in a tauri app, use hash mode: history-mode paths expect a server to answer on reload, and a tauri bundle has none. Neither choice is dogma — an app that grows enough screens converts, with a reason attached.
- Line endings: copy fuji's .gitattributes (`* text=auto eol=lf`) and .editorconfig — LF in the repo and both working trees, policy living in the repo rather than machines' git configs.
- .gitignore: fuji's current one is the reference — including the OS-junk entries for both platforms and the hide-convention block if the project adopts it.

## 6. Verify

- `pnpm install` clean; `pnpm vite-build` passes
- `pnpm local` — dev window opens, no version-mismatch warnings from the Tauri CLI, HMR works (edit a template string, watch it hot-swap)
- `cargo check` clean from src-tauri
- `pnpm build` — release bundles produced; `pnpm app` / `pnpm win` launches the built app

Fuji's git history (August 2026, the modernization sprint commits) records how each of these behaved during its own update, on mac and windows both.

## 7. Preparing to receive disk.rs

The sister project will take a copy of fuji's disk access module: `disk.rs` (Rust commands built on std::fs) plus its JS wrapper `disk.js`. Do **not** copy fuji's io.rs — disk.rs is that module *after* a rename and a correctness/security audit that is in progress in fuji. When it's ready, it arrives with its wrapper and registration instructions (module declaration in lib.rs, `generate_handler![]` entries, any capabilities the audit settles on).

Until then, the scaffold just needs to be structurally ready: the standard lib.rs shape (the scaffold provides it), and the knowledge that new Rust commands follow fuji's CLAUDE.md pattern — define with `#[tauri::command]`, register in `generate_handler![]`, wrap in a JS file, import in components.
