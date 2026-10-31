# Mac to Windows

## What the Mac is working on

**The list and the mock menu, side by side**, toward a navigational file manager: `FujiBar` across the top, `FujiTree` on the left, `FujiList` on the right.

- `FujiList` is drawn in plain page elements by `desktop/src/list.js`, beneath the thin `FujiList.vue`, with `selection.js` and `keyboard.js` beside it. Nothing in fuji places it yet but the bench, `FujiUserInterfaceSpike.vue`, which `u` opens from the sheet.
- The mock context menu is `MockMenu.vue`, opened by a right click on the contact sheet, in fuji's look. Its items do nothing yet.

**Read these first**, for this handoff only:

- `components.md`, the plan for the list, the tree and the bar.
- `components-shelf.html`, the survey of existing components, and the keyboard and mouse table compiled from Apple, Microsoft, VS Code and the ARIA guide.
- `file-managers-history-article.md`, the scope of a future article on file managers as products: orthodox, navigational, spatial and the rest. It is written from memory, nothing in it is checked, and it is not to be fleshed out yet.
