# Bookmark

Where the FujiBar work stands, 2026-11-16, for the next session. The decisions are in the documents and the code; this file holds what is open and what was only suggested.

## Where to read

- **The bar.** `bars.txt` is the text, `bar.js` reads it and its essay says how it is written, and `FujiBar.vue` draws it, with an essay on menus and panels. The bench shows it: `u` from the sheet opens `FujiUserInterfaceSpike.vue`, with a menubar and a toolbar across the top, a Bench button that shows the bench's own strip, and a Views button that opens the first panel.
- **The decisions.** components.md's FujiBar and Icons sections, `fuji-bar-dsl.md`'s section The syntax, architecture.md on when a component stays in Vue, `tabs.md`, and `theme.md`'s notes Not Zed all the way.
- **The palettes.** `pnpm color` in `desktop` opens the color page, whose Button Color tab lays out each candidate palette as a box of crayons, with FujiBar's own toolbar and the edit menu in its colors, answering the pointer as they do in fuji.
- **The screenshots.** The user's green guides, Windows Explorer and Office 2003 on Windows XP in its classic look, are on the Windows box in `C:\Documents\with-claude\fuji-bar-guides`, a folder the user has opened to Claude, unlike the rest of the computer. Four other pictures there, a Chrome menu, a glass menu, a Windows 10 menu and list, and balloons, have not been looked at yet.

## Open, by name

- **"Light title bar."** On Windows the bars take the title bar's color. Dark was matched by eye, black in front and `#2b2b2b` behind; light is still a guess of white in both, unchecked.
- **"Menu row height."** The rows come out at fractions of a pixel, L + 10 being about 25.96 with Segoe UI at 12, since L is the font's own. Claude suggested pinning them to whole multiples of 4, 24 for the menubar, as `style.css` does the list's rows; no decision.
- **"Hot look."** A menu's row under the pointer takes Office 2003's look, an accent outline over the accent at 30 percent, with the words in their own color, while a list's selected row is solid accent with white text. Whether the two should match is not decided.
- **"Keys."** Menus take no keys yet: no Escape, no arrows, no Alt or F10, no underlined letters. The shell's own bare keys, h, g, v, t and u, still act while a menu is open, and an Escape while the list has the keyboard goes to the list rather than the menu, so the keys need a decision in the shell. `fuji-bar-dsl.md`'s Access keys and Menu engineering sections hold the research.
- **"Not there yet."** Shortcuts beside a menu's rows, whose syntax is still to come; submenus opening from a menu's row; split buttons; a menu that would run past the window's right edge; and buttons with a drawing and no words.
- **"MockMenu."** It still draws the right-click menus on the sheet, the light table and the bench's list. Claude suggested moving them to FujiBar's `menu` kind, opened from code as `barShow` in `fuji-bar-dsl.md` sketches, and then removing MockMenu.
- **"Menubar off the Mac."** `menu.md` says the main window gets a toolbar on every platform, not a menu. The bench shows a menubar on every platform while the bars are designed, and leaves it out on the Mac at the end. Whether Windows and Linux get a menubar in the real window, and whether it takes Alt, was asked and not answered.
- **"Palette."** Radix may be the best and nothing is chosen. Claude suggested checking Delete's red beside Undo's green with a colorblindness simulator, since red and green are the pair most often confused.

## Seen in the screenshots, not asked for

- **Faded at rest.** Office 2003 draws its icons lighter at rest, outlines `#4d4d4d` rather than black, as if 30 percent toward white, and at full strength only under the pointer.
- **Dropped on the press.** A lifted command could drop back without its shadow as it is pressed, so the strike shows.

## Suggested for the tabs, not written into tabs.md

- **The video window's tabs** could be a fifth kind in the text, `tabs`, each tab naming the toolbar or panel it shows, and the browser-like tabs of the main window a control of their own, built from data the way the list is, since the user opens and closes them.
- **A palette beside a vertical video** runs down, while the kinds fix a toolbar as running across, so where a toolbar runs may be the page's choice of where it places it rather than the text's.
