# From Windows to the Mac: the two platform sorts, and the one that has never met a Mac

Written on the Windows 11 box, 2026-11-20. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one.

## What changed

Fuji has two new orders for a folder's names beside `StrcmpSort`, the plain order by the characters' own numbers that was called Alphabet: `WindowsSort`, the order File Explorer shows a folder sorted by name, and `MacSort`, the order Finder does. Both are offered on every platform, and both are the page's own code, on purpose; the essay above `modelSorts` in `desktop/src/model.js` says why fuji does not ask each system's own function. The setting is `order` under `[sort]` in `fuji.toml`, and the settings panel offers the three under Sort, below the fits, as Simple lexicographic, Windows and Mac. The table shows a folder in the chosen order; the contact sheet's pager does not follow it, on purpose, being a test bed kept apart from the sorts.

- `desktop/src/WindowsSort.js` builds the sort key Windows builds, from Microsoft's published table for Windows 11, packed in `WindowsSortTable.js`. It was held against Windows itself here and agrees; its essay has the numbers and the three corners where it stops.
- `desktop/src/MacSort.js` is CoreFoundation's `CFStringCompareWithOptionsAndLocale` written out, under the options `localizedStandardCompare:` passes, over the page's own `Intl.Collator`. **It was written from the open source in swift-corelibs-foundation and has never run beside a Mac.** It reproduces the Finder orders other people have published, and that is all that is known.
- `desktop/sort` is the test: `names.js`, twenty-one groups of names that each show one thing a shell does; `oracle.ps1` and `oracle.swift`, which ask each platform; `test.js`, which runs them; and `windows-sort.txt`, what Windows said about every pair of the names. `pnpm sort-test` in `desktop` runs `test.js`, with the same words after it.
- `sort.md` at the root has what the research found about both shells.

## What only the Mac can do

Everything about `MacSort`. In `desktop`, after `pnpm install` at the root:

1. **`swift sort/oracle.swift`** says which macOS this is and which language the comparison follows. `oracle.swift` has never been compiled, so the first job may be making it run. The language matters: `MacSort.js` is the order for English, and if the Mac's first language is anything else, say so before reading any result.
2. **`pnpm sort-test record`** asks `localizedStandardCompare:` about every pair of the names, 95,703 of them, and writes `sort/mac-sort.txt`, which is tracked, so Windows can then check `MacSort.js` against the Mac from here on.
3. **`pnpm sort-test`** checks both sorts against both records, then asks the Mac about random names in four mixes: ASCII, digits, accented Latin, and composed against decomposed. The Windows record should pass on the Mac as it does here; that half is a check of nothing but the code travelling.
4. **`pnpm sort-test folder ~/Desktop/sort-test`** makes the names into empty files, a folder a group, with `fuji-sort.txt` beside them: the order fuji expects Finder to show in each. The folder is made there from `names.js` and was never carried over, since a zip would lose or change the very names it is for. Then the user looks, in Finder, in list view sorted by name.
5. **A Finder window read back, if it can be.** On Windows `pnpm sort-test explorer` and the folder's path opens each group in a real Explorer window, sorts it by name, and reads the order the window shows, which matched fuji's in every group but 13, where Windows contradicts itself; `explorer` in `test.js` and the explorer mode of `oracle.ps1` are how. The same for Finder, through AppleScript or whatever Finder answers to, would turn the user's looking into a spot check, and is worth trying before asking them to compare twenty-one folders by eye.

## What to expect to be wrong, and where to look

- **Node's ICU is not the Mac's.** In fuji, the page's `Intl.Collator` on the Mac is the system's ICU, the one Foundation uses, so there the two should agree on every character. Node carries its own, usually newer. So from node, only the mixes every ICU agrees on are a fair test of `MacSort.js`, which is why step 3 runs four and not eleven, and why a disagreement in the record over Chinese characters, emoji or a character newer than macOS's Unicode is likely the two ICUs and not the code. A disagreement over ASCII or accented Latin is the code.
- **The 64 bits.** CoreFoundation's source keeps a number in a `uint64_t` with no check, and `MacSort.js` follows it, so `x18446744073709551616` sorts as `x0`. Whether the Mac that ships still does this is not known. Group 04, long numbers, is the test.
- **The folding.** Where two names differ and yet fold to the same thing, CoreFoundation walks on, and `_macFold` stands in for its folding tables with the language's own `normalize` and case mapping. A mistake there shows only as a tie between leading zeros going the wrong way after such a character, like `É1` against `é01`, which the source says puts `é01` first.
- **Case against width.** CoreFoundation's last ICU pass turns the case level on, which `Intl.Collator` has no switch for. Names that differ in case at one place and in width or kana at another may come out the other way round. Known, and written in the essay; say how often it shows.
- **Whether Finder calls this method at all.** Apple documents `localizedStandardCompare:` as sorting "as the Finder" does, and nothing found says Finder itself calls it on current macOS. Only eyes on Finder beside `fuji-sort.txt` answer that.
- **The name Finder sorts.** With extensions hidden, does Finder order the name it shows or the name on disk? Group 10, copies and extensions, looked at both ways.
- **List sort order.** System Settings has a list sort order under Language & Region for some languages. If it is set to anything but the default, Finder follows it and fuji does not.

## What is not the Mac's to settle

`WindowsSort.js` is settled on Windows; a failure of the Windows record on the Mac means the code depends on something it should not, and is worth a look and a word, not a change to make it pass. The choice of English for both sorts, and whether either should follow the user's language, is the user's and open in `sort.md`.

## A second errand: the scrollbar's colors

Apart from the sorts. On Windows the web engine paints a scrollbar's track in a near white of its own, which is not one of fuji's colors and shows as a strip down the list. Measured on the Windows 11 box on 2026-11-20, in a screenshot of fuji and again in Edge run without a window, the same engine as the web view at the same version, 157.0.4341.60:

- **As the engine draws it.** In light the track is `#fcfcfc` and the thumb `#8b8b8b`; in dark the track is `#2c2c2c`, which happens to be fuji's dark background exactly, and the thumb `#9f9f9f`. The bar is 15 css pixels wide and the thumb 9, with an arrow at each end.
- **`scrollbar-color` changes the two colors and nothing else.** With a thumb color and a track color set, the bar keeps its width, its thumb's width and rounded ends, and its arrows, which take the thumb's color; `transparent` for the track shows the ground through it. `scrollbar-width: thin` makes the bar 10 wide and the thumb 6. Styling `::-webkit-scrollbar-track` alone changes nothing.
- **Seen in fuji by the user, the same day.** With the thumb set to red and the track to green for the look, `scrollbar-color: #ff0000 #00ff00` on the root in `style.css`, the thumb is red and the track green in `pnpm local`, and the red goes darker while it is pressed, the thumb and the arrows alike, so the engine works out a pressed shade from the color it is given and fuji need not supply one; and the same red and green in dark.
- **Under the pointer, measured.** In the same windowless Edge, with the pointer moved and pressed over its debugging port. The thumb changes only while the pointer is on the thumb itself, not over the list or the track, and never changes width; pressed, it is the same shade as under the pointer. Left alone, the engine's thumb goes darker in light, `#8b8b8b` to `#636363`, and lighter in dark, `#9f9f9f` to `#d1d1d1`. Given a color, it always goes darker, to seven tenths of it, whatever the mode: red `#ff0000` to `#b30000`, `#8b8b8b` to `#616161`, and in a dark box `#9f9f9f` to `#727272`. So in light a thumb fuji sets to the engine's own `#8b8b8b` behaves as the engine's does to within two steps, and in dark a thumb fuji sets would dim under the pointer where the engine's own brightens.
- **The corner.** The square where a bar down the side meets one across the bottom takes the track's color.
- **Decided by the user, the same day:** fuji sets the colors in both modes, so the track is fuji's and does not move when the engine's does, and accepts that in dark its thumb dims under the pointer where the engine's own brightens.

So on Windows fuji paints the track in its own ground with one line of CSS, and that line is written: `scrollbar-color: var(--color-scrollbar) transparent` on the root in `style.css`, the thumb fuji's gray and the track showing the face through it. The same day the user made the face one color under everything, `#fcfcfc` in light and `#2c2c2c` in dark, the engine's own track colors here; components.md's One face and the essay in `style.css` say the rest.

**No test comes over with this letter.** The red and green were taken out again on Windows once they had been seen, so `style.css` holds only the real line. The Mac makes its own, first, before anything else here: in place of the real line, colors nobody could miss, like `scrollbar-color: #ff0000 #00ff00`, to see at a glance in `pnpm local` whether this engine takes the two colors at all. Then the real line goes back, and the garish one is not left behind.

What the Mac can say, on the Mac mini, in fuji itself and not in Safari, since the web view is the system's WebKit:

1. **Whether the engine takes the colors, with garish ones first.** With the user, the real line swapped for red and green as above, `pnpm local`, and a look at the list, the tree and the contact sheet: whether the thumb is red and the track green, in light and in dark, and a screenshot either way. Then the real line put back, and a look at fuji's own gray thumb on the Mac, in light and in dark.
2. **What a scrollbar looks like with nothing set**, the real line out for the moment. A Mac with a mouse attached, or with Show scroll bars set to Always in System Settings, draws a bar with a track that takes room; with a trackpad it draws a thumb that floats over the content and no track at all. Say which this Mac shows, and try the other setting. Where there is a track, its color and its edge line in light and in dark, read from a screenshot.
3. **Whether this WebKit follows `scrollbar-color`.** Support came to Safari late, so it depends on the macOS and Safari on the machine; say the versions. If step 1 showed no red and green, that is likely why. If it did, whether setting the colors changes the bar's shape or turns a floating thumb into one with a track, seen under both settings of step 2.
4. **Whether fuji's ground and the Mac's track meet.** If the track is a gray of the Mac's own with a line down its side, as memory has it, painting it in fuji's ground may look less like a Mac than leaving it. That is the user's call, with the screenshot in hand.

## A third errand: the face, in light and in dark

The same day everything in fuji took one face, `background` in `style.css`: the bars, a menu, the list, the tree and the sheet, `#fcfcfc` in light and `#2c2c2c` in dark. components.md's One face says why. The user looked at the built fuji on Windows 11 beside File Explorer, in both modes, and the takeaway, in the user's words: light looks right, dark looks charcoal.

What was measured there, from the two screenshots:

- **Light.** Fuji's face `#fcfcfc`; Explorer's list, sidebar and command bar `#ffffff`. Three steps of 255 apart.
- **Dark.** Fuji's face `#2c2c2c`; Explorer's list, sidebar and command bar `#191919`, nineteen steps darker. The title bar Windows draws over fuji read `#221f1f`, so there is a step down from it to fuji's bar.
- **The dark face is Zed's.** One Dark's editor background, `#282c33`, with its tint out, is `#2c2c2c`, and One Dark is a soft charcoal on purpose. The other dark grays, the border, the surface, the quieter texts, a selection's grays, were each taken for how far they stand from that ground.

The knee-jerk answer is a darker dark face. **The user wants to see fuji as it is on the Mac first, in light and in dark, before anything moves.** So on the Mac, with the user:

1. **Fuji beside Finder, in light and in dark,** a screenshot of each with a Finder window in list view next to fuji's list. Read from them: Finder's list ground, its sidebar, and its toolbar and title bar; and fuji's face and the title bar macOS draws over it, which fuji does not paint.
2. **Whether the Mac's title bar and fuji's bar under it meet or step,** in each mode, as they step on Windows in dark.
3. **What the user says on looking,** in the user's own words, for light and for dark.

Nothing here is the Mac's to change. If dark goes darker, two ways were talked through on Windows and neither chosen: move the whole dark set down together, keeping each gray's distance from the ground, so it stays Zed's balance at a darker level; or take the dark ground from a platform, Explorer's `#191919` or whatever Finder's turns out to be, as light took its ground from Windows, and check each dark gray against it again. A scrollbar's track is transparent and follows either.

## What to send back

`sort/mac-sort.txt`, whatever `oracle.swift` and `MacSort.js` needed, and the essay in `MacSort.js` and the open items in `sort.md` brought up to what the Mac showed: what agreed, what did not and why, and what the user saw in Finder. For the scrollbars, the screenshots and the four answers above. For the face, the two screenshots beside Finder, what they measure, and what the user said.
