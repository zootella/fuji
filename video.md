# Video

Fuji will play video through libmpv. Decided 2026-09-29 on the Windows box after a research pass, and nothing is built. This document owns playback — how a video file reaches the screen and the speakers once fuji has decided to show it. What fuji claims to open is `fileTypes.js`, whose `.mp4` and `.mov` entries already carry a `videoNative` route that names libmpv, and the registration side is `associations.md`.

A whiteboard: far more asked than settled, and the settled part is one paragraph.

## What is decided

**libmpv, through Rust.** Four reasons, in the order they weighed. It is FFmpeg underneath, so it plays what VLC plays, which is everything — and for fuji that word means the video a 1990s collection actually holds, which is not H.264 in an MP4. It is AVI with Cinepak, Indeo or Microsoft Video 1 inside, MPEG-1 off a Video CD, QuickTime with Sorenson, RealVideo, and later DivX, Windows Media and Flash Video; `fileTypes.js` now lists them all, off. No webview plays any of that, and neither does QuickTime Player or Movies & TV, so libmpv is the only route that makes such a folder open at all, the way the picture entries already open a folder of Paint's BMPs. Its render API draws into a context the host owns, instead of demanding a window of its own, and mpv's own documentation now recommends that route over window embedding, especially on macOS. Its video output is the best in open source — libplacebo's `gpu-next` is the default since 0.41, with color management, HDR tone mapping and proper scalers — and that is the part of playback fuji cares about, since `fidelity.md` is a whole document on whether a picture's colors arrive intact. And it is LGPL 2.1 or later when built with the lgpl option, which is what the prebuilt Windows libraries ship as, so linking it puts no license on fuji.

**Why not the others**, written once so nobody researches it twice. *libVLC*: VLC 4 was still beta and nightly on the desktop as of mid-2026 and libVLC 4 pre-release with it; the API is from the 2001 module design, and people embedding it in Tauri report it rendering over the webview rather than under it. *FFmpeg directly*: the right tool when frames are data — thumbnails, scrubbing — and it may still come in for that, but as a player it means writing audio output and A/V sync ourselves. *GStreamer*: a framework rather than a player, heavy to ship, and Windows and the Mac are its second-class platforms; it is what WebKitGTK already plays through. *The webview's video tag*: free and kept as the `videoWeb` route in the table, but it plays only what the platform engine plays, which is the problem VLC was invented to solve. *AVFoundation and Media Foundation*: three implementations to maintain, each playing only what its operating system plays.

## The central question: getting pixels under the webview

The webview covers the window. A native surface libmpv draws to is either behind it, showing through a transparent region, or in front of it, covering fuji's controls. Both mpv and VLC embedded in the same window as a Tauri webview were reported drawing on top, in the tauri-apps discussion, and the workarounds were child windows kept in sync with a div by hand. This is the spike, and it is platform-specific in exactly the way `window_build` is meant to hide.

The routes seen so far, none tried here:

- **A native child view under a transparent webview.** What `tauri-plugin-libmpv` does on Windows, with the window's transparency switched on, and what `get-air/tauri-video-plugin` does on Linux with a GTK GPU widget below the webview. Transparency has costs of its own on each platform and the Mac requires title bar customization to allow it at all, per the discussion.
- **Software rendering into a buffer the page draws.** The render API has a software backend, so Rust could hand each frame up as pixels and the page paint them into a canvas. One copy per frame across the boundary; 4K at 60 frames is a gigabyte and a half a second, so probably only viable for small tiles, and at odds with the rule that a command takes one thing.
- **WebView2's TextureStream.** `get-air` passes pooled D3D11 textures into a real video element on Windows. Windows only, and Chromium only.
- **A separate mpv process controlled over IPC**, `tauri-plugin-mpv`'s route. Needs mpv installed on the machine, so it is not shippable, but it is the fastest thing to try to see playback at all.

Questions: Does Tauri let a native view sit beneath the webview on each platform, or only beside it? What does a transparent webview cost on the Mac, where it has been the source of drawing bugs? Is mpv's Metal or Vulkan-on-Mac output usable through the render API with a context fuji creates, or does the Mac want the render API's OpenGL type against a deprecated GL?

## Shipping libmpv

Each of fuji's six packages has to carry libmpv or depend on it, and the answer is different in each.

- **Windows.** `zhongfly`'s `mpv-dev-lgpl` builds are the source the existing plugin uses; a single `libmpv-2.dll` carrying FFmpeg and libplacebo inside it. Bundled through Tauri's resources. Size to measure — the dll is tens of megabytes, against an installer that is two today.
- **Mac.** Homebrew's mpv is built for the player and not necessarily lgpl; the dmg would carry a dylib of our own build, or a prebuilt one, plus whatever it links. Universal or arm64-only follows whatever the app is.
- **Linux, deb and rpm.** Depend on the distribution's package — `libmpv2` on Debian 12 is 0.35, on Ubuntu 24.04 0.37 — which keeps the packages small and puts the codec licensing on the distribution, where it already is. The client API is versioned and has been ABI-stable across those releases, but the minimum version fuji needs is a question, and Debian 12's 0.35 predates `gpu-next` as the default. The build containers on the Mac would need `libmpv-dev` in the image.
- **Flatpak.** The freedesktop runtime has an ffmpeg-full extension but no mpv, so the flatpak bundles libmpv, built in the container from the same source as everything else.

Whether the lgpl build configuration loses anything fuji wants is a question for the first build: it drops some GPL-only code, none of it in the player core.

## The binding and the commands

The Rust bindings to libmpv have a history of going stale; the contributor to the Tauri discussion who suggested libmpv called them lacking. The client API is small — create, set option, command, observe property, wait event — and the render API smaller, so a bindgen of `client.h` and `render.h` inside fuji may be the honest choice over a crate. To check before deciding: `libmpv2` on crates.io and its state, and `tauri-plugin-libmpv` at 0.3.2 from November 2025, MPL-2.0, Windows tested, Linux embedding not working, Mac untested, and dependent on a wrapper library of its own.

The commands fuji would expose have to pass the `lib.rs` test: describable without naming a fuji feature. A player is stateful, so this is a handle per window rather than one atomic call, and the shape is probably three commands — open a file into the window's player, send it one mpv command with its arguments, and read one property — with mpv's events forwarded up as Tauri events. The page owns the controls, the sequencing and what plays when, exactly as `TestFlow.vue` owns the thumbnail loops. Audio output is mpv's, on every platform, and nothing to build.

## Beside playback

- **A frame for the sheet.** A video's thumbnail. The operating system's thumbnailer does video on Windows and the Mac already, through the same `thumbnail.rs` route pictures use. Linux has no native route and would need mpv or FFmpeg to pull a frame. Which one, and at what point in the file, is `thumbnail-open.md`'s kind of question and is noted there when video is switched on.
- **Security.** libmpv is FFmpeg parsing untrusted files inside fuji's process, a far larger surface than the image decoders `security.md` is already worried about. Whether playback belongs in a process of its own is that document's question; the IPC route above is also the answer to it, which is worth knowing before the spike settles on in-process rendering.
- **What the webview plays**, for the `videoWeb` fallback and for the day libmpv is not there. WebView2 plays H.264 out of the box and HEVC only once the HEVC Video Extensions package from the Store is installed and licensed. WebKit on the Mac plays what QuickTime plays. WebKitGTK plays through GStreamer, only from `http(s)`, `file` and `blob` URLs — a custom scheme fails before a pipeline is built — and only with the plugins the machine has. Once libmpv is in, this route may be worth nothing, and the table's `videoWeb` lists can say so.

## Versions, as of 2026-09-29

mpv 0.41 is the current release, requiring FFmpeg 6.1 or later and libplacebo 6.338.2 or later; development builds are at 0.41.0 plus a thousand commits. FFmpeg is at 8.1 "Hoare", after 8.0 "Huffman" brought Vulkan compute codecs. VLC 4.0 is still in beta on the desktop. GStreamer's stable series is 1.26. These are here so the reader can tell how far the world has moved since this was written.
