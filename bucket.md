# Buckets

A bucket is a box of thumbnails, and the sheet's one scroll runs over a stack of them rather than over the pictures themselves. This document records what a bucket is, what it is for, and where it is meant to go. It began as a temporary cheat, a box no file manager shows, put there because it made the sheet's hard problems buildable and measurable before they were solved. It has turned out to be the unit that would let fuji look at a whole drive in constant memory, and the second half of this file is that vision, written down so it is decided here rather than rediscovered.

`structure.md` names the parts and `architecture.md` says where each one lives. This says what the box between them is. The thumbnail pipeline document on the site says how the thumbnails inside it are made, `canvas.md` has the engine facts behind that, and `performance.md` is where a number goes once fuji has seen it happen.

## What everybody expects

**One scroll, top to bottom, over a wrapped flow of thumbnails.** The user scrolls to the top to see the first row, drags the scrollbar to the bottom to see the last, and there is no structure of any kind between the scrollbar and the pictures. That is Finder in icon view, Explorer in extra large icons, and every photo grid on the web. It is what somebody opening fuji's contact sheet expects to find, and it is the thing to hold in mind while reading the rest of this: the traditional app, doing the traditional thing.

## The bucket

**The sheet is one scroll over a stack of full-width buckets.** There is one contact sheet and it has one big top-to-bottom scroll, exactly as expected. What that scroll moves the viewport over is not individual thumbnails; it is buckets, each as wide as the sheet and as tall as its contents need, and the thumbnails are inside them.

**A bucket holds up to a maximum number of images, and the maximum is a setting.** `bucket.images` in `fuji.toml`, 20 at the factory. A folder of 15 images is one bucket with 15 thumbnails in it. A folder of 50 images is a bucket of 20, a second bucket of 20 beneath it, and a third holding the remaining 10.

**A bucket never mixes folders.** Images from two different folders never share a bucket, however few of them there are.

**So a bucket boundary falls in exactly two places:**

    the bucket reached its maximum image count
    a new folder began

## Why divide the scroll

**Because a bucket is a unit fuji can count, and loose thumbnails are not.** The purpose of putting a box here is to be able to limit how many of them are in the sheet at once. The sheet is meant to show some number of buckets — ten, as an example — and that number times the maximum images per bucket is a ceiling on how much the sheet is ever holding.

**Both numbers are meant to be the user's.** How many thumbnails fill a bucket is already a setting; how many buckets fill the sheet is the setting that goes beside it. Together they are the user's own control over what fuji costs to run. A raspberry pi and a machine with an rtx should not be holding the same number of thumbnails, and with both numbers in `fuji.toml` fuji never has to guess which one it is on.

**And the same two numbers are what make a whole drive walkable.** That is the next section.

## The walk: a whole drive in constant memory

**Every image on the drive as one long sorted list.** Picture the output of `find` run at the top of a drive: one list of every path, weaving in and out of folders all the way down, each folder's contents sorted. Keep only the images, and that list is what the sheet walks through, folder to folder, without end. Nobody ever builds the whole list; it is the order the walk follows.

**A page is one group of buckets, and the sheet shows one page at a time.** Below the last bucket on a page is a button for the next page, which discards every bucket on this one and fills the next set. Nothing accumulates: the memory a page costs is the same at the millionth image as at the first, because the buckets behind the user are gone.

**The two rules that cut a bucket are what carry the walk across folders.** A bucket holds at most `bucket.images` images and never holds images from two folders, so a new bucket starts when the last one is full or when a new folder begins. When a folder runs out inside a page that still has a bucket to show, the next folder begins in that next bucket, with no special case. A worked example: folder a holds 15 images, folder b holds 15, a bucket holds 10, and a page holds 3 buckets.

    page 1   bucket 1   a, images 1 to 10
             bucket 2   a, images 11 to 15, the 5 left over
             bucket 3   b, images 1 to 10
    page 2   bucket 1   b, images 11 to 15
             ...        and on into whatever folder comes after b

**It has seams, and they are accepted.** A short bucket where a folder runs out, and a button between pages where a scroll might have gone on. That is not the single uninterrupted flow a contact sheet should be, and the cost is taken deliberately, because it is what turns a drive of millions of images from something fuji can open one folder of into something fuji can traverse.

**What a page costs is a number, now.** Every raster thumbnail is a canvas fuji sized, whichever path made its pixels, and `TestFlow` totals their bytes per bucket. So a page costs its buckets' totals plus the imgs, which are the engine's and small, and the ceiling the two settings put on the sheet is a real bound rather than a count of things whose size fuji cannot see.

**Only the first page is built.** Today the sheet shows `sheet.buckets` buckets of `bucket.images` each from the start of one folder, three of twenty at the factory, and the rest of a larger folder is not shown; there is no second page and no button. What is open about the walk:

- **The order.** Whether a folder's own images come before its subfolders' or after, how folders are sorted against each other, and whether the sort the user chose for images applies to folders too.
- **Where a walk starts and how far it reaches.** A drive, a folder the user chooses, or the folder they opened; and whether the walk ever climbs out of the folder it started in.
- **What it skips.** Hidden folders, the system's own folders, application bundles and photo libraries that are folders on disk but one thing to a user, links that would loop forever, and folders fuji is not allowed to read.
- **Listing only as far as it needs.** The next page needs only the next few folders, so the walk should list a folder when it reaches it rather than walk the drive first, and keep where it is as a single position, the last path shown, rather than the list.
- **Going back,** which is the same walk pointed the other way, and whether the previous page has to come out exactly as it was.
- **Folders with no images,** which the walk passes through without showing anything; a long run of them on a big drive is a wait with nothing on screen.
- **What the button says,** and whether a bucket's caption, its folder and which of its images it holds, is enough for a user to know where they are.

## The experiment the bucket ran, and what it found

**Two flows in one slot asked whether fuji should make its own thumbnails at all.** `TagFlow` handed the engine plain img tags and let it decide everything; `CanvasFlow` painted each picture down into a canvas fuji sized. The bucket was what made them comparable: the same folder, the same scroll, one variable. `canvas.md` records the answers, measured. The engine's img thumbnails are smaller than a full decode on both platforms, but they are the engine's to keep or drop, and up to twenty megabytes each on the Mac; a canvas is a known number of bytes fuji owns and the engine cannot take back. A one-pass draw into a canvas came out rough on the Mac, and halving fixed it at a cost to the main thread. And the operating system makes the same thumbnail three to twenty-five times faster and costs the main thread nothing.

**So the answer is neither flow.** `TestFlow` replaces both: every raster thumbnail is a canvas, its pixels from the operating system where the platform's list allows and from the page where it does not, and a GIF or an SVG is an img. The thumbnail pipeline document on the site is what it does and why. The confound the experiment worried over, a canvas bucket raising the memory pressure the engine handles the other bucket's decodes under, is moot once every raster thumbnail is a canvas.

**What fuji can measure honestly is frames, and it is the number to keep taking.** `log.js` and the frame-time learning in `DiamondTable.vue` establish the technique: record what something cost in milliseconds and in frames, and refuse to claim a frame it only spilled into. A scroll is a stream of frames, and a bucket that hitches is a bucket that drops them. That is visible from inside the app, needs no view into the engine's memory, and is the closest number to what a person feels. Rows per thumbnail and per bucket are written now, and nobody has read them yet; `thumbnail-open.md` keeps that as the first test to run.

## What a canvas thumbnail costs

**A canvas is memory the engine can never take back.** An `img`'s decode can be dropped under pressure and rebuilt from the bytes; a canvas has nothing to rebuild from. So a canvas thumbnail costs a fixed, known amount for as long as it exists — its css width times its css height, times `devicePixelRatio` squared, times four bytes — and a ceiling on buckets is what turns that from a slower way to run out of memory into a bound. At a `devicePixelRatio` of 2, for a 3:2 photograph:

    size          one thumbnail    500 of them
    Small  120    150 KB            77 MB
    Medium 240    600 KB           307 MB
    Large  360    1.35 MB          691 MB
    Xl     480    2.4 MB           1.23 GB

At Medium, five hundred thumbnails cost about what three full-size 26-megapixel decodes do. That is the win. At Xl, a folder of two thousand is five gigabytes nobody can reclaim, which is why the count of buckets has to exist before the walk does.

**Three things an img gets from the engine for nothing, a canvas does by hand.** The engine's `devicePixelRatio`, which decides whether a thumbnail is sharp on a retina panel or on Windows at 150%, and which goes stale the moment the window moves to a monitor with a different one. A change of thumbnail size, which css answers over pixels the engine already has, while every canvas has to be made again. And the color space: a canvas is sRGB unless asked otherwise, so every canvas is made in the screen's own gamut, which on an ordinary sRGB screen is the canvas it always was. `canvas.md` has what each platform does with each of these.

**The engines are not the same engine, so a measurement on one is not a measurement on the other.** Both decode a large picture small when they know it will be painted small, Chromium at a half, a quarter or an eighth, WebKit by a five-megapixel rule, and both give an accelerated canvas its own compositing layer, on the Mac by a display-list mode that is on by default and on Windows always. `canvas.md` has the rules and the sources.

## What is built

**A bucket and one flow, on a sheet that scrolls.** `Bucket.vue` takes a list of images and decides nothing else; `TestFlow.vue` is the flow, and `TagFlow.vue` and `CanvasFlow.vue` were deleted once the experiment below had answered. In `fuji.toml`, `bucket.images` caps a bucket at 20 and `sheet.buckets` the sheet at 3, so the sheet never holds more than 60 thumbnails, and `thumbnail.beam` picks the length every thumbnail is measured against — 120, 240, 360 and 480 css pixels, so the user says once what Medium means; `fits.md` plans the ways a thumbnail can be measured against it. There is no setting naming the flow, because there is one flow; the name is in `Bucket.vue` and a second one brings a register back with it.

**A flow acts within one bucket, and one flow governs the whole sheet.** That was the open question when this file was first written and it is settled: every bucket makes the same flow, so a second one would switch all of them together.

**The sheet cuts a folder into buckets, and stops at its count.** `Sheet.vue` slices the folder into buckets of `bucket.images` and keeps the first `sheet.buckets` of them, all mounted at once; each bucket's tiles join the governors' lines as it mounts, so the first bucket fills first. Pages and Next are the walk above, and are not built.

**A bucket fills as its pictures arrive.** `TestFlow` reads nothing ahead: a tile takes its room when its pixels land, so the first thumbnails show at once and the rows settle as the rest fill in. Holding every box from the start would need every picture's size before any thumbnail is made, which is the look-ahead `thumbnail-open.md` scopes as a later system. A bucket goes on filling while the sheet is hidden behind the table, and its reads share the disk governor's line with the table's own.

## Open

- **Named bucket, 2026-10-06, and called a card until then.** `DiamondTable.vue` calls its one image container the card — `cardRef`, `cardShow()`, `.myCard`, and `quiverB.card1` and `card2` — and that is a different thing at a different altitude, so the user renamed this one. Bucket because it is a container sized for resources rather than a design element, and it sounds as under construction as it is. Settings, log rows, components and documents all say bucket now; the table keeps card.
- **The second setting.** Built as `sheet.buckets`, counted in buckets. Still open: whether a page should one day be measured in bytes instead, now that a bucket's bytes are a number the sheet can read.
- **What a bucket is to the user** — whether it is visible at all, whether it is bordered, headed, or named with the folder it came from, and where Next sits. A first answer is built: a line of quiet text in each bucket's upper right, giving the folder and which of its images the bucket holds, like Images 1 through 20 of 35.
- **Whether the bucket stays.** It began as scaffolding, it is the unit of the walk above, and the walk is the argument for keeping it. Nothing else here depends on the answer.
