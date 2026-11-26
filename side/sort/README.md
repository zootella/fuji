# Sort

Explorer's order and Finder's, written in JavaScript so Fuji can show a folder the way a user's own file manager would, on any platform, and a test that holds them against Windows and the Mac themselves.

## The sorts

Each is a plain function of two entries that compares them by name, and runs the same in every web engine.

- **`MacSort.js`**, Finder's order: CoreFoundation's comparison, which Apple publishes, written out over the engine's own collation. Exact. It lives in `desktop/src/plastic/sort`, and Fuji ships it.
- **`WindowsSort.js`**, Explorer's order without a table: Windows' rules for ASCII written out, and the engine's collation for every other character. Exact on names in ASCII, on digits and on emoji, and close for the rest. It lives in `desktop/src/plastic/sort`, and Fuji ships it.
- **`WindowsTableSort.js`**, Explorer's order built from Microsoft's own table: Windows' sort key, byte for byte, for every character. Exact everywhere. It lives here, with its table, and Fuji does not ship it.

`StrcmpSort.js`, beside Fuji's other sorts in `desktop/src/plastic/sort`, is JavaScript's own plain order. It copies no platform, so there is nothing here to hold it against.

## How Fuji uses them

`desktop/src/plastic/model.js` imports Fuji's sorts from `plastic/sort` and lists them in `modelSorts`, the orders the settings panel offers. Each is saved in `fuji.toml` by its name and shown by the word `modelSortLabels` gives it. Fuji ships the approximation because the exact sort's table is 69 KB of packed weights a reader can only trust, where the approximation's rules can be checked by eye and against any Windows machine.

To ship the exact sort instead, copy `WindowsTableSort.js` and `weight.js` into `desktop/src/plastic/sort`, and import it in `model.js` under the name `WindowsSort`, so a setting already saved keeps working. To offer it as a choice beside the approximation, import it under its own name and give it a label.

## Running the test

```
pnpm sort-test           from the side folder: every sort against what each platform was recorded saying, then against the platform this runs on, then WindowsSort.js against WindowsTableSort.js
pnpm sort-test record    what this platform says about every pair of the names in names.js, written beside the test
pnpm sort-table          remake weight.js from Microsoft's file
```

A run prints a line for each sort against each record and each pool of random names, with how many pairs it parts from the platform, or from `WindowsTableSort.js`, and lists the shortest pairs that disagree wherever a sort is meant to be exact. It fails only there. The first line of `test.js` names its other jobs: a folder of the names as empty files to look at in Explorer or Finder, the Windows shell's own folder object, and the order a real file manager window shows.

## The files here

- `test.js`, the test, and `names.js`, the names it is made of, in groups that each show one thing.
- `oracle.ps1` and `oracle.swift`, which ask Windows and the Mac for their own answers: StrCmpLogicalW, the function Explorer sorts with, and LCMapStringEx for its keys; and localizedStandardCompare, the method Apple says orders names as Finder does.
- `windows-sort.txt` and `mac-sort.txt`, what each platform said about every pair of the names. They travel, so the Mac checks the Windows sorts against what Windows said, and Windows the Mac's against what the Mac said.
- `WindowsTableSort.js`, the exact Windows sort, whose essay says how Windows' key is built and how it was held against Windows; `weight.js`, its table, packed; and `table.js`, the recipe that packs it.

## Microsoft's table

`weight.js` is Microsoft's sorting weights for Windows 11, the default table, packed into text a page can import. `table.js` makes it from the file Microsoft publishes on its Download Center, which calls itself `Unisort.txt`:

https://download.microsoft.com/download/C/F/7/CF713A5E-9FBC-4FD6-9246-275F65C0E498/Windows%2011%20Sorting%20Weight%20Table.txt

`table.js` checks the file's SHA-256, `f0bab873ed8901ae17afc4848c7d9e0a7de4310942effb2b54b538411b26d542`, the same file and hash Wine's build pins, and refuses any other bytes. The file itself is not kept here.

## What it found

- `MacSort.js` agrees with the Mac on every recorded pair and every random name it was asked about, and with the order Finder's own windows show.
- `WindowsTableSort.js` agrees with Windows on every recorded pair, and on Windows its key is Windows' own, byte for byte.
- `WindowsSort.js` agrees with Windows exactly on names in ASCII, on digits, and on emoji. Elsewhere it parts from `WindowsTableSort.js` on 0.4 to 21 percent of random pairs, measured on the Mac mini on 2026-11-25, with Chinese characters the farthest.

## What it needs

Node, and for the checks against a platform, Windows with PowerShell or a Mac with Swift. `pnpm sort-table` reaches the network once, to fetch Microsoft's file.
