# To the Windows box: the video moves

Written on the Mac mini, 2026-10-15, and added to through 2026-10-21. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one. One section is left: the audio panel's video, which moved on the Mac on 2026-10-16.

## The audio panel's video moves now

The video region in the audio panel now takes the playing video's shape, along the top for a wide video and down the left for a tall one, with the panel's lines scrolling beside or below it. `window_video_place` in `window.rs` moves and resizes the native video window to the region whenever the panel lays out, and hides it while no video shows; here that is `SetWindowPos` and `ShowWindow` on the child window `window_video_make` makes. It type-checks against this target from the Mac, in a scratch crate without Tauri, and has never run.

Two things only this machine can show. mpv draws in a window of its own inside the one fuji hands it, and resizes it to match through a hook it puts on that window's thread, read in mpv's `video/out/w32_common.c` and not yet seen at work: a video that keeps its first size, or stays in the corner, while the region moves would mean the hook misses a move fuji makes. And at 125 or 150 percent the panel's 10-pixel inset lands on a fraction of a backing pixel, which the Mac's 1 and 2 never do; the page rounds the video's rectangle to whole backing pixels, so a thin line of the page along one edge of the video, or a pixel of the blue frame under it, would be that rounding. Every placement is a line in the log, `audio: video placed at`, with the box in CSS pixels and the rectangle in backing pixels.

To see both, `pnpm local`, press `a`, drop a wide video and then a tall one, and resize the window; the blue frame and the divider are there to show the edges.
