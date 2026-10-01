# Between the Windows box and the Mac: a sheet placed twice, and file types reached a new way

Written on the Windows box, 2026-09-28. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one.

## A sheet placed twice, for the Windows box to look at once

Found on the Mac mini on 2026-09-30 and fixed there: coming from a preview, the sheet grew 28 CSS pixels taller than its saved size, because the Mac keeps the content when a title bar arrives and grows the frame around it. `sheetFromPreview` now places the sheet before adding the title bar and again after, at the same rectangle, and the comments at both `windowFrameSet` calls in `Shell.vue` say why.

On Windows the second placement should ask for the frame the window already has and change nothing. It is reasoned rather than seen. The title bar should land first, since tauri posts it to the main thread, tao applies it there in one step, and the next command arrives through the same queue; on the Mac the log showed it landed within 11 ms. But if `window_frame_set` ever measured before it and resized after, the sheet would come out a title bar taller on Windows instead, which is the one thing to look for. Open a picture from Explorer, click through the table to the sheet, quit, and expect `[sheet]` in `fuji.toml` unchanged.

## File types, reached a new way, for the Windows box to look at once

Written on the Mac mini on 2026-10-01, alongside the Mac's half of file types. `associate.js` now reaches every platform step through one table, `system`, so Windows' calls to `registryOpens`, `register()` and the installed test go through it rather than by name, and `installed()` is now `installedWindows()`. The statements Windows runs are the same, in the same order, but this shape of the file has only ever run on the Mac, so it is worth one pass on the installed copy: open the settings, expect the chips under the same headings as before, and choose Fuji for one type and No longer open with Fuji for another, expecting what they did on 2026-09-29.
