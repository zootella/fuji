# Mac to Windows

## The Windows body of the icons

**FujiList and FujiTree now show an icon on every row on the Mac, and the Windows body of `copper/src/icon.rs` is yours to write.** Everything above copper is shared and has never run on Windows: the command in brass, `icon.js`, `iconCache.js`, and the list and the tree. The essays in `iconCache.js` and in copper's `icon.rs` hold the design, and the Mac body is there to read beside them.

**The page asks for a folder, a medium or an extension, by the same rule on the Mac and on Windows, and only where each comes from differs.** A kind fuji opens shows its medium's icon, a picture, a video or a sound, never the icon of an app installed to open it elsewhere, since a double-click on it in fuji opens it in fuji. The folder shows the folder's icon. Every other kind shows the icon for its extension, which is the icon of the app a double-click hands it to.

- **A medium comes from fuji on Windows.** On the Mac the three come from the system's generic kinds, so an mkv on a Mac with VLC shows Apple's movie icon rather than VLC's. Windows has no generic kinds like those, and fuji already carries its own: `document-image.ico`, `document-video.ico` and `document-audio.ico`, which the installer puts beside `fuji.exe`, and which `associate.js` keeps by the same three kinds the page asks for, image, video and audio, as it names them in the registry. A metal never knows fuji's file names, so the page or brass names the file, and `associate.js` already works out where they are; how the command learns the path is yours to choose. These are fuji's own files, so reading them by path is not the reading of a user's file the shell must never do.
- **The folder and every other kind come from the shell, by kind and never by path.** The shell must not open a user's file to answer, and must never hand back a thumbnail; each route below has a switch for that. Ask for exactly the backing pixels the page asks for, never a fixed small or large size, and let the shell pick the nearest image the icon carries and shrink it, as Explorer does at each scale. Which route draws what Explorer draws is the open question, settled by setting fuji's list beside Explorer's details view at 100, 125 and 175 percent:
  - `SHGetFileInfo` with `SHGFI_USEFILEATTRIBUTES`, which leaves the file alone, and the system image list it indexes into. That list is what Explorer draws from, but it comes only in its own sizes.
  - The kind's icon location, then `SHDefExtractIcon` at an exact size. An app from the Store has no location to give.
  - `IShellItemImageFactory::GetImage` with `SIIGBF_ICONONLY` at an exact size, from a shell item made for a name with no file behind it.
- **From an icon to a png.** WIC makes a bitmap from an icon handle with `CreateBitmapFromHICON`, which handles an old icon with a mask as well as one with alpha, and WIC's PNG encoder writes the file. One of fuji's own `.ico` files reaches a handle at an exact size through `LoadImage`. Windows icons are sRGB, which a png with no profile already means. `copper/src/thumbnail.rs` has the WIC objects to follow.
- **A test beside the Mac's.** Copper's icon tests check what the Mac hands back, a png of the right size with its profile, and a Windows test of the same shape belongs beside them, a folder at 16 css pixels and 1.25 coming back as a 20 pixel png.
- **COM on the blocking pool.** The body runs on a pool thread, so COM is set up around each call, as the thumbnail does. The shell's documentation asks some of these calls for a single-threaded apartment, which may differ from what the thumbnail sets up, and that is part of choosing the route.

## What only Windows can show

- **Sharp at 125 and 175 percent.** The icon's square is centered in its row, which at those ratios starts it half a backing pixel off the grid. Nothing nudges it, on the reasoning that the engine snaps plain layout to whole backing pixels. Whether WebView2 draws the icon sharp there is the thing to look at.
- **The same in light and dark.** A file type's icon comes from its app's own icon file, which should not change with the theme. The Mac's are the same pixels in both.
