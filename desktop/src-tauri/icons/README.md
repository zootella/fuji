# Icons

Every icon Fuji ships: the files the project supplies, what `pnpm icons` makes from them, and where each platform's build takes them from. Every path is written from the repository root.

## The files we make

```
desktop/src-tauri/icons/app-icon.svg                     the disc filling its canvas, the source of the icons Windows and Linux show
desktop/src-tauri/icons/app-icon-mac.svg                 the disc inset, the source of the Mac's icon.icns, which macOS shows before Tahoe
desktop/src-tauri/icons/app-icon-tile.svg                the disc inset further, the source of the Windows Start menu tile
desktop/src-tauri/icons/AppIcon.icon/Assets/disc.svg     the disc for Tahoe and later
desktop/src-tauri/icons/AppIcon.icon/icon.json           how Tahoe draws it: on a white tile, flat, with no glass, sheen or shadow
desktop/src-tauri/icons/tile/VisualElementsManifest.xml  the Start menu tile's manifest, written by hand
desktop/src-tauri/icons/document/document-image.ico      a page with a landscape, on every picture type Fuji registers on Windows
desktop/src-tauri/icons/document/document-video.ico      a page with a strip of film, on every video
desktop/src-tauri/icons/document/document-audio.ico      a page with a note, on every sound
desktop/src-tauri/icons/document/document-sheet.ico      a blank page, for a kind registered later with no icon of its own
site/docs/public/favicon.svg                             the website's disc, written by hand, which pnpm icons never touches
```

The four document icons are custom artwork, supplied as finished files, and nothing generates them.

## What pnpm icons makes

`pnpm icons`, run from `desktop/`, rebuilds every generated icon from the sources above in one run. It runs `tauri icon` on each of the three SVGs, then `scripts.js` copies the files that come out under a generated name into place, and compiles the Tahoe icon.

```
desktop/src-tauri/icons/32x32.png               from app-icon.svg, by tauri icon
desktop/src-tauri/icons/128x128.png
desktop/src-tauri/icons/128x128@2x.png
desktop/src-tauri/icons/icon.ico
desktop/src-tauri/icons/mac/icon.icns           from app-icon-mac.svg, by tauri icon into .mac/, copied out by scripts.js
desktop/src-tauri/icons/tile/tile-medium.png    from app-icon-tile.svg, by tauri icon into .tile/ as Square284x284Logo.png, renamed by scripts.js
desktop/src-tauri/icons/tile/tile-small.png     and from its Square142x142Logo.png
```

On a Mac with Xcode installed, it also makes:

```
desktop/src-tauri/icons/mac/Assets.car          from AppIcon.icon, by Xcode's actool, run by scripts.js
```

On Windows, Linux, or a Mac without Xcode, it says it skipped `Assets.car` and leaves the committed one as it is, so `pnpm icons` runs anywhere and only a change to `AppIcon.icon` needs the Mac.

The `tauri icon` runs also write files no build reads:

```
desktop/src-tauri/icons/64x64.png, icon.png, icon.icns, Square*Logo.png, StoreLogo.png, android/, ios/
desktop/src-tauri/icons/.mac/, desktop/src-tauri/icons/.tile/     the two extra runs' scratch folders, gitignored
```

**After a run, expect `icon.icns`, `mac/icon.icns` and, on a Mac with Xcode, `mac/Assets.car` to come back modified.** `tauri icon` writes an `.icns` file's records in a different order every time, with every image byte-identical, and `actool` writes a new timestamp and new generated names for its small pictures every time, about 300 of `Assets.car`'s 760 KB, so committing them changes nothing anyone sees. Any other modified file is a real change. Run `pnpm icons` after changing a source, and after upgrading Tauri's command line tool, which never refreshes these committed files on its own.

## How the builds take them

**The Mac:**

```
desktop/src-tauri/icons/mac/icon.icns    → Fuji.app/Contents/Resources/icon.icns    bundle.icon in tauri.conf.json, named by CFBundleIconFile, shown before Tahoe
desktop/src-tauri/icons/mac/Assets.car   → Fuji.app/Contents/Resources/Assets.car   bundle.resources in tauri.macos.conf.json, named by CFBundleIconName in Info.plist, shown by Tahoe and later
desktop/src-tauri/icons/mac/icon.icns    → the dmg's own volume icon                volumeIcon in desktop/dmg.js
```

**Windows:**

```
desktop/src-tauri/icons/icon.ico                         → fuji.exe's own icon, compiled in by tauri's build step, and setup.exe's, by desktop/win-setup/win-setup.js
desktop/src-tauri/icons/tile/VisualElementsManifest.xml  → beside fuji.exe as fuji.VisualElementsManifest.xml    bundle.resources in tauri.conf.json
desktop/src-tauri/icons/tile/tile-medium.png             → beside fuji.exe
desktop/src-tauri/icons/tile/tile-small.png              → beside fuji.exe
desktop/src-tauri/icons/document/*.ico                   → beside fuji.exe, each named in the registry by desktop/src/associate.js for the types of its kind
```

**Linux:**

```
desktop/src-tauri/icons/32x32.png        → usr/share/icons/hicolor/32x32/apps/fuji.png in the deb and the rpm        bundle.icon in tauri.conf.json
desktop/src-tauri/icons/128x128.png      → usr/share/icons/hicolor/128x128/apps/fuji.png
desktop/src-tauri/icons/128x128@2x.png   → usr/share/icons/hicolor/256x256@2/apps/fuji.png
```

The flatpak takes those three from the deb and installs them under the app's identifier at 32x32, 128x128 and 512x512, in `linux/inside-flatpak.py`, since freedesktop has no folder named 256x256@2.

**The Windows files go to every platform.** `bundle.resources` in `tauri.conf.json` applies everywhere, so the tile and the four document icons also ride inside `Fuji.app` and the Linux packages, unused there. They're small, and one list is simpler than a list per platform.

## The sizes

Every source is the same flat disc in `#9FFFE0` on a 1024 canvas, at a different radius, because each place wants a different margin.

- **512 in app-icon.svg, the full canvas.** Windows draws an icon exactly as it comes and reads a margin as a defect, and Linux takes the same files.
- **446 in app-icon-mac.svg, 87.1% of the canvas.** Before Tahoe, macOS drew an `.icns` exactly as authored, at any shape, so the margin had to be in the file. 446 is where Apple's own round icons sit, measured against macOS 15.7.4's applications, Paired Devices to the pixel.
- **338 in app-icon-tile.svg, two-thirds of the canvas.** The Start menu tile's disc, judged right at the first radius on the Windows box on 2026-09-11.
- **410 in AppIcon.icon's disc.svg, 820 pixels across, 80% of the canvas.** Chrome's and Spotify's logos measured at that width on Tahoe's tile, and a disc of radius 410 lands on Chrome's pixels, measured on macOS 27.0.1 on 2026-10-22.

## The Tahoe icon, and the jail

**From Tahoe on, macOS owns the tile.** Every app's icon sits on the same rounded square, 824 pixels of the 1024 canvas, and the system draws it. An app hands in a square picture. From a fully opaque square, the system cuts its tile and finishes it with its own rim. Any other shape it shrinks onto a gray tile of its own, which critics call icon jail. Fuji's bare disc sat in it, seen on macOS 27.0.1 on 2026-10-22.

**So Fuji's Tahoe icon is the disc on a white square, drawn flat.** Fuji's identity is a flat circle, so the disc gets no glass, sheen or shadow, and the white tile is the one part the system requires.

**It's compiled rather than drawn into an `.icns`.** The system jails the 16 and 32 point pictures of any `.icns` at 1x, however they're stored, and the Mac mini Fuji is developed on has a 1x display. An Icon Composer icon, compiled into `Assets.car`, is right at every size. `CFBundleIconName` names it, and `CFBundleIconFile` still names `icon.icns`, which earlier systems show.

**Xcode is installed for this one step, and active for nothing else.** `actool` comes only with full Xcode, not with the Command Line Tools. The Mac keeps the Command Line Tools as its active developer folder, and `pnpm icons` reaches Xcode through `DEVELOPER_DIR` for the one command. `Assets.car` is committed, so no build needs Xcode: only a change to `AppIcon.icon` does.
