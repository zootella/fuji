# Cards

A card is a box of thumbnails, and the sheet's one scroll runs over a stack of them rather than over the pictures themselves. This document records what a card is, what it is for, and where it is meant to go. It is honest from the first line about how it began: as a temporary cheat, a box no file manager shows, put there because it made the sheet's hard problems buildable and measurable before they were solved. It is turning out to be something more — the unit that would let fuji look at a whole drive in constant memory — and the second half of this file is that vision, written down so it is decided here rather than rediscovered.

`structure.md` names the parts and `architecture.md` says where each one lives. This says what the box between them is. `cache.md` is the store beneath both flows, and `performance.md` is where any number claimed here has to come from before it is believed.

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

**What a page costs is the flow's price per thumbnail, times the two settings.** With CanvasFlow a thumbnail is a known number of bytes and the ceiling is a real bound. With TagFlow a thumbnail costs whatever the engine decides, and the ceiling is a count of things whose size fuji cannot see. That difference is the reason the experiment below exists, and it is why the ceiling is not simply a number of cards — see the open list.

**None of it is built.** Today every card the folder needs is rendered, there is one folder, and there is no page and no Next.

## The card as an experiment

**A card is also a slot a strategy plugs into.** `TagFlow` renders its thumbnails with nearly no JavaScript at all — plain `img` tags pointed at the pictures, letting the rendering engine decide everything about decoding, scaling, and what to keep. `CanvasFlow` does fuji's own work instead, painting each image down into a canvas to control the memory itself, which is the idea `Thumbnail.vue` recorded years before there was a sheet to put it in. `card.flow` names which one, and one flow governs every card at once — so today a comparison is two runs rather than two cards in one scroll, and a flow chosen per card is the small change that would make it one.

**The format wanted is a paired comparison, which is the strongest kind available.** Same folder, same display, same frame budget, same scroll gesture, same moment — everything held constant except the one thing being asked about. Two separate runs can never be that clean, because the machine, the folder, and the hand all drift between them.

**And the axis it exposes is the one the question actually lives on: distance from the viewport.** Whether the engine is doing better work than fuji could is not really a question about drawing a thumbnail. It is a question about a thumbnail that has been scrolled a thousand pixels off the bottom — whether its pixels are still there, whether the engine quietly dropped them, and what coming back to it costs. A card is a large enough unit to be genuinely far outside the viewport while its neighbour is at the edge, and nothing smaller would let that be seen.

**The trap is that fuji is blind to the strategy most likely to win.** `cache.js` counts blob bytes and estimated pixel bytes, and `cache.md` states the limit plainly: an image handed to the page and decoded by the browser is memory fuji can neither measure nor free. So the canvas card is fully accountable and the plain-`img` card is largely invisible, and a footprint readout comparing the two would show the canvas card holding a great deal and the other holding almost nothing. That would be an artifact of what fuji can see rather than a result. The number that settles it is process memory — what the operating system says the app is using — which lives outside the app entirely.

**The first confound is that the two cards are not independent.** The engine keeps one image budget for the whole process, not one per card, and a canvas backing store is memory it can never reclaim, because there is nothing to rebuild it from. So a canvas card in the scroll raises the pressure under which the engine decides what to do with the other card's decodes. That interference may be the most interesting result available — whether one strategy starves the other — but it means side by side cannot be the only arrangement. Each alone says what it costs; together says how they interact.

**The second confound, found in the audit of 2026-09-08 and taken out the same day, was that TagFlow was not a control.** The store decoded at full size on every load, for whoever asked, so a TagFlow card forced two hundred full decodes before the engine had been allowed to decide anything, and the arm meant to measure the engine deciding for itself measured it with the expensive work already done. Now a need says which steps it wants: TagFlow asks for the bytes and the url only, and the engine's decode of the `img` is the only decode there is. `cache.md` has the command.

**What fuji can already measure honestly is frames, and that may be enough to decide it.** `meter.js` and the frame-time learning in `DiamondTable.vue` establish the technique: record what something cost in milliseconds and in frames, and refuse to claim a frame it only spilled into. A scroll is a stream of frames, and a card that hitches is a card that drops them. That is visible from inside the app, needs no view into the engine's memory, and is the closest number to what a person feels anyway.

**The question underneath all of it is whether fuji should be writing this code at all.** A wall of images that scrolls is the most ordinary thing on the web, and the engines have been tuned against it for twenty years. The experiment exists to find out where that competence ends — whether fuji's own code is unnecessary, necessary, or necessary only past some size — rather than to assume any of the three.

## What the two flows cost

**A canvas is memory the engine can never take back.** An `img`'s decode can be dropped under pressure and rebuilt from the bytes; a canvas has nothing to rebuild from. So a canvas thumbnail costs a fixed, known amount for as long as it exists — its css width times its css height, times the pixel ratio squared, times four bytes — and a ceiling on cards is what turns that from a slower way to run out of memory into a bound. At a pixel ratio of 2, for a 3:2 photograph:

    size          one thumbnail    500 of them
    Small  120    150 KB            77 MB
    Medium 240    600 KB           307 MB
    Large  360    1.35 MB          691 MB
    Xl     480    2.4 MB           1.23 GB

At Medium, five hundred thumbnails cost about what three full-size 26-megapixel decodes do. That is the win. At Xl, a folder of two thousand is five gigabytes nobody can reclaim, which is why the count of cards has to exist before the walk does.

**Filling a canvas card holds one full decode at a time — per card.** CanvasFlow reads, decodes, paints, and releases one image at a time, so the store holds one full-size decode for each card that is filling. Every card fills at once, so a folder of a thousand is five decodes in flight rather than one. The draw itself is synchronous work on the main thread, tens of milliseconds for a large photograph, more than a frame — and today it runs whether or not the sheet is showing. After the sheet has been shown once, every drop on the table fills the sheet behind it, inside the very flips the table is protecting. `Sheet.vue` carries the note, and the fix is a visible gate the flow loop awaits.

**Three things TagFlow gets from the engine for nothing, CanvasFlow does by hand and can get wrong.** The css-to-backing pixel ratio, which decides whether a thumbnail is sharp on a retina panel or on Windows at 150%, and which goes stale the moment the window moves to a monitor with a different one. A change of thumbnail size, which TagFlow answers with two lines of css over pixels it already has, while every canvas has to be painted again from an image that was released. And the color space: a canvas is sRGB unless asked otherwise, so a Display P3 photograph — every iPhone's, since the 7 — loses its most saturated colors on the way in, and looks duller in the sheet than on the table on a Mac's wide-gamut panel. One argument to `getContext` fixes the third, and it is passed: every canvas is made in the screen's own gamut, which on an ordinary sRGB screen is the canvas it always was. `canvas.md` has what each platform does with it.

**What the engines do with a canvas, read from their sources on 2026-09-08.** WebKit on the Mac backs every 2d canvas with an IOSurface, at any size. Whether each one is also a compositing layer of its own turns on a display-list rendering mode that is on by default wherever it is compiled in, so the likely answer is yes, for every thumbnail size — and the Layers tab of Safari's Web Inspector, pointed at a running debug build, settles it in a minute. Chromium on Windows accelerates a canvas of 128 by 129 pixels or more — every size at 150%, and every size but Small at 100% — and gives every accelerated canvas a compositor layer of its own. On both engines `willReadFrequently: true` forces a plain CPU-backed canvas: no surface, no layer, memory on the ordinary heap, and a software resample in `drawImage`. That one attribute is a third arm of the experiment, and costs nothing to try.

**And the two engines are not the same engine, so the answer may differ.** Both decode a large picture small when they know it will be painted small — Chromium at a half, a quarter or an eighth, WebKit by a five-megapixel rule that `canvas.md` has — so a TagFlow thumbnail is far smaller than a full decode on either platform, as long as nothing decoded the file at full size first. That is what TagFlow's opt-out of the store's decode protects: a full frame already in hand counts as good enough for any smaller request, and the store used to make one for every load. Where the engines part is everything else — what each keeps under pressure, what its `createImageBitmap` will resize, and what a canvas draw does with the raster it is given. The paired comparison may still say canvas on one platform and `img` on the other, and that would be a result rather than a contradiction. What to do about it is in the open list.

## What is built

**A card and both flows, on a sheet that scrolls.** `Card.vue` takes a list of images and a flow name and decides nothing else. `TagFlow.vue` is the control: plain img tags, wrapped like words, everything else left to the engine. `CanvasFlow.vue` is the alternative: one image at a time, each painted into a canvas at the backing resolution of the display and then released, so the store holds nothing once a card is drawn. In `fuji.toml`, `card.images` is the cap at 200, `card.flow` names which flow the sheet uses, and `thumbnail.size` picks one of four squares a thumbnail fits inside — 120, 240, 360 and 480 css pixels, shared by every flow so the user says once what Medium means.

**A flow acts within one card, and one flow governs the whole sheet.** That was the open question when this file was first written and it is settled: the sheet holds the name and hands the same one to every card, so switching switches all of them together.

**Every card the folder needs is rendered.** The count of cards, the page, and Next are the walk above, and none of it is built.

**Both flows reflow while they fill.** A thumbnail has no size until its image has been read, so each arrival re-wraps the flow and moves everything after it — TagFlow's in the order the loads finish, CanvasFlow's in the model's order. Once painted, a thumbnail never changes size, and a wrap on a window resize is layout alone, with the pixels reused. Neither flow is doing anything wrong, and neither can be fixed without knowing an aspect ratio before its decode. It is a flow decision waiting to be made, not a renderer being told the wrong thing.

## Open

- **The word `card` is already taken.** `DiamondTable.vue` calls its one image container the card — `cardRef`, `cardShow()`, `.myCard`, and `quiverB.card1` and `card2` are the arrows that place and size it. That is a different thing at a different altitude, and `structure.md` is where the vocabulary gets settled.
- **The second setting.** How many cards fill a page, what it is called, and whether a page is measured in cards or in images. The first is already `card.images`.
- **What a card is to the user** — whether it is visible at all, whether it is bordered, headed, or named with the folder it came from, and where Next sits.
- **What the ceiling counts.** Ten cards times two hundred images bounds what the sheet holds only while every card is built the same way. A canvas card costs a known number of bytes per thumbnail; a tag card costs whatever the engine decides. The unit is settled for one flow and not the other.
- **Where a thumbnail is made is decided, and the routing is not built.** The operating system's thumbnailer wherever it decodes the format, on Windows and the Mac, and the page for the rest and for Linux, decided on 2026-09-08 from the measurements in `canvas.md`, on responsiveness before speed. What remains is the probe and the table that route a file, which `canvas.md` designs and nothing implements, and `security.md`'s walls, which come first.
- **Whether the card stays.** It began as scaffolding, it is the unit of the walk above, and the walk is the argument for keeping it. Nothing else here depends on the answer.
- **A flow chosen per card**, for the paired comparison the experiment wants and two runs cannot give.
