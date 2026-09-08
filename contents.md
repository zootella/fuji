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
- **`card.md`** — the box of thumbnails the sheet's scroll runs over, why fuji is putting a level there that no file manager shows, how two of them side by side become an experiment, and the walk through a whole drive in constant memory that the box makes possible.
- **`canvas.md`** — what WebKit on the Mac and Chromium on Windows actually do with a canvas, read from their sources: when one is GPU-backed, when it is a layer of its own, what it costs and can never give back, what makes a scaled thumbnail look right, and what colors it can hold. Also where a thumbnail should be made at all, in the web layer or by the operating system, and the direction fuji has taken.
- **`thumbnail-plan.md`** — fuji has a path and needs a thumbnail: how it chooses between the page and the operating system, the one flow that replaces TagFlow and CanvasFlow, and the steps in order.
- **`security.md`** — where untrusted bytes are parsed, why the web engine's sandbox is not protecting fuji as things stand, and the walls to build, in order.
- **`sort.md`** — the orders fuji will show a folder in. Half prior art, researched rather than remembered — how Windows and macOS actually sort names and date files — and half plan. Nothing in the plan is written yet.

## The project

- **`scaffold.md`** — how a project like this one is set up, start to finish, in the form used to stand up a sister client.
- **`icon.md`** — the application icon: fuji's design, and what each platform expects an icon to be.

## Letters

A letter is written to a specific reader — usually a Claude Code session on the other machine, or on the other side of a long gap — and it starts a conversation rather than settling a subject. Read a letter, then talk; do not treat one as a document of record the way the files above are.

- **`windows.md`** — from the macOS session that did the icon work, to whoever picks it up on Windows.

## What is not here

Private planning is kept out of the public repository by the `hide` naming pattern that `.gitignore` describes — `hide/`, `*.hide`, `hide.*`, `*.hide.*`. Those files hold the roadmap and the user stories, they are where an idea lives before it is adopted, and this list does not enumerate them. A planning document in this list holds work the user has adopted; anything still being decided belongs in conversation or in a private file until it is.

`README.md` is the repository's front door — what fuji is, and how to build it on each platform — rather than a design document.
