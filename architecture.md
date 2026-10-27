# Fuji Architecture

Fuji's code lives in four places, and all four build into one executable. Plastic is the page, which is the application and decides everything; brass, copper and steel are the three Rust crates beneath it, which do the work the page asks for and decide nothing. This document says what each place is and what it demands of the code in it, how the page is layered, how a new piece of work is placed, how the Rust is tested, and the rules that hold it together, so that adding to fuji is a matter of finding the place and the layer rather than rediscovering the shape.

It says what fuji does, not what it might, and where a question about structure has no answer yet, decide it and write it down here.

```
App.vue                  which kind of window this is, read once as the page mounts
├── VideoWindow.vue      a video window: a native window of its own, beside a browsing window or without one
└── Shell.vue            a browsing window: settings, reveal, window events, which view is showing
    ├── Sheet.vue        v-show   one sheet: a page of buckets, a window onto the pager's list, with Previous and Next
    │   ├── Bucket.vue            a capped number of images, all from one folder
    │   │   └── TestFlow.vue      the flow that sizes and arranges them, and routes each file to a decoder
    │   └── BucketMemory.vue      beneath the last bucket: the machine's memory, fuji's processes, and what the thumbnails cost
    ├── SettingsPanel.vue v-if    what a user changes from inside fuji, in the sheet's window
    │   └── SettingsAbout.vue     the About section: fuji's parts and their versions, asked of each part
    ├── AudioPanel.vue    v-if    plays sound and never a picture, in the sheet's window like the settings panel
    └── DiamondTable.vue :is      one of several tables: one image, sized to a diamond
        ComicTable.vue            another table, whenever it is written
        PreviewTable.vue          the picture a double-click opened, alone, before any other table
                ↓ both import, neither knows the other exists
        the model                 folder, sort, ordered list, current path, which view is showing; the table's folder
        the pager                 pager.js: every image on the volume as one list, read a folder at a time; the sheet's page
                ↓
        the cache                 path → pixels, bounded
                ↓
        disk.js → disk.rs
        thumbnail.js → thumbnail.rs   beside the cache, for the sheet only: the operating system's thumbnailer
        memory.js → memory.rs         for the report and the log's line at rest: the machine's memory and what fuji's processes take
        media.js → media.rs           for the audio panel: the media library, libmpv, which plays a file and answers what one holds
        uptime.js → uptime.rs         for the log's launch line: how long the process has been running, by the system's clock and main's
        find.js → find.rs             for the pager alone: the next folders holding an image, a stretch of the tree per call
        scout.js → scout.rs           for no view yet: what a file's first bytes say, read in safe rust by the tip scan in steel, which the inventory asks of every file
                ↓ each .rs above is a command in brass, the one bridge, over the work beneath it
        copper                        every call into C and the operating system's libraries away from a window
        steel                         safe rust: the tip scan, the walker, the reads, the disk, find and the fits
```

## The four places

**The names are materials:** plastic for the page, and brass, copper and steel for the crates, because the metals share a nature and the page does not, and because a short word that means nothing else in this repository can be said in conversation, written in a document and searched for in the code without being mistaken for anything.

**Plastic is the page, and the page is the application.** It is the JavaScript and the Vue components in `desktop/src`, and it decides everything: what a setting is, which files to show and in what order, which kinds of file to offer the operating system, when to ask a question of the machine and how often, how many questions may be in flight, and when to stop asking. The interface the user works in, the state of the application and the logic that drives it all live here, and nowhere else. It holds the framework lightly, thin components calling down into plain modules, so that the logic outlives any framework. It acts on nothing past its own window except by asking brass, and it trusts what it is told. Plastic is molded and remolded, under every hot reload and every change of mind, and every question of product is answered in it and nowhere below.

**Brass is the crate Tauri builds the application from**, `desktop/src-tauri`, whose package is named fuji because its name is the executable's. It holds only what cannot leave: whatever is tied to a window, to the event loop, or to Tauri's own types. That is the commands, each a thin wrapper over work done elsewhere; the windows and the native surfaces a video draws in; the menu bar, the dock and the trackpad; the files the operating system handed fuji before any page existed; the system's dialog boxes; the log and the text written as the last window closes. Unsafe code is allowed here, because AppKit and Win32 are reached no other way at a window, and so brass is the least safe of the four places, kept as small as it can be and read the hardest. Everything plastic may ask of the machine is named once, in the handler list in lib.rs, and nothing else is reachable from JavaScript, so that list is the map of fuji's Rust and the whole of its attack surface, and adding a line to it is the considered act of widening that surface.

**Copper is the crate for Rust that needs neither Tauri nor a window and cannot be safe Rust.** Every call fuji makes into C and into the operating system's own libraries away from a window lives here: the media library that reads a file's structure and decodes its frames, the operating system's thumbnailers, the registry, the Mac's record of which program opens which file, the display's true size and the machine's memory, and the walls an operating system offers a process. Copper is the conductor between the program and the libraries, and it is where a search for unsafe finds every such call fuji makes outside a window. Each function in it is a plain function, so it is tested without Tauri compiled, and so it can be built into a binary of its own, which is how dangerous work comes to run in a process that fuji can kill.

**Steel is the safe crate.** Its root forbids unsafe code, and clippy denies an index that could be out of range, an unwrap, a panic and arithmetic that could overflow, so Rust's own tools hold the promise on every line and a reviewer does not have to. It holds every piece of fuji's Rust that can live under those rules: the tip scan, fuji's own reader of a file's first bytes, the walker of a folder tree, and the plain reading, arithmetic and folder work that need no library. It builds, tests and fuzzes with nothing of fuji around it. The bytes of a collection, which nobody vouched for, pass through steel wherever fuji writes the parser itself, and that is the reason it exists: the history of decoders written in C and C++ made to run code by a file is the history of the files a collection holds, and the code fuji writes to read them is code a file cannot subvert. The promise covers steel's own lines and nothing beneath them: under every file it reads and every folder it lists runs the standard library's own unsafe code, the most examined Rust there is, and the crates steel depends on stand outside the wall, as its manifest says of ergo.

The dependencies run one way. Brass depends on copper and steel, copper depends on steel, and steel depends on nothing of fuji's; all three depend on ergo, the small helpers that give the same answers in Rust as on the page. Plastic reaches the metals only through brass's commands. Nothing points up.

**Four places, and more than one process.** The main process is fuji as it starts, with its windows: it runs brass, copper and steel, and hosts the page. The page itself runs in the web engine's content process, which the engine sandboxes on the Mac and Windows. The sandbox process is fuji's own executable started a second time, for work that reads files nobody chose: brass's first lines hand it over before Tauri starts, so it runs copper and steel and nothing of brass after them, inside the walls copper's sandbox puts around it.

## The page

The page is four layers: a **shell** that owns a browsing window, one **sheet**, a **settings panel** and several **tables** that own what the user looks at, a **model** that owns what the user is looking at, and a **cache** that owns pixels.

### The shell

**The shell owns a browsing window and none of the pixels.** It reads the settings file before anything else needs one, places and reveals the window, decides which view is showing and writes that to the model so a flow can wait on it, and puts the window in fuji's fullscreen for a table and in an ordinary window for the sheet and the settings panel. It draws no background and no chrome. The sheet is black and the tables have their own surfaces, and neither has to negotiate with a parent about what it looks like.

**It draws one thing, the help panel, and owns its `h` key.** Help has to work in every view, always, for a user who has forgotten a shortcut or gotten lost, so it belongs to the layer every view sits inside — the same reason the shell owns `g` and draws the gamma filters. `HelpPanel.vue` floats over whichever view is showing, lists every key and mouse action fuji has, and takes no clicks, so the view beneath is still the thing being used. A new table adds its keys to the panel's text and inherits the rest.

**It exists because window events are global and everything else is not.** A view's `wheel`, `pointerdown`, and `dblclick` handlers live on its own element, so a hidden view receives none of them and two views cannot collide. But `window.addEventListener` fires regardless of what is visible, so `keydown` and `resize` are the entire interference surface between views. The shell holds one listener each and hands the event to whichever view is active. A hidden view cannot react to a key because it is never given one, rather than because it remembered to check.

### The sheet and the tables

**There is one sheet and there are many tables.** The user calls it a contact sheet; the code calls it `Sheet`. A table shows one image the way some kind of reader wants to see it — `DiamondTable` sizes it to an invisible diamond on an infinite pannable plane, a comic table would run it full width down a vertical scroll, and whatever else earns a place. The sheet shows a folder.

**A table is named for what makes it different, not for what they all are.** They are all light tables, which is why none of them is called one: `DiamondTable` is the one that keeps a diamond, and the next is the one that keeps whatever it keeps instead.

**The sheet scrolls over buckets, not over thumbnails.** A bucket holds a capped number of images from one folder and hands them to a flow, which decides sizing, arrangement, loading, and what is held. One flow governs every bucket at once, and it belongs to the sheet rather than the model, because arranging thumbnails is the only thing that consumes it.

**A double-click moves between the sheet and the current table, and they swap with `v-show`.** A double-clicked thumbnail opens its picture on the table, and a double-click on the table goes back to the sheet. That switch is frequent and has to be instant with nothing reloading, which is what staying mounted means. Both keep their scroll, their pan, their decoded images, and their DOM.

**Tables switch between each other with `v-if`.** Choosing a different table is rare, a lost frame is fine, and a table nobody is using should not exist — five mounted tables each holding decoded bitmaps is the memory the cache exists to bound. Destroying one is affordable precisely because everything worth keeping lives below it: the new table reads the same model and asks the same cache, which answers from memory.

**Two orthogonal values, not one mode.** Whether the sheet is showing, and which table is behind it. A double-click flips the first. A menu, a click, or a key picks the second. That they are independent is the whole reason the sheet does not care how many tables exist.

### The settings panel

**The settings panel is the third kind of view, and it lives in the sheet's window.** Which view is showing has four values: the sheet, the settings panel, the audio panel, or a table. The sheet and the panels share the ordinary window and trade places without it changing size, and a table has fuji's fullscreen. A panel comes and goes with `v-if` rather than staying mounted, because a user visits it rarely and it holds nothing worth keeping, and a key on the sheet reaches each until fuji has a better way in.

**The audio panel is the fourth kind, made like the settings panel.** It plays sound and never a picture, from audio files and video files alike, through the media library, and it is a view in the sheet's window for now so that the library and the builds could be worked on before fuji's windows are; whether it becomes a window of its own is decided after.

**A video window is not a view but a window of its own.** It stands beside a browsing window rather than taking its place, there can be several, and it shares none of the state the views do, so `App.vue` gives it `VideoWindow.vue` for its root instead of the shell. It reads the settings for its look and the log and never writes them, and on the Mac it answers only Fuji Help from the menu bar, which sends every other item past it to the main window behind.

**A setting reaches a view by being read when that view comes on screen.** The shell hands every view `start()` each time it appears, and the sheet is never showing while the panel is, so a view reads what it depends on as it arrives and nothing has to watch a setting change. That is what lets the settings object stay plain, as the section on settings below says.

**The panel is a container of sections, and what a section decides lives below it.** A setting that is one box is a line of the panel's own template, committed through `settings.js`, so the schema that reads the file also judges the box. A section with a life of its own — state of its own, data from the operating system, timing — is a component of its own, and its policy lives in a plain module the way the model's does, so the component shows that module's state and hands the user's clicks down.

### The model

**The model holds what the user is looking at, and no view owns it.** The current folder, the sort order, the ordered list of images in it, the current path, which view is on screen, and the history of where the user has been.

**Tables are interchangeable views of the same thing, and that is what forces the model down here.** A user on image 47 who switches from one table to another expects to still be on image 47. If the folder listing and the index lived inside a table, the second table would either duplicate them or reach into the first, and reaching in is how two components stop being separable. The same argument settles sort order: the user sets it in the sheet, then double-clicks a thumbnail and flips — and expects to flip in the order they set. So sort order is not the sheet's, even though the sheet is where it is chosen.

**The model holds the current path, not the current index.** A user on image 47 who changes the sort from name to date expects to still be looking at that picture, not at whatever is now forty-seventh, so the path is the durable value and the index is `list.indexOf(path)`. Flipping is find where I am, step one, take that path. Two things follow: a shuffled order is a permutation that has to be kept rather than derived, since name, date, and size can be rebuilt from the listing at any time and a shuffle cannot; and a current path that is no longer in the list, because the file was deleted or a filter excludes it, needs a rule rather than an `indexOf` of −1 sailing on.

**A path is fuji's name for a file, and on two of its three platforms it is not a unique one.** The current path is an exact string, found in the list with `indexOf`, and the cache keys on the same string. macOS on APFS and Windows on NTFS are both case-insensitive by default, so `photo.jpg` and `PHOTO.JPG` are one file there and two entries here; on Linux they are genuinely two files, which is why folding case everywhere would be wrong rather than merely unnecessary. The listing fuji builds for itself is safe, because every path in it is joined from the name `disk_readdir` returned and `listFolder` lowercases only the extension it filters on, never the path. The exposure is a path that arrives from anywhere else and has to be found in that listing: a drag and drop today, and a double-click in Finder or Explorer if fuji is ever a handler for an image type. Either can hand over a spelling the disk accepts and `indexOf` does not, and `listSiblings` marks the −1 it silently turns into 0. Read from the code and from what each filesystem documents, not exercised — no one has yet dropped a miscased path on fuji and watched.

**Sort order lives here even though the sheet is where it is chosen, and no view is told when it changes.** The sheet writes one value, the model rebuilds the ordered list from it, and every table is already looking at that list. A table written next year inherits sort order by importing the model; there is nothing to wire, because there is no wire.

**Back belongs here too, not to a router.** A file manager's back has to move through folders and images as one sequence and survive moving between the sheet and the table, and no route history can hold that. The path box and the folder tree read and write the model the way a breadcrumb does.

### The cache

**The cache is a plain module keyed by path, and it knows nothing else.** Not which folder a path came from, not what order anything is in, not which view asked. That ignorance is what makes it correct: a user who clicks into a folder, clicks away, and clicks back gets a rebuilt listing, an unchanged sort order, and images that appear at once because the cache never heard about any of it.

**Getting an image on screen has two costs, and the second is the larger one.** Reading the bytes off the disk is noticeable. Decoding those bytes into pixels usually takes longer, and the result is far bigger than the file: a 6000 × 4000 photograph is about 96 MB of RGBA regardless of how small its JPEG was. A hundred of those is not a cache, it is an out-of-memory. So the cache is a few full-size decodes for the table. The sheet's small ones are canvases its flow owns, outside the cache.

**Prefer pixels fuji owns.** An image handed to the page as a data URL on an `img.src` is decoded by the browser, and fuji can neither measure that memory nor free it except by clearing the source. A canvas is memory fuji sized, with a byte count it can total and that the engine cannot take back. The sheet's thumbnails are that kind: a canvas per picture, its pixels from the operating system through `thumbnail.rs` where the platform allows and from the page where it does not.

**A bound goes in from the first line.** Fuji runs for weeks, and a cache without eviction fails slowly enough that no short test will show it.

### Settings

**Settings are not a fifth layer.** `fuji.toml` is the persistence facility the model and the views use for the values worth surviving a restart, and `settings.js` is the only file that knows what a setting is. Sort order belongs there, and so does the size the user last gave the contact sheet's window. Which view fuji opens on does not: a double-clicked picture opens as a preview and every other launch on the sheet, whatever the user was looking at when they quit. A current index does not either. The rule is whether the user would be annoyed to lose it when they quit.

**The settings object is plain and not reactive, on purpose.** A value the interface has to re-render on — sort order, once the sheet has a toolbar — is stored in `fuji.toml` and watched in the model. Two homes, two jobs: one keeps it across launches, the other tells the page it moved.

### No router and no store

**Fuji has neither, and both absences are decisions rather than things not yet done.**

Routing sells services that belong to the browser as a document-navigation shell: addressable urls, a back button the browser supplies whether the app wanted one or not, code splitting over a network, server rendering. Tauri keeps the rendering engine and discards that shell, so none of them apply here. `App.vue` carries the longer argument at the spot a router would occupy. It would earn its place if fuji ever shared this code with a web build, registered a url scheme for the operating system to hand it, opened several windows each showing a different page, or grew many cheap parameterized destinations a user opens deliberately and expects to retrace.

**Shared state is an exported ref in a plain module, and that is the whole convention.**

    export const sortOrder = ref('Alphabet')//application state: the sheet sets it, every table reads it

A module is already a singleton that outlives every component, `ref` already makes it watched, and an import is already how anything reaches it. A store library adds a devtools timeline, state kept across hot reloads, subscription hooks, and a name — worth having where a team needs the convention spelled out for it, and not worth a second way of doing things here. What matters is that the pattern is written down and followed, which is the part a library was doing for free.

**The rule for where a value goes:** state the interface watches is an exported `ref`; state it does not watch is a plain binding; nothing large goes inside a deep reactive. That last clause is why fuji's most important state — the quiver, the drag object, the folder listing — is not reactive at all, and why one rule covers both kinds rather than a convention plus a standing list of exceptions.

**Large collections get `shallowRef`.** A folder listing is replaced, never edited in place, so proxying thousands of strings to watch for a change that always arrives as a whole new array is work for nothing.

## Deciding where something goes

The first question about any new thing is whether it is a decision, a piece of application state, or the orchestration of work: which, in what order, how many at once, what next. If it is, it is plastic, and the question is over. The metals never decide, hold a fact about the application, or sequence anything on plastic's behalf. TestFlow.vue is the worked example: it decides per file whether a thumbnail comes from the operating system or from the page, runs two loops at different widths, and stops them when a bucket goes away, while the Rust beneath it does nothing but answer one question about one file.

Work goes below plastic for three reasons and no others. Speed: hashing a file, or reading a tree of a million names, where many files cost less to read than one trip across the bridge costs to make. Reaching the operating system or a library: a window, a menu, a thumbnail from the system's own imaging library, a probe through FFmpeg. Holding a security wall: a check that must stand before plastic could even look, like refusing a header that claims more memory than the machine has before a decoder allocates. A task is never sent below because it is "mostly Rust" or "involves Rust". The line is logic against work.

Once something is work, the metal is decided by two questions in order. Is it tied to a window, the event loop or a Tauri type? Then brass, and only then. Otherwise, can it live under steel's rules, no unsafe, no call into a library, nothing that could panic? Then steel. If it cannot, copper. The order matters: a thing goes to copper only after steel has refused it, and to brass only after both have.

The rule is applied while code is written, not only when existing code is sorted. A function in steel that wants its first unsafe call has to move to copper, and one in copper that wants its first Tauri type has to move to brass, and that move is the question asked at that moment: why does this need to be unsafe, why does this need Tauri, and is there a way to write it that does not. Most of the time there is. The code stays where it is, and what it wanted turns out to be a parameter, a return value, or a thin wrapper one crate up. This is the shape doing the thinking, so that the safe crate stays safe and the dangerous crate stays enumerable without anyone having to remember.

A feature's code therefore splits by what each part needs, and that is accepted. Elsewhere in fuji a feature is a vertical column kept in one file, because code that changes together should live together; where that rule meets this one, this one wins. The inventory is the example. Its walk and its reading of each file's first bytes are steel. Its loop, which also asks the media library about every video, is copper, because one of the things it calls is unsafe. Its commands, with the counters they share so the page can watch progress, are brass. The panel that starts it, the choice of which hashes to compute, the name of the file it writes, and the decision to run it in the main process or in the sandbox process are plastic.

## The boundary

The commands in brass are the one bridge between plastic and the metals, and six rules shape them.

**A command takes one thing.** A list stays in the page, which calls once per item and owns the order, how many are in flight, and when to stop; so the command that renders a thumbnail takes one path, and the one that writes a registry value writes one value. A crossing costs little, and a loop below over what the page handed down is a decision taken away from the page. The exceptions below say when a call may take many, and the tests it has to pass.

**A command that waits runs its body on the blocking pool.** Tauri runs a plain command on the thread that runs the window, one at a time in the order they arrive. So a command that works on the window stays plain, because on that thread Tauri makes a change at once, where from anywhere else it only queues the change and the command answers before the window has moved. A command that waits, on the disk, the processor, the user or another program, is an async function that hands its body to run_blocking in lib.rs, so a slow read or a decode never holds up a window event or another command's reply, and a panic in it comes back to the page as an error rather than ending the process. The rest return at once and are plain. On the Mac every such body runs inside an autorelease pool of its own, because a thread Tokio made has none and Apple's frameworks assume one. The essay above disk_readdir in disk.rs says why the bodies go to Tokio's blocking pool rather than Tauri's own workers, and what that costs.

**Rust trusts the page, and guards only what the page cannot.** A check below that repeats a decision the page made is a second copy of it, and second copies go stale. So the commands on the disk take any path and hold no guard, and the essay that opens them is the contract: the full, standard power a desktop application has over the disk, sharp edges included, with the care that implies for the code that calls it. The trust rests on three walls around plastic. Every path it acts on came from the user or was built by fuji itself, never from outside content. Untrusted text reaches it only through Vue's escaping interpolation, so it never becomes script. And the content security policy in tauri.conf.json keeps foreign script out even if one of those cracks. The walls the metals do hold are the ones that must stand before the page could look: the thumbnailer refuses a header claiming more memory than the machine has, before any decoder allocates. The next, a scope of folders the user has shown fuji, outside which every disk command refuses a path, waits for deleting to arrive, and disk.rs names it.

**The commands are plastic's only road past its window.** Tauri's plugins would give it a second one, JavaScript bindings granted in capabilities/default.json with scopes of their own, which suits an application with no Rust of its own. Fuji has three crates of Rust, so it grants the page no plugin at all: that file holds Tauri's core set and the calls the page makes on its own window, its title, its theme, its fullscreen and the events it hears, because running that frame is the interface, and nothing past them. So widening what the page may ask is a Rust change reviewed as code, never an edit to a policy file. Where a plugin's Rust half does the work well, a command calls it: dialog.rs calls the dialog plugin's off the Mac, registered for that and granted to nobody, and process.rs opens an address or a file through the crate Tauri's opener plugin wraps, and needs no plugin at all.

**No window is made in setup.** Every window but the ones a page asks for through window_new on the Mac, and the dock menu's New Window, comes from the event closure in lib.rs under one rule: Ready makes a window if there is none. Off the Mac, window_new starts a copy of fuji instead, whose window comes from Ready like any launch's. A double-click on the Mac delivers Opened before Ready, so the picture already has its window; on Windows and Linux Opened never fires, so Ready always makes it. Making a window in setup instead is what once opened two for one double-click, one of them blank, because the operating system may ask for a window, with the file the user double-clicked, before setup has finished.

**The platform question is asked once, where a thing is made, and never where it is used.** One function makes every window, and nothing downstream asks again. Write for the plural and let the simpler platform be the degenerate case: paths waiting for a page are keyed by window label everywhere, and Windows runs that map with one entry forever, needing no branch to do it. Where a rule can be identical for free, make it identical: settings and the log flush when the last window closes on every platform, not only at a quit. The alternative, per-process on one platform and per-window on another, is what makes every later feature ask which world it is in.

## The pitfalls

**Two brains.** The worst thing that can happen to this design is for application state or orchestration to live in both plastic and a metal. Then there are two places that know what the user is doing, and they have to agree, and keeping them agreeing is a conversation across the bridge that never ends and is never quite in step. The signs are the rules above broken: a check that repeats a decision the page made, a loop over a list the page handed down, a command that has to know why it was called or that behaves differently for one caller than another. Each is application logic that has leaked down, and the remedy is always the same: the metal keeps the one dumb thing it does well, and the decision goes back up.

**The leak of names.** A metal that knows it is part of fuji has already started to think like fuji. Even the product's two names reach Rust from the project's configuration rather than being spelled in any crate: through package_info, brandName from the product name in tauri.conf.json and brandStem from the package name in Cargo.toml, the same two files plastic reads them from through brand.js. The test for a new command is to describe it without naming a fuji feature. "Write this string value under the current user" passes. "Register fuji's image types with Windows" fails, and became a general command that writes one value and a page module that decides which values to write.

**Widening the surface by habit.** Every line in brass's handler list is a thing a compromised page could ask for. A command added because it was convenient, or granted a scope wider than its feature needs, widens that surface for nothing. Each command is as narrow as the feature that needs it, and the list is short because the discipline above keeps the metals simple; the two are the same discipline seen from different sides.

**Parsing in the wrong metal.** Code that reads bytes nobody vouched for belongs in steel, where the compiler holds it, and writing such a reader in copper or brass because the file it reads happens to be near a library call there throws away the one wall a crate can add. The reverse mistake is unsafe code in brass that needs no window; that is copper's, where it is counted.

## The exceptions, and how they are judged

Some state does live in the metals, and some loops, and the rule that allows them is narrow. What a metal holds is a mirror of the machine or a count of its own work, never a fact about the application. The walker's place in a folder tree, as it reads a million names in one call, is state, and it is simple, bounded, and a reflection of the disk rather than a decision. The inventory's counters, how many folders and files and bytes so far and which path is being read, are a count of the metal's own work, which the page reads each frame and decides what to show. The three tests are whether plastic can ask for the thing, whether plastic can stop it, and whether the metal ever has to tell plastic what to do next. Reading the next folders of a tree in find's order, many per call, passes all three: the page asks, the page stops, and the answer is a list the page sorts and shows as it likes.

Speed decides even the route of a single thumbnail: it comes from the operating system's imaging library through copper where that is faster than the page's own decode, and from the page where it is not, while every decision about which thumbnails to make, in what order, and how to size and place them stays in plastic.

## The metals serve plastic and do not know it

From the way they are written, steel and copper are a general library of desktop work: read a file, walk a tree, say what bytes are, hash them, ask the media library a question, write a registry value, make a thumbnail of a path, measure the memory. A sister project could take either crate as it stands. Brass is fuji's by construction, since it embeds fuji's configuration and makes fuji's windows, but its commands are written to the same standard, general and stateless, and none of them knows what it is part of. Plastic is one brain, the metals are tools on its bench, and a tool that does one thing and does not care why has nothing to get wrong about the application.

## Why three crates

One crate would have been simpler to name. Three were chosen for what the boundaries hold. The compiler forbids unsafe code at a crate's root and nowhere smaller, so steel's promise is only possible as a crate; a module can forbid unsafe in itself and still call the rest of its crate's. A crate without Tauri builds and tests in seconds, where a test inside brass compiles all of Tauri first and has to be shown where the media library is. Code that runs only on Windows can be type-checked in copper from a Mac, which Tauri's build script prevents in brass. And a crate can be built into a binary of its own, which a module cannot.

The costs are a feature split across crates and a third place to look. The three are one Cargo workspace, so a dependency resolves once in one lockfile at the root and compiles once into one target folder.

## Testing the Rust

How fuji's Rust is tested: what a test is for, where one lives, what shape it takes, what it should reach for, and the tools in the order they earn their place. The constitution's code chapter governs the code a test is written in; this section governs the tests, and it is written for a reader who knows testing from JavaScript and is new to Rust.

### What a test is for

A test suite is good when it makes the software easy to change and hard to break, in that order. Every other property follows from those two, and most bad suites fail the first while appearing to serve the second: they are long, present and complete, they break on every refactor, and they would pass against a program that gave the wrong answers.

The test for a test is the one [Alex Kladov](https://matklad.github.io/2021/05/31/how-to-test.html) gives, which he calls the neural network test. Replace the code under test with an opaque box that produces the right answers. Would the suite still pass? If it would, the suite tests what the code does. If it would not, the suite tests how the code is shaped, and it will have to be rewritten with every change to that shape while catching nothing a user would notice. A test that asserts a function was called, or that a private helper returns a particular structure, or that a list has the length the current loop happens to produce, fails this test. A test that feeds a file and checks what comes out passes it.

[Kent Beck's twelve properties](https://testdesiderata.com/) of a good test are sliders rather than a checklist, and three of them carry the weight here. A test should be *behavioral*: sensitive to changes in what the code does. It should be *structure-insensitive*: unchanged when the code is rearranged. And it should be *predictive*: when the suite passes, the code is fit to ship. The others, isolated, deterministic, fast, writable, readable, automated, specific, composable and inspiring, are the conditions under which the first three stay true over years.

So a suite is seldom one to one with the code. A few lines that decide something subtle, the order a tree is walked in or the byte a hash starts at, earn a large and careful suite, and a hundred lines of wiring earn none. What decides the weight is what the code could encounter and what it would cost to get wrong, never how much code there is.

### Where a test lives

Rust gives three places, and each has a job.

**A test module in the file**, marked so it compiles only under `cargo test`, is the home of most tests. It sits at the bottom of the file it tests, and `use super::*` gives it the file's private items, which is how a pure core can be tested without being made public for the purpose.

```rust
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_zero_size_in_the_header_is_corrupted() {
		let t = scout(&png_header(0, 20));
		assert_eq!((t.corrupted, t.reading.details), (true, "a size outside the format's range"));
	}
}
```

**The `tests` folder beside `src`** holds tests that use the crate only through its public interface, the way another crate would. Each file there is compiled as a crate of its own. Fuji has few of these, because the crates' public surfaces are small and thin, and the interesting behavior is a function or two below them, which the module test reaches.

**A doc test** is an example in a `///` comment that `cargo test` compiles and runs. Fuji does not write them: the comments here are prose at the line's own altitude, not examples, and an example that must compile pulls the comment's attention toward the mechanics of calling.

When a file's test module grows past its code, it can move to a file of its own beside it, `tip/tests.rs` for `tip.rs`, declared by `mod tests;` under the same `cfg`, keeping its private access. A shared helper for several files' tests, a tree a test builds, a sample's path, goes in one module declared under `cfg(test)` from the crate's root, as steel's `testing.rs` is.

The safe crate's lints apply to its tests, so a test module there opens with the one line that lets a test index, unwrap, add and panic, since a mistake in a test fails the test, which is what a test is for. Nothing lets a test write unsafe code.

### The shape of a good test

**A test is data against the code.** The strongest tests in this repository are tables: a sample file beside the answer its maker's own tools give, a built tree beside the order it should list in, a set of arguments beside the numbers a twin in another language produced for them. One check function runs the table, and a new case is a new row. This is Kladov's second rule, make adding a test trivial, and it is what lets property tests, exhaustive tests and a fuzzer grow from the same soil later: once an input is data, anything can produce it.

```rust
#[test]
fn samples_answer_as_their_makers_made_them() {
	let rows: &[(&str, &str, (u32, u32), f64)] = &[
		("picture.png", "png", (32, 20), 0.0),
		("video.mkv",   "matroska", (32, 20), 1.0),
	];
	let mut wrong = vec![];//every row that disagrees, so one run shows them all
	for &(name, format, raster, seconds) in rows {
		let t = scout_sample(name);
		if (t.format, (t.raster.width, t.raster.height)) != (format, raster) { wrong.push(format!("{name}: {:?} against {:?}", t.format, format)) }
		if (t.seconds - seconds).abs() >= 0.05 { wrong.push(format!("{name}: {} seconds against {seconds}", t.seconds)) }
	}
	assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
```

Collect every disagreement before failing. A table that stops at its first wrong row makes the reader run it ten times to learn ten facts. A table is one test to cargo: its rows are not in the list `cargo test` prints, one row cannot be run by name, and a row that panics rather than disagrees stops the table there. [rstest](https://docs.rs/rstest) and [test-case](https://docs.rs/test-case) make each row a test of its own with a name in the list; fuji takes the loop for how little a new row costs to add and read, knowing the trade.

**A test builds its own world and takes it down.** A folder under the system's temporary directory, named for the test, filled with small files whose contents are their own names so their sizes are known, and removed when the test ends through `Drop`, which runs whether the test passed or panicked. The name is what keeps tests apart: cargo runs the tests of one binary as threads of one process, so they share its id, and two tests that chose one name would meet, each wiping the other's folder as it started or finished. The process id in the name separates two runs in two terminals and nothing more, so a name is unique or the test is wrong. [tempfile](https://docs.rs/tempfile)'s `TempDir` is the crate the world uses for this, picking a name nothing else has and removing it in `Drop`, and it is the move when the hand-built helper is outgrown. Nothing is read from the developer's home folder, nothing depends on what another test left behind, and the pinned sample files under `samples/` are read and never written.

**A test's name is a sentence about the behavior**, in the lowercase words the code uses, so the list `cargo test` prints reads as the specification: `a_link_is_listed_and_never_followed`, `the_pipe_opens_without_waiting_for_a_writer`, `forward_from_a_folder_finds_the_folders_after_it_in_sorted_preorder`. A name that says what function is called, `test_walk_2`, says nothing when it fails.

**An assertion says what was seen.** `assert_eq!` on a tuple compares several facts in one line and prints both sides on failure. Where a bare `assert!` must do, its message carries the values, `"{} by {}", r.x, r.y`, so the failure is specific, in Beck's word: when a test fails, the cause is obvious. The comment on an assertion says why the expected value is what it is, when that is not plain from the arrangement, "by bytes, so capitals before lowercase".

**No test waits on the clock and no test shares state.** Rust runs tests in parallel threads of one process, so a test that changes the current directory, an environment variable or a global changes it for every test running beside it. Time-based tests are flaky by construction. The one place a thread and a timeout belong is proving that a call *does not* block, the open of a named pipe, say: the call runs on a thread of its own, the test waits on a channel for a bounded time, and a late answer is the failure. Kladov's rule for concurrency in tests is "don't do this", and that exception is the shape it takes when the thing under test is the not-blocking.

### What to reach for

**Test at the boundary where behavior is stable.** For a library crate, the functions a caller uses. For a function that walks a tree, the sequence it produces. Never the private helper's intermediate value, which is structure. If a private function needs a test of its own, it is a pure core that deserves one, like the arithmetic of a fit, and the test treats it as a boundary of its own: arguments in, answers out.

**Test what the code could encounter, not what it is.** The questions to ask of any function, in the order they find bugs:

- *Nothing.* An empty file, an empty folder, an empty string, a zero size, a zero timeout.
- *Too much.* A header that claims a billion by a billion, a chunk longer than the file, a tree deeper than a thread's stack, a count that would overflow a `u32`.
- *Missing or refused.* A path that is not there, a folder the user may not read, a file that vanishes between the listing and the open.
- *Lying.* A file whose name says one format and whose bytes say another, a size field that points past the end, a structure that loops back on itself.
- *Odd names.* Quotes and newlines in a file name, a name that is not valid UTF-8, a trailing dot or space on Windows, a dot file.
- *The platform.* A symbolic link, a named pipe, a socket, a device, a cloud placeholder, a reparse point, a case-insensitive file system, the other slash.
- *Stopping.* A stop flag set before the first item, and set from another thread partway through.

Each of these is a row in a table or a tree built for it, and the code's answer to each is a behavior a user will meet. One row meets the harness itself: a test runs on a thread whose stack is 2 MiB, as a command body on the blocking pool does, where a program's main thread has 8, so a test of depth either runs its walk on a thread whose stack size it sets to the one fuji's code will have, or says in its comment which thread it answers for.

**Test contracts across calls.** The most valuable tests are often about a sequence rather than a call. A walk that answers where it stopped promises that continuing from there produces the rest with nothing skipped and nothing repeated: the test walks in steps of one and compares the sequence with one walk in a single step. A reader that answers in two slices promises the same answer as one that saw the whole file: the test cuts the input at several places and compares. These are the tests a refactor is most likely to break and least likely to be caught by anything else.

**Errors are values here, so test them as values.** A command answers `Err` with words, never a panic, and a test checks the words where they carry meaning, `refused.contains("could not open")`, and the kind of failure where the words are the platform's own. In the safe crate nothing may panic, and three things hold that between them, since no one of them can. Clippy's lints refuse the panic sites they know, an index or a slice that could be out of range, an unwrap, an expect, a panic and the arithmetic operators, and they hold only while `cargo clippy` runs: rustc leaves clippy's namespace alone and `cargo test` never runs it, so a change to the safe crate is checked with `cargo clippy` in its folder before the commit. Safe Rust panics in places no lint sees, `split_at` past the end, `copy_from_slice` on two lengths that differ, a capacity that overflows in `with_capacity` or `repeat`, a sum that overflows in a debug build, and a stack overflow or a failed allocation aborts the process without a panic at all; the code holds those by taking no buffer's size and no loop's length from a number the file gave, and the tests and the fuzzer hold them where the code is wrong. So a test there is never written with `#[should_panic]`, and a panic found by the fuzzer is a bug.

**Never call a function that changes the user's machine.** A test may ask the Mac which application opens a kind of file; it never sets one. A test may read the registry under a key of its own and must delete what it wrote; it never touches a key the user owns. A test may read the display's size and the machine's memory; it never changes a setting. Where a function's whole job is such a change, it is tested by the user, by hand, on each machine, and the test suite says so by leaving it out.

**What the user tests, the suite leaves alone.** The constitution is firm that a visual outcome is checked by eyes on the running program and never by a harness: a thumbnail's look, a window's placement, a panel's text. A test may check that a thumbnail came back at 16 by 10 with 640 bytes of pixels; whether it looks right is not a question a test can answer, and a test that pretended to would be the vanity kind.

### The tools, in the order they earn their place

**Example tests** come first and stay. They hold the known cases, the samples, the past bugs with the input that found them, and they are the specification a reader learns the code from.

**Tables** are example tests that stopped repeating themselves. When the third test of a function looks like the first two with different numbers, it becomes a row.

**An expect test** is a table whose answers the tool writes. [expect-test](https://github.com/rust-analyzer/expect-test) and [insta](https://insta.rs/) run the code, compare what came out with the answer kept in the test or in a file beside it, and on request rewrite the kept answer to what came out, so a person reviews a diff rather than typing a dozen fields by hand; Kladov's own suites are built on them. They fit where an answer is a structure with many fields, as the tip's for a sample is, and the typed table stays where the answer is a few numbers a person should derive, since an answer accepted from the code is only as right as the eyes that read the diff.

**A twin in another language** is a table generated rather than typed. Where the same arithmetic is written in Rust and in JavaScript, as the fits are, the strongest test runs the JavaScript over a grid of inputs and writes a file both sides read, so the two are checked across thousands of arguments rather than the dozens a person pins by hand. The file is data, regenerated by hand when the arithmetic changes on both sides at once, and a divergence anywhere in it is a failure that names its row.

**Property tests** generate inputs. [proptest](https://proptest-rs.github.io/proptest/intro.html) runs an invariant over hundreds of generated values and, on a failure, shrinks the input to the smallest case that still fails, which is the part that makes them usable: the counterexample arrives small enough to read. They fit where a function has a contract that holds for every input, round trips above all, a line written then parsed back, a walk in steps against a walk in one, a size turned twice coming back as it was. They complement example tests and never replace them: the examples hold the cases a person knows matter, and the properties search for the ones nobody imagined. A property test that only restates the implementation, computing the expected value with the same formula, proves nothing, which is the same failure as a vanity example test.

**Fuzzing** is property testing with the property "does not crash" and an engine that learns from coverage which inputs reach new code. For anything that reads bytes nobody vouched for it is the strongest tool there is, and the safe crate runs [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html) over its reader, seeded from the samples. A finding is one of three things: a panic, an input that runs past the engine's timeout, and an input that pushes memory past its limit, both limits set on the run with `-timeout` and `-rss_limit_mb`. For a reader of headers the last two matter as much as the first, since a structure that loops back on itself hangs and a length that sizes an allocation takes the machine's memory, and the reader is built so neither can happen; the fuzzer is what shows it. A target may also assert a property rather than only survive, the same answer from two slices as from the whole file, say, and does for hours what the sweep test does for a bounded run. A fuzzer needs a pure function with bytes in and an answer out, which is one more reason to keep compute away from input and output.

**Mutation testing grades the suite.** [cargo-mutants](https://mutants.rs/) copies the crate, plants one small bug at a time, an operator swapped, a function returning a default, and runs the tests expecting them to fail. A mutant that survives is a place where the code can be broken and no test notices: code that is buggy, unnecessary, underspecified, or hard to test, as its author puts it. It is slow, an incremental build and a test run per mutant, so it runs on a crate with a fast suite and no flakes, when the suite is in question rather than on every change. Its report is the list of what to test next, and it is a better list than any person's guess.

**Coverage is a map, not a score.** A coverage tool shows which lines ran during the tests, and that is useful for finding a branch no test reaches. It cannot show whether any test checked what those lines did; a test that calls a function and asserts the result is not empty covers every line and verifies nothing. The number is not a goal, and a suite tuned to raise it is the junior suite from the first section.

### Speed

A suite is fast when input and output stay in the caller and compute in the callee, so that most tests exercise compute on data already in memory. The slow tests are the few that must touch the world: a real folder, a real decoder, a real player. They stay in the suite, since they are the ones that find what the pure tests cannot, and they are kept few and small, a 32 by 20 sample rather than a photograph.

A test that takes seconds is marked `#[ignore = "..."]` with its reason in the attribute, which `cargo test` prints beside the word ignored, and run by asking for it with `cargo test -- --ignored`, never gated behind conditional compilation, which hides it from the type checker as well. `cargo test` prints a line per test, ok or FAILED, and swallows what a passing test printed; `cargo test -- --nocapture` shows it, and `cargo test name` runs the tests whose names contain the word. A test that failed on one platform's run prints its left and right values, and reading them before touching anything is the first move.

### Debug and release

`cargo test` builds the debug profile, where an integer overflow panics, and a release build wraps it, so a sum in copper can pass every test and wrap in the shipped binary. Steel is immune, since its lints forbid the operators and every sum there is a checked call that answers an error. Copper takes the same shape wherever a number came from outside the program, a checked operation rather than an operator, and `cargo test --release` runs the suite once as the shipped binary would when arithmetic there has changed.

### Platforms

Fuji runs on three operating systems and is developed on five machines, and a test may be true on one and meaningless on another. A test of platform code is gated with the same `#[cfg]` as the code, so the Mac's launch services are tested on the Mac, the registry on Windows, and each has a counterpart proving the other platforms answer that the facility is not there. A crate without Tauri in it can be type-checked for Windows from the Mac, tests included, with `cargo check --tests --target x86_64-pc-windows-msvc`, which catches the compile errors a Windows-only test would otherwise meet on its first visit there; running it is still the Windows box's job. The check runs every dependency's build script for the target too, and today blake3's, beneath ergo in both crates, stops it: for the Windows target it assembles its x86-64 code with Windows' own ml64.exe, which no Mac has. Until blake3 is built another way, its pure or prefer_intrinsics feature, a Windows-only test is first compiled on the Windows box.

### When a test is wrong

A test that fails against correct code is wrong about the world, and the fix is to the test, with the lesson in its comment. A sample that was expected to need a second read turned out to fit in the first; the test now says so and builds a file that does not. The code is never bent to make a test pass, and a test is never deleted to make a run green: each is reread, and the one that misstates reality is corrected to state it.

The reverse case is what every test above is written for. A test that fails against changed code and names the behavior that changed has done its job, and the question is whether the change was meant. If it was, the test moves with the behavior, in the same commit, and its name and comment say the new truth.

### The vanity test

The failure to watch for in a growing suite is the test that proves the call ran. The display has at least 640 pixels, the machine has at least a gigabyte, the process started within the hour. Each executes the code, each would pass against almost anything, and together they raise the coverage number while telling nothing a user would feel. Three things keep such a test from misleading: it is called a smoke test in its comment, so nobody mistakes it for behavior; it checks the one fact that would differ if the call were wrong, the pid is this process's own rather than some pid; or it is deleted. A suite is measured by the bugs it would catch, and a test that catches none is a cost in reading time and in the false confidence a long green list gives.

## What this makes easy

**Writing a new table touches one line of the shell** — the entry that says which component that table is. The new table imports the model and the cache like every other table, and `DiamondTable.vue` and `Sheet.vue` do not change, do not learn it exists, and cannot be broken by it. If a new table ever requires editing an old one, the model layer has leaked and that is the bug to fix, not the table.

**Growing the sheet into a file manager** — a tree, a path box, a toolbar of sorts — is work inside `Sheet.vue` plus values in the model. No table changes.

## Rules that hold it together

- **Views never import each other.** The only thing they share is the layer beneath them.
- **Imports point one way.** No chain of imports, however long, leads back to the module it started from, so every module has finished running before anything that imports it starts, and startup is one explicit sequence in the shell rather than an order modules happen to load in. The build enforces it: `vite.config.js` fails on a loop among fuji's own files.
- **The cache never learns about folders, order, or views.** A path in, pixels out.
- **A hidden view measures nothing.** `v-show` is `display: none`, which destroys the layout box, so `clientWidth` reads 0. A view that measures its container does it when it becomes active, never at mount.
- **Window events have one listener, in the shell**, and go to the active view. A part of a view that needs a key or a click elsewhere to itself gets it by holding the focus on its own element, never by a second window listener.
- **A part of a view is a component when it has a life of its own**, and plain markup when it does not. Neither one long file nor a component for every button: the test is whether the part has state, data or timing that the rest of the view would otherwise have to carry.
- **Nothing large goes in a reactive proxy.** The quiver bypasses Vue on purpose, and a decoded bitmap inside `reactive()` is the same mistake with more zeros.

## What is built today

The shell and its views are real: the sheet, the settings panel, the diamond table, and the preview, which is short on purpose, with no flip, no pan and no zoom. The audio panel is real as a view and reaches the media library, which answers its version; it plays nothing yet. The comic table is a stub. Beneath them the model, the settings file, the cache and its flip policy, the log, and the operating system's thumbnailer are real too. `CLAUDE.md` lists the files and what each holds, and changes with them; this section changes only when a layer or a kind of view does.

The model holds the folder, the sort, the ordered list and the current path. Back is planned and not written, and no view has a use for it yet.

`AlphabetSort` is one of the planned sorts, and it is javascript's own `sort()` kept deliberately. There is one flow, `TestFlow`, named by `Bucket.vue` rather than chosen from a register: `TagFlow` and `CanvasFlow` were the two halves of an experiment, and both were deleted once it had answered. The thumbnail pipeline document on the site is what the surviving flow does.

There is no router and no store library. `App.vue` picks the window's root once, as the page mounts, from the note the window was made with and the files it was opened with: the shell for a browsing window, and `VideoWindow.vue` for a video window, asked for with the note Video or opened on a video, which shares none of a browsing window's state and stands beside one rather than taking its place. `main.js` titles the page and mounts the app, and does nothing else.

Plastic is `desktop/src`. Steel is `steel`, holding the tip scan, the walker, the reads that open a file and hand its first bytes to the tip scan, the disk's listing, stating, reading and writing, the walk in find's order, and the fits. Copper is `copper`, holding the media library's declarations with the questions asked through it, the inventory's loop, the sandbox process's start and walls, the thumbnailers, the registry, launch services, the display's size, the memory counters and the process's start. Brass is `desktop/src-tauri`, holding the commands, the windows, the menus, the dock, the trackpad, the dialogs, the log, the players and their render contexts, and the one read only a window can make, the web engine's process ids.

## The cache, in one paragraph

`cache.js` is a store that holds a blob, one object url over it, and a decoded `<img>` per path, or a canvas painted from the operating system's imaging library for a picture only that library reads here, and does nothing clever: views `need` and `release` by name, and nothing is ever freed except on command. `flipCache.js` is the diamond table's policy over it, holding a window of `flip.back` and `flip.forward` images around the one on screen. The table shows **the store's own element**, adopted into its card — pointing an element of its own at the same picture costs the whole decode again, which is a mistake fuji made once and measured.
