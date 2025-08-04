notes:

```
## setup development workstation

### mac

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

second, to scaffold the project:

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

### windows 10


from that above, on mac, check the rust version







on windows, we want an older version, from late 2024, the last version of rust that builds executables that run for sure on windows 7


















```

from scaffolding:

# Tauri + Vue 3

This template should help get you started developing with Tauri + Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
