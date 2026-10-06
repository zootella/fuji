# Dates

Which date fuji shows for a file, where it comes from, and how it is written. Begun 2026-10-06 on the Mac mini, as the first of the pieces of data for the caption under each thumbnail on the contact sheet — the filename, a date, the picture's dimensions and the file's size — and built the same day.

## Decided

**Fuji shows a file's modified time.** The user's choice, 2026-10-06, from the stat times the file system keeps. In local time, the user's own clock and time zone.

## The three times a file system keeps

- **Accessed**, atime: when the file was last read. Nobody thinks of a file by it, systems often barely update it (macOS and linux mount with relaxed rules that skip most updates, and Windows has switched it off on large volumes at times), and fuji reading a file to make its thumbnail could be what moves it. Out.
- **Modified**, mtime: when the file's contents last changed. The default date column in both file managers, Finder's Date Modified and Explorer's Date modified, and on every platform fuji runs on. Copying in Finder or Explorer keeps it. For a picture nobody edits, it is in effect when the picture was made or arrived.
- **Created**, the birth time: when this copy of the file came into being. Finder and Explorer both offer it as a column. Explorer gives a copied file a new created time, the moment of the copy, while it keeps the modified time, so a picture copied from an old drive shows yesterday as its creation. Linux often keeps none at all, and Rust answers an error there.

A fourth is easy to confuse with these. **POSIX ctime** is the change time, when the file's metadata last changed — a rename, new permissions — not its creation. Rust's standard library does not offer it portably. `disk_stat` in `disk.rs` names its creation time `ctime` today, which reads as the POSIX one to anybody who knows POSIX; it should be renamed, to `birthtime` or `created`, when this work touches it.

## What sets a modified time, so a strange one can be explained

The program that writes the file sets it, and programs disagree. Known behavior, not measured here:

- **A browser download** gets the time of the download. The Firefox 4 betas in 2010 set it from the server's `Last-Modified` header instead, Mozilla's bug 178506, and that was taken out again before release; a 2011 survey on the W3C TAG's list found neither Firefox nor Chrome 15 keeping the server's time by default, and Firefox needing an extension for it.
- **The user remembers a Chrome version doing the same a few years ago**, with the download's own date landing in the access time. Not confirmed: nothing found so far says Chrome ever did either. Worth a look if a folder of downloads ever shows dates that make no sense.
- **wget** sets the modified time from `Last-Modified` by default, and **curl** does with `-R`, so files fetched on the command line can carry the server's date.
- **Unzipping** usually restores the time stored in the archive; **`cp` in a terminal** gives the copy a new modified time unless told `-p`; **a git checkout** gives every file the time of the checkout.

None of this changes the decision. It is why the date under a thumbnail will sometimes surprise a user, and it is the same date their file manager shows them, which is the point.

## Getting it

**`disk_readdir` returns each file's modified time**, as `mtime` in `DirEntry`, in milliseconds since 1970 and 0 where the file system has no answer, the way `FileStat` does. It comes from the `symlink_metadata` the listing already reads for each file's type and size, so it costs no more disk; and a listing with each file's modified time names no fuji feature, the test `lib.rs` opens with. A `disk_stat` per file was the alternative, a second trip to the disk and a second call into Rust for every thumbnail, for something the listing had already read. The same change renamed `FileStat`'s creation time from `ctime` to `birthtime`.

**From the listing to the caption.** `listDirectory` in `library.js` keeps the whole entry, `walk.js` carries each bucket's entries into the bucket, and `TestFlow.vue` takes each tile's entry as it makes the tile, so the name, the size in bytes and the modified time are in hand before any pixels arrive. The model indexes the table's folder by path the same way, and `modelFile` in `model.js` answers a path's entry there. The caption under the diamond table's picture reads the same entry through `modelFile` on every flip, and writes the same second line after the full path, since 2026-10-06.

## Writing it

**Year, month, day, the day alone, like 2026 Oct 6.** The user calls it the Swedish form for the order; Swedish itself writes 2026-10-06. Built as `sayDay` in `library.js`, which the caption under each thumbnail on the sheet calls, and the caption under the table's picture. Decided with the user, 2026-10-06:

- **Local time.** A JavaScript `Date` made from the milliseconds, read with its local getters, so a file modified at 23:30 UTC is still that day for a user to the west of Greenwich.
- **Fuji's own month names**, a three-letter English list, `Sep` among them, rather than `Intl`'s, so the caption reads the same on every machine whatever its language.
- **Thin spaces around the month**, U+2009, written as `${thinSpace}` in the code, the rule `style.md` now states.
- **No time of day.**
- **Nothing at all for a file with no modified time**, which the listing reports as 0, rather than a day in 1970.
