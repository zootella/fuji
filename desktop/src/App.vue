<script setup>

import {ref, onMounted} from 'vue'
import parse from 'path-browserify'
import {windowNote} from './window.js'
import {openFiles} from './open.js'
import {forwardize} from './components/library.js'
import {fileTypesWindow} from './fileTypes.js'
import Shell from './components/Shell.vue'
import VideoWindow from './components/VideoWindow.vue'

/*
Fuji has no router, and this file is where one would go.

Routing is a browser thing, and a browser is two things: a rendering engine, and a shell for navigating documents — an address bar, a back button, bookmarks, reload, sharing. Tauri keeps the engine and throws that shell away, and every service a router sells belongs to the half that is gone. There is no address bar, so a route name would be invisible. There is no back button unless fuji draws one, so there is no history stack fuji has to stay consistent with. The assets are on local disk, so a lazily loaded chunk buys nothing and adds a way to fail between launch and the window's reveal. There is no server, so nothing renders on one.

The test for a route is that a destination should be destroyed when the user leaves it. Fuji's images are the opposite: flipping changes what is inside a view that survives it, and a wheel flick would push fifty entries onto a history stack in a second. Its folders are closer, being deliberate and worth retracing — but back has to move through folders and images as one sequence, so that stack lives in the model where it can hold both.

There are shapes fuji could grow into where a router would earn its place, and none of them has arrived.

What this file does decide is which kind of window it is in, once, as the page mounts, and that is not a route: nothing navigates, and a window stays the kind it was made. It reads the two things a window can be made with, the note a page asked for it with and the files the operating system handed over, and hands the files to the root it chooses. A browsing window gets the shell, which shows the preview, the contact sheet, the light table and the panels one at a time; a video window, which a page asked for with the note Video or the operating system opened on a video, gets a root of its own, since it shares none of a browsing window's state and stands beside one rather than taking its place.
*/

const kind = ref('')//Browse or Video, blank until the window has said what it was made for, so neither root mounts first and has to be taken down
const opened = ref([])//the files the operating system handed this window, forwardized, for the root to show; empty on an ordinary launch and for every window a page asks for
onMounted(async () => {
	let note = ''
	try { note = await windowNote() } catch (error) { console.error('app: asking what this window was made for', error) }//a top gate with no log yet, since the log starts inside the root; a window that cannot say goes on as a browsing window, which reveals itself either way
	try { opened.value = (await openFiles()).map(forwardize) } catch (error) { console.error('app: asking what this window was opened with', error) }//forwardized here, at the same boundary a dropped path crosses, since windows hands these over with backslashes; the same top gate, and a window that cannot say goes on as an ordinary launch
	let extension = parse.extname(opened.value[0] || '').toLowerCase()//the first file decides, since one window shows one thing
	kind.value = note == 'Video' ? 'Video' : fileTypesWindow(extension)
})

</script>
<template>

<Shell v-if="kind == 'Browse'" :opened="opened" />
<VideoWindow v-else-if="kind == 'Video'" :path="opened[0] || ''" />

</template>
