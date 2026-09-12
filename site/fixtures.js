//./site/fixtures.js

import {existsSync} from 'node:fs'
import {copyFile, mkdir, rm} from 'node:fs/promises'

/*
Sidecars in two places, for two reasons. On the server they sit in the downloads directory beside the installers they describe, which is where the page fetches them from. During local development there is no server, so copies go into docs/public/ where vitepress serves them at the same paths — enough to work on the page's reveal-the-hash behavior without publishing anything.

Those copies are throwaway and gitignored, and they must never reach a build: a sidecar baked into the site would be served from static1 and shadow the real one in static2, pinning the page to whatever hash happened to be lying around when the site was last built. So the upload script clears them before vitepress runs, which is the "clear" mode below.

This file is committed rather than hidden because it names no destination — it only moves files around inside the repository. That matters on the second computer: a gitignored file has to be recreated by hand on every machine, and this one does not.
*/

//the stable published names release.js writes into desktop/release/, each with its sidecar beside it
const sidecars = ['fuji.dmg.json', 'fuji.exe.json', 'fuji.deb.json']

const staging = '../desktop/release'//where the desktop workspace leaves a finished release
const serving = 'docs/public'//where vitepress serves static files from during local development

let mode = process.argv[2]//copy or clear

async function copyFixtures() {//bring whatever sidecars this machine has built into the dev server's static directory
	await mkdir(serving, {recursive: true})
	let copied = 0
	for (let name of sidecars) {
		if (!existsSync(`${staging}/${name}`)) continue//a machine only ever builds its own platform
		await copyFile(`${staging}/${name}`, `${serving}/${name}`)
		copied++
		console.log(`copied   ${name}`)
	}
	if (copied == 0) console.log('no sidecars staged on this machine; run pnpm release in desktop first')
}

async function clearFixtures() {//take them back out, so a build cannot carry one
	for (let name of sidecars) {
		await rm(`${serving}/${name}`, {force: true})
	}
}

async function main() {//move the dev-only sidecar copies in or out of docs/public/
	if (mode == 'copy')  { await copyFixtures();  return }
	if (mode == 'clear') { await clearFixtures(); return }
	throw new Error('say which: copy, clear')
}
main().catch(e => { console.error('🚧 Error:', e); process.exitCode = 1 })
