# To the Mac from Windows

Read this, then talk with the user before changing anything; a letter starts a conversation rather than settling one. Each section is one area, and goes once its area is settled.

## SVG tiles in WebKit

**The contact sheet cut every SVG's right side and every tile's top frame line, on both platforms, and the fix has run only in WebView2.** The list's unscoped style block in `FujiList.vue` held a bare `.myCell` rule, padding 8 pixels each side with `overflow: hidden`, and it landed on the sheet's cells, which are named `myCell` too: an img tile sat 8 pixels inside a cell exactly its width and lost those 8 on the right, and every tile's outline, drawn 2 pixels above its cell, lost its top. The list's cells are now `myListCell`, `style.css` has an essay on why a block that cannot be scoped uses names only its component uses, and `.myTile` says `overflow: clip` for itself. Measured on Windows from the log: every tile's box equals its cell's, left edge and width.

**To try**, on twenty SVGs in a folder of their own, each named for how it gives its size, which the binary opens straight onto the sheet when handed the folder. The fifteen made by hand draw a blue line on their own four edges, which should show whole inside fuji's frame on every one. The six that try to paint past their box, `f`, `m`, `n`, `r`, `s` and `t`, should show their pink only inside their tiles: `r` animates a circle out past its edges, and `s` holds a `foreignObject` with a fixed-position HTML block, which is where WebKit and Chromium are likeliest to differ. Every tile's frame should have its top line again.

Save each block as the file its name gives. Three are too large to carry here and come from Wikimedia instead: `i-tiger-viewbox-only.svg` is [the Ghostscript tiger](https://commons.wikimedia.org/wiki/Special:FilePath/Ghostscript_Tiger.svg), `k-svglogo-percent-size.svg` is [the SVG logo](https://commons.wikimedia.org/wiki/Special:FilePath/SVG_Logo.svg), and `l-wikipedia-px-size.svg` is [Wikipedia's logo](https://commons.wikimedia.org/wiki/Special:FilePath/Wikipedia-logo-v2.svg).

`a-viewbox-and-size.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="256" cy="256" r="240" fill="none" stroke="#e02020" stroke-width="8"/><polygon points="256,16 496,256 256,496 16,256" fill="none" stroke="#10a040" stroke-width="8"/></svg>
```

`b-size-only.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="256" cy="256" r="240" fill="none" stroke="#e02020" stroke-width="8"/><polygon points="256,16 496,256 256,496 16,256" fill="none" stroke="#10a040" stroke-width="8"/></svg>
```

`c-viewbox-only.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="256" cy="256" r="240" fill="none" stroke="#e02020" stroke-width="8"/><polygon points="256,16 496,256 256,496 16,256" fill="none" stroke="#10a040" stroke-width="8"/></svg>
```

`d-wide-size-only.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="800" height="400"><rect x="2" y="2" width="796" height="396" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="400" cy="200" r="184" fill="none" stroke="#e02020" stroke-width="8"/><polygon points="400,16 784,200 400,384 16,200" fill="none" stroke="#10a040" stroke-width="8"/></svg>
```

`e-tall-viewbox-and-size.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="400" height="800" viewBox="0 0 400 800"><rect x="2" y="2" width="396" height="796" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="200" cy="400" r="184" fill="none" stroke="#e02020" stroke-width="8"/><polygon points="200,16 384,400 200,784 16,400" fill="none" stroke="#10a040" stroke-width="8"/></svg>
```

`f-escape.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><rect x="-4000" y="-4000" width="8512" height="8512" fill="#ff00aa" fill-opacity="0.35"/><circle cx="256" cy="256" r="900" fill="none" stroke="#e02020" stroke-width="40"/><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><polygon points="256,-600 1100,256 256,1100 -600,256" fill="none" stroke="#10a040" stroke-width="16"/></svg>
```

`g-small-size-only.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect x="1" y="1" width="62" height="62" fill="none" stroke="#2060ff" stroke-width="2"/><circle cx="32" cy="32" r="28" fill="none" stroke="#e02020" stroke-width="2"/><polygon points="32,4 60,32 32,60 4,32" fill="none" stroke="#10a040" stroke-width="2"/></svg>
```

`h-japan-size-only.svg`, Japan's flag from Wikimedia
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="900" height="600"><rect fill="#fff" height="600" width="900"/><circle fill="#bc002d" cx="450" cy="300" r="180"/></svg>
```

`j-github-viewbox-only.svg`, GitHub's icon from Simple Icons 13.0.0
```svg
<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>GitHub</title><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>
```

`m-root-overflow-visible.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512" overflow="visible" style="overflow:visible"><circle cx="256" cy="256" r="700" fill="#ff00aa" fill-opacity="0.35"/><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="256" cy="256" r="240" fill="none" stroke="#e02020" stroke-width="8"/></svg>
```

`n-blur-beyond.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><defs><filter id="b" x="-500%" y="-500%" width="1100%" height="1100%"><feGaussianBlur stdDeviation="60"/></filter></defs><rect x="400" y="400" width="200" height="200" fill="#ff00aa" filter="url(#b)"/><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="256" cy="256" r="240" fill="none" stroke="#e02020" stroke-width="8"/></svg>
```

`o-huge-intrinsic.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="100000" height="100000" viewBox="0 0 512 512"><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="256" cy="256" r="240" fill="none" stroke="#e02020" stroke-width="8"/><polygon points="256,16 496,256 256,496 16,256" fill="none" stroke="#10a040" stroke-width="8"/></svg>
```

`p-viewbox-offset.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="-256 -256 512 512"><rect x="-254" y="-254" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="0" cy="0" r="240" fill="none" stroke="#e02020" stroke-width="8"/><polygon points="0,-240 240,0 0,240 -240,0" fill="none" stroke="#10a040" stroke-width="8"/></svg>
```

`q-aspect-none.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="800" height="200" viewBox="0 0 100 100" preserveAspectRatio="none"><rect x="1" y="1" width="98" height="98" fill="none" stroke="#2060ff" stroke-width="2"/><circle cx="50" cy="50" r="46" fill="none" stroke="#e02020" stroke-width="2"/><polygon points="50,4 96,50 50,96 4,50" fill="none" stroke="#10a040" stroke-width="2"/></svg>
```

`r-animated-grow.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/><circle cx="256" cy="256" r="40" fill="#ff00aa" fill-opacity="0.4"><animate attributeName="r" values="40;1200;40" dur="4s" repeatCount="indefinite"/></circle></svg>
```

`s-foreignobject.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><foreignObject x="-1000" y="-1000" width="3000" height="3000"><div xmlns="http://www.w3.org/1999/xhtml" style="width:3000px;height:3000px;background:rgba(255,0,170,0.35);position:fixed;inset:-500px"></div></foreignObject><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/></svg>
```

`t-transform-scale.svg`
```svg
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><g transform="translate(256 256) scale(4) translate(-256 -256)"><circle cx="256" cy="256" r="120" fill="#ff00aa" fill-opacity="0.35"/></g><rect x="2" y="2" width="508" height="508" fill="none" stroke="#2060ff" stroke-width="4"/></svg>
```

## A folder on the command line

**A folder handed to fuji at launch opens on the contact sheet, which works on Windows and has not been tried on the Mac.** `open_argv` in `open.rs` takes a folder as well as a file, and `Shell.vue` sends a folder down the road a folder chosen in Open… takes. The binary run with a folder argument reaches `open_argv` on the Mac too, since `RunEvent::Ready` calls it on every platform. What is untried is `open -a Fuji` with a folder, which arrives as an Apple event through `open_urls`; Launch Services may hand a folder only to an application whose Info.plist says it opens folders.

**To try**: `Fuji.app/Contents/MacOS/fuji` with a folder, then `open -a Fuji` with the same folder; each should open straight onto that folder's contact sheet.

## Large transparent PNGs in thumbnails

**The Mac's thumbnail of a picture with an alpha channel may alias.** For the fringe fix, ImageIO hands such a picture back at its own size and Core Graphics shrinks it as it draws it into the bitmap context, at `CGInterpolationQualityHigh`. That is the filter the page route's halving essay in `TestFlow.vue`, and the Image Fidelity page, say reads a fixed footprint of source pixels and aliases at large reductions, which is why the page route halves. A picture without alpha keeps ImageIO's scaled decode and is not affected.

**To try**: a large PNG with an alpha channel and fine repeating detail, one-pixel stripes or a fine checkerboard several thousand pixels across, beside the same picture saved without alpha, on the sheet. If the transparent one comes out jagged where the opaque one is smooth, the shrink in `thumbnail.rs` wants halving as the page route does, and that is the user's to decide.
