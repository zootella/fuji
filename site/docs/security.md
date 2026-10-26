# Security

For twenty years the most serious attacks on personal computers and phones have arrived inside ordinary files, a JPEG, a song, a font, a GIF, and gone to work the moment a program read them to be helpful: to draw a thumbnail, index a folder or preview a message, before anyone chose to open anything. Each of those attacks worked through a few facts about how a computer runs a program, and each defense that held was built from the same facts. Fuji reads files by the thousand to show them, so its design starts from this history, and answers it with two walls: a reader of its own in safe Rust, and the operating system's sandbox around a second process.

## Twenty years of hostile files

### What a decoder does

A file is a row of bytes, and a format is an agreement about what they mean. A decoder is the program that keeps its side of the agreement: it reads the bytes in order and does what they say. Much of what they say is numbers about the rest of the file: this picture is 640 pixels wide, this block of compressed data is 4,096 bytes long, the next table starts 12,000 bytes further on. The decoder cannot know those numbers are true until it has acted on them, and whoever made the file wrote every one.

Most decoders are written in C or C++, chosen for speed and because they run anywhere, and those languages do exactly what the code says, including the wrong thing. If a decoder sets aside room for 640 pixels and a later number in the file makes it copy 700, the extra bytes are written into whatever memory comes next. That memory belongs to something else: other data, the note of where the program goes when the current function returns, or the address of a function it is about to call. Microsoft's security bulletins call this a buffer overrun. The file's author chose the extra bytes, and when they land on an address the program is about to use, the file has chosen what the program does next. Languages that check every access to memory, [Rust](#rust), Swift and Java among them, cannot make this particular mistake; a program written in one stops instead.

A decoder also decides for itself what kind of file it is reading, and it decides from the bytes rather than the name. Most formats begin with a signature: a JPEG starts with the two bytes `FF D8`, which [the JPEG standard](https://www.w3.org/Graphics/JPEG/itu-t81.pdf) calls the start of image, and a PNG with eight bytes that spell out `PNG` among them, as [the PNG specification](https://www.w3.org/TR/png-3/#5PNG-file-signature) lays out. Programs that read pictures look at those first bytes, so a picture renamed `.txt` still opens as a picture, which is a convenience for anyone whose files have lost their names. It also means a name protects no one. Microsoft answered the question of whether blocking a dangerous extension would help in [its bulletin of January 2006](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2006/ms06-001):

> The Graphics Rendering Engine does not determine file types by the file name extensions that they use. Therefore, if an attacker alters the file name extension of a WMF file, the Graphics Rendering Engine could still render the file in a way that could exploit the vulnerability.

### Looking was enough, 2004 to 2011

In September 2004 Microsoft fixed a buffer overrun in GDI+, the code that Windows XP, Office, Internet Explorer and the .NET Framework all used to draw JPEG pictures ([MS04-028](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2004/ms04-028), CVE-2004-0200). The bulletin's list of ways in included a web page, an email shown in the preview pane, a picture inside an Office document, and a folder on a network share previewed in Windows Explorer. It also said, "There is no way for an attacker to force a user to open a malicious file," which was how security was still described in 2004: a person had to open something. The list above it showed that opening had stopped being something only a person did. A mail client previewed, a file manager drew thumbnails, and each read a stranger's file on behalf of someone who never chose it.

The Windows Metafile came next. A WMF is a recording of drawing commands from 16-bit Windows, replayed to draw the picture, and one of its records, `SETABORTPROC`, handed Windows a function to call while a print job spooled, the job [the `SetAbortProc` call](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setabortproc) does for a program that prints. In a file from a stranger, the function was whatever the stranger supplied, and the record, as [Tom Liston's technical explanation for the SANS Internet Storm Center](https://web.archive.org/web/20060824090717/http://handlers.sans.org:80/tliston/WMFTech.pdf) put it that January, "allows what should be a 'data only' file to actually carry executable content." It was being exploited before any fix existed, and Microsoft's fix, [MS06-001](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2006/ms06-001) for CVE-2005-4560 on January 5, 2006, repaired no bounds check: it removed support for the record from WMF images. Google Desktop Search, which read a computer's files to index them, reached the bug without anyone opening anything, and Brandon Paddock of Microsoft, explaining the design of Windows Search, [cited it a year later](https://web.archive.org/web/20260920044830/https://brandonlive.com/2007/06/20/faq-why-does-wds-windows-vista-use-so-many-processes/) as the reason Windows' own indexer reads files differently, which the next section comes to.

Looking at a folder stayed enough for years. In 2010 Windows ran code when it displayed the icon of a crafted shortcut ([MS10-046](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2010/ms10-046), CVE-2010-2568). Stuxnet, the worm [Ralph Langner described](https://foreignpolicy.com/2013/11/19/stuxnets-secret-twin/) as built to wreck the centrifuges of Iran's uranium enrichment plant at Natanz, spread through it on USB drives, infecting machines, [according to CISA](https://www.cisa.gov/news-events/ics-advisories/icsa-10-201-01c), "after users open a USB drive with a file manager capable of displaying icons." In 2011 it was a thumbnail ([MS11-006](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2011/ms11-006), CVE-2010-3970).

### Walls and fuzzers, 2007 to 2016

The first answer was to stop trusting the code that reads. Every program on a computer runs as a process, and the operating system decides what each process may touch: which files, the network, other programs. A decoder that runs inside a program has everything that program has. A decoder moved into [a process of its own](#twinning), with almost nothing allowed, can still be fooled, but the code that fools it inherits almost nothing. Windows Search was built this way in Windows Vista: its filters run in a separate process with reduced privileges that ["cannot even read the filesystem"](https://web.archive.org/web/20260920044830/https://brandonlive.com/2007/06/20/faq-why-does-wds-windows-vista-use-so-many-processes/), handed each file's contents by the indexer. Google built Chrome around the same idea and described [its sandbox](https://blog.chromium.org/2008/10/new-approach-to-browser-security-google.html) in October 2008: the code that renders web pages and runs their scripts works in processes that cannot read or change the user's files. And Explorer moved thumbnail drawing out of its own process into a helper, because, as [Raymond Chen put it in 2009](https://devblogs.microsoft.com/oldnewthing/20090212-00/?p=19173), "Explorer has learned not to trust thumbnail extractors." The name for this is a sandbox, and Chromium has since written its rule down as [the Rule of 2](https://chromium.googlesource.com/chromium/src/+/main/docs/security/rule-of-2.md): of untrustworthy input, code in an unsafe language, and a process with high privilege, a program may combine any two and never all three.

The second answer was to find the bugs before anyone else did, by machine. [Fuzz testing](https://google.github.io/oss-fuzz/) feeds a decoder millions of files made by changing real ones a little at a time, and watches for a crash; the fuzzers in use today also watch which parts of the decoder each file reaches, and keep the files that reach somewhere new, so they work their way into code nobody thought to test. In January 2014 Google reported that its own fuzzing had led to [1,120 fixes](https://security.googleblog.com/2014/01/ffmpeg-and-thousand-fixes.html) in FFmpeg, the free library that reads and decodes hundreds of audio and video formats, and advised that until FFmpeg and Libav, a fork of it, were "fuzz clean," people should "refrain from using either of the two projects to process untrusted media files." In December 2016 Google opened [OSS-Fuzz](https://google.github.io/oss-fuzz/), started in response to Heartbleed, which fuzzes open-source projects around the clock; by August 2023 it had helped fix "over 10,000 vulnerabilities and 36,000 bugs across 1,000 projects." [FFmpeg's own history](https://github.com/FFmpeg/FFmpeg) holds more than three thousand fixes that cite an OSS-Fuzz test case. A number like that says two things at once: the project is examined more closely than almost any other code that reads media, and that close examination keeps finding more.

### No click at all, 2015 to 2017

In July 2015 Joshua Drake of Zimperium showed that Android's media library, Stagefright, could be taken over by a video sent in a text message, through several bugs, CVE-2015-1538 among them ([Zimperium](https://www.zimperium.com/blog/experts-found-a-unicorn-in-the-heart-of-android/), [CERT](https://kb.cert.org/vuls/id/924951)). Android's messaging app fetched an MMS attachment and handed it to Stagefright as it arrived, to show a preview, which made the worst of the attacks one that, in Zimperium's words, "requires no user-interaction." The bugs reached an estimated 950 million devices. Google's response, in Android 7, split the media server into separate processes with no camera, microphone or network, because ["media parsers do not require access to most of the privileged permissions held by mediaserver"](https://android-developers.googleblog.com/2016/05/hardening-media-stack.html), and later moved the software decoders into a process of their own after counting that about [80% of 2018's critical and high media bugs](https://security.googleblog.com/2019/05/queue-hardening-enhancements.html) were in them.

In November and December 2016 Chris Evans did the same to Linux desktops with music from old game consoles. A crafted NES music file (CVE-2016-9447) took over Ubuntu ["without the user opening the exploit file -- they only have to navigate to the folder"](https://scarybeastsecurity.blogspot.com/2016/11/0day-exploit-compromising-linux-desktop.html), because the file manager's thumbnailer read it, and he named it `.mp3`, since the media framework "will ignore a file's suffix and use file format auto detection." A web page could drop a file into the Downloads folder, where Fedora's search indexer, Tracker, [read it on its own](https://scarybeastsecurity.blogspot.com/2016/11/0day-poc-risky-design-decisions-in.html) (CVE-2016-9445), and a SNES music file named `.flac` (CVE-2016-9957 to CVE-2016-9961) [took over Fedora 25](https://scarybeastsecurity.blogspot.com/2016/12/redux-compromising-linux-using-snes.html) the same way. [His conclusion](https://scarybeastsecurity.blogspot.com/2016/11/0day-exploit-advancing-exploitation.html) was that "Code that automatically indexes or thumbnails media files really needs to be sandboxed in this day and age," and GNOME did it: Tracker's extractor got a sandbox [within weeks](https://download.gnome.org/sources/tracker/1.11/tracker-1.11.2.changes), and every thumbnailer ran in one from [GNOME 3.26 in 2017](https://www.hadess.net/2017/07/security-for-security-gods-sandboxing.html), with no network and [nothing of the user's files but the one it was reading](https://gitlab.gnome.org/GNOME/gnome-desktop/-/raw/master/libgnome-desktop/gnome-desktop-thumbnail-script.c). Tracker's extractor also learned to give each file [five seconds](https://gitlab.gnome.org/GNOME/tracker-miners/-/raw/master/src/tracker-extract/tracker-extract.c), and to write down which file it was reading before it began, so a file that crashed or stalled it is [never read again](https://gitlab.gnome.org/GNOME/tracker-miners/-/raw/master/src/tracker-extract/tracker-extract-decorator.c).

Some formats point at other files, and that turned out to be a door of its own. A playlist lists the files it plays, and a video streamed over the web is a playlist of short segments. In January 2016 Maxim Andreev showed that a playlist hidden in a file "with any extension: avi/mov" could make FFmpeg read a file from the same computer and send its first line to a web server, ["when file manager tries to generate thumbnail"](https://seclists.org/oss-sec/2016/q1/85) (CVE-2016-1897 and CVE-2016-1898), and in 2017 Emil Lerner and Pavel Cheremushkin made an AVI read a local file and draw it into the video's own picture ([their proof of concept](https://github.com/neex/ffmpeg-avi-m3u-xbin), CVE-2017-9993). FFmpeg answered with lists of what a file may reach, and its fix admitted that leaks remained possible ["from files and symlinks ending in common multimedia extensions."](https://github.com/FFmpeg/FFmpeg/commit/189ff4219) The same year Check Point took over VLC, Kodi, Popcorn Time and Stremio, on an estimated 200 million video players and streamers, through [subtitle files](https://research.checkpoint.com/2017/hacked-in-translation/), the small text files that ride along with a film (CVE-2017-8310 to CVE-2017-8314).

### Around the wall, 2020 to 2023

In March 2020 Microsoft warned of a bug in how Windows read a kind of font (ADV200006, later CVE-2020-0938 and CVE-2020-1020), reachable by looking at a document in Explorer's preview pane, and added that on Windows 10 an attack ["could only result in code execution within an AppContainer sandbox context"](https://api.msrc.microsoft.com/sug/v2.0/en-US/vulnerability?$filter=cveNumber%20eq%20%27ADV200006%27), so it advised Windows 10 users not to bother with the workarounds. That is a sandbox doing its job: the bug was the same, and what it could reach was not.

Apple built one for the iPhone's messages. iOS 14 moved almost all the reading of untrusted data in iMessage into BlastDoor, a [tightly sandboxed service written in Swift](https://projectzero.google/2021/01/a-look-at-imessage-in-ios-14.html), a memory-safe language, and slowed its restarts too. Computers now load each program at a randomly chosen place in memory every time it starts, so an attacker who can overwrite an address does not know what address to write; one way through is to guess, crash, and guess again. BlastDoor's supervisor doubles the wait before restarting it after each crash, up to about twenty minutes, so an attack that guesses its way through takes, by Project Zero's estimate, "multiple hours to roughly half a day" instead of a few minutes.

Attackers went around the wall instead. In March 2021 Citizen Lab examined the phone of a Saudi activist infected with NSO Group's Pegasus spyware, and in [September they reported](https://citizenlab.ca/research/forcedentry-nso-group-imessage-zero-click-exploit-captured-in-the-wild/) what they had found in its backup: thirty-one files named `.gif`, twenty-seven of them 748-byte Photoshop files and four of them PDFs. Apple fixed the bug, CVE-2021-30860, the day Citizen Lab published. Google's Project Zero [took the exploit apart](https://projectzero.google/2021/12/a-deep-dive-into-nso-zero-click.html) in December. iMessage handed any attachment ending in `.gif` to a service outside BlastDoor, which passed it to ImageIO, Apple's picture library, and ImageIO chose among "over 20 image codecs" by the contents, "completely ignoring the file extension." It found a PDF, and inside it a JBIG2 stream: "a domain specific image codec designed to compress images where pixels can only be black or white." JBIG2 can refine a region of the picture by combining bits with AND, OR, XOR and XNOR, and those four operations are enough to compute anything. Once a bug in the decoder let the file write past its buffer, "over 70,000 segment commands defining logical bit operations" built "a small computer architecture with features such as registers" and "a full 64-bit adder and comparator," inside a picture decoder, and that small computer ran the rest of the attack. Project Zero called it "one of the most technically sophisticated exploits we've ever seen." Apple narrowed what that service accepts and moved GIFs into BlastDoor.

Two years later Citizen Lab caught another one, [BLASTPASS](https://citizenlab.ca/blastpass-nso-group-iphone-zero-click-zero-day-exploit-captured-in-the-wild/), in pictures inside PassKit attachments, the passes of Apple Wallet (CVE-2023-41064 and CVE-2023-41061). Four days later Chrome fixed a bug in the WebP picture library, reported by Apple and Citizen Lab, for which ["an exploit for CVE-2023-4863 exists in the wild"](https://chromereleases.googleblog.com/2023/09/stable-channel-update-for-desktop_11.html), and [Ben Hawkes judged the two likely to be the same bug](https://blog.isosceles.com/the-webp-0day/). Fonts kept their place on the list as well: FreeType, the library that draws text on Android, Linux and much else, had one bug used in a Chrome exploit chain in 2020 ([CVE-2020-15999](https://www.cve.org/CVERecord?id=CVE-2020-15999)) and another in 2025 (CVE-2025-27363) that ["may have been exploited in the wild"](https://www.facebook.com/security/advisories/cve-2025-27363), with ["limited, targeted exploitation"](https://source.android.com/docs/security/bulletin/2025-05-01) on Android.

### Bugs found by machines, 2025 and 2026

Fuzzers now have company. In 2025 Google's Big Sleep, an AI system built to find vulnerabilities, was credited in eleven of FFmpeg's fixes, among them a heap overflow in its reader for OpenEXR, the format of film effects houses ([CVE-2025-59733](https://www.cve.org/CVERecord?id=CVE-2025-59733)), and a use-after-free write in its decoder for SANM, the video format of LucasArts games of the 1990s ([CVE-2025-59734](https://www.cve.org/CVERecord?id=CVE-2025-59734)). FFmpeg's developers answered in public, [as It's FOSS reported](https://itsfoss.com/news/ffmpeg-google-fiasco/), that the SANM bug touched "the first 10-20 frames of Rebel Assault 2," and called such reports "CVE slop"; FFmpeg's [security page](https://ffmpeg.org/security.html) now says "Automated submissions are not accepted." The bug was real, a write to memory already freed, and any program built with every decoder carries the SANM decoder whether or not anyone owns a copy of Rebel Assault 2.

In June 2026 JFrog published [PixelSmash](https://jfrog.com/blog/pixelsmash-critical-ffmpeg-vulnerability-turns-media-files-into-weapons/) (CVE-2026-8461), an out-of-bounds write in FFmpeg's decoder for MagicYUV, a format for recording video without loss, and tried it on the programs that scan media collections. It ran attacker code through the library scan of Jellyfin, a home media server, with the memory randomization described above turned off, and crashed it reliably with randomization on. It corrupted another server's memory, Emby's, and the process exited as if nothing had happened. And it did nothing to Plex, which builds FFmpeg with its decoders turned off but for an allow-list, an approach JFrog called "the only effective defense we observed." FFmpeg fixed the bug in version 8.1.2.

These attacks are rare, and the record says how rare. The United States' Cybersecurity and Infrastructure Security Agency keeps a [catalog of vulnerabilities known to be exploited](https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json), 1,756 entries in October 2026, and it lists no bug in FFmpeg; a search of that catalog, the National Vulnerability Database and public reporting in October 2026 found no account of one used in the wild. The attacks above that did happen in the wild were aimed at particular people, an activist's phone among them. A collection of downloads holds far more files that are simply broken, truncated by a dropped connection or damaged by a dying disk, than files built to attack anything, and a program that reads collections meets the broken ones every day.

### What twenty years show

A name decides nothing. Every decoder in this history [chose what to run from the bytes](#the-tip-scan), and every attack that mattered used a name that looked harmless.

The attacks land on [software that reads files to be helpful](#fuji-s-protections): a preview pane, a thumbnailer, an indexer, a message that shows its attachment as it arrives. Nobody chose those files, so nobody was there to hesitate.

[Decoders written in C and C++ have bugs](#rust), and fuzzers and now AI systems find them by the thousand. A library examined this closely [has its bugs found and fixed in public](#ffmpeg), one release after another; a library nobody examines keeps whatever bugs it has, known only to whoever finds them first.

Walls work. A decoder in a sandbox can still be fooled, but the code that fools it reaches only what the sandbox allows, as Windows 10's font sandbox showed, and the systems that read untrusted files at scale, Windows Search, Android's media server, GNOME's indexer and Apple's BlastDoor, all put the reading in [a separate process with almost nothing allowed](#sandboxing). And attackers go around walls: FORCEDENTRY reached the one decoder left outside.

A crash is a signal, and its absence is not. A fooled decoder can exit cleanly, as Emby's did, so a program that reads untrusted files cannot judge them by [whether its reader survived](#twinning).

No wall is perfect. GNOME's thumbnail sandbox had a hole of its own ([CVE-2019-11460](https://www.cve.org/CVERecord?id=CVE-2019-11460)), no sandbox stops a bug in the operating system's kernel, and the best defenses raise the cost of an attack from one bug to several; none makes it impossible. That is the ground Fuji is built on.

## Fuji's protections

Fuji reads files the way the programs in this history did, to be helpful and by the thousand: the files of a folder as the user moves into it, and every file on a drive overnight as Fuji catalogs an archive. Nobody chooses each of those files, and an attacker can name a file anything, so every wall in Fuji is built for any bytes under any name. And Fuji has no network code: it sends nothing and fetches nothing, so a file is the only way anything reaches it.

Fuji keeps the [Rule of 2](#walls-and-fuzzers-2007-to-2016) in two ways. Where it reads a file with code of its own, it takes away the unsafe language: its reader is written in safe Rust, and the compiler holds it there. Where it reads the files nobody chose with a library written in C, it takes away the privilege: that library runs in a second process the operating system walls in. The two walls are independent. The first is a wall around code, which no file can steer, and it holds in any process. The second is a wall around a running process, and it holds whatever code runs inside.

The work divides between two processes. The **main process** is Fuji as the user meets it, with its windows. It draws thumbnails through the operating system's own imaging library, ImageIO on the Mac and WIC on Windows, which Apple and Microsoft keep current with the system, and before either decodes a picture, Fuji refuses one whose header claims more pixels than half the machine's memory could hold. It plays the file the user opens. The web engine decodes most of the pictures on the light table, on the Mac and Windows in a content process the engine sandboxes. The **sandbox process** reads the files nobody chose, every file in a folder or on a drive, with Fuji's own reader and with FFmpeg.

### Rust

Rust refuses the mistake this history began with. When a decoder in C copies 700 pixels into room for 640, the extra bytes land on whatever memory comes next. Rust checks every index against the length of what it indexes, and [the Rust book's chapter on data types](https://doc.rust-lang.org/book/ch03-02-data-types.html) says what happens instead:

> In many low-level languages, this kind of check is not done, and when you provide an incorrect index, invalid memory can be accessed. Rust protects you against this kind of error by immediately exiting instead of allowing the memory access and continuing.

The compiler also tracks which part of a program owns each piece of memory and when it is freed, and, as [the book's chapter on references](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) puts it, guarantees "that references will never be dangling references." The use-after-free write Big Sleep found in FFmpeg's SANM decoder is a program safe Rust refuses to compile.

The largest codebases bear this out. [Microsoft's security response team](https://www.microsoft.com/en-us/msrc/blog/2019/07/a-proactive-approach-to-more-secure-code) counted in 2019 about 70% of the vulnerabilities it assigns a CVE each year as memory safety issues, and [Chromium](https://www.chromium.org/Home/chromium-security/memory-safety/), looking at 912 high and critical security bugs since 2015, found around 70% of them to be "mistakes with C/C++ pointers." When Android began writing its new code in memory-safe languages, Rust among them, memory safety bugs fell from 76% of its vulnerabilities in 2019 to [24% in 2024](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html) and [below 20% in 2025](https://blog.google/security/rust-in-android-move-fast-fix-things/). That year Google counted about five million lines of Rust in Android, and put their rate of memory safety bugs at more than a thousand times lower than that of Android's C and C++.

By the Rule of 2, a reader in a safe language needs no sandbox. Chromium rewrote its QR code generator in Rust, removed the sandbox it no longer needed, and [the generator ran 20 times faster](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html). Fuji's reader runs in either process for the same reason.

### Four places

Fuji's code lives in four places, and each keeps a different promise. **Plastic** is the page, the JavaScript that decides everything: what to show, what to ask of the machine and when. The other three are Rust crates, named for metals. **Brass** is the crate Tauri builds the application from, and holds only what is tied to a window. **Copper** holds every call into C and the operating system's libraries away from a window, FFmpeg among them. **Steel** is the safe crate, and every reader Fuji writes for the bytes of a file lives in it.

Steel is a crate of its own because a module can forbid unsafe code in itself and still call the rest of its crate, while a crate reaches only what it names, and steel names nothing of Fuji's. Its root, [`steel/src/lib.rs`](https://github.com/zootella/fuji/blob/main/steel/src/lib.rs), is two lines:

```rust
#![forbid(unsafe_code)]
#![deny(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects)]
```

The first refuses unsafe code anywhere in the crate, at [a level no line beneath it can lower](https://doc.rust-lang.org/rustc/lints/levels.html). The second has [Clippy](https://doc.rust-lang.org/clippy/), Rust's linter, refuse everything that could stop a reader partway: an index that could fall outside its slice, an unwrap or an expect, a panic, and arithmetic that could overflow. Rust's own tools hold those promises on every line, so no reviewer has to.

Copper is steel's opposite. Every unsafe call Fuji makes outside a window sits in copper, so a search for `unsafe` there finds the complete list. And copper needs nothing of Tauri, so it runs in a process with no window at all, which is where the sandbox process comes from.

The page has walls of its own. Everything it may ask of the machine is one list of commands in brass, and nothing else is reachable from JavaScript. Fuji grants the page none of [Tauri's plugins](https://v2.tauri.app/security/capabilities/), so widening that list is a change to Rust, reviewed as code. A file's name reaches the page only through [Vue's escaping](https://vuejs.org/guide/best-practices/security.html), so no name ever becomes script, and Fuji shows an SVG only in an `<img>`, where [its scripts never run](https://developer.mozilla.org/en-US/docs/Web/SVG/Guides/SVG_as_an_image). And the [content security policy](https://v2.tauri.app/security/csp/) in [`tauri.conf.json`](https://github.com/zootella/fuji/blob/main/desktop/src-tauri/tauri.conf.json) allows the page its own scripts, the bridge to its own Rust, and pictures from its own bundle and from memory, and nothing more:

```json
"csp": "default-src 'self'; connect-src 'self' ipc: http://ipc.localhost; img-src 'self' data: blob:"
```

### The tip scan

The first thing Fuji reads in any file is its tip, the first 64 KiB, and at most one more 64 KiB slice where the tip points. Its reader, the **tip scan**, says from those bytes whether the file is a picture, a video or a sound, and what its format keeps in its header: a picture's size and turn, a video's size and the shape it shows at, a running time, the times the file states and the place it was taken. It names a file by its bytes and never by its name, and it reads no pixel and no sample.

The tip scan lives in steel, and five rules of its own make a hostile file harmless to it:

- No number from the file sizes anything, because the reader allocates nothing at all.
- Every read goes through one helper, which answers nothing outside the bytes in hand, so a length or an offset in the file can point anywhere and the worst it does is end the reading.
- The reader checks every sum of offsets, because a release build of Rust lets an overflowing sum wrap around, as [the same chapter of the Rust book](https://doc.rust-lang.org/book/ch03-02-data-types.html) warns, and a wrapped sum can look valid.
- Every loop moves forward and stops after 256 steps, so the 128 KiB in hand bound the work, whatever the file holds.
- Nothing can panic: the reader has no index, no unwrap, and no division before a check for zero.

The helper is two short functions in [`steel/src/tip.rs`](https://github.com/zootella/fuji/blob/main/steel/src/tip.rs), and every byte the tip scan reads passes through them:

```rust
fn held(&self, at: usize, n: usize) -> Option<&'a [u8]> {
	let end = at.checked_add(n)?;
	if let Some(s) = self.tip.get(at..end) { self.depth.set(self.depth.get().max(end)); return Some(s) }
	let from = at.checked_sub(self.slice_at)?;
	self.slice.get(from..from.checked_add(n)?)
}
fn read(&self, at: usize, n: usize) -> Step<&'a [u8]> {
	if let Some(s) = self.held(at, n) { return Ok(s) }
	if add(at, n)? as u64 > self.size { return Err(Stop::Chopped(SHORT)) }
	Err(Stop::Ask(PAST, at as u64))
}
```

A read beyond the tip becomes the request for the second slice, a read past the end of the file says the file was cut short, and neither touches any memory outside the bytes in hand. So a hostile file gets one of two outcomes, no answer or a wrong one, and every stop says which of four kinds it was: a format named by its signature and read no further, the reader's own limit, the file corrupted, or the file chopped.

Tests hold the tip scan's answers on 72 sample files in Fuji's repository, and [libFuzzer](https://llvm.org/docs/LibFuzzer.html) runs it through [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html) on inputs grown from those samples, a corpus of more than 17,000, watching for a panic. Each input goes through twice, once as Fuji reads a file and once through a tip cut to half its length, so even a small input reaches the code that reads the second slice, in [`steel/fuzz/fuzz_targets/scout.rs`](https://github.com/zootella/fuji/blob/main/steel/fuzz/fuzz_targets/scout.rs):

```rust
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
	scout(data, steel::tip::TIP_BYTES);
	scout(data, data.len() / 2);
});
```

### FFmpeg

The tip scan reads the formats most files are in. Forty years of collecting hold far more: AVI, MPEG program and transport streams, ASF, RealMedia, Flash video and DV, and the long tail of codecs inside them. For those Fuji asks FFmpeg, which reads them all, through mpv's player with no window.

FFmpeg is examined more closely than almost any other code that reads media. Google [fuzzed it before OSS-Fuzz existed](https://security.googleblog.com/2014/01/ffmpeg-and-thousand-fixes.html), and [OSS-Fuzz has fuzzed it](https://github.com/google/oss-fuzz/tree/master/projects/ffmpeg) around the clock since 2017. The other libraries Fuji builds with it that read what a file holds, mpv, dav1d, zlib, lcms2, FreeType, HarfBuzz, FriBidi and libass, are each [an OSS-Fuzz project](https://github.com/google/oss-fuzz/tree/master/projects) too. Their bugs are found and fixed in public, and Fuji's FFmpeg carries the fix for [PixelSmash](#bugs-found-by-machines-2025-and-2026).

Fuji builds FFmpeg and the libraries beside it into one library of its own, from each project's own release, every source pinned by its SHA-256 hash. Each release's sidecar, the small file beside the download that holds its hash, also holds the library's record: the library's hash, the hash of the recipe that built it, the tools, and every source with its own hash, as [the sidecar of the Mac download](https://fujidesktop.app/fuji.dmg.json) shows. Anyone can check what a copy of Fuji carries and where every part of it came from.

The recipe, [`desktop/libmpv/build.py`](https://github.com/zootella/fuji/blob/main/desktop/libmpv/build.py), builds whole kinds of attack out of FFmpeg:

```python
configure_make(folder, '--pkg-config-flags=--static', '--enable-static', '--disable-shared', '--disable-debug', '--disable-doc', '--disable-programs',
	'--disable-autodetect', '--enable-zlib', '--enable-libdav1d', *ffmpeg,
	'--disable-encoders', '--disable-muxers', '--disable-devices', '--disable-avdevice', '--disable-network', '--disable-protocols', '--enable-protocol=file')
```

FFmpeg in Fuji has no network and one protocol, files, so the playlist trick of [2016](#no-click-at-all-2015-to-2017), which used FFmpeg's other protocols to read a local file and send it to a web server, has no road. It carries no encoders and no muxers, the parts of FFmpeg that make a file. And Fuji builds mpv without Lua or JavaScript, so no script runs inside the player.

Each question narrows what runs still further. Fuji makes a player for every question and destroys it after, so nothing carries over from one file to the next, and starts every player with five of [mpv's options](https://mpv.io/manual/stable/), in [`copper/src/media.rs`](https://github.com/zootella/fuji/blob/main/copper/src/media.rs):

```rust
const ASK_ALWAYS: [(&str, &str); 5] = [
	("demuxer", "lavf"),
	("access-references", "no"),
	("ordered-chapters", "no"),
	("autoload-files", "no"),
	("resume-playback", "no"),
];
```

FFmpeg's demuxers alone read the file, so mpv's own readers for playlists, cue sheets and edit lists never see it. The player follows no reference to another file, loads no segment from another file in the folder, reads no subtitle or cover art from beside the file, and looks for no saved position. Under those options an ffconcat file and an HLS playlist come back unrecognized, wherever the files they name sit. And the question of a file's running time, format and tracks selects no track at all.

When PixelSmash reached the media servers, Plex was the one it could not touch, because Plex builds FFmpeg with its decoders disabled. Fuji keeps every demuxer and every decoder, since the formats of forty years of collecting are the reason it exists, and puts the questions it asks FFmpeg behind the second wall instead.

### Twinning

Fuji asks FFmpeg its questions in the sandbox process, which it builds from two features, ***Twinning*** and ***Sandboxing***.

***Twinning*** is Fuji's own executable started a second time to do the reading. The main process starts it with a switch on its command line, and the program's first lines hand it the work before Tauri starts, so it opens no window and runs only copper and steel, the same code the main process runs. The two talk over the three standard streams: the request goes down, the answers come up, and the main process reads every answer as it would read a stranger's file.

The main process starts it carrying nothing else of Fuji's. On the Mac it closes every file but those three streams in the new process, and points the new process's crash ports back at itself, in [`copper/src/sandbox.rs`](https://github.com/zootella/fuji/blob/main/copper/src/sandbox.rs):

```rust
let set = [
	libc::posix_spawn_file_actions_adddup2(&mut actions, stdin_read.as_raw_fd(), 0),
	libc::posix_spawn_file_actions_adddup2(&mut actions, stdout.as_raw_fd(), 1),
	libc::posix_spawn_file_actions_adddup2(&mut actions, stderr_write.as_raw_fd(), 2),
	libc::posix_spawnattr_setflags(&mut attributes, libc::POSIX_SPAWN_CLOEXEC_DEFAULT as libc::c_short),
	posix_spawnattr_setexceptionports_np(&mut attributes, EXC_MASK_CRASH | EXC_MASK_CORPSE_NOTIFY, port, EXCEPTION_DEFAULT, THREAD_STATE_NONE),
].into_iter().find(|code| *code != 0).unwrap_or(0);
```

Linux closes everything else as the process starts, and on Windows the main process hands the new process a list naming those three handles, [the one way Windows has](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute) to say which handles a process gets.

So a crash is one file's answer. A file that crashes FFmpeg ends the sandbox process alone, Fuji names the file it was reading, and the main process goes on. A file that hangs it, through a read the kernel never finishes on a dying disk or a demuxer going in circles, cannot hold Fuji, because the main process ends the sandbox process the moment its work is no longer wanted, whatever it is doing. And if Fuji itself ends, however it ends, the sandbox process sees its request stream close and ends with it.

The operating system never hears of the crash. On the Mac the main process answers the crash itself, on the port it holds, as [Crashpad's exception swallower](https://chromium.googlesource.com/crashpad/crashpad/+/main/test/mac/exception_swallower.h) does for the test programs of Chromium's crash reporter, so macOS puts up no "quit unexpectedly" dialog and writes no report; a handler inside the sandbox process could not do this, since a thread that has overflowed its stack leaves a signal handler no stack to run on. On Linux the sandbox process makes itself [undumpable](https://man7.org/linux/man-pages/man2/PR_SET_DUMPABLE.2const.html), so the kernel hands no core to a crash reporter. On Windows it sets its own [error mode](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-seterrormode) before anything else, so a crash puts up no dialog and reaches no Windows Error Reporting, including the [fast fail](https://learn.microsoft.com/en-us/cpp/intrinsics/fastfail?view=msvc-170) Rust and the C runtime use to abort, which a job object's own flag misses. A night of crashed files leaves no stack of dialogs in the morning.

***Twinning*** makes a crash one file's answer and a hang something Fuji ends. Walling in code a file has taken over is the work of ***Sandboxing***.

### Sandboxing

***Sandboxing*** puts the operating system's walls around the sandbox process, and Fuji builds them for one attacker: code a hostile file has started running inside FFmpeg. That code takes itself for a program running as the user, and reaches for what such code reaches for: the user's keys and the browser's cookies, a place to persist, a program to start, a connection out. It knows nothing of Fuji. The walls judge each reach, whatever code makes it, and they hold whether or not anything crashes, so a decoder that [exits cleanly after it has been fooled](#bugs-found-by-machines-2025-and-2026) has still reached only what its walls allow.

The sandbox process puts the walls on itself once, after the main process has named the folder and before it opens a single file, so no byte of any file arrives outside them. It holds nothing open but its three streams, since a wall judges a file as it opens and a file already open would pass it, and it starts with an environment emptied of everything it does not need.

#### On the Mac

On the Mac the wall is Apple's sandbox, which the kernel enforces. The sandbox process hands it a profile through `sandbox_init_with_parameters`, the call [Firefox's sandbox makes](https://github.com/mozilla-firefox/firefox/blob/main/security/sandbox/mac/Sandbox.mm), and the profile in [`copper/src/sandbox.rs`](https://github.com/zootella/fuji/blob/main/copper/src/sandbox.rs) denies everything, then allows reading under one folder:

```rust
const PROFILE: &str = "(version 1)
(deny default)
(allow file-read* (subpath (param \"FOLDER\")))
";
```

The folder reaches the profile as a parameter rather than spliced into its text, so no folder's name can change what the profile says. Inside that wall the sandbox process cannot read a file outside the folder or list the folder above it, write anywhere, connect even to the machine it runs on, start a program, look up Launch Services, the system's service for opening programs and addresses, or send a signal to the main process.

#### On Linux

On Linux the walls are two, both the kernel's own. [Landlock](https://docs.kernel.org/userspace-api/landlock.html) takes every right the process has over files and gives back one, reading under the folder. A [seccomp filter](https://docs.kernel.org/userspace-api/seccomp_filter.html), in the same file, refuses the system calls that reach past the process:

```rust
for call in [libc::SYS_socket, libc::SYS_execve, libc::SYS_execveat, libc::SYS_ptrace, libc::SYS_process_vm_readv, libc::SYS_process_vm_writev, libc::SYS_pidfd_open, libc::SYS_tkill] { rules.insert(call, vec![]); }
for call in [libc::SYS_io_uring_setup, libc::SYS_io_uring_enter, libc::SYS_io_uring_register] { rules.insert(call, vec![]); }
for call in [libc::SYS_kill, libc::SYS_tgkill, libc::SYS_rt_sigqueueinfo, libc::SYS_rt_tgsigqueueinfo] { rules.insert(call, vec![others().map_err(|e| fail(&e))?]); }
```

The filter refuses a socket, so the process reaches neither the network nor the desktop's message bus. It refuses starting a program, attaching to another process or reading and writing its memory, and sending a signal to any process but itself. And it refuses io_uring, through which a process could open a socket inside the kernel, where, as [LWN explains](https://lwn.net/Articles/1054225/), seccomp has "no visibility into — and thus no way to control — operations requested via io_uring." Before either wall goes up, the sandbox process makes itself undumpable, so no other process of the user's can attach to it.

#### On Windows

On Windows the walls are a job object and the Untrusted integrity level, two of the four Windows mechanisms [Chromium's sandbox](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) relies on. The sandbox process makes a job for itself that [allows one process](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information), so any program it starts fails to start. The job also caps its committed memory at a gigabyte and [keeps it off the desktop](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_ui_restrictions): no clipboard, no window of another program's, no display settings, no desktop made or switched to. Its limits, in [`copper/src/sandbox.rs`](https://github.com/zootella/fuji/blob/main/copper/src/sandbox.rs):

```rust
limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_ACTIVE_PROCESS | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
limits.BasicLimitInformation.ActiveProcessLimit = 1;
limits.ProcessMemoryLimit = MEMORY_LIMIT;
```

Then it lowers its own integrity level to Untrusted, the level [below Low](https://learn.microsoft.com/en-us/windows/win32/secauthz/well-known-sids). A process at Low already cannot write to most of the user's files or registry, as [Microsoft explains](https://learn.microsoft.com/en-us/previous-versions/dotnet/articles/bb625960%28v=msdn.10%29). Below it, the sandbox process can write no file or registry key the user owns, and can neither read nor write another process's memory.

### Tested from inside

Fuji tries each wall with code running inside it. `fuji --sandbox-check`, in [`copper/src/sandbox.rs`](https://github.com/zootella/fuji/blob/main/copper/src/sandbox.rs), takes a folder and a file outside it, puts on itself the walls the sandbox process puts on, and tries every reach in turn. It lists the folder and reads a file in it, which the walls allow. Then it reads the file outside, lists the folder above, writes in the folder and in the temporary folder, connects to the machine itself, starts a program, reaches for a service of the system's and signals the process that started it, along with the reaches each platform adds: io_uring and a crash reporter's core on Linux, and the registry, another process's memory, the job's memory limit and Windows Error Reporting on Windows. It prints a line for each, the answer beside the one the walls should give.

A second test, the walk in [`copper/walk.js`](https://github.com/zootella/fuji/blob/main/copper/walk.js), drives a built Fuji through what ***Twinning*** promises. It reads the same folder in the main process and in the sandbox process, which must write the same answers line for line, and again with the folder named through a link, since the kernel judges a path by the place it resolves to. It stops a sandbox process partway. It crashes one partway and counts the crash reports the platform keeps before and after, a count that must not move. It holds one still, the way a read stuck in the kernel would hold it, and Fuji must still end it. And it kills the main process, which must take its sandbox process with it. The walk runs on the Mac and on Windows, and on Linux it runs in containers of Debian 12, Ubuntu 24.04 and Fedora 40 from one build, through [`linux/build.js`](https://github.com/zootella/fuji/blob/main/linux/build.js), so each distribution's own libraries load beneath the same walls.

### Run it yourself

The check ships in every copy of Fuji, so anyone can try the walls on their own machine. Open a terminal in your home folder, and name a folder of pictures and any file outside it. On a Mac:

```sh
/Applications/Fuji.app/Contents/MacOS/fuji --sandbox-check Pictures Documents/notes.txt
```

On Linux, with Fuji installed from its package:

```sh
fuji --sandbox-check Pictures Documents/notes.txt
```

Each line says what a reach answered, allowed or refused, and begins with ok when that is the answer the walls should give. On the Mac mini it prints:

```
ok allowed list the folder: 2 names
ok allowed read a file in it: 3045 bytes of Pictures/video.mkv
ok refused read a file outside it: Operation not permitted (os error 1)
ok refused list the folder above it: Operation not permitted (os error 1)
ok refused write a file in it: Operation not permitted (os error 1)
ok refused write a file in the temporary folder: Operation not permitted (os error 1)
ok refused connect to this machine: Operation not permitted (os error 1)
ok refused start a program: Operation not permitted (os error 1)
ok refused look up a service of the system's: launchservicesd, refused with 1100
ok refused signal the process that started it: Operation not permitted (os error 1)
```

The first two lines are the sandbox process reading the folder it was given. Every line after them is a reach that code from a file might make, for the user's other files, the disk, the network, a program, the system and Fuji itself, and each one meets a wall.

## Sources

Every address below was fetched on October 26, 2026.

- Andreev, Maxim. "[FFmpeg: Stealing Local Files with HLS+concat](https://seclists.org/oss-sec/2016/q1/85)." Forwarded to the oss-security mailing list by Vladimir Dubrovin, January 13, 2016.
- Android Open Source Project. "[Android Security Bulletin—May 2025](https://source.android.com/docs/security/bulletin/2025-05-01)." May 5, 2025.
- Austin, Dan, and Jeff Vander Stoep. "[Hardening the Media Stack](https://android-developers.googleblog.com/2016/05/hardening-media-stack.html)." *Android Developers Blog*, May 5, 2016.
- Beer, Ian, and Samuel Groß. "[A Deep Dive into an NSO Zero-Click iMessage Exploit: Remote Code Execution](https://projectzero.google/2021/12/a-deep-dive-into-nso-zero-click.html)." *Project Zero*, December 15, 2021.
- CERT Coordination Center. "[Android Stagefright Contains Multiple Vulnerabilities](https://kb.cert.org/vuls/id/924951)." Vulnerability Note VU#924951, July 28, 2015.
- Check Point Research. "[Hacked in Translation: From Subtitles to Complete Takeover](https://research.checkpoint.com/2017/hacked-in-translation/)." May 23, 2017.
- Chen, Raymond. "[What Does the COM Surrogate Do and Why Does It Always Stop Working?](https://devblogs.microsoft.com/oldnewthing/20090212-00/?p=19173)" *The Old New Thing*, February 12, 2009.
- Chromium Project. "[Memory Safety](https://www.chromium.org/Home/chromium-security/memory-safety/)." The Chromium Projects.
- Chromium Project. "[The Rule of 2](https://chromium.googlesource.com/chromium/src/+/main/docs/security/rule-of-2.md)." Chromium security documentation.
- Chromium Project. "[Sandbox](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md)." Chromium design documentation.
- Citizen Lab. "[BLASTPASS: NSO Group iPhone Zero-Click, Zero-Day Exploit Captured in the Wild](https://citizenlab.ca/blastpass-nso-group-iphone-zero-click-zero-day-exploit-captured-in-the-wild/)." September 7, 2023.
- Corbet, Jonathan. "[Task-Level io_uring Restrictions](https://lwn.net/Articles/1054225/)." *LWN.net*, January 19, 2026.
- Crashpad. "[exception_swallower.h](https://chromium.googlesource.com/crashpad/crashpad/+/main/test/mac/exception_swallower.h)." Crashpad source code.
- CVE Program. "[CVE-2019-11460](https://www.cve.org/CVERecord?id=CVE-2019-11460)." CVE Record.
- CVE Program. "[CVE-2020-15999](https://www.cve.org/CVERecord?id=CVE-2020-15999)." CVE Record.
- CVE Program. "[CVE-2025-59733](https://www.cve.org/CVERecord?id=CVE-2025-59733)." CVE Record.
- CVE Program. "[CVE-2025-59734](https://www.cve.org/CVERecord?id=CVE-2025-59734)." CVE Record.
- CVE Program. [*CVE List*](https://www.cve.org). The record of every CVE identifier on this page.
- Cybersecurity and Infrastructure Security Agency. "[Known Exploited Vulnerabilities Catalog](https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json)." Catalog version 2026.10.23, October 23, 2026.
- Cybersecurity and Infrastructure Security Agency. "[USB Malware Targeting Siemens Control Software (Update C)](https://www.cisa.gov/news-events/ics-advisories/icsa-10-201-01c)." ICS Advisory ICSA-10-201-01C, July 2010, revised January 8, 2014.
- Evans, Chris. "[\[0day\] \[exploit\] Advancing Exploitation: A Scriptless 0day Exploit against Linux Desktops](https://scarybeastsecurity.blogspot.com/2016/11/0day-exploit-advancing-exploitation.html)." *Security*, November 21, 2016.
- Evans, Chris. "[\[0day\] \[exploit\] Compromising a Linux Desktop Using... 6502 Processor Opcodes on the NES?!](https://scarybeastsecurity.blogspot.com/2016/11/0day-exploit-compromising-linux-desktop.html)" *Security*, November 14, 2016.
- Evans, Chris. "[\[0day\] \[PoC\] Risky Design Decisions in Google Chrome and Fedora Desktop Enable Drive-by Downloads](https://scarybeastsecurity.blogspot.com/2016/11/0day-poc-risky-design-decisions-in.html)." *Security*, November 15, 2016.
- Evans, Chris. "[Redux: Compromising Linux Using... SNES Ricoh 5A22 Processor Opcodes?!](https://scarybeastsecurity.blogspot.com/2016/12/redux-compromising-linux-using-snes.html)" *Security*, December 13, 2016.
- FFmpeg. "[FFmpeg Security](https://ffmpeg.org/security.html)." ffmpeg.org.
- FFmpeg. [*FFmpeg*](https://github.com/FFmpeg/FFmpeg). Source repository and commit history, mirrored on GitHub.
- Fuji. [*Fuji*](https://github.com/zootella/fuji). Source repository on GitHub, from which this page quotes `copper/src/media.rs`, `copper/src/sandbox.rs`, `copper/walk.js`, `desktop/libmpv/build.py`, `desktop/src-tauri/tauri.conf.json`, `linux/build.js`, `steel/fuzz/fuzz_targets/scout.rs`, `steel/src/lib.rs`, and `steel/src/tip.rs`.
- Fuji. "[fuji.dmg.json](https://fujidesktop.app/fuji.dmg.json)." Release record of the Mac download, fujidesktop.app.
- GNOME. "[gnome-desktop-thumbnail-script.c](https://gitlab.gnome.org/GNOME/gnome-desktop/-/raw/master/libgnome-desktop/gnome-desktop-thumbnail-script.c)." gnome-desktop source code.
- GNOME. "[tracker-1.11.2.changes](https://download.gnome.org/sources/tracker/1.11/tracker-1.11.2.changes)." Tracker release notes, December 8, 2016.
- GNOME. "[tracker-extract.c](https://gitlab.gnome.org/GNOME/tracker-miners/-/raw/master/src/tracker-extract/tracker-extract.c)." tracker-miners source code.
- GNOME. "[tracker-extract-decorator.c](https://gitlab.gnome.org/GNOME/tracker-miners/-/raw/master/src/tracker-extract/tracker-extract-decorator.c)." tracker-miners source code.
- Google. "[OSS-Fuzz](https://google.github.io/oss-fuzz/)." OSS-Fuzz documentation.
- Google. "[oss-fuzz/projects](https://github.com/google/oss-fuzz/tree/master/projects)." OSS-Fuzz repository on GitHub.
- Google. "[oss-fuzz/projects/ffmpeg](https://github.com/google/oss-fuzz/tree/master/projects/ffmpeg)." OSS-Fuzz repository on GitHub.
- Groß, Samuel. "[A Look at iMessage in iOS 14](https://projectzero.google/2021/01/a-look-at-imessage-in-ios-14.html)." *Project Zero*, January 28, 2021.
- Hawkes, Ben. "[The WebP 0day](https://blog.isosceles.com/the-webp-0day/)." *Isosceles*, September 21, 2023.
- International Telecommunication Union. [*Recommendation T.81: Information Technology, Digital Compression and Coding of Continuous-Tone Still Images, Requirements and Guidelines*](https://www.w3.org/Graphics/JPEG/itu-t81.pdf). September 1992.
- Jurczyk, Mateusz, and Gynvael Coldwind. "[FFmpeg and a Thousand Fixes](https://security.googleblog.com/2014/01/ffmpeg-and-thousand-fixes.html)." *Google Online Security Blog*, January 10, 2014.
- Klabnik, Steve, Carol Nichols, and Chris Krycho. "[Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)." In *The Rust Programming Language*. Rust Project.
- Klabnik, Steve, Carol Nichols, and Chris Krycho. "[References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)." In *The Rust Programming Language*. Rust Project.
- Langner, Ralph. "[Stuxnet's Secret Twin](https://foreignpolicy.com/2013/11/19/stuxnets-secret-twin/)." *Foreign Policy*, November 19, 2013.
- Lerner, Emil, and Pavel Cheremushkin. "[ffmpeg-avi-m3u-xbin](https://github.com/neex/ffmpeg-avi-m3u-xbin)." Proof of concept on GitHub, 2017.
- Linux Kernel Documentation. "[Landlock: Unprivileged Access Control](https://docs.kernel.org/userspace-api/landlock.html)." The Linux Kernel documentation.
- Linux Kernel Documentation. "[Seccomp BPF (SECure COMPuting with Filters)](https://docs.kernel.org/userspace-api/seccomp_filter.html)." The Linux Kernel documentation.
- Linux Man-Pages Project. "[PR_SET_DUMPABLE(2const)](https://man7.org/linux/man-pages/man2/PR_SET_DUMPABLE.2const.html)." Linux manual page.
- Liston, Tom. "[The WMF SETABORTPROC Vulnerability and Ilfak Guilfanov's Patch: A Technical Explanation of the Issues and Available Workarounds for the WMF Flaw](https://web.archive.org/web/20060824090717/http://handlers.sans.org:80/tliston/WMFTech.pdf)." SANS Internet Storm Center, January 5, 2006. Archived by the Internet Archive.
- LLVM Project. "[libFuzzer: A Library for Coverage-Guided Fuzz Testing](https://llvm.org/docs/LibFuzzer.html)." LLVM documentation.
- Marczak, Bill, John Scott-Railton, Bahr Abdul Razzak, Noura Aljizawi, Siena Anstis, Kristin Berdan, and Ron Deibert. "[FORCEDENTRY: NSO Group iMessage Zero-Click Exploit Captured in the Wild](https://citizenlab.ca/research/forcedentry-nso-group-imessage-zero-click-exploit-captured-in-the-wild/)." Citizen Lab, September 13, 2021.
- Meta. "[CVE-2025-27363](https://www.facebook.com/security/advisories/cve-2025-27363)." Security advisory, last updated March 13, 2025.
- Microsoft. "[ADV200006: Type 1 Font Parsing Remote Code Execution Vulnerability](https://api.msrc.microsoft.com/sug/v2.0/en-US/vulnerability?$filter=cveNumber%20eq%20%27ADV200006%27)." Microsoft Security Response Center, March 23, 2020.
- Microsoft. "[Designing Applications to Run at a Low Integrity Level](https://learn.microsoft.com/en-us/previous-versions/dotnet/articles/bb625960%28v=msdn.10%29)." Microsoft Learn.
- Microsoft. "[\_\_fastfail](https://learn.microsoft.com/en-us/cpp/intrinsics/fastfail?view=msvc-170)." Microsoft Learn.
- Microsoft. "[JOBOBJECT_BASIC_LIMIT_INFORMATION (winnt.h)](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information)." Microsoft Learn.
- Microsoft. "[JOBOBJECT_BASIC_UI_RESTRICTIONS (winnt.h)](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_ui_restrictions)." Microsoft Learn.
- Microsoft. "[Microsoft Security Bulletin MS04-028: Buffer Overrun in JPEG Processing (GDI+) Could Allow Code Execution (833987)](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2004/ms04-028)." September 14, 2004.
- Microsoft. "[Microsoft Security Bulletin MS06-001: Vulnerability in Graphics Rendering Engine Could Allow Remote Code Execution (912919)](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2006/ms06-001)." January 5, 2006.
- Microsoft. "[Microsoft Security Bulletin MS10-046: Vulnerability in Windows Shell Could Allow Remote Code Execution (2286198)](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2010/ms10-046)." August 2, 2010.
- Microsoft. "[Microsoft Security Bulletin MS11-006: Vulnerability in Windows Shell Graphics Processing Could Allow Remote Code Execution (2483185)](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2011/ms11-006)." February 8, 2011.
- Microsoft. "[SetAbortProc Function (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setabortproc)." Microsoft Learn.
- Microsoft. "[SetErrorMode Function (errhandlingapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-seterrormode)." Microsoft Learn.
- Microsoft. "[UpdateProcThreadAttribute Function (processthreadsapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute)." Microsoft Learn.
- Microsoft. "[Well-Known SIDs](https://learn.microsoft.com/en-us/windows/win32/secauthz/well-known-sids)." Microsoft Learn.
- Moravchick, Yuval, and Ori Hollander. "[PixelSmash: Critical FFmpeg Vulnerability Turns Media Files into Weapons](https://jfrog.com/blog/pixelsmash-critical-ffmpeg-vulnerability-turns-media-files-into-weapons/)." *JFrog Blog*, June 22, 2026.
- Mozilla. "[Sandbox.mm](https://github.com/mozilla-firefox/firefox/blob/main/security/sandbox/mac/Sandbox.mm)." Firefox source code.
- Mozilla. "[SVG as an Image](https://developer.mozilla.org/en-US/docs/Web/SVG/Guides/SVG_as_an_image)." *MDN Web Docs*.
- mpv. "[mpv Manual](https://mpv.io/manual/stable/)." mpv.io.
- Niedermayer, Michael. "[avformat/hls: Check Local File Extensions](https://github.com/FFmpeg/FFmpeg/commit/189ff4219)." FFmpeg commit 189ff4219, June 3, 2017.
- Nocera, Bastien. "[Security for the Security Gods! Sandboxing for the Sandboxing Throne](https://www.hadess.net/2017/07/security-for-security-gods-sandboxing.html)." July 21, 2017.
- Paddock, Brandon. "[FAQ: Why Does WDS / Windows Vista Use So Many Processes?](https://web.archive.org/web/20260920044830/https://brandonlive.com/2007/06/20/faq-why-does-wds-windows-vista-use-so-many-processes/)" *BrandonLive*, June 20, 2007. Archived by the Internet Archive.
- Rudra, Sourav. "[FFmpeg Calls Google's AI Bug Reports 'CVE Slop'](https://itsfoss.com/news/ffmpeg-google-fiasco/)." *It's FOSS*, November 14, 2025.
- Rust Fuzz Book. "[Fuzzing with cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html)." *Rust Fuzz Book*.
- Rust Project. "[Clippy Documentation](https://doc.rust-lang.org/clippy/)."
- Rust Project. "[Lint Levels](https://doc.rust-lang.org/rustc/lints/levels.html)." In *The rustc Book*.
- Sista, Srinivas. "[Stable Channel Update for Desktop](https://chromereleases.googleblog.com/2023/09/stable-channel-update-for-desktop_11.html)." *Chrome Releases*, September 11, 2023.
- Sylvain, Nicolas. "[A New Approach to Browser Security: The Google Chrome Sandbox](https://blog.chromium.org/2008/10/new-approach-to-browser-security-google.html)." *Chromium Blog*, October 2, 2008.
- Tauri. "[Capabilities](https://v2.tauri.app/security/capabilities/)." Tauri documentation.
- Tauri. "[Content Security Policy (CSP)](https://v2.tauri.app/security/csp/)." Tauri documentation.
- Thomas, Gavin. "[A Proactive Approach to More Secure Code](https://www.microsoft.com/en-us/msrc/blog/2019/07/a-proactive-approach-to-more-secure-code)." *Microsoft Security Response Center Blog*, July 16, 2019.
- Vander Stoep, Jeff. "[Rust in Android: Move Fast and Fix Things](https://blog.google/security/rust-in-android-move-fast-fix-things/)." *The Keyword*, Google, November 13, 2025.
- Vander Stoep, Jeff, and Alex Rebert. "[Eliminating Memory Safety Vulnerabilities at the Source](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html)." *Google Online Security Blog*, September 25, 2024.
- Vander Stoep, Jeff, and Chong Zhang. "[Queue the Hardening Enhancements](https://security.googleblog.com/2019/05/queue-hardening-enhancements.html)." *Google Online Security Blog*, May 9, 2019.
- Vue.js. "[Security](https://vuejs.org/guide/best-practices/security.html)." Vue.js documentation.
- W3C. [*Portable Network Graphics (PNG) Specification (Third Edition)*](https://www.w3.org/TR/png-3/). Edited by Chris Blume, Pierre-Anthony Lemieux, Chris Lilley, Chris Needham, Leonard Rosenthol, Chris Arley Seeger, Simon Thompson, and Cosmin Truta. W3C Recommendation, June 24, 2025.
- Zimperium. "[Experts Found a Unicorn in the Heart of Android](https://www.zimperium.com/blog/experts-found-a-unicorn-in-the-heart-of-android/)." July 27, 2015.
