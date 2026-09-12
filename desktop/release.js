//./desktop/release.js

import {createHash} from 'node:crypto'
import {copyFileSync, mkdirSync, readdirSync, readFileSync, writeFileSync} from 'node:fs'

/*
Fuji's release command, and the producing half of a two-part arrangement. This workspace makes an installer and the numbers that describe it together, on the machine that can build them. The site workspace publishes: it copies finished files to the server and does no thinking about them.

Tauri writes a bundle under a versioned, architecture-specific name, Fuji_0.1.0_aarch64.dmg, which is right for a build directory and wrong for a download link — a link in a page or a README has to keep working across releases. So this copies the bundle into release/ under a stable publishing name, fuji.dmg, and writes a sidecar beside it. Doing the rename here rather than at upload time is what keeps the other half dumb: the site side copies known filenames from a known path, and needs no rules about versions or architectures.

Two things the sidecar cannot be allowed to lie about, and each is answered by reading rather than assuming. The version comes from tauri.conf.json, the same file that named the bundle, so the sidecar and the artifact cannot disagree about which release this is. The architecture comes out of the bundle's own filename, so it describes the file that exists rather than the machine that happened to run the script.

Worth knowing plainly, so the number on the page is not mistaken for more than it is: a published hash catches a corrupted or truncated download, and the ordinary mistake of shipping the wrong build. It is not a defense against a compromised build machine — whoever can replace the installer can replace the sidecar, which rides the same script to the same directory. Code signing is the real answer there, and it is deliberately not in this pass.

One platform per run, because a dmg needs macOS and an exe needs Windows and no single machine makes both. Each writes its own pair into release/, where the installers stay out of git and the sidecars are committed.
*/

//where tauri leaves each platform's bundle, what the tail of its filename looks like, and the stable name we publish it under
const bundles = {
	darwin: {folder: 'dmg',  suffix: '.dmg',       publishedName: 'fuji.dmg'},
	win32:  {folder: 'nsis', suffix: '-setup.exe', publishedName: 'fuji.exe'},
	linux:  {folder: 'deb',  suffix: '.deb',       publishedName: 'fuji.deb'},
}

async function main() {//stage this platform's installer under its publishing name and write the sidecar beside it
	let bundle = bundles[process.platform]
	if (!bundle) throw new Error('no bundle shape for this platform: ' + process.platform)

	let configuration = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'))
	let version = configuration.version//the file that named the bundle, so the two cannot disagree
	if (!version) throw new Error('tauri.conf.json has no version')

	//find this version's bundle, and only this version's; an old one left beside it would otherwise be a coin flip
	let folder = `src-tauri/target/release/bundle/${bundle.folder}`
	let prefix = `Fuji_${version}_`
	let names = readdirSync(folder).filter(name => name.startsWith(prefix) && name.endsWith(bundle.suffix))
	if (names.length != 1) throw new Error(`expected one ${prefix}*${bundle.suffix} in ${folder}, found ${names.length} of them: ${readdirSync(folder).join(', ')}`)

	let name = names[0]
	let architecture = name.slice(prefix.length, name.length - bundle.suffix.length)//what tauri called it, between the version and the extension

	//copy first, then hash and measure what landed, so every number describes the file the site will actually ship
	mkdirSync('release', {recursive: true})
	let destination = `release/${bundle.publishedName}`
	copyFileSync(`${folder}/${name}`, destination)
	let bytes = readFileSync(destination)//whole file into memory; an installer is a few megabytes, and streaming would buy nothing here

	let sidecar = {
		file: bundle.publishedName,
		version,
		arch: architecture,
		bytes: bytes.length,
		sha256: createHash('sha256').update(bytes).digest('hex'),
		date: new Date().toISOString().slice(0, 10),//iso, because the page sorts three of these as text to find the earliest build
	}
	writeFileSync(`${destination}.json`, JSON.stringify(sidecar, null, '\t') + '\n')

	console.log(`staged  ${name}`)
	console.log(`     ->  ${destination}  ${sidecar.bytes} bytes`)
	console.log(`        ${destination}.json  ${sidecar.sha256}`)
}
main().catch(e => { console.error('🚧 Error:', e); process.exitCode = 1 })
