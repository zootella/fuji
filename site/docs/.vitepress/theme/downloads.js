/*
Fuji's downloads, end to end: from the machine that built an installer to the page that offers it.

A package is built where it can be — the dmg on a Mac, the exe on Windows, and the four Linux packages in Docker containers on that same Mac — and `pnpm hash` there stages it under a published name, writing a small JSON sidecar beside it: file, version, arch, size, sha256, and the moment it was hashed, kept twice: as ergo's tick in tick, and as its iso text in date, which no code reads. `pnpm upload` sends the pairs to the server. A sidecar each rather than one combined file, because these builds happen on different machines on different days and no single machine ever holds every hash at once.

The server answers one hostname from two directories and falls through to downloads on a miss, so an installer and its sidecar sit at the apex: fujidesktop.app/fuji.dmg beside fujidesktop.app/fuji.dmg.json. Nothing in this build knows a hash. A page fetches the sidecars when it opens, and that is the point of the whole arrangement — publishing an installer changes what the site says without the site being rebuilt.

	scripts.js                      pnpm hash writes a sidecar, pnpm upload sends it
	  ↓                             the server's downloads directory — or, in development, .vitepress/config.js
	theme/downloads.js              this file — the list, the fetch, one small reading and the clipboard call
	  ↓
	components/DownloadLink.vue     one download; download-fuji.md places six of them by hand
	components/DownloadCommand.vue  the hash written into a fenced install command; download-fuji.md places two
	components/HomePage.vue         the same six hashes, in the reveal behind Hashes

Development has no downloads directory, so config.js stands in for it twice over: a plugin serves whatever is staged in desktop/release and linux/release, and a proxy asks production for whatever is not. Staged work wins and production fills the gaps, which means `pnpm local` shows the release you are assembling rather than only the one already out.

**Fetch on mount, never at module scope.** VitePress prerenders components in Node at build time, and Node has fetch — so a module-scope call would run rather than fail loudly, but a relative url has no origin to resolve against there and nothing is serving these files during a build, so every row would bake in as unpublished.
*/
import { hasInt, hasTick, saySize4, sayTick } from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in

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

//the day a tick falls on in UTC, like "2026 September 13", the one way the site says a day: a sidecar's from the tick it keeps, and an article's last update; blank for no tick, which is a sidecar written before sidecars kept one, until its machine publishes again. UTC, so every reader sees the same day wherever they are
export function sayDay(t) {
	if (!hasTick(t)) return ''
	return sayTick(t, 'YYYY Mmmm D')
}

//a sidecar's size, like "17 MB", or blank for a sidecar with no whole count of bytes, which saySize4 would refuse, so a malformed sidecar shows its hash and version and leaves the size out
export function saySize(n) {
	if (!hasInt(n) || n < 0) return ''
	return saySize4(n)
}

//copy text to the clipboard, answering whether it worked. the clipboard needs a secure context, which
//https and localhost both are, so a false here is a browser refusing rather than a mistake to fix
export async function copyText(text) {
	try {
		await navigator.clipboard.writeText(text)
		return true
	} catch (error) { return false }
}
