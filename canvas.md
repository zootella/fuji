# Canvas

What the two engines fuji ships on actually do with a `<canvas>`: when one is backed by the GPU, when it becomes a compositing layer of its own, what its memory costs and why that memory can never come back, how a picture is scaled into it and what makes the result look right, and what colors it can hold. It ends with where a thumbnail should be made at all — in the web layer or by the operating system — and the direction fuji has taken on that. `CanvasFlow.vue` is the code all of this governs, and `card.md` is where the flows are weighed against each other; this file is the engine facts underneath both, so a choice there can cite a rule rather than a guess.

The Mac runs WebKit inside a WKWebView, and Windows runs Chromium inside WebView2. Those are the two that have to be excellent. Linux runs WebKitGTK and appears below only where its answer decides whether code that must not break there can be written at all. Everything here was read from the engines' sources at their main branches on 2026-09-08, and the files are listed at the end. None of it has been measured in fuji: a number in this file is what the engine says it does, and `performance.md` is where a number goes once fuji has seen it happen.

## What a canvas costs

**A canvas holds one raster at its backing size, and the engine can never take it back.** An `img`'s decoded pixels can be dropped under memory pressure and rebuilt from the bytes; a canvas has nothing to rebuild from, so its backing store lives exactly as long as the element does. That is the whole trade CanvasFlow makes: a small, known, permanent cost in place of a large, reclaimable, invisible one. The cost is the css width times the css height, times the pixel ratio squared, times four bytes, whatever the context's settings. At a pixel ratio of 2, for a 3:2 photograph:

    size          one thumbnail    500 of them
    Small  120    150 KB            77 MB
    Medium 240    600 KB           307 MB
    Large  360    1.35 MB          691 MB
    Xl     480    2.4 MB           1.23 GB

On Windows at 150% the same numbers are a little over half of these.

**Where that memory lives depends on whether the canvas is accelerated, below.** GPU-backed, it is a texture or an IOSurface; CPU-backed, it is ordinary process memory. It is the same number of bytes either way, and neither kind is duplicated for compositing: a layer's contents are the backing store itself.

**There is a cap per canvas and it does not matter here.** WebKit refuses a canvas whose area exceeds 16384 by 16384 on the Mac and logs a warning; a thumbnail is a thousand times smaller. The page-wide canvas memory limit that older WebKit enforced is gone from the current source.

**What an `img` costs instead is the engine's decision, and both engines decode a large picture small when they know it will be painted small.** Chromium decodes a JPEG at a half, a quarter or an eighth of its size for a small paint, so an `img` of a 26-megapixel photograph shown at 240 css pixels may hold under two megabytes there, and its `createImageBitmap` honours a `resizeWidth`. WebKit on the Mac subsamples too, by a rule of its own: image subsampling is on by default on Apple platforms, the level for a draw is the power of two nearest the reduction, and the deepest level allowed is the first at which the frame falls under five megapixels — so a 26-megapixel file is decoded at a quarter, 1560 by 1040, and a 12-megapixel one at a half. Its `createImageBitmap` ignores a resize, which fuji measured. Two things follow. An `img` thumbnail is far smaller than a full decode on both platforms, so the memory case for the canvas is the permanence and countability of its raster, not a difference in size. And a full-size decode taken earlier defeats all of it, which is the story of the first thumbnails, below.

## GPU or CPU

**WebKit backs every 2d canvas with an IOSurface, at any size.** The decision is `CanvasBase::shouldAccelerate`: accelerated canvas drawing is on by default in WebKit, and the minimum area below which a canvas stays in software is zero on Apple platforms — 128 by 129 only in the Skia builds used elsewhere. A Small thumbnail and a full-window canvas are made the same way.

**Chromium accelerates a canvas of 128 by 129 pixels or more when a GPU is available.** Below 16,512 pixels of area a canvas is drawn in software. A Small thumbnail at 100% is 9,600 to 14,400 pixels and stays on the CPU; at 150% it is over 21,000 and goes to the GPU, as does every larger size at any scale. Chromium also demotes to software a canvas that keeps reading its own pixels back, and hibernates the GPU copies of accelerated canvases while the page is hidden, moving them to the CPU and back.

**`willReadFrequently: true` forces the CPU on both.** WebKit takes the unaccelerated path whenever it is set, and Chromium does the same. It is one attribute on `getContext`, and it is the honest way to run the painted-not-composited arm of the experiment: no surface, no texture, no layer, and a software resample in `drawImage`.

**Drawing into an accelerated canvas has a cost of its own.** On Chromium, `drawImage` of a large `img` into a GPU canvas uploads the source — the whole decode, a hundred megabytes for a 26-megapixel photograph — as a texture before it can scale it. On the Mac, CoreGraphics draws into the IOSurface directly. Either way `drawImage` is synchronous work on the main thread, tens of milliseconds for a large source, and the decode that precedes it is mostly not: both engines decode an `img` off the main thread. The draw is the part that lands in a frame.

## A layer of its own

**On the Mac, whether an accelerated canvas gets a compositing layer of its own turns on one rendering mode, and it is likely on.** `canvasCompositingStrategy` in `RenderLayerBacking.cpp` gives an accelerated 2d canvas its own layer when the page's accelerated drawing is off — it is on by default — or when dynamic content scaling display lists are in use for DOM rendering. That preference, `UseCGDisplayListsForDOMRendering`, defaults to true wherever the feature is compiled in, which is Apple's own builds on recent macOS. `RenderLayerCompositor::requiresCompositingForCanvas` then composites it if its area is at least 50 by 100 pixels, which every thumbnail size clears. So the likely answer is one layer per thumbnail, and the certain answer is a minute away: Safari, the Develop menu, this Mac, Fuji, the Layers tab, against a debug build.

**On Windows, every accelerated canvas is its own compositor layer.** `CanvasRenderingContext2D::IsComposited` is true whenever the canvas has a GPU-backed provider and accelerated compositing is on, and a composited canvas is a texture layer in the compositor. A software canvas is painted into the page's own tiles, as an image is.

**What a layer costs is compositor work, not pixels.** Each one is a quad the compositor places every frame and an entry in its tree; hundreds are routine for both compositors and thousands begin to tell. The number that matters is the frame time while scrolling a full sheet, measured the way `performance.md` measures a flip.

## Scaling: what makes a thumbnail look right

**A one-pass draw of a large picture into a small canvas comes out rough on the Mac, and the fix is to halve.** Fuji's first thumbnails aliased — the roof tiles of a 26-megapixel photograph turned to jaggies at 240 pixels — while Safari showed the same file smooth as an `img`, and the difference is two things. First, `imageSmoothingQuality` was set to `high`, but CoreGraphics' high interpolation reads a fixed footprint of source pixels around each output pixel; at 26 to 1 that leaves most of the picture unread, and unread pixels alias. Chromium's `high` is a chain of successive halvings sampled with a cubic filter, which does read everything, and is why the same one-pass draw is fine on Windows. Second, WebKit hands `drawImage` a subsampled frame only when it has to decode one. `decode()` on an element asks for the full-size frame, and a cached frame counts as compatible with any request for a smaller level, so a `drawImage` after the store's decode gets all 26 megapixels, where Safari's `img`, which never called `decode()`, got a quarter. So `CanvasFlow` halves through scratch canvases until the last draw is within 2 to 1, the ratio at which a fixed footprint is an honest average of the source. The default quality, `low`, reads only the four nearest source pixels and would alias even at that ratio, so every step sets `high`, and `imageSmoothingEnabled` stays at its default of on.

**The cheaper cure is to let `drawImage` do the decode, and it is unmeasured.** A canvas flow that asked the store for the bytes only, pointed an element of its own at the url, waited for it to load rather than to decode, and then drew, would get exactly Safari's path: WebKit would decode at the subsampled level for that draw, through ImageIO's own scaled decode, and the full-size raster would never exist — a peak of a couple of megabytes in place of a hundred. Chromium decodes at the draw scale in its raster step the same way. The price on WebKit is that the decode then happens inside `drawImage`, synchronously on the main thread, where today's `decode()` runs off it — a few hundred milliseconds of frozen interface per large progressive JPEG, since every scan is still entropy-decoded before scaling saves anything. The direction below rules that out: the interface thread is never frozen. It stays here as the explanation of why Safari's `img` is smooth and cheap, not as a path fuji will take.

**The backing store is sized so the compositor has nothing to scale.** A canvas is painted at its css size times the pixel ratio, so at a whole-number ratio every backing pixel lands on one device pixel and nothing is resampled on the way to the screen. At 150% a css size of 181 is 271.5 backing pixels, rounded to 272, and one axis is resampled by half a pixel; an `img` at the same css size is resampled the same way, so the two arms match. CanvasFlow also never asks for more backing pixels than the file has, so a small picture is drawn at its own pixels and enlarged by the compositor exactly as an `img` would be, rather than enlarged twice.

**Orientation and animation come out right.** Both engines apply a JPEG's EXIF orientation to `naturalWidth` and `naturalHeight` and to `drawImage` from an `img`, so a portrait photograph from a phone is upright in the canvas without any code; worth one look with a real one. An animated GIF or WebP drawn into a canvas is its first frame, which is what a thumbnail should be, where two hundred `img` tags would run two hundred animations at once.

**What no setting gives is sharpening.** A photo tool often sharpens its thumbnails after the downscale. A canvas has no filter for that and neither does an `img`, so the two arms are even; it is the residual difference between either of them and a tool built for photographs.

## Color

**A canvas is sRGB unless asked otherwise, and a Display P3 photograph loses its most saturated colors on the way in.** `drawImage` converts the source into the canvas's color space first, and any color outside sRGB — which every iPhone since the 7 records — is clamped to the nearest color inside it. On a wide-gamut screen the thumbnail then looks duller than the table shows the same file, because the table's `img` is converted straight to the screen. `getContext('2d', {colorSpace: 'display-p3'})` makes a wide-gamut canvas and keeps them.

**Each platform answers that request differently, which decides how it can be asked.** Chromium accepts `display-p3` everywhere. WebKit enables canvas color spaces by default on the Mac and on Linux, but compiles the `display-p3` value only into its Apple builds: on WebKitGTK the value does not exist, and passing it throws. So CanvasFlow does not ask for a color space by name. It asks the screen, with the `color-gamut: p3` media query, and requests whichever space the screen can show. On the Mac that query is true on a wide-gamut panel and false on an ordinary monitor; on Windows it follows the display; on WebKitGTK it is always false, because `screenSupportsExtendedColor` there returns false unconditionally, so Linux is never asked for the value it lacks. An sRGB screen gets the sRGB canvas it always got, and there is no difference to see there, in precision or in speed.

## Where a thumbnail is made

**Today's path, one image at a time.** The store reads the file through Rust, wraps the bytes in a blob, makes an object url, and points an element at it. The store calls that element's `decode()`, which asks the engine for the full-size frame — all 26 megapixels for a 26-megapixel file — decoded off the main thread. `CanvasFlow` halves from that element down into the small canvas, on the main thread. Then it releases, and the store drops the element, the url and the blob, so the full raster becomes garbage. Bytes do reach a canvas, but by way of a full-size decode fuji asked for.

**The three ways to make one, weighed.** The web layer has two, and the operating system has the third.

    path                                   decode work per large JPEG   full raster exists   main thread          new rust
    today: store decode, then halving      a full decode                yes, briefly         tens of ms each      none
    a worker: the same, off the thread     a full decode                yes, in the worker   none                 none
    ImageIO on the Mac, WIC on Windows     a scaled decode              never                none                 one platform module

The worker is the web layer's answer to the main thread and nothing else: `createImageBitmap` from the blob in a worker, the same halving into an `OffscreenCanvas`, and a small bitmap handed back to a `bitmaprenderer` canvas on the page. Same work, same peak, none of it in a frame.

**The operating system's thumbnailer is the only one that changes the work.** ImageIO and WIC both decode a baseline JPEG at an eighth of its size by skipping most of the inverse transform, which is several times less work than a full decode, and baseline is what every camera and phone writes. For a progressive JPEG the gain is modest, because every scan has to be entropy-decoded before scaling helps, and the six test images are all progressive, so a test on them understates it. For PNG there is no scaled decode and no gain. What the native path gives on every format is the other two columns: the full raster never exists, and the work is off the main thread by construction, with a few hundred kilobytes crossing the bridge instead of the file.

**Why the web layer is the right home for display and only an adequate one for this.** The web has been built for images since the nineties, but for showing them: the `img` tag is where the engine's tuning lives, and `TagFlow` gets all of it. What the sheet does is make thumbnails of huge originals, which websites never do — they ship thumbnails made in advance — and which operating system thumbnailers have been good at since the early nineties. That is why the canvas ran into a fixed-footprint filter and a cached-frame rule that the `img` never meets.

**The direction, decided 2026-09-08 and revised the same evening by the measurements below.** Fuji leans on the operating system. On the Mac and on Windows, every format the operating system decodes is thumbnailed by `thumbnail.rs`, and the halving canvas flow stays as the path for everything else: the formats the operating system lacks, SVG, and all of Linux. The measurements decided it, and not only on speed. The native path is three times faster on ordinary web pictures and up to twenty-five times on camera originals, but the reason that weighs more is responsiveness: a user who waits a second for a folder while fuji stays completely responsive is far better off than one who waits the same second while fuji stutters, and the web path costs the main thread 50 to 220 milliseconds per camera original where the native path costs it nothing, because the decode runs on a pool thread and the page only receives small pixels. The interface thread is never frozen, on either path. The web engine's sandbox was the argument for keeping decoding in the page, and `security.md` says why it is not protecting fuji as things stand, which is a reason to lean on Rust rather than a reason not to. Both paths ship, and choosing between them per file is the design below.

**What the Rust is, and is not.** It is written, in `thumbnail.rs`, and nothing calls it yet. It is not a decoder: it is the plumbing to call two libraries the operating systems already ship, in a module shaped like `panel.rs`. Both libraries take a path or bytes, and the command takes a path, so the library reads the file itself and the page never holds it. On the Mac: an image source from the path, one call for a thumbnail with `kCGImageSourceThumbnailMaxPixelSize` set to the longest side, `kCGImageSourceCreateThumbnailFromImageAlways` so it renders one rather than handing back the tiny preview a camera embeds, and `kCGImageSourceCreateThumbnailWithTransform` so the file's orientation is applied; then a draw into a bitmap context in the wanted color space, sRGB or Display P3, which is where CoreGraphics does the color management, and one loop of ours to undo the premultiplied alpha that context produces. On Windows: an imaging factory, a decoder over the path, the first frame, the EXIF orientation read out of the metadata by hand because WIC does not apply it, a scaled decode through the codec's source transform where it offers one, the Fant scaler for the rest, a flip-rotator for the orientation, a color transform from the file's profile to sRGB, and a format converter to straight-alpha RGBA. Either way the page gets one buffer — eight bytes of width and height, then the pixels — and `thumbnail.js` unpacks it for `ImageData`. The Mac body is about sixty lines and the Windows body about a hundred and fifteen, comments included. `core-graphics`, `core-foundation` and `windows` were already beneath fuji; ImageIO needed five `extern` declarations. Both bodies type-check, the Mac one in place and the Windows one against the MSVC target, and neither has run against a file yet. What Windows cannot do is Display P3 — it answers in sRGB whatever is asked — and the two mirrored EXIF orientations no camera writes are expressed on WIC's documented rotate-then-flip order and untested.

**What each side decodes, verified 2026-09-08.** The assumption to check was that the web renderer is slower but safer and more capable with unusual formats. Slower is measured above. Safety is `security.md`'s subject, and the short answer is that the two are equal in effect today. More capable turns out to be true on Windows and false on the Mac.

    ImageIO on this mac, asked directly   61 types: jpeg png gif tiff bmp ico cur icns webp heic heif avif jpeg-xl jpeg-2000 psd openexr tga dds pict pbm radiance mpo, and some thirty camera raw formats. Not svg, not pdf
    WebKit on this mac                    the subset it allows through: jpeg png gif bmp ico cur tiff mpo webp avif heic heif jpeg-xl, plus svg, which it renders as a document rather than decodes. Lockdown mode cuts that to jpeg png gif webp
    Chromium, main branch                 what its own decoders sniff: jpeg png gif webp bmp ico cur avif, and jpeg-xl, on by default at main and reaching WebView2 as Edge updates. No heic, no tiff. Plus svg
    WIC, windows 10 and 11 at the factory from documentation rather than a machine: jpeg png gif bmp tiff ico jpeg-xr dds. Heic, webp, avif and camera raw arrive as store extensions, webp and avif free and often preinstalled on 11, heic paid unless the maker included it. No jpeg-xl

So on the Mac the native side is the capable one — every format WebKit shows and dozens it does not, camera raws included — and the web side adds only SVG. On Windows the web side is the capable one for the web-born formats, WebP, AVIF and JPEG XL, until the codec packs are present, and the native side has TIFF and, with the pack, HEIC, which Chromium never has. An animated GIF or WebP is a first frame on either path, which is what a thumbnail should be. On names: JFIF is plain JPEG under another extension, WebM is video and not on fuji's list, and the file nobody can open is HEIC on a Windows machine without the paid codec.

**WebP and AVIF are the two that matter most, and both are handled.** They are the formats the user meets: WebP nearly as common as JPEG on the web now, and AVIF saved from a browser and then openable by nothing else on the machine. On the Mac, `thumbnail.rs` was run against a WebP, an AVIF and a HEIC on 2026-09-08 and decoded all three, in 3, 11 and 16 milliseconds. On Windows they take the web path unless WIC reports the free extensions, and Chromium has always had both, so fuji shows them either way. ImageIO can decode both and encode neither, which is why the samples came from the web rather than from the six.

## Choosing a path for a file

**The extension gets a file into the list; the first bytes decide what it is.** Every format fuji handles announces itself in its first sixteen bytes — `FF D8 FF` for JPEG, the eight-byte PNG signature, `GIF8`, `BM`, `RIFF....WEBP`, an ISO box beginning `ftyp` with a brand of `avif` or `heic`, `FF 0A` or the box form for JPEG XL, `II*` or `MM*` for TIFF, and text beginning `<svg` or `<?xml` for SVG — and Chromium chooses its decoder by exactly this sniff. A `.jpg` whose bytes say something else is not a JPEG whatever its name, and a file whose bytes say nothing fuji knows is refused and shown as the placeholder the table already uses for a file it cannot read. That is the malicious case, and it costs sixteen bytes to catch.

**Then a table, per platform, sends the format one of three ways.**

    format                   mac                  windows                                                     linux
    jpeg png gif bmp tiff    native               native                                                      web
    webp avif jpeg-xl        native               web, or native when WIC reports a decoder for it            web
    heic heif                native               native when the pack is installed, else refused: Chromium has no heic   refused
    svg                      web, TagFlow only    web, TagFlow only                                           web
    anything else            refused              refused                                                     refused

Whether WIC has a decoder for a format is one enumeration at startup, a per-machine fact fuji reads rather than guesses.

**Then a size check from the header, before anything decodes.** Both libraries give the pixel dimensions from the header alone — an ImageIO properties call, WIC's GetSize — and a file claiming more than a ceiling, two hundred megapixels say, is refused before any decoder allocates. That is the decompression bomb, the one attack that needs no bug.

**Where each piece lives.** The sniff and the header read are I/O, so they are one Rust command that takes a path and answers with a format name and a size, or a refusal. The table is policy, so it is a plain JavaScript module the flow consults, with the platform's answer read once at startup. `thumbnail.rs` then refuses any file whose bytes are not a format it was told to expect, so a caller that skipped the probe still cannot hand it a mystery. None of this is written: `thumbnail.rs` today decodes whatever ImageIO or WIC will take, and nothing calls it.

## Measured, outside the app, 2026-09-08

On the M2 Mac mini, macOS 15.7.4, one image at a time, third run of three, thumbnail longest side 480. The native column is `thumbnail.rs` timed from a scratch binary. The web columns are `CanvasFlow`'s exact code run in a headless WKWebView: `decode` is the element's decode, `main` is what the halving cost the main thread, and `complete` is when the pixels were done in whichever process drew them. All in milliseconds.

    the six unsplash originals, 7 to 26 megapixels, progressive, 0.4 to 6.2 MB
    file       native    web decode   web main   web complete
    1red         48         971          224         260
    2orange      17          80           52          76
    3yellow      57         106           59          64
    4green       78         143           80          86
    5blue        73         123           74          80
    6purple      22         217           86         104

    the same six made into baseline web jpegs at 1600 pixels, 120 to 680 KB
    all six      6 to 9    10 to 15      1 to 2     11 to 18

    and at 1000 pixels, 57 to 270 KB
    all six      3 to 4     5 to 7       0 to 1      4 to 6

**What the numbers say.** For web-sized JPEGs, which is most of what a consumer has, the two paths are both fast and the native one is about three times faster on the clock, with the web path costing the main thread one or two milliseconds an image: a folder of two hundred fills in a few seconds either way, and nobody drops a frame. For camera originals the picture changes: native is three to twenty-five times faster, and the halving costs the main thread 50 to 220 milliseconds per image, which is a dropped frame or a dozen per picture while a folder of phone photographs fills. That main-thread cost is the intermediate canvases forcing the engine to finish each step before the next reads it; a lone draw into a canvas costs the main thread nothing, because the engine defers it. That is what turned the direction above toward the operating system. A draw column in the meter is still worth having, to see what the web path costs in the running app for the formats that stay with it.

## Seeing any of this

**Layers.** On the Mac, run the debug build with `pnpm local`, open Safari, and choose this Mac under the Develop menu, then Fuji, then the Layers tab; it lists every compositing layer with its size, its memory, and the reason it exists. On Windows the same panel is under More tools in the WebView2 developer tools.

**Memory.** Process memory from Activity Monitor or Task Manager is the only number that sees an `img`'s decode and a canvas's backing store alike; `cache.js` counts what fuji holds and cannot see either. `card.md` says why that makes a footprint readout comparing the two flows misleading.

**Frames.** A scroll is a stream of frames and a hitch is a dropped one, which `meter.js` can record the way it records a flip.

## Sources

Read at the main branch of each repository on 2026-09-08.

    WebKit    Source/WebCore/platform/graphics/BitmapImageDescriptor.cpp    subsamplingLevelForScaleFactor, maximumSubsamplingLevel and its five-megapixel rule
              Source/WebCore/platform/graphics/BitmapImageSource.cpp        decode() asking for the full-size frame
              Source/WebCore/platform/graphics/ImageFrame.cpp               a cached frame counting as compatible with any smaller level
              Source/WebCore/page/Settings.yaml                             ImageSubsamplingEnabled, on for Apple platforms
              Source/WebCore/html/CanvasBase.cpp                            shouldAccelerate, maxCanvasArea
              Source/WebCore/html/canvas/CanvasRenderingContext2DBase.cpp   willReadFrequently
              Source/WebCore/rendering/RenderLayerBacking.cpp               canvasCompositingStrategy
              Source/WebCore/rendering/RenderLayerCompositor.cpp            requiresCompositingForCanvas, the 50 by 100 threshold
              Source/WebCore/page/Settings.yaml                             MinimumAccelerated2DContextArea
              Source/WTF/Scripts/Preferences/UnifiedWebPreferences.yaml     CanvasUsesAcceleratedDrawing, AcceleratedDrawingEnabled, UseCGDisplayListsForDOMRendering, CanvasColorSpaceEnabled
              Source/WebCore/html/canvas/PredefinedColorSpace.idl           display-p3 behind ENABLE_PREDEFINED_COLOR_SPACE_DISPLAY_P3
              Source/WebCore/platform/gtk/PlatformScreenGtk.cpp             screenSupportsExtendedColor
              Source/WebCore/css/query/MediaQueryFeatures.cpp               color-gamut
    Chromium  third_party/blink/renderer/core/html/canvas/html_canvas_element.cc                   ShouldAccelerate, kMinimumAccelerated2dCanvasSize
              third_party/blink/renderer/modules/canvas/canvas2d/canvas_rendering_context_2d.cc   IsComposited, Is2DCanvasAccelerated, CcLayer

The scaling and color behaviour of `drawImage`, the reduced-size JPEG decode, and the off-thread decode are engine behaviour known from their documentation and from fuji's own experiments rather than from a line cited above; `CanvasFlow.vue` carries the createImageBitmap experiment.
