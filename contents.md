# Contents

The documents fuji is designed in. Each one owns a subject and is the place that subject gets decided, so a question answered here does not get reopened from scratch somewhere else. This file lists them and says what each is for; it is the table of contents `CLAUDE.md` used to carry, kept here so it can grow without the instructions growing with it.

**Read `structure.md` and `architecture.md` before changing anything structural, and `style.md` before the first edit.** Those three come first because the rest assume them: one gives the words, one gives the layers, and one gives the way code is written.

## The three to read first

- **`structure.md`** — the vocabulary. What the parts are called, how many there are of each, and which ones the user swaps between. Read it to learn what a word means before reading a file that uses it.
- **`architecture.md`** — the four layers, and where a value or a view belongs. The shell, the sheet and the tables, the model, the cache — what goes in each, and the rules that keep them separable. It is also where the absent router and store are argued for.
- **`style.md`** — how the code itself is written. Not a general guide but a delta: where this project deliberately differs from prevailing practice, and why. Comment density, reserved signals, guards, factoring. Its `ttd` rule matters before anything else — those comments belong to the user alone.

## The subjects

- **`cache.md`** — images: what is held per path, and why the store is deliberately dumb. Three designs failed before this one, and the reasoning for each failure is here.
- **`performance.md`** — what any of it costs, measured, with the instrument that measured it. Three findings so far, including the one where a cache reported perfect behaviour while the app got slower.
- **`card.md`** — the box of thumbnails the sheet's scroll runs over, why fuji is putting a level there that no file manager shows, what the experiment it hosted found, and the walk through a whole drive in constant memory that the box makes possible.
- **`canvas.md`** — what WebKit on the Mac and Chromium on Windows actually do with a canvas, read from their sources: when one is GPU-backed, when it is a layer of its own, what it costs and can never give back, what makes a scaled thumbnail look right, and what colors it can hold. Also where a thumbnail should be made at all, in the web layer or by the operating system, and the direction fuji has taken.
- **`thumbnail-open.md`** — what is left to decide, build and measure about thumbnails. Short on purpose: the pipeline itself has moved to the site, and this is the list of what that page does not get to claim yet.
- **`fidelity.md`** — whether a picture arrives with its pixels and its colors intact. The three pixel units a Retina Mac works in and the two a Windows box works in, the Display P3 path measured end to end, and the one-device-pixel defect that was making half of all thumbnails resample. Measured on hardware rather than reasoned, on two machines that say which numbers are whose, and the record of the audit that found and fixed that defect.
- **`security.md`** — where untrusted bytes are parsed, why the web engine's sandbox is not protecting fuji as things stand, and the walls to build, in order.
- **`associations.md`** — becoming an application the operating system will hand a picture to. What macOS and Windows each offer, what Tauri writes on fuji's behalf, why being available and being the default are different questions, and the plan for the first pass: declare every type fuji can show, take none of them, and leave the choosing to the operating system's own interface.
- **`instances.md`** — one window or several, and what happens when the user opens a second picture while the first is still on screen. A whiteboard: the goal is a document-centric feel over a single process, and the hard part is that two webview windows share no JavaScript.
- **`sort.md`** — the orders fuji will show a folder in. Half prior art, researched rather than remembered — how Windows and macOS actually sort names and date files — and half plan. Nothing in the plan is written yet.

## The project

- **`scaffold.md`** — how a project like this one is set up, start to finish, in the form used to stand up a sister client.
- **`icon.md`** — the application icon: fuji's design, and what each platform expects an icon to be.

## Letters

A letter is written to a specific reader — usually a Claude Code session on the other machine, or on the other side of a long gap — and it starts a conversation rather than settling a subject. Read a letter, then talk; do not treat one as a document of record the way the files above are.

None open right now. The last was `windows.md`, written for the Windows machine and retired in September 2026 once it had been answered — everything in it that outlived the conversation went into `icon.md`, `fidelity.md`, `performance.md` and `CLAUDE.md`, which is what a letter is supposed to leave behind.

## What has moved to the site

A planning document is written while something is being built and is finished the day it is. The ones worth keeping do not get deleted — they get rewritten as engineering documents on the website, in `site/docs/`, where they are addressed to a reader who does not have this repository open. What stays behind, in a file here, is only the part that is still open.

- **`site/docs/thumbnail-pipeline.md`** — how a path becomes a thumbnail: the fork between an img tile and a canvas, between the operating system's decoder and the page's, and what was measured to choose each one. Speed, and fidelity of size, sharpness and color. It replaced the planning document written while the pipeline was being built, and `thumbnail-open.md` is the remainder.

**Code refers to these pages and never to a planning document.** A planning document churns and is eventually retired, so a comment pointing at one goes stale or dangles; a page on the site is where an explanation has settled. A comment that needs the long version says "the thumbnail pipeline document on fuji's site" and stops there.

## What is not here

Private planning is kept out of the public repository by the `hide` naming pattern that `.gitignore` describes — `hide/`, `*.hide`, `hide.*`, `*.hide.*`. Those files hold the roadmap and the user stories, they are where an idea lives before it is adopted, and this list does not enumerate them. A planning document in this list holds work the user has adopted; anything still being decided belongs in conversation or in a private file until it is.

`README.md` is the repository's front door — what fuji is, and how to build it on each platform — rather than a design document.

## The shape of the repository

The documents above live at the repository root, which is a pnpm monorepo. The application is the `desktop` workspace, so a path a document writes as `src/` or `src-tauri/` is relative to `desktop/`. The website for fujidesktop.app is the `site` workspace, and a finished document eventually becomes a page there — the section above lists the ones that have.

`notes/` is a plain folder rather than a workspace, and it is the raw material rather than a document of record: the four `fuji` text files the roadmap was distilled from in August 2026, and a few early sketches of brand and pointer work. Nothing imports it and nothing builds it, which is exactly the rule — a directory holding a `package.json` is a workspace, and everything else is just a folder.

**Keep notes out of `desktop/src/`, where these files used to live.** Tailwind's Vite plugin finds its own source files by scanning the project rather than by following imports, and it reads plain text as readily as markup — so a `.txt` file in the source tree contributes class names to the shipped stylesheet whether or not anything renders it. Measured on 2026-09-09, moving this folder to the root removed seven rules from the production CSS: four from an HTML snippet pasted into `marble1.txt`, and three — `grid`, `grayscale`, `rounded` — read out of ordinary English sentences in `fuji1.txt` and `fuji3.txt`. Only 390 bytes, but they were rules for classes no component uses, and nothing would ever have reported them.
