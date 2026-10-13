<script setup>//the About section at the end of fuji's settings: fuji's version, the parts it is made of with the version of each, the site, and the mark

import {ref, onMounted, version as vueVersion} from 'vue'
import {getTauriVersion} from '@tauri-apps/api/app'//the framework asked for its own version, which the page may do for its own window
import {brandName, brandVersion, urlHome} from '../brand.js'
import {windowEngine} from '../window.js'
import {mpvVersion} from '../mpv.js'
import {processOpen} from '../process.js'//the site in the system's browser, rather than in the web view
import {logTrouble} from '../log.js'
import {platform} from './library.js'

//the parts fuji runs on, each with its version asked of the part itself where it can answer at runtime, and read as this copy was built where it cannot: the framework and the web engine answer through rust, the media library answers for itself and for the FFmpeg beneath it, and vue's is the one its package exports. A line per part, the name quiet and the version strong, so the numbers read down the lines. The engine's name is fuji's word, since the framework answers only the number and which engine it is never changes on a platform

const engineLine = {//the engine's line less its version: its name and its project's page, per platform
	mac:     {name: 'WebKit',    url: 'https://webkit.org'},
	windows: {name: 'WebView2',  url: 'https://developer.microsoft.com/en-us/microsoft-edge/webview2/'},
	linux:   {name: 'WebKitGTK', url: 'https://webkitgtk.org'},
}[platform()]
const homeLink = `https://${urlHome}`//the site, https://fujidesktop.app, both where the link goes and its words
const projects = {Tauri: 'https://tauri.app', Vue: 'https://vuejs.org', libmpv: 'https://mpv.io', FFmpeg: 'https://en.wikipedia.org/wiki/FFmpeg'}//each part's page, which its name links to; FFmpeg's is the encyclopedia's rather than the project's own
function visit(url) { processOpen(url).catch(error => logTrouble(`about: opening ${url}`, error)) }//the system's browser, which may refuse; the top gate for a click

const rows = ref([])//{name, version, url} per line; what is known at once first, and the list is rebuilt in its order as each part answers
let tauri = '', engine = '', media = null//the answers, blank until they come
list()

onMounted(() => {
	ask(getTauriVersion(), 'asking tauri its version').then(text => { tauri = text; list() })
	ask(windowEngine(), 'asking the web engine its version').then(text => { engine = text; list() })
	ask(mpvVersion(), 'asking the media library its version').then(answer => { media = answer; list() })
})
function ask(promise, where) { return promise.catch(error => { logTrouble(`about: ${where}`, error); return String(error) }) }//an answer or the trouble, both shown, since a part that cannot say is itself something to see
function list() {//the lines in their fixed order, with whatever has answered so far
	let found = [{name: brandName, version: brandVersion, url: homeLink}]
	if (tauri) found.push({name: 'Tauri', version: tauri, url: projects.Tauri})
	if (engine) found.push({...engineLine, version: engine})
	found.push({name: 'Vue', version: vueVersion, url: projects.Vue})
	if (typeof media == 'string') found.push({name: 'libmpv', version: media, url: projects.libmpv})//the trouble, in the library's one line
	else if (media) found.push({name: 'libmpv', version: media.mpv.replace(/^mpv\s+/, ''), url: projects.libmpv}, {name: 'FFmpeg', version: media.ffmpeg, url: projects.FFmpeg})//the library names itself at the start of its text, and the line already does
	rows.value = found
}

const box = ref(null)
function show() { box.value?.scrollIntoView({block: 'start'}) }//the panel's root scrolls, so this scrolls it; About Fuji in the Mac's application menu asks

defineExpose({show})

</script>
<template>

<div ref="box"><!-- at the end of the settings, where About Fuji in the Mac's application menu brings the panel scrolled to; the system's own About panel is gone, so this is the one place fuji says its version -->
	<h2 class="mb-2 text-strong">About {{brandName}}</h2>
	<p v-for="row in rows" :key="row.name" class="text-quiet"><a :href="row.url" class="hover:underline" @click.prevent="visit(row.url)">{{row.name}}</a> <span class="text-strong">{{row.version}}</span></p><!-- the name quiet and the version strong, so the numbers read down the lines; each part's name is the link to its project's page, underlined only under the pointer -->
	<p class="mt-4"><a :href="homeLink" class="underline" @click.prevent="visit(homeLink)">{{homeLink}}</a></p><!-- the system's browser opens it rather than the web view navigating away from fuji -->
	<svg class="my-8 fill-brand" width="360" height="360" viewBox="0 0 360 360" :aria-label="brandName"><circle cx="180" cy="180" r="180" /></svg><!-- fuji's mark: the disc that is the application icon, the site's favicon and the README's banner, at the README's size, 360 CSS pixels across; in the brand color by its palette name, which is the same in light and dark. Twice the panel's usual space above and below, the disc's alone, so the line after it sets none of its own -->
</div>

</template>
