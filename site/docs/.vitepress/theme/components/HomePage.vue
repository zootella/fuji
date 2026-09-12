<script setup>//./.vitepress/theme/components/HomePage.vue
import { ref, onMounted } from 'vue'

/*
The apex page. index.md carries layout: false, so VitePress renders no navbar, sidebar, or footer, and this component is the whole document.

It is a port of the Nuxt page it replaces, class for class, now that Tailwind is gone. It names its own fonts rather than reading the theme's variables, so that the documentation pages can stay stock VitePress and the two never pull on each other. The Tailwind values it was written in are noted beside the CSS below so the two can be compared.

The Hashes reveal is the one moving part. Each installer is published with a small JSON sidecar beside it, written by the same command that built it, and the three live in the downloads directory on the server rather than anywhere in this build — so nothing here knows a hash at build time, and publishing a new installer changes what this page shows without the site being rebuilt. It has to be three files and not one: the dmg is built on a Mac, the exe on Windows, the deb on Linux, on three different days, so no single machine ever holds all three hashes to write them into a combined file. update.md carries the longer version of that reasoning.
*/

//the three installers, in the order the page lists them, each beside its sidecar. all three rows always show; a hash appears on the ones that have been published
let installers = [
	{file: 'fuji.dmg', url: '/fuji.dmg.json'},
	{file: 'fuji.exe', url: '/fuji.exe.json'},
	{file: 'fuji.deb', url: '/fuji.deb.json'},
]

let showing = ref(false)//is the hash list open
let rows = ref(installers.map(installer => ({file: installer.file, sha256: ''})))//one row per installer, named from the start so opening the list never changes its height; a hash arrives when its sidecar does
let release = ref(false)//the version and date above the rows, or false before anything is published
let loaded = ref(false)//the three fetches have all settled, so a blank hash now means unpublished rather than unread
let status = ref('')//the line beneath Close: the hover hint, then the copy confirmation

/*
The sidecars are fetched when this component mounts rather than when the reader opens the list, so the hashes are already in hand the moment Hashes is clicked. That costs every visitor three requests they may never look at, which was weighed rather than measured and accepted: the files are a couple of hundred bytes each, they go in parallel, and they are the only thing on this page that is not already in the bundle.

onMounted rather than top-level, because VitePress prerenders this component in Node at build time. Node has fetch, so the call would run rather than fail loudly — but a relative url has no origin to resolve against there, and nothing is serving these files during a build anyway, so every row would bake in as unpublished. Mount is also the only honest moment for it: the whole reason these are separate files is that publishing an installer must change what this page shows without the site being rebuilt.

One consequence of mount rather than load: VitePress navigates between pages on the client, so leaving this page and coming back mounts the component again and fetches again. That is three small requests for a fresher answer, and it is the behaviour we want.
*/
onMounted(async () => {
	let sidecars = await Promise.all(installers.map(installer => _fetchSidecar(installer.url)))
	rows.value = installers.map((installer, index) => {
		let sidecar = sidecars[index]
		return {file: installer.file, sha256: sidecar ? sidecar.sha256 : ''}
	})
	release.value = earliestBuild(sidecars)
	loaded.value = true
})

//open and close the list; the hashes are already in hand by the time anyone can click this
function toggleHashes() {
	showing.value = !showing.value
	if (!showing.value) status.value = ''//collapsing clears the status
}

let monthNames = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December']

//turn the sidecar's "2026-09-09" into "2026 September 9". the sidecar itself stays ISO 8601 for two reasons: earliestBuild below compares dates as text, which only sorts correctly when they are big-endian and zero-padded, and a spelled month is unambiguous where 09-09 would leave a reader guessing whether the day or the month comes first. the parts are split by hand rather than passed to Date, which would read a bare date as UTC midnight and render it as the day before for anyone west of Greenwich
function readableDate(iso) {
	let parts = iso.split('-')
	if (parts.length != 3) return iso//anything shaped unexpectedly shows exactly as it arrived
	let month = monthNames[Number(parts[1]) - 1]
	if (!month) return iso
	return parts[0] + ' ' + month + ' ' + Number(parts[2])//Number drops the day's leading zero
}

//the version and date to show above the rows. the three installers are built on three machines, so there are three dates; take the earliest, which understates how fresh the release is rather than overstating it, and read the version off that same build so the two always describe one real artifact. iso dates compare correctly as text
function earliestBuild(sidecars) {
	let earliest = false
	for (let sidecar of sidecars) {
		if (!sidecar) continue
		if (!earliest || sidecar.date < earliest.date) earliest = sidecar
	}
	return earliest
}

//one sidecar, or false. a missing one is an ordinary answer rather than an error: it means that installer has not been released yet, which is true of all three today
async function _fetchSidecar(url) {
	try {
		let response = await fetch(url)
		if (!response.ok) return false
		let sidecar = await response.json()
		if (!sidecar || !sidecar.sha256 || !sidecar.file) return false//malformed reads the same as missing
		return sidecar
	} catch (error) { return false }//the network refused, or the body was not json
}

//the hash is clickable but not underlined, so the status line carries the affordance
function hintCopy() { status.value = 'Click to Copy' }
function unhintCopy() { if (status.value == 'Click to Copy') status.value = '' }//a Copied confirmation stays standing

//copy one hash to the clipboard, and say so beneath
async function copyHash(row) {
	try {
		await navigator.clipboard.writeText(row.sha256)
		status.value = 'Copied'
	} catch (error) {
		status.value = 'Could not copy'//the clipboard needs a secure context; https and localhost both qualify
	}
}

</script>

<template>

<div class="page">
	<div class="home">
		<div class="row">

			<!-- the logotype and the disc, the whole identity -->
			<div class="mark">
				<h1>Fuji</h1>
				<svg viewBox="0 0 360 360" xmlns="http://www.w3.org/2000/svg">
					<circle cx="180" cy="180" r="180" fill="white" />
				</svg>
			</div>

			<div class="copy">
				<p>A multimedia file manager designed</p>
				<p>with privacy and precision in mind</p>
				<!--
				Downloads on the left of the hyphen, everything else on the right.
				
				The three installer paths are the real ones and are deliberately stable: the build names its output Fuji_0.1.0_aarch64.dmg and the like, and the upload renames on the way out, so a link posted today keeps working across releases. They 404 until the first release is uploaded, which is expected — nothing about this markup changes when it stops being true.
				
				GitHub points at the application's own repository, not this one.
				-->
				<p class="links">
					<a href="/fuji.dmg" download>Mac</a> <a href="/fuji.exe" download>Win</a> <a href="/fuji.deb" download>Linux</a> - <a tabindex="0" @click="toggleHashes" @keyup.enter="toggleHashes">Hashes</a> <a href="https://github.com/zootella/fuji">GitHub</a> <a href="/markdown-examples.html">Docs</a>
				</p>

				<div v-if="showing" class="hashes">
					<p class="heading">SHA-256<span v-if="release" class="apart">Fuji {{ release.version }}</span><span v-if="release" class="apart">{{ readableDate(release.date) }}</span></p>
					<p v-for="row in rows" :key="row.file"><a
						v-if="row.sha256" class="hash" tabindex="0"
						@click="copyHash(row)" @keyup.enter="copyHash(row)"
						@mouseenter="hintCopy" @mouseleave="unhintCopy"
						>{{ row.sha256 }}</a><span v-else-if="loaded">not yet published</span><span class="gap">{{ '  ' }}</span>{{ row.file }}</p>
					<!--
					The status shares Close's line, and its span is always rendered rather than v-if'd in. Both are deliberate. On its own line it changed the block's height as it appeared, which moved the hash out from under the pointer, which fired mouseleave, which cleared the status and gave the height back — a flicker loop you could hold the mouse still inside. Nothing here may change layout on hover.
					-->
					<p class="close"><a tabindex="0" @click="toggleHashes" @keyup.enter="toggleHashes">Close</a><span class="status">{{ status }}</span></p>
				</div>
			</div>

		</div>
	</div>
</div>

</template>

<style scoped>

/*
The old page did two jobs with two elements: body carried the mint over the whole viewport, and the box below was 80vh. Body is shared with the documentation pages now, so it can't be painted mint — .page takes over that job, and .home goes back to being exactly the 80vh box it was. Collapsing the two into one element is what pushed the content down a tenth of a viewport, since it centers inside its box.
*/

.page {
	min-height: 100vh;
	background-color: #9FFFE0;

	/*
	Decline VitePress's font smoothing, for this page only.
	
	Its base.css sets -webkit-font-smoothing: antialiased on body, a convention that spread widely around 2012. On macOS that switches off the stem-darkening pass the system applies, which renders text thinner. The property does nothing at all on Windows or Linux, and almost nothing on a Retina Mac — it is visible only to a Mac driving a low-resolution display, where the thinning costs real legibility.
	
	auto is not a competing opinion; it is the absence of one, leaving each machine to do what its own operating system judges right for its own display. The page this one replaces set nothing here and so behaved this way, which is the other reason to prefer it: it is what makes the two a pixel match rather than a near miss.
	
	Scoped to .page on purpose. The documentation pages keep VitePress's default, and nothing here reaches them.
	*/
	-webkit-font-smoothing: auto;
	-moz-osx-font-smoothing: auto;

	/*
	Black, for the same reason. VitePress sets body to --vp-c-text-1, which is #3c3c43, a soft slate rather than black — a sensible choice for long documentation, and not what this page had. The old page set no color at all and so got the browser default, pure black, which on the mint measures 17.85:1 against 9.30:1 — nearly double. Lower contrast is what reads as "lighter", which is why this looked like a font weight problem and was not.
	*/
	color: #000;
}

.home {                          /* was min-h-[80vh] p-8 flex */
	min-height: 80vh;
	padding: 2rem;                 /* p-8 */
	display: flex;
}

.row {                           /* was flex flex-col gap-8 flex-1 items-center */
	display: flex;
	flex-direction: column;
	gap: 2rem;                     /* gap-8 */
	flex: 1;
	align-items: center;
}

.mark {                          /* was flex flex-col gap-4 */
	display: flex;
	flex-direction: column;
	gap: 1rem;                     /* gap-4 */
}

.mark h1 {                       /* was font-helvetica font-bold text-8xl text-white leading-none */
	margin: 0;
	font-family: "Helvetica Neue", ui-sans-serif, system-ui, sans-serif;
	font-weight: 700;
	font-size: 6rem;               /* text-8xl */
	line-height: 1;                /* leading-none */
	color: #fff;
}

.mark svg {                      /* was width="100%" class="block aspect-square" */
	display: block;
	width: 100%;
	aspect-ratio: 1;
}

.copy {                          /* was flex-1 font-mono */
	flex: 1;
	font-family: "IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
}

.copy p {
	margin: 0;
}

.copy .links {                   /* was mt-8; qualified because .copy p above outranks a bare class */
	margin-top: 2rem;
}

.copy .hashes {                  /* the reveal, in the same rhythm as the links row above it */
	margin-top: 2rem;
}

.links a,
.hashes a {
	color: inherit;                /* the mint page sets its own black; links follow it rather than the theme's brand */
	text-decoration: underline;
	cursor: pointer;               /* the toggles carry no href, so they would not get one on their own */
}

.hashes a.hash {                 /* qualified: .hashes a above is (0,1,1) and would outrank a bare .hash */
	text-decoration: none;         /* clickable without looking like a link; the status line says so on hover */
	overflow-wrap: anywhere;       /* sixty-four hex characters offer the browser no break of their own */
}

.hashes .close {
	margin-top: 1.5rem;            /* one blank line, matching the 24px line-height, between the hashes and Close */
}

.hashes .status,
.hashes .apart {
	margin-left: 2ch;              /* two characters of the mono the line is set in */
}

.hashes .gap {
	white-space: pre;              /* keep both spaces: hash, two spaces, filename is the shasum -c format.
	                                  they arrive as an interpolation because Vue's compiler drops a
	                                  whitespace-only text node, which collapsed them to nothing */
}

@media (min-width: 768px) {      /* Tailwind's md: */
	.home { padding: 4rem; }       /* md:p-16 */
	.row {
		flex-direction: row;         /* md:flex-row */
		gap: 4rem;                   /* md:gap-16 */
		align-items: flex-start;     /* was center, from items-center. anchoring the tops is what lets the hashes open without moving anything already on the page: centered, growing text pushes itself upward */
	}
	.mark h1 { font-size: 12rem; } /* md:text-[12rem] */
	.copy { margin-top: 13rem; }   /* the logotype (12rem, line-height 1) plus the gap below it (1rem), so the first line of copy starts level with the top of the disc */
}

</style>
