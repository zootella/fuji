# Organize

What is left of putting fuji's folders and their contents in place.

- **The Linux walk.** Its containers copy `rust/copper/walk.js` without installing its packages, so the walk stops at once on `@zootella/ergo`, which walk.js has imported since commit `dc6e4c8`. The containers need copper's packages before the walk runs.
- **TestFlow.vue's logic.** `TestFlow.vue` holds application logic in a Vue file: it decides per file whether a thumbnail comes from the operating system or from fuji's own JavaScript, runs its loops at different widths, and stops them when a bucket goes away. By the rule that a component is a thin view over plastic, look at moving that logic into plain JavaScript in `plastic`, leaving the Vue file to place and draw.
