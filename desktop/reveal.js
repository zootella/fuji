//./desktop/reveal.js

import {execFile} from 'node:child_process'
import {existsSync} from 'node:fs'
import {resolve} from 'node:path'

/*
Open the graphical file manager on this platform's finished installer, so it can be double-clicked the way a person who downloaded it would.

That is the whole point: `pnpm win` launches the built executable directly, which is the right thing for trying a change quickly and the wrong thing for seeing what an installer does. An installer has a first-run experience — the publisher warning, the wizard, where it puts itself — and none of that happens when the binary is started in place.

This is a node script rather than a line in package.json because the line would be three lines, one per platform, and the Windows one would be fragile. Explorer resolves a relative path against its own working directory rather than the one pnpm is running in, so the path has to be absolute; and whether pnpm hands a script to Git Bash or to cmd decides whether forward slashes and shell quoting survive. Node resolves the path itself and spawns the opener directly, which has neither problem.

It opens the folder rather than selecting the file, because the filename carries the version and the architecture — Fuji_0.1.0_x64-setup.exe — and a script that named it would need editing at every release. The folder holds one file anyway.
*/

const bundles = {//where tauri leaves each platform's installer, and the command that shows a folder to the user
	darwin: {folder: 'dmg',  open: 'open'},
	win32:  {folder: 'nsis', open: 'explorer'},
	linux:  {folder: 'deb',  open: 'xdg-open'},
}

let bundle = bundles[process.platform]
if (!bundle) { console.error(`no bundle folder known for this platform: ${process.platform}`); process.exit(1) }

let path = resolve(`src-tauri/target/release/bundle/${bundle.folder}`)
if (!existsSync(path)) { console.error(`nothing built yet at ${path}\nrun pnpm build first, or pnpm release to stage and hash it too`); process.exit(1) }

console.log(`opening  ${path}`)
execFile(bundle.open, [path], error => {
	//explorer answers 1 even when it has opened the window, so a non-zero exit here means nothing and is not reported
	if (error && process.platform != 'win32') console.error(`could not open the file manager: ${error.message}`)
})
