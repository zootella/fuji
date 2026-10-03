import {readFileSync} from 'node:fs'
import {parse as parseToml} from 'smol-toml'

/*
What uninstall takes out of the registry: the one list an app changes, and beneath it the rules every app shares, as code. This is the one file in this folder with fuji's own content; the others name no product, and read every name from tauri.conf.json and Cargo.toml.

The running program writes these entries, not the installer, when it offers itself for its file types and when the user says yes to opening them with it; the installer only takes them back, and only on uninstall, never on an upgrade, which is what keeps a user's choice of the program through a release. Every key is under HKCU, the user's own hive, which is the only one the installer touches.

The list holds the only thing that differs from one app to the next: its associations, the file types and link protocols it opens, as plain strings written the way a person sees them. A file extension begins with its dot, .webp, and a protocol ends with its colon, magnet:, which is stripped for the registry; anything that is neither stops the build. Fuji opens files and no links, so its list is extensions alone.

Everything else is the same for every app, and the code below knows it: a ProgID for each extension and protocol named the way associate.js in the page names them, {brandName}.webp for a file and {brandName}.url.magnet for a link; the app's listing in Settings under Default apps, the Capabilities key that listing points at, and whatever else the app kept under its own key beneath Software; its key under Applications, which names the executable; and its line under Run for starting with Windows. An extension's own key and a protocol's shared class are never taken whole, since another program may own them next: the extension's default goes only while it still names the app, the class only while its command still runs the app, and the extension's keys only when nothing else is left in them.
*/

const brandStem = parseToml(readFileSync(new URL('../src-tauri/Cargo.toml', import.meta.url), 'utf8')).package.name//the stem of the executable's name, which is the crate's name, since Cargo names the executable from it

/*
The file extensions uninstall cleans up, with their dot. Setup registers none of them; the installed app does that itself, in associate.js, while it runs, so this list matters only on the way out, so that everything the app wrote for these types is taken back.

These are the enabled entries of fileTypes in src/fileTypes.js, typed out a third time: src-tauri/Info.plist holds the second copy, for the Mac. Info.plist cannot read that table, and this file does not, since keeping three short lists in step by hand, in the same commit, costs less than a build step. Fuji only ever turns a type on, and never back off, so the three only grow, and together.
*/
const uninstallAssociations = [
	'.bmp',
	'.gif',
	'.jpg',
	'.jpeg',
	'.jpe',
	'.jfif',
	'.png',
	'.svg',
	'.avif',
	'.webp',
]

/*
The rules, as the five instructions the setup program carries out, in order, each a tab-separated string of verb, key, name, equals, and at, which setup.c reads in that order. delete-key removes a key and everything under it. delete-value removes one value; an empty name is the key's default value. delete-value-if removes the value only while it still equals the text given. delete-key-if-empty removes the key only if no values and no subkeys are left in it. delete-key-if removes the key and everything under it only while the value named, at the subkey given, still equals the text given. {program} is left in the text for the setup program, which alone knows the installed executable's path, to fill at uninstall time.
*/
export function uninstallInstructions(brandName) {
	let found = []
	let add = (verb, key, {name = '', equals = '', at = ''} = {}) => found.push([verb, 'HKCU\\' + key, name, equals, at].join('\t'))
	let classes = 'Software\\Classes\\'
	let command = '"{program}" "%1"'//how a shell-open command is written in the registry, quotes included, and what associate.js writes for every type

	//the list sorted into its two kinds by the dot or the colon, which is the whole of the rule; an entry with neither, or with a character no extension or protocol may hold, stops the build rather than reaching the registry
	let extensions = [], protocols = []
	for (let association of uninstallAssociations) {
		let word = /^[\w+-][\w.+-]*$/
		if (typeof association == 'string' && association.startsWith('.') && word.test(association.slice(1))) extensions.push(association)
		else if (typeof association == 'string' && association.endsWith(':') && word.test(association.slice(0, -1))) protocols.push(association.slice(0, -1))
		else throw new Error(`registry.js: ${String(association)} is neither a file extension like .webp nor a protocol like magnet:`)
	}

	//the listing in Settings under Default apps, taken down first, so Settings stops naming the app before the keys it points at go; then the app's own key under Software, which holds the Capabilities block and anything an earlier installer kept there
	add('delete-value', 'Software\\RegisteredApplications', {name: brandName})
	add('delete-key', 'Software\\' + brandName)

	//each file type: the app's own ProgID, its place in the list of programs offered for the extension, the extension's default while it still names the app, and the extension's keys if that leaves them empty
	for (let extension of extensions) {
		let program = brandName + extension//.webp becomes Fuji.webp, the way associate.js names it
		add('delete-key', classes + program)
		add('delete-value', classes + extension + '\\OpenWithProgids', {name: program})
		add('delete-value-if', classes + extension, {equals: program})
		add('delete-key-if-empty', classes + extension + '\\OpenWithProgids')
		add('delete-key-if-empty', classes + extension)
	}

	//each protocol: the app's own ProgID whole, and the shared class named for the protocol only while its command still runs the app; another program that has taken the class since keeps it
	for (let protocol of protocols) {
		add('delete-key', classes + brandName + '.url.' + protocol)
		add('delete-key-if', classes + protocol, {at: 'shell\\open\\command', equals: command})
	}

	//the executable's own entry under Applications, with the types it said it supports, and the value under Run that starts the app with Windows, which fuji does not write
	add('delete-key', classes + 'Applications\\' + brandStem + '.exe')
	add('delete-value', 'Software\\Microsoft\\Windows\\CurrentVersion\\Run', {name: brandName})
	return found
}
