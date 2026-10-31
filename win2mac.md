# To the Mac from Windows

Read this, then talk with the user before changing anything; a letter starts a conversation rather than settling one. Each section is one area, and goes once its area is settled.

## The list carries a selection

**A drag from a selected row now carries the selection**, in `_listCarry` in `list.js`. On Windows its pointer is OLE's own, from the new `copper/src/pointer.rs`, whose stub off Windows nothing calls, so the Mac keeps its plain arrow.

**To compile and look at:** `cd copper && pnpm test` and `pnpm local` build the new module's stub on the Mac. Then, in the bench, `u` from the sheet: select a few rows, press on one and drag, and the selection should hold under the plain arrow; press on a selected row and let go without moving, and only that row should stay selected; press on an unselected row and drag, and a run should still be selected, as before.

**What the pointer should be, and how a drag reaches past the window, is parked in `pointer-and-drag.md`**, with what the Mac would do from memory and unchecked: its plain arrow, and whether WebKit's `copy` and `alias` are AppKit's own drag badges.
