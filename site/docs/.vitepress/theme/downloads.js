//./.vitepress/theme/downloads.js

/*
Fuji's downloads, end to end: from the machine that built an installer to the page that offers it.

A package is built where it can be — the dmg on a Mac, the exe on Windows, and the four Linux packages in Docker containers on that same Mac — and `pnpm hash` there stages it under a published name, writing a small JSON sidecar beside it: file, version, arch, bytes, sha256, date. `pnpm upload` sends the pairs to the server. A sidecar each rather than one combined file, because these builds happen on different machines on different days and no single machine ever holds every hash at once.

The server answers one hostname from two directories and falls through to downloads on a miss, so an installer and its sidecar sit at the apex: fujidesktop.app/fuji.dmg beside fujidesktop.app/fuji.dmg.json. Nothing in this build knows a hash. A page fetches the sidecars when it opens, and that is the point of the whole arrangement — publishing an installer changes what the site says without the site being rebuilt.

	scripts.js                 pnpm hash writes a sidecar, pnpm upload sends it
	  ↓                        the server's downloads directory — or, in development, .vitepress/config.js
	theme/downloads.js         this file — the list, the fetch, three small readings and the clipboard call
	  ↓
	components/Download.vue    one download; download-fuji.md places six of them by hand
	components/HomePage.vue    the same six hashes, in the reveal behind Hashes

Development has no downloads directory, so config.js stands in for it twice over: a plugin serves whatever is staged in desktop/release and linux/release, and a proxy asks production for whatever is not. Staged work wins and production fills the gaps, which means `pnpm local` shows the release you are assembling rather than only the one already out.

**Fetch on mount, never at module scope.** VitePress prerenders components in Node at build time, and Node has fetch — so a module-scope call would run rather than fail loudly, but a relative url has no origin to resolve against there and nothing is serving these files during a build, so every row would bake in as unpublished.
*/

//the published names, in the order the home page lists them. no name carries a version, so a link anyone shares keeps pointing at the current build, and every linux package names its architecture because linux is where architectures multiply and a bare name would read as the default while being the rarer one
export let installerFiles = [
	'fuji.dmg',            //macOS, Apple silicon
	'fuji.exe',            //Windows, x86-64
	'fuji.arm64.deb',      //Linux, ARM — Raspberry Pi and like machines
	'fuji.amd64.deb',      //Linux, x86-64
	'fuji.x86_64.rpm',     //Fedora, RHEL and kin, x86-64
	'fuji.x86_64.flatpak', //any distribution, sandboxed, x86-64
]

export async function fetchSidecars() {//all of them, in that order, so a row can render before anyone knows whether it has a hash
	return Promise.all(installerFiles.map(file => fetchSidecar(file)))
}

//one sidecar, or false — a missing one means that installer is not published yet, which is an ordinary
//answer rather than an error. it sits beside its installer under the same name, so callers name the file
export async function fetchSidecar(file) {
	try {
		let response = await fetch('/' + file + '.json')
		if (!response.ok) return false
		let sidecar = await response.json()
		if (!sidecar || !sidecar.sha256 || !sidecar.file) return false//malformed reads the same as missing
		return sidecar
	} catch (error) { return false }//the network refused, or the body was not json
}

//the one version and date for a release as a whole, which the home page needs because it has room for a
//single heading. the earliest of the three builds, understating how fresh the release is rather than overstating it
export function earliestBuild(sidecars) {
	let earliest = false
	for (let sidecar of sidecars) {
		if (!sidecar) continue
		if (!earliest || sidecar.date < earliest.date) earliest = sidecar
	}
	return earliest
}

//"2026-09-13" as "2026 September 13". the sidecar stays ISO 8601 so earliestBuild can compare dates as
//text, and the month name comes from the platform. the parts go into Date as local numbers rather than
//through its string parser, which reads a bare date as UTC midnight and would render it as the day
//before for anyone west of Greenwich; the locale is fixed because the rest of the page is English
export function readableDate(iso) {
	let [year, month, day] = iso.split('-').map(Number)
	if (!year || !month || !day) return iso//a date we did not expect shows exactly as it arrived
	return year + ' ' + new Date(year, month - 1, day).toLocaleDateString('en', {month: 'long'}) + ' ' + day
}

//copy text to the clipboard, answering whether it worked. the clipboard needs a secure context, which
//https and localhost both are, so a false here is a browser refusing rather than a mistake to fix
export async function copyText(text) {
	try {
		await navigator.clipboard.writeText(text)
		return true
	} catch (error) { return false }
}

//whole kilobytes, counted by 1000 the way macOS and browsers do. whole, because a decimal point is a
//comma in half the world, and a number with no separator is right in both
export function saySize(bytes) {
	return Math.round(bytes / 1000) + ' KB'
}
