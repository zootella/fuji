# From the Mac to Windows: the dark face, the Mac's scrollbar, and the Mac's sort

Written on the Mac mini, macOS 27.2, 2026-11-20. Read it, then talk to the user before changing anything.

## The dark face is Finder's now

`--color-background` in `desktop/src/style.css` is `#222222` in dark, gray-825, where it was Zed's `#2c2c2c`. The user chose it for these reasons:

- **Both platforms are darker than Zed.** Finder's list in dark is `#212121`, in list view and icon view alike, and Explorer's is `#191919`, as measured on Windows. Beside either one, `#2c2c2c` read as charcoal.
- **Zed has nothing darker to give.** One Dark's darkest background is the editor's, `#282c33`, which is where `#2c2c2c` came from. Every other background in One Dark comes out lighter, at `#343434` with the tint taken out.
- **gray-825 is on the ladder, a step from Finder's list.** It stays nine steps lighter than Explorer's, which is the part only Windows can show.

What only Windows can say: Fuji beside File Explorer in dark, with the user, and what they say on looking. Only the face moved. The other dark grays, the border, the panels, the quieter texts and a selection's grays, were each set for their distance from `#2c2c2c` and still are, so on the darker face they stand further out than they were made to. Whether they follow it down is the user's call on seeing them.

Measured on the Mac mini from window screenshots converted to sRGB, for the comparison:

- **Light.** Finder's list, toolbar and title bar are `#ffffff`, and its sidebar `#ebebeb` to `#efefef`, moving with the wallpaper behind it. The title bar macOS draws over Fuji is `#ffffff`, over Fuji's `#fcfcfc` bar.
- **Dark.** Finder's sidebar is `#282828` to `#2b2b2b`. The title bar macOS draws over Fuji is `#282828`, so Fuji's bar now steps six down from it, where it stepped four up before.

## The Mac's scrollbar

Nothing changes on Windows. The user chose no track on any platform, only Fuji's gray thumb, which is the line in `desktop/src/style.css` as it stands. On the Mac a transparent track leaves nothing: in dark, every pixel above and below the thumb measured exactly the face, `#222222`. Finder draws a distinct track of its own, and Fuji leaves it out. The stylesheet's paragraph on scrollbars says what the Mac's engine does with the two colors.

## The Mac's sort

- **`desktop/sort/mac-sort.txt` is written**, by `pnpm sort-test record` on the Mac mini, in English with no list sort order set. `pnpm sort-test` on Windows now holds `MacSort.js` against what the Mac said about every pair of the names.
- **The Windows record passes on the Mac**, on every pair outside the group Windows contradicts itself about, so `WindowsSort.js` depends on nothing of the machine it runs on.
- **`MacSort.js` agrees with the Mac** on every recorded pair, and on 400,000 random pairs in each mix the Mac is asked about from node, after changes where it disagreed. Each change was traced through CoreFoundation's source and is written in the code. The long numbers agree too, so the Mac that ships still wraps at 64 bits.
- **Finder itself agrees.** Read back from a Finder window in list view sorted by name, every folder `pnpm sort-test folder` makes came out in `MacSort.js`'s order, and so did Finder's own scripted sort by name. `pnpm sort-test finder` and the folder's path reruns the window half, as `explorer` does on Windows.
