# Bookmark

Where the FujiBar work stands, 2026-11-19, for the next session, and after it where the hunt for the drawings' colors stands, which is what the next session picks up. The decisions are in the documents and the code; this file holds what is open and what was only suggested.

## Where to read

- **The bar.** `bars.txt` is the text, `bar.js` reads it and its essay says how it is written, and `FujiBar.vue` draws it, with an essay on menus and panels. The bench shows it: `u` from the sheet opens `FujiUserInterfaceSpike.vue`, with a menubar and a toolbar across the top, a Bench button that shows the bench's own strip, and a Views button that opens the first panel.
- **The decisions.** components.md's FujiBar and Icons sections, `fuji-bar-dsl.md`'s section The syntax, architecture.md on when a component stays in Vue, `tabs.md`, and `theme.md`'s notes Not Zed all the way.
- **The palettes.** `pnpm color` in `desktop` opens the color page, at `http://localhost:1430/color/`. Its Button Color tab is a survey of 82 palettes other people made, each whole, with who made it and what for, and FujiBar's own toolbar and the edit menu in its colors, answering the pointer as they do in fuji; its Crayon Boxes tab picks a sample box from each. All in light, since the drawings are monochrome in dark. The crayon hunt, below, says the rest.
- **The screenshots.** The user's green guides, Windows Explorer and Office 2003 on Windows XP in its classic look, are on the Windows box in `C:\Documents\with-claude\fuji-bar-guides`, a folder the user has opened to Claude, unlike the rest of the computer. Four other pictures there, a Chrome menu, a glass menu, a Windows 10 menu and list, and balloons, have not been looked at yet.

## Open, by name

- **"Light title bar."** On Windows the bars take the title bar's color. Dark was matched by eye, black in front and `#2b2b2b` behind; light is still a guess of white in both, unchecked.
- **"Menu row height."** The rows come out at fractions of a pixel, L + 10 being about 25.96 with Segoe UI at 12, since L is the font's own. Claude suggested pinning them to whole multiples of 4, 24 for the menubar, as `style.css` does the list's rows; no decision.
- **"Hot look."** Settled on 2026-11-19: a menu's row under the pointer is solid accent on the Mac, as a list's selected row is there, and on Windows and Linux white in light and black in dark with a gray line around it, in place of Office 2003's accent outline over the accent at 30 percent. components.md's highlight colors say the rest.
- **"Keys."** Menus take no keys yet: no Escape, no arrows, no Alt or F10, no underlined letters. The shell's own bare keys, h, g, v, t and u, still act while a menu is open, and an Escape while the list has the keyboard goes to the list rather than the menu, so the keys need a decision in the shell. `fuji-bar-dsl.md`'s Access keys and Menu engineering sections hold the research.
- **"Not there yet."** Shortcuts beside a menu's rows, whose syntax is still to come; submenus opening from a menu's row; split buttons; a menu that would run past the window's right edge; and buttons with a drawing and no words.
- **"MockMenu."** It still draws the right-click menus on the sheet, the light table and the bench's list. Claude suggested moving them to FujiBar's `menu` kind, opened from code as `barShow` in `fuji-bar-dsl.md` sketches, and then removing MockMenu.
- **"Menubar off the Mac."** `menu.md` says the main window gets a toolbar on every platform, not a menu. The bench shows a menubar on every platform while the bars are designed, and leaves it out on the Mac at the end. Whether Windows and Linux get a menubar in the real window, and whether it takes Alt, was asked and not answered.
- **"Palette."** Nothing is chosen, for light alone: on 2026-11-19 the user decided the drawings are in color in light and monochrome in dark, components.md's Color in light, monochrome in dark. The crayon hunt, below, is this item grown. Claude suggested checking Delete's red beside Undo's green with a colorblindness simulator, since red and green are the pair most often confused.

## The crayon hunt, 2026-11-19

What the user wants, in the user's words: a box of crayons for the bar's drawings, bright colors, strong, numerous, distinct, classroom fun. Several reds, oranges, greens, yellows, blues, purples and browns, not the same number of each, chosen so any few can be taken out and drawn beside each other and told apart, so that the choosing is done once and coloring a drawing is picking the second blue crayon. The hope was to find such a box already made, by a team that spent years on it, and inherit it. After two rounds of research the user's verdict was that there is no obvious choice: some boxes are too small, some too light, some too sparse.

**Where to read.**

- `desktop/color/upstream.js` holds the 82 palettes in eight groups, each with who made it, what people take it for, its license and where its values were read, and says where a value is less than certain. It is the one file on the color page that holds colors of its own. It was put together by scripts that are gone, and is edited by hand from here.
- `desktop/color/crayon.js` picks a box from a palette by measurement, and its essay says how; its limits are six numbers at the top.
- `ColorButtons.vue` draws the survey and `ColorCrayons.vue` the boxes, with a table at its top of every box by size and family.
- components.md's The drawings' palette and A box, not a grid hold what is decided.
- The seven research reports behind the survey are in the transcript of the session of 2026-11-19 and nowhere else. The page's words about each palette are their summary; the reports also hold values the page does not draw, Apple's colors for dark, all of Fluent's ramps, Gmail's grid, and the old guidelines' own words on outlines.

**What the two rounds found.**

- A grid is not a box. Most palettes are a few hues times a ladder of lightness, where nobody checked one hue's step against its neighbor's. One row across a wide grid is a box, and the grids that hold every hue to one contrast, Carbon, Spectrum and Primer, make the worst ones, since along a row only hue is left to tell two crayons apart.
- The first round, interface palettes, chart palettes and editor themes, was the wrong water: each is sober, small or pastel on purpose.
- The second round came closer. Crayola is the right spirit, with the most browns, and a third of it is too pale for a thin line on white. Google Calendar's 24 was made for a small mark on white picked from a row of others. Pixel artists' palettes are ramps for shading, half darks and pales, with bright middles; Famicube and Resurrect 64 gave the fullest boxes, and Resurrect's the most even. KDE's Oxygen is the richest palette written down for icons, under a license that forbids commercial use. Windows XP's icon colors are the bar's own era, and are seven blues with no purple, pink or teal.
- Every old guideline for small icons paired its palette with an outline: black on the Mac, black below and right on Windows, a darker shade of the fill later. Lucide's drawings are a line and nothing else, which is much of why every palette looks weaker on the bar than in its swatches.

**Open, by name.**

- **"The box."** Nothing is chosen. Claude suggested giving up on a finished box and building fuji's own on the color page from the best parts, starting from Resurrect 64's or Famicube's box for brightness and spread, borrowing browns from Crayola or Oxygen, and checking each addition by the same measurements. Not answered.
- **"Line or fill."** Under each box are four rows, the drawings as lines at 24 and 16 css pixels and then again with a quarter of the crayon's color filled inside, a trial. Claude suggested settling how a drawing carries color, a fill or a darker outline, before choosing crayons, since it changes which ones are strong enough. Not answered, and a fill across an open shape like an arrow's head has not been looked at closely.
- **"The rule."** The user said not every pair in a box has to be distinct, only that any few taken out are. `crayon.js` holds every pair at least 0.09 apart in OKLab, which is stricter. Its limits, a floor of 1.6 to 1 against the ground, the gap from the text's color, the least chroma, the lean toward strong colors and a box of at most 24, are Claude's and have not been reviewed, and the hue ranges that name a crayon's family are rough. It passes over a palette's best known red for a neighbor a hair farther from everything else, so every box is a first pass for a person to change.
- **"Too light."** The floor lets in crayons under the 3 to 1 the accessibility guidelines ask of a drawing, so a box can have a yellow, and marks them pale. Whether fuji draws anything that pale is not decided.
- **"Names and licenses."** Crayola's, LEGO's and Transport for London's names are trademarks, fine on the color page and not as fuji's own names for colors; pixel artists' colors have no names, only places in a list. Oxygen's license forbids commercial use and changes. Apple asks apps not to write its system colors down. Fuji's box will want names of its own.
- **"Still to read."** Windows 11's grid of 48 accents, off the Windows 11 box. The Mac's accent and Finder's tag colors, on the Mac mini, from AppKit. Apple's icon colors for System 7 are a reconstruction from a figure, to be checked in ResEdit, and one of Apple's books says 36 where the guidelines say 34. JetBrains' repository's license was from memory. Not searched at all: famfamfam's Silk icons, Fugue, NetBeans' guidelines, and any study of Office 2003's own toolbar icons.
- **"Not looked at."** Claude saw both tabs only through a headless browser, the top of each and one box's rows, and counted the rest. Nobody has scrolled all 82. The Button Color tab holds 82 live toolbars and about 2,500 crayons, and whether it is too heavy has not been asked.

## Seen in the screenshots, not asked for

- **Faded at rest.** Office 2003 draws its icons lighter at rest, outlines `#4d4d4d` rather than black, as if 30 percent toward white, and at full strength only under the pointer.
- **Dropped on the press.** A lifted command could drop back without its shadow as it is pressed, so the strike shows.

## Suggested for the tabs, not written into tabs.md

- **The video window's tabs** could be a fifth kind in the text, `tabs`, each tab naming the toolbar or panel it shows, and the browser-like tabs of the main window a control of their own, built from data the way the list is, since the user opens and closes them.
- **A palette beside a vertical video** runs down, while the kinds fix a toolbar as running across, so where a toolbar runs may be the page's choice of where it places it rather than the text's.
