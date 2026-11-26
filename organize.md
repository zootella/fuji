# Organize

## Linux walk fix

`pnpm walk` in `linux` builds fuji once in debug and walks the sandbox process in containers of several distributions, but each container copies `rust/copper/walk.js` and its `package.json` without installing any packages, so the walk stops at once on `@zootella/ergo`, which `walk.js` has imported since commit `dc6e4c8`. The containers need copper's packages before the walk runs. One way is to add `npm` to the packages each container installs, and run `npm install --omit=dev` in `/work/rust/copper` before `node rust/copper/walk.js`.

## Dark grays

Each gray named for its job has its dark half at the distance from the ground that Zed set against its own dark ground, `#2c2c2c` with its tint out, while the face now follows the file managers and is darker. So every dark line, panel and quiet text stands 14 steps farther from the face than Zed tuned it; a border is 45 steps above the face where it was 31. Whether the other dark grays follow the face down. The grays of a selection and the scrollbar came from the platforms and are apart from this.

## TestFlow refactor

`TestFlow.vue` holds application logic in a Vue file: it decides per file whether a thumbnail comes from the operating system or from fuji's own JavaScript, runs its loops at different widths, and stops them when a bucket goes away. By the rule in `architecture.md` that a component is a thin view over plastic, that logic moves into plain JavaScript in `plastic`, leaving the Vue file to place and draw.
