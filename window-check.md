# Window check

## The kinds of window the user meets

- **The main window** holds the contact sheet, and the user thinks of it as fuji. It has a title bar, the user moves and resizes it freely, and fuji remembers its size. The settings panel, on s, and the audio panel, on a, take the sheet's place inside it without changing its size or its place.
- **The preview** is what a picture opened from the file manager gets: a window with no title bar and no help panel, shaped to the picture, as large as the work area allows, and never more than two CSS pixels per picture pixel for a small picture. Fuji places it in the larger part of the work area on either side of the pointer, so it lands beside where the user double-clicked rather than under it. A preview only ever comes first.
- **The light table** is never a window to the user. It always has the whole screen, in fuji's own fullscreen. The user reaches it by double-clicking a thumbnail, by clicking a preview, or on the Mac through the View menu, and a double-click on it goes back to the sheet.
- **The video window** is a window of its own, on the screen beside the main window or without one, and there can be several. For now v on the sheet opens one on top of fuji's other windows, and Esc, v or the Close beside its name closes it. It has no title bar, so its page fills it; the user drags it from anywhere and resizes it from its edges; and for now it shows the word VIDEO in its middle.
- **The audio panel and the video window** each have two states, a window and the whole screen, and a double-click trades one for the other. The audio panel's window is the main window at the main window's size; the video window is a window of its own.

## The preview

- A click on the preview opens the picture on the light table, fullscreen, with the picture starting where it stood in the preview.
- A click anywhere outside fuji closes the preview's window, and so does Esc, and Backspace on Windows and Linux. Opening the Mac's menu bar leaves the preview up, since the menu bar does not take the focus.
- Choosing Fuji › Settings… or About Fuji from the Mac's menu bar while the preview is up closes the window, seen on the Mac: the shell hides the window on its way to the main window's shape, the hide takes the focus, and the preview closes on a lost focus. The code's own comments intend the settings to open there.
- A sound or a video opened from the file manager opens the audio panel in a main window and plays, rather than a preview.
- A file with a picture's extension that fuji cannot read opens a blank window the size of a main window, with a title bar and fuji's name as its title. This is read in the code and not yet seen.
- The first main window after a preview's light table appears fresh, at the remembered size and a random place. The window leaves the screen from the preview's spot and comes back as an ordinary window, rather than growing a title bar where the preview was.

## The main window's size and place

- A new main window opens at the size the user last gave a sheet, whenever that size fits the work area, the screen less the menu bar, the dock or the taskbar.
- A remembered size that no longer fits, from a larger screen or from before the dock moved, gives way to a preset of 60 percent of the work area's width and 85 percent of its height, never wider than 16 to 9 for its height.
- Fuji puts every new main window at a random place inside the work area, so two opened together almost never land on each other. Fuji never remembers a position.
- Fuji remembers a maximized main window as maximized, and keeps the size from before it was maximized as the remembered size, so restoring a maximized window goes back to a size the user chose. On the Mac, maximized is zoomed.
- A minimized window, a window in a macOS Space, the light table, the preview and the video window never change the remembered size.
- On Windows, a sheet snapped or tiled to half the screen leaves only its size behind, and the next one comes back at that size somewhere random rather than against the edge.
- On Windows, a sheet snapped to an edge comes back from the light table at the size and place it had before the snap, no longer snapped.
- On Windows, every frame fuji measures or sets leaves out the invisible resize border Windows 10 and 11 keep around a window, so a window placed against the edge of the work area touches it as the user sees it.

## The video window

- It appears at 800 by 600 CSS pixels, which with no title bar is its page's own size, at a new random place in the work area every time, by the rule the main window is placed by. On a work area too small for 800 by 600, it takes the main window's preset instead.
- Fuji remembers nothing about it: a move or a resize lasts until the user closes it, and the next one starts fresh.
- Opening and closing a video window leaves the main window exactly where and as it was, maximized or in a Space included; the two never change each other.
- Every v on the sheet opens another one, so several can be open at once, each on its own.
- A double-click takes it into fuji's own fullscreen, and another brings it back at the same size and place.
- Esc closes it, as its red button would if it had one, and so do v and the Close beside its name. Closing it from fuji's fullscreen hides it and ends the fullscreen first.
- A drag from anywhere inside it moves the window once the pointer has moved a few pixels with the button down. A press that stays put stays a click, so a double-click still reaches fuji. On the Mac, AppKit's drag stops it at the menu bar.
- On the Mac its corners are square, since AppKit draws the rounded ones as part of a title bar's frame.
- On the Mac, Settings…, About Fuji, Open… and the View item go past a video window in front to the main window most recently in front, which comes forward first, or to a new main window when there is none. Fuji Help opens the browser from the video window itself, and the Edit menu goes gray while a video window is in front.
- v opens it only from the sheet, not from the settings panel or the audio panel.
- It reads the settings for its look and the log, and never writes them.

## Two fullscreens

- Fuji's own fullscreen fills the screen the window is on at once, with no animation and no Space. The light table always uses it, and the audio panel and the video window use it on a double-click.
- On the Mac, fuji's fullscreen hides the menu bar and the dock for the whole application rather than for one window, and they come back when it ends. Fuji closes a window in its fullscreen by hiding the window and ending the fullscreen first, so they come back for the windows left.
- The macOS fullscreen moves a window into a Space of its own, with about a second of animation, and allows Split View. The user reaches it with the green button or the system's Enter Full Screen item, which macOS puts into the View menu itself. Fuji's own item in that menu reads Show Light Table or Show Contact Sheet and switches views, on ⌃⌘F.
- The two never stack. A window already in a Space takes the light table as it is, and a double-click on the audio panel in a Space does nothing. Leaving the light table turns off only what fuji turned on. The video window has no green button.
- On Windows and Linux, fuji's fullscreen is the platform's ordinary fullscreen, and there is no second kind.
- On Windows, leaving fullscreen while the window is hidden, on the way to closing it or to a new shape, never flashes the old frame on the screen.
- Every change between a window and the whole screen happens behind a cover in the page's background color, so the user never sees a view stretched or squeezed for a frame. The cover lifts as soon as the new view is showing, before a newly chosen picture has decoded.
- After any change of fullscreen or of the title bar, the keyboard works at once, without a click first.

## How windows come to be

- On every platform, starting fuji with nothing makes one main window, and opening a picture from the file manager makes a preview.
- Fuji builds every window hidden, places it, and then shows it, so the user never sees a window jump to its place or change size as it arrives.
- On the Mac, fuji is one process with any number of windows. Opening another picture from Finder while fuji runs makes another window in the same process, and so do File › New Window on ⌘N, the dock menu's New Window, and a click on the dock icon while no window is showing. Several pictures opened from Finder together make one window.
- On the Mac with no main window open, File › Open… puts up the open box by itself and makes a window only for what the user chooses, so a cancel leaves no window. Settings… and About Fuji make a window that opens on the settings, and Fuji Help makes one and opens the help address in the browser in front of it.
- On every platform, v on the contact sheet asks for a video window, which comes from the same place every window does, as another window in the same process.
- On Windows and Linux, every open from the file manager starts a separate fuji with its own window, and the taskbar groups them under one icon. Two main windows side by side are two processes, with two Rust processes beneath them, and they share nothing but the settings file.
- On Windows, pictures selected together and opened with Enter make one window when their copies start within 200 milliseconds of each other, and a later open makes a window of its own.
- On Windows and Linux, a copy of fuji whose first window never begins to load quits after 20 seconds, so no fuji runs without a window.

## How windows go

- Esc closes the window from the light table or the preview, and Backspace does too on Windows and Linux. Esc closes a video window as well. On the Mac the delete key closes nothing.
- On Windows and Linux, closing a process's last window ends that fuji, and a video window opened from the sheet is one of its windows.
- On the Mac, closing the last window leaves fuji running with its dock icon, and a click on the icon brings a new main window. A development build run with pnpm local quits instead.

## What fuji writes down

- The settings file, fuji.toml in the home folder, holds the main window's width, its height and whether it was maximized, and nothing else about any window.
- Fuji writes the file when its last window closes, on every platform, and again at a Mac Quit or logout, which end the process without closing its windows first.
- Each window reads the whole file as it opens and hands the whole file down whenever anything in it changes, so with several windows open the changes replace each other rather than merge.
- On the Mac the windows share one process, so the window changed most recently is the one written. On Windows and Linux each process writes its own copy when its last window closes, if anything in it changed, so of two side by side the one closed last wins.

## The title bar, the menus and the keys

- The title names what the window shows: the folder on the sheet, the picture on a table, Settings or Audio on those panels, Video on a video window, and fuji's own name when there is none.
- On Windows the title reads "name - Fuji", and on the Mac and Linux it is the name alone.
- On the Mac, the Window menu lists every open window by its title, and the one menu bar serves whichever window is in front, or, for an item a video window does not answer, the main window behind it.
- The menu bar is the Mac's alone. Windows and Linux windows have no menu.
- In a release build, the web view's own browser keys, like F5 and Ctrl+R to reload, do nothing, and its right-click menu appears only in a field that takes typing.

## Linux

- On Wayland the platform does not say where the pointer is, so a preview centers in the work area.
- On Wayland the compositor decides where a window goes, so the main window's random place, the video window's and the preview's place beside the pointer may not hold there. This is assumed, not yet seen.
