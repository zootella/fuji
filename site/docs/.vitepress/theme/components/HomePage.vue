<script setup>//./.vitepress/theme/components/HomePage.vue
import { ref, onMounted } from 'vue'
import { installerFiles, fetchSidecars, earliestBuild, readableDate, copyText } from '../downloads.js'

/*
The apex page. index.md carries layout: false, so VitePress renders no navbar, sidebar, or footer, and this component is the whole document.

It is a port of the Nuxt page it replaces, class for class, now that Tailwind is gone. It names its own fonts rather than reading the theme's variables, so the two never pull on each other: nothing style.css does to the documentation pages can move this one, and nothing here can reach them. The Tailwind values it was written in are noted beside the CSS below so the two can be compared.

Three reveals open below the links, one at a time: Mac and Win each show a short paragraph on getting past the first-run warning, on the same click that starts the download, and Hashes shows the hash list. The hash list is the one with a mechanism, and downloads.js holds it — what a sidecar is, why there is one per package rather than one combined file, and why the fetch has to happen on mount. The download page reads that same module and shows the rest of what a sidecar carries; this page shows a hash and a filename per row, which is what its box has room for. The row count follows installerFiles rather than being written here, so adding a package to that list adds a row to this reveal and nothing else has to change.
*/

let showing = ref('')//which reveal is open below the links: mac, win, hashes, or blank for none. One at a time, because they share the space
let rows = ref(installerFiles.map(file => ({file, sha256: ''})))//one row per installer, named from the start so opening the list never changes its height; a hash arrives when its sidecar does
let release = ref(false)//the version and date above the rows, or false before anything is published
let loaded = ref(false)//every fetch has settled, so a blank hash now means unpublished rather than unread
let status = ref('')//the line beneath Close: the hover hint, then the copy confirmation

/*
The sidecars are fetched when this component mounts rather than when the reader opens the list, so the hashes are already in hand the moment Hashes is clicked. That costs every visitor one request per package they may never look at — three when this was written, six now — which was weighed rather than measured and accepted: the files are a couple of hundred bytes each, they go in parallel, and they are the only thing on this page that is not already in the bundle. Mount rather than module scope is required rather than preferred, for the reason downloads.js gives.

One consequence of mount rather than load: VitePress navigates between pages on the client, so leaving this page and coming back mounts the component again and fetches again. That is a handful of small requests for a fresher answer, and it is the behaviour we want.
*/
onMounted(async () => {
	let sidecars = await fetchSidecars()
	rows.value = installerFiles.map((file, index) => ({file, sha256: sidecars[index] ? sidecars[index].sha256 : ''}))
	release.value = earliestBuild(sidecars)
	loaded.value = true
})

//a download link opens its reveal and never closes it: the click is a download first, and downloading again should not fold the instructions away
function show(name) { showing.value = name }

//open and close the list; the hashes are already in hand by the time anyone can click this
function toggleHashes() {
	showing.value = showing.value == 'hashes' ? '' : 'hashes'
	status.value = ''//collapsing clears the status
}

function close() { showing.value = ''; status.value = '' }//every reveal ends with a Close

//the hash is clickable but not underlined, so the status line carries the affordance
function hintCopy() { status.value = 'Click to Copy' }
function unhintCopy() { if (status.value == 'Click to Copy') status.value = '' }//a Copied confirmation stays standing

//copy one hash to the clipboard, and say so beneath
async function copyHash(row) {
	status.value = await copyText(row.sha256) ? 'Copied' : 'Could not copy'
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
				
				The Mac and Windows paths are the real files and are deliberately stable, and each click also opens the reveal for that system beneath the links, without getting in the way of the download: the build names its output Fuji_0.1.0_aarch64.dmg and the like, and the upload renames on the way out, so a link posted today keeps working across releases. They 404 until the first release is uploaded, which is expected — nothing about this markup changes when it stops being true.
				
				Linux is a link to the download page rather than to a file, and has to be. Fuji ships four linux packages — two debs, an rpm and a flatpak — and one Linux button could only pick one of them for everybody, handing most people something their machine cannot install. One word here cannot ask which distribution and which processor; that page can.
				
				GitHub points at the application's own repository, not this one.
				-->
				<p class="links">
					<a href="/fuji.dmg" download @click="show('mac')">Mac</a> <a href="/fuji.exe" download @click="show('win')">Win</a> <a href="/download-fuji.html">Linux</a> - <a tabindex="0" @click="toggleHashes" @keyup.enter="toggleHashes">Hashes</a> <a href="https://github.com/zootella/fuji">GitHub</a> <a href="/getting-started.html">Docs</a>
				</p>

				<!-- what the first launch asks for, in a few sentences, with the words on the screen in italics; the download page has the long version and More Information points at it -->
				<div v-if="showing == 'mac'" class="reveal">
					<p>Note: Open your download <i>fuji.dmg</i> and drag <i>Fuji</i> into <i>Applications</i>. The first time you run Fuji, macOS will say <i>"Fuji Not Opened. Apple could not verify Fuji is free of malware"</i> with the buttons <i>Done</i> and <i>Move to Trash</i>. Click <i>Done</i>. Go to macOS settings, click <i>Privacy & Security</i>, and scroll to the bottom. <i>Fuji</i> will be listed, click <i>Open Anyway</i>. Fuji runs normally after these first-time steps. <a href="/download-fuji.html#running-fuji-for-the-first-time">More Information</a></p>
					<p class="close"><a tabindex="0" @click="close" @keyup.enter="close">Close</a></p>
				</div>

				<div v-if="showing == 'win'" class="reveal">
					<p>Note: Double-click your download, <i>fuji.exe</i>. Windows will show a blue warning with the text <i>"Windows protected your PC."</i> Click <i>More info</i>, then <i>Run anyway</i> to install Fuji. <a href="/download-fuji.html#running-fuji-for-the-first-time">More Information</a></p>
					<p class="close"><a tabindex="0" @click="close" @keyup.enter="close">Close</a></p>
				</div>

				<div v-if="showing == 'hashes'" class="reveal hashes">
					<p class="heading">SHA-256<span v-if="release" class="apart">Fuji {{ release.version }}</span><span v-if="release" class="apart">{{ readableDate(release.date) }}</span></p>
					<p v-for="row in rows" :key="row.file"><a
						v-if="row.sha256" class="hash" tabindex="0"
						@click="copyHash(row)" @keyup.enter="copyHash(row)"
						@mouseenter="hintCopy" @mouseleave="unhintCopy"
						>{{ row.sha256 }}</a><span v-else-if="loaded">not yet published</span><span class="gap">{{ '  ' }}</span>{{ row.file }}</p>
					<!--
					The status shares Close's line, and its span is always rendered rather than v-if'd in. Both are deliberate. On its own line it changed the block's height as it appeared, which moved the hash out from under the pointer, which fired mouseleave, which cleared the status and gave the height back — a flicker loop you could hold the mouse still inside. Nothing here may change layout on hover.
					-->
					<p class="close"><a tabindex="0" @click="close" @keyup.enter="close">Close</a><span class="status">{{ status }}</span></p>
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

/* The vertical rhythm of this column, said once rather than per element. Wherever a new thought starts it is one blank line away, and one blank line is 1.5rem because that is the line-height the mono is set in — so the gap is a real empty line rather than a number that merely looks about right. The Tailwind port had 2rem here, from mt-8, which is a line and a third and sat outside the rhythm.

The tagline's two lines get none: they are one sentence broken to fit, not two paragraphs. That is why this is a list of the places a gap belongs rather than a blanket rule on adjacent paragraphs. */
.copy {
	--blank-line: 1.5rem;
}

.copy .links,                    /* qualified because .copy p above outranks a bare class */
.copy .reveal {
	margin-top: var(--blank-line);
}

.links a,
.reveal a {
	color: inherit;                /* the mint page sets its own black; links follow it rather than the theme's brand */
	text-decoration: underline;
	cursor: pointer;               /* the toggles carry no href, so they would not get one on their own */
}

.hashes a.hash {                 /* qualified: .reveal a above is (0,1,1) and would outrank a bare .hash */
	text-decoration: none;         /* clickable without looking like a link; the status line says so on hover */
	overflow-wrap: anywhere;       /* sixty-four hex characters offer the browser no break of their own */
}

/* The hashes sit between two blank lines, so the block reads as its own thing rather than as text crowding the heading and the Close. */
.hashes .heading {
	margin-bottom: var(--blank-line);
}

.reveal .close {
	margin-top: var(--blank-line);
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
