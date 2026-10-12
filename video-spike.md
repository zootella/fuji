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

**Startup and memory.** Time from launch to the first window painted, and fuji's memory at rest with a folder on the sheet and no sound ever played, before and after, on each machine, through the log and the memory command fuji already has. The answer turns on whether the library is mapped at launch, as a library linked at build time is, or opened on the first play. The spike measures whichever it builds and says which, and if the first costs something a collector who never plays a video would notice, tries the second.

**The Mac pipeline.** How long the release takes from a build to a published dmg with the library in the bundle, against today's, and whether minutes become an hour. And what the library asks of the pipeline: every binary in the bundle under the same signature, and the library found at a path inside the bundle rather than the path it was built at.

## The extension cluster

Switch on the video entries in `fileTypes.js` that libmpv plays, so the settings panel shows them as chips under whichever program opens each one today, in the cluster the picture chips already make. `Info.plist` and the Windows uninstaller's list name the enabled types by hand, so each gains the same entries. Whether the audio entries come on in the same pass, since libmpv plays those too.

Questions: What does a double-click on an enabled video file do while nothing draws video, and is the audio window below where it opens? What does the sheet show for one, a chip, the placeholder, or a frame from the operating system's thumbnailer, which already does video on Windows and the Mac?

## Fuji Audio

A view that plays sound and never a picture, from audio files and from video files alike. It plays through libmpv with its video switched off, so the spike exercises the library linked and shipped, the binding, the commands, the events coming back up, and the page's controls, with no native view and no pixels anywhere. Audio output is mpv's on every platform, and nothing to build. The page owns what it shows and what plays when.

For the spike it is a view in the shell and not a window, so that window and process management wait: `a` on the sheet goes to Fuji Audio and `a` there goes back to the sheet, the way `c` goes between settings and the sheet, and `a` on settings goes to Fuji Audio and then back to the sheet. Where it belongs for real, a window of its own or not, and how it opens then, from the sheet, from the table or from the operating system handing fuji a file, is designed once the library and the builds are known.

Questions: What does the view show, and which keys does it answer? One player for everything, or one per file? Does a video file opened there play its sound while the sheet goes on underneath?

## Shipping and the binding

Both are decided by doing. The spike takes the first route that gets the library into the build on each machine, the prebuilt lgpl dll on Windows, a dylib inside the bundle on the Mac, the distribution's package in the deb and rpm containers and a build of our own in the flatpak's, and writes what it took where it took it, in the build scripts and the Rust. The binding, a crate or a bindgen of mpv's two headers, is settled the same way, by whichever links first on all three.

## The Mac first

Here, where the pipeline question lives and the dylib is the hardest of the three to source. Homebrew's libmpv links the spike, and it comes with FFmpeg and dozens of dylibs of its own, every one of which a shipped bundle has to carry, rewrite to find each other inside the bundle, and sign. That is the pipeline measurement, and it may be the argument for one self-contained library built our own way, which is what the Windows prebuilt already is.

## Windows

After the Mac, with a letter. Linking is expected to be the easiest of the three: the prebuilt lgpl dll is one file carrying everything, and Rust can name a dll in the extern block itself, with no import library and no tool to make one. Shipping is the dll beside the executable, which is how resources already stage into the installer. What is sharpest there is size, the dll against an installer under four megabytes, and startup, since a dll in the import table is mapped at launch. There is no signing pipeline on Windows, so the pipeline question does not travel.

## Linux

Last, and each of the four packages is a decision to do, waive, research or test. The deb and the rpm link against the distribution's libmpv, which is `libmpv-dev` in the three container images and a dependency line in the packages, with nothing to carry; the version in the base image predates `gpu-next` as the default, which matters nothing to sound. The flatpak is the hard one: the runtime has no mpv, so the flatpak's container builds libmpv from source, and its sandbox has to be allowed to make sound at all. Whether the flatpak plays sound in this spike or waits is a decision to make before its build is attempted. Installed packages are tested on the Linux desktop and the Raspberry Pi, where startup with the distribution's shared objects is measured.
