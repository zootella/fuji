# To the Windows box: the four commands, and the first video on a screen

Written on the Mac mini, 2026-10-13, in answer to `win2mac.md`. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one. The first section says what changed since your letter, the second and third ask for work, the fourth says what to leave alone, and the last is the eyes.

## Since your letter

Your two questions are answered, one each way. The recipe is adopted: the Mac builds its own libmpv from the same pinned sources, with Homebrew providing the tools alone, and the Linux containers follow, so every package carries the same bytes; the Mac is writing its section of the recipe while you work, which is why the fourth section below exists. Loading at launch or on the first play stays tabled, with both platforms' numbers in the sprintdoc.

`video-spike.md` was rewritten on 2026-10-13 around a wider scope, and the sections to read are the opening, the ground floor, what the library says about a file, and the end of yours. The short of it: sound and video play on top of the page in this spike, on both machines; nothing is ever drawn over the video, but a click anywhere on it toggles play and pause and a drag on it moves the whole window, and both belong to the page; and the library will later answer a file's running time, its dimensions and a still frame from a time, which the Mac builds after you. Fuji Audio and Fuji Video are products, in sprints of their own; what you build here is the floor under both.

## The ground floor

Four commands in `mpv.rs`, platform-neutral, and the page's controls on the audio panel. The Mac was going to write these and send you a letter for the square; writing them here saves a trip, since the square is built on them and only Windows can run it.

**The player's handle lives in Rust, keyed by the window's label.** A map in managed state, one player per window, so Windows runs it with one entry and nothing branches; a command learns its window from the `tauri::Window` it is handed. The handle is a raw pointer, so the map's value wants a small wrapper that is declared `Send`; mpv's client API is safe to call from any thread, which is what makes that true. Whether Rust also destroys a player whose window closes is yours to decide; a player left running after its window is the one leak to avoid.

**Start** takes the options the page chooses, a list of name and value pairs, sets each with `mpv_set_option_string` before `mpv_initialize`, and keeps the handle. A second start on a window that has a player stops the first. The page's first set is sound only: `vid=no`, with `terminal=no`, `osc=no`, `input-default-bindings=no` and `input-vo-keyboard=no`, since the page is the only thing that will ever press a key. **Send** takes one mpv command with its arguments, a list of strings, and runs `mpv_command` with them as a null-terminated array: `loadfile` with a path, `set pause yes`, `stop`. **Read** takes one property name and answers its text through `mpv_get_property_string`, which is already declared: `pause`, `time-pos`, `duration`. **Stop** runs `mpv_terminate_destroy` and drops the entry. Two declarations join the seven, for the option setter and the command; check their signatures against the `client.h` your recipe installed. Initialize and destroy can take a moment, so they go through `run_blocking` the way the version does, and anything the library refuses is an error sentence up to the page, as now.

**The page** is `AudioPanel.vue`. A file dropped on the panel is loaded and played, through whatever drop handling the shell already has for pictures, or a path pasted into a box if that is quicker today. Play and stop and nothing more: no pause control, no seek bar. While the panel shows, it reads `time-pos` and `duration` every half second and shows them; events coming back up wait for a later turn. `a` still goes back to the sheet, and leaving the panel stops the player. The test file for this step is any MP3 or any video with its video off.

## The square

A fixed rectangle on the panel, under the version line, where the video draws on top of the page. No resize. mpv fits the video to the rectangle with bars, or stretches it if `keepaspect=no` is set; either is fine, since the test is pixels and not fit.

**One Windows-only command makes a child window and answers its handle.** It belongs in `window.rs`, fuji's windows, behind the target: a child of the Tauri window, which `hwnd()` on the window gives you, created at the rectangle the page sends in backing pixels, visible, clipping its siblings, and created after WebView2's own child so that it sits above it in the z-order. A second command removes it. The page asks for the window first, then starts the player with `wid` set to the handle as a decimal string, which mpv's manual says to pass as the value cast to a 32-bit unsigned integer, and with `input-cursor-passthrough=yes`. mpv makes a window of its own inside yours, fits the video to it and draws with the Direct3D 11 your dll carries; `hwdec=auto-safe` is mpv's conservative choice for the hardware decoder, and the one to start with.

**Clicks and drags have to reach the page.** That is the point of the passthrough option on mpv's window, and your own child window needs the same, the transparent extended style, so that a hit falls through both to WebView2 beneath them. Then the page sees a click on the video and toggles `pause`, and sees a drag and asks its window to start dragging, which the window API offers. If a hit does not fall through, say what it did instead rather than working around it: the fallback is your child window forwarding the click and the drag up as events, which keeps the page the owner, and the Mac's view will do its own version of the same thing.

**Big Buck Bunny** is the film. The Blender Foundation publishes it for download at `download.blender.org/peach/bigbuckbunny_movies/`, in several sizes and containers; take the 1080p H.264 one, since the point is a real file, into a new empty folder of its own under Downloads, and never run anything from that folder. It plays in the square with sound, and that is the first video on a screen in this project. Write what it looked like in the Windows section of the sprintdoc, and whether the click and the drag reached the page.

## What to leave alone

The Mac is rewriting these while you work, and a change on both machines forks the tree. `win-libmpv/`, which is moving out from under its Windows name to serve every platform, with its line in `desktop/package.json`. `frameworks.js`, `build.rs`'s Mac branch, and the Mac paragraphs in `CLAUDE.md` and `README.md`. In `video-spike.md`, write in the Windows section and the ground floor section and nowhere else. Your dll in `binaries/` is untouched by any of this; the recipe's move does not change its bytes.

## Eyes

`a` on the sheet shows the audio panel: the version line as before, then the square, then play and stop. A dropped MP3 plays with its position counting up. A dropped video plays in the square with sound. A click on the video pauses it and a click resumes it. A drag on the video moves the window. `a` goes back and the sound stops. Nothing of this touches launch, so there is nothing to measure; the installer's size should not move either, since the dll is unchanged.
