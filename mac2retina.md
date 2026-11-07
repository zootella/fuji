# Mac to Retina

## Why this machine

**The MacBook runs a custom scaled setting, so macOS draws fuji at 2 backing pixels per css pixel and then resamples that onto the panel at a ratio that is not a whole number.** No page can reach that resample, and the Mac mini's 5K at 2x has none, so nothing on the Mac mini shows what it does to the lines fuji is trying.

**First, tell the user the ratio.** Run `system_profiler SPDisplaysDataType`: Resolution is the panel, and UI Looks like is the size in css pixels. Panel pixels per backing pixel is the panel's width over twice the looks-like width. Say it to the user as a fraction in lowest terms and as a decimal, since they want to know exactly what it is.

**Judge by eye, on the panel.** From memory, a macOS screenshot captures backing pixels, from before the resample, so a screenshot shows what fuji drew and not what the panel shows. The session there never takes the screenshots; the user looks and says what they see.

## The spike

The user's aim: lines that are not fragile across every display, ratio and pixel unit fuji lands on, that look good, and that are not a solid line.

- The spike runs on the bench, `FujiUserInterfaceSpike.vue`, which `Shell.vue` says how to open.
- Each experiment is a block of rules in `FujiTree.vue` or `FujiList.vue`, switched on by an attribute the bench sets from a toggle in its strip. The comment on each block says what it does.
- The tree's twisties are hidden for the spike, so its lines are the whole show.
- The magnifier among the toggles is for measuring geometry in a screenshot, and is beside the point here, where the question is what happens at fuji's own size.

## What to look at

For a pattern with a rhythm along a line, the question is whether the resample leaves it reading as it does on the Mac mini, or blends it toward one gray, or makes it beat, darker and lighter patches repeating along a line and crawling as the view scrolls.

- **The tree's Pattern,** where the guide lines are cut from stripes into dashes. Compare it against the same lines with Pattern off, which are solid and should only come out a little softer.
- **The list's focus rectangle with its stripe experiment on,** black and white dashes, the finest rhythm fuji draws and so the most likely to suffer. Click a row and press an arrow to bring the rectangle up.
- **The score between the rows of a run of selected rows,** a thin line in a color a step away from the fill. Look at whether it stays a faint, even line along the whole run, with the keyboard in the list and without.
- **The tree's lines themselves,** centered by a translate at `--guide-width`, which can be a fraction of a css pixel. Look at whether they read evenly from row to row, or some rows' lines look heavier than others.
