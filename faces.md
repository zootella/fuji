# Faces

## What this is for

**Fuji's own labels are going to mix small capitals and the multiplication sign into a line of text**, a date like 2026NOV05 and a size like 640×480. The faces setting offers three choices, and each one is a different face on each platform, so whether those two come out right depends on which face is drawing them. This sprintdoc records which faces draw their own and which do not, read from the font files, and what is still to check.

**The ability is built, and the labels that use it are not.** `.mySmallCaps` in `style.css` is the one rule. A Vue template puts it on a span, and a list's say function returns runs that `_listPut` in `list.js` turns into text nodes and small spans, so no text is ever parsed as markup. The bench's list opens with six examples, `examples` in `FujiUserInterfaceSpike.vue`: three dates with the month and the hour's AM or PM in small capitals, and three sizes with the multiplication sign. Which labels take small capitals, and in what form, is still to design: `sayDayTime` still says a date as 2026-Oct-06 2:45p.

## How the engine draws them

**Small capitals are `font-variant-caps: all-small-caps`**, which asks the face for its capitals-to-small-capitals feature, `c2sc`, and its lowercase one, `smcp`, and leaves digits full height. The text stays in capitals, so copying and searching get NOV. A face that has the features draws its own small capitals, made for their size, with strokes as heavy as the letters beside them. For a face that has none, the engine draws its capitals at 70 percent of the size, which come out lighter than the letters beside them; the 0.7 is in Chromium's `simple_font_data.cc` and in WebKit's `Font.cpp`, so all three platforms fake them the same way.

**The multiplication sign is the character ×, U+00D7**, never the letter x. A face without it would borrow it from another face one character at a time, as `style.css` says of any character a face lacks, so it can never show as a box.

**Each face places its letters in the one line its own way, and fuji leaves that alone.** The line is the root's, 1rlh, so changing the face never moves a box. Measured on the Windows box at 100 percent on 2026-11-05, in css pixels, inside the 16 of the line, which the 20 of the row centers:

| | system, Segoe UI 12 | bundled, Inter 13 | retro, Verdana 12 |
|---|---|---|---|
| above the capitals | 4 | 3 | 3 |
| capitals and tall letters | 9 | 10 | 9 |
| of those, above the lowercase | 3 | 3 | 2 |
| lowercase | 6 | 7 | 7 |
| below the baseline | 3 | 3 | 4 |

The capitals center on the 16 css pixel icon within half a pixel in all three, which is as close as an odd height gets to an even one. The lowercase sits lower, by the room the line keeps for descenders, which is where a Windows list puts Segoe UI too. Verdana's baseline is a pixel higher than the other two.

**From the files, at the same sizes, in css pixels:** Segoe UI's digits are 8.4 to 8.6 tall, its own small capitals 6.3 to 6.4, and its lowercase 6.0. Verdana's are 8.7 to 8.9, 6.5 to 6.7, and 6.5. Inter's capitals are 9.5, its faked small capitals 6.6, a little short of its lowercase at 7.1. Segoe UI's multiplication sign sits from 1.0 to 6.0 above the baseline and Verdana's from 0.6 to 7.0, each about halfway between the middle of the lowercase and the middle of the digits beside it, half a pixel above the one and half a pixel to 0.7 below the other.

## Which faces draw their own

**Read from each file's substitution features and character map**: Windows 10's own fonts on the Windows box, fuji's bundled Inter, and each Linux face as its maker or its distribution's package ships it. Which face each Linux desktop names comes from release notes, not from running machines. WebKitGTK answers `system-ui` with the desktop's own font setting, `gtk-font-name`, which is in WebKit's `FontCacheFreeType.cpp`. *Own* is the face's own glyphs, and *faked* is the engine's capitals at 70 percent.

| platform | choice | face | small capitals | × |
|---|---|---|---|---|
| Windows 10 | system | Segoe UI 5.62 | own | own |
| | bundled | Inter 4.1 | faked | own |
| | retro | Verdana 5.33 | own | own |
| Mac | system | SF | to check | to check |
| | bundled | Inter 4.1 | faked | own |
| | retro | Verdana, the Mac's own copy | to check | to check |
| Linux | system, Debian 12 GNOME and Fedora 40 and 41 | Cantarell 0.303 | faked | own |
| | system, Fedora 42 and later, GNOME 48 | Adwaita Sans, made from Inter | faked | own |
| | system, Ubuntu 24.04 | Ubuntu Sans | own | own |
| | system, Mint 22 | Ubuntu 0.869 | own | own |
| | system, KDE Plasma | Noto Sans | own | own |
| | system, Raspberry Pi OS Bookworm | Piboto Light | from lowercase only, so faked from capitals | own |
| | system, Raspberry Pi OS Trixie | Nunito Sans | faked | own |
| | bundled | Inter 4.1 | faked | own |
| | retro, with Microsoft's core fonts installed | their Verdana, from the 1990s | faked | own |
| | retro, without them | the desktop's sans-serif, DejaVu Sans or Noto Sans | DejaVu faked, Noto own | own |

**So the multiplication sign is every face's own, everywhere checked, and small capitals are faked in about half the faces**, Inter's on every platform among them. Retro on Linux is Verdana only where the user installed Microsoft's core fonts.

## The Mac's two faces

**SF and the Mac's Verdana are files only a Mac has.** `win2mac.md` has the command that reads them. Fill in the table's two Mac rows from what it prints, then delete this section.

## Looking at them

**On each machine, in each of the three faces, the bench's examples at the top of the list**: the months and AM and PM in small capitals beside full-height digits, and the multiplication sign in the sizes. Whether faked small capitals look good enough, in Inter everywhere and in the Linux faces that fake them, is the user's call once they are on screen.

## Windows 11

**Windows 11 ships later versions of Segoe UI and Verdana than the Windows box has**, unread. The same check against a Windows 11 machine's `segoeui.ttf` and `verdana.ttf` fills in two more rows, or finds them the same as Windows 10's.

## Linux, on a running desktop

**The Linux rows are from the files and the release notes**, so a running desktop is still to confirm them. `fc-match sans-serif` names the face retro falls back to, the desktop's font setting names the face system is, and the bench in WebKitGTK shows both, in the `.deb` and in the Flatpak, whose GNOME runtime brings fonts of its own.
