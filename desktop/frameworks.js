import {spawnSync} from 'node:child_process'
import {chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, realpathSync, rmSync, statSync, symlinkSync, writeFileSync} from 'node:fs'
import {basename, dirname, join} from 'node:path'
import {fileURLToPath} from 'node:url'

/*
The libraries fuji carries on the Mac, gathered into one folder before every build, so the binary links against copies that already know where they will live.

A Mac binary records each library it needs as a path string, and so does each library for the ones beneath it. Linked straight against Homebrew's libmpv, fuji would record /opt/homebrew/opt/mpv/lib/libmpv.2.dylib, a path that exists on the Mac that built it and nowhere else, and libmpv names nineteen more Homebrew libraries the same way, each naming others. So this walks that tree from libmpv down, copies every library into src-tauri/target/gathered, and rewrites each copy to name itself and its siblings as @executable_path/../Frameworks/name, which the loader expands at launch relative to wherever the binary is. The binary links against the copies, so it records the same relative name from the start. Tauri puts the copies where that name finds them: the list this writes into tauri.macos.conf.json, which Tauri merges into its configuration on the Mac without being asked, is copied by tauri_build into target/Frameworks at every compile, beside the debug and release folders, so a binary in either finds them at ../Frameworks, and by Tauri's bundler into Contents/Frameworks beside Contents/MacOS/fuji, where it signs them and the app inside out. So the installer and the apple workspace both carry the tree with no new flag. The gather stays out of target/Frameworks because tauri_build empties that folder before it fills it.

Rewriting a path changes a library's bytes, and on Apple silicon the kernel refuses to map code whose signature no longer matches, so every copy is signed again here, ad hoc, which is enough for a dev build; Tauri signs the bundled copies with fuji's identity. Nothing happens off the Mac, which alone has a tree to gather: on Windows the library is one dll that sits beside the executable, and on Linux it is the distribution's own package, each reached by its own road.

The copying is skipped when nothing under Homebrew has changed since the last gather, and the configuration file is rewritten only when its list changes, because tauri_build reads it at compile time and a changed file recompiles the crate.
*/

//every path is built from this file's own location, for the reason scripts.js gives at the root
const here          = fileURLToPath(new URL('.', import.meta.url))
const tauri         = join(here, 'src-tauri')
const folder        = join(tauri, 'target/gathered')//the gathered copies, which tauri copies to where a binary finds them; never target/Frameworks, which tauri_build empties
const manifest      = join(folder, 'manifest.json')//what the last gather copied, source by source, so an unchanged tree is not copied again
const configuration = join(tauri, 'tauri.macos.conf.json')//the list for tauri's bundler, gitignored and written here
const start         = '/opt/homebrew/opt/mpv/lib/libmpv.2.dylib'//where homebrew keeps the library the tree grows from; where the library comes from in the end is the video spike's subject
const linkName      = 'libmpv.dylib'//the unversioned name the linker looks for when build.rs says -lmpv, a symlink to the copy as homebrew's own folder has it
const system        = ['/usr/lib/', '/System/']//libraries every mac has, which the loader finds without help and which stay out of the tree

const inside = name => `@executable_path/../Frameworks/${name}`//what a copy is called, by the binary and by the copies beside it

//gather the tree, and answer how many libraries it holds and their bytes together; a result rather than an exception for the tools failing, which is the bottom gate
export function frameworksMake() {
	if (process.platform != 'darwin') return {success: true, outcome: 'nothing to gather off the mac', count: 0, bytes: 0}
	if (!existsSync(start)) return {success: false, outcome: `no library at ${start}; brew install mpv puts one there`}

	let tree = walk(start)//real path to the dependency strings its load commands carry
	let names = [...tree.keys()].map(path => basename(path))
	if (new Set(names).size != names.length) return {success: false, outcome: `two libraries share a name, and the folder is flat: ${names.filter((name, i) => names.indexOf(name) != i).join(', ')}`}//a tree this was not built for, said rather than one copy overwriting the other
	let sources = [...tree.keys()].map(path => ({path, size: statSync(path).size, modified: statSync(path).mtimeMs}))
	let bytes = sources.reduce((sum, source) => sum + source.size, 0)
	let same = existsSync(manifest) && readFileSync(manifest, 'utf8') == JSON.stringify(sources)
	if (!same) {
		rmSync(folder, {recursive: true, force: true})//a fresh folder, so a library the tree no longer holds is not left beside the ones it does
		mkdirSync(folder, {recursive: true})
		for (let [source, dependencies] of tree) {
			let copied = copyOne(source, dependencies)
			if (!copied.success) return copied
		}
		writeFileSync(manifest, JSON.stringify(sources))//last, so a gather that failed partway is done over
	}
	if (!existsSync(join(folder, linkName))) symlinkSync(basename(realpathSync(start)), join(folder, linkName))//for the linker alone; the list below holds real files only

	let list = [...tree.keys()].map(path => `target/gathered/${basename(path)}`)//relative to src-tauri, as every path in tauri's configuration is, so the binary carries no absolute path of this machine's
	let text = JSON.stringify({bundle: {macOS: {frameworks: list}}}, null, '\t') + '\n'
	if (!existsSync(configuration) || readFileSync(configuration, 'utf8') != text) writeFileSync(configuration, text)
	return {success: true, outcome: same ? 'unchanged' : 'gathered', count: tree.size, bytes}
}

function walk(first) {//every library under first by its real path, each with the dependency strings written in its load commands, the system's left out
	let tree = new Map()
	let queue = [first]
	while (queue.length) {
		let path = realpathSync(queue.shift())
		if (tree.has(path)) continue
		let dependencies = listed(path).filter(d => !system.some(prefix => d.startsWith(prefix)))
		tree.set(path, dependencies)
		for (let d of dependencies) queue.push(resolve(d, path))
	}
	return tree
}

function listed(path) {//the load command strings otool prints for a library, its own name first and then each library it needs
	let shown = spawnSync('otool', ['-L', path], {encoding: 'utf8'})
	if (shown.status != 0) throw new Error(`otool -L ${path}: ${shown.stderr}`)//a programmer error or a broken install, either way not a Tuesday
	return shown.stdout.split('\n').slice(1).map(line => line.trim().split(' (')[0]).filter(Boolean)
}

function resolve(dependency, from) {//the file a load command string names: an absolute path as it is, and a name relative to the loader or the run path as the file beside the library that names it, or under homebrew's lib
	if (dependency.startsWith('/')) return dependency
	let name = basename(dependency)
	for (let candidate of [join(dirname(from), name), join('/opt/homebrew/lib', name)]) if (existsSync(candidate)) return candidate
	throw new Error(`cannot find ${dependency}, named by ${from}`)
}

function copyOne(source, dependencies) {//one library into the folder, told its own name and its siblings' as they will be inside the app, and signed again since the telling changed its bytes
	let name = basename(source)
	let copy = join(folder, name)
	copyFileSync(source, copy)
	chmodSync(copy, 0o755)//homebrew's are read-only, and install_name_tool writes in place
	let args = ['-id', inside(name)]
	for (let d of dependencies) {
		let real = realpathSync(resolve(d, source))
		if (real != source) args.push('-change', d, inside(basename(real)))//every library it names, by the name its copy has here; its own id line is set by -id above
	}
	let told = spawnSync('install_name_tool', [...args, copy], {encoding: 'utf8'})
	if (told.status != 0) return {success: false, outcome: `install_name_tool ${name}: ${told.stderr}`}
	let signed = spawnSync('codesign', ['--force', '--sign', '-', copy], {encoding: 'utf8'})
	if (signed.status != 0) return {success: false, outcome: `codesign ${name}: ${signed.stderr}`}
	return {success: true}
}

function main() {
	let made = frameworksMake()
	if (!made.success) { console.error('🚧 ' + made.outcome); process.exitCode = 1; return }
	console.log(`frameworks: ${made.outcome}, ${made.count} libraries, ${made.bytes.toLocaleString('en-US')} bytes`)
}
if (process.argv[1] && realpathSync(process.argv[1]) == fileURLToPath(import.meta.url)) main()//only when run, so notarize.js can import the gather
