# Windows agenda

The Windows box's work for the session that began 2026-10-22, after eight days away and 39 commits from the Mac. Most of it comes from `mac2win.md`, and each section here is one piece of that letter, or of getting this machine building again, under a nickname we use while we burn it down. A section is burned down as its work is done or decided, deleted rather than marked, and the last one, LetterBack, carries whatever the Mac needs to hear.

## BuildFloor

**Open: the upload, before any rebuild.** The staged installer, `45e5a88a…`, is the build installed and looked at, its video window included, and its sidecar, `desktop/release/fuji.exe.json`, is already committed, in `e6f1fde`. `pnpm upload` sends it; it goes before VideoProcess builds anything, since a `pnpm hash` after a rebuild would rewrite that sidecar for a build that never went out.

## VideoProcess

**Open, decided and not built.** The Mac is the special case: one process holds every window there, so the Dock shows one icon. Everywhere else one window is one process, whatever opened it, written in code as not the Mac rather than as Windows and Linux. Today `v` on the sheet calls `window_new`, which puts the video window in the sheet's process on every platform; off the Mac it should start a copy of fuji instead, by the same launch an Explorer double-click takes, so what the user sees is what fuji is doing, and a decoder that crashes on an old file takes down only its own window.

- **Measure first.** With a test folder on the sheet and `v` pressed once, double-click two videos in Explorer, and list the processes: whether the fuji processes share one WebView2 browser process, since they share a data folder, is what each extra window costs.
- **Two things to settle.** `v` with no video has no path to hand the new copy, so it needs a way to ask for an empty video window; and Windows makes one window of launches that start within 200 milliseconds of each other, for a selection opened together, which two quick presses of `v` must not trip.
- **The documents that change with it.** `instances.md`'s opening paragraph, `window-check.md`'s How windows come to be and How windows go, and the essays in `window.rs` and `lib.rs` that name `window_new`; and LetterBack, since it changes the Mac's design of 2026-10-21 for every platform but its own.

## PanelVideo

**Open.** The audio panel's video region now takes the playing video's shape, and `window_video_place` moves and resizes the native child window to it with `SetWindowPos`, never yet run. With `pnpm local`, press `a`, drop a wide video and then a tall one, and resize the window. A video that keeps its first size or stays in a corner means mpv's hook on the window misses fuji's moves. At 125 or 150 percent, a thin line of the page or a pixel of the blue frame along one edge of the video is the rounding to whole backing pixels. Every placement is an `audio: video placed at` line in the log. Wants a wide and a tall test video in a folder made for fuji.

## AlphaRim

**Open.** Two steps in `render_com` in `thumbnail.rs` where Microsoft's documentation says nothing about alpha, both running on a PNG's straight alpha: the Fant scaler, which may average colors hidden under transparent pixels into every edge, and `IWICColorTransform` for a PNG with an ICC profile, which may drop alpha altogether. The letter's two test files, a white disc with bright green hidden under its transparent pixels, plain and with Windows' sRGB profile, are made with ImageMagick and exiftool, and opened on the contact sheet. A green rim means Fant averages straight colors, and a green or black square behind the profiled disc means the transform drops alpha. If either shows, the fix is converting to 32bppPBGRA before the scaler and letting the final converter to 32bppRGBA unpremultiply, as the Mac's code already does.

## SeeThrough

**Open.** What shows through the transparent parts of a picture should be the view's own ground, which the user has decided and nothing yet does. On the Diamond Table that is the dots, which pan with the picture and keep their size as it zooms, where today the card's `bg-well` sits behind every picture. On the contact sheet it is a repeating background that stays fixed as the sheet scrolls, where today the sheet is plain paper. The letter puts the correction here, since this box is the harder place to get transparency right, and it follows AlphaRim, whose test files show it.

## LetterBack

**Open, filled as we go.** `win2mac.md` written fresh in answer to `mac2win.md`, with a section only for what the Mac needs to know or do, since everything else is in git. So far: nothing.
