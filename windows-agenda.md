# Windows agenda

The Windows box's session of 2026-10-22, after eight days away and 39 commits from the Mac, burned down. What's left are the future questions it found, each under a nickname and each waiting on something that hasn't arrived yet.

## VideoSelection

**A future question, found on 2026-10-22.** Several videos selected in Explorer and opened together with Enter start a copy of fuji each within milliseconds, and the flurry rule in `window.rs` keeps one window between them, the first copy's video, while the others exit without a window. For pictures that is the design, one window with the rest a flip away. For videos, each a window of its own, it may not be: the choices are a window for every video in the selection, or the one window as now. Nothing opens several videos together yet that the user would meet, so it waits for the contact sheet to list videos and for the user to decide.

## VideoSeam

**A future check, found on 2026-10-22.** At 125 percent a 10-pixel inset lands on 12.5 backing pixels, and the page rounds a video's rectangle to whole ones, so a one-pixel sliver of the page along one edge of the video would be that rounding; 150 is a whole 15. It wasn't looked at on the audio panel, whose region is on its way to the video window, so it's to be seen there, at 125 percent, the first time the video window plays a video. The audio panel logs each placement, `audio: video placed at`, with the box in CSS pixels and the rectangle in backing pixels, and a line like it from the video window is the way to read the rounding there.

## SystemBoxes

**Two cases with no caller yet, found on 2026-10-22.** `dialog.rs` handles both, and neither has run in Fuji, so each is seen the day a caller arrives: `dialog_save`, which nothing in Fuji asks for, and a box asked for from a hidden or minimized window, which stands on its own rather than on the window, as reasoned from how AppKit attaches a sheet and how Windows treats an owned window.
