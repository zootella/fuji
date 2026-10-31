# File managers

## The article

**A research article, history all the way down until its last section.** It tells the early history of file managers as products, by their design and the experience of the person using them: the families of design, the programs that started each one, the people who wrote them, and the lineage of ideas from one program to the next. Only the last section turns to fuji and says which conventions fuji took and from whom. It belongs in Craftsmanship, toward the historical end.

**Not the List and Tree page.** List and Tree is the history of software components: the list and tree controls that operating system and platform vendors made for the engineers who write applications. This article is the history of the applications, designed at the product level: how a file manager's windows are laid out, and how a person works in it. Where the two meet, Windows 3.0's File Manager, the Finder's list view in System 7 and Explorer in Windows 95, List and Tree already states the facts with sources, and this article links there rather than telling them again.

**Blockquotes from Nikolai Bezroukov.** He wrote about orthodox file managers at length, and his writing should hold passages that teach in his own voice, the kind a blockquote on the site carries.

**It closes with its sources**, a bibliography in Chicago style, as List and Tree does, since it is a paper of cited research.

**Where it started.** The user named Norton Commander, File Manager and the Finder together as the complete traditional file manager fuji is becoming, with `FujiBar` across the top, `FujiTree` on the left and `FujiList` on the right. Norton Commander turned out to be a different design from File Manager and the Finder, and that difference opened the subject.


## What is known so far

**Everything below is from memory and unchecked**, except where it says List and Tree has already checked it. Each date, name and claim is checked against a source before the article states it, and left out if none can be found.

**Orthodox, after Norton Commander.** Two file lists side by side, each open on a folder of its own. The user selects files in the active panel and presses a function key, and they are copied or moved into the folder the other panel shows. A row of function-key labels runs along the bottom, and a command line sits beneath the panels. Text mode, and the keyboard first. "Orthodox" means keeping to Norton Commander's conventions, and the name is usually credited to Bezroukov. Orthodox file managers stayed especially popular in Russia and Eastern Europe.

**Navigational, or Explorer-style.** A tree of folders on the left and the open folder's contents on the right, in one window that moves from folder to folder, later with back and forward buttons. Windows 3.0's File Manager, Explorer from Windows 95 on, and the Finder's window with its sidebar.

**Spatial.** Every folder is a window of its own, and opening it again brings back that same window at the same place and size, so a folder is an object with a place on the screen rather than a location a browser visits. The classic Mac Finder was spatial from 1984 until Mac OS X made it a browser, which drew John Siracusa's long essays in Ars Technica arguing for the spatial Finder. GNOME's Nautilus made spatial windows the default in GNOME 2.6, in 2004, and later went back to a browser. OS/2's Workplace Shell, in 1992, was spatial and built on objects.

**Columns, or Miller columns.** Each folder opens as a column to the right of its parent, so the path from the top of the disk to the open folder shows at once. Named for Mark S. Miller. NeXTSTEP's File Viewer had them, and the Finder took them as its column view in Mac OS X.

**Tree first, after XTree.** XTree for DOS, from Executive Systems in 1985, filled most of its screen with a tree of the whole disk above a small window of files, and XTreeGold followed it. Whether XTree is a family of its own or an early navigational manager is open.

**The programs, roughly in order of their years:**

- The Xerox Star, 1981, put documents and folders on a desktop as icons.
- The Apple Lisa, 1983, had a desktop of its own, before the Macintosh.
- Bruce Horn and Steve Capps wrote the Macintosh Finder, which shipped in 1984.
- Windows 1.0, in 1985, managed files in the MS-DOS Executive, a list of files with menus, the file manager before File Manager.
- Digital Research's GEM Desktop, 1985, ran on the PC and the Atari ST.
- The Amiga's Workbench arrived in 1985.
- Executive Systems released XTree in 1985.
- John Socha wrote Norton Commander, published by Peter Norton Computing in 1986.
- MS-DOS 4.0, in 1988, brought the MS-DOS Shell, a tree and a file list side by side in text mode.
- Windows 3.0 brought File Manager in 1990, and Microsoft published its source in 2018; List and Tree has checked both.
- Jonathan Potter wrote Directory Opus for the Amiga in 1990, two panels with a bank of buttons between them.
- IBM shipped the Workplace Shell with OS/2 2.0 in 1992.
- Christian Ghisler released Windows Commander in 1993, and renamed it Total Commander in 2002 after Microsoft objected to the name.
- Miguel de Icaza started Midnight Commander in 1994, Norton Commander's design on Unix.
- Windows 95 brought Explorer in 1995, which List and Tree has checked, and by default opened each folder from My Computer in a window of its own, until a later Windows changed the default.
- Eugene Roshal, the author of RAR, released FAR Manager in 1996.
- Volkov Commander and DOS Navigator carried the orthodox design on DOS.
- Mac OS X, in 2001, turned the Finder into a browser with a toolbar and a column view.
- GNOME 2.6 made Nautilus spatial in 2004.


## Sources to find

- Bezroukov's writing on orthodox file managers, on his site Softpanorama, the source of the blockquotes.
- Siracusa's articles on the spatial Finder in Ars Technica.
- Microsoft's winfile repository on GitHub, File Manager's source, which List and Tree already cites.
- Norton Commander's manuals, and Socha's own accounts of writing it.
- Bruce Horn's account of writing the Finder, on Folklore.org.
- Magazine reviews of the day, in BYTE, PC Magazine and InfoWorld, many of them scanned on GUIdebook and in Google Books.


## Fuji's conventions

**The last section says which conventions fuji took, and from whom.** Fuji is a navigational file manager, with `FujiBar` across the top, `FujiTree` on the left and `FujiList` on the right. What else it takes from each family is open, and is decided as the components are built.
