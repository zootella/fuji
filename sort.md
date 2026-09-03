# Sorts

The order fuji shows a folder in. One value, chosen in the sheet, used by every table — `architecture.md` says why it lives in the model rather than in either.

This file has two halves. The first is prior art: how Windows and macOS actually sort and date files, researched rather than remembered, because fuji is going to reproduce some of it and deliberately depart from the rest. The second is the plan. Nothing in the plan is written yet.

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
- **ASCII digits only.** It recognises `0`–`9` and does not treat digits from other writing systems as numbers.
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

This is what fuji ships today — `listSiblings()` in `library.js` ends with a bare `.sort()`.

In the page, the closest thing to Finder is `Intl.Collator` with `numeric: true` and a base sensitivity. It is numeric-aware, case-insensitive and diacritic-insensitive, and it follows a locale. It is close to `localizedStandardCompare:` and is not the same function.

### Where they disagree, and the leading-zero hole

Both shells compare digit runs by value. Neither documents what happens when two runs have the *same* value but different spellings — `01` and `1`, `007` and `7`. The numeric comparison is a tie, and every implementation invents its own tiebreak: shorter first, longer first, fall back to character comparison, or leave the order to whatever the sort algorithm happened to do. This is the seam where "natural sort" implementations visibly disagree with each other and with the shell they were trying to copy.

It is also the case the user is most likely to have in a folder of scanned pages, where `page1`, `page01`, `page02` and `page2` sit together.

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

    Alphabet   Mac   Windows   Smart   Modified   Created   Size   Shuffled

**Enumerated values are Title Case throughout fuji**, so a named member of a set is recognizable on sight: `Alphabet` is a member, `alphabet` would be a variable holding one. This extends `style.md`, which covers `camelCase` and bans `ALL_CAPITALS` but does not yet speak to enumerations.

A separate boolean reverses whichever order is chosen. Reverse is not a member — every order has both directions, Shuffled included.

## The four name sorts

**Alphabet** is JavaScript's own `sort()`, kept deliberately. It is the only order that is exactly predictable from the bytes, with no locale, no numeric guessing, and no opinion — the one to reach for when the clever orders have done something surprising and the user wants to see the folder plainly.

**Mac** and **Windows** reproduce the two shells, so a user who arranged a folder in Finder or Explorer can get the same arrangement in fuji. Both are ours to write: `localizedStandardCompare:` and `StrCmpLogicalW` are platform calls fuji cannot reach from the page, and reaching them from Rust would still leave each platform unable to produce the other's order.

Both names make a promise fuji should be honest about. Microsoft says its function's results can change between releases and should not be used for canonical sorting. Apple's follows the user's locale, so there is no single Mac order to match. These sorts will be faithful in intent and close in practice, and the open question below asks whether the names should say so.

**Smart** is fuji's own, and it is the one worth the effort, because it is the only one not copying somebody. Both shells were reaching for the same thing — a person reading `page2` and `page10` means two and ten — and both left the same hole where digit runs tie: `page1`, `page01`, `page02`, `page2`. Smart is where that gets an answer, along with the other cases a folder of scanned pages actually contains.

## The date sorts

**Modified** and **Created** map to what `disk_stat` already returns, and the prior art says what to expect from them: Modified survives copies and is the more durable answer; Created describes this volume and can legitimately be newer than Modified, or older than the bytes, thanks to tunneling on Windows.

**Created has no answer on some filesystems** — `disk.rs` already returns 0 where `metadata.created()` fails, which is common on Linux. That needs a rule rather than a pile of files dated 1970.

**Last accessed is not offered.** It is lazily updated on both systems, disabled outright on much of Windows, and touched by indexers and antivirus. It would be a sort that changes for reasons the user did not cause.

**Date Added and Date Taken are the two dates users most often mean, and neither is free.** macOS keeps Date Added in the directory entry, outside `stat`. Date taken lives in the image's EXIF. Both are real work rather than another field on a struct, and both are worth doing eventually, because a photographer sorting a folder by date means the day of the photograph.

## Size and Shuffled

**Size** is the cheapest sort fuji has: `disk_readdir` already returns a size for every entry, so it needs no extra call and no new Rust.

**Shuffled is not a comparison.** Every other order is a function of the listing and can be rebuilt at any moment. A shuffle is a permutation that has to be kept, or flipping back through it lands somewhere new each time. It is state, and belongs with the model's other state rather than with the comparators.

## What each costs to build

**The name sorts are pure functions of a listing fuji already has.** No disk work, no framework, and each one testable against a fixed list of nasty filenames with nothing running. That makes them the natural first ones, and `Smart` the natural first of those, since it is the only one whose behaviour is fuji's to decide.

**Size is free.**

**The date sorts need Rust that does not exist.** `disk_readdir` returns names and sizes but no timestamps, so ordering by date today means one `disk_stat` per file — five thousand IPC round trips for a large folder. The answer is a listing that carries times, either a widened `DirEntry` or a second command, and it should be settled before a date sort is written rather than after it is slow.

## Where it lives

The comparators are pure functions in their own plain module, imported by the model. Listing in, order out; no disk, no framework. They are the easiest thing in fuji to test, and the fixed list of filenames they are tested against is itself worth keeping, because it is the specification of what `Smart` means.

The chosen sort is one value in the model, persisted through `fuji.toml`. The settings schema validates it against this enumeration, so a hand-edited file naming a sort that does not exist reports the problem and falls back to factory rather than ordering by nothing.

## Open

- **There are two different "smart" ideas in the notes.** This file describes Smart as a name order. `roadmap.hide.md` describes smart as a *date* order with numbered-sequence grouping, for the brochure feature. They may be two members, and if so the name order needs its own word.
- **How faithful should Mac and Windows be, and should the names say so?** Neither target is fixed: Microsoft reserves the right to change its results, and Apple's answer depends on the user's language.
- **What `Smart` does with tied digit runs** — `page1` against `page01` — which is the hole both shells left and the reason this sort exists.
- **What `Created` does where the filesystem has no answer:** sort those to one end, fall back to Modified, or refuse.
- **Whether Date Added and Date Taken earn the Rust and the EXIF reading they cost.** Probably yes for photographs, eventually.

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
