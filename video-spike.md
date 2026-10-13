# Video spike

libmpv linked into fuji and carried by every package, with no frame ever drawn, to learn what carrying it costs before playback is designed around it, and one feature that uses it without a picture. Playback, how a frame reaches the screen, is not this spike's subject. Nothing is built.

## What it costs to carry

**Size.** The six packages today, from the sidecars staged here: the dmg built on the Mac on 2026-10-11, the exe built on the Windows box and the four linux packages built in the containers on the Mac on 2026-10-03.

| package | bytes |
| --- | --- |
| fuji.dmg | 5,559,271 |
| fuji.exe | 3,857,673 |
| fuji.amd64.deb | 5,974,246 |
| fuji.arm64.deb | 5,960,996 |
| fuji.x86_64.rpm | 5,976,187 |
| fuji.x86_64.flatpak | 4,480,440 |

The same six with the library inside is the measurement. The dll alone is tens of megabytes against an installer under four, but the installer compresses what it carries, so the installer is the number and not the dll. The deb and the rpm are meant to depend on the distribution's libmpv rather than carry it, so they should not grow at all; the flatpak carries it and will.

**Startup and memory.** The log's first two lines, written by the shell at every launch: the launch timed whole from the operating system starting the process, through the loader, main and the page, to the first frame; and the footprint of each of fuji's processes two seconds later, before anything is opened. The loader's share is the one a linked library adds to. The answer turns on whether the library is mapped at launch, as a library linked at build time is, or opened on the first play. The spike measures whichever it builds and says which, and if the first costs something a collector who never plays a video would notice, tries the second.

The baseline, before the library, on the Mac mini on 2026-10-12: the release app in the bundle folder, launched by double-click, with the binary at 12,810,576 bytes. The first launch after a build is the cold one, where macOS validates and assesses the new executable before main; the two after it are the warm ones, and the number to compare against.

| launch | to main | main to page | page to first frame | to first frame | host at rest |
| --- | --- | --- | --- | --- | --- |
| cold, first after the build | 467 | 386 | 137 | 990 | 21,957,248 |
| warm | 30 | 274 | 100 | 404 | 21,776,896 |
| warm | 42 | 269 | 102 | 413 | 21,580,480 |

Milliseconds, and the host's footprint in bytes; web content sat near 26 MB and the gpu process near 15 MB in every run, and neither should move with the library.

After, the same day and the same way, with libmpv and the 47 libraries under it linked at build time and carried in the bundle, so the loader maps all 48 before main: the binary at 12,901,120 bytes and the dmg at 32,834,062, against 5,559,271 before.

| launch | to main | main to page | page to first frame | to first frame | host at rest |
| --- | --- | --- | --- | --- | --- |
| cold, first after the build | 506 | 296 | 105 | 907 | 23,284,736 |
| warm | 85 | 232 | 97 | 414 | 22,268,800 |
| warm | 87 | 238 | 95 | 420 | 24,234,944 |

So the loader's share of a warm launch went from about 36 ms to about 86 ms, some 50 ms for 48 libraries, and the first frame still arrived at about 410 ms, within the run-to-run noise of the baseline. The host's footprint at rest grew by one to two megabytes, and the web engine's processes did not move. Web content and the gpu process sat where they were. That is the cost of linking at launch on this machine; loading on the first play would remove it, and whether 50 ms and two megabytes are worth that is a decision, not a measurement.

**The Mac pipeline.** How long the release takes from a build to a published dmg with the library in the bundle, against today's, and whether minutes become an hour. And what the library asks of the pipeline: every binary in the bundle under the same signature, and the library found at a path inside the bundle rather than the path it was built at. One question the fallback build already answered: the hardened runtime's library validation refused the 48 libraries under the self-signed identity, which has no Team ID, so the fallback signs without the hardened runtime now and the apple build alone turns it on, where the Developer ID gives every binary one Team ID. The notarized app launches under the runtime's other rules, LuaJIT among the 48 and all: launched twice from its kept folder on the Mac mini on 2026-10-12, it ran both times, and the warm launch read 495 ms to the first frame with main at 80 ms, in step with the fallback build's. Its cold first launch outran the five seconds the check allowed before quitting, so that number is unmeasured; the first launch of the next notarized build is timed with a longer wait, since a notarized app's first launch is where macOS validates every binary it carries.

The pipeline, measured the same day against the release of the day before, which carried no library: the signed build 29.2 seconds against 23.7, the zip and upload of the app 11.7 against 2.7, Apple's answer on the app 31.3 against 31.2, which is one poll of thirty seconds in both, the dmg made and signed in 4.6 against 4.0, its upload 10.6 against 2.9, and Apple's answer on the dmg 31.3 against 31.2. Two minutes end to end, not an hour: 49 binaries to sign cost five seconds, 30 MB to upload cost nine seconds twice, and Apple's own check did not slow at all.

## The extension cluster

Switch on the video entries in `fileTypes.js` that libmpv plays, so the settings panel shows them as chips under whichever program opens each one today, in the cluster the picture chips already make. `Info.plist` and the Windows uninstaller's list name the enabled types by hand, so each gains the same entries. Whether the audio entries come on in the same pass, since libmpv plays those too.

Questions: What does a double-click on an enabled video file do while nothing draws video, and is the audio window below where it opens? What does the sheet show for one, a chip, the placeholder, or a frame from the operating system's thumbnailer, which already does video on Windows and the Mac?

## Fuji Audio

A view that plays sound and never a picture, from audio files and from video files alike. It plays through libmpv with its video switched off, so the spike exercises the library linked and shipped, the binding, the commands, the events coming back up, and the page's controls, with no native view and no pixels anywhere. Audio output is mpv's on every platform, and nothing to build. The page owns what it shows and what plays when.

For the spike it is a view in the shell and not a window, so that window and process management wait: `a` on the sheet goes to Fuji Audio and `a` there goes back to the sheet, the way `s` goes between settings and the sheet, and `a` on settings goes to Fuji Audio and then back to the sheet. That much is built, in `AudioPanel.vue`, and the view shows one line, the library's version with its client api and FFmpeg's, which is how the link was proven; nothing plays yet. Where it belongs for real, a window of its own or not, and how it opens then, from the sheet, from the table or from the operating system handing fuji a file, is designed once the library and the builds are known.

Next, the sound: a player started for the window with the options the page chooses, video off among them; one command that sends it one mpv command with its arguments, one that reads one property as text, and one that stops it; a dropped file loaded and played, with play and stop on the page and nothing more; and the test file, a Creative Commons track in a folder under Downloads on the Mac mini, named in chat. Events coming back up wait, and the view reads the position itself while it shows.

Questions: Which keys does it answer? One player for everything, or one per file? Does a video file opened there play its sound while the sheet goes on underneath?

## Shipping and the binding

Shipping is decided by doing. The spike takes the first route that gets the library into the build on each machine, the prebuilt lgpl dll on Windows, Homebrew's tree inside the bundle on the Mac, the distribution's package in the deb and rpm containers and a build of our own in the flatpak's, and writes what it took where it took it, in the build scripts and the Rust. The binding is settled: declarations written by hand in `mpv.rs` against mpv's client header, seven of them today, with no crate and no bindgen, since fuji calls a handful of functions whose signatures the API's second major version fixed.

## The Mac first

Here, where the pipeline question lives and the dylib is the hardest of the three to source. Built: Homebrew's libmpv links the spike, and `frameworks.js` gathers it with the 47 libraries under it, FFmpeg's among them, rewrites each copy to find its siblings inside the bundle, signs each, and hands Tauri the list, so the fallback build carries the tree and launches; the numbers above are from it. Left here: the pipeline measurement, and whether 48 libraries from Homebrew are the argument for one self-contained library built our own way, which is what the Windows prebuilt already is.

## Windows

Built on the Windows box on 2026-10-12, from the Mac's letter in `mac2win.md`. The link was the easiest of the three, as expected: the gates in `lib.rs` and `mpv.rs` opened to windows, and the extern block there carries `#[link(name = "libmpv-2", kind = "raw-dylib")]`, so the linker writes the imports from the seven declarations and no import library is made or kept. The library is zhongfly's mpv-winbuild lgpl dev archive of the same day, the plain x86_64 one, whose `client.h` matches the seven declarations at client api 2.5; its `libmpv-2.dll` sits untracked in `src-tauri/binaries/`, and is 101,207,552 bytes, a hundred megabytes rather than tens. Shipping is a tracked `tauri.windows.conf.json` naming the dll a resource, which Tauri merges on Windows alone and copies beside the executable in both profiles, and which `win-setup.js` now merges the same way, since it read `tauri.conf.json` alone before and would have made an installer without the dll in it. Two things found on the way. `tauri_build` watches only the configuration files that existed when it last ran, so the first build after the platform file appears does not copy the dll into `target/release` until `build.rs` or `tauri.conf.json` changes; a fresh checkout never sees this. And the dll hard-imports `vulkan-1.dll`, which graphics drivers install and Windows does not, so on a machine without one the loader fails before main and fuji never starts; loading on the first play would remove that along with the launch cost.

The baseline, the installer built on the code as pulled and installed, with the executable at 11,561,984 bytes and the installer at 3,872,685: the cold launch is the one the installer makes at its end, and the warm one is from the Start menu.

| launch | to main | main to page | page to first frame | to first frame | host at rest |
| --- | --- | --- | --- | --- | --- |
| cold, the installer's own launch | 25 | 463 | 176 | 664 | 4,435,968 |
| warm | 29 | 351 | 120 | 500 | 4,177,920 |

After, the same way, with the dll in the import table and mapped before main: the executable at 11,584,512 bytes and the installer at 38,595,036, its cabinet 36.6 MB where it was 3.5, made in 40 seconds where it was 6.

| launch | to main | main to page | page to first frame | to first frame | host at rest |
| --- | --- | --- | --- | --- | --- |
| cold, the installer's own launch | 291 | 406 | 130 | 827 | 6,737,920 |
| warm | 39 | 342 | 122 | 503 | 6,287,360 |
| warm | 29 | 327 | 127 | 483 | 6,164,480 |

Milliseconds, and the host's footprint in bytes; the engine's processes, the browser near 30 MB, the gpu process 26 to 29 and the renderer 19 to 22, sat where they were in every run. So a warm launch paid nothing anyone could measure: Windows maps the dll and reads its pages on demand, and nothing calls into it before the first frame. The cold launch paid about 260 ms before main, the first time the hundred megabytes were mapped after the installer wrote them, which is where Windows' antimalware looks at a new file and where its pages are read for the first time; which of the two it mostly was is not measured, and an install or an update pays it once either way. The host at rest grew by two megabytes, and the installer by thirty-five, ten times its size. That is the cost of linking at launch here; the warm number is the one every double-click on a picture pays, since each picture window is a process of its own, and here it is nothing. The size is the decision.

The eyes: on the installed after build, `a` on the sheet showed the audio panel's one line with the library's version, mpv v0.41.0-1150-g88eb9f2e7 on FFmpeg N-127388-g615b07170, a development build of both, and the About section listed WebView2, libmpv and FFmpeg with their numbers. The panel's text could not be copied, which is noted and not chased.

**The dll is a nightly, and that is the next thing to change here.** mpv publishes no Windows binaries, and the two build farms it points at, shinchiro's and zhongfly's fork of it, build mpv's and FFmpeg's git master every day and tag nothing else: the version the panel showed is 1,150 commits past 0.41.0, on an FFmpeg with no number at all, and zhongfly keeps thirty days of builds, so the archive this build came from is gone by November and no second machine could make these bytes. The Mac links Homebrew's mpv 0.41.0 on FFmpeg 9.0.2, plain release numbers. Fuji plays MP3s and MOVs that are years old, which 0.41.0 and the FFmpeg it was released against decode whole, so nothing newer buys anything but churn. Decided on the Windows box on 2026-10-12: Windows builds its own libmpv from the tagged release, pinned by version in the repository so every machine gets the same bytes, the way the flatpak's container plans to from source; the nightly stays as what the spike measured with and nothing more. That build is also where the Vulkan import and everything else fuji will not use come out, and where the question of how small a library of our own would be gets its number.

**Built, the same evening.** `pnpm libmpv` in desktop runs `win-libmpv/build.sh` inside MSYS2's ucrt64 environment, which `win-libmpv/libmpv.js` installs the toolchain into the first time: eight release tarballs, each fetched from its own project and checked against the hash written in the script, built static in order, zlib 1.3.2, FreeType 2.15.0, FriBidi 1.0.17, HarfBuzz 14.6.0, libass 0.17.5, libplacebo 7.360.1 at its tagged commit, FFmpeg 9.0.2 and mpv 0.41.0, and the last linked as the one dll with the other seven and the mingw runtime inside it. FFmpeg keeps every decoder and demuxer and loses encoders, muxers, devices and network; mpv loses the player, Lua and JavaScript, the disc formats and the frame servers, and keeps WASAPI, OpenGL and Direct3D 9; libplacebo was built without Vulkan and, at that point, without Direct3D 11, which wanted a shader compiler and its translator, two packages more. Fourteen minutes on the Windows box, FFmpeg nine of them, and the first run stopped three times, on an optional HarfBuzz dependency looked up through cmake, on the Windows profile path meson's cmake module needs, and on a FFmpeg flag that 9.0 removed, each a one-line fix now in the script. The dll is 34,136,064 bytes stripped, a third of the nightly's 101, and imports Windows' own libraries and the universal C runtime and nothing else: no Vulkan, no mingw runtime. The installer carrying it is 16,526,106 bytes, against 38,595,036 with the nightly and 3,872,685 with no library at all. The library the panel names is mpv 0.41.0 on FFmpeg 9.0.2, the same two numbers Homebrew gives the Mac. The build is reproducible to the byte: the linker and strip stamp a time into the PE header, the recipe pins that time to the day it was written through SOURCE_DATE_EPOCH, and two relinks of mpv from the same sources gave the same sha256, 9bddf4f479d5806a67b344fbd2bba9fb850d20cd24046a6f6c1eb857b8ee1274, so the hash names the recipe and not the minute.

Launched the same way as the two tables above, installed and started by the installer, then twice from the Start menu; the installer's launch was quit before its two seconds were up, so its footprint went unrecorded.

| launch | to main | main to page | page to first frame | to first frame | host at rest |
| --- | --- | --- | --- | --- | --- |
| cold, the installer's own launch | 42 | 381 | 147 | 570 | |
| warm | 20 | 341 | 119 | 480 | 4,587,520 |
| warm | 19 | 331 | 123 | 473 | 4,337,664 |

So the built dll costs a warm launch nothing, like the nightly, and its cold launch a quarter of a second less than the nightly's, a third of the bytes to map for the first time. The host at rest is back where the baseline had it, 4.3 to 4.6 MB against the nightly's 6.2 to 6.7: the two megabytes the nightly took before anything played came with that build and not with the library as such.

**Finished for video, the same night.** Sound alone the web view could have played; libmpv is here for video that is excellent on every Windows machine, so the dll was completed rather than left at what the audio panel needs, with nothing to come back to this part of the garage for. Four packages joined the recipe, and three more inside one of them. Direct3D 11 output, which every Windows 10 draws with, including a machine with no graphics driver through Microsoft's software rasterizer, and which hardware decoding hands frames to without a copy, where OpenGL on Windows depends on the vendor's driver and is missing in virtual machines and over Remote Desktop: libplacebo and mpv both wanted it, and both want shaderc and SPIRV-Cross for it, since Direct3D takes shaders in HLSL and mpv writes them in GLSL, so shaderc 2026.4 compiles GLSL to SPIR-V, with glslang, SPIRV-Tools and SPIRV-Headers built inside it at the commits its DEPS file names, and SPIRV-Cross at the Vulkan SDK's 1.4.363.0 tag translates SPIR-V to HLSL. dav1d 1.5.4 for AV1, which present-day files carry and FFmpeg decodes in software through no other library. lcms2 2.19 for ICC profiles, since a fidelity-minded viewer honors a calibrated display. And mpv's media keys, through Windows' own transport controls. Two wrinkles, both now in the script: mpv and libplacebo ask pkg-config for SPIRV-Cross and shaderc under the names of their shared builds, so the recipe writes a file under each name listing the static libraries and the C++ runtime they need; and the media keys are written against Microsoft's C++/WinRT headers, which the mingw toolchain lacks and MSYS2 packages beside it, header-only, so they joined the toolchain list. The build is about twenty minutes now, shaderc four of them. The dll is 48,202,240 bytes, the installer 20,842,016, and the imports are still Windows' own and the C runtime, with two Windows Runtime API sets for the media keys; no Direct3D library is in the import table, since mpv loads those at runtime, so a machine without one still starts fuji. mpv's summary reads d3d11, opengl, shaderc, spirv-cross, lcms2, wasapi and win32-smtc yes, FFmpeg's config reads libdav1d, d3d11va and dxva2 on, and libplacebo's reads d3d11, opengl, shaderc and lcms. Left out on purpose: Vulkan, which Direct3D covers on Windows; ANGLE, which nothing needs; Lua and JavaScript, which only mpv's own on-screen scripts use; and the disc formats, the frame servers and the archive reader. Reproducible like the smaller one: the first build and two relinks after it all gave sha256 a6d92d7b428943d43a7fe4021548e16d1b4998fdf7d6f0ceb6f11b38f454e615.

Launched the same way, the installer's own launch and then one from the Start menu, with the audio panel naming mpv 0.41.0 on FFmpeg 9.0.2 and About listing both:

| launch | to main | main to page | page to first frame | to first frame | host at rest |
| --- | --- | --- | --- | --- | --- |
| cold, the installer's own launch | 60 | 364 | 133 | 557 | |
| warm | 22 | 321 | 128 | 471 | 4,546,560 |

The complete dll costs what the smaller one cost, which is nothing warm and a half second cold. Decided on 2026-10-12, with the numbers in hand: the sizes and the speeds are fine as they are, 48 MB on disk and 20.8 MB to download, and the library's build is finished rather than open; what remains of the spike on Windows is sound, which the Mac takes first.

## Linux

Last, and each of the four packages is a decision to do, waive, research or test. The deb and the rpm link against the distribution's libmpv, which is `libmpv-dev` in the three container images and a dependency line in the packages, with nothing to carry; the version in the base image predates `gpu-next` as the default, which matters nothing to sound. The flatpak is the hard one: the runtime has no mpv, so the flatpak's container builds libmpv from source, and its sandbox has to be allowed to make sound at all. Whether the flatpak plays sound in this spike or waits is a decision to make before its build is attempted. Installed packages are tested on the Linux desktop and the Raspberry Pi, where startup with the distribution's shared objects is measured.
