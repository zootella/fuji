# Icons

What fuji's application icon is, what each desktop platform expects one to be, and where the two disagree. Everything below was measured on 2026-09-07 against the files in this repository and against macOS 15.7.4's own applications; nothing is reasoned from first principles, and where something has not been researched yet it says so rather than guessing.

`scaffold.md` says how a project like this one is set up, and this says how it gets its icon.

## The design, and why it is a bare circle

**A single flat colour, and nothing else.** No border, no gradient, no shadow, no shine. `#9FFFE0` is a mint that almost smells of photo chemicals, and that association is the whole of the idea — the icon says darkroom without drawing one.

**Simplicity is the differentiator, not a shortcut.** A dock and an app store are cluttered, and something this plain is genuinely rare in both. It is easy to pick out precisely because everything around it is busy.

**Every literal photo icon looks contrived.** A mountain landscape, the Mona Lisa, ACDSee's single close-up black-and-white eye — whatever picture a photo application puts on its icon, the choice reads as arbitrary and dated the moment it is made. Fuji declines the category.

This matters below because it separates two things that are easy to confuse. The disc is a design decision and it is settled. Its *size on the canvas* is a platform requirement, and that is where the problem is.

## The instrument

**Alpha bounding box, at two thresholds.** An icon's real size is not its canvas — it is the box around the pixels that are actually opaque. Measuring any pixel above alpha 8 finds the artwork including its drop shadow; measuring above alpha 200 finds the solid body alone. The difference between those two numbers is the shadow, and the second is the one to compare against a specification.

**The known colour, as a probe.** Fuji's artwork is one flat `#9FFFE0`, so every pixel's true colour is known before looking. That makes a second measurement possible that a normal icon does not permit: read the RGB of the half-transparent pixels along the edge and compare it against the colour it is supposed to be. Anything else is a defect, and the *shape* of the discrepancy names the cause.

Layers come out of an `.icns` with `iconutil -c iconset`, and out of an `.ico` by reading the directory at the front of the file. Both hold ordinary PNGs at every size fuji uses.

**One trap in that instrument.** `iconutil -c iconset` is not a faithful inverse of `iconutil -c icns`. Exporting the 16 and 32 layers back out returns their edge pixels as white, whatever colour they really are — reading the small-size entry straight out of the container instead gives the true mint. Sizes survive the round trip and colours do not, so any colour claim about a small layer has to be read from the icns directly, not from an exported iconset.

## What fuji had, and how it got there

The state the investigation started from, kept because both defects are only legible against it.

**One circle, one command, everything downstream.** `src-tauri/icons/app-icon.svg` is 162 bytes:

    <circle cx="512" cy="512" r="512" fill="#9FFFE0"/>

on a 1024 viewBox — a mint disc that touches all four edges, and still the source for every platform but macOS. `tauri icon` had been run on it once, in commit `931648c custom icon`, generating the whole tree: `icon.icns`, `icon.ico`, the Linux PNGs, the Microsoft Store `Square*Logo` set, and the `android/` and `ios/` folders. `tauri.conf.json` lists five of those in `bundle.icon`.

**The generated `.icns` is well formed.** Twelve entries, real PNGs from 16 through 1024, both legacy masks present. Nothing about the file is malformed, which is what made the problem hard to see: it is a correct container around wrong artwork.

**On macOS one `.icns` reaches three places, byte for byte identical:**

    Fuji.app/Contents/Resources/icon.icns    the dock and finder, via Info.plist CFBundleIconFile
    the Fuji.app inside the mounted dmg      the same file, shown in the installer window
    /Volumes/Fuji/.VolumeIcon.icns           the mounted volume itself

So there is one icon to get right on this platform, not three.

## The rule macOS actually applies

**macOS does not mask app icons.** iOS clips every icon to its own squircle, so artwork can run to the edge and the system makes it fit. macOS draws the `.icns` exactly as authored, at any shape — which is why a Mac icon can be a hammer or a torn page, and why the inset every well-behaved icon appears to have must be *baked into the file*.

**Apple's grid is a 1024 canvas holding an 824 body**: 100 transparent pixels on each side, 80.5% of the canvas. Measured against macOS 15.7.4's own applications, solid body only:

    icon           canvas   body        inset            % of canvas
    Calculator      256     206 x 206   25 25 25 25        80.5%
    TextEdit        256     206 x 241   25 25  3 12        80.5% wide
    Preview         256     213 x 216   25 18 25 15        83.2%
    fuji            256     256 x 256    0  0  0  0       100.0%

Calculator is the template to the pixel — 206/256 scales exactly to 824/1024. TextEdit is the same width and taller because a page shape is allowed to break the square. Preview runs slightly wide because parts of its artwork stick out on purpose. All three sit on the same grid.

**Fuji sits on no grid at all.** Its body is 100% of the canvas at every size, so it renders 1.24× wider than its neighbours in the dock — and reads as bigger still, because a full-bleed circle has no corners falling away to soften the comparison.

## Why the scaffolded app looks right and fuji does not

This is the part worth remembering, because the obvious conclusion is the wrong one.

**A freshly scaffolded Tauri app has a correctly inset macOS icon.** Measured from `pnpm create tauri-app`, its `icon.icns` body is 412 on a 512 canvas — inset 50 on all four sides, 80.5%, Apple's grid exactly. It is a dark squircle with a gradient and a drop shadow.

**Its `icon.ico` is different artwork.** The 256 layer of the same scaffold's Windows icon is the bare two-ring logo on transparency, 187 × 210, no squircle and no shadow.

**One source image resized to every size cannot produce two different compositions**, and the scaffold ships no `app-icon.png` for anything to have been generated from. So the defaults are static, hand-authored, per-platform assets checked into the template. They look right because a designer drew them right — not because the tool produces right output.

**`tauri icon` is that tool, and it adds nothing.** It resizes the source to every size the platforms want, edge to edge, and writes the containers. `--help` offers `-o`, `-v`, `-p`, `--ios-color`, and a manifest; there is no padding or inset option anywhere. The docs ask only for "a squared PNG or SVG file with transparency" and never mention margins.

Tellingly, the manifest does carry `android_fg_scale`, defaulting to 85, for Android's adaptive-icon safe zone. Tauri understood the idea of a platform safe area, implemented it for Android, and has no equivalent for macOS.

**So the trap is precisely shaped:** the default is correct, the first thing anyone does is replace it, and the supported way of replacing it silently drops the property that made the default correct. Running `tauri icon` is the moment the icon breaks, and it breaks on the platform that will not fix it for you.

Reported upstream and still open as [discussion #10999](https://github.com/tauri-apps/tauri/discussions/10999).

## The second defect: a grey fringe on every curved edge

**Symptom.** Against the white of a Finder window — the dmg installer worst of all — the disc has a dirty outline. Some edge pixels are a darker, desaturated cyan; the border reads as jagged rather than smooth.

**It is not jaggedness.** The edge is properly antialiased. The transition is simply going through the wrong colours: mint, then *grey*, then white. The eye reads a non-monotone ramp as a stair rather than a curve, which is why it looks jagged when nothing is aliased.

**Measured.** Reading the faintest edge pixels of the committed files, against the mint they should all be:

    file                    alpha   rgb found      rgb it should be
    app-icon.png  SOURCE      16    159,255,223     159,255,224     correct
    icon.icns 512 layer        9      3,  5,  4     159,255,224     wrong
    128x128.png               14     16, 25, 22     159,255,224     wrong
    icon.ico 256 layer        11     13, 22, 19     159,255,224     wrong

The source is clean. Everything generated from it is wrong, and wrong in one exact way: the RGB has been scaled down in proportion to the pixel's own alpha. At alpha 9 of 255, the mint has been multiplied to a thirtieth of itself — very nearly black.

**Every generated file, on every platform.** All twelve `.icns` entries, all six `.ico` layers, the Linux PNGs, and the Microsoft Store logos. This is not a macOS problem.

**The cause.** Fuji's `app-icon.png` stores black in its fully transparent pixels — 223,272 of them, RGB `(0,0,0)` at alpha 0, which is what an SVG renderer ordinarily produces and is harmless in a correctly written file. The old generator resized the four channels independently, so along the edge the Lanczos filter averaged mint against that invisible black and kept the result. The alpha channel averaged the same way, which is why the darkening tracks the alpha so exactly. A renderer then applies the alpha *again* when compositing, and the edge lands halfway to black.

**It was a real bug, and it is already fixed.** [Issue #14351](https://github.com/tauri-apps/tauri/issues/14351), closed, fixed by premultiplying before the resize and undoing it afterwards. Verified here rather than taken on trust — the same `app-icon.png` through `tauri-cli` 2.11.4, the version this repository already depends on:

    file                 old cli                new cli 2.11.4
    128x128.png          16, 25, 22   wrong     163,255,218   correct
    icon.png             3,  5,  4    wrong     169,254,226   correct
    icns 512 layer       3,  5,  4    wrong     169,254,226   correct

So fuji's icons are simply stale. They were generated in `931648c` by a CLI old enough to have the bug and have not been regenerated since.

**Why simplicity did not prevent this, and why that is not a mark against it.** A flat disc of one saturated colour is the worst possible case for an edge defect — a large uniform field ending at a hard curve, with nothing anywhere to hide a bad pixel. A busy icon with a drop shadow already has dark pixels along its border, so the same fringe is invisible on it. The simplicity did not cause the bug; it made it *visible*. Every Tauri application built with that CLI has this fringe, and almost none of them can see it. It is also the reason it could be diagnosed at all: with one known colour, the arithmetic is unambiguous.

## Why upgrading Tauri did not fix it

The modernization brought the CLI from `^2` to 2.11.4, which contains the fix. The fringe survived it anyway, and the reason is worth keeping.

    2025-07-08   931648c "custom icon" — tauri icon run once, on a CLI resolved from "^2"
    2025-10-22   the fringe reported upstream as #14351
    2025-11-04   fixed in PR #14353, shipped in @tauri-apps/cli 2.9.3
    2026-09      fuji's CLI is 2.11.4, and its icons are still the files from July 2025

**`tauri icon` is a one-shot generator, not a build step.** Nothing in `package.json` calls it, and `pnpm build` never does. It was run by hand once, fourteen months ago, and its output was committed.

**So the upgrade upgraded the tool and could not touch the files.** A dependency bump reaches code that runs at build time. It cannot reach an artifact that was generated once and checked in — that artifact is frozen at the version that made it, and it stays frozen until somebody re-runs the generator on purpose.

Worth remembering beyond icons: every generated-and-committed asset in this repository has the same property, the `android/` and `ios/` icon trees included. Upgrading never repairs them, and nothing will report that they are stale.

## Regenerating, and the one command that does it

**One run covers every platform.** `tauri icon` is not a per-platform tool run on a per-platform machine — it reads one source and writes the whole tree at once, `.icns` and `.ico` and the Linux PNGs and the Store logos and the mobile trees together. Every one of those is a build artifact committed to this repository, so a Mac regenerates the Windows icon exactly as well as a Windows machine would. There is nothing to do on the other platform.

    pnpm icons

added to `package.json` for that, taking `app-icon.svg` directly — the CLI accepts SVG as readily as PNG, and the default output directory is already `src-tauri/icons` because that is where `tauri.conf.json` lives.

Run it from `desktop/`, the workspace that owns the script; every path in this document is written from there, the way the code refers to itself.

**The `.ico` carries the fringe today and one run clears it**, verified layer by layer:

    layer   committed, old cli        after pnpm icons, 2.11.4
    16      37, 59, 52   fringe       159,255,223   clean
    24      21, 34, 30   fringe       159,255,223   clean
    32      17, 27, 23   fringe       159,255,223   clean
    48      18, 29, 26   fringe       159,255,223   clean
    64      15, 24, 21   fringe       159,255,223   clean
    256     13, 22, 19   fringe       159,255,223   clean

**Run it after a CLI upgrade, not only after an artwork change.** That is the whole lesson of the section above: the generator improves, and the committed output does not follow until somebody re-runs it. Nothing reports that the files are stale.

**The script does the macOS step too**, so there is no way to regenerate the shared tree and leave the macOS icon behind. The arrangement below says how that is kept apart.

## Two defects, two different fates

**They are unrelated, and it matters not to treat them as one thing.**

    defect                what it is                          does regenerating fix it
    grey fringe           a Tauri bug, since fixed upstream    yes — verified above
    oversized on macOS    a missing feature, on every version  no, and it never will

**The fringe: regenerate, and keep using the tool.** `tauri icon` at 2.11.4 produces clean edges from the same source, on all three platforms. This costs one command and nothing else. Abandoning the generator over this would be fixing a bug that is already fixed.

**The size: no version of `tauri icon` will ever do this, so the macOS icon needs a source of its own.** The tool has no notion of a platform safe area for macOS — the option does not exist, and the only safe-area setting in the whole of it is `android_fg_scale`. Regenerating with the newest CLI still produces a body at 100% of the canvas, measured.

**Which answered the question: the automatic icon maker stays, it just stops owning one file.** Everything is still generated; the macOS `.icns` is generated from different artwork and written somewhere the shared run cannot reach. That is one extra source file, not a workflow abandoned.

**What is not in question is the disc.** The design section settles that. macOS asks nothing of an icon's *shape* — TextEdit's own icon is a page, 206 wide and 241 tall, not a squircle at all. The grid is a rule about size, not about style, so fuji can sit on it while staying exactly as plain as it was drawn to be.

## What everybody else does

The question is not academic: every cross-platform team with a Mac developer meets this, and they have all solved it. Measured from the applications installed on this machine, solid body against canvas, largest layer that exports cleanly:

    app                     canvas   body    % wide
    Cursor                    256     206     80.5%
    Discord                   256     206     80.5%
    Figma                     256     206     80.5%
    Obsidian                  256     206     80.5%
    Slack                     256     206     80.5%
    Spotify                   256     206     80.5%
    Telegram                  256     206     80.5%
    Visual Studio Code        256     206     80.5%
    Calculator (apple)        256     206     80.5%
    Cyberduck                 256     204     79.7%
    Signal                    256     204     79.7%
    iTerm                     256     204     79.7%
    balenaEtcher              256     204     79.7%
    fuji                      256     256    100.0%

**Fourteen applications, one answer, and fuji is the only outlier.** Most of that list is Electron — the same cross-platform problem, the same tooling gap, and every one of those teams supplies a macOS icon drawn to the grid rather than generated from a shared source. There is no clever alternative in the field. The mainstream way *is* the hand-made `.icns`. (Fuji is no longer at 100%, and no longer at 80.5% either — the circle section below says why a round icon is sized against Apple’s round icons instead.)

**Tauri's maintainers reached the same conclusion the hard way.** In [discussion #10999](https://github.com/tauri-apps/tauri/discussions/10999), FabianLars calls it "one of the cross-platform problems", confirms macOS needs the padding inside the file, and reports attempting to automate it in PR #11037 — abandoned, because generating a padded macOS icon from one shared source came out "uncanny" at different scales. The conclusion offered was that developers should provide multiple input icons, and that remains unimplemented. So there is no Tauri feature coming to rescue this, and no configuration to wait for.

**And padding the single source would be the wrong fix, because Windows wants the opposite.** Microsoft's guidance has app icons filling roughly 90% or more of their canvas, and artwork at 78.9% has been reported as a bug for looking *too small* beside its neighbours. The number macOS requires is very close to the number Windows treats as a defect. One source padded to please the Dock would shrink the icon in the taskbar, which is fuji's present complaint reproduced on the other platform in reverse.

## A circle is not a squircle, and the grid is a squircle's

The 824 grid settled the blimp, and immediately raised a question the blimp had been hiding: fuji sat exactly on the grid and still looked small. That was not a misreading. Measured 2026-09-09.

**Two shapes filling the same box are not the same size.** Counting the opaque pixels in the 1024 layer of every installed application whose artwork fills the mask:

    Karabiner-Elements     649,638
    DiffusionBee           648,366
    Path Finder            648,314
    Discord                648,233
    Slack                  647,779
    Spotify                647,704
    balenaEtcher           647,264

Eight applications spanning 0.37% end to end, which is what makes this a property of Apple's shape rather than of anyone's artwork. Median 648,300. A circle inscribed in the same 824 box carries π × 412², which is 533,267 — **17.7% less**. On the grid, fuji matched its neighbours on width and height while giving up a sixth of its weight, and a sixth is visible.

**Apple's own circular icons are not on the grid.** Three survive on macOS 15.7.4, never redrawn for Big Sur:

    Rosetta 2 Updater      848     82.8% of canvas
    Paired Devices         892     87.1%
    AddPrinter             903     88.2%

Against which every application Apple *did* redraw measures 206 × 206 on 256, to the pixel — Finder, Terminal, Preview, Grapher, Siri, Control Center, Keychain Access, Screenshot, and about twenty-five more. The grid is followed exactly where the shape is a squircle and departed from where it is a circle, by the same designers on the same system. That is the argument for going past it, and it is Apple's rather than fuji's.

**The measuring vocabulary: invasion.** The grid leaves a 100-pixel frame on each side. Invasion is how much of that frame the disc takes, and because the frame is exactly 100 pixels it doubles as a percentage. The radius is `412 + invasion`, so the vocabulary converts straight into the file.

    invasion   diameter   what sits there
    0          824        the squircle grid
    12         848        Rosetta 2 Updater
    34         892        Paired Devices          <- fuji
    40         904        AddPrinter
    42         908        equal area with the squircle
    50         924        halfway to the canvas
    100        1024       the full canvas, the blimp

**Built and looked at, rather than calculated.** Each candidate went through `pnpm icons` and `pnpm build` whole, and was judged in the dmg window, in /Applications at several icon sizes, and in the dock beside real neighbours:

    50   owns its space, but blimpy — reads as never having understood the safe area
    42   still reads as a mistake, as though the safe area had been aimed at and missed
    34   present among the squircles, neither diminutive nor intruding

Worth recording that 42 lost. It is the diameter at which a circle has equal area with the 824 squircle, it lands two pixels from AddPrinter, and it was the number the arithmetic argued hardest for — and in the dock it read as an error rather than a decision. The geometry located the neighbourhood and the eye picked the house.

**Fuji sits at 34, which is Paired Devices exactly.** That is the line to keep if this ever needs defending. Fuji's disc is not a number somebody liked: it is the diameter of a circular icon Apple ships on this operating system, and it sits between Apple's other two.

## How fuji's icons are built now

Applied 2026-09-07, and revised 2026-09-09 when the disc grew off the grid. This is the current arrangement; everything above is why it is this one.

**Three sources, because three places want different things.** The third was added 2026-09-11 for the Windows Start menu tile, and the section on that tile says why.

    src-tauri/icons/app-icon.svg        r="512"   the disc filling its canvas
    src-tauri/icons/app-icon-mac.svg    r="446"   the same disc drawn for the dock
    src-tauri/icons/app-icon-tile.svg   r="338"   and drawn for the Start menu tile

That is the entire difference between them: one number each. `structure.md`'s rule about a family sharing a leading noun is why they are `app-icon-mac` and `app-icon-tile` rather than `mac-app-icon` and `tile-app-icon`.

**Three destinations, so none can overwrite another, and a scratch tree behind each of the extra runs.**

    src-tauri/icons/          everything tauri icon generates from app-icon.svg
    src-tauri/icons/mac/      icon.icns alone, generated from app-icon-mac.svg
    src-tauri/icons/tile/     two logos, generated from app-icon-tile.svg, beside the manifest that names them
    src-tauri/icons/.mac/     what the app-icon-mac.svg run leaves behind, gitignored
    src-tauri/icons/.tile/    what the app-icon-tile.svg run leaves behind, gitignored

Each extra run generates a whole tree and only one or two files are wanted out of it, so the run is pointed at a dotted scratch directory and the wanted files are copied into the plain one beside it. `tauri icon` owns the first directory completely and never writes into `mac/` or `tile/`, which is what makes both fixes permanent rather than something to remember. The full-bleed `icons/icon.icns` is still generated and is simply not used by anything.

**One line of configuration selects it.** `bundle.icon` in `tauri.conf.json` lists `icons/mac/icon.icns` where it used to list `icons/icon.icns`. There is no per-platform icon setting in Tauri — the array is picked over by file extension, one `.icns` for macOS and one `.ico` for Windows — so pointing that one entry somewhere else *is* the per-platform mechanism, used as intended rather than worked around.

**One command rebuilds all of it.**

    pnpm icons

runs `tauri icon` twice and copies the second run's `icon.icns` into `mac/`. The copy is `node -e` rather than `cp` because this repository is built on Windows too. Run it after changing artwork, and after a Tauri CLI upgrade — the section above says why the second case is the one that gets forgotten.

**Why the macOS icon is generated rather than hand-drawn.** Every other application solves this with a hand-made `.icns`, and fuji does not have to, because its artwork is arithmetic: the macOS disc is a radius of 446 instead of 512 and nothing else changes. Feeding a correctly padded source to `tauri icon` gets a correctly padded icon out — the tool was never wrong about resizing, only about padding, and this hands it a source that needs none. Fuji's simplicity, which made both defects visible, is also what makes this fix a one-line file rather than a binary asset to maintain by hand.

**What was measured after applying it:**

    macOS icon.icns          1024 canvas   892 body   87.1%   edge 159,255,223 clean
                              512 canvas   446 body   87.1%   edge 159,255,223 clean
                              256 canvas   222 body   86.7%   edge 159,255,223 clean
    icon.ico, all six layers            full bleed, unchanged   edge 159,255,223 clean
    linux pngs, all three               full bleed, unchanged   edge 159,255,223 clean

87.1% is Paired Devices to the pixel; the section above says why a circle is sized against Apple’s circles rather than against the squircle grid. The other platforms keep the full-bleed disc on purpose — Windows treats a macOS-sized margin as a defect, and Linux is not researched yet.

## Two oddities worth recording

**`ic14` holds the wrong size, in every version.** That entry means 512@2x and should carry a 1024 image; `tauri icon` writes a 512 into it, old CLI and new alike. `ic10` does carry a true 1024, and macOS reads that, so nothing visibly suffers. Recorded because it will look like a discovery to whoever next opens the file with a hex editor.

**`tauri icon` does not write `.icns` entries in a stable order.** Regenerating from an unchanged source produces a file of the same length, holding the same twelve entries, with every image byte-identical — shuffled. So `git status` reports `icon.icns` and `mac/icon.icns` as modified after every `pnpm icons` run whether or not the artwork changed, and a clean status there proves nothing either way. The `.ico` and the PNGs do not have this property, and did come back byte-identical.

## Windows — researched only as far as the current files

**Both `.ico` files carry the same six layers**, so this is `tauri icon`'s shape rather than anything fuji chose:

    16x16  24x24  32x32  48x48  64x64  256x256      all PNG, 32bpp

Fuji's are full bleed at every size, for the same reason as the `.icns`, and all six carry the grey fringe. The scaffold's are the bare logo at 73% of the canvas — but that is the logo's own shape, not evidence of a Windows grid.

**Windows wants the opposite of macOS, which is the one finding so far.** Microsoft's guidance puts app icon artwork at roughly 90% or more of its canvas, and an application whose artwork filled 78.9% has been reported as a bug for appearing smaller than its neighbours in the taskbar. So the macOS number is close to the Windows defect threshold, and the two files must not share a margin. Tauri's own template agrees by example: its `.ico` is a different composition from its `.icns`, not a copy.

**What has not been researched** is whether fuji's full-bleed disc is already right for Windows or merely close, how Windows 11's rounded taskbar treatment interacts with a circle, and whether the 16 and 24 layers need their own artwork rather than a downscale. Nothing should change here until it has been looked at on a real Windows machine.

`fuji.exe` is the binary both installers wrap; the `.ico` reaches the taskbar, explorer, and the two installers.

## The Windows Start menu tile

Researched and built on the Windows 10 box, 2026-09-11. A pinned Start menu entry can be small or medium, and by default a medium tile is the app's icon sitting small in the middle of a larger square, on a background colour Windows picks out of that icon. An application can replace both.

**The mechanism is one file, and it is old and stable.** Windows reads `<exename>.VisualElementsManifest.xml` from the directory holding the executable — so `fuji.VisualElementsManifest.xml` beside `fuji.exe`. It has been there since Windows 8.1 and Microsoft's page for it is now in the archived documentation, which is worth knowing: this is a finished feature, neither growing nor being removed from Windows 10.

**Three applications, three looks, one mechanism.** Firefox shows its logo comfortably inset; Dropbox shows a white logo on solid blue; Minecraft fills the whole tile with artwork. Those are not three techniques. All three ship the same file, and what differs is only the content of the PNG and the value of `BackgroundColor` — Dropbox's blue *is* the background showing through a transparent logo. Any of the three costs the same to build, which is what makes this a choice about how fuji should look rather than about what is achievable.

**The schema, and its one trap.** `BackgroundColor`, `ForegroundText` and `ShowNameOnSquare150x150Logo` are all required, `ForegroundText` even when the name is switched off. The two logo attributes are optional — but **all or nothing: name one without the other and the whole file is silently ignored**, and the tile falls back to the default. Every failure is silent that way, including malformed XML and an image that cannot be found. The diagnosis channel is Event Viewer, event 28032 under `Applications and Services Logs\Microsoft\Windows\Shell-Core\Operational`, which records the manifest's path and an HRESULT. Images must be at most 1024 × 1024 and 200 KB, in PNG, JPEG or GIF. The `Resources.pri` machinery the documentation goes on to describe is only for per-DPI, high-contrast and localised variants, and is skippable.

**Windows 11 ignores it, which is the good outcome.** Live tiles were removed there and Start is a grid of icons, so the manifest is inert rather than broken — it cannot make Windows 11 worse. Stated at the strength it was established: that tiles are gone from Windows 11 is documented, that the manifest is then harmlessly ignored is inference from the feature's absence, not a source.

**Getting the file beside the exe turned out to be free.** This was expected to be the expensive part, because `bundle.resources` sounds like it implies a `resources/` subfolder and the manifest must be a sibling of the executable. It does not. Tauri's `resource_dir()` on Windows "resolves to the directory that contains the main executable", and `bundle.resources` accepts a map whose values are destinations relative to that directory — so a bare filename lands exactly where Windows looks. Measured rather than trusted: after a build, all three files sit in `target/release/` beside `fuji.exe`, and no `resources/` directory is created. **No custom NSIS or WiX template is involved**, which was the outcome worth checking before anything else.

**How fuji's tile is made.** The disc is arithmetic here as everywhere else, so the tile gets a third source in the same shape as the macOS one:

    src-tauri/icons/app-icon.svg        r="512"   the disc filling its canvas
    src-tauri/icons/app-icon-mac.svg    r="446"   the same disc drawn for the dock
    src-tauri/icons/app-icon-tile.svg   r="338"   and drawn for the Start menu tile

`pnpm icons` runs `tauri icon` over that third source into the gitignored `icons/.tile`, then copies two of the Store logos out under names that do not claim a size, since the manifest attribute names the tile and the file is just an asset:

    icons/.tile/Square284x284Logo.png  →  icons/tile/tile-medium.png    Square150x150Logo
    icons/.tile/Square142x142Logo.png  →  icons/tile/tile-small.png     Square70x70Logo

The larger sources are taken deliberately: Windows scales whatever it is given, and the documentation notes that scaling down gives a better result than scaling up. Measured by alpha bounding box, both come out **66.2% of their canvas**, against the stock `Square150x150Logo.png` at 100% — which is the whole reason the existing Store assets could not simply be reused. They are full bleed, so pointing the manifest at them would have produced Minecraft's look rather than Firefox's.

`BackgroundColor` is `#000000`. The intent was no branded colour at all, and the schema does not allow that — the attribute is required and there is no "leave it to Windows" value — so black is the nearest thing: it is the sheet's own colour, and it reads as an absence of decoration rather than as a brand statement.

**Seen, and it works.** The user looked at the medium tile on this machine the same day and judged it right at the first radius — a cyan disc at two-thirds of a black square, the name beneath it. No adjustment was called for, so 338 stands where the macOS disc needed two attempts to reach 446.

It was looked at **without installing**, which is worth recording because it makes this cheap to re-check. Windows reads the manifest from whatever directory holds the executable, so a Start menu shortcut pointed at `target/release/fuji.exe` — where `pnpm build` has already placed the manifest and both logos — produces the real tile. Make the shortcut in `%APPDATA%\Microsoft\Windows\Start Menu\Programs`, touch its `lastwritetime`, pin it, and look.

**That the installed layout matches was checked too**, by listing the NSIS payload rather than by installing: `fuji.exe`, `fuji.VisualElementsManifest.xml`, `tile-medium.png` and `tile-small.png` all sit at the archive root together, so the installer puts them in one directory exactly as the build does.

**What remains unestablished** is only ordering. Microsoft's instructions say the manifest must be in place *before* the shortcut is created, and that an existing shortcut must be nudged — touching its `lastwritetime` — for a changed manifest to be picked up. A first install should be fine, since installers write their files before creating shortcuts, but that was inferred rather than watched. **An upgrade over an existing install may keep a stale tile**, and fixing that would need an NSIS hook, which is the machinery this was meant to avoid. The symptom would be a default tile after an upgrade, and the workaround is one `lastwritetime` touch.

## Linux — researched only as far as the current files

Fuji ships `32x32.png`, `128x128.png`, and `128x128@2x.png`, listed in `bundle.icon`. All three are full bleed, 100% of the canvas, and all three carry the grey fringe.

**Which desktops fuji targets, and what each expects, is undecided and unresearched.** The freedesktop icon theme specification is the likely authority, and whether GNOME's and KDE's differing conventions matter to a single-window application is the first question to answer.

## macOS 26 and Liquid Glass, later

Tahoe replaces `.icns` with `.icon`, a bundle of layers that Apple's Icon Composer produces, and the system renders the light, dark, clear, and tinted variants from it. Fuji develops on macOS 15.7.4, so nothing here is urgent. Tauri is tracking support in [issue #14207](https://github.com/tauri-apps/tauri/issues/14207); until that lands, an `.icns` remains what a Tauri app can ship, and Tahoe falls back to rendering it.
