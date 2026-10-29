# To Windows from the Mac

Read this, then talk with the user before changing anything; a letter starts a conversation rather than settling one. Each section is one area, and goes once its area is settled.

## Lists: the selection's colors, the keys, and the feel

**Windows' selection colors in `style.css` rest on a theme file nobody has checked against Explorer.** The `win-` colors for a selected row with the keyboard and without it were read from a copy of Windows 10's theme file, aero.msstyles 10.0.19041, found in a public archive, from the ItemsView::ListView part. The menu's colors from the same file agree with a screenshot and with code that patches them, but no screenshot of Explorer's list was measured. What settles it is Explorer and fuji's list side by side, a row selected in each, then the same after clicking into another window: the pale blue, and the gray once the keyboard is elsewhere, should match, with the text left dark on both.

**FujiList's keys have only run in WebKit.** On Windows the command key is Control, set as keyboard.js's default with nothing on Windows changing it, and WebView2 hands the page keys its own way. To try: Control with the arrows moves only the focus rectangle, Control and space picks the row it is on, Control and A selects every row, and an Alt chord goes to Windows rather than the list. Plain letters type a name to jump to, and Escape hands the keyboard back to fuji.

**The speed and the feel**, which the user is checking there anyway: building, filtering, scrolling and dragging across ten thousand rows on the interface spike, which `u` opens.

## Menus: the mock menu's colors, and a right-click from behind

**The menu highlight comes from a white menu, and fuji's menu is gray.** Windows 10 sets its white highlight on a `#eeeeee` menu, and the dark one on `#2b2b2b`; fuji's menu stands on Zed's gray instead, so whether the white still reads as a highlight in light mode, and the dark one in dark, has not been seen. A right-click on the contact sheet opens the mock menu.

**A right-click on a window behind another now brings it forward first**, because WebKit on the Mac tracked the pointer only in the window in front, so the menu's rows stayed dark until a click. WebView2 may track the pointer differently in a background window. To try: click into another program, right-click fuji's contact sheet, and move over the rows, which should light one at a time with no click.

## The video window's buttons

**These are the buttons meant to look like Windows' own**, so the test is beside a real title bar: under the pointer, minimize and maximize show a faint gray and close Windows 10's red; pressed, all three flash purple on day-glo, which is fuji's own touch. The video window is `v` from the contact sheet.

## Thumbnails: a dark fringe on transparent PNGs

**The Mac's thumbnailer let the color hidden under a PNG's clear pixels into its edges, and WIC may too.** A PNG keeps a color for every pixel, even a clear one, usually black, and a correct shrink weighs each color by its opacity first. ImageIO's resample did not, so the hidden black bled into every outline; at the sheet's 210 pixels its thumbnail of `firefox.png` had 1,215 near-black partly clear edge pixels against a correct shrink's 1, and the green-hidden test disc came back with 1,104 green-tinted ones. The Mac half of `thumbnail.rs` now shrinks a picture with alpha in premultiplied pixels itself. The user saw it only with Firefox's logo as a PNG and an SVG side by side, since the engine draws the SVG correctly ([the SVG](https://commons.wikimedia.org/wiki/File:Firefox_logo,_2019.svg), and [the PNG Wikimedia renders from it](https://commons.wikimedia.org/wiki/Special:FilePath/Firefox_logo,_2019.svg?width=800)).

**WIC's Fant scaler gets straight alpha too**, colors not weighed by opacity, and Microsoft does not document whether it weighs them. The green disc looked clean on the Windows box by eye, as it did on the Mac while carrying 1,104 tinted pixels. What settles it is a count: WIC's thumbnail of `firefox.png` at 210 pixels, through the steps `thumbnail.rs` takes, against a correct shrink, counting partly clear edge pixels whose unpremultiplied color is near black; then the PNG and the SVG side by side on the sheet. If WIC bleeds, the likely fix is converting to one of WIC's premultiplied formats ahead of the scaler and back to straight alpha in the conversion the pipeline already ends with.

## The inventory's file system facts

**The changed time, the count of a file's names, and whether it is hidden come through code only Windows runs**, behind `#[cfg(windows)]` in copper's `inventory.rs`: the first two from `GetFileInformationByHandleEx` on the open file, and `hidden` from its attribute. The walk ran it in both processes there, so it runs; nobody has held its answers against files made to test them. FAT and exFAT keep no change time, and Windows may answer zero there, which the inventory leaves out, or something that would show as a `changed` meaning nothing.

**What to look at:** an inventory of a folder on the NTFS system drive holding a file renamed after its last edit, whose `changed` should be later than its `modified`; a hard link made with `mklink /H`, carrying `"links": 2` under both names; and a file marked hidden in Explorer, carrying `"hidden": true`. Then an inventory of a USB drive, for what `changed` says there. Every path should read with forward slashes.

## The Recycle Bin, and erase's count of a file's names

**The Windows half of copper's `trash.rs` has never been built or run**, and nothing in fuji calls it; the test calls it from outside the repository. The Windows halves of `erase.rs` have never run either, and are no part of this: no session runs `disk_erase` or `disk_move` while smoke testing.

**On a drive with no Recycle Bin the shell deletes outright, as the flags stand, and says nothing.** A drive with a `$RECYCLE.BIN` folder takes the item into the combined Recycle Bin, as Explorer's Delete does. On a drive with none, a network share, a memory card, or one set to "Don't move files to the Recycle Bin", Explorer asks "Are you sure you want to permanently delete" first. Fuji passes `FOF_ALLOWUNDO | FOF_NO_UI`, and the `FOF_NOCONFIRMATION` inside `FOF_NO_UI` answers that question yes, so a move to the Recycle Bin there is a plain delete the user never chose. Microsoft's flag for the case is `FOF_WANTNUKEWARNING`, which "partially overrides FOF_NOCONFIRMATION", and the trash crate passes it in its own Windows half, read in trash 5.2.9's windows.rs; fuji's hand-written half left it out.

**Two roads, for the user to choose between there.** The first adds that flag: the shell warns where Explorer would, and a No comes back through `GetAnyOperationsAborted` as fuji's error. The second refuses rather than asks, as the Mac half already does on a volume with no trash, so the page could ask with a dialog of fuji's own on both; but no documented flag asks the shell to refuse, and nothing clean tells beforehand whether a drive recycles. `FOFX_RECYCLEONDELETE`, "When a file is deleted, send it to the Recycle Bin rather than permanently deleting it," is the flag whose behavior on such a drive the documentation leaves open.

**The test, on files the user sets up:**

- A folder of a few test pictures on the system drive with a second copy beside it to restore from; the same on a USB stick with a `$RECYCLE.BIN`; and, if one is at hand, on a drive with none.
- The control: the user deletes one picture in File Explorer, right-clicking it and choosing Delete, then right-clicks it in the Recycle Bin, where Restore should be offered.
- A small binary crate in the session's scratchpad, depending on copper by its path, calls `copper::trash::trash` on one named picture on each drive and prints each answer, the first time the Windows half links and runs.
- The user restores the system drive's and the stick's pictures from the Recycle Bin and says whether each returns; for the drive with no Recycle Bin, whether its picture is in the bin, gone, or still there, and what the program printed.
- On that drive, the crate tries twice more with a copy of the Windows half changed, once with `FOF_WANTNUKEWARNING` and once with `FOFX_RECYCLEONDELETE`, and the user says what each did.

**Erase's count of a file's names, which the same crate can settle without erasing anything.** Erase asks `copper::inventory::links` for `FileStandardInfo` on the open handle, and a file with a second hard link is removed without the overwrite. When the query fails the function answers one, right for the inventory and wrong here, since erase would write over a file another name still reaches; the fix if one is found is to refuse. The crate can open one file on each drive at hand, the system drive, the stick, a memory card and a network share, and print whether the query answered, with its `NumberOfLinks` and `AllocationSize`, the second of which erase also reads to write over the slack past the file's end.

## Drive space

**disk_statvfs reads a volume's space through the fs4 crate, whose Windows half has never run in fuji.** There it finds the volume with `GetVolumePathNameW` and reads it with `GetDiskFreeSpaceExW` and `GetDiskFreeSpaceW`. The page hands paths over with forward slashes, which `GetVolumePathNameW` has not been seen to take, and Microsoft documents that it ignores a missing end of a path, which is why steel's `statvfs` checks that the path exists first. steel's `pnpm test` covers both, and a failure prints the numbers the call answered.
