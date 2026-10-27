# To Windows: a dark fringe on transparent thumbnails, the inventory's file system facts, and the Recycle Bin

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

## The inventory's file system facts, which only Windows answers its own way

**Three things the inventory reads about each file come through code only Windows runs, and nobody has checked what they answer.** The changed time and the count of a file's names both come from `GetFileInformationByHandleEx` on the file the inventory already has open, one struct each, since Rust's standard library reads neither outside its unstable builds; `hidden` comes from the file's hidden attribute. All of it sits behind `#[cfg(windows)]` in copper's `inventory.rs`. The walk has run the inventory on the Windows box in both processes, which wrote the same lines, so the code runs; nobody has held its three answers against files made to test them.

**What could be wrong there.** The arithmetic from hundred-nanosecond steps since 1601 to milliseconds since 1970 is checked on the Mac; what the call answers is not. FAT and exFAT keep no change time, and Windows may answer zero there, which the inventory leaves out, or something else, which would show up as a `changed` that means nothing. The inventory also writes every path with forward slashes, turning Windows' backslashes over as it writes each line.

**What to look at.** An inventory of a folder on the NTFS system drive holding: a file renamed after it was last edited, whose `changed` should be later than its `modified`; a hard link made with `mklink /H`, which should carry `"links": 2` under both names; and a file marked hidden in Explorer, which should carry `"hidden": true`. Then an inventory of a USB drive, for what `changed` says there. Every path in the file should read with forward slashes.

## The Recycle Bin, which fuji has never put anything in

**The Windows half of copper's `trash.rs` has never compiled or run.** Nothing in fuji calls it, and nothing should for this test; the test calls it from outside the repository. The Windows halves of copper's `erase.rs` have never compiled either, and are not part of the test: no session runs `disk_erase` or `disk_move` while smoke testing, on any machine.

**The case most likely wrong is a drive with no Recycle Bin.** A drive with a `$RECYCLE.BIN` folder, the user's USB sticks among them, should take the item as Explorer's Delete does, into the one combined Recycle Bin. A drive with none, a network share or a drive Windows counts as removable, is where Explorer asks "Are you sure you want to permanently delete" first; with no interface, Microsoft's documentation of the flags does not say whether the shell deletes outright, asks anyway, or refuses. Deleting outright would make a Move to the Recycle Bin there a plain delete the user never chose.

**The test, with the user, on files the user sets up:**

- A folder of a few test pictures on the system drive, and a second copy beside it to restore from; the same on a USB stick that has a `$RECYCLE.BIN`, and, if one is at hand, on a drive that has none, a memory card or a network share, where Explorer's own Delete asks before deleting permanently.
- The control first: the user deletes one picture in File Explorer by right-clicking it and choosing Delete, then opens the Recycle Bin and right-clicks it there, where Restore should be offered.
- A few lines in the session's own scratchpad, a small binary crate depending on copper by its path in the repository, call `copper::trash::trash` on one named picture on each drive, and print each answer. That build is the first compile of the Windows half.
- The user opens the Recycle Bin, right-clicks the system drive's picture and the stick's, chooses Restore on each, and says whether each returns to its folder. For the drive with no Recycle Bin: whether its picture is in the Recycle Bin, gone from the drive, or still there, and what the program printed.

**Bring the answer for the drive with no Recycle Bin to the user.** What fuji's trash should do on a drive with no Recycle Bin is a decision, and the code's comment says the case is untried until this test runs.
