# Organize

A plan to move folders, written on 2026-11-21. Step one is next; nothing here has been done. It leans on architecture.md, whose four places, plastic, brass, copper and steel, are what the folders are arranged around, and on structure.md, whose names for fuji's parts are the names the folders take.

## Why

The top of the repository and the top of `desktop/src` have both grown broad without being grouped. The Rust crates float at the root beside the planning documents. `desktop` holds two folders that have nothing to do with how fuji runs or builds, the color page and the sort tests. And `desktop/src` is 49 files side by side, 46 of them JavaScript modules, the page's whole code with only the Vue components in a folder of their own.

The aim is a few useful groups, not depth. The user's words: not Java, where everything is seven levels deep, but it is too broad and not usefully grouped at present.

## Where it ends up

```
./site                          as it is
./linux                         as it is
./desktop                       only what Tauri needs to run locally and to build
./side                          side projects, one workspace
./side/color                    the color page, from desktop/color
./side/sort                     the sort tests, from desktop/sort
./rust/steel                    from ./steel
./rust/copper                   from ./copper

./desktop/src/App.vue           alone at the top
./desktop/src/components        the Vue files, and the plain modules under them
./desktop/src/plastic           every other module of the page, loose until a group is plain
./desktop/src/plastic/sort      WindowsSort.js, MacSort.js, StrcmpSort.js
./desktop/src/plastic/fit       the fits
./desktop/src/plastic/disk      what the page knows and asks about files on the disk
```

## The rules

**`desktop` holds what Tauri needs to run or to build, and nothing else**, even when the thing is about the desktop client. A design tool or a test of two of its modules lives in `side`.

**A side project is complete, correct, working and clean.** It is an experiment or a supporting thing, not part of how fuji runs or builds, and held to the same standard as what is. Each folder under `side` has a `README.md` that says what the thing is, why it exists, how to run it, what it found and what it needs, without sending its reader anywhere else for that.

**A folder under `plastic` is made when a group is plain, and not before.** Everything else stays a miscellaneous sea of modules in `plastic` itself. This is the start of sorting them and not the end. How a group is named, the user's rules, after turning down Claude's first list of five:

- **By the app's own structures.** structure.md names what fuji has, Sorts, Fits, Flows, Tables, Buckets, and a folder takes one of those names where its modules are that thing. So `sort` and `fit`, and not a word made up for the folder, as `pictures` was.
- **By the names the Rust already uses.** `disk` and not `files`, since `disk.rs` is what brass and steel call it.
- **The plain modules under a component go with the components.** `list.js`, `tree.js`, `bar.js` and the like are `components`, where their Vue files are, and not a folder of their own under another name.

## Step one: out of desktop, and the first folder of plastic

Next. Four moves, one commit each or one for all, as the user says. Before any of them, what is uncommitted goes in, so the tree is clean and a move shows as a move; and every move is made with `git mv`, so each file's history follows it.

**The sort tests, to `side/sort`.**

1. Move `desktop/sort`'s six files: `names.js`, `test.js`, `oracle.ps1`, `oracle.swift`, `windows-sort.txt`, `mac-sort.txt`.
2. Bring back the windows sort that was exact, the one that built Windows' own key from Microsoft's whole table, which fuji carried from 2026-11-20 and the user took out the next day for the size of its table in the shipped source. From commit `525fd07`: `desktop/src/WindowsSort.js`, as a module of `side/sort` under a name of its own, and `desktop/sort/table.js`, with the keys test in `test.js` that held it against Windows byte for byte. In `side` it is an asset where in the app it was a weight: it matched Windows on every name tried, so on a Mac or on Linux it can stand in for Windows and check the sort fuji ships; it is what the essay in fuji's `WindowsSort.js` speaks of; and nothing in `side` ships.
3. The table itself is not committed, Claude's suggestion and the user's choice: `table.js` fetches Microsoft's file, checks its hash and writes the packed table into a file git ignores, so the repository holds the readable code and the recipe, and Microsoft's data is in neither fuji nor the repository. Add that file's name to `.gitignore`.
4. Point `test.js` at the sorts' new home, `desktop/src/plastic/sort`.
5. Write `side/sort/README.md`.

**The color page, to `side/color`.**

1. Move `desktop/color`'s twelve files.
2. The page imports fuji's own stylesheet and components, and reads some as text, so its imports of `../src/...` become `../../desktop/src/...`.
3. It has been served by `desktop`'s Vite, at `/color/` on port 1430. `side` gets a Vite configuration of its own: the Vue and Tailwind plugins `desktop` uses, leave to read files under `desktop/src`, and `desktop/public` as its public folder, since the stylesheet's faces are fetched from `/fonts`.
4. Write `side/color/README.md`.

**`side` as one workspace.**

1. `side/package.json`, with the two scripts taken out of `desktop/package.json`, `color` and `sort-test`, and the packages the color page's build needs, at `desktop`'s versions: `vite`, `@vitejs/plugin-vue`, `@tailwindcss/vite`, `tailwindcss` and `vue`.
2. Name `side` in `pnpm-workspace.yaml`, with a line saying what it is, and run `pnpm install` at the root.
3. The page must load one copy of Vue. Its own files find Vue in `side`, and fuji's components, which it draws, find it in `desktop`; at one version pnpm gives both the same copy, and the Vite configuration says so outright, so a second copy is never loaded and a component is never drawn by a Vue that does not know it.

**The sorts, to `desktop/src/plastic/sort`**, which starts `plastic`.

1. Move `WindowsSort.js`, `MacSort.js` and `StrcmpSort.js`.
2. `model.js` imports all three, and is the only module of the page that does.
3. Each essay that says the other sort is beside it stays true, since they move together.

**The documents, for all of step one.** CLAUDE.md's commands, its section on the frontend and its account of the workspaces; `sort.md`'s paragraph on how the sorts are tested; `components.md` and the comment in `style.css` that say `pnpm color` draws the palette; `structure.md` and `architecture.md` where they name a sort's file; the essay in `test.js`, which says it runs from the desktop folder; the root README where it names either; the comment at the top of `.gitignore`, which lists the workspaces; and `linux/build.js`'s essay on the workspaces its containers leave out, which `side` joins.

**How step one is checked.**

- `pnpm install` at the root finds the new workspace.
- In `side`, `pnpm sort-test` passes as it does today: no disagreement for either sort on the recorded pairs in ASCII, and none on Windows in the ASCII, digits and emoji mixes.
- In `side`, `pnpm color` serves the page, every tab draws, and the Accent Colors tab's four panes say the faces they say today.
- In `desktop`, `pnpm vite-build` succeeds, which fails on any import that no longer resolves, and `pnpm local` runs with a sort chosen in settings.
- Nothing under `desktop` names `color` or `sort` as a folder any more.
- On the Mac, the same: `pnpm sort-test` there asks the Mac through `oracle.swift`, and `pnpm sort-test finder` reads Finder's windows.
- The Linux build is not checked by any of that, and is touched lightly: `pnpm-workspace.yaml` and the lockfile, which its containers copy, gain a workspace the containers do not, as they already lack `site` and `linux`, which `linux/build.js` says pnpm does not mind. The next `pnpm build` on the Mac is the proof.

## Step two: the crates, and the rest of plastic

Later, and after step one is committed and seen working on both machines.

**The crates, to `rust/steel` and `rust/copper`.** The Cargo workspace keeps its root at the top of the repository, with `desktop/src-tauri` still its third member.

1. Move the two folders whole, `steel` with its `fuzz` crate, its `samples` and its fit table.
2. The root `Cargo.toml`: its members become `rust/steel` and `rust/copper`, and its comment says so.
3. `desktop/src-tauri/Cargo.toml` reaches the two at `../../steel` and `../../copper`, which become `../../rust/steel` and `../../rust/copper`. `copper/Cargo.toml` reaches `../steel`, which stays true, since they move together.
4. `pnpm-workspace.yaml` names both by path, and its comments describe each.
5. `linux/build.js` copies `steel/` and `copper/` into its containers by those names, and its walk copies files out of `/src/copper`. Both lists change.
6. `copper/walk.js` finds the built fuji and a big tree by paths from the repository's root, and `steel/fit-table.js` imports `../desktop/src/fit.js`; each is one folder deeper after the move.
7. `.cargo/config.toml` keeps the target folder at `desktop/src-tauri/target`, which the pipeline, the bundlers and the containers all name. It does not move.
8. `steel/fuzz/Cargo.toml` reaches steel at `..`, which stays true.
9. **The site links to these files by their paths on GitHub.** `site/docs/security.md`, `bike-lock.md` and `erase.md` link to `steel/src/tip.rs`, `steel/src/lock.rs`, `copper/src/sandbox.rs`, `copper/src/erase.rs`, `copper/src/media.rs` and `copper/walk.js`, and each link breaks when its file moves. The pages change with the move and the site is published again, `pnpm upload` in `site`; a link somebody else copied from the site before then stops working, which nothing here can mend.
10. The words that place the crates: the READMEs of `steel`, `copper` and `linux` and the one at the root, CLAUDE.md, `architecture.md`, `mac2linux.md`, `pointer-and-drag.md`, the comments in `copper/src/inventory.rs`, `media.rs` and `thumbnail.rs` that name a path, and the comments in `scout.js` and `lib.rs` that call steel the safe crate at the repository root.

Checked by `pnpm test` in each crate, `pnpm fit-table` in steel, `pnpm walk` in copper and `pnpm compile` in `desktop`, on Windows and on the Mac. **The Linux packages can only be checked on the Mac**, where Docker runs: `pnpm build` in `linux`, about ten minutes, and `pnpm walk` there.

**The rest of plastic.**

1. Every module still loose in `desktop/src` moves into `desktop/src/plastic`, and the groups that are plain by then get their folders: `fit`, `disk`.
2. The plain modules under a component move beside it in `components`.
3. Every import among the modules, from the components, from `side`, and `steel/fit-table.js`'s of `fit.js`, is rewritten, by a script, since it is mechanical. `pnpm vite-build` fails on one missed.
4. The rule that each Rust module with commands has a JavaScript file of the same name in `src` becomes a rule about where those files are by then.
5. CLAUDE.md, `architecture.md` and `structure.md` say where everything is.

Checked by `pnpm vite-build`, `pnpm local` on both machines, and `side`'s two scripts.

## Still to decide

- **Which modules are `disk`.** Claude proposed `disk.js`, `find.js`, `scout.js`, `inventory.js` and `fileTypes.js`, the modules about files and folders on the volume, least sure of `fileTypes.js`, which is about kinds of file and not about reading the disk. Not settled.
- **Which modules move beside the components.** Claude proposed `list.js` and `selection.js` under `FujiList.vue`, `tree.js` under `FujiTree.vue`, `bar.js` and `bars.txt` under `FujiBar.vue`, and `label.js`, which the list, the tree and a thumbnail's caption all hand their text to. Not settled.
- **The modules that wrap Rust's commands**, one function a command and each named for its Rust module, 21 of the 46. Claude's first list gave them a folder, which the user neither took nor turned down. Left loose they are most of the sea.
- **Where `main.js`, `style.css` and `bars.txt` go**, with `App.vue` alone at the top of `src`. Two things tie them where they are: `desktop/index.html` loads `/src/main.js` by its path, and `style.css` tells Tailwind to look for class names in its own folder, so moved into `plastic` it has to be told to look across `components` and `App.vue` as well, or their classes stop being made, with nothing to say so.
- **Whether `windows-sort.txt` is still wanted**, 100 KB of what Windows said about every pair of the test names, once the exact sort is back to say the same. `mac-sort.txt` is the only record of what a Mac said and stays.
- **Whether the `fit` folder is worth making for one file**, or waits until the fits are more than one.
- **What the exact sort and its table are called in `side/sort`.** Fuji's own is `WindowsSort.js`, so the one that comes back needs a name that cannot be taken for it, and so does the file `table.js` writes.

## When it is done

This file is a plan, and goes when the plan is carried out: what it decided is by then in CLAUDE.md, `architecture.md` and `structure.md`, which say where everything is, and anything still open moves to the document that owns it.

## The modules, to sort from

As of 2026-11-21, in `desktop/src`: `appearance`, `associate`, `bar`, `brand`, `cache`, `desktop`, `dialog`, `disk`, `fileTypes`, `find`, `fit`, `flipCache`, `gamma`, `governor`, `icon`, `iconCache`, `inventory`, `keyboard`, `label`, `launch`, `list`, `locale`, `log`, `MacSort`, `main`, `media`, `memory`, `menu`, `model`, `open`, `pager`, `panel`, `paths`, `process`, `raster`, `registry`, `scout`, `selection`, `settings`, `StrcmpSort`, `thumbnail`, `touch`, `tree`, `uptime`, `window` and `WindowsSort`, with `App.vue`, `style.css` and `bars.txt` beside them.
