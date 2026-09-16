//./.vitepress/theme/downloads.js

/*
Fuji's downloads, end to end: from the machine that built an installer to the page that offers it.

An installer is built where it can be — the dmg on a Mac, the exe on Windows, the deb on Linux — and `pnpm hash` there copies it out from under tauri's versioned name to a stable one, writing a small JSON sidecar beside it: file, version, arch, bytes, sha256, date. `pnpm upload` sends that pair to the server. Three sidecars rather than one combined file, because those three builds happen on three machines on three days and no single machine ever holds all three hashes at once.

The server answers one hostname from two directories and falls through to downloads on a miss, so an installer and its sidecar sit at the apex: fujidesktop.app/fuji.dmg beside fujidesktop.app/fuji.dmg.json. Nothing in this build knows a hash. A page fetches the sidecars when it opens, and that is the point of the whole arrangement — publishing an installer changes what the site says without the site being rebuilt.

	scripts.js                 pnpm hash writes a sidecar, pnpm upload sends it
	  ↓                        the server's downloads directory
	theme/downloads.js         this file — the list, the fetch, three small readings and the clipboard call
	  ↓
	components/Download.vue    one download; download-fuji.md places three of them by hand
	components/HomePage.vue    the same three hashes, in the reveal behind Hashes

Development has no downloads directory, so config.js proxies those three paths to production and the dev server shows the hashes that are really live.

**Fetch on mount, never at module scope.** VitePress prerenders components in Node at build time, and Node has fetch — so a module-scope call would run rather than fail loudly, but a relative url has no origin to resolve against there and nothing is serving these files during a build, so every row would bake in as unpublished.
*/

//the published names, in the order the home page lists them
export let installerFiles = ['fuji.dmg', 'fuji.exe', 'fuji.deb']

export async function fetchSidecars() {//all three, in that order, so a row can render before anyone knows whether it has a hash
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
