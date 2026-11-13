# FujiBar DSL

## One text for every bar

**All of fuji's toolbars and menus are defined in one plain text, and JavaScript never assembles them.** Adding a menu item, moving one or changing its wording is an edit to that text and nothing else, so nobody digs through JSON or JavaScript to find how to change a toolbar or a menu safely, and no brittle code stands between the reader and the change. The code knows a bar by its name and hears the words that come back, as in `barShow('Sheet Menu', {x, y})`. That makes one language in one place, rather than a small language inside each bar with JavaScript around it gluing the bars together.

**The text can be a multiline template literal, as the user's notes say, or a file of its own.** Vite imports a plain text file as a string with `?raw` on the import, so the text needs no JavaScript around it at all, markdown headings and all.

**The Windows resource script is the precedent, and the user misses it.** Since Windows 1.0 in 1985, a program's resource script has held every menu and shortcut table the program has in one text file, as `POPUP "&File"` and `MENUITEM "&Open...\tCtrl+O", ID_OPEN`, and Microsoft's resource compiler built it into a program written in C. It was a declarative format for an interface before XML, JSON or HTML, in years when an application's own file format was expected to be binary that only the C that wrote it could read. Near it, from memory: the Mac had Rez, the text form of its resources in Apple's developer tools; Windows' `.ini` files date from the same years, SGML from 1986, and Lisp had written data as text since the 1960s. KDE applications describe their whole menu bar and every toolbar in one file each.

**What one text gives**, as raised in chat:

- **A submenu is another named section.** A line in the menu bar and a line in a context menu can both name an Open With section, so a shared submenu is written once and reused by name, with no JavaScript composing anything.
- **The whole command set reads in one view**: every word fuji answers to and every shortcut it claims.
- **Mistakes fail loudly at launch.** Fuji reads the text once as it starts, and a submenu naming a section that does not exist, a shortcut claimed twice, or a word given two different icons or shortcuts throws then and names the line.


## Words come out over one channel

**The code that shows a bar asks what the user commands, and hears a word.** It says, put up this context menu and tell me what the user chooses from it, and a word like `Open` comes back to one function. A bar is a roadway over which freeform commands travel, never a laundry list of callbacks hooked up item by item, so making a bar and showing one is easy, portable and one communication channel.

**The shape sketched in chat**, to react to:

```js
function onCommand(word) {//every command the sheet can be given, from its toolbar, its context menu or a key
	if      (word == 'Open')          ...
	else if (word == 'Copy Path')     ...
	else if (word == 'Move to Trash') ...
}

async function onMenu(e) { onCommand(await barShow('Sheet Menu', {x: e.clientX, y: e.clientY})) }//a right click
```

with `<FujiBar name="Sheet Bar" @command="onCommand" />` in the template. `barShow` resolves to the word chosen, or to a blank when the user dismisses the menu, since a blank string means nothing.

**It is Win32's shape, and older than a callback per item.** Every Win32 menu item and toolbar button sends the window one message, `WM_COMMAND`, carrying the item's id, and the window handles all of them in one switch. In Cocoa, from NeXT, a menu item sends an action name like `openDocument:` up a chain of objects until one handles it, and the menu never knows who answers. GTK's actions, XUL's commands and VS Code's command ids work the same way.

**Libraries that hand every item to one handler**, read 2026-10-31: React Aria's menu has one `onAction(key)`, Zag's menu one `onSelect({value})`, Element Plus a `command` event, `@szhsin/react-menu` one `onItemClick` that receives the item's `value`, and RibbonWeb a central `ribbon-command` event carrying `{id, value}`. Electron, Reka UI and `@imengyu/vue3-context-menu` give each item a callback of its own, `click:`, `@select` and `onClick`.


## What the text says, and what it leaves out

**Structure, wording and icons, never visual style.** FujiBar has one look out of the box, and what the text makes easy to change is structure, wording and iconography. For an item it says the words, the underlined letter, the Lucide icon by name, the keyboard shortcut, and whether a button is big or small. For a bar it says which of four kinds it is, a menubar, a toolbar, a menu or a panel, and the kind says which way it runs.

**No user customization and no vendor theming.** Many toolbar and menu systems let users rearrange their bars and vendors theme them, and FujiBar does neither, which keeps the language small. It is flexible and customizable for fuji itself, in the code.

**What makes the big systems big.** XUL, KDE's XMLGUI and VS Code are large for what FujiBar leaves out: users rearranging toolbars, with KDE saving a rearranged toolbar back in its own format; plugins merging their items into the host's bars; theming; and translation. The idea at their center, a table from a word to its label, icon and shortcut, costs fuji a few lines, built while the parser reads the text.

**XUL itself cannot run in fuji.** It is Mozilla's XML language for building interfaces, and only Firefox's engine ever read it; from memory, Firefox stopped accepting it from web pages around Firefox 4, in 2011, and has since moved its own interface toward HTML. WebKit and Chromium never read it, and nothing on npm renders it, so it is a source of ideas.

**`@lumino/commands`, from JupyterLab, is the nearest npm module**, BSD-3-Clause and maintained. It registers each command in code, `addCommand(id, {label, execute})`, and menus place commands by id, `addItem({command: id})` ([example](https://github.com/jupyterlab/lumino/blob/main/examples/example-menubar/src/index.ts)). Its commands are JavaScript objects registered one by one, which is the digging through code the text exists to avoid, and its menus and toolbars come from Lumino's own widgets rather than Vue.

**Icons by name have precedent.** Office's RibbonX names an icon from Office's own set with `imageMso`, VS Code writes a codicon as `$(book)`, freedesktop's Icon Naming Specification has names like `document-open` and `edit-copy`, and Lucide's names read like `folder-open` and `copy`.


## The syntax

**Decided 2026-11-12, and `bar.js` says how the text is written**, with `bars.txt` as the text. The user's first notes were that names are markdown, tags make buttons big or small, nesting is `>>`, and the text starts by saying which way a bar runs. What came of them:

- **Names are markdown headings**, `# Views Panel`, nicknames for the code and the text that the user never sees.
- **Four kinds rather than two directions.** A bar's first line is `menubar`, `toolbar`, `menu` or `panel`, and the kind fixes which way it runs and how its items look, so a context menu cannot be written running sideways. A `menu` is narrow rows down, whether a title, a button or a right click opens it.
- **A name in brackets opens a bar.** An item ending in `[Sort Menu]` opens that menu or panel, as markdown links to a heading; this is the pointer reading of `>>`, spelled as a link, so a shared menu is written once and opened from anywhere by name. Depth and paths, the other readings, were set aside, and so was placing one bar inside another, since arranging controls is a panel's job.
- **A panel is Vue.** The text only names it, so a button can point at it and a misspelled name fails at launch; the page draws its insides in `FujiBar.vue`'s panel slot, in the app's own styles, and a bar inside a panel is a small toolbar from the text placed with `FujiBar`.
- **Big and drawings are told apart by their kind**, `big` and a Lucide name lowercase with dashes, so columns line up for the eye and the alignment means nothing. Shortcuts are still to come; the first sketch told them apart by a `+` or a function key, which a shortcut of one key, like `Esc` or `Delete`, does not have.

**`&` marks the underlined letter** in Windows resource scripts, Qt, Electron, Vim and VS Code's menus. GTK writes `_Open`, and markdown already uses `_` for emphasis, so `&` sits more easily in a text that reads as markdown.

**Menus from plain text that came before**, read 2026-10-31. No package on npm builds menus or toolbars from plain text. xmenu, a small Unix program, reads lines of a label, a tab and an output, nests items by indentation, and prints the output of the item chosen, which is text in and one word out ([xmenu](https://github.com/phillbush/xmenu)). mdBook's `SUMMARY.md` defines a whole book's tree as a markdown list, where indenting nests an item and `---` makes a separator ([format](https://rust-lang.github.io/mdBook/format/summary.html)). `@softwarity/projects` builds a menu from a markdown file whose items are links, and mdmenu has been abandoned since 2017.


## The label and the word

**If the label is the word that comes back, rewording an item changes what the code receives.** Renaming Open to Open File breaks the switch listening for `Open`, against the want that a change of wording touches no code.

**A label that differs by platform has already come up.** A menu that says Exit on Windows and Quit on the Mac and linux hands back two words for one command, and the code listens for both, unless an item can name a word other than its label. GNOME and KDE both say Quit, with `Ctrl+Q`, from memory.

**Every format checked keeps the label apart from the command it sends**, read 2026-10-31:

- **Windows resource scripts** send an integer id in `WM_COMMAND`: `POPUP "&Elements" { MENUITEM "&Oxygen", 200 }`, with `\t` before the shortcut text ([POPUP](https://learn.microsoft.com/en-us/windows/win32/menurc/popup-resource), [About Menus](https://learn.microsoft.com/en-us/windows/win32/menurc/about-menus)).
- **Office RibbonX** has one callback that branches on `control.Id`: `<button id="customButton" label="Button" imageMso="HappyFace" size="large" onAction="OnButtonClicked" />` ([MS-CUSTOMUI2](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-customui2/bbce5012-b383-47f3-a29c-b6b1c59280dd)).
- **The Windows Ribbon Framework** defines each command once, carrying its label, KeyTip and images, `<Command Name="cmdExit" LabelTitle="Exit application" />`, and places it by name, `<Button CommandName="cmdExit" />` ([step by step](https://learn.microsoft.com/en-us/windows/win32/windowsribbon/windowsribbon-stepbystep)).
- **KDE's XMLGUI** places one action in a menu and a toolbar alike, `<ToolBar name="mainToolBar"><Action name="clear" /></ToolBar>`, and sets its label in code ([actions](https://develop.kde.org/docs/getting-started/kxmlgui/using_actions/)).
- **GTK's GMenu XML** gives an item a label and an action, `_New Window` and `app.new`, with `target` passing a parameter to the action ([GtkPopoverMenu](https://docs.gtk.org/gtk4/class.PopoverMenu.html)).
- **XUL** links an element to `<command id="cmd_open">` with `command="cmd_open"` ([MDN archive](https://github.com/mdn/archived-content/blob/main/files/en-us/archive/mozilla/xul/tutorial/commands/index.html)).
- **VS Code's contributions** keep the title and the icon on the command, and a menu entry names the command, a `when` clause and a `group@N` that orders it ([contribution points](https://code.visualstudio.com/api/references/contribution-points)).
- **Tauri** sends an item's id to `on_menu_event`, as in `MenuBuilder::new(app).text("open", "Open")` ([window menu](https://v2.tauri.app/learn/window-menu/)).
- **Electron** separates them only through `role`, and otherwise gives each item its own `click` ([menus](https://www.electronjs.org/docs/latest/tutorial/menus)).
- **Vim** nests by a dotted path and sends the command written after the label, `:amenu &File.&Open<Tab>:e  :browse e<CR>`, with priorities like `80.500` ordering items ([gui.txt](https://vimhelp.org/gui.txt.html#%3Amenu)).
- **mdBook** takes the link text as the label and the link's target as the payload.
- **Qt's `.ui` files, Sublime Text, Emacs's easy-menu, JupyterLab and WPF** separate them too ([Qt](https://github.com/qt/qtbase/blob/dev/examples/widgets/tutorials/notepad/notepad.ui), [Sublime](https://www.sublimetext.com/docs/menus.html), [Emacs](https://www.gnu.org/software/emacs/manual/html_node/elisp/Easy-Menu.html)).

**The ways on the table, all open:**

- **The word defaults to the label.** An item names a different word only where the two differ, so most items are just their name. A line can then carry a word of its own beside its label, which is one more piece of syntax, on one line in one place.
- **A commands section.** Each command is defined once, with its label, icon, shortcut and access key, and the bars list only words, as the Windows Ribbon Framework, XUL, KDE and VS Code do, so a rewording happens once and shows everywhere. Reading a bar then means looking in another place, and a label that differs from its word is still extra syntax on the command's line.
- **The command table built from the bars.** Each bar's lines carry what they need, and the parser collects every word across all the bars into the table. A commands section can come later, if the same words repeat across bars often enough to be a chore; an Edit menu and a few toolbars repeat few.


## Graying, checks and pressed buttons

**The text is fixed, and some things about an item change while fuji runs.** Paste grays when there is nothing to paste, a sort item shows a check, a toolbar button stays pressed. Keeping that state out of the text is what lets the text stay static, so the channel carries something back in as well. The ideas on the table, open:

- **Ask just before showing.** As a menu is about to open, the bar asks one function about each word: is Paste available, is by Name checked. Windows sends `WM_INITMENUPOPUP` as a menu opens and Cocoa asks `validateMenuItem:`, so the code never pushes state at a menu and the state is never stale. The channel stays one channel, with words going out and questions about words coming back.
- **Gray the command, not the item.** In XUL, disabling a command disables every menu item and button linked to it. Since the parser knows every place a word appears, graying a word could gray it everywhere, and the words going in name commands rather than places.
- **Hand the state over.** The code passes the words that are off or checked when it shows a menu, and calls to update a toolbar already on screen.
- **A toolbar has no moment before it shows**, since it is on screen all the time, so asking fits a menu as it opens, and a toolbar needs a call when the state changes or some other moment to ask.

**The toolbar hands the state over, for now.** `FujiBar.vue` takes `pressed`, the words whose buttons show pressed in, and the bench's Bench button is the first to use it. Which way a menu learns its state is still open.


## The responder chain

**A word goes first to whatever has the keyboard, and moves outward until something handles it.** It is NeXT's idea, and still the Mac's. Copy copies the list's selection when the list has the keyboard and the text in a field when the field does, and an item that nothing can handle grays itself. In a page it comes nearly free: dispatch the word as an event at the focused element and let it bubble up. `@lumino/commands` does the same for keys, binding a shortcut to the part of the page whose CSS selector matches where the keyboard is. Whether FujiBar routes its words this way is open.


## Access keys and KeyTips

**The underlined letter is in the user's notes**, beside the keyboard shortcuts. What the platforms do, read 2026-10-31:

- **Hidden until Alt.** Windows has hidden underlines until Alt is pressed since Windows 2000, and a menu opened from the keyboard shows them while one opened with the mouse does not ([Chen 2006](https://devblogs.microsoft.com/oldnewthing/20061005-00/?p=29463), [Chen 2018](https://devblogs.microsoft.com/oldnewthing/20180112-00/?p=97785)). Raymond Chen quotes the Windows 2000 designer: "Aesthetically, these things are distracting and intimidating. Functionally, they're only useful when you're navigating by keyboard."
- **Every menu item, and no toolbar button.** Windows 7's guidelines give every menu item an access key, unique within its level, and toolbar buttons none ([Win7 menus](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus), [Win7 toolbars](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-toolbars)). GNOME also gives every menu item one ([GNOME menus](https://developer.gnome.org/hig/patterns/controls/menus.html)).
- **A letter typed in an open menu.** The vendors disagree. The ARIA guide moves to the next item whose label starts with that letter, and Win32 chooses the item whose access key it is, at once ([About Menus](https://learn.microsoft.com/en-us/windows/win32/menurc/about-menus)). From memory, macOS has no underlined access keys, and its menus jump to an item as its label is typed.
- **Alt and F10.** In Win32, Alt toggles in and out of menu bar mode, and the menu activates as Alt is released ([Chen 2006](https://devblogs.microsoft.com/oldnewthing/20061216-13/?p=28703)); F10 activates the menu bar. Windows 7 shows a hidden menu bar on Alt, with focus on its first menu. GNOME's F10 opens the primary menu. On the Mac, Control-F2 moves focus to the menu bar and Control-F5 to the window's toolbar ([Full Keyboard Access](https://support.apple.com/guide/mac-help/mchlc06d1059/mac), [shortcuts](https://support.apple.com/en-us/102650)). VS Code's toggle mode shows its menu bar on a single press of Alt.
- **KeyTips.** In Office 2007's ribbon, pressing Alt shows a letter over every control, which is how an access key reaches an icon button with no text to underline. Jensen Harris gave the reasons: "One, the Ribbon is dense, and on some tabs, we would have run out of letters quickly. Two, we wanted to provide keyboard access to controls regardless of their display state" ([Stroking the Keys](https://learn.microsoft.com/en-us/archive/blogs/jensenh/stroking-the-keys-in-office-12)). RibbonX gives a control a `keytip`, the Windows Ribbon Framework's command carries its KeyTip, and WinUI shows access keys as keytip badges on Alt ([access keys](https://learn.microsoft.com/en-us/windows/apps/develop/input/access-keys)).
- **In the libraries.** VS Code's widgets have `&` mnemonics and Alt to reach the menu bar ([menubar.ts](https://github.com/microsoft/vscode/blob/main/src/vs/base/browser/ui/menu/menubar.ts)). Reka UI and Zag document no access keys.


## The ribbon, and the bar that stays up

**The user's notes list a tabbed ribbon and a popup bar that stays up**, beside the toolbar, the menu and the context menu.

- **The Windows Ribbon Framework, from Windows 7, is the closest model for a ribbon.** Its XML defines each command once, with its label, its KeyTip and its images, and places it in the ribbon by name.
- **Office's RibbonX, from 2007,** is XML of tabs, groups and buttons, where a button is `size="large"` or normal, which is precedent for big and small.
- **Shrinking declared as data.** The Windows ribbon lists, for each group, the order its buttons shrink in as the window narrows, from large icons to small ones to a dropdown. Harris lists rules from usability testing, among them "No commands ever appear or disappear between chunk sizes," and "Each of the chunk sizes is designed by hand... There's no attempt to 'auto-scale' the UI" ([Scaling](https://learn.microsoft.com/en-us/archive/blogs/jensenh/scaling-up-scaling-down)).
- **A toolbar that runs out of room.** Windows 7: "if a window is too narrow to display a toolbar, the toolbar displays what fits and makes everything else accessible through an overflow button" ([Win7 ribbons](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-ribbons)), with the most used commands kept out of it. WinUI's command bar moves commands into a "see more" menu, VS Code's toolbars and menu bar into a More menu, and Apple says "Don't add an overflow menu manually," since the system adds one ([HIG toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars)).
- **No open-source ribbon on the web is mature.** RibbonWeb, `@wieslawsoltes/ribbon-web` 0.1.1, MIT and 11 stars, has KeyTips on Alt or F10, small, medium and large controls, groups that collapse by priority into overflow, RibbonX import and a central command event, inside an open shadow root ([API](https://wieslawsoltes.github.io/RibbonWeb/docs/api.md.html)). Syncfusion's ribbon is under a community or commercial license, and DHTMLX's npm package is GPL-3.0 from 2018.
- **A bar that stays up is what tear-off menus were.** From memory, Tk's menus tear off by default at the dashed line along their top, and HyperCard had tear-off palettes. Harris on Office 97: "Office 97 introduced 'command bars', an ultra-customizable user interface in which menus and toolbars were really the same thing. Every menu and toolbar could be dragged around to every side of the screen and floated or docked" ([Part 2](https://learn.microsoft.com/en-us/archive/blogs/jensenh/ye-olde-museum-of-office-past-why-the-ui-part-2)).


## One call gives an element its context menu

**An idea raised in chat, open.** Rather than each place that has a menu wiring it itself, turning away the web view's own menu, bringing the window forward and opening the menu at the pointer, one call would say an element has a context menu, naming the menu or a function that picks one from what was clicked. FujiBar would listen for the page's `contextmenu` event and nothing else, which the web view sends for the right button, for a Control-click on the Mac and for the menu key on Windows, though nobody has watched each of those yet; and it would turn away the web view's menu, bring the window forward, open the menu and hand the chosen word to the same handler a key or a toolbar button reaches. Beside it, one test in `keyboard.js` would say whether a press is a menu press, true for the right button and for Control with the primary button on the Mac, which any component that selects on a press asks first.


## Behavior to copy from the libraries

**No package does everything FujiBar needs**, read 2026-10-31: none builds from text, and the closest lack access keys and a ribbon. Every library built on web components puts a shadow root on each item.

- **Reka UI 2.11.0**, MIT and Vue. It has released a Menubar, a DropdownMenu, a ContextMenu and a Toolbar. Its menus move between titles with the arrow keys, return focus to the title on Escape, and have type-ahead, submenus, and checkbox and radio items; hovering another title opens it while a menu is open; a grace polygon guards the path to a submenu, with `graceDuration: 300` ([MenuSubTrigger.vue](https://raw.githubusercontent.com/unovue/reka-ui/v2/packages/core/src/Menu/MenuSubTrigger.vue)); and a pointerup that did not start on an item fires a click, which is press-drag-release ([MenuItem.vue](https://raw.githubusercontent.com/unovue/reka-ui/v2/packages/core/src/Menu/MenuItem.vue)). It ports Radix's design, gives each item its own `@select` and portals its menus into `body`, and the whole library is 189 KB gzipped.
- **Zag's menu machine, `@zag-js/menu` 1.47.0**, MIT and framework-free. It has an intent polygon toward a submenu, type-ahead, long press, and nested and context menus, and hands every choice to one `onSelect({value})` at the root, while the page renders its own elements and styles them by `data-part` ([docs](https://zagjs.com/components/vue/menu), [machine](https://raw.githubusercontent.com/chakra-ui/zag/main/packages/machines/menu/src/menu.machine.ts)). Its items select on click only, so it has no press-drag-release, and its menubar and toolbar exist only as prereleases.
- **VS Code's widgets**, MIT and unpackaged. Its menus write mnemonics with `&`, count a mouseup on an item only 100 ms after the menu opens, which is press-drag-release, and open a submenu 250 ms after hover and hide it 750 ms after focus leaves, with no polygon ([menu.ts](https://raw.githubusercontent.com/microsoft/vscode/main/src/vs/base/browser/ui/menu/menu.ts)). Its menu bar takes Alt mnemonics and the arrows, switches between open menus on hover, and has a More menu and a compact mode ([menubar.ts](https://raw.githubusercontent.com/microsoft/vscode/main/src/vs/base/browser/ui/menu/menubar.ts)), and its action bar moves with the arrows of its orientation, takes Home and End, and is a `role=toolbar` ([actionbar.ts](https://raw.githubusercontent.com/microsoft/vscode/main/src/vs/base/browser/ui/actionbar/actionbar.ts)). The code is TypeScript tangled into VS Code's base layer.
- **React Aria**, Apache-2.0 and React. Its menu hands every choice to one `onAction(key, value)`, and has submenus, long press and context menu triggers, and type-ahead ([Menu](https://react-aria.adobe.com/Menu)). Its path to a submenu watches the angle of the pointer's movement with 15° of padding, throttled at 50 ms, allows two movements away, sets pointer-events to none behind the path and closes after a second ([useSafelyMouseToSubmenu.ts](https://raw.githubusercontent.com/adobe/react-spectrum/main/packages/react-aria/src/menu/useSafelyMouseToSubmenu.ts)).
- **Radix**, MIT and React. Its menu tracks `pointerGraceIntent` and `isPointerMovingToSubmenu` and has a type-ahead search, which Reka ports ([menu.tsx](https://raw.githubusercontent.com/radix-ui/primitives/main/packages/react/menu/src/menu.tsx)).
- **Floating UI.** Its `safePolygon` is in the React build and absent from the DOM and Vue builds ([useHover](https://floating-ui.com/docs/useHover)).
- **Ruled out.** Headless UI for Vue was last published 2024-09-09 and has no submenus or context menus; PrimeVue 5 moved to a license with revenue and headcount limits and a key; Vuetify, Quasar, Naive UI and Element Plus are whole design systems; Web Awesome gives each item its own shadow root and Shoelace is sunset; Vaadin's menu bar collects usage statistics at development time; Material Web is in maintenance mode.


## Menu engineering

- **Submenu delay.** Windows exposes it as `SPI_GETMENUSHOWDELAY`, and it "used to be 4/5 of the double click speed (0.4s), but now it has its own setting" ([Chen 2008](https://devblogs.microsoft.com/oldnewthing/20080423-00/?p=22623)); 400 ms as today's default is from memory. Chen credits the delay with letting a diagonal path survive: "There's a place for maddeningly fiendish mouse dexterity games, but trying to create a pivot table is not it" ([Chen 2008](https://devblogs.microsoft.com/oldnewthing/20080619-00/?p=21903)). GTK 3 opens a submenu after 225 ms, hides it after 1000 ms, and keeps a stay-up triangle toward it extended 50 pixels past its bottom ([gtkmenu.c](https://gitlab.gnome.org/GNOME/gtk/-/blob/gtk-3-24/gtk/gtkmenu.c)). VS Code uses 250 and 750.
- **The safe triangle.** Ben Kamens named it for the web on 2013-03-06: "At every position of the cursor you can picture a triangle between the current mouse position and the upper and lower right corners of the dropdown menu" ([Kamens](https://bjk5.com/post/44698559168/breaking-down-amazons-mega-dropdown)), with a 300 ms delay and a 75-pixel tolerance in his plugin. A secondhand account credits Bruce Tognazzini and Jim Batson with it on the Mac in 1986, and says OS X dropped it by accident ([Qt list](https://lists.qt-project.org/pipermail/interest/2013-May/007344.html)). Smashing Magazine says macOS protects the path into a submenu and not the path back ([Smashing](https://www.smashingmagazine.com/2023/08/better-context-menus-safe-triangles/)).
- **Press, drag, release.** Classic Mac OS required holding the button down; Mac OS 8 made a quick click leave the menu open, while a press longer than the double-click interval kept release-to-choose ([Mac OS 8 HIG](https://discmaster.textfiles.com/file/32774/Dev.CD%20Dec%2098%20RL2.toast/Technical%20Publications/mac/HIGOS8Guide/thig-55.html?html=true)). From memory, the Mac opens a context menu on the press, and Windows menus take press-drag-release as well as a click and a click.
- **Where a context menu opens.** Win32 opens it "at the position of the caret or mouse cursor," and `TrackPopupMenuEx` takes a rectangle the menu must not cover ([TrackPopupMenuEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-trackpopupmenuex)). A menu asked for from the keyboard gets the coordinates (-1, -1), and the program places it at the selection ([Chen 2004](https://devblogs.microsoft.com/oldnewthing/20040921-00/?p=37813)). Apple says a menu may open above or below its content, "so you might also need to reverse the order of items." VS Code's layout reports ok, flipped or overlap, and WinUI's command bar opens its overflow upward unless there is no room.
- **The keyboard in a toolbar.** The ARIA guide gives a whole toolbar one Tab stop, with Left and Right moving between its controls ([APG toolbar](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/)). Windows 7 puts toolbars last in the Tab order, and calls them "less accessible for keyboard users" than a menu bar.
- **The keyboard in a menu bar and a menu.** Left and Right move along the menu bar; Down opens a menu at its first item and Up optionally at its last; in a submenu, Right opens a nested menu or moves to the next menu bar item, and Left closes the submenu ([APG menubar](https://www.w3.org/WAI/ARIA/apg/patterns/menubar/)). Tab leaves the menu bar and closes every menu. Enter chooses an item, and Space toggles a checkbox item without closing the menu. Escape closes the current menu and returns focus to whatever opened it. In WinUI, focus wraps, since a menu "can open in unpredictable directions."
- **Shift+F10 and the Menu key** open the context menu ([WM_CONTEXTMENU](https://learn.microsoft.com/en-us/windows/win32/menurc/wm-contextmenu)). Shift+F10 can bring up an extended menu by accident, since Shift is held, and the Menu key avoids that ([Chen 2011](https://devblogs.microsoft.com/oldnewthing/20110809-00/?p=9943)).
- **What goes in a context menu.** Windows 7 shows the default command in bold, removes unavailable items but disables standard commands like Paste rather than removing them, avoids submenus, and caps a menu at 15 items ([Win7 menus](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus)). Apple hides unavailable items, except that macOS may dim Cut, Copy and Paste, and allows one level of submenu ([HIG context menus](https://developer.apple.com/design/human-interface-guidelines/context-menus)). KDE recommends a real menu bar once a hamburger menu would need more than about 15 items.
- **How deep a menu goes.** Harris: "There's a point beyond which menus and toolbars cease to scale well. A flat menu with 8 well-organized commands on it works just great; a three-level hierarchical menu containing 35 loosely-related commands can be a bit of a disaster" ([Part 5](https://learn.microsoft.com/en-us/archive/blogs/jensenh/tipping-the-scale-why-the-ui-part-5)). On Word 97: "It's much more difficult for people to form a scanning strategy with hierarchical menus."
- **Menus that change themselves.** Harris on Office 2000's adaptive menus: "Auto-customization, unless it does a perfect job, is usually worse than no customization at all," and "The order of commands was no longer constant, scanning for functionality was inefficient, and predictability suffered as nothing could ever be guaranteed to be in the same place even from click to click" ([Part 3](https://learn.microsoft.com/en-us/archive/blogs/jensenh/combating-the-perception-of-bloat-why-the-ui-part-3)).


## Native menus and the web platform

- **Tauri's native menus.** The page can pop up a real system menu with `Menu.popup`, whose permission is part of Tauri's core set. The promise resolves "as soon as the menu is shown, not when an item is picked," and the choice arrives through each item's `action`, which receives the item's id, so one function can receive the word; a native icon on an item is unsupported on Windows and linux ([JS menu reference](https://v2.tauri.app/reference/javascript/api/namespacemenu/)). Underneath is muda, over Win32, AppKit and GTK. Fuji's palette and look cannot reach a native menu. From memory, a native menu brings each platform's own safe triangle and type-ahead.
- **The Popover API** works in every engine fuji runs on, Debian 12's WebKitGTK included, and puts a menu above everything else on the page, so no container clips it.
- **Invoker commands**, the web's own command words, `<button commandfor="list" command="--open">` dispatching an event that carries the word, are on in Chromium 135 and Safari 26.2. WebKitGTK ships them off at 2.50, which Debian 12 carries, and on at 2.52, which Ubuntu 24.04 carries, so fuji's linux floor lacks them.
- **CSS anchor positioning** is complete only in the newest engines, `position-anchor` from Chromium 151 and Safari 27, so a menu placed from script stays the simple answer.


## The look, from the research

**These answer questions about the bar's appearance**, read 2026-10-31.

- **The coolbar.** Nancy Winnick Cluts, October 1996: "One of the controls created for Internet Explorer 3.0 is the Rebar control. It is most commonly known, however, as the coolbar." Her sample toolbar uses `TBSTYLE_FLAT` and "appears 'flat' (rather than having raised buttons)" ([MSDN](https://learn.microsoft.com/en-us/previous-versions/ms997549(v=msdn.10))). In a flat toolbar "both the toolbar and the buttons are transparent and hot-tracking is enabled" ([styles](https://learn.microsoft.com/en-us/windows/win32/controls/toolbar-control-and-button-styles)); a hot button is one "under the pointer or pressed," and a toolbar can hold a separate set of images for hot buttons ([TB_SETHOTIMAGELIST](https://learn.microsoft.com/en-us/windows/win32/controls/tb-sethotimagelist)). GUIdebook has a screenshot of Internet Explorer 3 ([GUIdebook](https://guidebookgallery.org/screenshots/browser)). From memory, Internet Explorer 3 shipped in August 1996, and its icons were grayscale until hovered.
- **The joined menu title, in Microsoft's own code.** WinForms' `ToolStripProfessionalRenderer`, documented as an "Office-style appearance," draws a one-pixel border around the dropdown, then fills the strip where the open title overlaps the dropdown with the dropdown's background, under the comment "make the neck connected," and skips that when the menu opens upward ([renderer](https://github.com/dotnet/winforms/blob/main/src/System.Windows.Forms/System/Windows/Forms/Controls/ToolStrips/ToolstripProfessionalRenderer.cs)). In CSS that is two outlined boxes and a patch over the join. The dropdown's shadow there is a flag, `DropShadowEnabled`.
- **The 2003 flatness.** Chen: "Office XP and Outlook 2003 continue the trend and flatten nearly everything aside from the scrollbar elements. Blocks of color are used to separate elements onscreen, sometimes with the help of simple outlines" ([Chen](https://devblogs.microsoft.com/oldnewthing/20040728-00/?p=38313)). Windows XP's flat menus fill the current item with `COLOR_MENUHILIGHT` and outline it with `COLOR_HILIGHT`. Office 2003 drew from about 1,500 designer-chosen colors following XP's themes, which Harris says limited it to "solid fills and simple gradients" ([Which Color When](https://learn.microsoft.com/en-us/archive/blogs/jensenh/which-color-when-part-1)).
- **No measurements anywhere.** No Microsoft or GUIdebook page gives figures for the 2003 outline, the shadow or the joined title, so those come from screenshots. From memory, Visual Studio .NET 2002 and 2003 used Office XP's menu look rather than Office 2003's.
