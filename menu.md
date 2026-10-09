# Menus

Every menu fuji has on each platform, what each item does, what fuji has decided to leave out, and what is still open. The menu bar is built in `menu.rs` and the Dock's item in `dock.rs`, and each carries the reasoning for what it offers. In the fences, items macOS or Tauri's menu library adds on their own are marked **(system)**, with the label and shortcut the library gives them; shortcuts are written the way the platform shows them, and a line of dashes is a separator. Checked against the code on 2026-10-09. Of the screen, only the Window and Help menus have been looked at; Edit's last three come from Transmission's Edit menu.

## What the items do

**New Window** is Rust's, and makes a window. The other five go to the page of the window in front.

- **Open…** puts up the system's open box, unfiltered, taking a file or a folder, as one Mac open box can. A folder opens on the contact sheet, a picture on the table the settings name, fullscreen, as a double-click on its thumbnail brings it there, and any other file opens the sheet at its folder.
- **The View item** switches between the contact sheet in a window and the light table fullscreen, and its title says which way it will go.
- **About** shows the settings panel scrolled to its About section.
- **Settings…** shows the settings panel.
- **Fuji Help** opens `https://fujidesktop.app/help` in the system's browser, an address that forwards to wherever help lives on the site.

**With no window open**, which happens only in an installed copy on the Mac, Open… puts the box up on its own and makes a window only for what is chosen: a picture on the preview, as a double-click in Finder opens it, and a folder on the contact sheet, and a cancel leaves no window. The View item does nothing. About, Settings… and Fuji Help make a window first and answer in it, Fuji Help with the browser opening in front of it.

## macOS

The menu bar, and the Dock icon. An installed copy stays running after its last window closes, with its Dock icon and the menu bar, so every item below can be chosen with no window open. A development build is not resident, and closing its last window ends it. Labels that carry the app's name take it from the product name.

```
Menu bar
  Apple menu                        (system)
  Fuji                              the app menu
    About Fuji                      the settings panel at its About section
    ---
    Settings…               ⌘,      the settings panel
    ---
    Services                ▸       (system) filled by macOS
    ---
    Hide Fuji               ⌘H      (system)
    Hide Others             ⌥⌘H     (system)
    Show All                        (system)
    ---
    Quit Fuji               ⌘Q      quits
  File
    New Window              ⌘N      Rust makes a window
    Open…                   ⌘O      the open box, every file and folder, unfiltered
    ---
    Close                   ⌘W      (system) closes the window; the last one closing leaves fuji resident
  Edit                              (system) each gray until something in the window in front can answer it, which the page decides
    Undo                    ⌘Z
    Redo                    ⇧⌘Z
    ---
    Cut                     ⌘X
    Copy                    ⌘C
    Paste                   ⌘V
    Select All              ⌘A
    ---
    AutoFill                ▸       (system) added by macOS to any menu titled Edit
    Start Dictation…        fn D    (system) added by macOS
    Emoji & Symbols         fn      (system) added by macOS
  View
    Show Light Table        ⌃⌘F     fuji's own fullscreen, the table; reads Show Contact Sheet from the table, set by the page as the view changes
    Enter Full Screen       🌐F      (system) macOS adds its own to any menu titled View, and retitles it Exit Full Screen in a Space
  Window                            registered with macOS as the Window menu, which adds its own items and the list of windows
    Minimize                ⌘M      (system)
    Zoom                            (system)
    Fill                    ⌃🌐F     (system) added by macOS
    Center                  ⌃🌐C     (system) added by macOS
    ---
    Move & Resize           ▸       (system) added by macOS
    Full Screen Tile        ▸       (system) added by macOS
    ---
    Remove Window from Set          (system) added by macOS
    ---
    Bring All to Front              (system)
    ---
    [window list]                   (system) every open window by its title, the one in front checked
  Help                              registered with macOS as the Help menu, which adds the search field
    [search field]                  (system) finds any menu item by name
    Fuji Help                       opens the help address in the browser; no shortcut, since with ⇧⌘/ set on it the key opened this menu's search field instead

Dock icon
  right click, or click and hold
    New Window                      fuji's own, at the top; no shortcut, since ⌘N lives in the menu bar
    [the rest]                      (system) every open window by its title, Options with Keep in Dock, Open at Login and Show in Finder, Show All Windows, Hide, Quit
  click                             makes a window when none is open
```

## Windows and Linux

No menu bar and no tray icon, so the window has only its own frame, and closing it ends the process. A menu belongs along the top of the screen on the Mac and inside the window everywhere else, and fuji puts none there. Until there is a toolbar, the empty contact sheet's welcome carries the open box and a link to the help address, and those are the only way to either. Its open box is two buttons, one for a file and one for a folder, on every platform, since the open boxes of Windows and GTK choose one or the other and never both. Settings… is `s` from the sheet, and About is the end of the settings panel.

```
Windows
  Window's system menu (Alt+Space)  (system) Restore, Move, Size, Minimize, Maximize, Close Alt+F4
  Taskbar button                    (system) the window's own; fuji adds no jump list

Linux
  (nothing)
```

## The web view's own menu, every platform

The engine inside the window has a right-click menu of its own, with items like Reload and Print. A release build turns it away everywhere but a field that takes typing, where the engine's Cut, Copy and Paste stay, and a development build keeps it for inspecting the page; that is `onContextMenu` in `Shell.vue`. The preview, the diamond table and the space turn it away in every build, release or not, with `@contextmenu.prevent` on their root.

## No Open Recent, decided 2026-09-15

**Fuji will not have one.** A decision rather than a deferral, so nobody needs to cost it again.

**The metaphor does not survive the move from documents to pictures.** Recent works because a person has a handful of documents in play over a few days, so the last ten really are the ones they want. Pictures arrive in thousands, and the last ten are an arbitrary slice of a folder the user can already open.

**And it would leak.** Picture files are private in a way a spreadsheet is not, and such a menu does not merely reopen them — it *names* them, to anyone glancing at the screen, in a screen share, or in a photograph of a desk. A feature that offers nothing and discloses something is an easy call.

**The cost, named once.** A Dock menu is exactly where Open Recent shines, which is why other applications put it there. So this also decides that fuji's Dock menu stays a single item.

## Next

**A toolbar on every platform, not a menu**, which will carry Open… and Help, and on Windows and Linux be the way to both. The empty sheet's welcome holds their place until then.

## Still open, all of it about traces rather than features

**`NSRecentDocumentsLimit` is not set, and setting it to `0` in `Info.plist` is one line.** Nothing populates a recent documents list today — measured, since `tao`, `tauri` and `muda` mention `noteNewRecentDocumentURL:`, `NSRecentDocuments` and `LSSharedFileList` nowhere — so it changes no behavior. It is insurance against a future code path quietly starting to.

**File → Open… left a trace, on 2026-09-15.** `~/Library/Preferences/app.fujidesktop.Fuji.plist`, the file macOS names for the bundle identifier, then held three keys AppKit's open panel wrote: its size, its position, and `NSOSPLastRootDirectory`, 684 bytes of bookmark data recording the last folder browsed. Not a list of files, but the same class of thing. The box is AppKit's open panel still, called by `dialog.rs` now rather than by the dialog plugin, so it writes the same keys; that is reasoned from the class rather than measured again. Two ways to answer it, neither taken: give the picker an explicit starting folder every time, which leaves the key written but never consulted; or clear those keys on the way out, which `desktop.rs` is shaped for and which means fighting AppKit over its own preferences every launch.

**Unchecked:** whether double-clicking a picture lands it in the system's own Apple menu → Recent Items. That list belongs to Finder and LaunchServices and fuji may have no say in it. Look once, then document it as a limit rather than coding around it.
