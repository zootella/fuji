# Windows agenda

The Windows box's work for the session that began 2026-10-22, after eight days away and 39 commits from the Mac. Most of it comes from `mac2win.md`, and each section here is one piece of that letter, or of getting this machine building again, under a nickname we use while we burn it down. A section is burned down as its work is done or decided, deleted rather than marked, and the last one, LetterBack, carries whatever the Mac needs to hear.

## BuildFloor

**Open: the upload, of one of two builds.** The staged installer, `45e5a88a…`, has its sidecar, `desktop/release/fuji.exe.json`, committed in `e6f1fde`, and `pnpm upload` sends it as it is. The bundle folder holds a newer one, `c8df6cd5…`, with `v` as a process of its own, the build installed and looked at last; sending it instead is `pnpm hash`, which rewrites the committed sidecar for it, then `pnpm upload` and a commit of the sidecar, and leaves `e6f1fde` recording a build that never went out.

## VideoSelection

**A future question, found on 2026-10-22.** Several videos selected in Explorer and opened together with Enter start a copy of fuji each within milliseconds, and the flurry rule in `window.rs` keeps one window between them, the first copy's video, while the others exit without a window. For pictures that is the design, one window with the rest a flip away. For videos, each a window of its own, it may not be: the choices are a window for every video in the selection, or the one window as now. Nothing opens several videos together yet that the user would meet, so it waits for the contact sheet to list videos and for the user to decide.

## PanelVideo

**Open.** The audio panel's video region now takes the playing video's shape, and `window_video_place` moves and resizes the native child window to it with `SetWindowPos`, never yet run. With `pnpm local`, press `a`, drop a wide video and then a tall one, and resize the window. A video that keeps its first size or stays in a corner means mpv's hook on the window misses fuji's moves. At 125 or 150 percent, a thin line of the page or a pixel of the blue frame along one edge of the video is the rounding to whole backing pixels. Every placement is an `audio: video placed at` line in the log. Wants a wide and a tall test video in a folder made for fuji.

## AlphaRim

**Open.** Two steps in `render_com` in `thumbnail.rs` where Microsoft's documentation says nothing about alpha, both running on a PNG's straight alpha: the Fant scaler, which may average colors hidden under transparent pixels into every edge, and `IWICColorTransform` for a PNG with an ICC profile, which may drop alpha altogether. The letter's two test files, a white disc with bright green hidden under its transparent pixels, plain and with Windows' sRGB profile, are made with ImageMagick and exiftool, and opened on the contact sheet. A green rim means Fant averages straight colors, and a green or black square behind the profiled disc means the transform drops alpha. If either shows, the fix is converting to 32bppPBGRA before the scaler and letting the final converter to 32bppRGBA unpremultiply, as the Mac's code already does.

## SeeThrough

**Open.** What shows through the transparent parts of a picture should be the view's own ground, which the user has decided and nothing yet does. On the Diamond Table that is the dots, which pan with the picture and keep their size as it zooms, where today the card's `bg-well` sits behind every picture. On the contact sheet it is a repeating background that stays fixed as the sheet scrolls, where today the sheet is plain paper. The letter puts the correction here, since this box is the harder place to get transparency right, and it follows AlphaRim, whose test files show it.

## LetterBack

**Open, filled as we go.** `win2mac.md` written fresh in answer to `mac2win.md`, with a section only for what the Mac needs to know or do, since everything else is in git. So far: one window, one process, everywhere but the Mac, and what the Mac should build and look at for it.
