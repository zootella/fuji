# Windows agenda

The Windows box's work for the session that began 2026-10-22, after eight days away and 39 commits from the Mac. Most of it comes from `mac2win.md`, and each section here is one piece of that letter, or of getting this machine building again, under a nickname we use while we burn it down. A section is burned down as its work is done or decided, deleted rather than marked, and the last one, LetterBack, carries whatever the Mac needs to hear.

## VideoSelection

**A future question, found on 2026-10-22.** Several videos selected in Explorer and opened together with Enter start a copy of fuji each within milliseconds, and the flurry rule in `window.rs` keeps one window between them, the first copy's video, while the others exit without a window. For pictures that is the design, one window with the rest a flip away. For videos, each a window of its own, it may not be: the choices are a window for every video in the selection, or the one window as now. Nothing opens several videos together yet that the user would meet, so it waits for the contact sheet to list videos and for the user to decide.

## SystemBoxes

**Two cases with no caller yet, found on 2026-10-22.** `dialog.rs` handles both, and neither has run in Fuji, so each is seen the day a caller arrives: `dialog_save`, which nothing in Fuji asks for, and a box asked for from a hidden or minimized window, which stands on its own rather than on the window, as reasoned from how AppKit attaches a sheet and how Windows treats an owned window.

## PanelVideo

**Open.** The audio panel's video region now takes the playing video's shape, and `window_video_place` moves and resizes the native child window to it with `SetWindowPos`, never yet run. With `pnpm local`, press `a`, drop a wide video and then a tall one, and resize the window. A video that keeps its first size or stays in a corner means mpv's hook on the window misses fuji's moves. At 125 or 150 percent, a thin line of the page or a pixel of the blue frame along one edge of the video is the rounding to whole backing pixels. Every placement is an `audio: video placed at` line in the log. Wants a wide and a tall test video in a folder made for fuji.

## LetterBack

**Open, filled as we go.** `win2mac.md` written fresh in answer to `mac2win.md`, with a section only for what the Mac needs to know or do, since everything else is in git. So far: one window, one process, everywhere but the Mac, the system's boxes, and what shows through a transparent picture, each with what the Mac should build and look at for it.
