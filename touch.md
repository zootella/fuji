# Touch

Fuji's controls across the input devices people actually have, and the plan for getting there from a table that was built with one of them. Touch here means a trackpad, a Magic Mouse, or any surface that scrolls by sliding a finger rather than turning a notched wheel, on any platform. The controls are flip, pan, zoom and gamma. `HelpPanel.vue` lists the bindings as built, and `DiamondTable.vue` is the only code that reads the wheel.

**Two cohorts carry most users:** Windows gamers with a notched, often clicky, wheel, and Mac designers on MacBooks with a trackpad and no mouse at all. The corners are a machine with both, on either platform; a Mac Studio with a Magic Mouse, which is a mouse with a touch surface and scrolls exactly like a trackpad; and a Windows laptop with a precision touchpad, which is rarer and waits.

## Where it stands, 2026-09-24

**Built and tested with a mouse with a notched wheel,** on the Mac mini and the Windows box. The wheel flips, control with it zooms a step, shift with it steps gamma, and each wheel event is one command with only its sign read. That is right for a notch, and it works well.

**A trackpad or a Magic Mouse reaches the page as the same wheel events,** dozens per swipe and more through the momentum after the fingers lift, so one swipe should flip through a dozen pictures. That is reasoned from what the engines do rather than watched: the MacBook Air is where it gets watched, and it has not been yet.

**Control with any arrow flips,** right or down for the next picture and left or up for the one before. It is presented as a control of its own beside the page keys, and it also happens to be the only flip a MacBook keyboard has, since a MacBook has no page keys. On the Mac, control with an arrow is the system's shortcut for moving between Spaces and never reaches fuji, so there it is command with an arrow, which the table reads as control everywhere. The help panel says Ctrl for those rows and for the wheel rows; whether it should say ⌘ on the Mac is open.

## The stages

**Stop the bleeding, on the Mac, next.** Tell touch from a wheel reliably, and do nothing with touch on the table. The reliable signal is AppKit's, not the page's. Every scroll event macOS delivers says whether its deltas are precise, true for every trackpad and the Magic Mouse and false for a notched wheel. WebKit reads that flag to choose pixel or line scrolling and never exposes it to the DOM, nor the gesture's phase, nor its momentum. What the page sees is deltaY, and macOS accelerates mouse wheels, so one slow notch arrives as a few pixels and looks like the first event of a swipe. A page-side guess would be wrong on exactly the event that flips.

So the plan is a local event monitor in Rust, installed once at setup, that sees every scroll wheel event before the window does and drops the precise ones aimed at a window that asked. The page never receives them, `onWheel` never fires, and no page logic changes. The page's half is one command per window, on or off, which the shell calls from `showView`: on for a table, off for the sheet, which has to keep two-finger scrolling. Behind `#[cfg(target_os = "macos")]` it is a no-op elsewhere, like `associate_register`. No crate is added: `objc2`, `objc2-app-kit`, `objc2-foundation` and `block2` are already in the lock beneath tao, and `menu.rs` already reaches AppKit through them.

What it gets right: a trackpad, a Magic Mouse and the momentum tail are all precise, and a machine with both devices is handled per event, since each event carries its own flag. What it gets wrong, and accepts: smooth-scrolling utilities such as Mos, Mac Mouse Fix and Logitech's Options+ re-post a mouse's notches as precise events, which is how they make scrolling smooth, so a mouse behind one reads as a trackpad and its owner has the keyboard until parity.

Verified when, on the mini, the wheel still flips and control with it still zooms; and on the Air, a two-finger swipe does nothing on the table, scrolls the sheet, and command with an arrow flips. The one assumption a first minute on the mini settles is that nothing in tao or wry takes the scroll event before AppKit hands it to the monitor.

The alternative not taken is a debounce in the page, acting on the first event of a burst and ignoring the rest for a fraction of a second. It needs no Rust and works on every platform, but it changes what a fast spin of a notched wheel does, for the largest cohort, and it still flips once per swipe. It may yet serve Windows.

**Parity.** A simple set of touch commands that reaches everything the wheel does, flip, zoom and gamma, before anything custom. Open, and the questions are: whether a swipe flips once per gesture, which needs the gesture's beginning, which the monitor sees and the page does not; whether pinch zooms, which needs to know if a WKWebView with magnification off dispatches WebKit's gesture events at all; and what a Magic Mouse user, who cannot pinch, gets instead. Rust telling the page what kind of scroll each event was, rather than dropping it, is the likely shape, and the ordering of that message against the DOM event is the thing to verify.

**Beyond.** Every popular device on every platform, done well: pinch to zoom about the fingers, two fingers to pan with momentum, a swipe with a meaning, and the same on a Windows precision touchpad. Chromium hands the page a pinch as control with the wheel and a notch as a multiple of 120 in the legacy field, so the page may classify alone there. Continuous gestures land on the table's quiver, which already takes a distance rather than a count.

## What the engines hand the page

From memory of the engines rather than measured here, which is the first thing this subject should fix. A few lines logging each wheel event's deltas, deltaMode, wheelDeltaY, modifiers, timestamp and webkitDirectionInvertedFromDevice, run on the Air with the trackpad, the mini with the Microsoft mouse and the Windows box, settle all of it in an afternoon.

- **Windows, Chromium.** A notch is a deltaY of 100 on one axis, with a wheelDeltaY that is a multiple of 120. A precision touchpad sends small uneven deltas on both axes, with momentum. A pinch is a wheel event with ctrlKey set.
- **Mac, WebKit.** A notch is 40 pixels times the accelerated line count, so a slow notch is about 4 and a fast spin hundreds. A trackpad and a Magic Mouse send precise deltas with momentum. A pinch is not a wheel event, and a WKWebView does nothing with one unless magnification is on. Natural scrolling inverts the sign for mouse and trackpad together unless a utility splits them, and webkitDirectionInvertedFromDevice says whether it did.
- **Neither** exposes phase, momentum or precision to the page.
