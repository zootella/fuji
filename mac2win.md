# To Windows: a dark fringe on transparent thumbnails, fixed on the Mac and maybe on Windows too

Written on the Mac mini on 2026-10-23. Read this, then talk with the user before changing anything; a letter starts a conversation rather than settling one.

## What the Mac found, and fixed

**The Mac's thumbnailer let the color hidden under a PNG's clear pixels into its edges.** A PNG keeps a color for every pixel, even a fully clear one, and most programs that save PNGs leave black there. A correct shrink weighs each pixel's color by its opacity first, so a clear pixel's hidden color counts for nothing. ImageIO's thumbnail resample didn't, so the hidden black bled into every outline: a thin dark fringe, olive where it meets yellow. Measured on the Mac mini at the contact sheet's 210 pixels, ImageIO's thumbnail of `firefox.png` had 1,215 near-black partly clear edge pixels, and a correct shrink of the same file has 1. The green-hidden test disc, white on clear pixels that hide pure green, came back with 1,104 green-tinted edge pixels.

**How the user saw it: only with Firefox's logo, as a PNG and an SVG side by side on the contact sheet.** The sheet draws an SVG in the web engine, which shrinks correctly, so the SVG was clean beside it, and the PNG's olive line along the flame's yellow edge stood out against it. Neither picture alone showed it, and the green disc looked clean to the user by eye while carrying those 1,104 tinted pixels. The two pictures, from Wikimedia Commons:

- [Firefox's logo as an SVG](https://commons.wikimedia.org/wiki/File:Firefox_logo,_2019.svg)
- [the same logo as the PNG Wikimedia renders from it, 960 pixels across](https://commons.wikimedia.org/wiki/Special:FilePath/Firefox_logo,_2019.svg?width=800)

**The fix, in the Mac half of `thumbnail.rs`.** A picture with an alpha channel now comes back from ImageIO whole, decoded and turned upright, and CoreGraphics shrinks it, in premultiplied pixels. A picture with no alpha channel keeps ImageIO's scaled decode, since it hides nothing. After the change, `firefox.png` has 1 near-black edge pixel and the green disc none.

## Why Windows may have it too

**WIC's Fant scaler gets the same input ImageIO did.** For a PNG, the Windows half of `thumbnail.rs` hands the scaler straight alpha, colors not weighed by opacity, and Microsoft doesn't document whether Fant weighs them as it averages.

**The only evidence it's clean is a look, and a look missed it here.** The comment above the scaler said the green disc came back clean on the Windows box on 2026-10-22, judged by eye; it now says so. On the Mac, the same disc looked clean too, and a count found 1,104 tinted pixels.

**What would settle it is a count.** WIC's thumbnail of `firefox.png` at 210 pixels, through the same steps `thumbnail.rs` takes, against a correct shrink of the same file, counting the partly clear edge pixels whose color, unpremultiplied, is near black; the Mac's 1,215 against 1 is the size of the fault to expect. Then the look the user took here: `firefox.png` and `firefox.svg` side by side on the sheet.

**If WIC bleeds, the fix is likely small.** WIC has premultiplied pixel formats, so converting to one ahead of the scaler, and back to straight alpha in the conversion the pipeline already ends with, would keep the hidden colors out, in WIC's own terms.
