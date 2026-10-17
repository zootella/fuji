# Video spike

The floor is built and released on all three platforms: one media library from one recipe, four player commands and a panel that are the same code everywhere, and a native rectangle per platform that a video draws in. What the floor is and why it is shaped that way lives in the code's essays, `CLAUDE.md`, the README and the Free Software article. What is still open in this sprint is on the picture side beside it, below.

## Transparency through both image roads (nicknamed "ClearTest": smoke test in this sprint)

The native road keeps a picture's alpha channel: the thumbnailer hands back straight-alpha pixels on the Mac and on Windows, the sheet's canvas keeps them, and the ground shows through, so a transparent PNG needs no background color sent to rust and stays on both roads, while GIF and SVG stay on the engine for animation and vectors rather than for transparency. To see it: a transparent PNG, a transparent GIF and an SVG with open space, on the contact sheet and on the Diamond Table, in light and in dark, where the ground should show through each in the theme's own color and never as white, black or a checkerboard. The PNG tile is the one that takes the native road on the Mac and Windows, and the other two are the engine's own.
