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

After the Mac, with a letter. Linking is expected to be the easiest of the three: the prebuilt lgpl dll is one file carrying everything, and Rust can name a dll in the extern block itself, with no import library and no tool to make one. Shipping is the dll beside the executable, which is how resources already stage into the installer. What is sharpest there is size, the dll against an installer under four megabytes, and startup, since a dll in the import table is mapped at launch. There is no signing pipeline on Windows, so the pipeline question does not travel.

## Linux

Last, and each of the four packages is a decision to do, waive, research or test. The deb and the rpm link against the distribution's libmpv, which is `libmpv-dev` in the three container images and a dependency line in the packages, with nothing to carry; the version in the base image predates `gpu-next` as the default, which matters nothing to sound. The flatpak is the hard one: the runtime has no mpv, so the flatpak's container builds libmpv from source, and its sandbox has to be allowed to make sound at all. Whether the flatpak plays sound in this spike or waits is a decision to make before its build is attempted. Installed packages are tested on the Linux desktop and the Raspberry Pi, where startup with the distribution's shared objects is measured.
