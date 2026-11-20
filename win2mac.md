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

## What to send back

`sort/mac-sort.txt`, whatever `oracle.swift` and `MacSort.js` needed, and the essay in `MacSort.js` and the open items in `sort.md` brought up to what the Mac showed: what agreed, what did not and why, and what the user saw in Finder.
