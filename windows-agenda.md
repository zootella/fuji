# Windows agenda

The Windows box's work for the session that began 2026-10-22, after eight days away and 39 commits from the Mac. Most of it comes from `mac2win.md`, and each section here is one piece of that letter, or of getting this machine building again, under a nickname we use while we burn it down. A section is burned down as its work is done or decided, deleted rather than marked, and the last one, LetterBack, carries whatever the Mac needs to hear.

## BuildFloor

**Open: the upload and the release commit, after VideoWindow.** The build is staged, installed and looked at; `pnpm upload` sends it, and the release commit carries its sidecar, `desktop/release/fuji.exe.json`. It waits for VideoWindow because this is the first Windows build with the video window in it, and a fix there would replace the build anyway.

## VideoWindow

**Open.** The things in the letter's last section that only this machine can show, on the video window `v` opens from the sheet.

- **The resize band over the buttons.** Tauri lays a child window, `TAURI_DRAG_RESIZE_WINDOW`, across the top of an undecorated window, `SM_CYFRAME` deep. Bring the pointer slowly down onto fuji's buttons from above the window: the number wanted is how many pixels show the resize arrow before a button lights, at 100 percent and at 150.
- **Maximize with no title bar.** The middle button should fill the work area with the taskbar still showing, the three buttons whole in the top right corner and the middle one showing restore, and a second press should bring the window back where it was.
- **Fuji's own drag.** Dragged by its middle, the window should follow the pointer without lagging, snap to nothing, and go past the top of the screen as far as the pointer takes it, staying there on release. A click afterward should be an ordinary click. Maximized, the drag is Windows' own, which restores the window as it begins. A double-click should still go into fuji's fullscreen and back.
- **Past the edges.** With the window partly above the top or past the taskbar: whether the taskbar covers it or it covers the taskbar, and whether Windows moves it back on its own after a monitor sleeps or when it is restored from the taskbar.

## PanelVideo

**Open.** The audio panel's video region now takes the playing video's shape, and `window_video_place` moves and resizes the native child window to it with `SetWindowPos`, never yet run. With `pnpm local`, press `a`, drop a wide video and then a tall one, and resize the window. A video that keeps its first size or stays in a corner means mpv's hook on the window misses fuji's moves. At 125 or 150 percent, a thin line of the page or a pixel of the blue frame along one edge of the video is the rounding to whole backing pixels. Every placement is an `audio: video placed at` line in the log. Wants a wide and a tall test video in a folder made for fuji.

## AlphaRim

**Open.** Two steps in `render_com` in `thumbnail.rs` where Microsoft's documentation says nothing about alpha, both running on a PNG's straight alpha: the Fant scaler, which may average colors hidden under transparent pixels into every edge, and `IWICColorTransform` for a PNG with an ICC profile, which may drop alpha altogether. The letter's two test files, a white disc with bright green hidden under its transparent pixels, plain and with Windows' sRGB profile, are made with ImageMagick and exiftool, and opened on the contact sheet. A green rim means Fant averages straight colors, and a green or black square behind the profiled disc means the transform drops alpha. If either shows, the fix is converting to 32bppPBGRA before the scaler and letting the final converter to 32bppRGBA unpremultiply, as the Mac's code already does.

## SeeThrough

**Open.** What shows through the transparent parts of a picture should be the view's own ground, which the user has decided and nothing yet does. On the Diamond Table that is the dots, which pan with the picture and keep their size as it zooms, where today the card's `bg-well` sits behind every picture. On the contact sheet it is a repeating background that stays fixed as the sheet scrolls, where today the sheet is plain paper. The letter puts the correction here, since this box is the harder place to get transparency right, and it follows AlphaRim, whose test files show it.

## LetterBack

**Open, filled as we go.** `win2mac.md` written fresh in answer to `mac2win.md`, with a section only for what the Mac needs to know or do, since everything else is in git. So far: nothing.
