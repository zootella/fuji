# Router and Store

Fuji has [Vue Router](https://router.vuejs.org/) 5 and [Pinia](https://pinia.vuejs.org/) 4 installed and idle — one route, zero stores. They are here so the shape is settled before anything is built on it, and the cost of having them sit unused is about 10 kB gzipped in a bundle loaded from local disk.

This file is the standing answer to two questions: *should this be a route?* and *does this belong in a store?* Take the answer and get back to building. The mechanics below matter only when you are editing the router or writing the first store.

## The route rule

**A route is for a destination that should be destroyed when the user leaves it.** If leaving should preserve what was there, it is not a route — it is a mode, and modes swap with `v-show` while staying mounted.

Fuji's lightbox and its coming file-manager view are **modes**. Clicking a thumbnail should land in the lightbox on that image with the folder listing, the index, the decoded triad, and the pan all continuous. They swap with `v-show`, both stay mounted, and that stays true now that the router exists.

The deciding question, when a case is unclear, is what the destination would be addressed *by*. Ask what the route for "image 47 of this folder" would be: either the image is encoded in it, and every flip becomes a navigation that puts hundreds of entries on the history stack, or it is not, and the route carries no information. Neither is good, and that is the signal that a view is a mode. A settings screen has no such problem — it is one place, addressed by its own name.

**Settings, or tabs holding genuinely separate work, are destinations.** Those become routes, and that is what the router is here for.

**The two compose.** `v-show` handles modes inside a destination; routes handle destinations. If a routed view ever needs to survive navigation after all, wrap the outlet in `<KeepAlive>` — that is a one-line change to `App.vue` and gives routing's naming and history without the unmounting.

**Fuji's "back" belongs to the model, not the router.** A viewer eventually wants to return to the image or folder it was just on. That is history over the domain, and Vue Router is the wrong container for it. Keep it as its own stack.

## The store rule

**A store holds facts that (1) outlive the component displaying them, (2) more than one part of the app reads, and (3) benefit from being watched.** All three clauses, and the third is the one usually skipped.

Below that bar, reach for a plain module first. An exported module-scope `ref` is already a singleton that outlives every component, is importable anywhere, and already survives component hot reloads. What a store adds over it is `acceptHMRUpdate`, `$subscribe`/`$patch`, the devtools timeline, and a convention legible on sight — worth having for the app's shared record, not for a value one component reads.

### What must never go in a store

Fuji's core rendering state — `quiverA`, the triad, the drag object — is mutated on every `pointermove` and is **deliberately non-reactive**. The quiver's whole design is to bypass Vue: it computes styles and writes `element.style` directly, because a vdom diff on the pan path would be strictly worse than an assignment. A reactive proxy there would pay trap overhead sixty times a second and, worse, invite someone to bind it into a template and reintroduce exactly the cost the quiver exists to avoid.

The line already drawn in `LightTable.vue` — reactive refs for HUD text and visibility, plain assignment for transforms — is the same line. A store lives strictly on the reactive side of it.

### The first store should be the image cache, in two pieces

Fuji will want a durable cache of decoded image data that manages its own footprint and is what the rest of the app calls instead of reading the disk itself. Build it as two pieces, not one:

- **A plain module holding the mechanism** — the map from path to entry, eviction, footprint accounting, the refcount that keeps an image alive while something displays it, and the release of whatever handle the decode produced. This is an algorithm, it is the part most likely to be wrong, and as a plain module it can be tested without mounting anything.
- **A thin store as its reactive façade**, exposing only what the interface watches: bytes resident, entry count, and the load status of the current key.

The payload is handed out as a value and never observed. A decoded bitmap or a large buffer inside `reactive()` is the bug this paragraph exists to prevent; if a reference must live in the store, it belongs in `shallowRef`. Anything that accumulates needs a bound written in from the start — this process runs for weeks, and a cache without eviction fails slowly enough that no short test will show it.

The same two-piece shape is the default for anything else arriving later with real machinery behind it.

### What does not transfer from the web

Pinia's per-request instancing exists for server rendering; fuji opens one window once, so `createPinia()` is called a single time in `main.js`. And `localStorage` is the web's durable layer only because a page has no other — fuji has a filesystem behind the Rust core, so neither `pinia-plugin-persistedstate` nor `@tauri-apps/plugin-store` should arrive without a reason that survives that sentence.

## Mechanics worth knowing before you edit

**Hash mode is not optional.** History mode writes real paths and expects a server to answer a request for one on reload; a built Tauri window resolves paths against bundled assets with no fallback. The sharp edge is that the **Vite dev server does fall back**, so history mode would work through all of development and break only in the shipped app.

**An empty outlet means a hidden window.** The window is created `visible: false` and nothing reveals it except `LightTable`'s own `onMounted` calling `revealWindow()`, so an outlet that renders nothing is not a blank page — it is a process with no window on screen and no way to report it. The catch-all route is what closes this: with `/` plus a catch-all redirect there is no URL that matches nothing, and the eager `/` import means no chunk can fail between launch and the reveal. Keep both. If the reveal ever moves into `main.js`, this stops being load-bearing.

**A hidden view cannot be measured.** `display: none` destroys the layout box and `<KeepAlive>` detaches the DOM subtree, so `clientWidth` reads **0** either way. `dimensionStart()` depends on that measurement, so any view that measures the viewport must do it while visible. `content-visibility: hidden` skips rendering while preserving layout and is worth testing in the dev window if the mode swap ever needs measurement to stay valid.

**`onActivated` does not replace `onMounted`.** Under `<KeepAlive>`, `onMounted` still fires exactly once on first creation, `onActivated` fires right after it and again on every re-entry, `onDeactivated` fires on the way out, and `onUnmounted` only if `:max` evicts. One-time setup stays in `onMounted` and correctly never re-runs.

**A mounted view stays reactive** — a hidden view can keep doing render work nobody sees. Fuji is largely immune because its hot path is imperative, which is the quiver design paying off somewhere nobody designed it for.

## What is installed today

- `pinia` ^4.0.3 and `vue-router` ^5.2.0.
- `createPinia()` and the router installed on the app in `src/main.js`. **Zero stores** — the first one should be the cache above, not a placeholder written to prove the wiring works.
- `src/router/index.js` holds one real route, `/` to `LightTable`, plus the catch-all. `src/App.vue` is nothing but the outlet.
- Bundle cost, measured: **91.21 kB → 117.87 kB raw, 33.43 kB → 43.60 kB gzipped.** Devtools machinery is absent from the production build; one chunk, no lazy loading.
- Two transitive packages arrive with Pinia and are worth recognizing rather than rediscovering: `@vue/devtools-api` and `nostics`, a small dependency-free library Pinia 4 depends on at runtime.
