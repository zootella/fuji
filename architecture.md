# Fuji Architecture

Fuji is four layers: a **shell** that owns the window, one **sheet** and several **tables** that own what the user looks at, a **model** that owns what the user is looking at, and a **cache** that owns pixels. This file says where each thing goes and why, so that adding to fuji is a matter of finding the layer rather than rediscovering the shape.

It says what fuji does, not what it might. It is the only architecture document: `style.md` governs how code is written, `scaffold.md` how the project is set up, and where all three are silent, decide and write it down here.

```
App.vue
└── Shell.vue            the window: settings, reveal, window events, which view is showing
    ├── Sheet.vue        v-show   one sheet: thumbnails, tree, path box, sorts
    └── LightTable.vue   v-if     one of several tables: the image, full size
        ComicTable.vue            another table, whenever it is written
                ↓ both import, neither knows the other exists
        the model                 folder, sort order, list, index, back
                ↓
        the cache                 path → pixels, bounded
                ↓
        disk.js → disk.rs
```

## The shell

**The shell owns the window and none of the pixels.** It reads the settings file before anything else needs one, sizes and reveals the window, records where the user puts it, holds which view is showing, and owns the `c` key that switches between them. It draws nothing: no background, no HUD, no chrome. The sheet is black and the tables have their own surfaces, and neither has to negotiate with a parent about what it looks like.

**It exists because window events are global and everything else is not.** A view's `wheel`, `pointerdown`, and `dblclick` handlers live on its own element, so a hidden view receives none of them and two views cannot collide. But `window.addEventListener` fires regardless of what is visible, so `keydown` and `resize` are the entire interference surface between views. The shell holds one listener each and hands the event to whichever view is active. A hidden view cannot react to a key because it is never given one, rather than because it remembered to check.

## The sheet and the tables

**There is one sheet and there are many tables.** The user calls it a contact sheet; the code calls it `Sheet`. A table shows one image the way some kind of reader wants to see it — `LightTable` on an infinite pannable plane, a comic table as a full-width vertical scroll, and whatever else earns a place. The sheet shows a folder.

**`c` switches between the sheet and the current table, and they swap with `v-show`.** That switch is frequent and has to be instant with nothing reloading, which is what staying mounted means. Both keep their scroll, their pan, their decoded images, and their DOM.

**Tables switch between each other with `v-if`.** Choosing a different table is rare, a lost frame is fine, and a table nobody is using should not exist — five mounted tables each holding decoded bitmaps is the memory the cache exists to bound. Destroying one is affordable precisely because everything worth keeping lives below it: the new table reads the same model and asks the same cache, which answers from memory.

**Two orthogonal values, not one mode.** Whether the sheet is showing, and which table is behind it. `c` flips the first. A menu, a click, or a key picks the second. That they are independent is the whole reason the sheet does not care how many tables exist.

## The model

**The model holds what the user is looking at, and no view owns it.** The current folder, the sort order, the ordered list of images in it, the current index, and the history of where the user has been.

**Tables are interchangeable views of the same thing, and that is what forces the model down here.** A user on image 47 who switches from the light table to the comic table expects to still be on image 47. If the folder listing and the index lived inside a table, the second table would either duplicate them or reach into the first, and reaching in is how two components stop being separable. The same argument settles sort order: the user sets it in the sheet, then double-clicks a thumbnail and flips — and expects to flip in the order they set. So sort order is not the sheet's, even though the sheet is where it is chosen.

**The model holds the current path, not the current index.** A user on image 47 who changes the sort from name to date expects to still be looking at that picture, not at whatever is now forty-seventh, so the path is the durable value and the index is `list.indexOf(path)`. Flipping is find where I am, step one, take that path. Two things follow: a shuffled order is a permutation that has to be kept rather than derived, since name, date, and size can be rebuilt from the listing at any time and a shuffle cannot; and a current path that is no longer in the list, because the file was deleted or a filter excludes it, needs a rule rather than an `indexOf` of −1 sailing on.

**Sort order lives here even though the sheet is where it is chosen, and no view is told when it changes.** The sheet writes one value, the model rebuilds the ordered list from it, and every table is already looking at that list. A table written next year inherits sort order by importing the model; there is nothing to wire, because there is no wire.

**Back belongs here too, not to a router.** A file manager's back has to move through folders and images as one sequence and survive the `c` key, and no route history can hold that. The path box and the folder tree read and write the model the way a breadcrumb does.

## The cache

**The cache is a plain module keyed by path, and it knows nothing else.** Not which folder a path came from, not what order anything is in, not which view asked. That ignorance is what makes it correct: a user who clicks into a folder, clicks away, and clicks back gets a rebuilt listing, an unchanged sort order, and images that appear at once because the cache never heard about any of it.

**Getting an image on screen has two costs, and the second is the larger one.** Reading the bytes off the disk is noticeable. Decoding those bytes into pixels usually takes longer, and the result is far bigger than the file: a 6000 × 4000 photograph is about 96 MB of RGBA regardless of how small its JPEG was. A hundred of those is not a cache, it is an out-of-memory. So the cache is a few full-size decodes for the table and many small ones for the sheet, over one byte-reading layer they share.

**Prefer pixels fuji owns.** An image handed to the page as a data URL on an `img.src` is decoded by the browser, and fuji can neither measure that memory nor free it except by clearing the source. An `ImageBitmap` is an object with a size fuji can account for and a `close()` that releases it. A cache with a real byte budget needs the second kind.

**A bound goes in from the first line.** Fuji runs for weeks, and a cache without eviction fails slowly enough that no short test will show it.

## Settings

**Settings are not a fifth layer.** `fuji.toml` is the persistence facility the model and the views use for the values worth surviving a restart, and `settings.js` is the only file that knows what a setting is. Sort order belongs there, and so does which view the user was last in, so fuji opens where they left it. A current index does not. The rule is whether the user would be annoyed to lose it when they quit.

**The settings object is plain and not reactive, on purpose.** A value the interface has to re-render on — sort order, once the sheet has a toolbar — is stored in `fuji.toml` and watched in the model. Two homes, two jobs: one keeps it across launches, the other tells the page it moved.

## No router and no store

**Fuji has neither, and both absences are decisions rather than things not yet done.**

Routing sells services that belong to the browser as a document-navigation shell: addressable urls, a back button the browser supplies whether the app wanted one or not, code splitting over a network, server rendering. Tauri keeps the rendering engine and discards that shell, so none of them apply here. `App.vue` carries the longer argument at the spot a router would occupy. It would earn its place if fuji ever shared this code with a web build, registered a url scheme for the operating system to hand it, opened several windows each showing a different page, or grew many cheap parameterized destinations a user opens deliberately and expects to retrace.

**Shared state is an exported ref in a plain module, and that is the whole convention.**

    export const sortOrder = ref('Alphabet')//application state: the sheet sets it, every table reads it

A module is already a singleton that outlives every component, `ref` already makes it watched, and an import is already how anything reaches it. A store library adds a devtools timeline, state kept across hot reloads, subscription hooks, and a name — worth having where a team needs the convention spelled out for it, and not worth a second way of doing things here. What matters is that the pattern is written down and followed, which is the part a library was doing for free.

**The rule for where a value goes:** state the interface watches is an exported `ref`; state it does not watch is a plain binding; nothing large goes inside a deep reactive. That last clause is why fuji's most important state — the quiver, the triad, the drag object — is not reactive at all, and why one rule covers both kinds rather than a convention plus a standing list of exceptions.

**Large collections get `shallowRef`.** A folder listing is replaced, never edited in place, so proxying thousands of strings to watch for a change that always arrives as a whole new array is work for nothing.

## What this makes easy

**Writing a new table touches one line of the shell** — the entry that says which component that table is. The new table imports the model and the cache like every other table, and `LightTable.vue` and `Sheet.vue` do not change, do not learn it exists, and cannot be broken by it. If a new table ever requires editing an old one, the model layer has leaked and that is the bug to fix, not the table.

**Growing the sheet into a file manager** — a tree, a path box, a toolbar of sorts — is work inside `Sheet.vue` plus values in the model. No table changes.

## Rules that hold it together

- **Views never import each other.** The only thing they share is the layer beneath them.
- **The cache never learns about folders, order, or views.** A path in, pixels out.
- **A hidden view measures nothing.** `v-show` is `display: none`, which destroys the layout box, so `clientWidth` reads 0. A view that measures its container does it when it becomes active, never at mount.
- **Window events have one listener, in the shell**, and go to the active view.
- **Nothing large goes in a reactive proxy.** The quiver bypasses Vue on purpose, and a decoded bitmap inside `reactive()` is the same mistake with more zeros.

## What is built today

`LightTable.vue` and `settings.js` are real. The shell, the sheet, the model, and the cache are the plan above, being built in that order. `LightTable.vue` still holds folder and index as local bindings; they move to the model when it exists, and until then nothing new should join them there.

There is no router and no store library. `App.vue` renders the one view directly and `main.js` mounts the app and does nothing else.
