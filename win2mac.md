# From the Windows box to the Mac: the R key, and the Ben Day page

Written 2026-10-08 on the Windows 10 box, at 1920 by 1200 and 100 percent, where CSS, backing and panel pixels are one grid. This is a letter rather than a document of record: read it, do what it asks, then delete it and leave what you found in the code and in `site/docs/ben-day.md`. The feature is built in its first form and smoke tested here, and the rest of it needs a Retina Mac.

## What landed here

**The `R` key on the diamond table.** `src/raster.js` is a module shaped like `gamma.js`: one ref, `raster`, false at every launch and written nowhere, and `rasterToggle()`. `DiamondTable.vue` answers bare `r` beside `i`, binds `myRaster` on the card while the ref is true, and has one new scoped rule, `.myCard.myRaster :deep(.myImage) { image-rendering: pixelated }`. The existing image rule is untouched, so off is exactly what the table drew before. The hud carries the lenses on one line, `Gamma OFF · Smooth` or `Gamma 1.22 · Rasterized`, with the zoom in CSS and backing pixels per picture pixel above it and the caption's two lines below. The help panel has an `R` line. Nothing on the sheet imports the ref and no selector outside the card can match the rule, which is the whole of the promise that the thumbnail pipeline cannot be affected.

**Smoke tested here.** A GIF on the table at `2`, then `r`: blocks at once, and back to smooth on the second press. That is the whole of what this box has established. At 100 percent every number key is a whole number of backing pixels per image pixel, so this machine cannot say anything about a fractional ratio and nothing about a Retina one.

**The Ben Day page**, `site/docs/ben-day.md`, in Craftsmanship between Gamma and File Types. The history of the dot and of the word moiré, the engineering half written from the source, a section that works the setting through the four pixel units, and a section on the `R` key as built. Every link was fetched from here; Apple's two documentation pages answered only with their titles, since they render by script, so a look at them in a browser there would close that.

## What only a Retina Mac can answer

Each of these names its mechanism, which is the bar for asking another machine to look.

**1. Does WKWebView repaint the card when the class flips?** The gamma lens needed two filters taking turns because WebKit did not redraw an element when a filter it was already showing through changed underneath it; that is in the essay above the gamma watch in `Shell.vue`. A class toggle that changes `image-rendering` is an ordinary style change and should invalidate the picture, but the same engine is involved and the same symptom would look the same: the hud says `Rasterized` and the picture stays smooth until something else forces a draw, like leaving fullscreen. Press `r` on a GIF at `2` in a fullscreen table and watch the picture itself, not the hud. If it does not change, the fix is the gamma lens's: two rules or two classes, so every change is a new value the engine has to rebuild for.

**2. What does `pixelated` give at a ratio of 2?** On a Retina Mac `1` is already two backing pixels per image pixel, a two-by-two block, and `2` is four by four. The expectation is exact blocks at every number key, since the card's size and place are snapped to the backing grid by `xySnap` in `library.js` and every key is a whole number of CSS pixels. The measurement is the one `fidelity.md` describes under *Repeating this*: author a small test GIF or PNG with one-pixel black and white rows, show it at `1` with `r` on, capture the window's own buffer with `screencapture -x -o -l <CGWindowID>`, and count pixels that are neither black nor white over the card. Zero means the blocks reached the backing store. Do it on the Mac mini first, where backing is panel, and the number is also what the lights show.

**3. What the panel does to the blocks on the MacBook Air.** In its "looks like" 1710 by 1112 mode macOS resamples the 3420 by 2224 backing store onto 2560 by 1664 lights, so a clean two-by-two block becomes a one-and-a-half smear and no setting can touch it; the page says so, and it is reasoned from `fidelity.md`'s measurements rather than seen. Seeing it is a look with a loupe or a phone camera, at `1` with `r` on, against the same picture in the "looks like" 1280 by 832 mode, where backing and panel agree. Whichever way it comes out, the page's paragraph *The panel has the last word* states what the lights show as fact, in the present tense, with no mention of who looked or when; `fujis-voice.md` has the rule.

**4. What `pixelated` does at a fractional ratio in WebKit.** Chromium's answer is on the page: plain nearest-neighbor at any ratio, measured here at 2.94 on the one-pixel checkerboard, two colors and nothing blended, squares of three with a square of two every sixteenth. Nudge the wheel one notch from `2` with `r` on and look at the same checkerboard on the Mac: the same hard stutter, or seams softened to gray, which is the reading the specification describes and which WebKit may have. Write WebKit's answer beside Chromium's in the page's *What there is to choose* section, as a fact about the engine.

## Then finish it

The Ben Day page carries only finished sections, by the user's rule, so it names nothing to do; this letter is where the plan lives. With the four answers above in hand, what is left is design, to decide in conversation with the user and then build, and the page's grid section says what each one weighs:

- **One toggle or two.** Whether the reduction case, a scanned page of dots that wants a filter wide enough to average a whole dot, is the other position of `r` or a setting of its own. CSS has no value for it, so it would be a canvas drawn by the page, as the thumbnails are, and it would give up a GIF's animation.
- **Whole in CSS or whole in backing.** Whether a number key keeps meaning CSS pixels per image pixel, and raster accepts the stutter on the fractional Windows scales, or whether raster changes what the keys mean there, which on a Retina Mac would make `1` half the size it is today.
- **What the hud says.** The three ratios, CSS, backing and panel per image pixel, so a user who sees soft edges in a scaled Mac mode knows which grid put them there. `panel.rs` answers for the main display only.
- **Whether it stays a lens.** It is one today, like gamma. It may one day belong to a picture, since the GIF wants it every time and the photograph beside it never does.

Each decision, once built, becomes a finished section on the page, written the way the `R` key's is. The Ben Day page is the document of record; `contents.md` lists it under what has moved to the site.

When this letter has been answered, delete it, and leave the answers in `ben-day.md` and in comments beside the code they describe.
