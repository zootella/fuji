# Video

Fuji will play video through libmpv. Decided 2026-09-29 on the Windows box after a research pass, and nothing is built. This document owns playback — how a video file reaches the screen and the speakers once fuji has decided to show it. What fuji claims to open is `fileTypes.js`, whose `.mp4` and `.mov` entries already carry a `videoNative` route that names libmpv.

A whiteboard: far more asked than settled, and the settled part is two paragraphs.

## What is decided

**libmpv, through Rust.** Four reasons, in the order they weighed. It is FFmpeg underneath, so it plays what VLC plays, which is everything — and for fuji that word means the video a 1990s collection actually holds, which is not H.264 in an MP4. It is AVI with Cinepak, Indeo or Microsoft Video 1 inside, MPEG-1 off a Video CD, QuickTime with Sorenson, RealVideo, and later DivX, Windows Media and Flash Video; `fileTypes.js` now lists them all, off. No webview plays any of that, and neither does QuickTime Player or Movies & TV, so libmpv is the only route that makes such a folder open at all, the way the picture entries already open a folder of Paint's BMPs. Its render API draws into a context the host owns, instead of demanding a window of its own, and mpv's own documentation now recommends that route over window embedding, especially on macOS. Its video output is the best in open source — libplacebo's `gpu-next` is the default since 0.41, with color management, HDR tone mapping and proper scalers — and that is the part of playback fuji cares about, since fuji measures whether a picture's colors arrive intact. And it is LGPL 2.1 or later when built with the lgpl option, which is what the prebuilt Windows libraries ship as, so linking it puts no license on fuji.

**Why not the others**, written once so nobody researches it twice. *libVLC*: VLC 4 was still beta and nightly on the desktop as of mid-2026 and libVLC 4 pre-release with it; the API is from the 2001 module design, and people embedding it in Tauri report it rendering over the webview rather than under it. *FFmpeg directly*: the right tool when frames are data — thumbnails, scrubbing — and it may still come in for that, but as a player it means writing audio output and A/V sync ourselves. *GStreamer*: a framework rather than a player, heavy to ship, and Windows and the Mac are its second-class platforms; it is what WebKitGTK already plays through. *The webview's video tag*: free and kept as the `videoWeb` route in the table, and the section on the two routes below says what it is for. *AVFoundation and Media Foundation*: three implementations to maintain, each playing only what its operating system plays.

**Nothing is drawn over a video.** No controls, no title, no readout, nothing that appears because the mouse moved; the Design Choices page on fuji's site says why. That is a design rule, and it also settles where libmpv's pixels go, below.

## On top of the page

libmpv draws into a native view that sits on top of the webview, opaque, in the rectangle the page names. The webview stays as it is. Nothing is drawn over the video, so nothing of the page has to show through it, the webview is never made transparent, and none of the costs transparency carries on each platform — the Mac's private-API feature among them — is paid. The reports of mpv and VLC drawing on top of a Tauri webview, in the tauri-apps discussion, describe the arrangement wanted here rather than a failure.

What that needs, per platform, is a native child view above the webview in the same window, which is the platform question `window_build` already asks once, where a thing is made: an NSView added above the WKWebView on the Mac, a child window z-ordered above WebView2's on Windows, a GTK widget over WebKitGTK's on Linux. The page tells Rust the rectangle in backing pixels on every pan, zoom, resize and fullscreen change, and the view trails the page by at most a frame, which shows only if the page draws something that moves with the video. The view ignores the mouse, so a drag over the video reaches the page the way a drag over a picture does. The page owns the controls, beside the video and on the keyboard.

Questions: Does mpv's render API take a context fuji makes on that view on each platform, or does the Mac want the render API's OpenGL type against a deprecated GL? Can the view's rectangle follow a drag on the table frame for frame, or does a video sit still on the table where a picture would pan? What does a window hand a child view on each platform when the window goes fullscreen, and does the view come along?

**A separate mpv process controlled over IPC**, `tauri-plugin-mpv`'s route, draws into a window handle the host gives it and takes commands over a socket. It needs mpv installed on the machine, so it is not shippable, but it is the fastest thing to try to see playback at all, and it is also the answer to the security question below.

## Two routes, and the manifest

`fileTypes.js` lists two routes per video format per platform, the way it lists two for thumbnails: `videoWeb`, the webview's own video tag, and `videoNative`, libmpv. The first argument between them is capability. The webview plays only what its engine plays: WebView2 plays H.264 out of the box and HEVC only once the HEVC Video Extensions package from the Store is installed and licensed; WebKit on the Mac plays what QuickTime plays; WebKitGTK plays through GStreamer, only from `http(s)`, `file` and `blob` URLs — a custom scheme fails before a pipeline is built — and only with the plugins the machine has. libmpv plays everything in the table.

The second argument is performance, and it cuts the other way. On the Mac, a recent, long, huge 4K file visibly costs VLC something: by eye, around a hundred milliseconds of spinning up before the first frame, and a seek that is slow. The very same file dragged into Chrome plays as if it took no effort, and seeks the same way. So the route is not simply the web for the small, the old and the odd, and libmpv for the new. The guess is that the engine's player is tuned for exactly the file a phone or a video site makes today, with the platform's hardware decoder underneath, and whether libmpv matches it on that file is not known: libmpv is not VLC, and mpv reaches the same hardware decoders, so the gap may close or may not, and measurement decides.

To explore, once libmpv plays at all: the same big modern file through both routes on each machine, with libmpv's hardware decoding on, and time to first frame, time to seek and frames dropped written to the log. Then the manifest is adjusted from the results: the `videoWeb` and `videoNative` lists, and which of the two the page prefers where both can play, which the thumbnail flow already decides per format by putting `imageNative` first.

## Shipping libmpv

Each of fuji's six packages has to carry libmpv or depend on it, and the answer is different in each.

- **Windows.** `zhongfly`'s `mpv-dev-lgpl` builds are the source the existing plugin uses; a single `libmpv-2.dll` carrying FFmpeg and libplacebo inside it, bundled through Tauri's resources, from `tauri.windows.conf.json`. Measured on 2026-10-12: the dll is 101 MB, and the installer carrying it 36.8 against 3.7 without; `video-spike.md` has the launch numbers. Those builds are nightlies of mpv's and FFmpeg's git master, kept thirty days, so the spike decided the same day that Windows builds its own from the tagged release, pinned in the repository: `pnpm libmpv`, from `win-libmpv/build.sh`, a 48 MB dll of mpv 0.41.0 on FFmpeg 9.0.2 with Direct3D 11, both hardware decoders, dav1d and lcms2 inside and nothing from anywhere but twelve release sources.
- **Mac.** Homebrew's mpv is built for the player and not necessarily lgpl; the dmg carries it today, with the 47 libraries under it, gathered by `frameworks.js` and rewritten to find each other inside the bundle, and whether it stays that way or becomes one library built our own way is the video spike's question. Universal or arm64-only follows whatever the app is.
- **Linux, deb and rpm.** Depend on the distribution's package — `libmpv2` on Debian 12 is 0.35, on Ubuntu 24.04 0.37 — which keeps the packages small and puts the codec licensing on the distribution, where it already is. The client API is versioned and has been ABI-stable across those releases, but the minimum version fuji needs is a question, and Debian 12's 0.35 predates `gpu-next` as the default. The build containers on the Mac would need `libmpv-dev` in the image.
- **Flatpak.** The freedesktop runtime has an ffmpeg-full extension but no mpv, so the flatpak bundles libmpv, built in the container from the same source as everything else.

Whether the lgpl build configuration loses anything fuji wants is a question for the first build: it drops some GPL-only code, none of it in the player core.

## The binding and the commands

The Rust bindings to libmpv have a history of going stale; the contributor to the Tauri discussion who suggested libmpv called them lacking. The client API is small — create, set option, command, observe property, wait event — and the render API smaller, so fuji declares the functions it calls by hand in `mpv.rs`, against `client.h`, with no crate and no bindgen; the render API's declarations join them when a frame is drawn.

The commands fuji would expose have to pass the `lib.rs` test: describable without naming a fuji feature. A player is stateful, so this is a handle per window rather than one atomic call, and the shape is probably three commands — open a file into the window's player, send it one mpv command with its arguments, and read one property — with mpv's events forwarded up as Tauri events, and one more that places the player's view at a rectangle. The page owns the controls, the sequencing and what plays when, exactly as `TestFlow.vue` owns the thumbnail loops. Audio output is mpv's, on every platform, and nothing to build.

## Beside playback

- **A frame for the sheet.** A video's thumbnail. The operating system's thumbnailer does video on Windows and the Mac already, through the same `thumbnail.rs` route pictures use. Linux has no native route and would need mpv or FFmpeg to pull a frame. Which one, and at what point in the file, is a thumbnail question for when video is switched on.
- **Security.** libmpv is FFmpeg parsing untrusted files inside fuji's process, a far larger surface than the image decoders fuji already worries about. Whether playback belongs in a process of its own is an open security question; the IPC route above is also the answer to it, which is worth knowing before the spike settles on in-process rendering.

## Versions, as of 2026-09-29

mpv 0.41 is the current release, requiring FFmpeg 6.1 or later and libplacebo 6.338.2 or later; development builds are at 0.41.0 plus a thousand commits. FFmpeg is at 8.1 "Hoare", after 8.0 "Huffman" brought Vulkan compute codecs. VLC 4.0 is still in beta on the desktop. GStreamer's stable series is 1.26. These are here so the reader can tell how far the world has moved since this was written.
