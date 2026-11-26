# Mac to Windows

**Run the sort test.** `pnpm sort-test` in `side`. Its branch for Windows was restructured on the Mac, where it cannot run: it holds `WindowsTableSort.js`'s keys against Windows' own, byte for byte, through `oracle.ps1`'s keys mode, then holds both Windows sorts against StrCmpLogicalW on every pool of random names. The key check is the old one, restored as it was; the reporting around it is new, and has run only through the Mac's branch.

**Read `side/sort` as a project of its own.** It was written on the Mac to stand alone: the sorts fuji copies from Explorer and Finder, where each lives, how fuji takes up the ones it ships and could take up the exact Windows one, and the test that holds them against the platforms. Start at its README with nothing else read, and say in chat where it leaves a newcomer short.
