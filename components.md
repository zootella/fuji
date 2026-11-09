# Components

## How the components get built

**The components are `FujiList`, the list view; `FujiTree`, the tree view; and `FujiBar`, the toolbar.** The list and the tree are the views every file manager has beside its icons. A menu is a toolbar too, one with text and no icons where every item leads to a submenu, so fuji's menus are `FujiBar` rather than a component of their own.

**The research behind them** is a survey of existing components, and a table of the keyboard and mouse interactions compiled from Apple, Microsoft, VS Code and the ARIA guide.

**Incrementally, with `pnpm local` running.** Build a small piece, look at it in the running program, then the next piece, so the work never gets over its skis. Simple all the way through: simple code is shorter, and it tends to run faster.

**One list on every platform, informed by Explorer and Finder, and its own thing.** Each choice, in how the list looks and in how it behaves, starts from what Windows Explorer and Finder do, because those are what a user already knows. Then fuji makes the choice itself, for a list that is simple, familiar and clean and the same on Windows, the Mac and Linux, and the comment on each choice names who it follows, such as a type-ahead timeout taken from Chromium or a focus rectangle taken from Windows. Where Explorer and Finder disagree, or another program does it better, fuji takes one answer and every platform gets it. The alternative is copying Explorer exactly on Windows and Finder exactly on the Mac. That is a list per platform, with code and selectors for each and a lot of back and forth between machines; an imperfect match reads as wrong to a user who knows the real thing, so every miss would be suspect; few users would benefit; and on Linux there is no one file manager to copy. A list built from both and copying neither becomes fuji's own. For the look, the choice is Swing's, one look consistent across platforms, without Swing's result: the list and the tree are made of web and do not try to match the native controls pixel for pixel, and they look good by using the system's own typography correctly and keeping their appearance flat and simple.

**An exception would customize the list to the platform it runs on, where matching the platform is what makes the list right there.** None is one today: even the highlight color, where the case was strongest, is one accent on every platform, as the highlight color below describes. A new case is weighed by whether matching the platform there is what makes the list read as the machine's rather than a web page's. Command on the Mac doing what Control does elsewhere is not an exception: it is the same behavior under the platform's own key, as in every cross-platform program, since a Control-click on the Mac is a right click.

**The feel: nothing a desktop user reaches for is missing.** The components have to be complete and robust enough that a native desktop user never finds them limited or incomplete. The failure is a web list that is clearly a web page and not Finder or Explorer, which is fine, but skips the ordinary second-order interactions that make a list feel like a list: control-click to toggle one item in the selection, shift-click to extend it, the arrow keys moving a keyboard focus that is separate from the selection, a name typed to jump to it. A user who sees such a list, thinks "this is like my file manager," tries one of those, and meets nothing, decides the whole thing is incomplete and worse than what they already have. Google Drive's file list is the kind of list meant, and its own shortcuts page documents a focus that moves under Ctrl with the arrows, as fuji's list moves its focus under the command key. Fuji's list and tree pass the test.

**Showcases and innovations.** Some features are in the art but rare, and fuji's list and tree showcase them. Others are interactions fuji's list adds beyond what Finder and Explorer do, which are built after the rest of the list, and then written up in the article on fuji's design choices or in the List and Tree article on the site. Each such item below says which it is.

**Shipping on npm.** Whether the components ship on npm as `@zootella/components` is still to answer.


## The order of work

**The suggested order of work**, each step seen running before the next: the full text of a cut-off cell shown in place; the list scrolling while a selecting drag is past its edge; and the bar last, with the menu as its first use.


## FujiList

**The full text of a cut-off cell, in place.** When the pointer rests on the text of a cell that is cut off, the whole text shows exactly over it, in the same face at the same place, so it seems simply to carry on past the edge while the rest of the row stays where it was; a right-aligned value grows to the left, its right edge on the column's. Windows calls this an in-place tooltip, "used to display text strings for objects that have been clipped," with text "identical to the complete label text, and normally is the same size and font," placed to overlay the label, shown after the tooltip delay, by default "the user's double click time and is typically about one-half second," and gone on a click or when the pointer leaves ([in-place tooltips](https://learn.microsoft.com/en-us/windows/win32/controls/implement-in-place-tooltips), [tooltip controls](https://learn.microsoft.com/en-us/windows/win32/controls/tooltip-controls)); the list view does it with `LVS_EX_LABELTIP` ([extended list-view styles](https://learn.microsoft.com/en-us/windows/win32/controls/extended-list-view-styles)). AppKit calls it an expansion tooltip: when the mouse "is hovered over the text," "the full contents will be shown in a special floating tool tip view," off by default on a control and recommended for a table's text fields, `allowsExpansionToolTips` in the macOS SDK's `NSControl.h`. Open: the delay, Windows' half second or at once, once Finder and Explorer are watched on their machines; and the look, the row's own background, so the text reads as carrying on, or a tooltip's border and shadow.

**Scrolling at the edge at a speed a person can steer, an innovation.** While a drag selects past the list's edge, the list scrolls faster the further past the edge the pointer goes, smoothly, so a user can creep a row at a time or race to the end and everything between. Explorer's does not: its scrolling jumps from too slow to the whole list at once, with nothing in between, and no file manager is known to get this right. What others do, read 2026-10-29: GTK 4's list views scroll while the pointer is within 30 pixels inside the edge, faster the deeper it goes into that margin (`SCROLL_EDGE_SIZE` in `gtklistbase.c`); Qt scrolls within 16 pixels of the viewport's edge, its [`autoScrollMargin`](https://doc.qt.io/qt-6/qabstractitemview.html); VS Code scrolls only for drag and drop, in a 35-pixel zone at up to 14 pixels a frame. A write-up on Windows 11 says that past Explorer's top or bottom edge "a small overshoot scrolls more slowly than a large one" ([Geek Rewind](https://geekrewind.com/select-multiple-files-with-a-selection-rectangle-in-windows-11/)), which differs from what the user sees and is checked on the Windows box.

**No ellipsis, an innovation.** The list cuts off text too long for its column and draws a line on the edge beside that row, where other lists draw an ellipsis. An ellipsis takes a lot of width right where space is tight, and it makes the reader stop and wait, as though a speaker were about to go on, which pulls attention away from the list.

**Shading the sorted column.** Explorer fills the column the list is sorted by with a very light gray that is still noticeable, and Finder does not. What fuji's list does is open, a choice for one list on every platform.

**No candy stripes.** Finder shades alternate rows a little apart down the whole list, and fuji's list does not, since the stripes read as old computer paper.

**The highlight color, one accent on every platform.** A selected row with the keyboard, and a menu's row under the pointer, are painted in one accent, `accent.color` in `fuji.toml`, `#0064e1` at the factory and the user's to change; a selected row without the keyboard is a Zed gray. The stylesheet's essay says why the platforms' own colors were set aside: Windows' own programs disagree among themselves, and the platforms' neutral grays clashed with Zed's. Windows 11 added a border around selected items in a 2024 update ([Eleven Forum](https://elevenforum.com/t/the-blue-outline-in-file-explorer-suddenly-started-appearing.35226)). The desktops' sources below are kept for the List and Tree article. What the desktops' own sources say, read 2026-10-28: GNOME's libadwaita, on Debian, Fedora and under Ubuntu's theme, fills a selected row in a view with the accent mixed 25 percent over the view's background and leaves the text its usual color ([_views.scss](https://raw.githubusercontent.com/GNOME/libadwaita/main/src/stylesheet/widgets/_views.scss), [_colors.scss](https://raw.githubusercontent.com/GNOME/libadwaita/main/src/stylesheet/_colors.scss)), with the blue accent `#3584e4` over `#ffffff` in light and `#1d1d20` in dark ([css-variables.md](https://raw.githubusercontent.com/GNOME/libadwaita/main/doc/css-variables.md)). KDE's Breeze selects in `#3daee9` in both light and dark, with white text and no separate color for an inactive list ([BreezeLight.colors](https://invent.kde.org/plasma/breeze/-/raw/master/colors/BreezeLight.colors), [BreezeDark.colors](https://invent.kde.org/plasma/breeze/-/raw/master/colors/BreezeDark.colors)). Mint-Y's source selects in its accent, `#35A854`, with white text in both ([_colors.scss](https://raw.githubusercontent.com/linuxmint/mint-themes/master/src/Mint-Y/gtk-3.0/sass/_colors.scss)), though Mint 22.2 made the accent the user's choice. Ubuntu's Yaru selects in orange, hex not found.

**Each column's sort breaks its own ties.** When a header sorts the list, rows that tie, files of the same size or the same date, need an order among themselves. The sort a column plugs in brings its tie-breaker with it, so the same click always gives the same order, however the user arrived. For instance, Size and Date may break ties by name, in fuji's name sort and in the direction the column is sorted: a list sorted by Size from largest down holds its tied files in reverse name order, and reversing a sort turns the whole list over, ties included. The spreadsheet's way, a stable sort of the order already on screen that makes the column sorted before the second key, is the one Microsoft documents for Explorer's list view, and it suits a table a person is studying, where the sorts they clicked through are layers they built. In a file listing the order of earlier clicks is usually an accident, so tied files would land in an order nobody chose, which changes with how the user got there.

**Rename in place, and rename with the arrow keys, a showcase.** The list renames a file where its name is, and how a rename begins is open: a key, F2 in Explorer and Return in Finder; a click on the name of a row that is already selected, which both file managers take as a rename after a pause; and the timeouts that tell that click apart from the start of a double-click and from a drag. Under the list's rule for selecting and carrying, where every part of a row does the same thing, that click would be anywhere on a selected row rather than on its name. The keys and the pauses are from memory, and are checked on each machine as the rename is built. Two ideas are transplanted from XYplorer and attributed to it: while renaming a file, the up and down arrows move the rename to the file above or below, and the I-beam cursor keeps its column as it goes, so the whole listing behaves like a text editor. Brilliant, and copied. Before the article names XYplorer for it, its own documentation is checked for what it does and what it calls it.

**Space on a folder goes into it.** Once the list shows a folder's subfolders beside its files, space with the keyboard on a subfolder opens it, rather than toggling it into the selection as space does on a file. Neither file manager does this, from memory: Explorer opens a folder with Enter and space only selects, and Finder's space is Quick Look.

**Update when the pointer is away.** Chrome's tab strip set the standard: close a tab and the rest could all widen, and they do not until the mouse leaves the strip, so the next click lands where the last one did. A control whose contents move under the pointer turns every click into a chase after a herd of targets, and the experience is awful. So when rows are flowing into a sorted list, say a list sorted by Name with the pointer resting over a patch of A names while more names arrive in bursts over seconds, the rows under the pointer hold still. The list shows that it is stale with a mark beside the sort column's header, an asterisk on the sort, essentially. When the pointer moves away, to that header or anywhere else, the list is free to update, the new rows take their places, and the mark goes.

**Media details in the margin, a showcase.** It is a section of the List and Tree article in its own right. A file manager's grid has Name, Date, Size and Type, and no place for the one more thing a media file knows about itself: an image's dimensions, a song's or a video's runtime, a video's bitrate. Windows answered with a gazillion more columns and a way to customize each folder's view, and that design is a disaster from end to end. The user does not know how to get there. Arriving, they find fifty columns they have never heard of and not the one they need. They add one, and the columns no longer fit the display. When the view is finally right it belongs to that one folder, not to the folder beside it holding the same kinds of files, and the moment they move the files out, the work is useless. It is so high-maintenance that nobody did it. Fuji knows that solution and kept inventing to avoid it, and the article says so, loudly. `FujiList` notes a file's one useful extra detail in the whitespace of a wide column, pinned to the right margin, and the details are hand-picked rather than everything a file could carry, the few fundamentals media files share: runtime for audio and video, dimensions for images and video.

**Sizes, said the way the user picks, a showcase.** Still open: Kilo follows Explorer from memory, and is checked against it on the Windows box; a way that matches Finder's joins the choices, since Finder and Explorer do not agree on what a kilobyte is, also from memory; and a palette could code the unit like the metals of credit cards, where platinum outranks gold: KB blue, MB silver, GB gold, TB platinum, or something near it.


## Selecting and carrying in FujiList, an innovation

**A press on a row that is not selected selects, and a press on a row that is selected carries.** Dragged from an unselected row, anywhere in it, a press selects the rows it passes over; dragged from a selected row, it carries the whole selection, to a subfolder, to the tree, or out of the window once the list drags files. So a file is dragged by selecting it first, a click and then a drag. `_listSmear` and `_listCarry` in `list.js` are the code.

**Why: every part of a row does the same thing.** Finder and Explorer drag an unselected file pressed on its name or icon, and select from the space beside it. Where a name ends cannot be seen, so the place that drags grows and shrinks with each name's length: a long name is almost all drag and a short one almost all select, and a folder of short names behaves differently from a folder of long ones. Here the one thing that decides is what the user can see, whether the row is highlighted.

**How it differs from Finder and Explorer, slightly.** Like both, a press on a selected row drags the selection. Like Finder's list from beside a name, and Explorer's rectangle from blank space, a press elsewhere selects, but here from anywhere in the row; what each does is from memory, and checked on each machine as the article is written. Unlike both, an unselected file cannot be dragged in one motion. The difference fails safe: a user who presses a name expecting to move the file gets a highlighted run, nothing moves, and the next drag carries it. The opposite slip, a drag meant to select that drops files into the folder beside them, is the costly one, and this rule cannot make it.

**The pointer while carrying is the closed hand for now, a spike.** It is CSS's `grabbing`, which every engine draws in its own way, and it does for trying the carry. What it becomes is open: pointers fuji authors and brings along, drawn to look good beside the system's own on Windows and the Mac, or the plain pointer with something floating beside it, like a count of the files being carried.


## FujiTree

**The tree stands on a panel.** A tree beside the content is Zed's panel.background, One Light's #ebebec and One Dark's #2f343e, which joins the palette under its Zed name when the tree paints it.


## FujiTree's keys beyond the arrows

**Type-ahead.** Typing the start of a folder's name moves the selection to the next folder on screen whose name starts with it, as the list's `_listFind` does.

**`*` opens every folder at the selected folder's level.** The ARIA tree pattern lists it as optional; Windows shows all subfolders with Num Lock and the asterisk; Finder's Option-click on a triangle opens every folder within.


## The tree's right-click menu

**A right click on a folder in the tree opens a menu for that folder**, as a right click in the list opens one for what it selects.


## Dragging between fuji, the list and the tree

**A drag goes into and out of fuji, between the list and the tree, and within one list or one tree.** The happy path is a user selecting files in the list and dropping them on a folder, in the list or in the tree.

**Dragging out of the tree is open**, until what Explorer does there is checked on the Windows box.


## Remembering which folders were open

**The tree keeps which folders the user opened**, since Apple's guidelines ask that expansion choices be retained and Microsoft's that they persist, so `fuji.toml` is where they go.


## Spacing and sizes

**The tree's behaviors that `tree.js` says are from memory are checked on the Windows box** against the classic Win32 tree.

**A text size setting, later, picks the row height and the icon size from a short table of whole numbers.** On Windows it follows Windows' own text size setting, which fuji does not obey yet; on the Mac it is an option of its own or none. No combination the table offers may collide or look bad.


## Publishing updates to the list and the tree

**Rows arrive gradually, the way search results fill in, and the structure underneath changes**, as when something else changes a drive the tree is showing, or a torrent's files fill in as they download in a sister project that shows them in the same components.

**App code says exactly what changed, and nothing compares the whole.** `listAdd` takes a batch of rows and places each in the current sort and filter; `listChange` takes a fresh record, which finds its row by its identity and takes that row's place, moving it if the sort says so; `listRemove` takes identities alone, since a deletion often carries nothing else. The tree's own versions do the same under a parent named by its identity. Each call costs the rows it names and their places in the order.

**A folder the app has not read yet still shows its twistie.** Its record says it has folders inside, `branch` beside the `children` it does not have yet; opening it fires `onOpen`, and the app reads the disk and hands the folders over with the tree's add.

**A batch costs one sort.** Fuji's sorts are comparators, so a row finds its place by binary search, and a burst of rows lands in one pass rather than a sort per row.

**A record crosses a boundary as a fresh object**, from Rust, from a watcher, from another peer, which is why a row is found by the identity it carries rather than by being the same object the list was handed.


## The light table's menu

**The light table is headed toward having no context menu, and its menu has not been taken out yet.** A menu shows which row is under the pointer by lighting it, and in fuji's fullscreen on the Mac the page hears none of the pointer's moves, so the highlight cannot follow and the menu looks broken; the fullscreen essay in `window.rs` says why. Today a right click on the light table still opens its menu, from `onMenu` in `DiamondTable.vue`.

## FujiBar

**Microsoft's trail, and its high points.** Microsoft's habit, per version of Windows and per version of Office, was to make the next version look different so people would see value in it, and the result over the years was a meandering and worsening design. The bar copies the high points that stand out from that trail.

**The bar stands on the content's own color.** A toolbar in Zed is editor.background, the same as the content it sits over, divided from it by Zed's border rather than by a change of color.

**The Internet Explorer 3 coolbar, from the mid-1990s.** Its buttons were flat until the pointer floated over one, and then an outline appeared around the button. The user could click the moment the border appeared, without even looking for the pointer, because the border said where the pointer was. That is the hover behavior for `FujiBar`'s buttons: flat at rest, outlined under the pointer, and really, really usable.

**The menus of Microsoft's software around 2003**, Visual Studio 2003 or Office 2003, for their appearance. Click a menu title and the title does not move. A card that is the menu, with its title still at the top, shaped like a skyscraper with a house on top, is set forward, drawn with a simple one-pixel outline, and casts a light shadow on everything behind it. The shape tells the user at once where they are and what they can choose next, and it looks really good. The user has a VM or screenshots to share, and the design is studied from those and copied: the card's shape, the outline, the shadow's weight, and how the open title joins the dropdown without a seam.

**Research areas for the bar**: the coolbar as it shipped, so the hover outline is drawn from the real thing rather than from memory; the 2003 menu from screenshots, measured, so the card's outline, shadow and joined title come out right; what the web gives for the same effects, `box-shadow` for the shadow and a one-pixel border for the outline, and how to draw the house-on-a-skyscraper shape as one outlined card, since a title tab and a dropdown drawn as two boxes show a seam where they meet.


## A key and a menu item send the same word

**The text that defines the bars declares each item's shortcut, and FujiBar turns the key into the item's word**, so a key and a menu item arrive at one onCommand identically, and a shortcut is written once, beside the item it chooses. The item on the light table's menu that closes the window would carry Esc, and pressing Esc would send that item's word just as choosing the item does. Until then a key and an item each have wiring of their own that ends at the same function, so no behavior is written twice, only the routes to it.

**A context menu declares shortcuts it does not show.** Apple and Microsoft both say a context menu shows no keyboard shortcuts, so the text can give an item its key while the menu on screen shows only the item's name.


## Icons

**FujiBar starts with Lucide, by name.** The text that defines the bars names each icon by its Lucide name. A bundler cannot see which names a string holds, so fuji keeps a roster of the icons in use, imported by name, and only those reach the build.

**Icons fuji supplies itself, planned together.** A fallback for a kind the system gives no icon for, which is every kind on linux until fuji asks the desktop's icon theme there; and folders told apart by what they hold, like an empty folder, which has an icon of its own in the tree so a user can see it holds nothing and could be removed without a second look.

**Still to be seen on Windows.** Whether the icons draw sharp at 125 and 175 percent, where their square starts half a backing pixel off the grid and the code relies on WebView2 snapping plain layout to whole pixels; and that they are the same in light and dark.

**Monochrome where fuji shows media.** The contact sheet, the light table and the video window keep their icons to one color, so nothing on screen competes with the pictures.

**Color where fuji is a file manager.** With the toolbar, the path bar, the tree and the list on screen, color can help the user find a thing at a glance: a yellow folder, a red X on delete. Lucide's icons are lines in one color, so this means shopping for icon sets that come in color, under a license fuji's GPL can carry.


## System icons on Linux

**Fuji asks no system for icons on linux yet, so every row there keeps the cyan square.** What the desktops offer, from memory and not yet checked on the Linux machine:

- **Standard names.** The freedesktop Icon Naming Specification names `folder`, `text-x-generic` for a plain file, and `image-x-generic`, `video-x-generic` and `audio-x-generic`, the counterparts of the Mac's generic kinds and Windows' stock icons, and the major themes draw them.
- **Per type.** The shared MIME database maps an extension to a type like `application/pdf`, whose icon name is `application-pdf`, and names a generic icon like `image-x-generic` for a theme without the specific one. GIO, already in copper, does the mapping.
- **The lookup.** GTK's icon theme finds a name in the desktop's chosen theme, Adwaita on GNOME and Yaru on Ubuntu, Breeze on KDE, Mint-Y on Cinnamon, and Raspberry Pi OS's own, falling back through the theme's parents to hicolor. It belongs on GTK's main thread, where the Mac's and Windows' calls run on the blocking pool.
- **The flatpak** sees its runtime's icon themes rather than the host's, so without the matching theme installed for flatpaks it may draw Adwaita's icons on a KDE desktop. The deb and the rpm see the host's.
- **Minimal setups,** a window manager with no full theme, fall back to hicolor, which may hold no file type icons, and there fuji's own fallback icon draws instead.
- **Coverage differs by theme,** Adwaita having dropped many of its old full-color icons in recent releases, so how each theme looks at 16 pixels is to be seen on each desktop.

## The article

**The article comes first.** The List and Tree page on the site is written section by section, and the code is built to match it.

**The history section is written and on the site**, in Craftsmanship after File Types. Open: the page has no first paragraph yet, because the thesis says what Fuji's list and tree are, and that waits for the design here; its last line, "the rest of this page is how," is the promise the design sections keep.
