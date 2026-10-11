# Components

Fuji's list and tree, the two views every file manager has beside its icons, and the components they are built from. Begun 2026-10-11 on the Mac mini, with nothing built. The article comes first: the List and Tree page on the site is written section by section before any Vue, and the code is built to match it.

## The history section of the article

The article opens with history, and the history is written first, so our own work has its context before anything is designed. The tale it tells is an argument that is still going: that as software on a graphical desktop is built from components, the common ones should look and work the same everywhere, to the benefit of the user, who meets a list in a second program and already knows it from the first, and of the developer, who takes a list off the shelf instead of writing one. The section is where that argument has been fought and what each side built, and it ends on the form the argument takes today, where the code a developer writes becomes the thing the user sees and touches. Every claim links its source inline, Wikipedia and otherwise, on pages that will stay put, and every link is fetched before the page says it. A patent links to its page at Google Patents. A book links to a page where a reader can find the book itself; which page is open.

**The section is short.** The research underneath it is deep and broad so that the right subjects get chosen and said correctly, not because the page needs that depth anywhere; the finished section is a fraction of what was learned.

Three parts, and the players in each. Whether these are the right players and the right division is the first research question, and the section is written from what turns up rather than from this outline.

**The desktop vendors.** Briefly the genesis at Engelbart and Xerox PARC, then the Mac of the System 9 era and before, and Windows' list view and tree view in Win32 from Windows 95 to Windows 11.

**The cross-platform libraries and initiatives.** The mainstream attempts beyond the operating system vendors, ambitious or well funded, whether or not they found the market they imagined, including the ones that settled into a niche their makers meant as domination. Java Swing ignored the host platform and imagined users wanting one consistent Java look; the Eclipse alternative did the opposite, matching the native look exactly by driving the actual native components. The stories wanted here are real and researched ones of that kind, what a look was modeled on and who ever saw it, written up from the history as it happened.

**The web generation.** Facebook as the hot young company with the best new graduates, the web as the dominant consumer platform about to be overtaken by mobile, and engineers and designers still after reusability: React and React Native, and components for lists and trees. Then the idea thins out, because the touch interface of a phone is not well served by lists and trees and the components that drive them get less attention. But "the code I write becomes what my user interacts with visually" stays alive in every component library, React, Vue and Svelte, and that ongoing argument about the right way to program the problem is the center of the section.

Build up incrementally
Keep simple
Use tailwind only if it helps
Be as feature rich as native but with a clean platform neutral visual look, feel is familiar while appearance is simple and modern
Light and speedy even with 10k items
Npm module @zootella/components

Icons
Sort
Update when pointer is away
Same highlight color as system
Same dotted keyboard focus as system
Filter, instant

List
List with Columns
Tree, with empty folder
Coolbar, menus like Microsoft 2003

Media metadata hangs on right margin, this is a really cool idea you had years ago and could now do, Fuji craftsmanship article
Color coded sizes, 1-4 places, ibibytes

Rename with up down arrow keys
Shift control selection keyboard focus
