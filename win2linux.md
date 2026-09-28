# From the Windows box to Linux: a watchdog that must see the page load

Written on the Windows box, 2026-09-28, alongside the change to `window_first` in `window.rs`. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one.

## What changed

**Copies of fuji that start together now follow three rules**, written for Windows, where Explorer starts a copy for every picture a user opens at once and WebView2 refused all but the first. A flurry of copies opens one window and the rest leave at once. A later launch waits its turn. And a copy whose first page never begins to load exits after twenty seconds, so nothing is left running without a window. The essay above the `WINDOW_TURN` presets in `window.rs` has the whole of it.

Linux runs a copy per launch, as Windows does, so all of it runs there too. Most of it should pass unnoticed. There is no WebView2 and so no race, and the turn only means copies bring their windows up one at a time. Nothing hands fuji pictures on Linux yet, since the desktop file declares no `MimeType`, so a flurry can only come from the command line. The lock is a file in the app's local data folder, which a Flatpak keeps inside its sandbox, and a copy that cannot have the file simply goes ahead without a turn.

## What only Linux can check

1. **Launch fuji from an installed package and leave its window open for thirty seconds.** Expect it to stay. The watchdog counts a window as arrived when Tauri's `on_page_load` reports its page beginning to load, and that has only been seen to fire with WebView2. Linux never keeps fuji running without a window, so if WebKitGTK's page load never reached Tauri, the watchdog would close every window at twenty seconds. Launched from a terminal, the copy would say "window: the first window's page never began to load" as it left. That would need fixing in `window.rs` before another package ships. One run on the Linux desktop answers it for the `.deb` and the `.rpm`; the Flatpak and the Raspberry Pi's `arm64` package are worth the same thirty seconds when convenient, since each is a different build of the same engine.

Put what you find in the essay above the `WINDOW_TURN` presets, a clause saying the page load was seen to fire on Linux, and then retire this letter.
