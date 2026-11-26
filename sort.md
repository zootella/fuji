# Sorts

The order fuji shows a folder in. One value, chosen in the sheet, used by every table — so it lives in the model, below both of them, rather than in either.

This file has two halves. The first is prior art: how Windows and macOS actually sort and date files, researched rather than remembered, because fuji is going to reproduce some of it and deliberately depart from the rest. The second is the plan. `StrcmpSort`, `WindowsSort` and `MacSort` are written; the other five are not.

---

# Part one: prior art

## How the two shells sort names

Both shells abandoned plain character-code ordering a long time ago, and both landed on the same idea — a run of digits is a number, not a sequence of characters — by different routes, in different functions, with different results.

### Windows: `StrCmpLogicalW`

File Explorer sorts with `StrCmpLogicalW` from `shlwapi.dll`. Microsoft's description is one sentence: *"Compares two Unicode strings. Digits in the strings are considered as numerical content rather than text. This test is not case-sensitive."*

The documentation gives the ordering it produces, and it is worth reading closely because it shows the rule applies to digit runs anywhere in the name, not just at the end:

    2string      3string      20string
    st2ring      st3ring      st20ring
    string2      string3      string20

The same strings under `StrCmpI`, which treats digits as text, come out `20string, 2string, 3string, st20ring, st2ring, st3ring, string2, string20, string3`.

Facts worth carrying:

- **It arrived after Windows 2000.** Windows XP and Server 2003 onward sort this way; the older lexicographic order is what anyone's memory of Windows 98 is made of.
- **ASCII digits only, once.** Before Windows 7 it recognised `0`–`9` and no other digits. Today the digits of every script count, each run in its own script's digits; the section on what was measured below has it.
- **It can be turned off.** A Group Policy restriction (`REST_NOSTRCMPLOGICAL`) reverts Explorer to the old ordering, so "how Windows sorts" is not even uniform across Windows machines.
- **Microsoft says not to rely on it.** The reference page carries this note: *"Behavior of this function, and therefore the results it returns, can change from release to release. It should not be used for canonical sorting applications."* That is Microsoft telling us that a fuji sort named after Explorer is aiming at a target that is allowed to move.

### macOS: `localizedStandardCompare:`

Finder has sorted with `-[NSString localizedStandardCompare:]` since Snow Leopard, and Apple exposes the same call so third-party code can match. Apple documents the rule as the Unicode Collation Algorithm (UTS #10) with exactly three modifications:

1. Punctuation and symbols are significant for sorting.
2. Substrings of digits are sorted by numeric value rather than by their characters.
3. Case is not considered.

Facts worth carrying:

- **It is localized, and that is not cosmetic.** The collation follows the user's language. In Danish, `Ø` is a letter after `Z`, not a decorated `O`, so the same folder genuinely sorts differently for two users on the same machine. There is no single "Mac order."
- **Diacritics usually fold to their base letter**, but where they do not is exactly the locale question above.
- **Normalization does not leak through.** `é` written as one code point and as `e` plus a combining accent compare equal, so a file made on one Mac and one from an old HFS+ volume sort together instead of splitting.

### JavaScript's own order

`Array.prototype.sort()` with no comparator converts each element to a string and compares UTF-16 code units. That gives two behaviours users notice: every uppercase letter sorts before every lowercase one, because `Z` is 90 and `a` is 97; and `page10` sorts before `page9`, because the comparison stops at `1` versus `9`.

This is what fuji ships as `StrcmpSort`, on purpose: the comparison is ECMAScript's, and the numbers compared are Unicode's and, for an English keyboard, ASCII's. Computing calls it lexicographic order, and C's `strcmp` is its oldest famous name.

In the page, the closest thing to Finder is `Intl.Collator` with `numeric: true` and a base sensitivity. It is numeric-aware, case-insensitive and diacritic-insensitive, and it follows a locale. It is close to `localizedStandardCompare:` and is not the same function.

### Where they disagree, and the leading-zero hole

Both shells compare digit runs by value. Neither documents what happens when two runs have the *same* value but different spellings — `01` and `1`, `007` and `7`. The numeric comparison is a tie, and every implementation invents its own tiebreak: shorter first, longer first, fall back to character comparison, or leave the order to whatever the sort algorithm happened to do. This is the seam where "natural sort" implementations visibly disagree with each other and with the shell they were trying to copy.

It is also the case the user is most likely to have in a folder of scanned pages, where `page1`, `page01`, `page02` and `page2` sit together.

### What was measured, November 2026

The two functions were read and run rather than taken from their documentation, on Windows 11 and from CoreFoundation's published source, and both turned out to be something other than the natural sort the documentation suggests.

**`StrCmpLogicalW` is one call to Windows' linguistic comparison.** On Windows 7 and later it is `CompareStringEx` in the user's locale with two flags, ignore case and digits as numbers, and nothing else: three hundred thousand random pairs through both agreed, and the function's disassembly shows the one call. So Explorer's order is Windows' collation, a table of four weights for every character which Microsoft publishes for each version of Windows, and what follows is that table's doing:

- **Symbols, then digits, then letters**, the symbols in an order of Windows' own, with `+ < = >` after `~`.
- **The hyphen and the apostrophe are set aside**, and only break a tie: `co-op` sorts beside `coop`, after it. Microsoft calls this a word sort.
- **Leading zeros decide on the spot, more zeros first.** `001`, `01`, `1`, `02`, `2`, and `file01b` before `file1a`.
- **A number is at most 19 digits.** A longer run is a second number after the first, so twenty nines sort after a one and twenty zeros.
- **Some characters are the same as others, or as nothing.** `a` and `A`, `ß` and `ss`, `æ` and `ae`, `é` and `e` with a combining accent; a soft hyphen and a zero width joiner weigh nothing. The shell breaks those ties by the characters' own numbers with case ignored.
- **The period before an extension is one more symbol**, and the whole name is compared, extension included.
- **It follows the user's locale.** A Swedish Windows puts `å` after `z`.

**Windows 10 and Windows 11 differ, outside ASCII.** Microsoft's two tables, NLS versions 0x00060305 and 0x00060502, give every ASCII character the same weights. They differ in Georgian, in the accent weights of voiced kana, and in 133 characters the newer table adds.

**`localizedStandardCompare:` is a walk of CoreFoundation's own and then ICU.** It walks both names while they are the same, reading runs of ASCII digits itself, and hands what is left at the first difference to ICU's collation for the user's language:

- **Leading zeros only break a tie, shorter first.** `1`, `01`, `001`, `2`, and `file1a` before `file01b`: the opposite of Explorer twice over.
- **A number is kept in 64 bits**, so one past 18,446,744,073,709,551,615 wraps around.
- **Every symbol has a place**, in CLDR's order, the hyphen and the apostrophe among them.
- **Nothing is the same as anything else.** Small letters before capitals, and `ss` before `ß`, once all else is equal.
- **It follows the user's first language**, and for English it is CLDR's root order.

## How the two systems date files

### Windows

**NTFS keeps four timestamps** per file: creation, last write, last access, and the MFT change time that records metadata edits. Explorer's Properties dialog shows three of them — Created, Modified, Accessed — and never surfaces the fourth.

- **Resolution is 100 nanoseconds** on NTFS. On FAT and FAT32 it is far coarser: creation to 10 milliseconds, last write to 2 seconds, and last access to **one day** — the access field holds a date with no time at all.
- **Last access is unreliable by design.** NTFS delays updating it by up to an hour, and modern Windows commonly leaves the update disabled for performance. Backup software, indexers, and antivirus scanners all touch it. It answers "did something read this," not "did the user look at this."
- **Created means created *on this volume*.** As a Microsoft moderator puts it: *"Date Created does not mean the Date you created the content. It means the date the file was first created on a given volume."* Copy a file to another disk and the copy's Created is the moment of the copy, while its Modified is carried across — which is why a file whose Created is newer than its Modified is completely normal and confuses people constantly.
- **File system tunneling.** Delete a file and create one with the same name in the same directory within about 15 seconds — the interval is a registry setting — and Windows restores the *old* file's creation date to the new file. It exists so that applications which save by writing a temp file and renaming it do not reset the document's age. It also means a creation date can be older than the bytes it describes.
- **Explorer shows a fourth date that is not a file date at all.** For photographs it reads **Date taken** out of the image's EXIF and offers it as a column and in Properties → Details.

### macOS

**APFS keeps four timestamps** per file, as nanoseconds since 1 January 1970 UTC, which Finder displays to the nearest second:

| APFS field | Finder calls it | What it means |
| --- | --- | --- |
| `create_time` | Created | when this record was made |
| `mod_time` | Modified | when the contents last changed |
| `access_time` | Last opened | when it was last read |
| `change_time` | — | when the metadata last changed |

- **There is a fifth date, and it is not on the file.** "Date Added" lives in the *directory entry*, not in the file's own metadata. It records when this item was put into this folder. It does not travel with the file, and moving a file to a new folder gives it a new one. It is also the date users most often mean by "recent," which is why Finder shows it.
- **Last opened is lazy.** Unless a volume is mounted with `APFS_FEATURE_STRICTATIME`, macOS only updates it when a file is read *and* the stored value is older than the modification time. QuickLook previews do not touch it at all, so a file you have looked at ten times can claim never to have been opened.
- **Copying loses Created across filesystems.** Within one APFS volume it is usually preserved; copy to a different filesystem and Created typically becomes the copy time.
- **A pre-1970 creation date is fragile.** Setting a modification date after 1970 on a file whose creation date is before it silently drags the creation date forward.

### Which date does the user actually mean?

Almost never the one the filesystem is most confident about.

**For a photograph, the meaningful date is inside the file, not on it.** The capture time is EXIF `DateTimeOriginal`, written by the camera. Transferring photos from a phone by anything other than a metadata-aware import stamps the file with the transfer time, so Created and often Modified answer "when did this arrive," not "when was this taken." Windows Explorer papers over this by surfacing EXIF as **Date taken**; Finder has no such column, though Spotlight indexes the same value as `kMDItemContentCreationDate`. A date sort that a photographer would call correct therefore needs to read inside the file.

**For a download, the meaningful date is when it landed, and that is usually what you get.** Contrary to a common impression, mainstream browsers do *not* set a saved file's modification time from the server's `Last-Modified` header — they use the download time. Firefox has needed an extension to do otherwise since version 5; Chrome has no such option. The genuinely old modification dates people run into come from other paths: extracting a zip, which stores and restores the archived times; `curl -R`, `wget`, and `rsync`, which preserve them on purpose; and some cloud-sync clients. In those cases Modified is old and Created is now — the same inversion as a Windows copy, from a different cause.

**So the ranking of trust, for fuji's purposes:** EXIF capture time is the truth for photographs and is not free to read. Modified is the most durable filesystem answer, because it survives copies. Created answers a question about *this disk*, which is sometimes exactly right — "what did I put here recently" — and sometimes nonsense. Last accessed is not worth showing.

---

# Part two: what fuji will do

## The enumeration

    StrcmpSort   MacSort   WindowsSort   SmartSort   ModifiedSort   CreatedSort   SizeSort   ShuffledSort

**Enumerated values are Title Case throughout fuji**, so a named member of a set is recognizable on sight: `StrcmpSort` is a member, `strcmpSort` would be a variable holding one. **Each ends in Sort, as a fit ends in Fit**, because the word stands alone in `fuji.toml`, where `Windows` or `Mac` could mean the platform or anything about it; the settings panel, whose title already says Sort, has a word of its own for each, Windows, Mac, and Simple lexicographic for `StrcmpSort`. The prose below calls each by its first word.

A separate boolean reverses whichever order is chosen. Reverse is not a member — every order has both directions, Shuffled included.

## The four name sorts

**Strcmp** is JavaScript's own `sort()`, kept deliberately, and was called Alphabet until it was plain that Zebra before apple is not alphabetical. It is the only order that is exactly predictable from the bytes, with no locale, no numeric guessing, and no opinion — the one to reach for when the clever orders have done something surprising and the user wants to see the folder plainly. It is written, in `StrcmpSort.js`.

**Mac** and **Windows** reproduce the two shells, so a user who arranged a folder in Finder or Explorer can get the same arrangement in fuji. Both are ours to write: `localizedStandardCompare:` and `StrCmpLogicalW` are platform calls fuji cannot reach from the page, and reaching them from Rust would still leave each platform unable to produce the other's order. Both are written, in `WindowsSort.js` and `MacSort.js`, each with an essay on what it follows and where it stops.

**Windows is Windows' rules for ASCII, written out, over the page's own `Intl.Collator`.** `WindowsSort.js` carries no table: the order of ASCII's symbols and the characters Windows sets aside are two short strings, and the rest is rules, a run of digits as one number, case ignored, the hyphen and apostrophe breaking ties. Every character outside ASCII is ordered by the engine's collation, and an emoji goes after every letter, where Windows puts one. It is exact for a name in ASCII, with no disagreement with `StrCmpLogicalW` on every pair of ASCII characters in four places or on two hundred thousand random pairs, and close for any other, from under one percent of random pairs for mixed names to about a quarter for Chinese characters; its essay has every number.

**Mac is CoreFoundation's walk, written out, over the page's own `Intl.Collator`.** On the Mac the page's collator is the system's ICU, the one Finder uses, so the order there is Finder's for every character; elsewhere it is the web engine's ICU, the same for ASCII and accented Latin and able to differ for Chinese characters and characters newer than one side knows. It is the order of a Mac whose language is English.

Both names make a promise fuji should be honest about. Microsoft says its function's results can change between releases and should not be used for canonical sorting, and each sort is one language's order where the shell follows the user's own.

**Fuji carries no table, decided.** From 2026-11-20 it carried Microsoft's whole table for Windows 11, 69 KB of base64 packed by a script, and built Windows' own key from it, exact on every name. The user took it out the next day: a block that long cannot be read by a person, and has to be trusted for where it came from, where two short strings and some rules can be checked by eye and against any Windows machine. Both matter, an order that matches and code that can be verified, and for names outside ASCII the second was chosen over the first. A table of Latin alone, 3 KB, had been weighed before the whole one and passed over as a set of choices about what to leave out; what replaced the table in the end is no excerpt of it but the engine's collation, which knows every script. The table is Microsoft's, published without a license of its own, and Wine ships data built from the same file. The table-built sort lives on in `side/sort` as `WindowsTableSort.js`, the exact reference fuji's sort is held against.

**The walk's order of folders stays as it is, decided.** The pager orders a folder's images by the chosen sort, and the folders themselves come from steel's walk in the order of their names' bytes, whatever the sort, so `Photos 10` comes before `Photos 2` as the sheet pages across them. The walk is a test bed for the speed and integrity of fuji's four parts, and may go once fuji browses as a file manager does, so its order is left rudimentary and outside the sorts a user chooses.

**Smart** is fuji's own, and it is the one worth the effort, because it is the only one not copying somebody. Both shells were reaching for the same thing — a person reading `page2` and `page10` means two and ten — and both left the same hole where digit runs tie: `page1`, `page01`, `page02`, `page2`. Smart is where that gets an answer, along with the other cases a folder of scanned pages actually contains.

## The date sorts

**Modified** and **Created** map to what `disk_stat` already returns, and the prior art says what to expect from them: Modified survives copies and is the more durable answer; Created describes this volume and can legitimately be newer than Modified, or older than the bytes, thanks to tunneling on Windows.

**Created has no answer on some filesystems** — `disk.rs` already returns 0 where `metadata.created()` fails, which is common on Linux. That needs a rule rather than a pile of files dated 1970.

**Last accessed is not offered.** It is lazily updated on both systems, disabled outright on much of Windows, and touched by indexers and antivirus. It would be a sort that changes for reasons the user did not cause.

**Date Added and Date Taken are the two dates users most often mean, and neither is free.** macOS keeps Date Added in the directory entry, outside `stat`. Date taken lives in the image's EXIF, which the tip scan now reads, with a movie's own times, and answers as one moment it calls taken; sorting by it costs a read of each file's first bytes, which a listing does not make. Both are real work rather than another field on a struct, and both are worth doing eventually, because a photographer sorting a folder by date means the day of the photograph.

## Size and Shuffled

**Size** is the cheapest sort fuji has: `disk_readdir` already returns a size for every entry, so it needs no extra call and no new Rust.

**Shuffled is not a comparison.** Every other order is a function of the listing and can be rebuilt at any moment. A shuffle is a permutation that has to be kept, or flipping back through it lands somewhere new each time. It is state, and belongs with the model's other state rather than with the comparators.

## What each costs to build

**The name sorts are pure functions of a listing fuji already has.** No disk work, no framework, and each one testable against a fixed list of nasty filenames with nothing running. That makes them the natural first ones, and `Smart` the natural first of those, since it is the only one whose behaviour is fuji's to decide.

**Size is free.**

**Modified is in the listing, and Created is not.** `disk_readdir` returns each entry's modified time beside its name and size, so Modified needs no new Rust. Ordering by Created today means one `disk_stat` per file — five thousand IPC round trips for a large folder — so the listing should carry the creation time too before Created is written rather than after it is slow.

## Where it lives

Each sort is a pure function in a file of its own, named for the member so it reads as the swappable thing it is — `StrcmpSort.js`, `WindowsSort.js`, `MacSort.js`, and `SmartSort.js` beside them when that exists. Two entries in, which comes first out; no disk, no framework. The model keeps the register of them and is the only caller.

**A sort is a comparator**, the language's own idiom, so a list places a row that arrives later by search instead of sorting everything again, and a column composes one to break its own ties. A sort whose key costs something to work out, a name split into runs of digits, works it out once per entry and compares the keys.

They are the easiest thing in fuji to test, and the fixed list of filenames they are tested against is itself worth keeping, because it is the specification of what `Smart` means.

**The platform sorts are tested against the platforms.** `side/sort` holds it: `names.js`, twenty-one small groups of names that each show one thing a shell does; `oracle.ps1` and `oracle.swift`, which ask Windows and the Mac; and `test.js`. `pnpm sort-test record` on each platform writes what it said about every pair of the names, `windows-sort.txt` and `mac-sort.txt`, which are tracked, so each machine checks the other's sort; `pnpm sort-test` checks both and then asks the platform it is on about random names by the hundred thousand; `pnpm sort-test folder` and a path makes the names into real files, a folder a group, with fuji's order beside them as `fuji-sort.txt`, to look at in Explorer or Finder, so the folder is made on each machine from `names.js` and never carried between them; `pnpm sort-test shell` and that path, on Windows, asks the shell's own folder object about those files; and `pnpm sort-test explorer` and that path opens each folder in a real Explorer window, sorts it by name, and reads back the order the window shows, which on Windows 11 on 2026-11-20 was fuji's in every folder but the one where Windows contradicts itself. `pnpm sort-test finder` and the path does the same with Finder's windows on the Mac. On the Mac mini, macOS 27.2, in English with no list sort order set, on 2026-11-20: `mac-sort.txt` was recorded; `MacSort.js` agreed with the Mac on every recorded pair and on 400,000 random pairs in each mix the Mac is asked about from node, after two fixes where it had not, each traced through CoreFoundation's source and written in the code; the long numbers agreed, so the Mac that ships still wraps at 64 bits; every folder read back from a Finder window in list view sorted by name came out in `MacSort.js`'s order, and so did Finder's own scripted sort by name; and Windows' record passed there on every pair outside the group Windows contradicts itself about, so `WindowsSort.js` depends on nothing of the machine it runs on. On Windows 11 the next day both records passed again, with no pair in 95,703 disagreeing for either sort. Every result to here was the windows sort that carried Microsoft's table. That table came out the same day, and with the sort that has none, the Mac's record passes as before, and Windows' passes on every pair of names in ASCII and differs on 1,824 pairs of 95,661 with a name outside it, which the test now counts and does not fail on; the folders shown in Explorer have not been read back again with it.

The chosen sort is one value in the model, persisted through `fuji.toml` and chosen in the settings panel, beside the fits. The settings schema validates it against this enumeration, so a hand-edited file naming a sort that does not exist reports the problem and falls back to factory rather than ordering by nothing.

## Open

- **There are two different "smart" ideas in the notes.** This file describes Smart as a name order. The other describes smart as a *date* order with numbered-sequence grouping, for the brochure feature. They may be two members, and if so the name order needs its own word.
- **Should Mac and Windows follow the user's language?** Each shell does, and each sort here is English's order. For both it would now be a locale handed to the collator, since neither carries a table; Windows' own table has every locale's exceptions, 440,000 lines of them, mostly Chinese, Japanese and Korean, which the engine's collation would stand in for and not match.
- **Windows 10 has not run the test, and need not.** Microsoft's tables give every ASCII character the same weights on both, which is the agreement that was wanted, so one sort serves every Windows; `pnpm sort-test` on a Windows 10 machine would say what that Windows itself does, should it ever matter.
- **What `Smart` does with tied digit runs** — `page1` against `page01` — which is the hole both shells left and the reason this sort exists.
- **What `Created` does where the filesystem has no answer:** sort those to one end, fall back to Modified, or refuse.
- **Whether Date Added earns the Rust it costs, and Date Taken the read of every file's first bytes.** Probably yes for photographs, eventually.

## Sources

- [StrCmpLogicalW function](https://learn.microsoft.com/en-us/windows/win32/api/shlwapi/nf-shlwapi-strcmplogicalw) — Microsoft, including the ordering example and the stability warning
- [What is up with number sorting?](http://archives.miloush.net/michkap/archive/2005/01/05/346933.html) — Michael Kaplan, on ASCII-only digits and the Group Policy switch
- [Sorting Rules](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPFileSystem/Articles/SortingRules.html) — Apple, the three modifications to UTS #10
- [Sort order, collation and the Finder](https://eclecticlight.co/2026/07/03/sort-order-collation-and-the-finder/) — locale effects, normalization
- [Be careful when interpreting APFS timestamps](https://eclecticlight.co/2025/10/24/be-careful-when-interpreting-apfs-timestamps/) — the four APFS timestamps, Date Added in the directory entry, lazy access time
- [Date created / modified / accessed](https://learn.microsoft.com/en-us/answers/questions/4281147/windows-explorer-date-created-modified-accessed-it) and [copying and dates](https://learn.microsoft.com/en-us/answers/questions/4062372/file-explorer-and-date-created-modified-change-whe) — Microsoft Q&A
- [NTFS](https://en.wikipedia.org/wiki/NTFS) and [Filesystem Timestamps: What Makes Them Tick?](https://www.giac.org/paper/gcfa/11322/filesystem-timestamps-tick/149007) — timestamp resolutions on NTFS and FAT
- [File system tunneling](https://unminioncurioso.blogspot.com/2020/07/ill-be-back-in-15-seconds-or-maybe-not.html) — the 15-second creation-date carry-over
- [Save original last-modified date on file downloads](https://bugzilla.mozilla.org/show_bug.cgi?id=178506) — browsers and `Last-Modified`
- [Date Taken, Date Created, Date Modified](https://organizingphotos.net/date-taken-date-created-date-modified-photo-time-stamps/) — EXIF versus file dates for photographs
