//./linux/build.js

import {execFileSync} from 'node:child_process'
import {cpSync, existsSync, mkdirSync, readdirSync, rmSync, statSync} from 'node:fs'
import {basename, join} from 'node:path'
import {fileURLToPath} from 'node:url'

/*
Fuji's linux packages, built on a mac through docker.

Fuji is developed on a mac and its linux users are somebody else's machines, so the linux packages have to come from somewhere. This workspace is that somewhere: three containers, no linux computer, and nothing to check out on a second machine and remember to keep current.

## The pipe flushes clean

Two layers, and only one of them lasts. An *image* is the toolchain — debian, node, rust, the webkit headers, or flatpak and its runtime — and it holds no source and no secrets. A *container* is one build: it is handed the source, makes a package, and is destroyed. Nothing carries between builds, so there is no warm target directory, no cargo registry, no node_modules from last time. Every run starts from the same known state, which is worth more here than speed: these packages are made a few times a year, not in a daily loop.

Reproducibility is what makes "clean" mean something. pnpm-lock.yaml and Cargo.lock are both in the whitelist and both enforced, so the versions that land are the ones the mac and the windows box already build with. Without them a fresh container would resolve every caret range in package.json against whatever the registry served that morning, and each build would differ from the last with nothing recording why.

## The whitelist, and why it is a whitelist

Four things go in — the root package.json, pnpm-workspace.yaml, pnpm-lock.yaml, and the desktop workspace. Nothing else, and the list is stated positively rather than as a set of exclusions, because a whitelist fails closed. The concrete thing it keeps out is the .env at the repository root, which holds the deploy host, both account names and the path to the installer upload key; that file has no business inside a container or baked into an image layer. .git goes the same way.

Neither the site workspace nor this one goes in, and nothing misses them. pnpm-workspace.yaml names all three and pnpm-lock.yaml has an importer for each, and pnpm simply does not find the two directories and carries on — verified, with --frozen-lockfile, installing 59 packages, all of them desktop's. So the shared lockfile gives reproducibility without dragging the website's dependency tree, or this workspace's own commands, into a build that wants neither.

Source is mounted read only. A container cannot write into the working tree on the mac; the one writable thing it is given is release/.

## The three containers

Two are the same image at two architectures. On apple silicon the arm64 one runs native and the amd64 one runs through rosetta, which is slower and still the right way round — an emulated build is cheaper than a second computer.

	fuji-tauri:arm64     .deb for aarch64          }  pnpm build-distro
	fuji-tauri:amd64     .deb and .rpm for x86_64   }
	fuji-flatpak:amd64   .flatpak for x86_64, built from the amd64 .deb     pnpm build-flatpak

The third consumes what the second produces, which is why `build` runs them in that order: the flatpak wraps the amd64 deb.

There was a fourth, an arch container that ran makepkg over an AUR recipe to prove the recipe worked. It was removed in September 2026. Arch's mirrors carry only today's version of every package, the container installed the recipe's dependencies at run time against a package list docker had frozen into the image, and the two disagreed within a day or two of every image build — a property of a rolling distribution rather than a bug in the recipe, and not worth carrying for a channel fuji never opened. Arch users take the flatpak, and linux-builds.md has the account.

## What this does not do

It builds and it does not deliver. Staging packages under their published names, hashing them, uploading them, submitting to flathub — none of that is here. `pnpm hash` and `pnpm upload` are scripts.js, which publishes and does not build, and this file is the other half of that split. What lands in release/ carries tauri's own filenames until hash stages it.

It also cannot smoke test. A container has no display, so the machines that actually run fuji are still the machines that run fuji.
*/

//every path is built from this file's own location, never from the working directory, for the reason the root scripts.js gives: the workspace a person is standing in is not a fact this file should read
const here    = fileURLToPath(new URL('.', import.meta.url))
const root    = join(here, '..')            //the monorepo, where the whitelist is gathered from
const stage   = join(here, '.stage')        //the whitelist copy, and the only thing a container reads
const release = join(here, 'release')       //what comes out, and the only thing a container writes

//the three toolchains. one file serves both architectures because the only difference is the platform docker is told to build for, and letting that be an argument rather than a second file is the same discipline the rust side uses for its three operating systems
const images = {
	'fuji-tauri:amd64':   {file: 'Dockerfile.tauri',   platform: 'linux/amd64'},
	'fuji-tauri:arm64':   {file: 'Dockerfile.tauri',   platform: 'linux/arm64'},
	'fuji-flatpak:amd64': {file: 'Dockerfile.flatpak', platform: 'linux/amd64'},
}

//what goes into a container, said positively. desktop/ is copied whole except for the names below, which are all things a build makes rather than things a build needs
const whitelist = ['package.json', 'pnpm-workspace.yaml', 'pnpm-lock.yaml', 'desktop']
const leaveBehind = ['node_modules', 'target', 'dist', 'release', '.DS_Store']

function run(args, options = {}) {//one place that shells out, so every docker invocation looks the same
	execFileSync('docker', args, {stdio: 'inherit', ...options})
}

//what a container runs is mounted rather than baked, so a change to a build step takes effect on the next run instead of on the next image build. it is also what keeps an image honestly the toolchain and nothing else — the two inside-*.sh scripts are this workspace's code, not debian's
function inside(script) { return ['--entrypoint', 'sh', '-v', `${join(here, script)}:/build.sh:ro`] }

function say(line) { console.log(line) }

//gather the whitelist into .stage. wiped first every time rather than synced, because a file deleted upstairs has to disappear down here too, and a copy is cheap next to what follows it
function stageSource() {
	say('==> staging source')
	rmSync(stage, {recursive: true, force: true})
	mkdirSync(stage, {recursive: true})

	for (let name of whitelist) {
		let from = join(root, name)
		if (!existsSync(from)) throw new Error('the whitelist names something that is not there: ' + from)
		cpSync(from, join(stage, name), {
			recursive: true,
			//node_modules and target are the two that matter: both are large, and both hold binaries compiled for macos that would be worse than useless to a linux container
			filter: source => !leaveBehind.includes(basename(source)),
		})
	}
	say('    ' + whitelist.join(', '))
}

function buildImage(tag) {
	let image = images[tag]
	say(`==> image ${tag}  (${image.platform})`)
	run(['build', '--platform', image.platform, '-f', join(here, image.file), '-t', tag, here])
}

function buildImages() { for (let tag of Object.keys(images)) buildImage(tag) }

//the two tauri runs. the bundle list is an argument rather than a change to tauri.conf.json, so what the mac and the windows box are told to build stays exactly as it was
function tauri(tag, platform, bundles) {
	say(`==> ${tag}  --bundles ${bundles}`)
	mkdirSync(release, {recursive: true})
	run(['run', '--rm', '--platform', platform,
		...inside('inside-tauri.sh'),
		'-v', `${stage}:/src:ro`,
		'-v', `${release}:/out`,
		tag, '/build.sh', bundles])
}

//--privileged is for one narrow thing: build-export validates the icon inside bubblewrap, which needs a namespace an ordinary container cannot make. inside-flatpak.sh carries the whole account, including why flatpak-builder is not what runs here
function flatpak() {
	let deb = newestIn(release, '.deb', 'amd64')
	if (!deb) throw new Error('the flatpak wraps the x86_64 deb and there is not one in release/ yet — run pnpm build-distro first')
	say(`==> fuji-flatpak:amd64  wrapping ${deb}`)
	run(['run', '--rm', '--platform', 'linux/amd64', '--privileged',
		...inside('inside-flatpak.sh'),
		'-v', `${release}:/out`,
		'fuji-flatpak:amd64', '/build.sh', deb])
}

/*
find a package in release/ by extension and architecture token. the filenames are tauri's own, which is deliberate — naming them for publication is the delivery half and belongs elsewhere.

That is also why only a capital Fuji counts. `pnpm hash` stages into this same folder, so after a release has been staged once, fuji.amd64.deb sits beside Fuji_0.1.0_amd64.deb and both end in the same three characters. The staged copy is the wrong input here — inside-flatpak.sh reads the version and the architecture straight out of a build's filename and a published name carries neither — so a build output is what this looks for, and tauri's leading capital is what tells them apart
*/
function newestIn(folder, extension, token) {
	if (!existsSync(folder)) return ''
	let names = readdirSync(folder)
		.filter(name => name.startsWith('Fuji') && name.endsWith(extension) && name.includes(token))
		.map(name => ({name, when: statSync(join(folder, name)).mtimeMs}))
		.sort((a, b) => b.when - a.when)
	return names.length ? names[0].name : ''
}

/*
The two steps `build` runs, in the order it runs them. They are grouped by container rather than by package format, and that grouping is forced rather than chosen: the rpm comes out of the same tauri run as the amd64 deb, which is the slow emulated one, so asking for it on its own would mean paying for that build twice.

`distro` is the word for those two together — a .deb and an .rpm are the distributions' own formats, installed into the system and leaning on its libraries, against flatpak's sandboxed bundle that carries its own.
*/
const steps = {
	//arm64 first on purpose: it runs native on apple silicon where amd64 runs emulated, so anything wrong with the image, the whitelist or the lockfile surfaces in a couple of minutes rather than twenty. the slow one is never worth discovering a typo in
	distro:  () => { stageSource(); tauri('fuji-tauri:arm64', 'linux/arm64', 'deb')
	                                tauri('fuji-tauri:amd64', 'linux/amd64', 'deb,rpm') },
	flatpak: () => flatpak(),
}

/*
Build the packages, after making sure the toolchains they need are current.

The images are rebuilt every time rather than being something a person has to remember. Docker's layer cache is what makes that free: with nothing changed it is about three seconds, and when a version inside a Dockerfile has moved — because somebody bumped rust and you pulled it — the affected image rebuilds itself and this build uses the new toolchain. Nobody has to know that happened, which is the point, because there is no way they could have.
*/
function build(only) {
	//the name is checked before anything runs, so a typo costs nothing rather than an image pass
	if (only && !steps[only]) throw new Error('no such step: ' + only + ' — say one of ' + Object.keys(steps).join(', '))

	buildImages()
	if (only) {
		steps[only]()
	} else {
		//order matters: the flatpak wraps the amd64 deb that distro makes
		steps.distro(); steps.flatpak()
	}
	report()
}

//what build says when it is done. not a command of its own: `pnpm hash` prints the same files a moment later with their hashes, so a person never needs to ask this question separately
function report() {
	say('')
	say('linux/release now holds:')
	if (!existsSync(release)) return say('    nothing')
	let names = readdirSync(release).sort()
	if (!names.length) return say('    nothing')
	for (let name of names) say('    ' + name.padEnd(34) + statSync(join(release, name)).size + ' bytes')
}

/*
The verbs package.json passes. build-images and stage are here for factoring rather than for typing — build runs the images itself, and stage is a window into what a container is handed, useful when something about the whitelist needs looking at.

Not named `prepare`, which was the first choice and is a trap: npm and pnpm treat prepare as a lifecycle script and run it on every install. A `prepare` here would mean that cloning this repository and running pnpm install tried to build ten gigabytes of docker images, and failed the install outright on a machine where docker was not running.
*/
const commands = {
	'build':        () => build(process.argv[3]),
	'build-images': () => buildImages(),
	'stage':        () => stageSource(),
}

function main() {
	let command = commands[process.argv[2]]
	if (!command) throw new Error('say which: ' + Object.keys(commands).join(', '))
	command()
}
try { main() } catch (e) { console.error('🚧 Error:', e.message || e); process.exitCode = 1 }
