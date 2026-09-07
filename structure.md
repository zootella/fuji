# Structure

What the parts of fuji are called, how many there are of each, and which ones the user swaps between. This is the vocabulary document: read it to learn what a word means before reading a file that uses it.

`architecture.md` says where code lives and why. This says what to call things. Where a name has a research document of its own — `sort.md` for the sorts — that document has the detail and this one just places it.

## The two things the user looks at

**The Sheet shows a folder. A Table shows one image.** That is the whole top-level split, and everything below is a refinement of it.

    Sheet     one of them
    Tables    several of them, one showing at a time

The user calls the Sheet a contact sheet; the code calls it `Sheet`. A Table is one image the way some kind of reader wants to see it.

## What is chosen, and how often

    thing     how many exist      showing at once     how often the user changes it
    Sheet     one                 it or a table       often, between it and a table
    Table     several             one                 rarely
    Sort      several             one                 often, while in the sheet
    Flow      several             one                 often, while in the sheet

**Frequency is the useful column.** It is why moving between the Sheet and a Table has to be instant with nothing reloading, and why choosing a different Table can afford to be slower. `architecture.md` turns that into the `v-show` and `v-if` rule.

## Sorts

**A Sort is an order.** It decides which image is first and which follows which, and nothing else.

**One order, two consumers.** The Sheet lays its thumbnails out in that order, and a Table flips through the same order — so the picture after this one is the same picture whether the user is looking at a wall of thumbnails or at one image. Changing the Sort changes both at once, because there is one sequence and both are reading it. That is why the choice belongs below both of them rather than to the Sheet that offers it.

`sort.md` is the whole subject: how Windows and macOS actually order filenames and dates, which of those fuji reproduces, which it departs from, and what each one costs to build.

## Flows

**A Flow is a thumbnail strategy.** Given a list of images with arbitrary and unrelated dimensions, a Flow decides how each becomes a thumbnail and how the thumbnails are arranged down the scrolling page — the resizing, any cropping, and the resulting layout. Different Flows make genuinely different-looking sheets from identical input.

One worked example, the kind Flickr uses: resize every image to a constant height, then set the thumbnails left to right like words in a left-aligned paragraph, wrapping at the edge. Wide images take more width than tall ones, rows come out ragged on the right, and **nothing is cropped** — every picture is shown whole. A Flow that instead squared everything to a grid would have to crop, and that is precisely the sort of trade a Flow exists to make.

Flows are open: which ones fuji ships, what each is called, and how a Flow describes itself to the Sheet are all undecided. What is settled is that they are a named, switchable family the user picks from, one at a time, the way Sorts are.

## Sorts and Flows are orthogonal, and not equally shared

**They answer different questions, in that order.** A Sort produces the sequence; a Flow arranges the sequence it is given. A Flow sizes and places thumbnails and never reorders them, so any Sort works with any Flow and changing one says nothing about the other.

**But they do not have the same reach.** A Sort is shared: the user sets it in the Sheet, then opens an image and flips, and expects to flip in the order they chose — so every Table is looking at it too. A Flow reaches no further than the Sheet, because arranging thumbnails is the only thing that consumes it and only the Sheet does that.

Whether that asymmetry should show up as two different homes for the two values, or whether both simply live wherever the Sheet's state lives, is not settled. What is settled is the rule that decides it, in `architecture.md`: a value two views need belongs below both of them.

Both survive a restart, by the test in `architecture.md` — the user would be annoyed to reopen fuji and find their order and their layout thrown away.

## Tables

**A Table is named for what makes it different, not for what they all are.** They are all light tables, which is why none of them is called one.

**`DiamondTable`** sizes an image into an invisible diamond on an infinite plane the user pans and zooms around. It is the first one and the one that works today.

**`ComicTable`** is imagined and stubbed: no panning and no infinite plane, every image sized to the full width of the window, running down an ordinary scrollbar. It reads like a PDF viewer, and it is what somebody wants when a folder holds the scanned pages of a document or a comic book.

More will follow, and the shape is meant to make that cheap: a new Table reads the same order and asks the same cache, and no existing Table learns it exists.

## Naming conventions

**A member of a named set is Title Case**, so it is recognisable on sight: `Alphabet` is a member, `alphabet` would be a variable holding one. This covers Sorts, Flows, and the Tables. `sort.md` states the rule where it first mattered.

**A family shares a leading noun** in code, so its parts sort and read together — `cacheNeed`, `cacheRelease`, `cacheFootprint`. `style.md` is the authority on how code is written.

**Singular names a role, plural names the set.** The Sheet is a specific thing; a Table is one of several; Sorts and Flows are families the user chooses within.

## Where the rest is written down

    architecture.md    the four layers, and where a value or a view belongs
    sort.md            the orders, researched and planned
    cache.md           images: what is held, and why the store is deliberately dumb
    performance.md     what any of it costs, measured
    style.md           how the code itself is written
    scaffold.md        how a project like this one is set up
