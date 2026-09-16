# Instances and windows

Built and running. Windows and Linux get a process per double-click, because their shells work that way and their taskbars group the windows; macOS holds as many windows as the user wants in one process, because its Dock draws a tile per process and several fujis would be a row of identical icons.

The reasoning lives with the code. `window.rs` carries why the platforms differ, how a window is made, where it opens and how big, and how long fuji outlives its last one. `open.rs` carries how a picture reaches the window made for it. `lib.rs` carries the rule that decides how many windows a launch makes. `CLAUDE.md` carries the three rules that keep one codebase from forking at every later feature.

## Still open

**Whether a settings guard is ever worth building**, and what it would be: a lock, a merge rather than a replace, or writing only the keys a window actually changed.

Settings are last-modified-wins — the window someone most recently changed something in is the one whose whole view survives, and the other's change is gone rather than merged. That is accepted while fuji has one user. It is now a Windows and Linux question rather than a general one: macOS has one process and therefore one writer, while separate processes genuinely cannot see each other.
