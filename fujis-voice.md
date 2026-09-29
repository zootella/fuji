# Fuji's voice

How the website is written: what Fuji sounds like, what it is about, what its three sections are for and how each grows, and the habits that spoil a page. This is `style.md` for `site/docs/` rather than for the code. Read it before writing or editing a page there, and read `style.md` too, since the two share a reader and most of an attitude.

Written 2026-09-29, at the end of the session that wrote the File Types and Gamma pages and rewrote both of them twice, which is where every rule below was learned.

## What Fuji is, and so what the site is

Fuji is a multimedia file manager, and it is a love letter to the years personal computers learned to show color and then to play video, and to the collections people made once they could: pictures pulled down from a BBS, clips off a CD-ROM, songs from Napster, and on through the forums, imageboards, Reddit and Discord that keep the same habit today. Its subject is the history of photography, communications technology and personal computing, leading up to the era its reader remembers best, and it is also a cross-platform application built to exact specifications for showing those collections without dropping a pixel or a frame.

So the site is several things at once, and deliberately. It is a software manual. It is a set of research papers into history. It is a collection of engineering essays with measurements in them. It is the home of an anime cat-girl mascot with one foot in reality. A reader who pauses to ask which of these it is should find that the answer is all of them, mixed, steadily growing, and that the mixture is the point. The reader should always be curious, always learning, and always able to trust what the page says.

**The test for what belongs.** Picture the homebrew computer club of the first multimedia decade meeting again with every hard drive its members ever owned, magically restored. What is on those drives, and what those people would want to know about it, is what the site covers. A format a collector would have to look up is the long tail and stays out; a format one kind of collector has thousands of belongs, even if that collector is rare. The same test decides a page: a subject earns one when a reader from that room would sit down and read it.

## The three sections

**User Guide** is how to use Fuji, and it showcases the inventions in its design: the diamond table, viewport persistence, the contact sheet's cards, the gamma key. Two currents feed it and mix, the way two feed Craftsmanship, and both are printed matter that comes with a thing someone has just bought, a computer, a phone, a car. One is the glossy brochure, which is terse, callouts and bullets, and touts the beautiful, functional design choices: why the picture is sized the way it is, what that lets you do that other viewers cannot. The other is the manual in the glove compartment, which is longer and plainer and says what the buttons and levers are and how to turn on the windshield wipers: which key, which drag, what happens next. A page here is both, in Fuji's voice, and it is short. It does not go into the software engineering; the mechanism underneath belongs in a paper, and a User Guide page points at the paper when there is one. Neither current runs alone. A page that only lists keys is the help panel again, and a page that only admires is an advertisement. The section grows one page per feature that has something to say for itself. Today it holds the download page and Diamond Table, which is the worked example.

**Craftsmanship** holds the papers, and two currents feed it too: the history of pictures, sound and personal computing, and quality engineering with measurements behind it. Each one runs from the history of pictures, sound or personal computing into how Fuji builds the thing and what was measured to decide it, and the section is ordered from the most engineering to the most historical: Thumbnails, then Gamma, then File Types. The heading names what they share, attention to detail, not dropping a pixel or a frame, rather than their form, and it holds even though history and engineering are mixed together in every one of them, intentionally. A paper is not brittle to Fuji's current abilities. File Types never says what Fuji opens; Gamma tells a hundred and thirty years before it reaches the `G` key, and its finished center, the equation, would stand if Fuji did not exist. The engineering half of a paper is written from the source as it is, with the code quoted rather than remembered, and it says which machine each number came from. A paper grows when a subject carries real research, measurements, alternatives weighed, pictures of the defect, and most work does not; a page nobody needed is worse than none. When one is warranted, the planning document that produced it is boiled down afterward, as `CLAUDE.md` describes.

**About** is the vibe. Today it holds the mascot, Aki, and it is deliberately fun, with one foot in reality and the other in the story. It grows with whatever says who Fuji is rather than what it does. It is the one section where the voice can wink.

The home page stands outside all three and links to the first finished document in each.

## The voice

Simple, pert, knowledgeable, friendly. Fuji explains things and presents facts in a straightforward way. It knows the history and says it plainly, with the dates, the companies and the machines, and it links to the specification or the account so a reader can go further or just check. It is warm without being chummy. The reader is a collector, an equal, and the page assumes they can read a table and follow an equation.

**Lead with the result.** A page states its finding or its finished equation first, then what each term means, then examples, and does not walk the reader up to it the way a textbook would. Gamma is the worked example: the equation section is the center, and everything before it earns it and everything after it applies it. A page's first sentence is its thesis, and it is a sentence a reader could quote.

**Every claim has a source or a measurement behind it.** A date, a name, a value said from memory is checked before the page states it, and if it cannot be checked it is left out. A draft may carry a note that says *verify*; a published page never does. Every link is fetched before publishing, because a history page with dead links is worse than none, and a link that this machine cannot reach is checked from another route before it is replaced.

**American English**, throughout, as in the code: color, gray, behavior, center.

**Fuji is a proper noun on the site**, capitalized in prose, always. It is lowercase in file names, `fuji.toml`, `fuji.exe`, and in the code and the planning documents by their own convention, and that convention stops at the site's door.

**Link text names what is at the other end.** Never "Wikipedia", never "here", never "the specification" alone. Write "Matroska's history is on Wikipedia" with the history as the link, or "the PNG specification", so a reader scanning the links learns something from each, and a screen reader reading them as a list does too.

**Keyboard keys are code.** `Space`, `F`, `Ctrl`, `Page Up`, `Esc`, `+`, `1` to `6`: a key is written in a code span, capitalized the way its keycap is, so it comes out in the brand color and reads as a thing to press. A combination is words around the keys, `Ctrl` and the wheel, `Shift` with `+`. Mouse actions are bold rather than code, **mouse drag**, **right drag**, the **mouse wheel**, since there is no keycap to quote; the first drag on a page says mouse, and after that right drag is enough. The arrows are their own glyphs, `←` `↑` `→` `↓`.

**Brand feature words.** A growing list, today ***Diamond Table*** and ***Viewport Persistence***. Each is introduced with fanfare where it first appears on a page, and from then on it is always written in bold and italics in prose; in a heading or a title it is plain text. In the application's own help and settings they are plain, since those panels have no italics.

**Fuji's parts by their long names.** The code says sheet and table; the site says contact sheet and light table, because a reader who reads table thinks of the kitchen and a reader who reads light table thinks of the photo room at a newspaper. The Diamond Table page introduces the ***Diamond Table*** as Fuji's light table, so a reader takes the term as a meticulously designed light table rather than a new word. `thumbnails.md` defines the short words in a glossary at its top and then uses them, which is the one page that may.

**Third-party names in the application's panels** are set in single quotes and italics, 'Set defaults by app', 'Paint'. Fuji and "nothing" are not. The site follows the application when it quotes the panel.

## The trouble spots

These are the habits that were written into the two papers and then edited out, named so they are not written again. Each is a way the house voice of a writing assistant leaks into Fuji's.

**Framing sentences that announce content instead of stating it.** "Here is the coincidence Poynton points at." "Here is the cultural moment." "Keep the number; it is the reason the rest of this page exists." "That is the argument for a manual control." A sentence that tells the reader what the next sentence will do is a sentence to delete; state the thing.

**The tics.** *Worth* in any construction, "worth having", "worth typing", "worth knowing", "worth describing". *On purpose* and *deliberately* as a reflex. *Quietly*, *plainly*, *honest*, *the whole story*, *the point*, *in its bones*, *which is what*, *that is why*, *the one thing*, *for a reason*. One of these in a page is fine; the pattern is what reads as a machine.

**The passive voice where the actor is the information.** "The gamma was rounded to 2.2" hides HP and Microsoft. "Seven drafts were done by February" hides the working group. "The decoders were reverse-engineered" hides FFmpeg's developers. Name the actor, the person, the company, the program, the setting, and the sentence gets shorter and says more. The passive is right only when the actor is unknown or does not matter.

**Sweeping claims about the reader's drive.** "No drive from that time on is without a folder of them." "Every drive of the era has a folder of them." "Anyone who shot a Canon has a drive of them." These are untrue, since a computer today may have Spotify installed and no music files at all, and they are the tell of a writer reaching for color. Say what the format was used for and by whom, which is the true part, and stop.

**Hokey color and editorial.** "A telephone underwater." "The most reluctantly installed program of its era." "Jerking along from a CD-ROM." Rich in history, informed and friendly is the target; a simile that draws attention to itself or a judgment about a company is past it. Where a judgment is history, source it: PC World's 2006 list is a fact about RealPlayer, and an adjective is not.

**Clever structure.** "Windows made no such choice, and that was a choice." A sentence built to be admired slows the reader down. Simple beats clever.

**References to planning documents.** A page never points at `performance.md` or any file at the repository root, and neither does a code comment; those documents churn and are burned down within a sprint, and a pointer to one dangles. A page points at another page, or states the fact.

**The distinction between a paper and the notes that produced it.** A page is informative and interesting to somebody arriving later. It does not carry the trail, "first we thought this, then we tried that," except where a failed attempt is the reason the design is what it is. The two-filters bug on the Gamma page earns its place because the mechanism is the explanation; the four values the drag setting passed through on the way to 5 earn one clause.

## The shape of a page

An H1 that is a name of one to three words, since the sidebar's text is its own and can say more. A first paragraph that is the thesis. Sections in the order the subject unfolds, which for a paper is history first, the finished center, then Fuji's own engineering. One to three paragraphs per subject, at the length that says the origin, the use, and where it stands now, and no longer. A Sources section at the end for a paper, naming the two or three references to read first and then the rest in the order they appear. Inline links throughout, on the phrase that names the destination.

Code on the page is quoted from the source as it stands, refreshed when the source changes, and kept to the lines that make the mechanism visible. An equation is MathML in a scrollable box, styled once at the end of the page, as Gamma does it.

A page is written into `site/docs/`, added to the sidebar in `config.js` under its section, and listed in `contents.md` under what has moved to the site, with one line saying what it owns. `pnpm local` in the site workspace shows it live, and a build is not needed to trust it.

## Worked examples

- `site/docs/file-types.md` — a paper that is all history: what belongs by the club's-drives test, one to three paragraphs a format, links named for their destinations, and a closing section on the formats that belong to history and to no list of what a program opens.
- `site/docs/gamma.md` — a paper with a finished center: the equation, with the history in front that earns it and Fuji's mechanism behind it written from the source, and a Sources section that says which three to read first.
- `site/docs/thumbnails.md` — a paper that is mostly engineering, with the measurements, the alternative that was tried, and a picture of the defect.
- `site/docs/diamond-table.md` — the User Guide voice: a cheat sheet, the controls that matter, one diagram, and the signature feature the design exists for, in under a thousand words and without the engineering.
- `site/docs/meet-aki.md` — the About voice, which is allowed to wink.
