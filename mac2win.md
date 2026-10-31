# Mac to Windows

## What the Mac is working on

**The list and the mock menu, side by side**, toward a navigational file manager: `FujiBar` across the top, `FujiTree` on the left, `FujiList` on the right.

- `FujiList` is drawn in plain page elements by `desktop/src/list.js`, beneath the thin `FujiList.vue`, with `selection.js` and `keyboard.js` beside it. Nothing in fuji places it yet but the bench, `FujiUserInterfaceSpike.vue`, which `u` opens from the sheet.
- The mock context menu is `MockMenu.vue`, opened by a right click on the contact sheet, in fuji's look. Its items do nothing yet.

**Read these first**, for this handoff only:

- `components.md`, the plan for the list, the tree and the bar.
- `components-shelf.html`, the survey of existing components, and the keyboard and mouse table compiled from Apple, Microsoft, VS Code and the ARIA guide.
- `file-managers-history-article.md`, the scope of a future article on file managers as products: orthodox, navigational, spatial and the rest. It is written from memory, nothing in it is checked, and it is not to be fleshed out yet.

## The question for Windows: which pointer shows during a drag

**Why it came up.** A press on a file's name in the list is kept for dragging the file, to a subfolder, to the tree, or out to Explorer, and a press anywhere else in the row drags a selection; `_listPress` in `list.js` makes that choice. The file drag is not built. The user asked whether a standard pointer says a drag is holding something.

**What the Mac session said, from memory and unchecked.** The Mac keeps the plain arrow during a drag, with a picture of the dragged items following it and a badge on the arrow over a target. Windows' drag and drop, OLE, has pointers of its own: an arrow with a small dotted box hanging off it for a move, the same with a plus for a copy, with a curved arrow for a link, and a slashed circle where a drop is refused. The user remembers the boxed arrow from drag and drop editing in Word for Windows 2.0, from 1991. Explorer since Vista shows the plain arrow, a drag image and a label like "Move to Pictures", because it draws through the shell's drag helper, while a program that takes OLE's default pointers would still get the boxed arrow.

**What to find out.** Whether Windows 10 still shows the boxed arrow, and for which programs; dragging selected text in WordPad and dragging a file in Explorer was the comparison suggested. And the mechanism behind it: where the pointers come from, believed to be resources in OLE rather than anything in the Mouse settings, and what decides between the boxed arrow and the plain one. The mechanism matters because a drag out of fuji would come from Rust through OLE, so which pointer it shows depends on whether fuji hands Windows a drag image through the shell's helper. A page cannot ask for the boxed arrow, since CSS has no cursor name for it.
