notes

## setup development workstation

### mac

```
$ xcode-select -p
/Library/Developer/CommandLineTools

or install with the command below if not there yet
$ xcode-select --install

$ gcc --version
Apple clang version 17.0.0 (clang-1700.0.13.5)
Target: arm64-apple-darwin24.5.0
Thread model: posix
InstalledDir: /Library/Developer/CommandLineTools/usr/bin

$ curl https://sh.rustup.rs -sSf | sh

$ rustc --version
rustc 1.88.0 (6b00bc388 2025-06-23)
$ cargo --version
cargo 1.88.0 (873a06493 2025-05-10)
```

second, to scaffold the project:

```
$ yarn create tauri-app fuji
✔ Identifier · com.zootella.fuji
✔ Choose which language to use for your frontend · TypeScript / JavaScript - (pnpm, yarn, npm, deno, bun)
✔ Choose your package manager · yarn
✔ Choose your UI template · Vue - (https://vuejs.org/)
✔ Choose your UI flavor · JavaScript

executable and installer on mac:
./fuji/src-tauri/target/release/bundle/macos/fuji.app
./fuji/src-tauri/target/release/bundle/dmg/kay_0.1.0_aarch64.dmg

executable and installer on windows:
./fuji/src-tauri/target/release/fuji.exe
./fuji/src-tauri/target/release/bundle/windows/fuji-0.1.0-x86_64.exe
```

### windows 10

https://visualstudio.microsoft.com/visual-cpp-build-tools/
downloads and runs a 4.5mb installer
Visual Studio Installer, wizard starts
Desktop development with C++, choose that one first card

you can paste the whole block below into powershell as a single command:

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

```
Resolved cl.exe path:
C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64

# Add cl.exe to the path for this session, and confirm it's there
$env:Path += ";$clPath"
Get-Command cl.exe

CommandType  Name    Version    Source
-----------  ----    -------    ------
Application  cl.exe  14.44.3... C:\Program Files (x86)\Microsoft Visua...

# Install the Rust toolchain
Invoke-WebRequest -Uri https://win.rustup.rs -OutFile rustup-init.exe
Start-Process .\rustup-init.exe
```

pops its own command line window, enter for default
now you can see everything in mingw64

```
$ node --version, v20.15.0
$ npm --version, 10.8.1
$ yarn --version, 1.22.22
$ rustc --version, rustc 1.88.0 (6b00bc388 2025-06-23)
$ cargo --version, cargo 1.88.0 (873a06493 2025-05-10)

$ cargo install tauri-cli --version "^2.0.0" --locked
$ cargo tauri --version, tauri-cli 2.7.1

$ git clone https://github.com/zootella/fuji
$ cd fuji
$ yarn install
$ yarn build
$ yarn local
```

## (from scaffolding)

# Tauri + Vue 3

This template should help get you started developing with Tauri + Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
