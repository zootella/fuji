# Components

Fuji's list and tree, the two views every file manager has beside its icons, and the components they are built from. Begun 2026-10-11 on the Mac mini. The article comes first: the List and Tree page on the site is written section by section, and the code is built to match it.

## The history section of the article

Written and on the site, in Craftsmanship after File Types. Open: the page has no first paragraph yet, because the thesis says what Fuji's list and tree are, and that waits for the design below; its last line, "the rest of this page is how," is the promise the design sections keep.

## How the components get built

**Incrementally, with `pnpm local` running.** Build a small piece, look at it in the running program, then the next piece, so the work never gets over its skis. Simple all the way through: simple code is shorter, and it tends to run faster.

**One list on every platform, informed by Explorer and Finder, and its own thing.** Each choice, in how the list looks and in how it behaves, starts from what Windows Explorer and Finder do, because those are what a user already knows. Then fuji makes the choice itself, for a list that is simple, familiar and clean and the same on Windows, the Mac and Linux, and the comment on each choice names who it follows, such as a type-ahead timeout taken from Chromium or a focus rectangle taken from Windows. Where Explorer and Finder disagree, or another program does it better, fuji takes one answer and every platform gets it. The alternative is copying Explorer exactly on Windows and Finder exactly on the Mac. That is a list per platform, with code and selectors for each and a lot of back and forth between machines; an imperfect match reads as wrong to a user who knows the real thing, so every miss would be suspect; few users would benefit; and on Linux there is no one file manager to copy. A list built from both and copying neither becomes fuji's own. For the look, the choice is Swing's, one look consistent across platforms, without Swing's result: the list and the tree are made of web and do not try to match the native controls pixel for pixel, and they look good by using the system's own typography correctly and keeping their appearance flat and simple.

**An exception customizes the list to the platform it runs on, where matching the platform is what makes the list right there.** The highlight color is the example: with the system's own typography, the color of a selected row is what makes a list read as the machine's rather than a web page's, and Swing brought its own selected colors and looked broken, foreign and undeveloped for it, so fuji paints a selected row in each platform's own colors, as the highlight color below describes. A new case is weighed against that example, by whether matching the platform there does what the highlight color does. Command on the Mac doing what Control does elsewhere is not an exception: it is the same behavior under the platform's own key, as in every cross-platform program, since a Control-click on the Mac is a right click.

**The feel: nothing a desktop user reaches for is missing.** The components have to be complete and robust enough that a native desktop user never finds them limited or incomplete. The failure is a web list that is clearly a web page and not Finder or Explorer, which is fine, but skips the ordinary second-order interactions that make a list feel like a list: control-click to toggle one item in the selection, shift-click to extend it, the arrow keys moving a keyboard focus that is separate from the selection, a name typed to jump to it. A user who sees such a list, thinks "this is like my file manager," tries one of those, and meets nothing, decides the whole thing is incomplete and worse than what they already have. Google Drive's file list is the kind of list meant, and its own shortcuts page documents a focus that moves under Ctrl with the arrows, as fuji's list moves its focus under the command key. Fuji's list and tree pass the test.

**Shipping on npm.** Whether the components ship on npm as `@zootella/components` is still to answer.


## What the list and the tree do

Written as the user meets them. A row is text first, and everything below comes after that works.

**Icons.** A row shows an icon beside its text: a folder, a picture, a file of some kind. Where the icons come from is open. The first choice is the system's own, the same defaults Finder and Explorer draw for a file of that type, so the row reads as the machine's. The alternative is a bundled set of simple, clean, Lucide-like icons, which would look custom rather than native. How a system icon would reach the page is a question of the same shape the thumbnailer already answered once.

**Update when the pointer is away.** Chrome's tab strip set the standard: close a tab and the rest could all widen, and they do not until the mouse leaves the strip, so the next click lands where the last one did. A control whose contents move under the pointer turns every click into a chase after a herd of targets, and the experience is awful. So when rows are flowing into a sorted list, say a list sorted by Name with the pointer resting over a patch of A names while more names arrive in bursts over seconds, the rows under the pointer hold still. The list shows that it is stale with a mark beside the sort column's header, an asterisk on the sort, essentially. When the pointer moves away, to that header or anywhere else, the list is free to update, the new rows take their places, and the mark goes.

**The highlight color, matched to the platform.** A selected row is painted in each platform's own selection colors, which fuji carries built in, the Mac's read from AppKit on the Mac mini, for the reason the exception above gives. The desktops' sources below are kept for the List and Tree article. What the desktops' own sources say, read 2026-10-28: GNOME's libadwaita, on Debian, Fedora and under Ubuntu's theme, fills a selected row in a view with the accent mixed 25 percent over the view's background and leaves the text its usual color ([_views.scss](https://raw.githubusercontent.com/GNOME/libadwaita/main/src/stylesheet/widgets/_views.scss), [_colors.scss](https://raw.githubusercontent.com/GNOME/libadwaita/main/src/stylesheet/_colors.scss)), with the blue accent `#3584e4` over `#ffffff` in light and `#1d1d20` in dark ([css-variables.md](https://raw.githubusercontent.com/GNOME/libadwaita/main/doc/css-variables.md)). KDE's Breeze selects in `#3daee9` in both light and dark, with white text and no separate color for an inactive list ([BreezeLight.colors](https://invent.kde.org/plasma/breeze/-/raw/master/colors/BreezeLight.colors), [BreezeDark.colors](https://invent.kde.org/plasma/breeze/-/raw/master/colors/BreezeDark.colors)). Mint-Y's source selects in its accent, `#35A854`, with white text in both ([_colors.scss](https://raw.githubusercontent.com/linuxmint/mint-themes/master/src/Mint-Y/gtk-3.0/sass/_colors.scss)), though Mint 22.2 made the accent the user's choice. Ubuntu's Yaru selects in orange, hex not found. No page gives Explorer's own colors on Windows 10 or 11, only a 2024 update that added a border around selected items on Windows 11 ([Eleven Forum](https://elevenforum.com/t/the-blue-outline-in-file-explorer-suddenly-started-appearing.35226)), so those are still to be read off the screen on the Windows box.

**Rename in place.** The list renames a file where its name is, and how a rename begins is open: a key, F2 in Explorer and Return in Finder; a click on the name of a row that is already selected, which both file managers take as a rename after a pause; and the timeouts that tell that click apart from the start of a double-click and from a drag. The keys and the pauses are from memory, and are checked on each machine as the rename is built.

**Space on a folder goes into it.** Once the list shows a folder's subfolders beside its files, space with the keyboard on a subfolder opens it, rather than toggling it into the selection as space does on a file. Neither file manager does this, from memory: Explorer opens a folder with Enter and space only selects, and Finder's space is Quick Look.


## The components

The components planned are `FujiList`, the list view; `FujiTree`, the tree view; and `FujiBar`, the toolbar. A menu is a toolbar too, one with text and no icons where every item leads to a submenu, so fuji's menus are `FujiBar` rather than a component of their own.

**The tree shows empty folders.** A folder with nothing in it is still a folder, and the tree draws it as one, so a user can see that it is there and that it is empty, rather than wondering where it went.

**The suggested order of work**, each step seen running before the next: the full text of a cut-off cell shown in place; icons; the list scrolling while a selecting drag is past its edge; the tree, which shares everything the list learned and adds expand, collapse and the empty folder; and the bar last, with the menu as its first use.

### The bar, the coolbar, and the 2003 menu

Microsoft's habit, per version of Windows and per version of Office, was to make the next version look different so people would see value in it, and the result over the years was a meandering and worsening design. The bar copies the high points that stand out from that trail.

**The Internet Explorer 3 coolbar, from the mid-1990s.** Its buttons were flat until the pointer floated over one, and then an outline appeared around the button. The user could click the moment the border appeared, without even looking for the pointer, because the border said where the pointer was. That is the hover behavior for `FujiBar`'s buttons: flat at rest, outlined under the pointer, and really, really usable.

**The menus of Microsoft's software around 2003**, Visual Studio 2003 or Office 2003, for their appearance. Click a menu title and the title does not move. A card that is the menu, with its title still at the top, shaped like a skyscraper with a house on top, is set forward, drawn with a simple one-pixel outline, and casts a light shadow on everything behind it. The shape tells the user at once where they are and what they can choose next, and it looks really good. The user has a VM or screenshots to share, and the design is studied from those and copied: the card's shape, the outline, the shadow's weight, and how the open title joins the dropdown without a seam.

**Research areas for the bar**: the coolbar as it shipped, so the hover outline is drawn from the real thing rather than from memory; the 2003 menu from screenshots, measured, so the card's outline, shadow and joined title come out right; what the web gives for the same effects, `box-shadow` for the shadow and a one-pixel border for the outline, and how to draw the house-on-a-skyscraper shape as one outlined card, since a title tab and a dropdown drawn as two boxes show a seam where they meet.


## Ties in a sort

When a header sorts the list, rows that tie, files of the same size or the same date, need an order among themselves.

**Each column's sort breaks its own ties.** The sort a column plugs in brings its tie-breaker with it, so the same click always gives the same order, however the user arrived. For instance, Size and Date may break ties by name, in fuji's name sort and in the direction the column is sorted: a list sorted by Size from largest down holds its tied files in reverse name order, and reversing a sort turns the whole list over, ties included. The spreadsheet's way, a stable sort of the order already on screen that makes the column sorted before the second key, is the one Microsoft documents for Explorer's list view, and it suits a table a person is studying, where the sorts they clicked through are layers they built. In a file listing the order of earlier clicks is usually an accident, so tied files would land in an order nobody chose, which changes with how the user got there.


## Shading the rows

Explorer and Finder each shade a list in ways the other does not, as the user sees them on Windows and the Mac, and what fuji's list does is open. Each is a choice for one list on every platform, weighed against the exception concept where a case for matching the platform comes up.

- **The sorted column.** Explorer fills the column the list is sorted by with a very light gray that is still noticeable. Finder does not.
- **Between selected rows.** Finder draws a faint separation between neighboring rows in a selection. Explorer does not.
- **Candy stripes.** Finder shades alternate rows a little apart, down the whole list. Explorer does not.


## Text cut off at a column's edge

The list cuts off text too long for its column and melts it over its last pixels, and never uses an ellipsis. An ellipsis takes a lot of width right where space is tight, and it makes the reader stop and wait, as though a speaker were about to go on, which pulls attention away from the list.

**A line at the edge.** Where text is cut off, a vertical line may show the column's edge. Open: whether there is a line at all, and whether it runs the list's full height or only beside the row whose text is clipped.

**The full text of a cut-off cell, in place.** When the pointer rests on a cell whose text is cut off, a tooltip opens exactly over it, the whole text in the same type at the same place, so the text seems simply to carry on past the edge while the rest of the row stays where it was. A cell whose text fits shows nothing. A right-aligned column, which loses the start of a value rather than its end, shows the whole value the same way, with its right edge on the column's. Finder and Explorer both do this: AppKit calls them expansion tooltips, `allowsExpansionToolTips` on a table view, and Windows' list and tree views call them in-place tooltips; that is from memory and not yet checked.


## What the components show off

Features that are in the art but rare, and that fuji's list and tree showcase. Media details in the margin is a section of the List and Tree article in its own right.

**Media details in the margin.** A file manager's grid has Name, Date, Size and Type, and no place for the one more thing a media file knows about itself: an image's dimensions, a song's or a video's runtime, a video's bitrate. Windows answered with a gazillion more columns and a way to customize each folder's view, and that design is a disaster from end to end. The user does not know how to get there. Arriving, they find fifty columns they have never heard of and not the one they need. They add one, and the columns no longer fit the display. When the view is finally right it belongs to that one folder, not to the folder beside it holding the same kinds of files, and the moment they move the files out, the work is useless. It is so high-maintenance that nobody did it. Fuji knows that solution and kept inventing to avoid it, and the article says so, loudly. `FujiList` notes a file's one useful extra detail in the whitespace of a wide column, pinned to the right margin, and the details are hand-picked rather than everything a file could carry, the few fundamentals media files share: runtime for audio and video, dimensions for images and video.

**Sizes, said the way the user picks.** Each way of saying a size is an option, the same on every platform and named for who it follows, so a user who wants the sizes their own file manager shows picks that one. Finder's is an option. Finder and Explorer do not agree on what a kilobyte is, which is from memory and confirmed on each machine as the options are built. ergo's `saySize4` is another, as the code already has it: binary units, up to four places, with a thin space before the unit in proportional text and an ordinary space in fixed-width text, so the face the list is set in decides which. A palette of color schemes could code the unit like the metals of credit cards, where platinum outranks gold: KB blue, MB silver, GB gold, TB platinum, or something near it. Explorer's is another, everything in KB, files under a kilobyte included, so that more orders of magnitude read at a glance from the width the digits take.

**Rename with the arrow keys, and a cursor that keeps its column.** Two ideas transplanted from XYplorer and attributed to it. While renaming a file, the up and down arrows move the rename to the file above or below, and the I-beam cursor keeps its column as it goes, so the whole listing behaves like a text editor. Brilliant, and copied. Before the article names XYplorer for it, its own documentation is checked for what it does and what it calls it.


## Innovations

Interactions fuji's list adds beyond what Finder and Explorer do. They are built after the rest of the list, and then written up in the article on fuji's design choices or in the List and Tree article on the site.

**Scrolling at the edge at a speed a person can steer.** While a drag selects past the list's edge, the list scrolls faster the further past the edge the pointer goes, smoothly, so a user can creep a row at a time or race to the end and everything between. Explorer's does not: its scrolling jumps from too slow to the whole list at once, with nothing in between, and no file manager is known to get this right. What others do, read 2026-10-29: GTK 4's list views scroll while the pointer is within 30 pixels inside the edge, faster the deeper it goes into that margin (`SCROLL_EDGE_SIZE` in `gtklistbase.c`); Qt scrolls within 16 pixels of the viewport's edge, its [`autoScrollMargin`](https://doc.qt.io/qt-6/qabstractitemview.html); VS Code scrolls only for drag and drop, in a 35-pixel zone at up to 14 pixels a frame. A write-up on Windows 11 says that past Explorer's top or bottom edge "a small overshoot scrolls more slowly than a large one" ([Geek Rewind](https://geekrewind.com/select-multiple-files-with-a-selection-rectangle-in-windows-11/)), which differs from what the user sees and is checked on the Windows box.
