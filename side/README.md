# Side

What doesn't ship: tools that help make and check Fuji but are neither part of the app nor needed to run or build it. Nothing here is built into Fuji, so a reader working on the app can leave this folder alone, and a tool here can be as large as its job asks, as long as it stays well factored and documented.

- `palette/` shows Fuji's colors on each platform and in light and dark at once, beside the candidates weighed for them.
- `sort/` holds Fuji's sorts against Windows and the Mac, and keeps the exact Windows sort Fuji could ship instead of its own.

Each folder's README says what the tool is, why it exists, how to run it, what it found and what it needs. `side` is one pnpm workspace, so the tools share one `package.json` and run from this folder, and each reaches into `desktop` by path for what it draws or tests.
