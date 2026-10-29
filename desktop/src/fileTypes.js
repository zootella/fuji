/*
Every kind of file fuji knows, keyed by extension, and the one place any part of fuji learns what it can do with one. The folder listing keeps only what is named here, the sheet's flow chooses how to draw a thumbnail by it, the store types a blob from it and has the engine or the imaging library make a table's picture by it, the shell routes a dropped or opened file by it, associate.js offers the operating system the ones fileTypesAssociations below answers for it, and the settings show them in the words below.

The table is the complete list fuji grows into rather than a list that grows: a manifest of what is out there, with an entry's history and its render tags written before fuji can do anything with it. The video and sound entries sat here with their tags filled in, off, before fuji carried a library that could play one. enabled is the switch, and it is a fact of its own: set by hand, one entry at a time, and never worked out from the tags, not by this file, not by a pass over it, and not by anything that tends the data. It says whether fuji opens this kind today. On, the listing keeps the kind where its tags say, the shell routes it, and the settings show it as a kind fuji opens and offer it to the operating system wherever its kind has a road on this platform. Off, the entry is described and waiting, shown by the settings alone, under coming soon, where the user can still rest the pointer on it and read its history. The tags say what could render a kind and the switch says whether fuji does, and the two disagree in both directions: a kind on with a road on one platform and none on another, which fileTypesAssociations sorts out per platform at run time by reading the switch and the tags together, and a kind off with a road on every platform, which the three pictures FFmpeg alone decodes are, kept off because fuji takes no mpv road for a picture. Code reads the switch and the tags together wherever a decision is made, so neither is ever regenerated from the other and the two cannot drift. Where they seem to disagree, the comment on the switch says why.

Every entry carries the same properties. kind says which part of fuji a file is for: image, the sheet and the tables; video and audio, the player; playlist, a list of files to play, and subtitle and cue, text beside a media file, which the player would take and nothing takes yet; and animation, which nothing takes. reach says how far the extension reaches into collections, on the scale the essay below this one defines. render lists the components a file of the kind is expected to render in, by six tags. webkit, edge and gtk are the web engines Tauri finds on the Mac, Windows and Linux, which draw a picture in an img and play a sound or a video in their own tag; gtk is WebKitGTK, GNOME's engine and the one on every Linux desktop Tauri runs on, GNOME or not. imageio and wic are the operating system's imaging libraries thumbnail.rs calls on the Mac and Windows, for pictures alone, since fuji never asks the operating system to play. mpv is the media library fuji builds and carries, the same on all three platforms, so one tag stands for all of them. A tag says the component can do it on fuji's floor, an updated Windows 10, an updated Sequoia and a current Linux desktop; what a library or an engine can do bounds the tags, and fuji's choices narrow them, each choice said in the entry's comment. GIF and SVG carry no imageio or wic so a GIF animates and an SVG stays vector. wic stands for what WIC reads at the factory and fuji has chosen to send it, and never a format WIC reads only through an extension from the Store. Linux has no imaging-library tag, because Linux desktops and distributions vary too much in what they carry for fuji to lean on any library of theirs, so a picture on Linux is the engine's or nothing. gtk is reasoned from what every desktop carries, GStreamer's base and good packages for sound and video and WebKitGTK's own picture decoders as the distributions build them, and none of it has been seen on a Linux desktop. An engine's tag goes on every sound and video it plays on the floor, so that a web player, if one comes, is a choice the table already allows; today every video and sound goes down the mpv road, which decodes every file under every name. One tag covers a video's picture and its sound together, and a component that would play one without the other is said in the comment and not in the tag. A picture carries mpv wherever FFmpeg inside the library decodes it and some platform has no other road for it, while keeping its kind, and the roads below never count mpv as a road for a picture.

The code above the table makes every choice among the roads an entry allows, and the tags never do. Each choice is a small function below the table that reads it directly, so the reasoning is written once and in one place, and a part of fuji asks its function rather than reading tags of its own. fileTypesEngine and fileTypesLibrary say which tags are each platform's, and fileTypesRoads gives the roads a kind takes there: for a picture the platform's imaging library and its engine, and for a sound or a video, mpv. fileTypesSheet chooses for the contact sheet, the imaging library first, since a thumbnail comes back from it small and cheap. fileTypesTable chooses for the tables, the engine first, since the table's flip was built and measured on the engine's img, and the imaging library where it alone reads the kind, so a TIFF on Windows and a Photoshop file on the Mac show on the table as they do on the sheet. The folder listing keeps, for the image parts, only an enabled picture with a road on this platform, so a sound or a video never reaches the sheet or the table, and the flow refuses a listed file whose route fails and counts it. The first entry's comments say what each property is.

Some pictures have no engine and no imaging library on one platform or another, Photoshop's files off the Mac and TIFF on Linux among them, and the media library's FFmpeg can still decode them, in the pipeline fuji plays video through. The table records that with the mpv tag, read from the decoders and demuxers the library carries rather than from files decoded. Fuji takes this road for no picture. It was weighed and set aside: the Mac opens natively every such kind a collection is likely to hold, and starting a video player to get a single still back, for every such file in a folder, has never been tried. So the pictures whose only tag is mpv are off, by hand, and a picture's mpv is a fact the code above can read and never a road it takes.

A type names the extension rather than the format, since a shared type would scatter a folder's .jpe files through the only sort by extension Explorer has. A title names what the format can do and never what it cannot, in one order, lossless or vector, then transparency, then animation, leaving out what the format allows but people rarely use, like animation in PNG, SVG and AVIF. Audio is the medium, beside image and video, so a title and a property say audio; sound is what a video has and what an audio file holds, and the prose says sound. A video's title is Video alone, since every video format here can carry sound and none needs to, so sound sets none of them apart.

This file imports nothing, so it sits beneath everything that reads it. Two lists live outside it and change with it in the same commit, because each is read where this file cannot reach. CFBundleDocumentTypes in src-tauri/Info.plist declares to the Mac what fileTypesAssociations below answers for it, read out of the bundle before any of fuji's code has run, and the list in win-setup/registry.js names its answer for Windows, for the Windows uninstaller, which takes back what fuji registered for them. Neither is generated by a build step, which was declined while the table changed about once a year, and neither is worked out by hand either: each is pasted from what fileTypesAssociations answers for its platform, in the same commit as the change to the table, and checked against it. Fuji only ever turns a type on, never back off, so those two only grow.

The table runs far past what fuji plays because it is a manifest of what is out there. Fuji is a love letter to the years personal computers learned to show color and then to play video, and to the collections people made once they could: pictures pulled down from a BBS at 2400 baud, clips off a CD-ROM, songs from Napster, and on through the forums, imageboards, Reddit and Discord that keep the same habit today. Picture the homebrew computer club of that first multimedia decade meeting again, with every hard drive its members have ever owned. What sits on those drives, from then to now, is what this table names, and no more: a format a collector would have to look up is the long tail, and it stays out.

So each entry's about tells the format's story in the article voice. Its name, its date and the people who made it, and for which machine. What it was made to do and what it holds. How it found its footing with the people who used it, said with the specifics, the program that wrote it, the shop that sold it, the scene that traded it. Then it stops, with a recent detail only when it is the story's own ending. Where lossless or transparency is the origin, or the format is the counterpoint to another, the about says so, and otherwise it never repeats what the title says. It describes the format and its place in the world, never the reader, never what fuji opens, and never what plays it today, which is the same for every entry and interesting in none; a sentence that sounds knowing is cut unless it says what it knows.
*/

/*
Reach is how far an extension reaches into collections, as a number: the spelling on the files, never the format inside them, so the four spellings of JPEG each have their own. It is here so the audit of this table spends its care where the files are. The test is what a collector thinks on reading the extension. 3 is mainstream now: yes, that is what collections are made of today, which for sound is MP3, FLAC and Ogg. 2 is remembered: yes, I remember that one, or I may have some, because a popular shop sold it or a popular program wrote it in recent years, like everything the iTunes Store sold, or because collections of the era hold it, like MIDI, WAV and Windows Media. 1 is the rest: a kind that exists and belongs here by the era test, since one sort of collector has thousands, and that anyone else would have to look up. The number is about files on drives and never about what fuji can do with them, so it is set apart from enabled and from every list, and a 1 can be on while a 3 is off. It is a fact about the world as of the year it was weighed, 2026, and a kind can move. What the number decides is the audit's care: a 3 is scrutinized for the best road rather than a working one, a 2 is checked for what plays it, and a 1 is taken on the library's and the documentation's word unless a measurement is cheap.
*/

export const fileTypes = {
	'.bmp': {
		enabled: true,//fuji opens this extension: the settings offer it to the operating system, the shell routes a file of it by its render tags below, and the folder listing keeps it for the image parts where those tags say so; false keeps an entry here, described and waiting, and out of sight everywhere but the settings. Set by hand, one entry at a time, and never worked out from the tags below; the essay says why
		reach: 2,//how far the kind reaches into collections, 1 to 3 by the essay on reach above
		kind: 'image',//which part of fuji a file is for: image, the sheet and the tables; video, Fuji Video; audio, Fuji Audio; or playlist, subtitle, cue and animation, which nothing takes yet. Separate from the render tags below, since a picture some platform has no image road for carries mpv while staying an image, and the roads below read the two together
		mime: 'image/bmp',//what the store types a blob as, and it has to be right: WebKit refuses a MOV, a FLAC or a TIFF handed to it in a blob with no type, found on the Mac mini on 2026-10-16
		type: 'BMP Image',//what Windows prints in Explorer's Type column once fuji opens the extension
		title: 'Lossless image',//the first line of the extension's card in fuji's settings
		render: ['webkit', 'edge', 'gtk', 'imageio', 'wic'],//the components a file of this kind is expected to render in, by the six tags the essay defines: webkit, edge and gtk, the web engine on the Mac, Windows and Linux; imageio and wic, the operating system's imaging library on the Mac and Windows, for pictures alone; and mpv, the media library fuji builds and carries, the same everywhere. A tag says the component can do it on fuji's floor, an updated Windows 10, an updated Sequoia and a current Linux desktop, narrowed by fuji's choices, each said where it is made. WIC reads BMP at the factory, and a BMP is the largest file there is for its pixels, which is where the native road's speed shows, so wic is listed, to be watched the first time a folder of them opens there
		contactSheet: 'canvas',//how the contact sheet shows it: canvas, a thumbnail fuji sized; img, a live img that animates or stays vector; or chip, the extension alone in a box
		about: `Microsoft's bitmap, from Windows 2.0 and OS/2 in the late 1980s, in the form Windows 3.0 fixed in 1990 with the header most files carry since. It stores every pixel as the display would show it, a row at a time from the bottom up, and compresses nothing unless asked, so a 1024 by 768 wallpaper at 24 bits is 2.3 megabytes whatever is in it. Windows wrote it everywhere: the wallpaper tiles that shipped with Windows 3.1 and 95, every picture Paint saved until Windows 7 made PNG its default, and the screenshot the Print Screen key put on the clipboard. It never carried on the web, where its size ruled it out, and a BMP today most often comes from a scanner's own software or a program of that decade.`,
	},
	'.dib': {
		enabled: true,
		reach: 2,
		kind: 'image',
		mime: 'image/bmp',
		type: 'DIB Image',
		title: 'Lossless image',
		render: ['webkit', 'edge', 'gtk', 'imageio'],//a BMP inside, and ImageIO reads it by content: the system types .dib as com.microsoft.bmp, and the thumbnailer's own call decoded a BMP so named, measured on the Mac mini on 2026-10-16. WIC chooses its decoder by the bytes too, but Windows is unmeasured under this name, so wic stays off until a .dib has been seen there. The store types the blob as a BMP and the engines decode it as one
		contactSheet: 'canvas',
		about: `The device-independent bitmap, Windows 3.0's name in 1990 for a picture stored by its own pixels rather than a display adapter's, which is what every BMP has been since. The name turned up as a file extension when a few of Windows' own tools and some others saved with it, and a .dib is a BMP inside, now and then missing the fourteen-byte file header a .bmp begins with.`,
	},
	'.gif': {
		enabled: true,
		reach: 3,
		kind: 'image',
		mime: 'image/gif',
		type: 'GIF Image',
		title: 'Image with transparency and animation',
		render: ['webkit', 'edge', 'gtk'],//no imageio or wic, though ImageIO and WIC both read GIF, because the sheet always shows a GIF live so it animates, and a still would never be used; a GIF is usually small, so the web engine drawing it whole costs little
		contactSheet: 'img',
		about: `CompuServe's Graphics Interchange Format, released in June 1987 so its subscribers could trade pictures over a modem, with the 89a revision of 1989 adding transparency and the frame delays that make an animation. Mosaic showed it inline from 1993, through the IMG tag Marc Andreessen proposed that February, and it became the web's logos, its blinking "under construction" signs and, in the 2000s, its reaction loops. A frame holds 256 colors from a palette, transparency is all or nothing, and a few seconds of video runs to megabytes. The LZW compression inside it was Unisys's patent, and the royalties Unisys announced in December 1994 are why PNG was designed the next year.`,
	},
	'.jpg': {
		enabled: true,
		reach: 3,
		kind: 'image',
		mime: 'image/jpeg',
		type: 'JPG Image',
		title: 'Image',
		render: ['webkit', 'edge', 'gtk', 'imageio', 'wic'],
		contactSheet: 'canvas',
		about: `The Joint Photographic Experts Group's standard, drafted from 1986 and published in 1992 as ITU recommendation T.81, in the three-letter spelling that fit DOS's eight-and-three file names and so became the common one. It shrinks a photograph to a tenth of its size by discarding the fine color detail the eye least notices, in blocks of 8 by 8 pixels, and it was the picture of the web from Mosaic on and of every digital camera from the late 1990s to the phone in a pocket today. The loss shows as a halo around sharp edges and text, and it compounds each time a picture is edited and saved again. It holds no transparency.`,
	},
	'.jpeg': {
		enabled: true,
		reach: 3,
		kind: 'image',
		mime: 'image/jpeg',
		type: 'JPEG Image',
		title: 'Image',
		render: ['webkit', 'edge', 'gtk', 'imageio', 'wic'],
		contactSheet: 'canvas',
		about: `The four-letter spelling of the same 1992 standard, from the Mac and Unix, which never had DOS's three-letter limit, and from the web's own tools and many phones since. Inside it is a .jpg in every way: the same photograph shrunk by the same discarding, with the same halo at a sharp edge. The spelling tells where a file came from and nothing more.`,
	},
	'.jpe': {
		enabled: true,
		reach: 2,
		kind: 'image',
		mime: 'image/jpeg',
		type: 'JPE Image',
		title: 'Image',
		render: ['webkit', 'edge', 'gtk', 'imageio', 'wic'],
		contactSheet: 'canvas',
		about: `The rarest three-letter spelling of JPEG, which a few programs of the 1990s wrote and the MIME registries still list beside .jpg and .jpeg. Inside it is an ordinary JPEG. Nothing writes it now.`,
	},
	'.jfif': {
		enabled: true,
		reach: 2,
		kind: 'image',
		mime: 'image/jpeg',
		type: 'JFIF Image',
		title: 'Image',
		render: ['webkit', 'edge', 'gtk', 'imageio', 'wic'],
		contactSheet: 'canvas',
		about: `The JPEG File Interchange Format, Eric Hamilton's 1992 layout at C-Cube Microsystems for carrying a JPEG in a file, which almost every JPEG follows inside whatever it is called. As an extension it was rare until a Windows 10 update in 2017 mapped the image/jpeg type to .jfif in the registry, after which Firefox and Chrome on Windows saved pictures from Twitter and elsewhere under that name for about three years, to the bafflement of everyone who found them. Inside it is an ordinary JPEG.`,
	},
	'.png': {
		enabled: true,
		reach: 3,
		kind: 'image',
		mime: 'image/png',
		type: 'PNG Image',
		title: 'Lossless image with transparency',
		render: ['webkit', 'edge', 'gtk', 'imageio', 'wic'],//the native road keeps the transparency: the thumbnailer hands back straight-alpha pixels on both platforms, the canvas keeps the alpha, and the sheet's own ground shows through, so no background color ever has to cross to rust
		contactSheet: 'canvas',
		about: `Portable Network Graphics, designed on a mailing list in the first weeks of 1995 after Unisys began charging for GIF's compression, led by Thomas Boutell, and published as a W3C recommendation in October 1996. It holds 16 million colors with an alpha channel for transparency, compresses with the same deflate as a zip file, and loses nothing. Browsers on Windows showed it whole only once Internet Explorer 7 fixed its alpha channel in 2006, and from then it was the format of screenshots, logos, icons and every interface graphic, and the one Paint has saved by default since Windows 7. A photograph saved as PNG comes out several times larger than as JPEG.`,
	},
	'.svg': {
		enabled: true,
		reach: 3,
		kind: 'image',
		mime: 'image/svg+xml',
		type: 'SVG Image',
		title: 'Vector image with transparency',
		render: ['webkit', 'edge', 'gtk'],//no imageio or wic, because neither ImageIO nor WIC reads SVG; the sheet shows it live, so it stays sharp at any size, and inside an img a script in the file never runs
		contactSheet: 'img',
		about: `Scalable Vector Graphics, the W3C's recommendation of September 2001, drawn up after Adobe and Macromedia had each proposed a vector format for the web and neither won. A file is XML describing shapes, paths and text, so a logo is a few hundred bytes and stays sharp at any size. Internet Explorer refused it for a decade while Adobe shipped a plug-in, and only when Internet Explorer 9 drew it in 2011 did it become the web's format for icons, logos, charts and diagrams; today two websites in three serve one. It has no way to hold a photograph.`,
	},
	'.avif': {
		enabled: true,
		reach: 2,
		kind: 'image',
		mime: 'image/avif',
		type: 'AVIF Image',
		title: 'Image with transparency',
		render: ['webkit', 'edge', 'imageio', 'mpv'],//no wic: WIC reads AVIF only with an extension from the Microsoft Store, which a machine may not have, and the web engine always can. WebKit gained AVIF in 2022, so a Mac on an older system shows the placeholder. No gtk, though WebKitGTK can be built with it: Debian's packaging turns AVIF off whenever the build's vendor derives from Ubuntu, since libavif is not in Ubuntu main, read in its rules file at 2.54.1 and carried in Ubuntu 24.04's own builds through 2.52.6, so Ubuntu and Mint show nothing in an img while Debian, Raspberry Pi OS and Fedora show the picture; gtk stands for what every desktop has, and the largest does not have this. mpv: FFmpeg's demuxer reads an AVIF item and dav1d in the recipe decodes it, the same library on every platform, and on Linux nothing else does, though fuji takes no mpv road for a picture, so an AVIF is not listed there. A tiled AVIF comes out as one tile, as a phone's HEIC does, though most AVIFs are not tiled
		contactSheet: 'canvas',
		about: `The AV1 Image File Format, published by the Alliance for Open Media in 2019, the group Google, Netflix, Mozilla, Amazon and Microsoft formed to make a video codec free of royalties. A picture is one intra frame of AV1 in the HEIF container, about half a JPEG's size at the same quality, with high dynamic range, wide color and transparency. Chrome drew it from version 85 in 2020, Firefox from 93 in 2021, Safari from 16.4 in 2023 and Edge from 121 in 2024, and the services that resize the web's pictures send it to browsers that ask, though it is still under two percent of websites. Encoding one takes several times as long as a JPEG.`,
	},
	'.webp': {
		enabled: true,
		reach: 3,
		kind: 'image',
		mime: 'image/webp',
		type: 'WebP Image',//WebP keeps its own capitals, being the one format whose name is not simply its letters
		title: 'Image with transparency and animation',
		render: ['webkit', 'edge', 'gtk', 'imageio'],//no wic: WIC reads WebP only with an extension from the Microsoft Store, which a machine may not have, and the web engine always can
		contactSheet: 'canvas',
		about: `Google's picture format, announced in September 2010 and built on the VP8 video codec it had bought with On2 Technologies that year, with a lossless mode, transparency and animation added by 2012. A WebP is about a quarter smaller than a JPEG of the same quality, which was the pitch: a faster web. Chrome showed it from the start, Edge took it in 2018, Firefox in 2019 and Safari in 2020, and in those years a picture saved from a Google property would not open in most programs. A fifth of websites serve it now, and it is the second commonest picture the web hands over.`,
	},
	'.pcx': {
		enabled: false,//off though the grid shows a road: FFmpeg alone reads it, which its mpv tag says, and fuji takes no mpv road for a picture, as the essay says
		reach: 1,
		kind: 'image',
		mime: 'image/vnd.zbrush.pcx',//registered with IANA in 2015, with image/x-pcx the older alias; nothing draws a PCX by either name
		type: 'PCX Image',
		title: 'Lossless image',
		render: ['mpv'],//neither ImageIO nor WIC reads it, no browser has ever shown it, and FFmpeg inside libmpv decodes it, which nothing else on any platform does
		contactSheet: 'chip',//no road fuji takes, so the extension alone, as for .wmf
		about: `ZSoft's PC Paintbrush format, from the program of 1984 and the format of 1985, and the picture format of DOS for the decade after. Microsoft licensed Paintbrush as the paint program of Windows 3.0 in 1990, so every Windows machine wrote PCX, and scanners, the clip-art collections sold on floppy and CD, and the games of the early 1990s kept their artwork in it. A file holds up to 256 colors from a palette, run-length compressed, and that palette was its limit when true color arrived. Windows 95 replaced Paintbrush with Paint, and GIF and JPEG took the rest.`,
	},
	'.tga': {
		enabled: true,
		reach: 1,
		kind: 'image',
		mime: 'image/x-tga',
		type: 'TGA Image',
		title: 'Lossless image with transparency',
		render: ['imageio', 'mpv'],//ImageIO reads Targa: the thumbnailer's own call decoded one, measured on the Mac mini on 2026-10-16; WIC does not. FFmpeg inside libmpv decodes Targa, the same library on every platform, and off the Mac nothing else decodes it
		contactSheet: 'canvas',
		about: `Truevision's format for its TARGA boards of 1984, among the first cards to put a true-color video frame on a PC, made while the company was still AT&T's EPICenter, which is why a TGA held 24-bit color and an alpha channel when every other PC picture was a palette. It was the file of broadcast graphics and 3D rendering through the 1990s, and the texture and screenshot format of the id Software games of the decade's end, and 3D tools still write it. A file is simple, usually uncompressed, and large.`,
	},
	'.tif': {
		enabled: true,
		reach: 2,
		kind: 'image',
		mime: 'image/tiff',
		type: 'TIF Image',
		title: 'Lossless image with transparency',
		render: ['webkit', 'imageio', 'wic', 'mpv'],//ImageIO and WIC both read it at the factory. Safari alone shows a TIFF in an img, and always has; Chromium never. FFmpeg inside libmpv decodes TIFF, the same library on every platform, and on Linux, which has neither ImageIO nor an engine that shows TIFF, nothing else decodes it
		contactSheet: 'canvas',
		about: `The Tagged Image File Format, which Aldus published in 1986 so the first desktop scanners had one file to write for PageMaker, and which Adobe has owned since it bought Aldus in 1994, in the three-letter spelling DOS needed. It is the format of the scanner, the fax machine, the print shop and the archive, lossless as a rule and often tens of megabytes. Inside is a list of tags rather than one layout, so a file can hold several pages, or a compression the reading program never learned, and TIFF readers have a long habit of refusing each other's files; the 6.0 specification of 1992 is still the one.`,
	},
	'.tiff': {
		enabled: true,
		reach: 2,
		kind: 'image',
		mime: 'image/tiff',
		type: 'TIFF Image',
		title: 'Lossless image with transparency',
		render: ['webkit', 'imageio', 'wic', 'mpv'],//as for .tif
		contactSheet: 'canvas',
		about: `The four-letter spelling of the 1986 format, from the Mac and Unix, which had no three-letter limit, and the one Apple's own software writes, Preview and the scanner driver among them. Inside it is a .tif: lossless, large, and built from tags, so a file may hold several pages or any of a dozen compressions.`,
	},
	'.pict': {
		enabled: true,
		reach: 1,
		kind: 'image',
		mime: 'image/x-pict',
		type: 'PICT Image',
		title: 'Image',
		render: ['imageio'],//ImageIO on Sequoia still lists com.apple.pict among the types it reads, measured on the Mac mini on 2026-10-16 under macOS 15.7.4, though no PICT file has been decoded through it yet
		contactSheet: 'canvas',
		about: `Apple's QuickDraw picture, from the first Macintosh of 1984: a recording of QuickDraw's drawing calls, so a file is lines, shapes and text as well as bitmaps, and the clipboard format of the classic Mac OS for sixteen years, where a cut from MacDraw pasted into MacWrite as a PICT. Version 2 of 1987 brought color with the Macintosh II. Mac OS X built its drawing on PDF instead in 2001, and Apple's own programs stopped writing it over the following decade.`,
	},
	'.pct': {
		enabled: true,
		reach: 1,
		kind: 'image',
		mime: 'image/x-pict',
		type: 'PCT Image',
		title: 'Image',
		render: ['imageio'],//as for .pict; the system types .pct as com.apple.pict too
		contactSheet: 'canvas',
		about: `The three-letter spelling of a PICT, used wherever a name had to fit DOS, so it is the form a Mac picture took on a PC floppy or a clip-art CD of the 1990s. Inside it is a .pict.`,
	},
	'.iff': {
		enabled: false,//as for .pcx
		reach: 1,
		kind: 'image',
		mime: 'image/x-ilbm',
		type: 'IFF Image',
		title: 'Image',
		render: ['mpv'],//nothing native reads it and no browser does; FFmpeg inside libmpv decodes it, as for .pcx
		contactSheet: 'chip',//as for .pcx
		about: `Electronic Arts' Interchange File Format, which Jerry Morrison designed in 1985 as a container of tagged chunks for the Amiga, and whose picture form, ILBM, the interleaved bitmap, was the file Deluxe Paint saved. Every Amiga picture passed through it: the game artwork drawn in Deluxe Paint from 1985 on, the demo scene's screens, and the 4096-color HAM pictures the Amiga alone could show. A file holds a palette of up to 256 colors, or the Amiga's own modes with more.`,
	},
	'.lbm': {
		enabled: false,//as for .pcx
		reach: 1,
		kind: 'image',
		mime: 'image/x-ilbm',
		type: 'LBM Image',
		title: 'Image',
		render: ['mpv'],//as for .pcx
		contactSheet: 'chip',//as for .pcx
		about: `The DOS spelling of an ILBM picture, from the PC version of Deluxe Paint that Electronic Arts shipped in 1988, which became the paint program of the game industry: the artists at LucasArts and id drew in it, so the backgrounds and sprites of the early 1990s were .lbm files before they were anything else, and some collections keep them that way. Inside it is an Amiga .iff.`,
	},
	'.wmf': {
		enabled: false,
		reach: 1,
		kind: 'image',
		mime: 'image/wmf',
		type: 'WMF Image',
		title: 'Vector image',
		render: [],//WIC does not read it, though Windows draws it everywhere else; ImageIO never has; FFmpeg has no reader for it
		contactSheet: 'chip',
		about: `The Windows Metafile, from Windows 3.0 in 1990: a recording of GDI's drawing calls, so a picture is lines and shapes that scale, or a bitmap, or both, the PC's answer to the Mac's PICT. It was the format of clip art, the thousands of drawings on the CDs sold in the 1990s and in the gallery inside Office, which still pastes a drawing as one. In December 2005 a crafted WMF could run code on any Windows machine that displayed it, through a record meant for printers, and Microsoft's fix that January removed the record rather than repair it.`,
	},
	'.emf': {
		enabled: false,
		reach: 1,
		kind: 'image',
		mime: 'image/emf',
		type: 'EMF Image',
		title: 'Vector image',
		render: [],
		contactSheet: 'chip',
		about: `The Enhanced Metafile, from Windows NT 3.1 in 1993, the 32-bit successor to the WMF: the same recording of drawing calls with 32-bit coordinates and the device independence the old one lacked. Office has pasted and saved drawings as it since, and it is mostly a clipboard and print format, the form a vector drawing takes between two Windows programs.`,
	},
	'.psd': {
		enabled: true,
		reach: 2,
		kind: 'image',
		mime: 'image/vnd.adobe.photoshop',
		type: 'PSD Image',
		title: 'Layered image with transparency',
		render: ['imageio', 'mpv'],//ImageIO reads the flattened picture Photoshop saves inside; WIC does not. mpv as for .tga: FFmpeg reads the flattened picture too
		contactSheet: 'canvas',
		about: `Photoshop's document, from the program's first release for the Macintosh in February 1990, with layers added in version 3.0 in 1994. It holds the working picture: the layers, the masks, the text still editable and the adjustments not yet applied, so a file is many times a JPEG's size and only Photoshop shows all of it. Photoshop writes a flattened composite beside the layers, which is all any other program shows of one. Designers work in it, and the templates and mockups of thirty years are shared in it.`,
	},
	'.heic': {
		enabled: true,
		reach: 3,
		kind: 'image',
		mime: 'image/heic',
		type: 'HEIC Image',
		title: 'Image',
		render: ['webkit', 'imageio'],//no wic, where WIC needs two extensions from the Microsoft Store, one of them paid. Safari shows it since 2023; Chromium does not. No mpv, though the recipe's FFmpeg reads a phone's tiled HEIC: since 7.1 the demuxer hands out one stream per tile with the grid as a stream group, and mpv plays streams and knows nothing of groups, so through libmpv the file comes out as one tile
		contactSheet: 'canvas',
		about: `The High Efficiency Image Container, Apple's name for a HEIF file holding HEVC pictures, which every iPhone has shot in by default since iOS 11 and the iPhone 7 in 2017, and many Android phones since. A picture is one intra frame of HEVC video, about half a JPEG's size at the same quality, with room for wide color, a depth map and a burst in one file. It is the camera roll of a billion phones, and a PC opens one only with two extensions from the Microsoft Store, one of them sold for a dollar, so a phone converts to JPEG when a picture leaves by mail or the web.`,
	},
	'.heif': {
		enabled: true,
		reach: 2,
		kind: 'image',
		mime: 'image/heif',
		type: 'HEIF Image',
		title: 'Image',
		render: ['webkit', 'imageio'],//as for .heic
		contactSheet: 'canvas',
		about: `The High Efficiency Image File Format's own extension, from the MPEG standard drafted in 2015 and published in 2017 that Apple's .heic is one profile of. Apple never writes the name, and though Samsung, Canon and Sony cameras can shoot HEIF, they save it as .heic or .hif, so a .heif file comes from a converter or a tool. Inside is usually the same HEVC still a .heic holds, though the container can also carry AV1 or JPEG.`,
	},
	'.dng': {
		enabled: true,
		reach: 1,
		kind: 'image',
		mime: 'image/x-adobe-dng',
		type: 'DNG Image',
		title: 'Camera raw image',
		render: ['imageio'],//ImageIO reads raw files for the cameras Apple lists; Windows needs the Raw Image Extension from the Store
		contactSheet: 'canvas',
		about: `Adobe's Digital Negative, published in September 2004 as an open raw format built on TIFF, meant to end every camera maker keeping its own. It holds the sensor's readings before a picture has been made of them, so exposure and color are still to be decided. Leica and Pentax wrote it in their cameras, Android has offered it since version 5 in 2014, Samsung phones since the Galaxy S6 and iPhones in ProRAW since 2020, every one of them opt-in, and Adobe's free converter turns any maker's raw into it.`,
	},
	'.cr2': {
		enabled: true,
		reach: 1,
		kind: 'image',
		mime: 'image/x-canon-cr2',
		type: 'CR2 Image',
		title: 'Camera raw image',
		render: ['imageio'],
		contactSheet: 'canvas',
		about: `Canon's second raw format, from the EOS-1D Mark II of 2004 to the EOS M50 of 2018, which replaced it with CR3: the sensor's data in a TIFF-like file with a JPEG preview beside it. Every Canon digital SLR of those fourteen years wrote it, and each new model changed it a little, so the raw converters trailed every camera by months.`,
	},
	'.nef': {
		enabled: true,
		reach: 1,
		kind: 'image',
		mime: 'image/x-nikon-nef',
		type: 'NEF Image',
		title: 'Camera raw image',
		render: ['imageio'],
		contactSheet: 'canvas',
		about: `Nikon's Electronic Format, the raw file of every Nikon digital camera since the D1 of 1999, the first digital SLR a camera maker built from the ground up. Inside is a TIFF/EP structure holding the sensor's data and a preview, and the details change with each model, so the converters trailed each camera as they did Canon's.`,
	},
	'.arw': {
		enabled: true,
		reach: 1,
		kind: 'image',
		mime: 'image/x-sony-arw',
		type: 'ARW Image',
		title: 'Camera raw image',
		render: ['imageio'],
		contactSheet: 'canvas',
		about: `Sony's raw file, from the Alpha line that began with the α100 in 2006, after Konica Minolta handed its digital SLR business to Sony, and that its mirrorless cameras carried to the top of the market in the 2010s. Inside is the sensor's data with a preview beside it, in a TIFF-like structure that outsiders have reverse-engineered rather than read from a specification.`,
	},
	'.mp4': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/mp4',
		type: 'MP4 Video',
		title: 'Video',
		render: ['webkit', 'edge', 'mpv'],//no imageio or wic, since no image part takes a video: the thumbnailer makes a still of a picture alone, and a frame of a video is libmpv's to make, which nothing asks for yet; and no engine shows a video in an img. mpv carries every decoder the containers below need, confirmed against the configuration FFmpeg was built with. WebKit and Chromium both play H.264 with AAC, and HEVC only through the platform's own decoder, which Windows has only with the HEVC Video Extensions from the Store; no gtk, where WebKitGTK plays through the GStreamer plugins the machine has and no desktop installs an H.264 decoder at the factory. A video's tag covers its sound with its picture
		contactSheet: 'canvas',
		about: `MPEG-4 Part 14, the container the Moving Picture Experts Group published in 2003 on the base of Apple's QuickTime file format, which Apple had offered the group in 1998. It holds video, most often H.264 or its successor HEVC, sound, most often AAC, and subtitles and chapters beside them, with an index that lets a player start anywhere. It became the one video file nearly everything writes and plays: phones, cameras, the downloads from every video site and the web's own video tag, which every browser plays in its H.264 form. The container is simple and free, and the codecs inside carry patent licenses, so a recent phone's file may hold HEVC, which Windows 10 decodes only with an extension from the Microsoft Store.`,
	},
	'.mov': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/quicktime',
		type: 'MOV Video',
		title: 'Video',
		render: ['webkit', 'edge', 'mpv'],//WebKit plays it, and Chromium files QuickTime under its MP4 container, so WebView2 plays a modern one too
		contactSheet: 'canvas',
		about: `Apple's QuickTime movie, from QuickTime 1.0 in December 1991, which played a postage-stamp video on a Macintosh and showed what a personal computer could do, and the container MPEG chose as the base of MP4 in 1998. Every iPhone records into it, as do many cameras, most often H.264 or HEVC with AAC sound, so inside a recent one is what an MP4 holds under another name. Apple's software plays it anywhere, and a .mov from the 1990s can hold Cinepak, Apple Video or Sorenson video, which a Mac stopped playing in 2019, when Apple removed the QuickTime 7 framework beneath them.`,
	},
	'.avi': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/x-msvideo',
		type: 'AVI Video',
		title: 'Video',
		render: ['mpv'],//no webkit: WebKit plays an AVI of Motion JPEG alone, through AVFoundation, which is not the AVI anyone has; Chromium plays none
		contactSheet: 'canvas',
		about: `Microsoft's Audio Video Interleave, from Video for Windows in November 1992, Microsoft's answer to QuickTime: a RIFF file interleaving each frame of video with the sound that goes with it, so a CD-ROM drive could play it without seeking. It was the first video most PC users ever saw, 160 by 120 pixels of Cinepak, Indeo or Microsoft Video 1 stuttering off a CD-ROM encyclopedia, and around 2000 it became the box DivX and Xvid films were traded in, so an AVI from that decade is often a whole film at 700 megabytes, the size of a CD-R. It is a container, and what is inside depends on the year: Cinepak, Indeo and Microsoft Video 1 in the CD-ROM years, Microsoft's own MPEG-4 and then DivX and Xvid after 1999, and H.264 in a few from the 2010s.`,
	},
	'.mpg': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/mpeg',
		type: 'MPG Video',
		title: 'Video',
		render: ['webkit', 'mpv'],//Chromium has no program stream demuxer. WebKit on the Mac plays through AVFoundation, which opens MPEG-1 and MPEG-2 program streams and decodes their video, and the engine answers maybe for video/mpeg, both measured on the Mac mini on 2026-10-16 under macOS 15.7.4 by asking rather than by playing a file; one .mpg played in a web view would settle it. The sound in a program stream is MP2 or AC-3, both of which AVFoundation decodes, measured with the video
		contactSheet: 'canvas',
		about: `MPEG-1, the Moving Picture Experts Group's first standard, published in 1993 and in the three-letter spelling: video and sound in one stream at 352 by 240, sized for the 1.5 megabits a second a single-speed CD-ROM delivered. It is the video of the Video CD, whose .dat files are the same stream, of the first digital cameras that shot movies, and of the clips traded on CD-ROM and posted on the websites of the early 2000s as a link to a file, saved with a right-click, and the extension also carries MPEG-2, the video of DVD and digital television from 1995. It was the first video format that was truly a standard, so a file from one machine played on any other with the decoder.`,
	},
	'.mpeg': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/mpeg',
		type: 'MPEG Video',
		title: 'Video',
		render: ['webkit', 'mpv'],//as for .mpg
		contactSheet: 'canvas',
		about: `The four-letter spelling of an MPEG-1 or MPEG-2 stream, from the Mac, Unix and the web, where DOS's three-letter limit did not apply. Inside it is a .mpg: the standard video of the Video CD, the DVD and the 1990s internet.`,
	},
	'.asf': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/x-ms-asf',
		type: 'ASF Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `Microsoft's streaming container from NetShow in 1996, named the ActiveX Streaming Format at first, the Advanced Streaming Format within a year and the Advanced Systems Format in the end: the box built to stream over a modem and the one every Windows Media file is in. An .asf is usually an early Windows Media video from before the .wmv name arrived with Windows Media 7 in 2000, or a stream someone saved. Windows Media Player played it, and a Mac needed Microsoft's own Windows Media Player for Mac, abandoned in 2006, and then the Flip4Mac plug-in.`,
	},
	'.wmv': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/x-ms-wmv',
		type: 'WMV Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `Windows Media Video, Microsoft's codec of 2000 in the ASF container, for the years when Windows Media Player came with the operating system and a video on the web was a choice among it, RealPlayer and QuickTime. The clip sites of the early 2000s posted a video as a link to a .wmv and a right-click saved it, so a drive from those years holds them by the hundred. Windows Movie Maker, which came with Windows ME and XP, saved every home movie as one, so the camcorder tapes of the 2000s were digitized into it by the million, and the music and film stores of the time wrapped some in Windows Media DRM, which no longer unlocks.`,
	},
	'.rm': {
		enabled: true,
		reach: 2,
		kind: 'video',
		mime: 'application/vnd.rn-realmedia',//the RealMedia container's type, which Apple, Windows and freedesktop all use; video/vnd.rn-realvideo is RealNetworks' type for a bare RealVideo stream, an .rv
		type: 'RM Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `RealMedia, from RealNetworks in 1997, when Progressive Networks added RealVideo to the RealAudio it had streamed since 1995: a container for both, built for a 28.8 modem, so a clip was a small blurry window that paused to buffer. It was how the web watched anything before broadband, news clips, trailers and the first webcasts, through RealPlayer and its nagging upgrades.`,
	},
	'.rmvb': {
		enabled: true,
		reach: 2,
		kind: 'video',
		mime: 'application/vnd.rn-realmedia-vbr',//RealNetworks' own type for the variable bitrate file, as this Mac declares it; freedesktop folds it into the .rm type as an alias
		type: 'RMVB Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `RealMedia Variable Bitrate, the 2000s form of RealVideo with RealVideo 9 and 10 inside, which spent its bits where a scene needed them and so fit a film on one CD at better quality than DivX managed. It became the standard for the video traded across China and among anime fansub groups for most of that decade, and the RealPlayer on every Chinese PC played it.`,
	},
	'.flv': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/x-flv',
		type: 'FLV Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `Flash Video, from Macromedia's Flash MX in 2002, which put Sorenson Spark video inside a Flash movie, and a file of its own from Flash Player 7 in 2003: the file behind every video a Flash player drew on the web. YouTube served its videos as .flv from its first day in 2005 until it moved to MP4 around 2010, in Sorenson Spark at first and H.264 from 2008, so a video saved from YouTube or any other tube site in those years, through the savers that pulled the .flv out of the player, is one, usually at 320 by 240. Adobe ended Flash on the last day of 2020.`,
	},
	'.m4v': {
		enabled: true,
		reach: 2,
		kind: 'video',
		mime: 'video/x-m4v',
		type: 'M4V Video',
		title: 'Video',
		render: ['webkit', 'edge', 'mpv'],//an MP4 to both engines
		contactSheet: 'canvas',
		about: `Apple's name for an MP4 of video, from October 2005, when the iTunes Store began selling music videos and television episodes for the video iPod. Inside it is an ordinary MP4, H.264 with AAC sound, unless it was bought from the store, in which case FairPlay locks it to Apple's software on an authorized machine. HandBrake named its rips .m4v whenever they carried chapters or Dolby sound, which most did, so a .m4v is as likely a DVD rip as a purchase.`,
	},
	'.3gp': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/3gpp',
		type: '3GP Video',
		title: 'Video',
		render: ['webkit', 'mpv'],//WebKit plays it; Chromium parses the container but decodes neither H.263, MPEG-4 Part 2 nor AMR, which is what a flip phone's clip holds
		contactSheet: 'canvas',
		about: `The video of the phones before smartphones, defined by the 3GPP standards group for the networks of 2003: an MP4 cut down for a flip phone's camera and its data plan, with H.263 or MPEG-4 video and AMR speech-quality sound. Most clips shot on a phone between about 2003 and 2010 are one, at 176 by 144 pixels and a few seconds long, sent by MMS or pulled off by cable.`,
	},
	'.mkv': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/matroska',//registered by RFC 9559 in 2024, with video/x-matroska now a deprecated alias; Chromium maps the extension to this name
		type: 'MKV Video',
		title: 'Video',
		render: ['mpv'],//no edge: Chromium plays Matroska when the codecs inside are ones it has, which an extension cannot promise, so the tag stays off and mpv, which decodes every one, takes them all; WebKit never opens the container
		contactSheet: 'canvas',
		about: `Matroska, begun in December 2002 as a fork of the Multimedia Container Format, named for the nesting doll, and open from the start: a container for any video and sound at all, with several audio tracks, soft subtitles and chapters in one file. The DVD-rip groups moved to it from OGM by 2004, the anime fansub groups followed later in the decade, when a release could carry the Japanese audio and a typeset subtitle track together, and it has been the file of the high-quality rip ever since; WebM was cut from it in 2010. No video site sends one and Apple's players do not open it, but Windows has played it at the factory since the first Windows 10 preview in 2014, and MakeMKV, the common tool for copying a disc with every track intact, writes nothing else.`,
	},
	'.webm': {
		enabled: true,
		reach: 3,
		kind: 'video',
		mime: 'video/webm',
		type: 'WebM Video',//WebM keeps its own capitals, like WebP
		title: 'Video',
		render: ['webkit', 'edge', 'gtk', 'mpv'],//every engine: Safari since 14.1 in 2021 with VP8, VP9 and Vorbis, Opus in WebM since 15, and on Linux GStreamer's base and good plugins, which every desktop installs, carry VP8, VP9, Vorbis and Opus
		contactSheet: 'canvas',
		about: `Google's cut of Matroska for the web's video tag, announced at Google I/O in May 2010 with the VP8 codec it had bought from On2 and opened: VP8, VP9 or AV1 video with Vorbis or Opus sound, with no patent fees on any of it. It is the video of the modern web's short clips, the animated GIF's replacement, and the file the imageboards, Reddit and Discord trade in, which is also why a clip saved from them often has no sound. Every browser plays it, Safari last, in 2021 with macOS 11.3.`,
	},
	'.ogv': {
		enabled: true,
		reach: 2,
		kind: 'video',
		mime: 'video/ogg',
		type: 'OGV Video',
		title: 'Video',
		render: ['gtk', 'mpv'],//Chromium dropped Theora in 2024 and WebKit never had it; GStreamer's base plugins carry Theora, so WebKitGTK plays it
		contactSheet: 'canvas',
		about: `Ogg video, from the Xiph.Org Foundation, most often Theora, the codec Xiph built from On2's donated VP3 in 2002 and that Mozilla and Wikipedia backed in 2009 as the free codec the web's video tag should require. It lost that argument to H.264 and then to WebM, and Chrome dropped Theora in 2024. Rare, and mostly a Wikipedia download or a free-software demo.`,
	},
	'.vob': {
		enabled: true,
		reach: 2,
		kind: 'video',
		mime: 'video/mpeg',
		type: 'VOB Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `A DVD's Video Object, from the DVD-Video standard of 1996: the disc's MPEG-2 video with its Dolby Digital sound, subtitle pictures and menus, in an MPEG program stream cut into files of a gigabyte because the disc's file system allowed no larger. They are what a DVD looks like copied straight onto a drive, in a VIDEO_TS folder beside the IFO files that hold the menus, and many were copied that way in the 2000s before ripping to a smaller file became the habit. The menus live in the IFO files, so a lone .vob plays without them.`,
	},
	'.m2ts': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/mp2t',
		type: 'M2TS Video',
		title: 'Video',
		render: ['mpv'],//Chromium has no transport stream demuxer for a file. WebKit on the Mac would try one, AVFoundation opening MPEG-2 transport streams, AVCHD among them, and the engine answering maybe for video/mp2t, measured on the Mac mini on 2026-10-16 by asking rather than by playing a file, and webkit is left for a played file to add
		contactSheet: 'canvas',
		about: `The MPEG-2 transport stream in the form Blu-ray Disc uses and the AVCHD camcorders Sony and Panasonic introduced in 2006 record into, each packet stamped with a four-byte timecode on top of the broadcast format. It holds H.264 video at full high definition, so the camcorder footage of the late 2000s and the copies of Blu-ray discs are in it.`,
	},
	'.divx': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/x-msvideo',
		type: 'DivX Video',//DivX keeps its own capitals, being a brand
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `DivX's own name for an AVI holding its MPEG-4 video, from the codec that began in 1999 as DivX ;-) 3.11 Alpha, Microsoft's MPEG-4 encoder cracked to write AVI files, and became a company in 2000. Most DivX films kept the .avi name, and the ones the company's own tools wrote from DivX 6 in 2005 carry this one; the DVD players of the mid-2000s with a DivX logo on the front played both from a burned disc. Inside it is an ordinary AVI.`,
	},
	'.qt': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/quicktime',
		type: 'QT Video',
		title: 'Video',
		render: ['webkit', 'edge', 'mpv'],//as for .mov
		contactSheet: 'canvas',
		about: `The earliest spelling of a QuickTime movie, from the Mac of 1991, before .mov became the habit. Inside it is a .mov, and the CD-ROMs and Mac collections of the early 1990s carry them, often holding Cinepak or Apple's own first codecs.`,
	},
	'.mpe': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/mpeg',
		type: 'MPE Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `The third spelling of an MPEG-1 or MPEG-2 stream, rarer than .mpg and .mpeg and mostly from the Unix tools and web servers of the 1990s. Inside it is the same standard video.`,
	},
	'.ts': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/mp2t',
		type: 'TS Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `The MPEG-2 transport stream under its plain name: the format of digital television since the standard of 1995, built in 188-byte packets so a broadcast survives a dropped one, so a file is a program recorded off the air or a satellite box, a piece of a live internet stream saved to disk, or the raw broadcast the anime scene shares before encoding. Inside it is MPEG-2 or H.264 video.`,
	},
	'.mts': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/mp2t',
		type: 'MTS Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `AVCHD's own name for its transport stream, the file a Sony or Panasonic camcorder wrote to its memory card from 2006 on, in a STREAM folder several levels down. Copied through the maker's software it became .m2ts; copied by hand it stayed .mts, so footage dragged straight off a card carries this one. Inside it is the H.264 of an .m2ts.`,
	},
	'.dv': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/x-dv',
		type: 'DV Video',
		title: 'Video',
		render: ['gtk', 'mpv'],//GStreamer's good plugins carry the DV demuxer and decoder, so WebKitGTK plays a raw stream by the same reasoning as the other gtk tags. WebKit on the Mac would try one, AVFoundation opening DV movies and the engine answering maybe for video/x-dv, measured on the Mac mini on 2026-10-16 by asking rather than by playing a file, and webkit is left for a played file to add; Chromium has no demuxer for it
		contactSheet: 'canvas',
		about: `The raw stream of a MiniDV tape, the digital camcorder format Sony, Panasonic and the others agreed in 1995 and sold until about 2005, captured over FireWire at 25 megabits a second, which is 13 gigabytes an hour, usually into an .avi or a .mov but sometimes left bare. It is the home video of a decade, lightly compressed at the resolution of standard television, and a captured tape is one of these or its wrapped form.`,
	},
	'.3g2': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/3gpp2',
		type: '3G2 Video',
		title: 'Video',
		render: ['webkit', 'mpv'],//as for .3gp
		contactSheet: 'canvas',
		about: `The 3GPP2 twin of .3gp, for the phones on CDMA networks, Verizon and Sprint in the United States, from the same years around 2003 to 2010, with the same video and QCELP or EVRC speech codecs in place of AMR. Inside it is the same small MPEG-4 relative.`,
	},
	'.f4v': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/mp4',
		type: 'F4V Video',
		title: 'Video',
		render: ['webkit', 'edge', 'mpv'],//an MP4 inside, typed video/mp4 by this entry, so both engines take it as one
		contactSheet: 'canvas',
		about: `Flash Video's second form, from Flash Player 9 Update 3 in December 2007, when Adobe added H.264 and moved Flash's video from its own container to MP4. A .f4v is an MP4 under a Flash name, so a video saved from a Flash player of the late 2000s may be one, and renaming one .mp4 is all it ever needed.`,
	},
	'.ogm': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/ogg',
		type: 'OGM Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `Ogg Media, Tobias Waldvogel's 2002 stretch of the Ogg container to hold video, several audio tracks and subtitles at once, which nothing else on a PC did at the time. It was the DVD-rip groups' format for a couple of years, a DivX or XviD picture with Vorbis sound and a subtitle track, until Matroska, built for that job, replaced it by 2004. Rare now.`,
	},
	'.nsv': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/x-nsv',
		type: 'NSV Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `Nullsoft Streaming Video, from the makers of Winamp in 2002, built for the internet television stations of Shoutcast, the same system that streamed its radio. A saved stream from that world is one, in VP3 or VP6 video with MP3 sound, and the stations themselves were a few hundred hobbyists. Winamp played it and almost nothing else did.`,
	},
	'.bik': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/vnd.radgamettools.bink',
		type: 'BIK Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `Bink Video, from RAD Game Tools in 1999, the cutscene format of thousands of PC and console games for two decades, which a studio licensed because it decoded fast on a console with no spare processor and let the game ship on one disc. Its logo opened the game, and its files sit in the folders of many installed games, Mass Effect among them. RAD's own player opened one outside a game, and FFmpeg reverse-engineered the codec in 2010.`,
	},
	'.wtv': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/x-ms-wtv',
		type: 'WTV Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `Windows Recorded TV Show, from the Windows Media Center TV Pack of 2008, what Media Center recorded television into once a PC had a tuner card, replacing .dvr-ms. A file is a program off the air at several gigabytes an hour, often under the copy protection the broadcaster's flag asked for. Windows 10 dropped Media Center in 2015, and a protected recording has had no player since.`,
	},
	'.dvr-ms': {
		enabled: true,
		reach: 1,
		kind: 'video',
		mime: 'video/x-ms-dvr',
		type: 'DVR-MS Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `Microsoft Digital Video Recording, from Windows XP Media Center Edition in 2002, the first Media Center recording format, from the years when a Media Center PC with a remote was the living room's video recorder. Inside it is MPEG-2 in an ASF wrapper, often with copy protection, and Windows played it as long as Media Center lasted, until 2015.`,
	},
	'.dat': {
		enabled: true,
		reach: 2,
		kind: 'video',
		mime: 'video/mpeg',
		type: 'DAT Video',
		title: 'Video',
		render: ['mpv'],
		contactSheet: 'canvas',
		about: `The video of a Video CD as the disc stores it, in an MPEGAV folder: an MPEG-1 stream in the disc's own 2324-byte sectors, which nearly any player reads as an .mpg. Video CD, the standard Philips and Sony published in 1993, carried films and karaoke across Asia through the 1990s and 2000s, pressed and pirated by the hundred million, and a disc copied to a drive brings its .dat files along. Countless programs call their own files .dat, so a .dat is video only when it came from a disc.`,
	},
	'.swf': {
		enabled: false,
		reach: 1,
		kind: 'animation',
		mime: 'application/vnd.adobe.flash.movie',//Adobe's registered type from 2013, with application/x-shockwave-flash the earlier name; nothing renders a SWF by either
		type: 'SWF Animation',
		title: 'Interactive animation',
		render: [],//nothing draws it: a Flash file is a program, not a picture or a video
		contactSheet: 'chip',//here so a folder of them names itself
		about: `Macromedia's Shockwave Flash, from FutureSplash Animator, the tool Macromedia bought in December 1996 and renamed Flash: the animation and game format of the web from the late 1990s until the 2010s, Newgrounds, Homestar Runner, the cartoons and games passed around by email, and the intro page of every corporate site of 2000. A file is a program with drawings and sound in it, not a video, so only a Flash player ran it, and Adobe ended Flash Player on the last day of 2020. Ruffle, an emulator written in Rust, runs most of them in a browser now.`,
	},
	'.wav': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/wav',
		type: 'WAV Audio',
		title: 'Lossless audio',
		render: ['webkit', 'edge', 'gtk', 'mpv'],//a sound has no picture to make, so no imageio or wic, and an audio file is always a chip. mpv plays every sound here that the library decodes. Every engine plays a WAV of plain PCM, Chromium none of the ADPCM kinds; Linux through GStreamer's base plugins
		contactSheet: 'chip',
		about: `Microsoft and IBM's Waveform Audio File Format, from the Multimedia PC specification of 1991 and Windows 3.1, holding sound as the plain samples a CD does, in the same RIFF container as AVI. It is Windows' own sound: the chimes and dings of the Windows Media folder, the first step of every CD rip, the working file of every recording studio, and the sound effects of a decade of DOS and Windows games. A minute of CD-quality sound is ten megabytes, so it was never traded over a modem.`,
	},
	'.aiff': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/aiff',
		type: 'AIFF Audio',
		title: 'Lossless audio',
		render: ['webkit', 'mpv'],//WebKit plays it and Chromium does not. No gtk, though WebKitGTK could read one: it plays sound through GStreamer, and GStreamer's AIFF parser, aiffparse in the plugin named aiff, ships in the bad set, not in base or good, which are the two every desktop installs and the only two the gtk tag counts on. Nothing is lost by it, since every sound takes the mpv road and FFmpeg's own AIFF demuxer is in the library on all three platforms, so an .aiff on Linux plays through libmpv like anywhere else. A review that finds the parser and adds gtk here has missed which set it is in
		contactSheet: 'chip',
		about: `Apple's Audio Interchange File Format, published in 1988 and built on Electronic Arts' IFF container from the Amiga, the Mac's plain sound file three years before WAV: samples as recorded, with no compression. It is the file the Mac's own alert sounds ship as, what Mac music software wrote for twenty years, from Sound Designer to Logic, and what iTunes rips to when told to rip uncompressed, so music made or ripped on a Mac is full of it.`,
	},
	'.aif': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/aiff',
		type: 'AIF Audio',
		title: 'Lossless audio',
		render: ['webkit', 'mpv'],//as for .aiff
		contactSheet: 'chip',
		about: `The three-letter spelling of AIFF, from the years a name had to fit DOS, and the spelling iTunes and the Music app still write when they rip to AIFF. Inside it is an .aiff.`,
	},
	'.mid': {
		enabled: false,
		reach: 2,
		kind: 'audio',
		mime: 'audio/midi',
		type: 'MID Audio',
		title: 'Music as notes',
		render: [],//FFmpeg has no MIDI reader and the library no synthesizer, so nothing in it plays one; the operating systems have a synthesizer, and asking them is a route of its own. No browser plays it
		contactSheet: 'chip',
		about: `A Standard MIDI File, written by Dave Oppenheim at Opcode and adopted by the MIDI Manufacturers Association in 1988, with General MIDI's 128 instruments following in 1991, for the notes a synthesizer plays: no sound, only which key, when, how hard and on which instrument, so a song is a few kilobytes and sounds like whatever plays it, a Sound Blaster's FM chip in 1993 or a Roland Sound Canvas's samples. It was the music of DOS and Windows games through the mid-1990s, of Windows itself, which shipped canyon.mid with Windows 3.1, and of web pages that played a tune when they opened. Windows still plays one through the Microsoft GS Wavetable synthesizer it has carried since Windows 98.`,
	},
	'.midi': {
		enabled: false,
		reach: 2,
		kind: 'audio',
		mime: 'audio/midi',
		type: 'MIDI Audio',
		title: 'Music as notes',
		render: [],//as for .mid
		contactSheet: 'chip',
		about: `The four-letter spelling of a Standard MIDI File, from the Mac, Unix and the software written after the three-letter limit was gone. Inside it is a .mid, notes and their timing rather than sound.`,
	},
	'.mod': {//also the name JVC and Panasonic camcorders gave their MPEG-2 clips in the mid-2000s, a second meaning one key cannot hold; this entry is the tracker module
		enabled: false,
		reach: 2,
		kind: 'audio',
		mime: 'audio/x-mod',
		type: 'MOD Audio',
		title: 'Music as samples and patterns',
		render: [],//FFmpeg reads a module only through libopenmpt, which the recipe does not build, so nothing in the library plays one. It could: a module carries its own samples inside the file and needs no instrument bank, unlike a MIDI file, so libopenmpt is one small package in the recipe, and with it this entry, .s3m, .xm, .it and .669 would play through libmpv like everything else, with nothing from outside the file
		contactSheet: 'chip',
		about: `The module, from Karsten Obarski's Ultimate Soundtracker on the Amiga in 1987 and the ProTracker form of 1990 that most files follow: a song as a set of recorded samples plus the patterns that play them on the Amiga's four channels, so it sounds the same on every machine and fits on a floppy. It was the music of the demo scene, of the BBS file areas, and of games on the Amiga and then the PC, and Modland and the Mod Archive hold them by the tens of thousands.`,
	},
	'.s3m': {
		enabled: false,
		reach: 1,
		kind: 'audio',
		mime: 'audio/x-s3m',
		type: 'S3M Audio',
		title: 'Music as samples and patterns',
		render: [],
		contactSheet: 'chip',
		about: `Scream Tracker 3's module, from Sami Tammilehto of Future Crew in 1994, the Finnish group whose demo Second Reality defined the PC scene: samples and patterns as on the Amiga, now in up to 16 channels, with the Gravis Ultrasound card to play them. The music of the DOS demo scene and the BBS file areas of 1994 and 1995 is in it, and Modland holds about 11,000 of them.`,
	},
	'.xm': {
		enabled: false,
		reach: 1,
		kind: 'audio',
		mime: 'audio/x-xm',
		type: 'XM Audio',
		title: 'Music as samples and patterns',
		render: [],
		contactSheet: 'chip',
		about: `FastTracker 2's extended module, from the Swedish demo group Triton in 1994: up to 32 channels, instruments with volume and panning envelopes rather than bare samples, and the tracker most of the scene moved to. More tracker music of the middle and late 1990s is in it than in any other format, Modland holds 44,000 of them, second only to ProTracker's, and Epic's Unreal and Deus Ex carried their music in it inside UMX packages.`,
	},
	'.it': {
		enabled: false,
		reach: 1,
		kind: 'audio',
		mime: 'audio/x-it',
		type: 'IT Audio',
		title: 'Music as samples and patterns',
		render: [],
		contactSheet: 'chip',
		about: `Impulse Tracker's module, from Jeffrey Lim in 1995, the last and most capable of the DOS trackers: 64 channels, stereo samples, filters and the note-off and volume-column effects the others lacked. It carried the scene into the 2000s and is the format the tracker world still writes by preference, so a collection of scene music runs from .mod through .s3m and .xm to this.`,
	},
	'.mp3': {
		enabled: true,
		reach: 3,
		kind: 'audio',
		mime: 'audio/mpeg',
		type: 'MP3 Audio',
		title: 'Audio',
		render: ['webkit', 'edge', 'gtk', 'mpv'],//every engine; on Linux through mpg123 in GStreamer's good plugins
		contactSheet: 'chip',
		about: `MPEG-1 Audio Layer III, standardized in 1993 from the work of Karlheinz Brandenburg's group at Fraunhofer, and named when the institute voted for the .mp3 extension on July 14, 1995: a song at a tenth of a CD's size with most of what the ear hears, which is what made music a file. Fraunhofer's l3enc encoder, Winamp in 1997, Napster in 1999 and the iPod in 2001 followed from it, and it has been the common form of a music file since. The last of its patents ran out in 2017. The 128 kilobit files of the Napster years show their age in the swirl the encoders of the time left in a cymbal.`,
	},
	'.ra': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/vnd.rn-realaudio',
		type: 'RA Audio',
		title: 'Audio',
		render: ['mpv'],//FFmpeg decodes the common RealAudio codecs
		contactSheet: 'chip',
		about: `RealAudio, from Progressive Networks in April 1995, the first sound that streamed over a modem: ABC News and National Public Radio at 14.4 kilobits a second, and that September a Mariners game from Seattle, the first live baseball anyone heard over the internet. It was the internet's sound for the few years before MP3, through RealPlayer, and a clip saved from a 1990s radio site is one.`,
	},
	'.wma': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/x-ms-wma',
		type: 'WMA Audio',
		title: 'Audio',
		render: ['mpv'],
		contactSheet: 'chip',
		about: `Windows Media Audio, Microsoft's codec of 1999 and its answer to MP3, and the format Windows Media Player ripped a CD to at 128 kilobits unless told otherwise, through Windows XP and 7, so the CDs ripped on a Windows PC in the 2000s are in it whether or not anyone chose it. Some files carry the copy protection of the music stores of the time, Napster 2.0 and Microsoft's own among them, and those no longer play anywhere.`,
	},
	'.ogg': {
		enabled: true,
		reach: 3,
		kind: 'audio',
		mime: 'audio/ogg',
		type: 'OGG Audio',
		title: 'Audio',
		render: ['webkit', 'edge', 'gtk', 'mpv'],//Chromium plays Vorbis in Ogg; WebKit plays Ogg Vorbis from Safari 18.4 on macOS 15.4, which fuji's floor clears; Linux through GStreamer's base plugins
		contactSheet: 'chip',
		about: `Ogg Vorbis, from Christopher Montgomery and the Xiph.Org Foundation, begun in 1998 when Fraunhofer announced it would charge for MP3 encoders and released as 1.0 in July 2002: the free MP3, with no patent on it anywhere and a better sound than MP3 at the same bitrate. It never displaced MP3, because the iPod would not play it, but it became the sound of games, Unreal Tournament 2004 and Minecraft among them, of every sound clip on Wikipedia, and of Spotify's streams for the service's first decade.`,
	},
	'.m4a': {
		enabled: true,
		reach: 3,
		kind: 'audio',
		mime: 'audio/mp4',
		type: 'M4A Audio',
		title: 'Audio',
		render: ['webkit', 'edge', 'mpv'],//both engines; no gtk, where AAC needs a plugin no desktop installs at the factory
		contactSheet: 'chip',
		about: `AAC sound in an MP4 container, under the name Apple gave it with the iTunes Music Store in April 2003 and the rip format iTunes has used since: AAC, the MPEG successor to MP3 from 1997, which sounds better at the same bitrate. The store sold its songs as .m4a once the protection came off in 2009, Voice Memos on every iPhone records to it, and Apple Lossless rides in the same container under the same name, so an iTunes library and a phone are largely made of them.`,
	},
	'.aac': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/aac',
		type: 'AAC Audio',
		title: 'Audio',
		render: ['webkit', 'edge', 'mpv'],//both engines play the bare ADTS stream; no gtk, as for .m4a
		contactSheet: 'chip',
		about: `Advanced Audio Coding, the MPEG-2 standard of 1997 that succeeded MP3, and the sound inside nearly every MP4 video, every iTunes song and every YouTube stream. A bare .aac is the stream without its container, in ADTS frames, which some rippers and radio recorders write and Android can; the same sound in a box is an .m4a.`,
	},
	'.flac': {
		enabled: true,
		reach: 3,
		kind: 'audio',
		mime: 'audio/flac',
		type: 'FLAC Audio',
		title: 'Lossless audio',
		render: ['webkit', 'edge', 'gtk', 'mpv'],//every engine
		contactSheet: 'chip',
		about: `The Free Lossless Audio Codec, from Josh Coalson in 2001 and under Xiph.Org since 2003, which packs a CD's sound to a little over half its size with every bit intact and checks each frame against a hash. It is the format of the exact CD rip made with Exact Audio Copy, of the Bandcamp download, of the concert recording traded among fans since the days of Shorten, and of the lossless collection as such, with more than a million items on the Internet Archive. Apple held out against it until 2017, the last to.`,
	},
	'.opus': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/ogg',//the file's type, since an .opus is an Ogg file and RFC 7845 registers audio/ogg for it; audio/opus is the type for Opus packets over RTP, and WebKit refuses a blob typed with it while accepting this one, measured on the Mac mini on 2026-10-16
		type: 'Opus Audio',//Opus keeps its capital, being a name and not letters
		title: 'Audio',
		render: ['webkit', 'edge', 'gtk', 'mpv'],//Chromium plays it; WebKit plays Ogg Opus from Safari 18.4 on macOS 15.4; Linux through GStreamer's base plugins
		contactSheet: 'chip',
		about: `Opus, standardized by the IETF in September 2012 from Xiph.Org's CELT and Skype's SILK, the codec of speech and music online now: every WhatsApp voice note is one, every Discord call, and the sound under YouTube's video in a WebM. It is small and good at every bitrate from a whisper to a concert, and it usually travels inside an .ogg or a .webm; a file named .opus is Ogg, and a collector who pulls sound off YouTube gets them by the hundred.`,
	},
	'.mp2': {
		enabled: true,
		reach: 1,
		kind: 'audio',
		mime: 'audio/mpeg',
		type: 'MP2 Audio',
		title: 'Audio',
		render: ['gtk', 'mpv'],//no edge: Chromium's FFmpeg carries no Layer II decoder. mpg123 in GStreamer's good plugins decodes Layer II, so WebKitGTK does by the same reasoning as the other gtk tags. WebKit on the Mac would try one, AVFoundation declaring the Layer II type and the engine answering maybe for audio/mpeg, measured on the Mac mini on 2026-10-16 by asking rather than by playing a file, and webkit is left for a played file to add
		contactSheet: 'chip',
		about: `MPEG-1 Audio Layer II, from the same 1993 standard as MP3 and simpler, the sound of the Video CD, of DAB radio and of digital television. A bare .mp2 is a track pulled from one of those or a broadcast recording, and the Internet Underground Music Archive of 1993 traded songs in it for two years before MP3 existed.`,
	},
	'.ac3': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/ac3',
		type: 'AC3 Audio',
		title: 'Audio',
		render: ['mpv'],//no engine: WebKit refused one on the Mac mini on 2026-10-16, and Chromium has no demuxer for it
		contactSheet: 'chip',
		about: `Dolby Digital, Dolby's AC-3, in cinemas from Batman Returns in 1992 and on DVD from the format's launch, where it is the soundtrack every disc must carry: five channels and a subwoofer in 384 or 448 kilobits. A bare .ac3 is a soundtrack pulled from a disc in the 2000s, kept beside the .vob or the rip for a home theater's receiver to decode.`,
	},
	'.dts': {
		enabled: true,
		reach: 1,
		kind: 'audio',
		mime: 'audio/vnd.dts',
		type: 'DTS Audio',
		title: 'Audio',
		render: ['mpv'],//as for .ac3
		contactSheet: 'chip',
		about: `Digital Theater Systems' surround sound, which opened with Jurassic Park in 1993 and became the alternative to Dolby on DVD and Blu-ray, at 768 kilobits or 1.5 megabits, up to three times Dolby's. A bare .dts is a disc's soundtrack extracted, and a few hundred albums were released as DTS Music Discs, ordinary CDs that a receiver decoded into surround.`,
	},
	'.ape': {
		enabled: true,
		reach: 1,
		kind: 'audio',
		mime: 'audio/x-ape',
		type: 'APE Audio',
		title: 'Lossless audio',
		render: ['mpv'],
		contactSheet: 'chip',
		about: `Monkey's Audio, Matthew Ashland's lossless codec of 2000, a few percent smaller than FLAC and several times slower to decode. It was the format of the lossless trading circles of the 2000s before FLAC won, in Russia and China above all, where a whole album was one .ape with a .cue beside it saying where the tracks begin.`,
	},
	'.wv': {
		enabled: true,
		reach: 1,
		kind: 'audio',
		mime: 'audio/x-wavpack',
		type: 'WV Audio',
		title: 'Lossless audio',
		render: ['gtk', 'mpv'],//GStreamer's good plugins carry a WavPack decoder, so WebKitGTK plays one by the same reasoning as the other gtk tags; neither WebKit nor Chromium does
		contactSheet: 'chip',
		about: `WavPack, David Bryant's open lossless format of 1998, three years before FLAC, with a hybrid mode that splits a song into a small lossy .wv and a .wvc correction file that restores every bit. It never had FLAC's following, but a share of the lossless rips traded in the 2000s are in it, and it is the one lossless format that also packs DSD.`,
	},
	'.m4p': {
		enabled: false,
		reach: 2,
		kind: 'audio',
		mime: 'audio/mp4',
		type: 'M4P Audio',
		title: 'Audio',
		render: [],//nothing outside Apple's software plays a FairPlay file, so no route; here so a folder of iTunes purchases names itself
		contactSheet: 'chip',
		about: `A protected iTunes Store purchase, from the store's opening in April 2003 until January 2009: an .m4a with Apple's FairPlay copy protection, AAC at 128 kilobits, playable only in iTunes on one of the three computers, five from 2004, authorized to the account that bought it. iTunes Plus offered the same songs unprotected from 2007, the whole store went unprotected in 2009 and Apple offered an upgrade at 30 cents a song, but a purchase never upgraded stays locked.`,
	},
	'.amr': {
		enabled: true,
		reach: 1,
		kind: 'audio',
		mime: 'audio/amr',
		type: 'AMR Audio',
		title: 'Audio',
		render: ['mpv'],//no engine: neither was seen to play one, and Chromium decodes no AMR
		contactSheet: 'chip',
		about: `Adaptive Multi-Rate, the speech codec ETSI chose for GSM phones in 1998 and 3GPP made mandatory for 3G the next year, and the file a Nokia or a Sony Ericsson of the 2000s saved a voice memo or a recorded call as, in frames of 4.75 to 12.2 kilobits. It holds speech and little else, at telephone quality, so a minute is under a hundred kilobytes, small enough for the networks of the time, and Android phones can still record it.`,
	},
	'.au': {
		enabled: true,
		reach: 2,
		kind: 'audio',
		mime: 'audio/basic',
		type: 'AU Audio',
		title: 'Audio',
		render: ['webkit', 'gtk', 'mpv'],//WebKit played one on the Mac mini on 2026-10-16, through AVFoundation, Chromium plays none, and GStreamer's good plugins carry an AU parser, so WebKitGTK does by the same reasoning as the other gtk tags
		contactSheet: 'chip',
		about: `Sun's audio file, from its Unix workstations of the late 1980s and NeXT's, where the same file was .snd: usually 8-bit mu-law samples at 8 kilohertz, telephone quality, since the SPARCstation's audio chip was a telephone codec. It was the sound of the early web, the first thing Mosaic could play and the only sound Java's first applets could, so a 1990s web page's sound effect was one. Nothing writes it now.`,
	},
	'.snd': {
		enabled: true,
		reach: 1,
		kind: 'audio',
		mime: 'audio/basic',
		type: 'SND Audio',
		title: 'Audio',
		render: ['webkit', 'gtk', 'mpv'],//as for .au
		contactSheet: 'chip',
		about: `NeXT's name for the sound file Sun called .au, from the NeXT Computer of 1988, since the two companies' formats were one. Inside it is the same telephone-quality sound. Other programs of the time used the name for sounds of their own, so not every .snd is one.`,
	},
	'.voc': {
		enabled: true,
		reach: 1,
		kind: 'audio',
		mime: 'audio/x-voc',
		type: 'VOC Audio',
		title: 'Audio',
		render: ['mpv'],
		contactSheet: 'chip',
		about: `Creative Voice, the sound file of the Sound Blaster from the card's launch in November 1989, when a PC first had a way to play back a recording: 8-bit samples with the Sound Blaster's own blocks for silence and loops, recorded and played by the VOXKIT tool in the box. DOS games of the early 1990s shipped their speech and effects in it, so it dates from the start of PC multimedia beside the first .wav files, and Windows made WAV the standard by 1995.`,
	},
	'.669': {
		enabled: false,
		reach: 1,
		kind: 'audio',
		mime: 'audio/x-669',
		type: '669 Audio',
		title: 'Music as samples and patterns',
		render: [],//a module, as .mod: nothing in the library reads it
		contactSheet: 'chip',
		about: `Composer 669's module, from Tran of Renaissance in 1992, among the first trackers written for the PC, with eight channels and a handful of effects, playing through the Sound Blaster before Scream Tracker arrived. The scene moved on within two years, so the earliest PC demo and BBS music is in it and little after; Modland holds 172 of them.`,
	},
	'.mka': {
		enabled: true,
		reach: 1,
		kind: 'audio',
		mime: 'audio/matroska',//as for .mkv
		type: 'MKA Audio',
		title: 'Audio',
		render: ['edge', 'mpv'],//Chromium reads Matroska when the codec inside is one it has
		contactSheet: 'chip',
		about: `Matroska audio, the 2002 container of .mkv holding sound alone: a whole album in one file with chapters for its tracks, or a film's soundtrack in several languages pulled from a disc. It is far rarer than .mkv, since no ripper writes it by default and MKVToolNix makes most of them.`,
	},
	'.rmi': {
		enabled: false,
		reach: 1,
		kind: 'audio',
		mime: 'audio/midi',
		type: 'RMI Audio',
		title: 'Music as notes',
		render: [],//as for .mid
		contactSheet: 'chip',
		about: `A MIDI file wrapped in Microsoft's RIFF container, the same wrapper WAV uses, from the Multimedia PC extensions of Windows 3.1 in 1992. Windows' own MIDI tools wrote it and Media Player plays it, and inside is an ordinary Standard MIDI File, so a player that knows the wrapper gets the notes out. Rare beside .mid, and Microsoft itself lists it as deprecated.`,
	},
	'.m3u': {
		enabled: false,
		reach: 2,
		kind: 'playlist',
		mime: 'audio/x-mpegurl',
		type: 'M3U Playlist',
		title: 'Playlist',
		render: [],//a playlist, a cue sheet and a subtitle file are text beside the media, and fuji names them and does no more
		contactSheet: 'chip',
		about: `The playlist of Winamp, from 1997: a text file listing the paths of songs, one per line, in the order to play them, with a comment line per track for its title and length. Every music player since has read it, internet radio stations hand one out to start a stream, and .m3u8 is the same file in UTF-8, which Apple borrowed in 2009 as the index of an HTTP Live Stream. Playlists from the MP3 years are in it, pointing at files that have since moved.`,
	},
	'.m3u8': {
		enabled: false,
		reach: 2,
		kind: 'playlist',
		mime: 'application/vnd.apple.mpegurl',//RFC 8216's type for an HLS playlist, which Apple, WebKit and Chromium all pair with the extension; nothing in fuji reads one
		type: 'M3U8 Playlist',
		title: 'Playlist',
		render: [],
		contactSheet: 'chip',
		about: `An .m3u written in UTF-8, so a song with an accent in its name is found; the 8 names the encoding. Music players began writing it in the 2000s, and Apple's HTTP Live Streaming took it in 2009 as the index of a video stream, a list of ten-second segments, so a .m3u8 today may be a playlist of songs or the recipe of a stream, and the second kind is far more common. Inside it is the same text as an .m3u.`,
	},
	'.pls': {
		enabled: false,
		reach: 2,
		kind: 'playlist',
		mime: 'audio/x-scpls',
		type: 'PLS Playlist',
		title: 'Playlist',
		render: [],
		contactSheet: 'chip',
		about: `The playlist Shoutcast adopted for its internet radio in December 1998, from codeArts' museArc player: a small text file in Windows' INI style, a numbered File1, Title1 and Length1 per entry, naming a station's stream or a set of songs. Clicking one on a radio site opened it in Winamp, and one saved from those years names a stream that may no longer exist; Shoutcast's directory still hands them out.`,
	},
	'.cue': {
		enabled: false,
		reach: 2,
		kind: 'cue',
		mime: 'application/x-cue',
		type: 'CUE Sheet',
		title: 'Track list for a disc image',
		render: [],
		contactSheet: 'chip',
		about: `A cue sheet, from Golden Hawk Technology's CDRWIN, the CD burning program of the mid-1990s: a text file saying where each track starts, to the frame, in a single file holding a whole disc, and what the disc and its tracks were called. It is how a CD ripped as one .ape, .flac or .wav still knows its tracks and how one is burned back exactly, so it sits beside those files in every lossless collection, and Exact Audio Copy writes one with each rip.`,
	},
	'.xspf': {
		enabled: false,
		reach: 2,
		kind: 'playlist',
		mime: 'application/xspf+xml',
		type: 'XSPF Playlist',
		title: 'Playlist',
		render: [],
		contactSheet: 'chip',
		about: `The XML Shareable Playlist Format, pronounced spiff, declared final by Xiph.Org in January 2005 as the open playlist that could name a song by its title, artist and album rather than a path alone. VLC adopted it as its own playlist format, and a playlist saved from VLC is one.`,
	},
	'.srt': {
		enabled: false,
		reach: 2,
		kind: 'subtitle',
		mime: 'application/x-subrip',
		type: 'SRT Subtitles',
		title: 'Subtitles',
		render: [],
		contactSheet: 'chip',
		about: `SubRip's subtitle file, from the program of 2000 that read a DVD's subtitle pictures with optical character recognition and wrote them out as text: a number, a start and end time to the millisecond, and the lines, for each subtitle. It became the one subtitle format everything reads, and a downloaded film or series comes with a .srt beside it, named to match, and OpenSubtitles holds them by the million.`,
	},
}

export const fileTypesEnabled = Object.fromEntries(Object.entries(fileTypes).filter(([extension, entry]) => entry.enabled))//the entries fuji acts on today, which is what everything but the settings reads

const platforms = ['mac', 'windows', 'linux']//the words a platform can be, as platform() answers them
export const fileTypesEngine  = {mac: 'webkit',  windows: 'edge', linux: 'gtk'}//the render tag of the web engine Tauri finds on each platform
export const fileTypesLibrary = {mac: 'imageio', windows: 'wic',  linux: ''}//and of the operating system's imaging library thumbnail.rs calls there, which Linux has none of, as the essay says
export function fileTypesRoads(platform, kind) {//the render tags the parts for this kind take on this platform, so a reader passes platform() in and names no tag or platform itself: for a picture the imaging library and the web engine; for a sound or a video mpv alone, since fuji never asks the operating system or an engine to play one; and none for a kind past the three media. Which of a picture's two a part takes is that part's own function below
	if (!platforms.includes(platform)) throw new Error(`expected mac, windows or linux: ${platform}`)
	if (kind == 'image') return [fileTypesLibrary[platform], fileTypesEngine[platform]].filter(tag => tag)
	if (kind == 'video' || kind == 'audio') return ['mpv']
	return []
}
export function fileTypesRender(entry, platform) { return fileTypesRoads(platform, entry.kind).filter(tag => entry.render.includes(tag)) }//the roads this entry has on this platform, and empty where fuji cannot open the kind there; a picture's mpv never counts, since the roads for a picture never name it

export function fileTypesSheet(extension, platform) {//how the contact sheet draws a picture of this extension on this platform: img, the engine's own img, for a kind whose contactSheet says so, so a GIF animates and an SVG stays vector even where the imaging library could make a still of it; native, a canvas the imaging library fills, wherever it reads the kind, since its thumbnail comes back small and cheap; page, a canvas the page fills from the engine's decode; and blank where nothing here draws it, or for anything but an enabled picture
	let entry = fileTypes[extension]
	if (!entry?.enabled || entry.kind != 'image') return ''
	let roads = fileTypesRender(entry, platform)
	let engine = roads.includes(fileTypesEngine[platform]), library = roads.includes(fileTypesLibrary[platform])
	if (engine && entry.contactSheet == 'img') return 'img'
	if (library) return 'native'
	if (engine) return 'page'
	return ''
}
export function fileTypesTable(extension, platform) {//how a table shows a picture of this extension on this platform at its own size: img, the engine's decode, wherever the engine reads the kind, since the table's flip was built and measured on it; native, a canvas painted once from the imaging library's pixels, where only the library reads it; and blank where nothing here shows it, or for anything but an enabled picture
	let entry = fileTypes[extension]
	if (!entry?.enabled || entry.kind != 'image') return ''
	let roads = fileTypesRender(entry, platform)
	if (roads.includes(fileTypesEngine[platform])) return 'img'
	if (roads.includes(fileTypesLibrary[platform])) return 'native'
	return ''
}

export function fileTypesWindow(extension) {//which kind of window a file the operating system hands fuji opens in: Video for a video, by the table's kind alone, whether or not fuji plays it on this platform yet, and Browse for anything else, a picture, a sound, a folder or nothing at all, which the shell sorts among its views
	return fileTypes[extension]?.kind == 'video' ? 'Video' : 'Browse'
}

export function fileTypesAssociations(platform) {//the extensions to declare to this platform as kinds fuji opens, in the table's order: an enabled entry with a road on that platform, so a picture's mpv, which fuji takes for no picture, never counts here. associate.js offers the answer for the platform it runs on and the settings show it as the kinds fuji opens; Info.plist types out the Mac's answer by hand and win-setup/registry.js the Windows one
	return Object.entries(fileTypesEnabled).filter(([, entry]) => fileTypesRender(entry, platform).length > 0).map(([extension]) => extension)
}
