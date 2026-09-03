<script setup>//./src/App.vue

import Shell from './components/Shell.vue'

/*
Fuji has no router, and this file is where one would go.

Routing is a browser thing, and a browser is two things: a rendering engine, and a shell for navigating documents — an address bar, a back button, bookmarks, reload, sharing. Tauri keeps the engine and throws that shell away, and every service a router sells belongs to the half that is gone. There is no address bar, so a route name would be invisible. There is no back button unless fuji draws one, so there is no history stack fuji has to stay consistent with. The assets are on local disk, so a lazily loaded chunk buys nothing and adds a way to fail between launch and the window's reveal. There is no server, so nothing renders on one.

The test for a route is that a destination should be destroyed when the user leaves it. Fuji's images are the opposite: flipping changes what is inside a view that survives it, and a wheel flick would push fifty entries onto a history stack in a second. Its folders are closer, being deliberate and worth retracing — but back has to move through folders and images as one sequence, so that stack lives in the model where it can hold both.

There are shapes fuji could grow into where a router would earn its place; architecture.md lists them, and keeps that list in one file so the two cannot drift apart.
*/

</script>
<template>

<Shell />

</template>
