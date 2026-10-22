# To the Windows box: the video moves, and transparency

Written on the Mac mini, 2026-10-15, and added to through 2026-10-21. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one. Two sections are left: the audio panel's video, which moved on the Mac on 2026-10-16, and transparency, added on 2026-10-17: two steps in WIC that may lose it, and what should show through a transparent picture.

## The audio panel's video moves now

The video region in the audio panel now takes the playing video's shape, along the top for a wide video and down the left for a tall one, with the panel's lines scrolling beside or below it. `window_video_place` in `window.rs` moves and resizes the native video window to the region whenever the panel lays out, and hides it while no video shows; here that is `SetWindowPos` and `ShowWindow` on the child window `window_video_make` makes. It type-checks against this target from the Mac, in a scratch crate without Tauri, and has never run.

Two things only this machine can show. mpv draws in a window of its own inside the one fuji hands it, and resizes it to match through a hook it puts on that window's thread, read in mpv's `video/out/w32_common.c` and not yet seen at work: a video that keeps its first size, or stays in the corner, while the region moves would mean the hook misses a move fuji makes. And at 125 or 150 percent the panel's 10-pixel inset lands on a fraction of a backing pixel, which the Mac's 1 and 2 never do; the page rounds the video's rectangle to whole backing pixels, so a thin line of the page along one edge of the video, or a pixel of the blue frame under it, would be that rounding. Every placement is a line in the log, `audio: video placed at`, with the box in CSS pixels and the rectangle in backing pixels.

To see both, `pnpm local`, press `a`, drop a wide video and then a tall one, and resize the window; the blue frame and the divider are there to show the edges.

## Transparency, through WIC and on both views

On the Mac the native road keeps a picture's alpha, but ImageIO's thumbnail reads a little of the color hidden under fully transparent pixels. Measured on the Mac mini on 2026-10-17: a white disc on transparent pixels hiding bright green came back with green at its edge, up to 24 levels of 255 as it composites, on 130 of the thumbnail's 2,588 edge pixels, though no pixel of the file with any opacity carries green. Most transparent PNGs hide black, so on the Mac that is a faint dark rim. Windows has two steps in `render_com` in `thumbnail.rs` where Microsoft's documentation says nothing about alpha, and both run on a PNG's straight alpha:

- **The Fant scaler.** PNG offers no scaled decode, so Fant shrinks the frame in its own 32bppBGRA. If it averages those colors without weighting them by alpha, the hidden color bleeds into every edge, and more strongly than on the Mac.
- **The color transform.** A PNG carrying an ICC profile goes through `IWICColorTransform` into 32bppBGRA, and whether alpha survives it is undocumented. If it does not, the transparent area comes out as the hidden color.

Two files show both, made with ImageMagick and exiftool, which `winget install --id ImageMagick.ImageMagick -e` and `winget install --id OliverBetz.ExifTool -e` bring if this box lacks them. The first is the white disc with bright green hidden under its transparent pixels and nothing else in the file, and the second adds Windows' own sRGB profile, which changes no color:

```
magick -size 1200x1200 "xc:rgba(0,255,0,0)" -fill white -draw "circle 600,600 600,100" -define png:exclude-chunks=cHRM,gAMA,sRGB,bKGD,tIME,tEXt,zTXt,date PNG32:disc.png
exiftool -q "-ICC_Profile<=C:\Windows\System32\spool\drivers\color\sRGB Color Space Profile.icm" -o disc-profiled.png disc.png
```

Open their folder on the contact sheet. A green rim plain at a glance around either disc means Fant averages straight colors, and a green or black square behind the profiled disc means the transform drops alpha. If either shows, the fix is converting to 32bppPBGRA before the scaler, the format Microsoft recommends for 8-bit WIC pipelines on its Direct2D bitmap source page, and letting the final converter to 32bppRGBA unpremultiply, as the Mac's code already does. A table shows a PNG through the engine on both platforms, so the two files test the sheet alone.

What should show through the transparent and partly transparent parts of a picture is the view's own ground, which the user has decided and nothing yet does. On the Diamond Table that is the dots, which pan with the picture and keep their size as it zooms; today the card's `bg-well`, `--color-well` in `index.css`, sits behind every picture instead, near white in light and near black in dark. On the contact sheet it is a repeating background that stays fixed as the sheet scrolls; the sheet's ground is plain paper today, with no pattern. This box is the harder place to get transparency right, so the correction is made and tested here.
