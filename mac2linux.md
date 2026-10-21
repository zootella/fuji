# From the Mac to Linux: the video window, and the buttons fuji draws on it

Written on the Mac mini, 2026-10-21. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one.

## What changed

v on the contact sheet opens a video window, `VideoWindow.vue`, a native window of its own beside the main one, whose title bar comes off and in whose place fuji draws its own minimize, maximize and close, the same on every platform rather than a copy of any one Linux desktop's. `window-check.md` at the root says how it behaves as the user meets it, and the essay atop `VideoWindow.vue` is the rule. None of it has been built for Linux yet.

## What only Linux can check

- **Two windows in one process.** Until now every fuji process on Linux held one window. A video window from the sheet is a second window in the main window's process, made by `window_new`, an async command that builds it from the blocking pool, and everything Rust keeps per window gets a second entry here for the first time. Press v on the sheet: a video window should appear on top, with nothing hanging, and v twice should give two. Close the main window with a video window still open: fuji should keep running until that last window closes, and then be gone.
- **The drag, on Wayland.** The page asks tao to drag only once the pointer has moved 4 CSS pixels with the button down, so the request comes on a motion rather than on the press. tao answers with GTK's `begin_move_drag(1, x, y, 0)`, current time and no press event (tao 0.35.3's Linux event loop), and on Wayland a compositor starts a move only for a press it can match. If a drag from the middle of the video window moves it under X11 and not under Wayland, that is the place to look.
- **The resize band and the close button.** On a window with no title bar, tao shows a resize arrow within 5 pixels of each edge and starts the window manager's resize on a press there. Fuji's close button sits flush in the top right corner, so its outermost pixels lie inside that band, and a press on the button's top right corner may start a resize instead of closing the window. How many pixels it is, and whether anyone would hit it, is the answer wanted.
- **Minimize and maximize.** On Wayland a minimized window cannot bring itself back, so a minimized video window should come back from the dock, Activities or Alt+Tab. Maximize should fill the work area below the top bar, with the middle button showing restore, and a second press bring the window back where it was.
