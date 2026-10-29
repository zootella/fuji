# Drive space

## The feature

- **A copy looks ahead.** Before fuji copies files from one drive to another, it checks that the destination has room for them, rather than copying for minutes and then reporting the drive full, as many file managers do.

## What the systems answer

- **Mac and Linux, statvfs.** One call for the volume holding any path: its total bytes, its free bytes, the bytes an ordinary user may write, and its block size.
- **Windows, GetDiskFreeSpaceExW.** Its lpFreeBytesAvailableToCaller is the bytes the calling user may write, which is smaller than the disk's free bytes only where per-user quotas are on ([Microsoft Learn](https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw)).
- **fs4 wraps both.** fs4 1.1.0, from April 2026, under MIT or Apache, is a fork of fs2: `fs4::statvfs(path)` answers the available, free and total bytes and the allocation granularity ([docs.rs](https://docs.rs/fs4/latest/fs4/struct.FsStats.html)). Its dependencies, rustix 1 and windows-sys 0.61, are already in the lockfile at versions it accepts, so it should add itself alone.
- **Its calls are safe Rust**, so steel can hold them under its rules, beside the standard library's calls in its disk module.

## What the Mac counts as available

- **Measured on the Mac mini on 2026-10-29, for the volume holding the home folder.** statvfs answered 60,876,795,904 bytes available, the same as Foundation's volumeAvailableCapacity and what df says, and 494,384,795,648 bytes total, the same as Foundation's. Foundation's volumeAvailableCapacityForImportantUsage answered 68,192,619,441 bytes, 7.3 GB more.
- **The difference is purgeable space**, which macOS clears on its own as it needs room, so the larger figure counts on that happening in time. Reading it takes a Foundation call, which belongs in copper, on the Mac alone. Finder keeps its own copy of these figures, which can drift out of date ([The Eclectic Light Company](https://eclecticlight.co/2023/04/27/where-does-macos-get-its-volume-free-space-figures-from/)).
- **Open: which figure the Mac's look ahead uses.**

## What else decides whether a copy fits

- **Whole allocation units.** A file takes whole units on the destination, so the room a copy needs is each file's size rounded up to the destination's allocation granularity, and a folder of many small files can need far more than its sizes add up to. The 128 KB unit of a large exFAT drive is from memory.
- **FAT32's file size limit.** FAT32 refuses a file of 4 GB or more whatever space is free, which is from memory, and checking it takes the destination's file system type, another call.
