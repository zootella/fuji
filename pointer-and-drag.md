# Pointer and drag

What the list's pointer should be while a drag carries a selection, and how a drag reaches past fuji's window. Parked after a spike on Windows; each section goes once it is settled.

## Where the code stands

**A drag from a selected row carries the selection**, in `_listCarry` in `list.js`, and the pointer while it does is a rule in `FujiList.vue`. Nothing takes a drop.

**For the spike, the bench sets that pointer to `grabbing`, the closed hand, on every platform.** A reader of OLE's own drag pointer, which drew it out of `ole32.dll` as pixels for the page, was built and deleted, since the choices it served were set aside; it is in commit 6359fde, `copper/src/pointer.rs`, whose essay says how it got a pointer's alpha by drawing it over black and over white.

## What CSS gives

**Every pointer CSS names is one word, and on Windows they come from two places.** Windows' own: `default`, `pointer`, `text`, `wait`, `progress`, `help`, `crosshair`, `move`, `not-allowed` and `no-drop`, and the resize arrows, which look exactly like the rest of Windows. WebView2's own pictures, read out of `msedge.dll`: `copy`, an arrow with a small plus; `alias`, an arrow with a link arrow; `grab` and `grabbing`; `zoom-in` and `zoom-out`; `cell`; `col-resize` and `row-resize`; `vertical-text`. Which name maps to which picture is from memory of Chromium's Windows code. Windows' drag pointers are in neither group, so the page can only show one as an image.

## The old pointers look dated

**OLE's drag pointers and WebView2's `copy` and `alias` are two-color masks**, each pixel black, white or clear, the format of Windows 3.1 and 95 pointers, which can hold no smoothed edge and no soft shadow. Windows' arrow is a full-color image with smoothed edges and a shadow. So even loaded exactly, they look wrong beside it, coming from an older part of Windows.

## The pointer while carrying

**Two choices remain**, open:

- **Authored**, the way fuji authors its icons: a source drawn once, exported as PNGs at 32, 40, 48 and 64 pixels for 100, 125, 150 and 200 percent, shipped with the page, and named in `cursor: image-set(url(...) 1x, url(...) 1.5x, url(...) 2x) 0 0, default`. Full color with alpha, so smooth edges and a soft shadow; no Rust; and WebKit takes the same CSS. The page can also draw one on a canvas as it runs, such as an arrow with a badge counting the rows carried. A pointer larger than 32 css pixels may be refused by Chromium in some cases, from memory.
- **The plain arrow, with something floating beside it**: a count of the files carried, or a picture of them, which is what Explorer since Vista and Finder both do, Explorer adding a label like "Move to Pictures".

**Set aside:** OLE's own pointer, which looks dated; and one built from Windows' parts, the user's current arrow read from Windows, a large or black one included, with a box or a badge drawn below it. The one thing only the second could do is follow a user who chose Windows' large or black pointers, which an authored pointer does not.

## Past the window's edge

**A drag the page runs keeps its pointer outside the window, and nothing else.** The list holds the pointer with pointer capture, so the pointer and its look carry on over Explorer and the desktop and the page keeps hearing where it is. Anything the page draws, a picture of the rows following the pointer, is cut off at the window's edge, and a drop on Explorer does nothing, since Windows never heard of a drag.

**A real drag belongs to Windows, and has to be started from Rust.** Windows draws the picture of what's carried in a floating window of its own, so it follows the pointer anywhere, and Explorer takes the drop. Rust starts it with `DoDragDrop`, handing in the files and a picture; from then on Windows owns the pointer and the picture, inside fuji too, and fuji's own targets, a subfolder row or the tree, would hear of it through drop events like the ones `Shell.vue` already listens to. The browser's own drag and drop is a real Windows drag as well, but it cannot carry files to Explorer, and Tauri's file-drop handling, which fuji uses, is believed to turn it off on Windows.

**The likely answer is a handoff**: inside the window the page runs the drag, with fuji's pointer and a picture it draws, and the moment the pointer crosses the window's edge, Rust starts the real drag with the same picture. Unchecked: whether `DoDragDrop` can start with the button already down and look seamless, which is believed to be how it is normally called; and whether Explorer's plain arrow and label come from the shell's drag image helper, `IDragSourceHelper`, which decides what fuji would hand Windows. CrabNebula's `drag` crate starts a real drag on all three platforms, and is weighed against writing each in copper and brass.

## The other platforms

**The Mac** keeps its plain arrow during a drag, with the dragged items' picture following it, and adds a badge over a target: a green plus for a copy, a curved arrow for an alias, the not-allowed pointer where a drop is refused. WebKit is believed to map `copy`, `alias` and `no-drop` to exactly those AppKit pointers, so the Mac would need no Rust for its badges. **Linux** has no one drag pointer; GTK draws pointers by name from the user's cursor theme, and WebKitGTK maps the same keywords to that theme. All of this is from memory.
