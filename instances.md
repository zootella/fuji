# Instances and windows

Built and running. The Mac is the special case: macOS holds as many windows as the user wants in one process, because its Dock draws a tile per process and several fujis would be a row of identical icons. Everywhere else one window is one process, whatever opened it: a double-click in the file manager starts a copy of fuji, because those shells work that way and their taskbars group the windows, and so does a page asking for another window, which is how `v` opens a video window beside the contact sheet. So what the user sees is what fuji is doing, and a video whose decoder crashes takes only its own window with it. The code says this as the Mac and not the Mac, never as a list of the others.

The reasoning lives with the code. `window.rs` carries why the platforms differ, how a window is made, where it opens and how big, and how long fuji outlives its last one. `open.rs` carries how a picture reaches the window made for it. `lib.rs` carries the rule that decides how many windows a launch makes. `CLAUDE.md` carries the three rules that keep one codebase from forking at every later feature.

## Still open

**Whether a settings guard is ever worth building**, and what it would be: a lock, a merge rather than a replace, or writing only the keys a window actually changed.

Settings are last-modified-wins — the window someone most recently changed something in is the one whose whole view survives, and the other's change is gone rather than merged. That is accepted while fuji has one user. It is now a Windows and Linux question rather than a general one: macOS has one process and therefore one writer, while separate processes genuinely cannot see each other.
