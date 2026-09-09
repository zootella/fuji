# Cards

A card is a box of thumbnails, and the sheet's one scroll runs over a stack of them rather than over the pictures themselves. This document records what a card is, what it is for, and where it is meant to go. It began as a temporary cheat, a box no file manager shows, put there because it made the sheet's hard problems buildable and measurable before they were solved. It has turned out to be the unit that would let fuji look at a whole drive in constant memory, and the second half of this file is that vision, written down so it is decided here rather than rediscovered.

`structure.md` names the parts and `architecture.md` says where each one lives. This says what the box between them is. `thumbnail-plan.md` says how the thumbnails inside it are made, `canvas.md` has the measurements behind that, and `performance.md` is where a number goes once fuji has seen it happen.

## What everybody expects

**One scroll, top to bottom, over a wrapped flow of thumbnails.** The user scrolls to the top to see the first row, drags the scrollbar to the bottom to see the last, and there is no structure of any kind between the scrollbar and the pictures. That is Finder in icon view, Explorer in extra large icons, and every photo grid on the web. It is what somebody opening fuji's contact sheet expects to find, and it is the thing to hold in mind while reading the rest of this: the traditional app, doing the traditional thing.

## The card

**The sheet is one scroll over a stack of full-width cards.** There is one contact sheet and it has one big top-to-bottom scroll, exactly as expected. What that scroll moves the viewport over is not individual thumbnails; it is cards, each as wide as the sheet and as tall as its contents need, and the thumbnails are inside them.

**A card holds up to a maximum number of images, and the maximum is a setting.** `card.images` in `fuji.toml`, 200 at the factory. A folder of 80 images is one card with 80 thumbnails in it. A folder of 220 images is a card of 200 and, beneath it, a second card holding the remaining 20.

**A card never mixes folders.** Images from two different folders never share a card, however few of them there are.

**So a card boundary falls in exactly two places:**

    the card reached its maximum image count
    a new folder began

## Why divide the scroll

**Because a card is a unit fuji can count, and loose thumbnails are not.** The purpose of putting a box here is to be able to limit how many of them are in the sheet at once. The sheet is meant to show some number of cards — ten, as an example — and that number times the maximum images per card is a ceiling on how much the sheet is ever holding.

**Both numbers are meant to be the user's.** How many thumbnails fill a card is already a setting; how many cards fill the sheet is the setting that goes beside it. Together they are the user's own control over what fuji costs to run. A raspberry pi and a machine with an rtx should not be holding the same number of thumbnails, and with both numbers in `fuji.toml` fuji never has to guess which one it is on.

**And the same two numbers are what make a whole drive walkable.** That is the next section.

## The walk: a whole drive in constant memory

**A page is one group of cards, and the sheet shows one page at a time.** Below the last card on a page is a large button, Next. Pressing it discards every card on the page and loads the next group. Nothing accumulates: the memory a page costs is the same at the millionth image as at the first, because the cards behind the user are gone.

**The pages run through every image on the drive.** A large folder is split across several cards, exactly as it is today. When a folder runs out inside a page that still has a card to show, the next folder begins in that next card. Because a folder boundary is already a card boundary, the crossing needs no special case: the rule that keeps two folders out of one card is the same rule that lets the walk continue into the next one. Going back is the same mechanism pointed the other way.

**It has seams, and they are accepted.** A seam between cards, and a larger one between pages, where the user presses a button rather than scrolling on. That is not the single uninterrupted flow a contact sheet should be, and the cost is taken deliberately, because it is what turns a drive of millions of images from something fuji can open one folder of into something fuji can traverse.

**What a page costs is a number, now.** Every raster thumbnail is a canvas fuji sized, whichever path made its pixels, and `SquareFlow` totals their bytes per card. So a page costs its cards' totals plus the imgs, which are the engine's and small, and the ceiling the two settings put on the sheet is a real bound rather than a count of things whose size fuji cannot see.

**None of it is built.** Today every card the folder needs is rendered, there is one folder, and there is no page and no Next.

## The experiment the card ran, and what it found

**Two flows in one slot asked whether fuji should make its own thumbnails at all.** `TagFlow` handed the engine plain img tags and let it decide everything; `CanvasFlow` painted each picture down into a canvas fuji sized. The card was what made them comparable: the same folder, the same scroll, one variable. `canvas.md` records the answers, measured. The engine's img thumbnails are smaller than a full decode on both platforms, but they are the engine's to keep or drop, and up to twenty megabytes each on the Mac; a canvas is a known number of bytes fuji owns and the engine cannot take back. A one-pass draw into a canvas came out rough on the Mac, and halving fixed it at a cost to the main thread. And the operating system makes the same thumbnail three to twenty-five times faster and costs the main thread nothing.

**So the answer is neither flow.** `SquareFlow` replaces both: every raster thumbnail is a canvas, its pixels from the operating system where the platform's list allows and from the page where it does not, and a GIF or an SVG is an img. `thumbnail-plan.md` is its plan. The confound the experiment worried over, a canvas card raising the memory pressure the engine handles the other card's decodes under, is moot once every raster thumbnail is a canvas.

**What fuji can measure honestly is frames, and it is the number to keep taking.** `log.js` and the frame-time learning in `DiamondTable.vue` establish the technique: record what something cost in milliseconds and in frames, and refuse to claim a frame it only spilled into. A scroll is a stream of frames, and a card that hitches is a card that drops them. That is visible from inside the app, needs no view into the engine's memory, and is the closest number to what a person feels. Rows per thumbnail are `thumbnail-plan.md`'s fifth step.

## What a canvas thumbnail costs

**A canvas is memory the engine can never take back.** An `img`'s decode can be dropped under pressure and rebuilt from the bytes; a canvas has nothing to rebuild from. So a canvas thumbnail costs a fixed, known amount for as long as it exists — its css width times its css height, times the pixel ratio squared, times four bytes — and a ceiling on cards is what turns that from a slower way to run out of memory into a bound. At a pixel ratio of 2, for a 3:2 photograph:

    size          one thumbnail    500 of them
    Small  120    150 KB            77 MB
    Medium 240    600 KB           307 MB
    Large  360    1.35 MB          691 MB
    Xl     480    2.4 MB           1.23 GB

At Medium, five hundred thumbnails cost about what three full-size 26-megapixel decodes do. That is the win. At Xl, a folder of two thousand is five gigabytes nobody can reclaim, which is why the count of cards has to exist before the walk does.

**Three things an img gets from the engine for nothing, a canvas does by hand.** The css-to-backing pixel ratio, which decides whether a thumbnail is sharp on a retina panel or on Windows at 150%, and which goes stale the moment the window moves to a monitor with a different one. A change of thumbnail size, which css answers over pixels the engine already has, while every canvas has to be made again. And the color space: a canvas is sRGB unless asked otherwise, so every canvas is made in the screen's own gamut, which on an ordinary sRGB screen is the canvas it always was. `canvas.md` has what each platform does with each of these.

**The engines are not the same engine, so a measurement on one is not a measurement on the other.** Both decode a large picture small when they know it will be painted small, Chromium at a half, a quarter or an eighth, WebKit by a five-megapixel rule, and both give an accelerated canvas its own compositing layer, on the Mac by a display-list mode that is on by default and on Windows always. `canvas.md` has the rules and the sources.

## What is built

**A card and three flows, on a sheet that scrolls.** `Card.vue` takes a list of images and a flow name and decides nothing else. `SquareFlow.vue` is the flow going forward; `TagFlow.vue` and `CanvasFlow.vue` sit beside it in the register for one comparison on one folder, and then retire. In `fuji.toml`, `card.images` is the cap at 200, `card.flow` names which flow the sheet uses, and `thumbnail.size` picks one of four squares a thumbnail fits inside — 120, 240, 360 and 480 css pixels, shared by every flow so the user says once what Medium means.

**A flow acts within one card, and one flow governs the whole sheet.** That was the open question when this file was first written and it is settled: the sheet holds the name and hands the same one to every card, so switching switches all of them together.

**Every card the folder needs is rendered.** The count of cards, the page, and Next are the walk above, and none of it is built.

**A card lays its boxes out before its pictures arrive.** `SquareFlow` probes a card's files in one call and knows every size from the header alone, on the Mac and Windows, so thumbnails land in boxes already there and nothing shifts as a card fills. On Linux, and for an SVG, a box appears with its picture. The two retiring flows reflow on every arrival, and a hidden sheet fills nothing under `SquareFlow`, where the other two keep working behind the table.

## Open

- **The word `card` is already taken.** `DiamondTable.vue` calls its one image container the card — `cardRef`, `cardShow()`, `.myCard`, and `quiverB.card1` and `card2` are the arrows that place and size it. That is a different thing at a different altitude, and `structure.md` is where the vocabulary gets settled.
- **The second setting.** How many cards fill a page, what it is called, and whether a page is measured in cards or in bytes, now that a card's bytes are a number the sheet can read. The first is already `card.images`.
- **What a card is to the user** — whether it is visible at all, whether it is bordered, headed, or named with the folder it came from, and where Next sits.
- **Whether the card stays.** It began as scaffolding, it is the unit of the walk above, and the walk is the argument for keeping it. Nothing else here depends on the answer.
