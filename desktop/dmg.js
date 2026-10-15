import {spawn} from 'node:child_process'
import {mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync} from 'node:fs'
import {tmpdir} from 'node:os'
import {join} from 'node:path'
import {fileURLToPath} from 'node:url'
import {buildLog, readSha256} from '../build-log.js'

/*
The mac installer, made without Finder.

Tauri can make a dmg itself, and fuji let it until October 2026. Its way is a fork of the create-dmg shell script, which mounts the new image and runs an AppleScript asking Finder to open the window, place the icons, set the background, close the window again, and save what it saw as .DS_Store — then waits, with no limit, until that file appears. So every build stole keyboard focus with a window, two builds at once fought over the one Finder, a Finder showing hidden files stored every icon in the wrong place, and a window lost among others stalled the build until somebody closed it.

All Finder ever contributed was that one .DS_Store, and dmgbuild writes it directly. It mounts the image with -nobrowse, so Finder never learns the volume exists, writes the window, icon view, background alias and icon positions as records, then detaches and compresses. A build takes about five seconds, touches nothing on screen, and comes out the same whatever Finder is showing. Mounted beside Tauri's dmg, its records matched the ones Finder wrote; what differs is where the background sits, /.background.jpg rather than .background/jinbocho.jpg, and Finder's own bookmarks to the picture, which dmgbuild does not write and Finder does not need.

## Where the numbers come from

Measured on the Sequoia Mac mini on 2026-10-01, with a checkerboard of 200-point squares and a diamond touching each edge in the picture's place, so a clipped edge or an icon off its corner showed at a glance. Four things were learned that the code cannot say:

- The window size is the frame, title bar included. Showing all 800 points of picture takes 828, because Sequoia's title bar is 28 with the toolbar hidden. Tahoe redrew window chrome, and if its title bar is another height the bottom of the picture is clipped or letterboxed by the difference.
- The picture is drawn at its own size from the top left of the area under the title bar, and never scaled to fit the window.
- An icon's position is the center of its picture, measured from that same top left. The label hangs below and is not counted, so y 400 puts the disc's center on the middle line of the picture exactly.
- The window's position runs from the bottom left of the screen to the bottom left of the frame, with y up — Cocoa's coordinates, not AppleScript's. So no number means the same place on every screen, and tools that take a position from the top, create-dmg among them, convert it on the screen of the Mac that ran the build. A window whose top would run past the menu bar is clamped just beneath it, which is what happens on most laptops.

## The pins

dmgbuild is python, run through uvx, so nothing is installed and uv is the one requirement, on the mac that builds the installer and nowhere else. It is pinned, and so are ds_store and mac_alias, the two libraries that write the records and the alias measured above; dmgbuild itself asks only for some version or later of each, which would let a new release change the bytes of a dmg with nothing here changing.

The extension of Fuji.app is not hidden. Hiding it sets a FinderInfo attribute on the bundle, and codesign --verify --strict rejects a bundle carrying one. Tauri's dmg never managed to hide it either, and nothing is lost: the label follows each viewer's own Finder setting, Fuji.app with every extension shown and Fuji without, the same as every application in /Applications, none of which carries the flag.
*/

//every path is built from this file's own location, for the reason scripts.js gives at the root
const here    = fileURLToPath(new URL('.', import.meta.url))
const tauri   = join(here, 'src-tauri')
const bundled = join(tauri, 'target/release/bundle')//tauri leaves Fuji.app in macos/ here, and this puts the dmg in dmg/ beside it, where hash in scripts.js looks

const configuration = JSON.parse(readFileSync(join(tauri, 'tauri.conf.json'), 'utf8'))
const brandName     = configuration.productName//the name people read, which tauri names the .app with and this names the dmg and its volume; brand.js says how it and brandStem are used

const dmgbuild     = 'dmgbuild@1.6.7'                     //pinned together, for the reason the essay gives
const dependencies = ['ds_store==1.3.3', 'mac_alias==2.2.3']

//the window a mac user meets: the whole picture and nothing around it, with each icon centered in its half. every number is in CSS pixels, and the essay says how each was measured
const layout = {
	window:       {position: {x: 200, y: 200}, size: {width: 800, height: 828}},//800 of picture under a 28 title bar. the position is from the bottom left of the screen; 200 keeps the rhythm of the 200s below, and puts the top 172 down on a 1200-tall screen and clamped under the menu bar on a laptop
	iconSize:     128,//the label under it is 16, dmgbuild's default, which the json form of its settings cannot change
	background:   join(tauri, 'dmg/jinbocho.jpg'),//1600 pixels tagged at 144 dpi, so 800 CSS pixels on an sRGB panel and a Retina one alike
	volumeIcon:   join(tauri, 'icons/mac/icon.icns'),//the dock icon, the same file tauri gave the volume
	app:          {x: 200, y: 400},//the center of each half, and the middle line of the picture
	applications: {x: 600, y: 400},
}

//the name a dmg of version takes, which hash in scripts.js reads the version and architecture back out of
export function dmgFile(version) {
	let arch = {arm64: 'aarch64', x64: 'x64'}[process.arch]//spelled the way tauri spells it, since hash reads the architecture out of the filename and into the published sidecar
	if (!arch) throw new Error('no dmg architecture for ' + process.arch)
	return `${brandName}_${version}_${arch}.dmg`
}

//wrap the app at app in a dmg written to file, with dmgbuild's output going to output, which is anything spawn takes for a stream. pnpm installer wraps the app tauri just built, and the apple workspace wraps a copy it keeps, so this takes both paths rather than knowing either. a bottom gate: dmgbuild failing comes back as a result, never an exception
export async function dmgMake({app, file, output}) {
	//the settings in the json form dmgbuild reads, which is appdmg's, in a private temporary folder so two builds at once never share a file
	let scratch = mkdtempSync(join(tmpdir(), 'dmg-'))
	let settings = join(scratch, 'settings.json')
	writeFileSync(settings, JSON.stringify({
		'title':             brandName,
		'icon':              layout.volumeIcon,
		'background':        layout.background,
		'icon-size':         layout.iconSize,
		'compression-level': 9,//zlib's best, about six percent smaller than hdiutil's default for this app
		'window':            layout.window,
		'contents': [
			{type: 'file', path: app,             ...layout.app},
			{type: 'link', path: '/Applications', ...layout.applications},
		],
	}, null, '\t'))

	//uvx takes its own options before the tool and hands everything after it to the tool. --no-hidpi copies the picture as it is: without it, an @2x file beside jinbocho.jpg would make dmgbuild combine the two into a tiff, and the one 144 dpi jpeg already serves both kinds of panel
	let made = await new Promise(resolve => {
		let child = spawn('uvx', [
			...dependencies.flatMap(dependency => ['--with', dependency]), dmgbuild,
			'--no-hidpi', '-s', settings, brandName, file,
		], {stdio: ['ignore', output, output]})
		child.on('error', e => resolve({success: false, outcome: e.code == 'ENOENT' ? 'uvx is not on the path; the mac installer needs uv — brew install uv' : e.message}))
		child.on('close', code => resolve(code == 0 ? {success: true} : {success: false, outcome: `dmgbuild exited ${code}`}))
	})
	rmSync(scratch, {recursive: true, force: true})
	return made
}

async function main() {
	if (process.platform != 'darwin') return//a dmg is the mac's alone; windows and linux finish with tauri build
	let file = dmgFile(configuration.version)

	//empty the folder first, so a dmg from an earlier version never gives hash two candidates. a copy still mounted in Finder is safe: it keeps reading the old file, and opening the new one mounts it beside, as Fuji 1
	let folder = join(bundled, 'dmg')
	rmSync(folder, {recursive: true, force: true})
	mkdirSync(folder, {recursive: true})

	buildLog('dmg', 'started', file, '-', `with ${dmgbuild}, of the app tauri just signed with Fuji Desktop`)
	let made = await dmgMake({app: join(bundled, `macos/${brandName}.app`), file: join(folder, file), output: 'inherit'})
	if (!made.success) { buildLog('dmg', 'failed', file, '-', made.outcome); process.exitCode = 1; return }
	buildLog('dmg', 'finished', file, readSha256(join(folder, file)), 'in the bundle folder, where pnpm hash finds it')
}

//only when run, so the apple workspace can import dmgMake without making a dmg. said with realpath rather than import.meta.main, which node before 22.18 lacks, and would skip this in silence and leave the last build's dmg for hash to find
if (process.argv[1] && realpathSync(process.argv[1]) == fileURLToPath(import.meta.url)) main().catch(e => { console.error('🚧 Error:', e); process.exitCode = 1 })
