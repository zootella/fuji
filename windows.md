# To the Windows session

Fuji can now be opened by double-clicking a picture. The macOS half is built and verified; the Windows half is built and has never run. That second clause is the reason for this letter.

**Read `associations.md` first, then talk to the user.** It is the record — what each platform offers, why fuji offers every type it can show and claims none of them, and exactly which registry values get written and why. This letter only says what that document cannot: where things actually stand, and the two things that are easy to get wrong on the first try.

## What is waiting for you

`associate.rs` writes the registration and `open.rs` reads the command line. Both compile against `x86_64-pc-windows-msvc` — checked from the Mac with the scratch-crate trick `CLAUDE.md` describes — and that is the whole of what can be claimed for them. No line of either has executed on Windows. Treat the first launch as an experiment rather than a verification.

The one line of evidence that it ran is in the log, with `log.record = true`: `associate: 10 types registered, N values written`. On a first launch N should be around sixty-five. On the second it should be zero, because every value is read before it is written.

## Two things to get right

**Capture the registry before you launch fuji the first time.** The comparison is the point and it cannot be taken afterwards.

    reg query "HKCU\Software\Classes\.webp" /s
    reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.webp" /s

**`pnpm local` registers nothing, deliberately.** A debug build is guarded out, because the command paths come from `current_exe()` and pointing the registry at `target\debug` would leave it aimed at a binary that moves and disappears. Use the installed build or this will look broken when it is working.

## What is yours to answer

Three observations the Mac cannot make, in `associations.md`'s open section: whether Windows 10 prompts the user on the next double-click of a registered type after a *runtime* registration rather than an install-time one; whether `ms-settings:defaultapps?registeredAppUser=Fuji` works on 10 or only on 11; and whether Explorer still draws thumbnails for a type fuji has been chosen for. That last one could change the design rather than merely disappoint, so it is worth doing early.

Then the uninstall hook, which is designed in that document and unwritten. Fuji writes its keys at runtime and the NSIS uninstaller knows nothing about them, so today uninstalling would leave fuji in Open with lists pointing at an executable that is gone.

## What we already know will disappoint

On macOS the registration is correct and the route to actually using it is close to invisible — Get Info, a collapsed section, a small button. The settings panel is the answer and is a sprint of its own. Windows may come out better here, because Settings lists an application by name with all its types and Windows may do the asking itself. Whether it does is your first question above.
