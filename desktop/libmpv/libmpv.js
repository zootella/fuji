import {execFileSync, spawnSync} from 'node:child_process'
import {existsSync, statSync} from 'node:fs'
import {join} from 'node:path'
import {fileURLToPath} from 'node:url'
import {buildLog, readSha256} from '../../build-log.js'

/*
Starts build.py beside this file with the Python of the environment it runs in: on Windows a login shell of MSYS2's ucrt64 environment, with the toolchain the script needs installed first if it is not there, and on the Mac the Mac's own python3, with the tools named here looked for first and the brew line that installs them said if any is missing. MSYS2 itself has to be installed, at C:\msys64, which winget does in one line and which this says if it is missing. Off both this does nothing and says so, since Linux builds the library inside its containers, so the one name, pnpm library, is safe to type anywhere; linux has its own pnpm library, which runs this recipe inside its containers. The build is a step in the build log like the installer's, started and finished with the library's hash, so the log says which library a release carried.

On Windows the Python is MSYS2's own, /usr/bin/python3, and not the native one meson brings into ucrt64: the recipe runs the twelve projects' configure scripts, which read a path as /c/Users and would choke on C:\Users, and MSYS2's Python spells paths the first way and converts them for the native compilers it starts, as bash did before it.
*/

const here   = fileURLToPath(new URL('.', import.meta.url))
const tauri  = join(here, '../src-tauri')
const script = join(here, 'build.py')
const msys   = 'C:\\msys64'
const msysBash = join(msys, 'usr/bin/bash.exe')
const msysPackages = ['git', 'make', 'diffutils', 'python', ...['gcc', 'meson', 'ninja', 'cmake', 'pkgconf', 'nasm', 'cppwinrt'].map(name => 'mingw-w64-ucrt-x86_64-' + name)]//what the packages call between them: git for the clones, make for zlib, libass and FFmpeg, diffutils for the cmp in FFmpeg's and libass's configure scripts, python as MSYS2's own, which runs the recipe, and from ucrt64 the compiler with its binutils, meson and ninja, cmake for the two Khronos packages, pkgconf as the pkg-config every configure asks, nasm for FFmpeg's and dav1d's assembly, and cppwinrt, Microsoft's header-only projection of the Windows Runtime, which mpv's media-key support is written against
const macTools = ['meson', 'ninja', 'pkg-config']//three tools from Homebrew over Apple's clang and Swift, which the Mac already has; pkg-config is pkgconf's name on the path, and cmake and nasm are Windows' alone, since nothing on the Mac is built with cmake and arm64 assembly goes through the compiler
const macFormulae = 'meson ninja pkgconf'

async function main() {
	if (process.platform == 'win32') return windows()
	if (process.platform == 'darwin') return mac()
	console.log('library: nothing to build off the mac and windows; pnpm library in the linux workspace builds it inside the containers')
}
async function windows() {
	if (!existsSync(msysBash)) throw new Error(`MSYS2 is not at ${msys}; install it with: winget install --id MSYS2.MSYS2`)
	let environment = {...process.env, MSYSTEM: 'UCRT64'}//the ucrt64 environment, which is the one whose gcc links against the universal C runtime every Windows 10 has
	execFileSync(msysBash, ['-lc', `pacman -Q ${msysPackages.join(' ')} >/dev/null 2>&1 || pacman -S --noconfirm --needed ${msysPackages.join(' ')}`], {env: environment, stdio: 'inherit'})//silent when every package is there, and the whole install shown the first time
	let path = script.replace(/\\/g, '/').replace(/^([A-Za-z]):/, (match, drive) => '/' + drive.toLowerCase())//C:\a\b\build.py as /c/a/b/build.py, which is how msys spells a windows path
	await build(msysBash, ['-lc', `/usr/bin/python3 "${path}"`], environment, 'libmpv-2.dll')
}
async function mac() {
	let missing = macTools.filter(tool => spawnSync('which', [tool]).status != 0)
	if (missing.length) throw new Error(`${missing.join(', ')} not found; install the tools with: brew install ${macFormulae}`)
	await build('python3', [script], process.env, 'libmpv.2.dylib')
}
async function build(interpreter, args, environment, name) {//run the script and log the library it made, by name and hash
	buildLog('library', 'started', '-', '-', 'from the release tarballs build.py names')
	let run = spawnSync(interpreter, args, {env: environment, stdio: 'inherit'})
	if (run.status != 0) throw new Error('build.py failed')
	let made = join(tauri, 'binaries', name)//where build.py leaves it, and the platform's tauri configuration file finds it
	buildLog('library', 'finished', name, await readSha256(made), `${statSync(made).size} bytes in src-tauri/binaries`)
}

main().catch(e => { buildLog('library', 'failed', '-', '-', e.message || String(e)); console.error('🚧 Error:', e.message || e); process.exitCode = 1 })
