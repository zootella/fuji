# To the Mac from Windows

Read this, then talk with the user before changing anything; a letter starts a conversation rather than settling one. Each section is one area, and goes once its area is settled.

## Measuring the faces and the row on the Mac

**Fuji's labels are going to mix small capitals and the multiplication sign into a line of text, like 2026NOV05 and 640×480, and `faces.md` is the sprintdoc for it.** Its table says which faces draw their own small capitals and their own ×, read from the font files. Windows and Linux are filled in, and so is Inter, which is the same file everywhere. The two Mac rows say to check, since SF and the Mac's Verdana are files only a Mac has.

**To run**, from any folder, with uv, which the Mac that builds the installer already has:

```sh
uv run --no-project --with fonttools python - <<'EOF'
from fontTools.ttLib import TTFont
for path in ['/System/Library/Fonts/SFNS.ttf', '/System/Library/Fonts/Supplemental/Verdana.ttf']:
    font = TTFont(path)
    tags = {r.FeatureTag for r in font['GSUB'].table.FeatureList.FeatureRecord} if 'GSUB' in font else set()
    try: aat = sorted({n.FeatureType for n in font['feat'].table.FeatureNames.FeatureName})
    except Exception as e: aat = repr(e)
    print(path, font['head'].fontRevision, 'smcp', 'smcp' in tags, 'c2sc', 'c2sc' in tags, 'aat types', aat, 'times', 0xd7 in font.getBestCmap())
EOF
```

It prints, for each file, whether its OpenType tables have small capitals from lowercase, `smcp`, and from capitals, `c2sc`, which `.mySmallCaps` asks for. It also prints the feature types of Apple's own tables, where 37 and 38 are the same two, and whether the face maps the multiplication sign. A path that has moved in this macOS is worth finding rather than skipping. Fill in the two Mac rows in `faces.md` and delete its section The Mac's two faces.

**To compile and look at:** `cd desktop` and `pnpm local`, then `u` from the sheet for the bench. The list opens with six examples in parentheses, three dates and three sizes. Switch between the three faces in the settings panel. In each face, the months and AM and PM should be small capitals beside full-height digits, and each size should have a multiplication sign, not an x.

**What the user measured on Windows at 100 percent, for the user to measure on the Retina screen**, where each css pixel is two backing pixels:

1. **The row:** a row is 20 css pixels, set by `--row-height` in `style.css`, and the green square in the list is a 16 css pixel placeholder for the icon, with 2 above it, 2 below it, and 4 to the text. Expect 40, 32, 4, 4 and 8 backing pixels.
2. **The line:** the yellow behind each cell's text is the box the text is laid out in, the root's line, 1rlh. It was 16 css pixels on Windows, from Segoe UI at 12. On the Mac it is SF at 13's own line, so it may not be 16, and whatever it is goes in `faces.md`.
3. **The letters in the line:** inside the yellow, the rows above the capitals, the capitals, the lowercase, and below the baseline, in each face. `faces.md` has Windows' numbers in a table to set the Mac's beside.

Put the Mac's numbers in `faces.md` beside Windows', and delete this section once they are there.
