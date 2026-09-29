# File Types

A file's extension is the shortest history lesson on a computer. Three or four letters after a dot say which company or committee made the format, which machine it was made for, roughly what year, and what people were doing with pictures, video and sound at the time. This page reads that history out of the extensions a collection of multimedia files accumulates: from the bulletin boards and CD-ROMs of the late 1980s, through the dial-up web, Napster and DivX, to the phones, Discord and Reddit of today.

This page is for a reader with a drive full of these who wonders what half of them are. So each section says where a format came from, what it was for, what people used it for instead, and where it stands now, and links to the specification or the account that tells the longer story. It is not a reference to any one program's abilities, and it does not say what will open a file today; that changes, and the history does not.

The formats fall into four rooms: pictures, video, sound, and the small text files that sit beside the media and tell a player what to do with it. Within each room they are roughly in the order a collector met them.

## Pictures

### BMP and DIB, Windows' own bitmap

Windows drew its first pictures in a format it invented for itself. The bitmap, `.bmp`, arrived with Windows 1.0 in 1985 and took the form most files still have with Windows 3.0 in 1990: a short header, a palette if the picture has one, and then the pixels, row by row, bottom row first, almost always uncompressed. That simplicity was the design. Any program on any machine can read a BMP with a page of code, and Windows itself keeps its wallpaper, its icons' insides and Paint's pictures in it. The cost is size: a 1024 by 768 screenshot is two and a quarter megabytes, however plain the screen was.

Through the 1990s it was the picture format a Windows user made without choosing one. Paint saved it, Print Screen made one, scanners' bundled software wrote it, and a folder of wallpaper was a folder of them. It never took hold on the web, where every byte crossed a modem, and by the end of the decade GIF, JPEG and then PNG had taken the jobs it was too large for. A BMP today usually came from old software, a scanner, or a program that wanted the simplest possible file.

`.dib`, the device-independent bitmap, is the same picture under the name Windows 3.0 gave the format's data structure, meaning one that could travel between displays without knowing their color depth. A few of Windows' own tools and some others wrote the name, rarely, and inside it is a BMP, sometimes without the fourteen-byte file header a `.bmp` starts with. [Microsoft's documentation](https://learn.microsoft.com/en-us/windows/win32/gdi/bitmap-storage) still describes both, and [the bitmap's history](https://en.wikipedia.org/wiki/BMP_file_format) is on Wikipedia.

### GIF

CompuServe was an online service before there was a public internet, and in 1987 it needed a picture format its subscribers could download over a 2400 baud modem and view on any of the incompatible computers of the day. The Graphics Interchange Format was the answer: up to 256 colors from a palette, compressed with LZW, with a 1989 revision that added animation and a transparent color. [The GIF89a specification](https://www.w3.org/Graphics/GIF/spec-gif89a.txt) is still hosted by the W3C, unchanged.

When Mosaic showed inline pictures in 1993, GIF was the format it showed, and for the first years of the web it was the web's only picture format that every browser drew. Logos, buttons, "under construction" signs, and the first animated banners were all GIFs, and so were the pictures traded on bulletin boards before the web, where a `.gif` of 640 by 480 in 256 colors was the state of the art. Its limits are what they always were: 256 colors per frame, transparency that is all or nothing, and files that grow enormous for anything like video.

Two later chapters made it famous twice more. In December 1994 Unisys, which held the LZW patent, announced it would collect royalties on software that wrote GIFs, and the free software world answered with PNG within a year. The patent expired in 2003 and 2004, and by then the web had two picture formats. Then, from the mid-2000s, GIF became the web's format for a few seconds of looping video, the reaction GIF, because it was the one moving picture every browser, forum and phone would play without a plugin. The video formats built for the job, WebM and MP4, have taken most of that traffic since, but the word stuck. [GIF's Wikipedia article](https://en.wikipedia.org/wiki/GIF) has the full history, including the argument about how to say it.

### JPEG, in four spellings

The Joint Photographic Experts Group published its standard in 1992, and it made photographs small enough for the disks, modems and web of the 1990s. A JPEG discards detail the eye barely notices, mostly fine color variation and high-frequency texture, and the encoder's quality setting decides how much. At the usual settings a photograph shrinks to a tenth of its raw size and looks the same; pushed hard, it grows blocky, and the discarding shows around sharp edges and text as a faint halo. Each save discards again, so a picture edited and saved ten times has been through the process ten times. It holds no transparency and no layers. [The JPEG standard](https://jpeg.org/jpeg/) is ITU T.81, and [how JPEG compresses a picture](https://en.wikipedia.org/wiki/JPEG) is explained well on Wikipedia.

The standard described how to compress a picture and not how to store one in a file, so in 1991 Eric Hamilton of C-Cube Microsystems gathered about forty companies and wrote [the JPEG File Interchange Format](https://www.w3.org/Graphics/JPEG/jfif.pdf), JFIF, the layout almost every JPEG follows inside. Digital cameras later added a second layout, Exif, with the camera's settings, and most files now carry both.

The spellings are the history of file names. `.jpg` is JPEG cut to three letters to fit the eight-and-three file names of DOS and Windows 3.1, and it became the common spelling because that is where most people made their files. `.jpeg` is the full name, used on the Mac and Unix, which had no such limit, and still by some web tools and phones. `.jpe` is a rarer three-letter cut, which some older programs wrote and most still open. `.jfif` names the layout rather than the compression, and was almost unknown as an extension until around 2019, when a registry value in Windows began pointing browsers at it as the extension for the `image/jpeg` type, and pictures saved from web pages started arriving with a name that some programs did not recognize. Inside, all four are the same file.

### PNG

The Portable Network Graphics format took shape in public, on a Usenet newsgroup, in the first weeks of 1995, in direct response to the Unisys patent announcement about GIF a week earlier. Thomas Boutell proposed a free replacement on January 4, an informal working group formed, the group produced seven drafts by February, and working libraries shipped on May 1. [The libpng project's history](http://www.libpng.org/pub/png/pnghist.html) tells it from the inside, and [the PNG specification](https://www.w3.org/TR/png/) became a W3C recommendation in 1996.

The design was better than the format it replaced. PNG is lossless, so every pixel comes back exactly as it was saved. It holds millions of colors rather than 256, and an alpha channel, so transparency can be smooth rather than all or nothing. Its compression, deflate, was patent-free and about ten percent better than GIF's. It left out animation deliberately, and that omission is why animated GIFs outlived the format meant to end them; an animated extension arrived in 2008 and spread slowly.

Browsers were slow too. Internet Explorer drew PNGs badly, and drew alpha transparency wrong, until version 7 in 2006, so for a decade web designers made GIFs they would rather have made PNGs. Once that ended PNG became the format of everything that is not a photograph: screenshots, logos, icons, diagrams, interface graphics, and anything with a transparent background. A photograph saved as PNG comes out several times larger than as JPEG, which is the one job it is wrong for. [PNG's Wikipedia article](https://en.wikipedia.org/wiki/PNG) has the rest.

### SVG

Scalable Vector Graphics is the W3C's picture format for shapes rather than pixels, written in XML, and a recommendation since 2001. An SVG holds lines, curves, fills, gradients and text, so it stays sharp at any size and is often tiny, and because it is text a designer can edit one by hand. [The SVG specification](https://www.w3.org/TR/SVG2/) is long, because the format can also hold animation, filters, and script.

Browsers were slow to take it up. Adobe shipped a plugin, Firefox and Safari drew SVG natively from 2005 and 2008, and Internet Explorer joined with version 9 in 2011, at which point the format became common for icons, logos, charts and diagrams, and for the illustrations that scale from a phone to a wall. It suits nothing photographic, and a picture from a camera converted to SVG is a large file of little squares. [SVG's history](https://en.wikipedia.org/wiki/SVG) begins with two competing proposals of 1998.

### WebP

Google bought On2 Technologies in 2010 for its VP8 video codec, released the codec as open source, and the same year cut a picture format from it: a WebP is a single VP8 frame, with lossless pictures, transparency and animation added over the following two years. Google's own [WebP documentation](https://developers.google.com/speed/webp) is the specification. The promise was pictures a quarter to a third smaller than JPEG or PNG at the same quality, and Google pushed it across the web through Chrome, Android and its own sites.

For most of the decade a WebP saved from a web page would not open anywhere else. Photoshop, Windows and the Mac did not read it, and Safari did not show it, so a picture that was WebP on the page arrived on the desktop as a file nothing recognized. Collectors remember it that way, and the memory is fading: every major browser has drawn it since Safari added it in 2020, and the operating systems followed. [WebP's timeline](https://en.wikipedia.org/wiki/WebP) is on Wikipedia.

### AVIF

The Alliance for Open Media is the group of Google, Netflix, Mozilla, Amazon, Microsoft, Apple and others that made the AV1 video codec, royalty-free, to replace H.264 and HEVC on the web. The AV1 Image File Format, 2019, is what a single frame of AV1 looks like as a picture, in the same container HEIF uses. [The AVIF specification](https://aomediacodec.github.io/av1-avif/) is the alliance's.

An AVIF is far smaller than a JPEG of the same quality, with room for high dynamic range, wide color and transparency. Browsers added it between 2020 and 2022, and image services on the web now send it widely to browsers that ask for it, so a picture saved from a web page is increasingly one. It is slow to encode, and software older than a few years cannot open it, which are the growing pains every new picture format has had. [AVIF's Wikipedia article](https://en.wikipedia.org/wiki/AVIF) covers the standard.

### HEIC and HEIF

The High Efficiency Image File Format is MPEG's container for pictures, standardized in 2015, built to hold a still frame of HEVC video and much else besides: bursts, depth maps, live photos, several pictures in one file. Apple chose it for iPhone photos in iOS 11, 2017, and the `.heic` extension names the HEVC-coded form Apple writes; `.heif` is the container's own name, which some Android phones and converters use, and which can also hold AV1 or JPEG inside. [Nokia's HEIF site](https://nokiatech.github.io/heif/) documents the format, and [Apple's ImageIO](https://developer.apple.com/documentation/imageio) reads it natively.

A HEIC is about half the size of a JPEG at the same quality, with wide color, which is why phones chose it and why their camera rolls fill with it. Its trouble is on the other end: HEVC carries patent licensing, so a Windows PC could not open a HEIC without installing two extensions from the Microsoft Store, one of them paid, and most Windows software still cannot. Phones know this, and convert to JPEG when you share a photo or copy it to a computer, so a folder of HEICs usually came straight off the phone. [HEIF's Wikipedia article](https://en.wikipedia.org/wiki/High_Efficiency_Image_File_Format) has the details.

### PCX

Before Windows, the PC's picture format was ZSoft's. PC Paintbrush, 1984, was the DOS paint program that came bundled with Microsoft's mouse, and its PiCture eXchange format, 1985, was what it saved. Windows 1.0's Paint was a licensed cut of the same program, and Windows 3.0's Paintbrush wrote `.pcx` as its native format, so a picture drawn on a PC before 1995 was very likely one. Scanners and clip art collections came in it, and the games of the early 1990s, Doom among them, kept their screens and artwork in it. [The Library of Congress](https://loc.gov/preservation/digital/formats//fdd/fdd000585.shtml) keeps the format's description, and [PCX's history](https://en.wikipedia.org/wiki/PCX) is on Wikipedia.

A PCX holds up to 256 colors from a palette, or true color in its last version, compressed with a light run-length scheme that suited screens of flat color. By Windows 95, BMP, GIF and JPEG had each taken one of its jobs, and Paint dropped it. Photoshop and the shareware viewers still open one; little else does.

### Targa

Truevision's Targa is older than most of what surrounds it, and more capable than its age suggests. AT&T's EPICenter, an internal venture that became Truevision in a 1987 buyout, defined the format in 1984 for its video capture and display boards, the Targa cards that put true color on a PC screen years before the PC could draw it. So a `.tga` held 24-bit color and an 8-bit alpha channel from the start, when other formats had 16 colors. [The Encyclopedia of Graphics File Formats](https://www.fileformat.info/format/tga/egff.htm) describes it, and [Targa's corporate story](https://en.wikipedia.org/wiki/Truevision_TGA) is on Wikipedia.

That made it the file of 3D rendering and video work through the 1990s, when a rendered frame needed exact color and an alpha for compositing, and afterward the file of game textures and screenshots: Quake III's textures were TGAs, and a generation of game tools wrote them. It is simple, usually uncompressed or lightly run-length compressed, and large, and no browser has ever shown one. 3D tools still write it, and most graphics software still opens it.

### TIFF, in two spellings

Aldus, the PageMaker company, made the Tagged Image File Format in 1986 with Microsoft, as the one format every scanner could write and every desktop publishing program could read. Adobe has owned it since buying Aldus in 1994, and the specification, TIFF 6.0, has not changed since 1992; [the Library of Congress](https://www.loc.gov/preservation/digital/formats/fdd/fdd000022.shtml) keeps its description. Inside, a TIFF is a set of tags rather than one layout, each saying something about the picture, and that flexibility is the format's character: a file can hold several pages, layers, any of a dozen compressions, or none, and a tag some program invented that no other program knows. A TIFF is lossless as a rule, often enormous, and not every TIFF opens everywhere.

It is the format of scanners, fax, print and archives, and of any workflow where the picture must survive editing intact. The Mac's own screenshots were TIFFs for years, and Apple's software still writes the format readily. Browsers never drew it, apart from Safari, which always has. `.tif` is the three-letter spelling DOS and Windows needed; `.tiff` is the full one, used on the Mac and Unix. [TIFF's many variants](https://en.wikipedia.org/wiki/TIFF) are cataloged on Wikipedia.

### PICT and PCT

The Macintosh of 1984 drew its screen with QuickDraw, and PICT was a recording of QuickDraw's drawing calls, so a picture could be shapes and text as well as pixels, and scaled cleanly when it was. It was the Mac's own picture and clipboard format for the whole of the classic era: what a Mac program pasted, what MacDraw and ClarisWorks saved, and what the clip art CDs sold to Mac users held. On a PC or a DOS-friendly disk the same file was a `.pct`, since PICT has four letters.

Mac OS X moved the Mac's drawing to PDF in 2001, and Apple let PICT go by degrees over the following twenty years, so newer Macs open one less and less reliably, and no PC ever did without a converter. [PICT's two versions](https://en.wikipedia.org/wiki/PICT) are described on Wikipedia, and [Apple's old documentation](https://developer.apple.com/library/archive/documentation/mac/QuickDraw/QuickDraw-2.html) the drawing model it recorded.

### IFF and LBM, Deluxe Paint's picture

Electronic Arts published the Interchange File Format in 1985 for the Amiga, as a general way to build files from tagged chunks, and its picture form, ILBM for interleaved bitmap, was what Deluxe Paint saved. Deluxe Paint was the Amiga's paint program and then the game industry's: EA brought it to the PC in 1988, and most game artists of the late 1980s and 1990s drew their backgrounds and sprites in it, so the pixel art of that era was an ILBM before it was anything else. The Amiga's demo scene and its picture collections were in it too. [The Amiga's own documentation](https://wiki.amigaos.net/wiki/ILBM_IFF_Interleaved_Bitmap) describes the format, and [ILBM's history](https://en.wikipedia.org/wiki/ILBM) is on Wikipedia.

An ILBM holds a palette of up to 256 colors, or the Amiga's HAM modes with thousands, stored as bitplanes rather than bytes per pixel, which suited the Amiga's hardware and nobody else's. `.iff` is the Amiga's name for the file, and `.lbm` the three-letter spelling Deluxe Paint used on DOS. Almost nothing on a PC or a Mac opened one at the time, and only the specialist viewers do now. The IFF idea outlived the format: RIFF, the structure of WAV and AVI, is Microsoft's copy of it, and AIFF, the Mac's sound file, is Apple's.

### WMF and EMF, the Windows Metafile

A Windows Metafile is a recording of calls to Windows' own drawing system, GDI, so a picture in it is lines, shapes and text that scale, or a bitmap, or both. It came with Windows 3.0 in 1990, and its 32-bit successor, the Enhanced Metafile, with Windows NT in 1993, adding the precision and features the old one lacked. [Microsoft publishes both specifications](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/), and [the metafile's history](https://en.wikipedia.org/wiki/Windows_Metafile) is on Wikipedia.

They were the format of clip art. The CDs of ten thousand images sold in the 1990s were mostly WMF, the gallery inside Office was, and Office still pastes a drawing between its programs as an EMF. A 2005 flaw in how Windows drew a WMF, which let a picture run code, made the format briefly famous outside the people who used it. Office and its open source cousins open both; browsers and the Mac never did.

### PSD

Photoshop's own document has existed since the program's first version in 1990, and it holds the working picture rather than the finished one: layers, masks, text still editable, adjustments not yet applied, paths, and the picture's whole history of decisions. A file is many times a JPEG's size and only Photoshop shows all of it. [Adobe publishes the format](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/), and it is complicated enough that most other programs read only the flattened copy Photoshop stores beside the layers, which is how a Mac shows a PSD's preview and how a web browser never has.

Designers work in it and share templates and mockups in it, and it is the only format on this page that is a workspace rather than a picture. [Photoshop's Wikipedia article](https://en.wikipedia.org/wiki/Adobe_Photoshop#File_format) has the outline.

### Camera raw: DNG, CR2, NEF and ARW

A digital camera's sensor records more than a JPEG can hold, and from the late 1990s serious cameras offered to save that data unprocessed, as a raw file, so the exposure, white balance and colors could be decided later on a computer. Every maker invented its own format. Nikon's NEF dates from the D1 of 1999, Canon's CR2 from 2004 and lasted until CR3 replaced it in 2018, and Sony's ARW from the Alpha cameras that began in 2006, after Sony bought Konica Minolta's camera business. Each changes a little with every model, so photo software and operating systems lag each new camera by months, and an old program may not read a new file. [Wikipedia's article on raw formats](https://en.wikipedia.org/wiki/Raw_image_format) lists the dozens of others.

Adobe's Digital Negative, 2004, set out to end that. [DNG](https://en.wikipedia.org/wiki/Digital_Negative) is an open raw format built on TIFF, documented by Adobe and free to use, and Adobe's converter turns any maker's raw into it. Leica and Pentax write it directly, and so do phones: Apple's ProRAW and Android's raw mode are DNG. Most camera makers kept their own formats anyway. A raw file of any kind holds a small JPEG preview inside, and a plain viewer shows that preview, if it shows anything.
## Video

Video on a personal computer is younger than sound or pictures, for a plain reason: a second of uncompressed standard-definition video is about twenty megabytes, and a 1991 hard drive held forty. Every format below is an answer to that arithmetic, and the answers changed as the drives and the networks did. The container, which is the file's shape, and the codec, which is how the pictures are squeezed, are separate things, and most of the trouble a collector has with old video is a familiar container holding a codec nothing decodes anymore.

### QuickTime: MOV and QT

Apple shipped QuickTime in December 1991, and it put moving pictures on a personal computer's screen for the first time in a way an ordinary user could make and play, in a window the size of a postage stamp, at about fifteen frames a second. The file format it defined, the QuickTime movie, was a container of tracks, video, sound, text, each with its own timing, held in a tree of atoms, and it was flexible enough that MPEG adopted it, nearly unchanged, as the basis of MP4 a decade later. [Apple's specification](https://developer.apple.com/documentation/quicktime-file-format) is still the reference, and [QuickTime's history](https://en.wikipedia.org/wiki/QuickTime) is on Wikipedia.

A `.mov` from the early 1990s holds Cinepak, or Apple's own Video and Animation codecs, or Sorenson Video from 1998, which little modern software decodes; one from an iPhone or a recent camera holds H.264 or HEVC with AAC sound, the same thing an MP4 holds under another name. `.qt` is the earliest spelling, from before `.mov` became the habit, and old CD-ROMs and Mac collections carry them. Apple's software has always played both; elsewhere it depends on the codec inside, and not every program recognizes the name.

### MP4, M4V, 3GP and 3G2

MPEG-4 Part 14, standardized in 2003 on the foundation Part 12 had laid in 2001, is the QuickTime container generalized and made a standard, and it became the one video file nearly everything writes and plays. Inside an MP4 is most often H.264 video and AAC sound, with subtitles and chapters beside them; the container is simple, and the codecs inside carry patent licensing, which is why the web's video tag plays an MP4 in its H.264 form on every browser and why a file from a recent phone may hold HEVC that older software cannot decode. [MP4's Wikipedia article](https://en.wikipedia.org/wiki/MP4_file_format) has the standard's structure.

Three relatives share the container. `.m4v` is Apple's name for an MP4 of video, from the iTunes Store and the video iPod of 2005; inside it is an ordinary MP4 unless it came from the store, in which case it carries Apple's FairPlay protection and plays only in Apple's software on an authorized machine. HandBrake and the ripping tools of the era wrote the unprotected kind. `.3gp` is the 3GPP standards group's cut of MP4 for the phones before smartphones, around 2003: a tiny file built for a flip phone's camera and its data plan, with H.263 or MPEG-4 video at 176 by 144 and AMR speech-quality sound, and most clips shot on a phone between 2003 and 2010 are one. `.3g2` is the 3GPP2 twin for the phones on CDMA networks, Verizon and Sprint in the United States, and plays wherever `.3gp` does. [3GPP](https://www.3gpp.org/) publishes the standards, and [the 3GP and 3G2 article](https://en.wikipedia.org/wiki/3GP_and_3G2) has the details.

### AVI and DivX

Microsoft's answer to QuickTime was Video for Windows, in November 1992, and its file was the Audio Video Interleave: a RIFF container, the same chunked structure as WAV, with frames of video and blocks of sound interleaved so a slow CD-ROM drive could read both in order. The first release came with Microsoft's own Video 1 codec and Intel's Indeo; Cinepak arrived a year later. An AVI of the early 1990s is a postage stamp of one of those, playing unevenly from a CD-ROM, and it was the first video most PC users ever played. [AVI's Wikipedia article](https://en.wikipedia.org/wiki/Audio_Video_Interleave) has the format, and [Microsoft's AVI reference](https://learn.microsoft.com/en-us/windows/win32/directshow/avi-riff-file-reference) still documents it.

The container's second life began in 1999, when a French graphic designer, Jérôme Rota, could not play his portfolio after Microsoft changed its MPEG-4 codec, and with a friend reverse-engineered the old one in about a week. The result, DivX ;-) 3.11 Alpha, put a DVD's film into an AVI that fit on a CD, and with broadband arriving it became the format people traded films in. The company that formed around it wrote a clean codec from scratch, and an open source fork of that became Xvid, so an AVI from the 2000s is often a whole film in one of the two. `.divx` is the extension the company's own tools wrote; most DivX films kept the `.avi` name, and DVD players of the mid-2000s with a DivX logo on the front played both. [DivX's Wikipedia article](https://en.wikipedia.org/wiki/DivX) tells the story.

An AVI is a box that can hold any codec at all, which is why a given AVI may play nowhere, and why players that carry every decoder with them exist.

### MPEG-1 and MPEG-2: MPG, MPEG, MPE, DAT and VOB

The Moving Picture Experts Group's first standard, MPEG-1, was published in 1993, and it was the first video format that was truly a standard: a file played on any machine with the decoder, whoever made either. The group designed it for the CD-ROM's bitrate, about 1.5 megabits a second, at 352 by 240, and that is what it looks like. Video CD, the disc format of the same year, stores films as MPEG-1; the first digital cameras that shot movies wrote it; and it was the video of the clips traded on CD-ROM and the early web. MPEG-2, 1995, raised the resolution and the bitrate for DVD and broadcast television, and both share a file extension. [MPEG-1's Wikipedia article](https://en.wikipedia.org/wiki/MPEG-1) covers the standard and its audio layers, which are a story of their own below.

The spellings follow the usual pattern. `.mpg` is DOS's three letters and the common one; `.mpeg` the full name, used on the Mac, Unix and the web; `.mpe` a third, rarer cut that some Unix tools and web servers wrote. Nearly everything then and now plays all three, once it recognizes the name.

Two more are what the discs look like copied to a drive. A Video CD stores its film in an MPEGAV folder as `.dat` files, MPEG-1 in a form nearly any player reads as an `.mpg`; Video CDs carried films and karaoke across Asia through the 1990s and 2000s, and a disc copied to a drive brings them along. The name is the trouble, since countless programs call their own files `.dat`, and a `.dat` is video only when it came from a disc. A DVD stores its film as `.vob` files, Video Objects, in a VIDEO_TS folder: MPEG-2 with Dolby Digital sound, subtitles and menus, cut into pieces of a gigabyte. They are what a DVD looks like copied straight to a drive, and many were, before ripping to a smaller file became the habit. A player that reads MPEG-2 plays one on its own; the menus work only from the folder. [The DVD-Video article](https://en.wikipedia.org/wiki/DVD-Video) has the disc's structure.

### Transport streams: TS, M2TS and MTS

MPEG-2 defined two ways to carry its video. The program stream, which DVD uses, assumes the file arrives intact; the transport stream assumes it does not, and is built for broadcast, with small fixed packets that a receiver can pick up mid-stream and error-check. Digital television everywhere is a transport stream, so a `.ts` file is a broadcast recorded off the air or from a satellite box, or a piece of a live internet stream saved to disk, holding MPEG-2 or H.264. The same three letters name a TypeScript program, so a programmer's folder has `.ts` files that are not video at all. [The transport stream's packet structure](https://en.wikipedia.org/wiki/MPEG_transport_stream) is described on Wikipedia.

Blu-ray and the AVCHD camcorder format of 2006, from Sony and Panasonic, use the same stream with a small timing header, called `.m2ts`. A camcorder wrote it to its card as `.mts`, and copied through the maker's software it became `.m2ts`, copied by hand it stayed `.mts`, so footage copied straight from the card usually carries the shorter name. Inside is H.264 at full high definition, and the camcorder footage of the late 2000s and Blu-ray copies are in it. Windows plays it; the Mac's own software wanted it converted first, and iMovie converted it on import. [The AVCHD site](https://www.avchd-info.org/) has the format's specification.

### DV

MiniDV was the digital camcorder tape of 1995 to about 2005, and its stream, DV, was lightly compressed video at standard television resolution, about 25 megabits a second, thirteen gigabytes an hour. A computer captured it over FireWire, which Apple built into the iMac DV of 1999 for the purpose, usually into an `.avi` on Windows or a `.mov` on the Mac, but sometimes as the bare stream, `.dv`. It is the home video of a decade, and a captured tape is one of these or its wrapped form. [DV's Wikipedia article](https://en.wikipedia.org/wiki/DV_(video_format)) covers the tape and the codec together.

### Windows Media: ASF, WMV, WTV and DVR-MS

Microsoft built its own streaming platform in 1996, NetShow, and the Advanced Systems Format was its container, made for a file that could be played while it was still arriving. Every Windows Media file is an ASF inside; `.asf` as an extension is usually an early Windows Media video, from before Microsoft gave them their own name, or a stream someone saved. Windows Media Video, `.wmv`, arrived in 1999 with the codec of the same name, and for the years when Windows Media Player came with the operating system and web video was a choice between it, RealPlayer and QuickTime, it was one of the three. Windows Movie Maker wrote it, so many home videos of the 2000s are in it, and some files carry the copy protection of the early music and film stores. Windows still plays it; a Mac needed a plugin, Flip4Mac, and everything else needs a decoder that knows the format. [Microsoft's ASF specification](https://learn.microsoft.com/en-us/windows/win32/wmformat/overview-of-the-asf-format) and [the Windows Media Video article](https://en.wikipedia.org/wiki/Windows_Media_Video) have the details.

Two more are Windows Media Center's. The Media Center edition of Windows XP, 2002, made a PC with a tuner card into the living room's video recorder, and it recorded television as `.dvr-ms`, MPEG-2 in an ASF wrapper, often with copy protection. Vista's TV Pack of 2008 replaced it with `.wtv`, Windows Recorded TV Show, at several gigabytes an hour. Media Center played both until Windows 10 retired it in 2015; other players read the unprotected ones. [Media Center's story](https://en.wikipedia.org/wiki/Windows_Media_Center) is on Wikipedia.

### RealMedia: RM and RMVB

RealNetworks was Progressive Networks when it shipped RealAudio in 1995, and it added RealVideo in February 1997, so RealMedia, `.rm`, is RealVideo and RealAudio in one file, made for streaming over a modem. A whole clip was a small blurry window and a lot of buffering, and it was how the web watched anything before broadband, through RealPlayer. The codecs were proprietary and changed with each version, and RealNetworks' own player was for years the only thing that decoded them. [RealVideo's versions and their names](https://en.wikipedia.org/wiki/RealVideo) are listed on Wikipedia.

RealMedia Variable Bitrate, `.rmvb`, is the 2000s form, from RealVideo 9 in 2002: it spent its bits where the picture needed them, and so fit a film on a CD at a quality DivX could not match at the time. That made it the standard for video traded across China and among anime fansub groups for most of that decade. Nothing native to an operating system ever played either; FFmpeg's developers reverse-engineered the decoders, and the players built on FFmpeg are what open one now.

### Flash: SWF, FLV and F4V

FutureSplash Animator, 1996, drew vector animations that played in a browser plugin, and Macromedia bought it that year and called it Flash. The Shockwave Flash file, `.swf`, was a program with drawings and sound in it rather than a picture or a video: it held the shapes, the timeline, and, from 2000, the ActionScript that made a game a game. It was the animation and game format of the web from the late 1990s until the 2010s, the file behind Newgrounds, the cartoons and games passed around by email, and the intro pages of the era's corporate sites. Adobe, which bought Macromedia in 2005, ended Flash Player at the end of 2020, and a SWF now runs in an emulator such as [Ruffle](https://ruffle.rs/) or not at all. [SWF's Wikipedia article](https://en.wikipedia.org/wiki/SWF) covers the format and [Adobe's specification](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf) is still published.

Flash Video, `.flv`, came with Flash MX in 2002 and was the file inside every Flash video player on the web. YouTube served its videos as `.flv` from 2005 until around 2010, in Sorenson Spark and then On2's VP6, so a video saved from YouTube in those years is usually one. In 2007 Adobe moved Flash to the MP4 container for H.264, and called the result `.f4v`, an MP4 under a Flash name, which renaming to `.mp4` usually makes play anywhere. Flash's end finished both, and the players with their own decoders are what open an `.flv` now. [Flash Video's article](https://en.wikipedia.org/wiki/Flash_Video) has the two formats' histories.

### Ogg video: OGV and OGM

The Xiph.Org Foundation makes free codecs and a free container, Ogg, to carry them, and Ogg has held video twice. Ogg Media, `.ogm`, 2002, was the Ogg container stretched by a third party to hold video, several audio tracks and subtitles at once, which nothing else did at the time, and it was the fansub and rip format for a few years until Matroska, built for that job, replaced it. Ogg Video, `.ogv`, is Xiph's own, most often holding Theora, the free codec built on On2's VP3 that Wikipedia and Firefox backed for the web's video tag around 2009. Theora lost to H.264 and then to WebM, Chrome dropped it in 2024, and a file from that brief moment now needs a player with its own decoder. Both are rare, and mostly from Wikipedia downloads and free software demos. [Xiph's Ogg documentation](https://xiph.org/ogg/) and [Theora's Wikipedia article](https://en.wikipedia.org/wiki/Theora) cover them.

### Matroska and WebM: MKV, MKA and WEBM

Matroska was announced in December 2002 as a fork of an earlier open container project, over a disagreement about whether to build the format on a binary form of XML, and the fork won. An `.mkv` holds any video and sound at all, with several audio tracks, subtitles and chapters in the same file, and that made it the format of anime fansubs, which needed all four, and of the high-quality rips traded since. No video site sends it and Apple's players do not open it, but Windows has since 2015, and it is often the container of a collector's best copy of a film or a series. `.mka` is the same container holding sound alone, a whole album in one file with chapters for tracks, or a film's soundtrack in several languages. [Matroska's specification](https://www.matroska.org/technical/elements.html) is public, and [Matroska's history](https://en.wikipedia.org/wiki/Matroska) is on Wikipedia.

WebM, May 2010, is Google's cut of Matroska for the web's video tag: VP8, VP9 or AV1 video with Vorbis or Opus sound, all free of patent fees, announced the same day Google released VP8. It is the video of the modern web's clips and reaction loops, the animated GIF's replacement, and the file Discord, Reddit and the imageboards trade in. Every browser plays it now, Safari last, in 2021. [The WebM project](https://www.webmproject.org/docs/container/) documents the container.

### NSV

Nullsoft, the company behind Winamp, built an internet television and radio platform on its Shoutcast servers, and Nullsoft Streaming Video, 2003, was its format: VP3 or VP6 video with MP3 sound, in a container as plain as the company could make it. A saved stream from that world is one. Winamp played it and almost nothing else did, and the players that carry their own decoders are what open one now. [Its Wikipedia article](https://en.wikipedia.org/wiki/Nullsoft_Streaming_Video) has what little there is to say.

### Bink

RAD Game Tools' Bink Video, 1999, was the cutscene format of thousands of PC and console games for two decades, from Baldur's Gate II to the Mass Effect series, chosen because it decoded fast on the hardware of the day and licensed at a flat fee. Its logo opened the game, and its `.bik` files sit in the folders of many installed games. RAD's own player opens one, and the decoders FFmpeg carries do; nothing that came with an operating system ever did. [RAD's site](http://www.radgametools.com/bnkmain.htm) still sells it, and [Bink's history](https://en.wikipedia.org/wiki/Bink_Video) is on Wikipedia.
## Sound

The PC had no sound to speak of until 1989, when Creative's Sound Blaster gave it a way to play recorded samples and a Yamaha chip to synthesize music, and the Amiga and the Mac had both from the start. The formats below divide by what they store: recorded sound as samples, which is large; music as instructions to a synthesizer, which is tiny; and, from 1993, recorded sound compressed by discarding what the ear cannot hear, the step that made music a file.

### Sampled sound: WAV, AIFF, AU and VOC

The Waveform file, `.wav`, arrived with Windows 3.1 in 1991 as Microsoft and IBM's way to store sound as the plain samples a CD does, in a RIFF container, the chunked structure Microsoft had copied from the Amiga's IFF. It is Windows' own sound: the chimes and dings, the first step of a CD rip, the recording studio's working file, and the sound effects of a decade of games. At ten megabytes a minute for CD quality it was never for trading. Everything opens it. [Microsoft's format reference](https://learn.microsoft.com/en-us/windows/win32/multimedia/waveform-audio) and [the WAV article](https://en.wikipedia.org/wiki/WAV) describe it.

Apple's Audio Interchange File Format, `.aiff`, is three years older, 1988, and built directly on IFF: the Mac's plain sound file, samples as recorded, large and lossless. It is what Mac music software wrote and what iTunes rips to when asked for lossless, so it is common in music made or ripped on a Mac. `.aif` is its three-letter spelling, for a name that had to fit DOS or a program that preferred it. Every serious audio program opens both, and Windows has since Windows 7. [AIFF's Wikipedia article](https://en.wikipedia.org/wiki/Audio_Interchange_File_Format) has the format.

Sun's audio file, `.au`, came from the late 1980s on its Unix workstations, and NeXT used the same format under the name `.snd`; the two companies' formats were one. Usually 8-bit samples at telephone quality, it was the sound of the early web: the first browsers and Java's first applets played `.au` before anything else, so a 1990s web page's sound effect was one. Most players still open it; nothing writes it now, and other programs used the `.snd` name loosely for sounds of their own, so not every `.snd` is one. [The Au file format article](https://en.wikipedia.org/wiki/Au_file_format) covers the header.

Creative Voice, `.voc`, 1989, is the Sound Blaster's own file, from the card that gave the PC sound. DOS games and the Sound Blaster's bundled tools recorded and played it, so it dates from the start of PC multimedia, alongside the first WAVs. Windows made WAV the standard and `.voc` faded by the mid-1990s, but the players with their own decoders still open it. [The Creative Voice article](https://en.wikipedia.org/wiki/Creative_Voice_file) has its blocks.

### MIDI: MID, MIDI and RMI

MIDI is a 1983 standard for connecting musical instruments, and a Standard MIDI File, agreed in 1988 and published by the MIDI Manufacturers Association in 1991, holds what would have gone down the cable: no sound at all, but the notes, their timing, and which instrument plays each. A song is a few kilobytes and sounds like whatever synthesizer plays it, a Sound Blaster's thin FM chip, a Roland card's real samples, or the software synthesizer Windows has carried since. General MIDI, 1991, fixed the instrument numbers so a file made on one machine sounded roughly right on another. [The Standard MIDI File specification](https://midi.org/standard-midi-files-specification) is the MIDI Association's, and [MIDI's Wikipedia article](https://en.wikipedia.org/wiki/MIDI#Standard_files) covers the file's three types.

It was the music of 1990s games and of web pages that played a tune when they opened. `.mid` is the DOS spelling and the common one; `.midi` is the full name, used on the Mac, Unix and by most modern software. `.rmi` is a MIDI file wrapped in Microsoft's RIFF, the same wrapper WAV uses, from Windows 3.1 in 1992: Windows' own MIDI tools wrote it and Media Player plays it, and inside is an ordinary Standard MIDI File. Rare beside `.mid`.

### Trackers: MOD, 669, S3M, XM and IT

Karsten Obarski wrote The Ultimate Soundtracker for the Amiga in 1987 to help a friend score a game, and invented a kind of music file: a set of recorded samples, plus patterns saying which sample plays at which pitch on which of the Amiga's four channels, tick by tick, down a screen laid out like a spreadsheet. A module sounded the same on every machine, since the samples traveled with it, and fit on a floppy. It was the music of the demo scene, of the BBS file areas, and of games on the Amiga and then the PC, and the module, `.mod`, is its name. [MOD's Wikipedia article](https://en.wikipedia.org/wiki/MOD_(file_format)) has the format and [the OpenMPT wiki](https://wiki.openmpt.org/Manual:_Module_formats) documents every tracker format in detail. JVC and Panasonic camcorders later used the same three letters for MPEG-2 clips, so a folder of `.mod` files may be either.

The PC caught up in the early 1990s, once it had a Sound Blaster or a Gravis Ultrasound to play samples on. Composer 669, 1992, was one of the first PC trackers, with eight channels and few effects, and the earliest PC demo and BBS music is in its `.669` files. Scream Tracker 3, 1994, from Future Crew, the Finnish group whose demo Second Reality had defined the PC scene the year before, gave the PC what the Amiga had and more, and its `.s3m` carried the DOS demo scene. FastTracker 2, also 1994, from Triton, another Scandinavian group, added up to 32 channels and instruments with envelopes rather than bare samples, and its extended module, `.xm`, is the format more tracker music of the middle and late 1990s is in than any other; the games that used trackers, Unreal among them, used it. Impulse Tracker, 1995, from Jeffrey Lim, was the last and most capable of the DOS trackers, with 64 channels, stereo samples and effects the others lacked, and its `.it` carried the scene into the 2000s and is what the tracker world still writes by preference. A collection of scene music runs from `.mod` through `.s3m` and `.xm` to `.it`, and [the Mod Archive](https://modarchive.org/) holds most of it.

### MP3 and MP2

MPEG-1's audio standard, 1993, defined three layers of increasing complexity. Layer II, `.mp2`, became the sound of Video CD, DVD and digital radio and television broadcasting, and a bare `.mp2` is a track pulled from one of those or a broadcast recording; most players open it and the web never adopted it. Layer III, from the Fraunhofer Institute under Karlheinz Brandenburg, was the one that changed things: a song at a tenth of a CD's size with most of what the ear hears. Fraunhofer's l3enc, July 1994, was the first encoder anyone could run, and the institute picked the `.mp3` extension in a poll in 1995. [Fraunhofer's own account](https://www.iis.fraunhofer.de/en/ff/amm/consumer-electronics/mp3.html) and [MP3's Wikipedia article](https://en.wikipedia.org/wiki/MP3) tell the story.

What followed is the history of music on computers. Winamp, 1997, played MP3s well on a Windows PC and was free; Napster, 1999, let sixty million people trade them; the iPod, 2001, carried a thousand of them in a pocket; and the iTunes Store in 2003 was the record industry's answer, in a different format below. MP3 has been the common form of a music file since, and the last of its patents ran out in April 2017. Everything plays it. The 128 kilobit files of the Napster years show their age, and the format's own successors, AAC and Opus, sound better at the same size, but nothing has displaced it.

### RealAudio: RA

Progressive Networks shipped RealAudio in April 1995, and it was the first sound that streamed over a modem: radio and speeches at 14.4 kilobits a second, thin and muffled, but live, which nothing else on the internet was. It was the internet's audio for the few years before MP3, and a saved clip from a 1990s radio site is one. RealPlayer, the company's player, grew over the years into the program [PC World ranked second](https://en.wikipedia.org/wiki/RealPlayer) on its 2006 list of the worst tech products of all time, which is a story about the company rather than the format. Nothing plays an `.ra` now but a player that carries its own decoders. [RealAudio's codecs](https://en.wikipedia.org/wiki/RealAudio) are listed by version on Wikipedia.

### Windows Media Audio: WMA

Microsoft's answer to MP3, 1999, and the format Windows Media Player ripped CDs to unless told otherwise, so CDs ripped on Windows in the early 2000s are often in it whether or not the user meant to choose it. It came with copy protection built in, and the music stores of the early 2000s, MSN Music, Napster's second life, and the PlaysForSure devices, sold songs in protected WMA; those stores closed and their servers went away, and a protected file from them no longer plays anywhere. Windows still plays the rest; a Mac and the web never did. [The codec's versions](https://en.wikipedia.org/wiki/Windows_Media_Audio) are covered on Wikipedia.

### Ogg Vorbis: OGG

In 1998 Fraunhofer announced it would charge for MP3 encoders, and Christopher Montgomery's answer was Vorbis, released in 2000 by the Xiph.Org Foundation: free of patents, better than MP3 at the same size, and carried in Xiph's own Ogg container, so the file is `.ogg`. It never displaced MP3 because the iPod would not play it, but it became the sound of games, Unreal Tournament and Minecraft among them, of Wikipedia, and inside Spotify's streams. Most players open it; Apple's do not. [Xiph's Vorbis site](https://xiph.org/vorbis/) has the specification and [Vorbis's Wikipedia article](https://en.wikipedia.org/wiki/Vorbis) the history.

### AAC: M4A, AAC and M4P

Advanced Audio Coding, 1997, was MPEG's successor to MP3, designed without MP3's compromises and sounding better at the same bitrate, and it is the sound inside MP4 video, iTunes and YouTube. Apple made it a consumer format in 2003 with the iTunes Music Store, and named a file of it `.m4a`, AAC sound in an MP4 container. iTunes rips to it and the store sold it, so an iTunes library is largely made of them. A bare `.aac` is the stream without its container, written by some rippers and radio recorders; the same sound in a box is an `.m4a`. Everything modern plays both. [AAC's Wikipedia article](https://en.wikipedia.org/wiki/Advanced_Audio_Coding) covers the standard.

`.m4p` is a protected iTunes Store purchase, 2003 to 2009: an `.m4a` with Apple's FairPlay copy protection, playable only in iTunes on a computer authorized to the account that bought it. The store dropped the protection in 2009 and offered to upgrade old purchases, but a purchase never upgraded stays protected, and nothing else opens it. [FairPlay's history](https://en.wikipedia.org/wiki/FairPlay) is on Wikipedia.

### Lossless: FLAC, APE and WV

Lossless compression packs a CD's sound to about half its size with every bit intact, and three formats competed for the collectors who wanted that. WavPack, `.wv`, 1998, came first, with a hybrid mode that splits a song into a small lossy file and a correction file that restores the rest. Monkey's Audio, `.ape`, 2000, was a little smaller and slower to decode, and became the format of the lossless music trading circles of the early 2000s, where a whole album is often one `.ape` with a cue sheet beside it. Neither had a following outside those circles, and nothing from an operating system opens them. [Monkey's Audio's Wikipedia article](https://en.wikipedia.org/wiki/Monkey%27s_Audio) covers it and its rival.

The Free Lossless Audio Codec, `.flac`, 2001, from Josh Coalson and now Xiph, won by being open, fast to decode and everywhere: the exact rip, the Bandcamp download, the concert recording traded among fans. For years only the dedicated players opened it; now nearly everything does, and the browsers and Apple's software have since 2017. [Xiph's FLAC site](https://xiph.org/flac/) has the format.

### Opus

Opus, 2012, from Xiph and the Internet Engineering Task Force, merged Skype's speech codec with Xiph's music codec into one that is good at every bitrate and fast enough for a conversation, and it became the codec of speech and music online: voice notes, Discord, video calls, and the sound under YouTube's video. It usually travels inside an `.ogg` or a `.webm`; a file named `.opus` is Ogg. Every modern player opens it. [The Opus site](https://opus-codec.org/) and [RFC 6716](https://www.rfc-editor.org/rfc/rfc6716) define it.

### Surround: AC3 and DTS

Dolby Digital, as Dolby's AC-3, was in cinemas from 1991 and on DVD from 1997, and it is the surround sound of every DVD and most Blu-ray discs. A bare `.ac3` is a soundtrack pulled from a disc, kept beside the video or for a home theater's receiver. Digital Theater Systems' format, 1993, was the alternative on DVD and Blu-ray, usually at higher bitrates, and a bare `.dts` is the same thing from the other company; a few labels released music albums as DTS on CD. Both need a decoder, which the players carrying their own have and the operating systems mostly do not, playing them inside a video and rarely alone. [The Dolby Digital article](https://en.wikipedia.org/wiki/Dolby_Digital) covers the two side by side.

### AMR

Adaptive Multi-Rate is the speech codec of GSM phones, standardized in 1999, and the file a phone of the 2000s saved a voice memo or a recorded call as. It holds speech and little else, at telephone quality, in files small enough to send over the networks of the time. Phones and the players with their own decoders play it, and QuickTime does as well. [AMR's Wikipedia article](https://en.wikipedia.org/wiki/Adaptive_Multi-Rate_audio_codec) has the codec.
## Beside the media

A collection holds files that are not media but say what to do with it: lists of what to play in what order, a map of where the tracks are in a disc image, and the words of a film in a language its soundtrack is not. They are small text files, and a folder of music or video rarely lacks one.

### Playlists: M3U, M3U8, PLS and XSPF

The playlist of Winamp, 1997, `.m3u`, is a text file listing the paths of songs, one per line, in the order to play them, and every music player since has read it. Internet radio stations hand one out to start a stream, and playlists from the MP3 years are in it, pointing at files that may since have moved. `.m3u8` is the same file written in UTF-8, so a song with an accent in its name is found; music players began writing it in the 2000s, and Apple's HTTP Live Streaming borrowed it in 2009 as the index of a video stream, so a `.m3u8` today may be a playlist of songs or the recipe of a stream. [The M3U article](https://en.wikipedia.org/wiki/M3U) covers both.

The Shoutcast playlist, `.pls`, came from Winamp's internet radio of 1998: a small text file in the style of a Windows INI file, naming a station's stream or a set of songs. Clicking one on a radio site opened it in Winamp, and one saved from those years names a stream that may no longer exist. The XML Shareable Playlist Format, `.xspf`, 2006, from Xiph, set out to be the open playlist that named songs by more than a path; VLC adopted it as its own, and a playlist saved from VLC is one. Few other players read it. [The XSPF specification](https://xspf.org/) is short.

### Cue sheets: CUE

A cue sheet came from CDRWIN, a CD burning program of the 1990s, as the text file that told the burner where each track started in a single file holding a whole disc, and what the disc and its tracks were called. It became the way a CD ripped as one large file, an `.ape`, a `.flac` or a `.wav`, still knows its tracks, and the way one is burned back exactly, so it sits beside those files in lossless collections. [The cue sheet article](https://en.wikipedia.org/wiki/Cue_sheet_(computing)) has the syntax.

### Subtitles: SRT

SubRip was a program of around 2000 that read the subtitles off a DVD, by recognizing the characters in the picture, and wrote them out as plain text, each line with the time it appears and disappears. Its file, `.srt`, became the one subtitle format everything reads, and a downloaded film or series comes with one beside it, named to match, which players pick up on their own. [SubRip's Wikipedia article](https://en.wikipedia.org/wiki/SubRip) has the format's few rules.

## What is not here

Some formats were everywhere for a few years and are named on no list of what a program opens, because nothing opens them and nothing will. AOL rewrote every picture its twenty million subscribers saw into its own `.art` format, and only AOL's software read it. Liquid Audio sold music with copy protection from 1997 until it closed in 2002. Sony's OpenMG wrapped every song moved to a MiniDisc or an early Walkman in encryption that only SonicStage understood. Audible's audiobooks, since 1997, are locked to the account that bought them. Macromedia Director's Shockwave movies were the CD-ROM era's interactive multimedia and the web's games before Flash, and run now only in an emulator.

They belong in this history and not in a list of file types a program might one day open, because their keys, their players or their companies are gone. A collector who has them has them as a record of a time, which is a fine reason to keep them.

## Sources

The links above are the specifications where a specification is public, and Wikipedia where the history is best told in one place. Three more cover anything this page does not. [The Encyclopedia of Graphics File Formats](https://www.fileformat.info/format/), from 1996, is the reference for the picture formats of that era, and its summaries are online. [Just Solve the File Format Problem](http://fileformats.archiveteam.org/wiki/Main_Page), Archive Team's wiki, is the fullest catalog of everything else, including the orphans. And [the Library of Congress's format descriptions](https://www.loc.gov/preservation/digital/formats/) say, for each format, what an archive needs to know to keep it readable, which is the question this page is really about.
