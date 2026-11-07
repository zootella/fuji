# Mac to Windows

## The spike

**The Mac has been trying ways to draw FujiTree's guide lines, and FujiList's selection and focus rectangle, and the next answers need a display at 100 percent, where a css pixel, a backing pixel and a panel pixel are the same pixel.** The Mac mini is on a 5K at 2x, so it cannot show what happens when a line falls between two of those pixels.

The user's aim: lines that are not fragile across every display, ratio and pixel unit fuji lands on, that look good, and that are not a solid line.

- The spike runs on the bench, `FujiUserInterfaceSpike.vue`, which `Shell.vue` says how to open.
- The twisties are hidden for it, so the lines are the whole show; a press anywhere in a folder's own cell still opens and closes it.
- Each experiment is a block of rules in `FujiTree.vue` or `FujiList.vue`, switched on by an attribute the bench sets from a toggle in its strip, so each can be seen alone or with the others. The comment on each block says what it does.
- A magnifier among the toggles draws the whole tree larger with CSS `zoom`, so the pixels can be measured in a screenshot. It magnifies the tree alone; the list's experiments are seen at their own size. The user measures by pasting screenshots into Figma and zooming in.

## How the lines are drawn now

- **A line is a pseudo-element box in a guide cell.** The cells are squares the row's height on a side, on one grid from the tree's corner, so every line is drawn the same way in every row and at every level.
- **A line is centered by a translate, not by where its box is placed.** The box sits at the cell's middle, a whole css pixel, and a translate of half `--guide-width` moves it back. A cell an even number of pixels wide has its middle on the boundary between two pixels, so a centered line has to straddle that boundary at 1x. The hope is that the engine draws the fraction as a fraction, smoothed across the pixels on either side, rather than nudging the line onto one of them.
- **Pattern draws the lines as cutouts onto stripes.** The stripes are a gradient tile anchored to each cell's corner, with a side that divides the row's height, so they join from cell to cell as if cut from one sheet. The aim is the varied, broken texture of the classic dotted line without depending on dots one pixel across, which cannot land on whole pixels through every unit, scale and resample fuji meets.

## Is a line still centered at 1x when it is smoothed

The user's hope is that a line stays exactly centered, half of it on the pixel to the left of the cell's middle and half on the pixel to the right, even though that blurs it. What the engine actually does is not known, and rounding can come in where the box is laid out or where the translate is drawn:

- **Where the box is laid out.** Layout works in css pixels, and the engine snaps a box's edges to whole device pixels before it paints, from memory. A `--guide-width` that is not a whole number of css pixels may be rounded there, at 1x, before the translate moves anything.
- **Where the translate is drawn.** From memory, Chromium draws a translated box where it lands and smooths its edges, which is the effect that blurs text a page centers with `translate(-50%)`. If WebView2 snaps the translate as well, the line ends up on whole pixels, off center.

How a screenshot at 100 percent tells these apart, reading one cell's own pixel columns for a line down, with the middle at half of `--row-height` from the cell's left edge:

- **Centered and smoothed:** the coverage is the same on each side of the middle, and the columns add up to `--guide-width`.
- **Rounded at layout, then translated:** the coverage is lopsided, heavier on one side, and adds up to the rounded width rather than `--guide-width`.
- **Rounded at layout and at the translate:** crisp whole pixels, all on one side of the middle.
- **The line across** reads the same way, by rows instead of columns.

On the Mac at 2x the user saw a 1 css pixel line centered, but there nothing was fractional: the box, the middle and the translate all came out whole backing pixels. A width that is not a whole number of css pixels leaves the translate half a backing pixel at 2x as well, and nobody has measured that on the Mac yet either.

## The magnifier at 100 percent

**At 100 percent the magnifier makes the tree's geometry exact**, provided its factor times every measure in the tree comes out to a whole pixel, which `FujiTree.vue` sets. So a screenshot of it there is the place to check that the design is laid out as the code says, without any smoothing in the way:

- each folder's icon square directly above the line its folders inside come down from, on the same grid as the guide cells;
- every line down and line across centered in its cell;
- Pattern's stripes carrying on unbroken across row boundaries and around corners;
- Caps' round ends where a line stops, square edges where a line runs on into the next row, and elbows round outside and sharp inside.

The Snipping Tool captures backing pixels, which at the display's native resolution are its panel pixels, so its screenshots measure exactly what the panel shows.

The tree's arrow keys keep the selection in view by mixing the layout's rectangles with the tree's scroll position, and CSS `zoom` changed how those relate in 2024, from memory. If the selection drifts out of view while the tree is magnified, that is the zoom, and it is no concern for the spike.

## The list

- **The selection is a rounded fill, inset from the list's sides, one block for each run of selected rows, with a score between the rows of a run.** The score is the fill's own color a fixed step lighter in OKLCH lightness in One Light and darker in One Dark, through CSS relative color syntax, which has never run in WebView2 here. Where it fails, the browser drops the rule and the run simply has no score. Look at all four cases, light and dark, with the keyboard in the list and without: the score should be faint and even in each.
- **The focus rectangle's experiment draws its line from black and white stripes at 45 degrees, through `border-image`, which leaves it square.** The stripes start a quarter css pixel along, so no edge between them runs through a pixel's center at 1x or 2x, where the engine would have to pick one color. On the Mac mini at 2x the user found them crisp black and white in a zoomed screenshot, with no smoothing. At 100 percent, look at whether every dash along every edge is the same whole number of pixels, in black and white with no gray, and at whether they carry on around the corners. How long a dash is along an edge is set in the experiment's rule.

## At 125, 150 and 175 percent

- **Every row should look identical.** The row's height is a multiple of 4 css pixels, so a row and a cell are a whole number of backing pixels at every 25 percent step, and whatever one cell draws, every cell should draw the same.
- **The stripe tile may fall between backing pixels even where a row does not.** A tile that divides the row in css pixels need not be a whole number of backing pixels at these scales. So look at whether the dashes are the same in every row and at every level, and at how WebView2 draws a hard color stop that falls between two pixels.
- **The lines' edges will be soft** at these scales however they are placed; what matters is that they are soft the same way everywhere.
- **The focus rectangle's dashes cannot all be one length** where a css pixel is a fraction of a backing pixel. Without smoothing they should alternate between two whole lengths; where WebView2 smooths a hard stop, some of their pixels come out gray.
