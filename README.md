
![Fuji](fuji.svg)

# Fuji

A multimedia file manager designed with privacy and precision in mind.

Made with
[Rust](https://www.rust-lang.org/),
[JS](https://developer.mozilla.org/en-US/docs/Web/JavaScript),
[Vue](https://vuejs.org/) and
[Vite](https://vite.dev/) in
[Tauri](https://tauri.app/).

### Workspaces

This repository is a pnpm monorepo: the application, the website, and the Linux packaging. The planning documents stay at the root.

```
./desktop           the Fuji desktop application, made with Tauri
./site              the website and documentation, made with VitePress
./linux             Fuji's Linux packages, built on a Mac through Docker
```

### Scripts

Run `pnpm install` once at the root and it installs every workspace. Every other command runs from inside the workspace it belongs to, so start with `cd`. The root has no scripts of its own. pnpm comes from corepack rather than a global install, and reads the `packageManager` field in the root package.json to get the version this project pins.

```
$ pnpm install

$ cd desktop
$ pnpm local        run Fuji here, in development mode with hot reload
$ pnpm compile      build the binary in release mode, and stop there
$ pnpm installer    build the installer, all the way through the app to the dmg
$ pnpm reveal       open the file manager on that installer, to run it as a user would
$ pnpm hash         stage and hash what is already built, building nothing
$ pnpm upload       send what is already staged to the production server

$ cd site
$ pnpm local        run the site here, in development mode with hot reload
$ pnpm build        build it to dist, to sanity check that works
$ pnpm upload       build it and send it to the production server

$ cd linux
$ pnpm build        build the Linux packages in Docker containers
$ pnpm hash         stage them and write the sidecars, building nothing
$ pnpm upload       send what is staged to the production server
```

Common flows through the desktop commands include:

- **local / compile** — work on a feature that doesn't involve desktop integration, and check the release build.
- **installer / reveal** — install Fuji the way a user does, and test file associations, the installed icon, and anything else that needs a real install.
- **installer / hash / upload / commit & push** — publish a release, then commit. Git tracks the sidecars.

And through the site commands, there is really only one:

- **local / upload** — write or restyle a page with hot reload, then put it live. `upload` builds on its way out, so `build` is not a step in between.

We've picked unconventional script names to be clear about the smaller steps these commands perform. `compile` never makes an installer, `installer` never hashes, `hash` never builds, and `upload` never builds. The site's `upload` is the one exception, and builds before it sends.

A name does the same job on every machine. In `desktop`, `installer` makes a `.dmg` on macOS and a `.exe` on Windows, and `upload` sends whichever one the machine you're on can build — so you don't have to remember a different command per platform. `linux` says `build` rather than `installer` because it makes four packages rather than an installer, but `hash` and `upload` mean exactly what they mean everywhere else.

And through the Linux commands:

- **build / hash / upload / commit & push** — the same shape as a desktop release, with the same three steps and the same reason for their being three.

`pnpm icons` is separate from all of this. It rebuilds every platform's icons from the SVG sources. Run it by hand after you change the artwork, and after you upgrade the Tauri CLI. `icon.md` explains why that second case is easy to forget.

The JavaScript behind these commands is in one file, `scripts.js`, at the root, and each command passes it a verb. All three workspaces reach it because they share facts: the published name of every artifact, where each one is built, and the file manager command for each platform. Those live in one table there. The Linux workspace has a second file, `linux/build.js`, which drives the containers — `scripts.js` publishes and does not build, and that one builds and does not publish.

### Linux packages

Fuji's Linux packages are built **on the Mac, in Docker containers**, rather than on a Linux machine. `./linux/README.md` is the long version — the commands, and where every file lands on the way through. This is what an operator needs.

**Docker Desktop has to be installed and running.** The `docker` command is only a client, so if Docker Desktop is quit everything there fails with "cannot connect to the Docker daemon". `docker desktop status` says whether it is up and `docker desktop start` starts it without opening the window. Turn on **Use Rosetta for x86/amd64 emulation** in its settings while you are there; the x86-64 packages build in an emulated container and Rosetta is several times faster than the alternative.

After that it is three commands per release, and no setup step to remember:

```
$ cd linux
$ pnpm build        four packages and a recipe check, about eight minutes on the Mac mini
$ pnpm hash         stage them and write the sidecars, seconds
$ pnpm upload       send them to the server, seconds
```

Three rather than one because each leaves behind a different kind of thing. `build` writes packages, which are gitignored and disposable. `hash` writes sidecars, which are committed. `upload` puts files on a server, which is the only one of the three you can't take back.

**The first `build` on a new machine takes much longer**, because it builds four toolchain images before it builds anything else, which come to about 12 GB on disk. There is no separate setup command to forget: `build` brings its own images up to date every time, which costs three seconds once they exist. That also means you never have to notice when somebody bumps Rust or the Debian base in a `Dockerfile` and you pull it; the next build rebuilds that image and uses it.

`build` makes five things: a `.deb` for ARM, a `.deb` and an `.rpm` for x86-64, a `.flatpak`, and the AUR's `PKGBUILD` — a recipe rather than a package, so it's checked here and published separately. Five more commands sit underneath for factoring — `build-images`, `build-distro`, `build-flatpak`, `build-aur` and `stage`. They're documented in the workspace and rarely typed.

### Publishing

**Two machines publish Fuji.** Windows builds and sends the exe. The Mac sends the dmg it built natively, and the four Linux packages it built in Docker containers — the ARM `.deb`, the x86-64 `.deb`, the `.rpm` and the Flatpak — so every Linux package comes from one machine, one base image and one lockfile rather than from a shelf of borrowed computers.

Linux is not a publishing machine, deliberately. You can clone this repository on Ubuntu or Raspberry Pi OS and run `pnpm installer` in `desktop` to build Fuji for the machine you're sitting at, which is development and works. Staging and uploading from there is refused, with a sentence saying why.

The site ships over rsync from any machine that has rsync, which in practice is the Mac. Windows can't, and says so clearly instead of failing somewhere deeper.

`hash` writes a small JSON sidecar next to each package holding its size, hash, version and date. No published name carries a version, and every Linux package states its architecture — `fuji.arm64.deb`, `fuji.amd64.deb`, `fuji.x86_64.rpm`, `fuji.x86_64.flatpak`. `fuji.dmg` and `fuji.exe` need no token, shipping one architecture each. That keeps every link anyone shares pointing at the current build instead of pinning the version that was current the day they shared it. Packages are gitignored and sidecars are committed. The download page fetches those sidecars when a visitor opens it, so a new installer shows up on the site as soon as you upload it, with no site deploy. If you skip `hash`, the upload compares the sidecar against the file next to it, sees they disagree, and stops.

Both `upload` commands read `.env` at the root for the server address and account details. The installer upload also needs an SSH key, belonging to a restricted account which can write the downloads directory and nothing else, while the site upload uses an administrative account. Neither the file nor the key is in git: `.gitignore` covers `.env`, and the key lives outside the repository. Set each machine up once from notes kept offline.

Downloading the installer from fujidesktop.app is different again. A browser marks a file it downloaded with the mark of the web, and that mark is what triggers SmartScreen on Windows. A copy you built locally carries no mark and runs without the warning.

### Scaffolded on macOS

The commands below are the original scaffolding from July 2025, kept as a record. Fuji moved from yarn to pnpm in August 2026.

```
$ yarn create tauri-app fuji
✔ Identifier · com.zootella.fuji
✔ Choose which language to use for your frontend · TypeScript / JavaScript - (pnpm, yarn, npm, deno, bun)
✔ Choose your package manager · yarn
✔ Choose your UI template · Vue - (https://vuejs.org/)
✔ Choose your UI flavor · JavaScript
```

The identifier in that transcript is what scaffolding proposed. Fuji's is now `app.fujidesktop.Fuji`, so the name it carries on every platform is the domain it publishes from rather than its author's.

### Output

Executable and installer on mac
```
./desktop/src-tauri/target/release/bundle/macos/Fuji.app
./desktop/src-tauri/target/release/bundle/dmg/Fuji_0.1.0_aarch64.dmg
```

Executable and installer on windows
```
./desktop/src-tauri/target/release/fuji.exe
./desktop/src-tauri/target/release/bundle/nsis/Fuji_0.1.0_x64-setup.exe
```

Linux packages, built in containers from the `linux` workspace
```
./linux/release/Fuji_0.1.0_arm64.deb
./linux/release/Fuji_0.1.0_amd64.deb
./linux/release/Fuji-0.1.0-1.x86_64.rpm
./linux/release/Fuji_0.1.0_x86_64.flatpak
./linux/release/aur/PKGBUILD
```

Staged for publishing by `pnpm hash`, on whichever machine built it. No published name carries a
version, and every Linux package states its architecture
```
./desktop/release/fuji.dmg              ./desktop/release/fuji.dmg.json
./desktop/release/fuji.exe              ./desktop/release/fuji.exe.json

./linux/release/fuji.arm64.deb          ./linux/release/fuji.arm64.deb.json
./linux/release/fuji.amd64.deb          ./linux/release/fuji.amd64.deb.json
./linux/release/fuji.x86_64.rpm         ./linux/release/fuji.x86_64.rpm.json
./linux/release/fuji.x86_64.flatpak     ./linux/release/fuji.x86_64.flatpak.json
```

Fuji ships those four packages and no more, so `bundle.targets` names them instead of Tauri's default `"all"`, which would also build an `.msi` beside the NSIS installer and an `.AppImage` beside the Debian package. The installers themselves stay out of git; their sidecars are committed, so the repository keeps a dated record of what hash each release had.

`pnpm reveal` opens whichever of those bundle folders this platform builds into, so there is no need to walk the path by hand.

## Setup macOS

Verify or Install Xcode Command Line Tools
```
$ xcode-select -p
/Library/Developer/CommandLineTools

$ xcode-select --install
```

Confirm you can reach Apple's Clang compiler, aliased as gcc
```
$ gcc --version
Apple clang version 17.0.0 (clang-1700.0.13.5)
Target: arm64-apple-darwin24.5.0
Thread model: posix
InstalledDir: /Library/Developer/CommandLineTools/usr/bin
```

Install Rust
```
$ curl https://sh.rustup.rs -sSf | sh
$ rustc --version
rustc 1.88.0 (6b00bc388 2025-06-23)
$ cargo --version
cargo 1.88.0 (873a06493 2025-05-10)
```

## Setup Windows 10

https://visualstudio.microsoft.com/visual-cpp-build-tools/
 * Downloads and runs a 4.5mb installer
 * Visual Studio Installer, wizard starts
 * Desktop development with C++, choose that one first card

You can paste the whole block below into *PowerShell* as a single command
```
# Detect Visual Studio Build Tools with MSVC
$vsPath = & "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe" `
  -latest -products * `
  -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
  -property installationPath
# Locate MSVC toolset folder
$msvcRoot = Join-Path $vsPath "VC\Tools\MSVC"
# Get the latest MSVC version folder
$latestMsvcVersion = Get-ChildItem $msvcRoot | Sort-Object Name -Descending | Select-Object -First 1
# Build full path to cl.exe
$clPath = Join-Path $latestMsvcVersion.FullName "bin\Hostx64\x64"
# Output the resolved path
Write-Host "Resolved cl.exe path:"
Write-Host $clPath
```
Should output a path like
```
Resolved cl.exe path:
C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64
```
You only need to get it on the path for this one *PowerShell* session...
```
# Add cl.exe to the path for this session, and confirm it's there
$env:Path += ";$clPath"
Get-Command cl.exe

CommandType  Name    Version    Source
-----------  ----    -------    ------
Application  cl.exe  14.44.3... C:\Program Files (x86)\Microsoft Visua...
```

...to be able to then install the Rust toolchain
```
# Install the Rust toolchain
Invoke-WebRequest -Uri https://win.rustup.rs -OutFile rustup-init.exe
Start-Process .\rustup-init.exe
```
This pops its own command line window, *Enter* for default

Now you can see everything in [Git](https://git-scm.com/downloads/win)'s *MINGW64*
```
$ node --version, v22.21.1
$ npm --version, 10.8.1
$ corepack --version, 0.34.0
$ rustc --version, rustc 1.98.0 (88d9e12ae 2026-08-18)
$ cargo --version, cargo 1.98.0 (797e8a9bc 2026-08-05)
$ git --version, git version 2.55.0.windows.3

$ git clone https://github.com/zootella/fuji
$ cd fuji
$ pnpm install --frozen-lockfile
$ cd desktop
$ pnpm installer
$ pnpm local
```
`pnpm` isn't installed globally — it comes from corepack, which ships with Node. Run `corepack enable` once from an elevated *PowerShell*; after that `pnpm` reads the `packageManager` field in the root package.json and runs the exact version this project pins, downloading it on first use. Use `--frozen-lockfile` when you're installing what's committed rather than changing dependencies; it refuses to quietly rewrite pnpm-lock.yaml, which is what keeps one lockfile serving both mac and windows.

No global `tauri-cli` either — the `@tauri-apps/cli` in devDependencies is what `pnpm local` and `pnpm installer` run, and keeping it there means the CLI can't drift out of step with the Rust crates.

## README from scaffolding

**Tauri + Vue 3**
This template should help get you started developing with Tauri + Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

**Recommended IDE Setup**
- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
