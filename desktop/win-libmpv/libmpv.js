import {execFileSync, spawnSync} from 'node:child_process'
import {existsSync, statSync} from 'node:fs'
import {join} from 'node:path'
import {fileURLToPath} from 'node:url'
import {buildLog, readSha256} from '../../build-log.js'

/*
Starts build.sh beside this file inside MSYS2, which is where it runs: a login shell of the ucrt64 environment, with the toolchain the script needs installed first if it is not there. MSYS2 itself has to be installed, at C:\msys64, which winget does in one line and which this says if it is missing. Off Windows this does nothing and says so, the way frameworks.js does off the Mac, so the one name, pnpm libmpv, is safe to type anywhere. The build is a step in the build log like the installer's, started and finished with the dll's hash, so the log says which library a release carried.
*/

const here     = fileURLToPath(new URL('.', import.meta.url))
const tauri    = join(here, '../src-tauri')
const dll      = join(tauri, 'binaries/libmpv-2.dll')//where build.sh leaves it, and tauri.windows.conf.json finds it
const msys     = 'C:\\msys64'
const bash     = join(msys, 'usr/bin/bash.exe')
const packages = ['git', 'make', 'diffutils', ...['gcc', 'meson', 'ninja', 'pkgconf', 'nasm'].map(name => 'mingw-w64-ucrt-x86_64-' + name)]//what the eight packages call between them: git for libplacebo's clone, make for zlib, libass and FFmpeg, diffutils for the cmp in FFmpeg's and libass's configure scripts, and from ucrt64 the compiler with its binutils, meson and ninja, pkgconf as the pkg-config every configure asks, and nasm for FFmpeg's assembly

function main() {
	if (process.platform != 'win32') { console.log("libmpv: nothing to build off windows; the mac gathers homebrew's and linux links the distribution's"); return }
	if (!existsSync(bash)) throw new Error(`MSYS2 is not at ${msys}; install it with: winget install --id MSYS2.MSYS2`)
	buildLog('libmpv', 'started', '-', '-', 'from the release tarballs build.sh names')
	let environment = {...process.env, MSYSTEM: 'UCRT64'}//the ucrt64 environment, which is the one whose gcc links against the universal C runtime every Windows 10 has
	execFileSync(bash, ['-lc', `pacman -Q ${packages.join(' ')} >/dev/null 2>&1 || pacman -S --noconfirm --needed ${packages.join(' ')}`], {env: environment, stdio: 'inherit'})//silent when every package is there, and the whole install shown the first time
	let script = here.replace(/\\/g, '/').replace(/^([A-Za-z]):/, (match, drive) => '/' + drive.toLowerCase()) + 'build.sh'//C:\a\b\ as /c/a/b/, which is how msys spells a windows path
	let run = spawnSync(bash, ['-lc', `bash "${script}"`], {env: environment, stdio: 'inherit'})
	if (run.status != 0) throw new Error('build.sh failed')
	buildLog('libmpv', 'finished', 'libmpv-2.dll', readSha256(dll), `${statSync(dll).size} bytes in src-tauri/binaries`)
}

try { main() } catch (e) { buildLog('libmpv', 'failed', '-', '-', e.message || String(e)); console.error('🚧 Error:', e.message || e); process.exitCode = 1 }
