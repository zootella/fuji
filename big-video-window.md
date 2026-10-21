# Big video window

## The want

- The video window has no title bar, so the page is full bleed. Its corners are whatever each platform gives a window without one, rounded or square.
- The user drags it from anywhere on it, and resizes it from its edges.
- The user drags it past all four edges of the screen, the top included, on the Mac, Windows and Linux, without the platform pulling it back or holding it in.
- Later, the window grows past the screen to the video's own size, one of its pixels per panel pixel, and the same drag brings whichever part the user wants into view.

## Decided

- A video window is a native window of its own, and there can be none, one or several, beside a browsing window or without one. A browsing window keeps changing shape among the preview, the contact sheet, the light table and the panels, because those replace each other and share one page's state: the model, the pager, the decoded pictures, gamma and the help panel. A video window shares none of that. The path of the file it plays is the one thing it is handed, and it never answers, so a window boundary falls exactly where a state boundary already is. Every native window runs its own page, which is why the browsing views stay in one.
- Every way to a video window ends in `window_build`: a double-click in Finder or File Explorer, with fuji running or not, and a double-click on the contact sheet, which asks Rust for a window made for that path. The new window's page decides at mount, from the path it was handed, that it is a video window. On Windows and Linux one process can then hold a browsing window and video windows, and closing its last window, of either kind, ends it.
- Fuji uses each platform's own window drag until a platform shows a limitation. Fuji's own drag, which the next sections call method two, stays in reserve. The hope is never to need it on only one platform, but it could be used that way.
- The drag starts from anywhere on the window because, without a title bar, any part of the window still on the screen has to be a handle; a window whose title bar had gone past an edge would otherwise be stuck. Pressing v twice brings a lost video window back fresh inside the work area as well.

## The drag fuji uses today

- On the Mac, tao's `drag_window` calls AppKit's `performWindowDragWithEvent:` on the window, read in tao 0.35.3. AppKit is Apple's framework for windows, menus and views, the NS classes.
- On Windows, tao releases the mouse capture and posts the window `WM_NCLBUTTONDOWN` with `HTCAPTION`, read in tao 0.35.3, so Windows runs its own move loop as if the title bar had been grabbed, Snap included.
- On Linux, tao asks GTK to `begin_move_drag`, and the window manager runs the move from there.
- The page starts the drag only once the pointer has moved 4 CSS pixels with the button down. On Windows the release never reaches the page once the move loop begins, which the audio panel found, so a drag begun on the press would swallow every double-click.
- One limit holds on every platform and for either method: the pointer cannot leave the screen, so a window goes past an edge only by the distance from where it was grabbed to that edge. An 800 by 600 window grabbed by its middle can put 300 pixels above the top; a window taller than the screen, grabbed near its bottom, can put most of itself there.
- Snap on Windows, and the Mac's tiling where it is on, act when the pointer itself touches an edge, so letting go just short of the edge leaves the window where it was dragged.

## Taking the title bar off

- Fuji already takes a title bar off and puts one back. The preview loses its title bar as it is placed, and the first main window after a preview gets one back, so the video window can take the same road both ways.
- The frame is set first and the title bar changed after, never the other way, because tao queues a change of title bar and `window_frame_set` measures the window at the moment it is called.
- When a title bar arrives, the Mac keeps the content and grows the frame up around it, 28 CSS pixels taller than asked, measured on the Mac mini on 2026-09-30, so the frame is set a second time after the title bar is back. Windows keeps the frame.
- On the Mac, any change of the title bar drops the keyboard focus, and the shell hands it back to the page after every change.
- With no title bar, `window_frame` and `window_frame_set` measure no chrome, so the frame fuji sets is the page's own size.
- Hiding the window during the swap makes one window go and the other appear, which matches the two windows the user sees. On Windows, maximizing a hidden window shows it, so a maximized main window is maximized last, right before it is shown.
- The title stays: the Mac's Window menu still lists the window as Video, and Windows' taskbar as "Video - Fuji".
- The buttons go with the title bar: close, minimize and zoom on the Mac, and close, minimize and maximize on Windows. What stays is v, ⌘W on the Mac and Alt+F4 on Windows. Esc does nothing in the video window yet.

## The Mac

- Decorations off in tao make the window `Borderless` and keep it `Resizable`, read in tao 0.35.3's `set_decorations`.
- [Apple's documentation of `constrainFrameRect(_:to:)`](https://developer.apple.com/documentation/appkit/nswindow/constrainframerect(_:to:)) says AppKit modifies a window's frame "so that its top edge lies on a specific screen", that "if the window is resizable and the window's height is greater than the screen height, the rectangle's height is adjusted to fit within the screen as well", that "the rectangle's width and horizontal location are unaffected", and that it is invoked automatically "whenever a titled NSWindow object is placed onscreen and whenever its size is changed". So a window with a title bar cannot be taller than the screen, and a window without one is outside the rule as Apple documents it.
- AppKit's own drag stops a borderless window at the menu bar, as it does a window with a title bar, seen on the Mac mini on 2026-10-21. So getting above the top on the Mac takes method two.
- A borderless window has square corners: AppKit draws the rounded corners, and the shadow that follows them, as part of the frame it puts around a window with a title bar, and a borderless window has none of it, seen on the same day.
- Believed and not yet seen on fuji: a window with a title bar drags off the left, the right and the bottom, under the dock included, and not up under the menu bar.
- AppKit resizes a borderless window from its edges, since tao adds nothing on the Mac, seen on the Mac mini on 2026-10-21.
- Sequoia tiles a window dragged to an edge or a corner. Sources disagree on whether that is on by default or needs Option held: [TidBITS](https://tidbits.com/2024/11/08/how-to-tame-sequoias-window-tiling/) says on, [AppleInsider](https://appleinsider.com/articles/24/06/11/tiled-window-management-comes-to-macos-sequoia-but-its-hidden) says Option. The switch is in System Settings › Desktop & Dock, under Windows. Whether it tiles a borderless window is not known.
- With no title bar there is no green button, so the easy road into a Space goes. Whether the system's Enter Full Screen in the View menu takes a borderless window into a Space is not known.
- Fuji's own fullscreen saves the window's style on the way in and restores it on the way out, read in tao 0.35.3, so a borderless window comes back borderless.

## Windows

- With decorations off, Tauri keeps a shadow by default, read in tauri-runtime-wry and tao, and the shadow keeps Windows' resize border. tao answers Windows' hit test along the top edge of a resizable window with no title bar, so all four edges resize, read in tao 0.35.3's `WM_NCHITTEST`. Not yet seen.
- Snap applies to the drag, on by default: the pointer at the top edge maximizes the window, at a side snaps it to half the screen, and at a corner to a quarter, per [Eleven Forum](https://www.elevenforum.com/t/enable-or-disable-drag-maximized-windows-in-windows-11.3503/post-68802) and [Geekrewind](https://geekrewind.com/?p=43057). Snap applies while the window keeps a resize border, which resizing needs, so the two come together.
- [Microsoft's documentation of `GetSystemMetrics`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsystemmetrics) gives `SM_CXMAXTRACK` and `SM_CYMAXTRACK` as "the default maximum width" and height "of a window that has a caption and sizing borders", which "refers to the entire desktop", and says "the user cannot drag the window frame to a size larger than these dimensions"; a window can override it by processing `WM_GETMINMAXINFO`.
- One [Windows 11 user reports](https://www.elevenforum.com/t/title-bar-can-no-longer-be-moved-off-screen.23050/) that a recent update stopped windows going above the top edge at all. Nothing official from Microsoft backs it.
- Believed and not yet seen on fuji: a window goes as far off the left, the right and the bottom as the pointer takes it, and the taskbar draws over whatever goes behind it.
- Believed and not yet seen: Windows 11 rounds the corners of a window with no title bar that keeps its shadow, and Windows 10 leaves them square.
- A maximized window has 8 of its 31 backing pixels of title bar off the top of the screen at 100 percent, measured on the Windows box, and `window_frame_set` measures the title bar the window has at that moment, so the shell un-maximizes and waits before it sets a frame.
- `isFullscreen` reports fuji's own fullscreen here, which is the platform's ordinary fullscreen, so the shell asks about a Space only on the Mac.
- None of the video window has run on Windows. The letter to the Windows box asks how far a window goes off each edge, whether Windows moves it back on release, whether the taskbar covers it, and whether a drag of its border can make it bigger than the screen.

## Linux

- The window manager runs the drag and has the last word on where a window may go. Believed and not yet seen: GNOME keeps windows below its top bar, and KDE is similar with settings of its own.
- On Wayland an application cannot place its own window, so the compositor's drag is the only drag, and method two cannot work there. Being correct on Wayland means taking what the compositor allows.
- On Wayland the platform does not say where the pointer is, which fuji's code already handles for the preview by centering it.
- tao starts the window manager's resize on a press within 5 pixels of any edge of a resizable window with no title bar, read in tao 0.35.3's Linux event loop, and its own comment says the pointer does not change to a resize arrow there.
- Believed and not yet seen: a window with no title bar has square corners and no shadow.
- None of the video window has run on Linux.

## Method two, in reserve

- The page follows the pointer and sets the window's position on every move, rather than handing the drag to the platform.
- It escapes the platform's drag rules, Snap and tiling everywhere an application may place its own window: the Mac, Windows, and Linux under X11.
- Every move becomes a crossing to Rust, which probably puts the window a frame behind the pointer.
- The page has to keep the pointer's events while the pointer is outside the window with the button down.
- It cannot work on Wayland.

## A window bigger than the screen

- On the Mac, Apple's documented rule cuts a resizable window with a title bar to the screen's height whenever it is placed or resized, which is why the big video window has no title bar.
- On Windows, the user cannot drag a window's border past the size of the whole desktop by default, from the documentation above. Whether a frame bigger than the screen that fuji sets itself holds is not known, and needs code to answer.
- On Linux, the window manager decides.
