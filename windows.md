# Letter to windows claude — two window-behavior changes to verify

From the mac session, 2026-08-24. The user pulled this and will smoke test with you. Read this, then chat with them.

Ground rules for this session, briefly (they're not in your memory): the user runs all mutating git commands — when a change is verified, end your response with 📌 **a suggested commit message** on its own line. Read `./style.md` before touching code; `ttd` comments are the user's alone — never add, reword, or delete one. Files with "hide" as a dot-separated name part are private planning docs, gitignored.

## Change 1: window size and position

The window now starts hidden (`visible: false` in tauri.conf.json), sizes itself at startup to 60% × 80% of the monitor's **work area** (the desktop minus taskbar — `Monitor.workArea`, converted backing→logical via scaleFactor; see `sizeWindow()` in src/components/library.js), then reveals. Position is never set — the OS places the window, deliberately and permanently.

**Smoke test:** launch `pnpm local` — the window should appear once, already sized, positioned by windows. The headline check: **it must no longer clip into the taskbar**, which was the original bug on this machine. Also worth a look: launch a second instance and confirm the OS offsets it.

## Change 2: fullscreen — simple mode and the curtain

Fullscreen now uses `setSimpleFullscreen()` — on windows this is documented as identical to `setFullscreen`, so the mechanism here should be unchanged; the win was on mac (it skips the macOS Spaces fade). New on both platforms: the **curtain** — a `fullscreenCurtain = true` factory preset in LightTable.vue blinks the frame to black through the transition, hiding the one-frame shear that raced the corrective pan. An 800ms failsafe drops the curtain even if the resize event never arrives.

**Smoke test:** drag an image in, pan to a landmark, double-click into fullscreen and Escape out, several times, quickly. Expect: a crisp blink to black, the image landing on the same physical pixels, no visible shear, no stuck black frame. The standing caveat from the last windows session still applies: this machine is 1920×1200 at 100% scaling, so the scaled-pixel path stays untested here.

## Reporting

Small scope — results travel home with the user; no written report needed unless something fails, in which case exact symptoms and console output help.
