//./src/router/index.js

import {createRouter, createWebHashHistory} from 'vue-router'
import LightTable from '../components/LightTable.vue'

/*
The router is here before fuji needs it, holding a single route, so that the day a second destination arrives it arrives as one line rather than as a refactor. Read router-and-store.md for why it's installed early and what it's expected to carry; this comment covers only what a reader editing this file has to know.

Hash mode, not history mode. History mode writes real paths like /settings and expects a server to answer a request for that path on reload. A built Tauri window resolves paths against bundled assets with no fallback, so such a reload would find nothing — but the Vite dev server does fall back, so history mode works all through development and breaks only in the shipped app. Hash mode keeps the whole route after a #, which a browser resolves locally and never requests, and removes that class of bug. The user never sees it: the window has no address bar.

The rest of this file is shaped by one local fact that makes an ordinary web-app mistake expensive here. The window is created hidden — tauri.conf.json sets visible false — and nothing reveals it except LightTable's own onMounted calling revealWindow(). So an outlet that renders nothing isn't a blank page the user can navigate away from; it's a process running with no window on screen and no way to say so. Two choices follow. The route for / imports its component directly rather than lazily, because a chunk fetched at startup is one more thing that can fail between launch and the reveal. And the catch-all below sends anything unrecognized home, so a stale or malformed hash — left in the webview by a dev session, or arriving from anywhere else — lands on the light table instead of on nothing.

One rule for what belongs here, since the answer isn't obvious in a viewer. A route is for a destination that should be destroyed when the user leaves it. Fuji's lightbox and its coming file-manager view are not that: clicking a thumbnail should land in the lightbox on that image with the folder listing, index, decoded triad and pan all continuous, so those two swap with v-show inside this one route and stay mounted. Settings, or tabs holding genuinely separate work, are the other kind, and those become routes. The two compose — wrapping the outlet in <KeepAlive> keeps a routed component alive if one ever wants both.
*/

const router = createRouter({
	history: createWebHashHistory(),
	routes: [
		{path: '/', name: 'lightTable', component: LightTable},//the light table is the whole app today, imported up front so nothing is fetched between launch and the window's reveal
		{path: '/:pathMatch(.*)*', redirect: '/'},//anything unrecognized goes home rather than rendering nothing, which here would mean a hidden window and a silent failure
	],
})
export default router
