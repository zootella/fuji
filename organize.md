# Organize

Moving folders into place. Nothing is renamed but the color page, and every move keeps what links to it working.

## Where things go

```
desktop/                         the app and its build
desktop/src/                     App.vue, main.js and style.css at the top, the scaffolding every Tauri app starts with
desktop/src/components/          the Vue files, the front end
desktop/src/plastic/             every plain module: the app's logic and running state in JavaScript
desktop/src/plastic/sort/        StrcmpSort.js, WindowsSort.js, MacSort.js
desktop/src/brass/               the files that wrap brass's commands, one for each brass module with commands
desktop/src-tauri/               brass, as it is
rust/steel/, rust/copper/        the crates fuji authors, which stand alone
side/                            what does not ship, one workspace
side/palette/                    the color page, under its new name
side/sort/                       the sort tests, and the Windows sort built from Microsoft's table
linux/, site/                    as they are
```

## Decided

- **`components` holds the Vue files.** A plain module there now, like `library.js`, moves to `plastic`. An exception either way is fine where it makes sense.
- **A folder inside `plastic` is made once a group is plain.** `sort` is the first, and others come later.
- **Folder names are singular**, from fuji's own names and the Rust's.
- **`brass` keeps the scout's wrapper**, which nothing calls yet, since it was built ahead of a planned view.
- **`side` is one workspace**: a `package.json` at `side/` with the scripts `palette`, `sort-test` and `sort-table`. A README at `side/` says what `side` is, and one in each tool's folder says what the tool is, why it exists, how to run it, what it found and what it needs.
- **`side/palette` has a Vite configuration of its own in `side`**, reaching into `desktop/src` for the stylesheet and the components and into `desktop/public` for the faces, and loading one copy of Vue.
- **`side/sort` gets back the Windows sort that builds Windows' own key**, from commit `525fd07`, as `WindowsTableSort.js`. Its packed table is committed as `weight.js`, and `table.js` is the recipe that remakes it from Microsoft's `Unisort.txt`, at its address on the Download Center and checked by the SHA-256 that Wine's build pins too. The README names the file and its address. `windows-sort.txt` stays.
- **The sort test**: the key check against Windows comes back for `WindowsTableSort.js`, which is held exact on every pool of random names; fuji's `WindowsSort.js` is compared against it on any platform, its disagreements counted for each pool.
- **The Cargo workspace keeps its root at the top**, and the target folder stays at `desktop/src-tauri/target`. The site's links to crate files change with the move, and the site is published again after the commit.
- **`architecture.md`, `CLAUDE.md` and `structure.md` say where everything is**, and use plastic for the logic and JavaScript for the language in place of the page. Source comments change as each file is edited, and the articles stay as they are.
- **The placement rules go into `architecture.md`'s section on deciding where something goes**: `desktop` holds the app, and a concern that stands apart gets a folder at the root; `components` holds the Vue files; `plastic` holds the logic.

## The passes

Each is a turn or two, closed by its checks.

- **`desktop/src`.** Every plain module into `plastic`, the sorts into `plastic/sort`, the wrappers into `brass`, and the imports rewritten by a script. Checked by `pnpm vite-build`, and a search that nothing imports an old path.
- **`side`.** The palette and the sort tests move in, the table-built sort and its tests come back, and `side` joins the workspace list. Checked by `pnpm install`, `pnpm sort-test`, `pnpm vite-build` in `desktop`, and `pnpm palette` for the user to look at.
- **The crates.** `rust/steel` and `rust/copper`, with Cargo's paths, the workspace list, the Linux build's copy list and walk, the two scripts that find files from the root, and the site's links. Checked by each crate's `pnpm test`, `pnpm fit-table` in steel, and `pnpm compile`.
- **The documents.** The READMEs in `side`, the sweep of `architecture.md`, `CLAUDE.md` and `structure.md`, and every other mention of a moved path.
- **End to end.** `pnpm compile`, `pnpm walk` in copper, `pnpm build` and `pnpm walk` in `linux` with Docker Desktop running, `pnpm build` in `site`, and the user looking at `pnpm local` with each sort and at `pnpm palette`. Then the review, and one commit.
