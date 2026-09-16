# Menus

Fuji's macOS menu bar and its Dock icon menu are built and working. `menu.rs` holds the menu bar and `dock.rs` the Dock item, and each carries the reasoning for what it offers. What is left here is one decision with no code to live in, and the loose ends that follow from it.

Menus are macOS-only. Windows and Linux have no menu bar at all, because a menu belongs along the top of the screen here and inside the window everywhere else.

## Open Recent, decided against 2026-09-15

**Fuji will not have one.** A decision rather than a deferral, so nobody needs to cost it again.

**The metaphor does not survive the move from documents to pictures.** Recent works because a person has a handful of documents in play over a few days, so the last ten really are the ones they want. Pictures arrive in thousands, and the last ten are an arbitrary slice of a folder the user can already open.

**And it would leak.** Picture files are private in a way a spreadsheet is not, and such a menu does not merely reopen them — it *names* them, to anyone glancing at the screen, in a screen share, or in a photograph of a desk. A feature that offers nothing and discloses something is an easy call.

**The cost, named once.** A Dock menu is exactly where Open Recent shines, which is why other applications put it there. So this also decides that fuji's Dock menu stays a single item.

## Still open, all of it about traces rather than features

**`NSRecentDocumentsLimit` is not set, and setting it to `0` in `Info.plist` is one line.** Nothing populates a recent documents list today — measured, since `tao`, `tauri` and `muda` mention `noteNewRecentDocumentURL:`, `NSRecentDocuments` and `LSSharedFileList` nowhere — so it changes no behaviour. It is insurance against a future code path quietly starting to.

**File → Open… left a trace, on 2026-09-15.** `~/Library/Preferences/com.zootella.fuji.plist` now holds three keys AppKit's open panel wrote: its size, its position, and `NSOSPLastRootDirectory`, 684 bytes of bookmark data recording the last folder browsed. Not a list of files, but the same class of thing. Two ways to answer it, neither taken: give the picker an explicit starting folder every time, which leaves the key written but never consulted; or clear those keys on the way out, which `desktop.rs` is shaped for and which means fighting AppKit over its own preferences every launch.

**Unchecked:** whether double-clicking a picture lands it in the system's own Apple menu → Recent Items. That list belongs to Finder and LaunchServices and fuji may have no say in it. Look once, then document it as a limit rather than coding around it.
