# From the Windows box to the Mac: copies that start together

Written on the Windows box, 2026-09-28, alongside the change to `window_first` in `window.rs`. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one.

## What changed

**On Windows, opening several pictures at once used to leave processes behind.** Explorer starts a copy of fuji for every picture a user selects and opens with Enter, all within a fraction of a second, and WebView2 refuses every copy but the first to ask. Tauri had already answered success and dropped each half-made window without a word, so the copies that lost sat forever with no window and nothing to end them.

**Now copies that start together follow three rules**, in `window_first` and the functions below it. A flurry of copies opens one window, and the rest leave at once. A later launch waits its turn for a window of its own. And a copy whose first page never begins to load exits after twenty seconds. The essay above the `WINDOW_TURN` presets in `window.rs` has the whole of it, and `lib.rs` now notes each copy's launch moment before anything else.

**None of it is gated by platform, and on the Mac it should do nothing you can see.** The turn is never contended. A flurry needs two fuji processes launched within 200 milliseconds, and macOS runs one. And a release build never exits on the watchdog, because it stays resident. What the Mac does notice is an empty `window-turn.lock` in the app's local data folder, and the first window being asked for from a thread of its own, a few milliseconds later than before.

**Every way out of fullscreen now goes through a new command, `window_fullscreen_leave`.** On Windows, Tauri's own call showed a hidden window again at its old frame, which flashed at Esc; the command keeps it hidden there. Its Mac body is the one line the page used to call, `set_simple_fullscreen(false)`, but it has only ever been compiled for Windows, so if the build complains, or leaving the table for the sheet or pressing Esc behaves differently, look there first.

## What only the Mac can check

1. **Select five pictures in Finder and open them with fuji.** Expect one window. The essay says the Mac already opens a selection as one window, because the shell expects Finder to hand the whole selection over in one `Opened` event and opens its window on the first picture. That was read from the shell's code rather than seen. Five windows would mean Finder sends an event per file, and the Mac would then behave differently from Windows, where the same selection now opens one window.
2. **Leave a `pnpm local` window open for thirty seconds.** The watchdog counts a window as arrived when Tauri's `on_page_load` reports its page beginning to load, and that has only been seen to fire with WebView2. A development build is not resident, so if WKWebView's page load never reached it, the watchdog would close the window at twenty seconds and the terminal would say "window: the first window's page never began to load". Expect it to stay. A release build is resident and would stay either way, which is why only the development build can tell.

Put what you find in the essay above the `WINDOW_TURN` presets, replacing "still to be confirmed there" with the answer, and then retire this letter.
