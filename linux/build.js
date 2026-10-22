import {execFileSync, spawn, spawnSync} from 'node:child_process'
import {appendFileSync, cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync} from 'node:fs'
import {basename, join} from 'node:path'
import {createInterface} from 'node:readline'
import {fileURLToPath} from 'node:url'
import {buildStep, hashSha256} from '../build-log.js'

/*
Fuji's linux packages, built on a mac through docker.

Fuji is developed on a mac and its linux users are somebody else's machines, so the linux packages have to come from somewhere. This workspace is that somewhere: three containers, no linux computer, and nothing to check out on a second machine and remember to keep current.

## The pipe flushes clean

Two layers, and only one of them lasts. An *image* is the toolchain — debian, node, rust, the webkit headers, or flatpak and its runtime — and it holds no source and no secrets. A *container* is one build: it is handed the source, makes a package, and is destroyed. Nothing carries between builds, so there is no warm target directory, no cargo registry, no node_modules from last time. Every run starts from the same known state, which matters more here than speed: these packages are made a few times a year, not in a daily loop.

One thing carries, on purpose and under proof: the compiled media library, in cache/. Compiling it is twelve minutes under emulation and the recipe that makes it changes a few times a year, so `pnpm library` is the command that compiles it, in both containers, and keeps each architecture's library beside the record the recipe writes; `pnpm build` uses it again only when the record names the recipe it was handed by hash and the library still hashes to what the record says, under a folder named for a hash of the toolchain image's layers as well, so a bumped compiler is a miss, and refuses to build without one rather than compiling on its own. The essay in inside-tauri.py has the whole argument; here the point is that the cache is keyed by everything that could change the bytes, and deleting the folder and running `pnpm library` is the way to compile fresh.

## The ledger

Every library the containers compile gets an entry in one of the two library files for linux at the repository root, library-arm.txt and library-amd.txt, which git tracks: when it was compiled, from which sources checked against which hashes, with which compilers, under which recipe, and what came out. The recipe inside the container writes the entry beside the record, since it alone knows all of that, and this file appends it to the root file once the container has finished, because a container never writes into the working tree. The recipe's essay says what an entry holds and why the mac and windows append their own.

Reproducibility is what makes "clean" mean something. pnpm-lock.yaml and Cargo.lock are both in the whitelist and both enforced, so the versions that land are the ones the mac and the windows box already build with. Without them a fresh container would resolve every caret range in package.json against whatever the registry served that morning, and each build would differ from the last with nothing recording why.

## The whitelist, and why it is a whitelist

The whitelist below names what goes in, and nothing else does. The list is stated positively rather than as a set of exclusions, because a whitelist fails closed. The concrete thing it keeps out is the .env at the repository root, which holds the deploy host, both account names and the path to the installer upload key; that file has no business inside a container or baked into an image layer. .git goes the same way.

The other workspaces stay out, and nothing misses them. pnpm-workspace.yaml names every workspace and pnpm-lock.yaml has an importer for each, and pnpm simply does not find the missing directories and carries on, verified with --frozen-lockfile. So the shared lockfile gives reproducibility without dragging the website's dependency tree, or this workspace's own commands, into a build that wants neither.

Source is mounted read only. A container cannot write into the working tree on the mac; the one writable thing it is given is release/.

## The three containers

Two are the same image at two architectures. On apple silicon the arm64 one runs native and the amd64 one runs through rosetta, which is slower and still the right way round — an emulated build is cheaper than a second computer.

	fuji-tauri:arm64     .deb for aarch64          }  pnpm build-distro
	fuji-tauri:amd64     .deb and .rpm for x86_64   }
	fuji-flatpak:amd64   .flatpak for x86_64, built from the amd64 .deb     pnpm build-flatpak

The two tauri containers run at the same time, and the third waits for the second, since the flatpak wraps the amd64 deb. Running the two together hides the arm64 leg under the amd64 one, which is emulated and three times as long. What it hides is the idle time in the legs rather than their length, since two cargo builds that each want all eight cores only cover each other's single-threaded stretches, the copy in, the npm install and the final link; on the Mac mini on 2026-10-15, with the library cached, that was the difference between 8 minutes 46 seconds one after the other and 6 minutes 2 seconds together, with the machine otherwise quiet. The same run beside a sibling project's emulated build took 22 minutes, so one Mac builds one project's linux packages at a time. Each container's lines reach the terminal prefixed with its architecture, since two writers interleave otherwise, and a container that fails stops the other, so the native leg keeps its old job as the early warning: a wrong image or a broken lockfile fails arm64 in two minutes and the emulated leg is stopped rather than run on.

There was a fourth, an arch container that ran makepkg over an AUR recipe to prove the recipe worked. It was removed in September 2026. Arch's mirrors carry only today's version of every package, the container installed the recipe's dependencies at run time against a package list docker had frozen into the image, and the two disagreed within a day or two of every image build — a property of a rolling distribution rather than a bug in the recipe, and not worth carrying for a channel fuji never opened. Arch users take the flatpak.

## What this does not do

It builds and it does not deliver. Staging packages under their published names, hashing them, uploading them, submitting to flathub — none of that is here. `pnpm hash` and `pnpm upload` are scripts.js, which publishes and does not build, and this file is the other half of that split. What lands in release/ carries tauri's own filenames until hash stages it.

It also cannot smoke test. A container has no display, so the machines that actually run fuji are still the machines that run fuji.
*/

//every path is built from this file's own location, never from the working directory, for the reason the root scripts.js gives: the workspace a person is standing in is not a fact this file should read
const here    = fileURLToPath(new URL('.', import.meta.url))
const root    = join(here, '..')            //the monorepo, where the whitelist is gathered from
const stage   = join(here, '.stage')        //the whitelist copy, and the only thing a container reads
const release = join(here, 'release')       //what comes out, and the only thing a container writes
const cache   = join(here, 'cache')         //the compiled media library between runs, the one exception the essay above argues
const ledgers = {arm64: 'library-arm.txt', amd64: 'library-amd.txt'}//the two library files at the root, by the architecture debian names a container with

function readStem() {//the first name in Cargo.toml, which is the package's, found with a pattern the way scripts.js finds it, since one line of toml needs no parser
	let found = readFileSync(join(root, 'desktop/src-tauri/Cargo.toml'), 'utf8').match(/^name\s*=\s*"([^"]+)"/m)
	if (!found) throw new Error('Cargo.toml names no package')
	return found[1]
}

//the product's two names and its identifier, from the two files that set them, which the README's section The two names describes. read here because the flatpak container is given no source, so it is handed all three: brandName is productName in tauri.conf.json, which tauri names every package with; brandStem is the crate's name in Cargo.toml, the executable's; and the identifier, tauri.conf.json's too, is the name a flatpak is installed under
const configuration = JSON.parse(readFileSync(join(root, 'desktop/src-tauri/tauri.conf.json'), 'utf8'))
const brandName  = configuration.productName
const brandStem  = readStem()
const identifier = configuration.identifier

//the three toolchains. one file serves both architectures because the only difference is the platform docker is told to build for, and letting that be an argument rather than a second file is the same discipline the rust side uses for its three operating systems. each is tagged with brandStem, so a fork built on the same mac keeps images of its own
const images = {
	[`${brandStem}-tauri:amd64`]:   {file: 'Dockerfile.tauri',   platform: 'linux/amd64'},
	[`${brandStem}-tauri:arm64`]:   {file: 'Dockerfile.tauri',   platform: 'linux/arm64'},
	[`${brandStem}-flatpak:amd64`]: {file: 'Dockerfile.flatpak', platform: 'linux/amd64'},
}

//what goes into a container, said positively. desktop/ and scout/ are copied whole except for the names below, which are all things a build makes rather than things a build needs
const whitelist = ['package.json', 'pnpm-workspace.yaml', 'pnpm-lock.yaml', 'desktop', 'scout']//scout because fuji's rust depends on it by path
const leaveBehind = ['node_modules', 'target', 'dist', 'release', '.DS_Store', 'corpus', 'artifacts']//corpus and artifacts are what pnpm fuzz leaves in scout/fuzz, thousands of small files a build never reads

function run(args, options = {}) {//one place that shells out, so every docker invocation looks the same
	execFileSync('docker', args, {stdio: 'inherit', ...options})
}

//a container run beside another: started with a name, its lines prefixed so two containers can share a terminal, and resolved as a result rather than thrown, so the caller sees both outcomes. a bottom gate: docker's exit code becomes a value
function container(name, prefix, args) {
	let child = spawn('docker', ['run', '--rm', '--name', name, ...args], {stdio: ['ignore', 'pipe', 'pipe']})
	for (let stream of [child.stdout, child.stderr]) createInterface({input: stream}).on('line', line => process.stdout.write(`${prefix} | ${line}\n`))
	return new Promise(resolve => {
		child.on('error', e => resolve({success: false, outcome: `docker could not start: ${e.message}`}))
		child.on('close', code => resolve(code == 0 ? {success: true} : {success: false, outcome: `${prefix} exited ${code}`}))
	})
}

//two containers at once, and the other stopped as soon as one fails, so a failure found by the fast native leg never leaves the slow emulated one running on; both results come back, and a failure throws with both outcomes
async function together(first, second) {
	let names = [first.name, second.name]
	let stopOthers = name => { for (let other of names) if (other != name) spawnSync('docker', ['stop', other], {stdio: 'ignore'}) }//stopping a container that has already exited is a harmless error
	let watched = [first, second].map(one => container(one.name, one.prefix, one.args).then(result => { if (!result.success) stopOthers(one.name); return result }))
	let results = await Promise.all(watched)
	let failed = results.filter(result => !result.success)
	if (failed.length) throw new Error(failed.map(result => result.outcome).join('; '))
}

//what a container runs is mounted rather than baked, so a change to a build step takes effect on the next run instead of on the next image build. it is also what keeps an image the toolchain and nothing else — the two inside-*.py scripts are this workspace's code, not debian's
function inside(script) { return ['--entrypoint', 'python3', '-v', `${join(here, script)}:/build.py:ro`] }

//what an image is, as a hash of its layers, which names the cache folder beside the architecture, so a rebuilt toolchain never serves a library the old one compiled. not the image's id: docker gives an image a new id on every build, even with every layer cached, three ids from three builds of one unchanged Dockerfile on the Mac mini on 2026-10-15, while the layer digests stayed one, since those are the toolchain's own content
async function imageKey(tag) { return (await hashSha256(Buffer.from(execFileSync('docker', ['image', 'inspect', '--format', '{{join .RootFS.Layers ","}}', tag], {encoding: 'utf8'}).trim()))).slice(0, 12) }

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

//one tauri container as together runs it, in one of inside-tauri.py's two modes: the word library compiles the media library into the cache, and a bundle list builds those packages from it. the bundle list is an argument rather than a change to tauri.conf.json, so what the mac and the windows box are told to build stays exactly as it was. the name carries this process's id, so a run killed partway leaves no name for the next run to collide with
async function tauri(architecture, mode) {
	let tag = `${brandStem}-tauri:${architecture}`
	say(`==> ${tag}  ${mode == 'library' ? 'the media library' : '--bundles ' + mode}`)
	mkdirSync(release, {recursive: true})
	mkdirSync(cache, {recursive: true})
	return {name: `${brandStem}-${architecture}-${process.pid}`, prefix: architecture, args: ['--platform', `linux/${architecture}`,
		...inside('inside-tauri.py'),
		'-v', `${stage}:/src:ro`,
		'-v', `${release}:/out`,
		'-v', `${cache}:/cache`,
		'-e', `IMAGE=${await imageKey(tag)}`,
		tag, '/build.py', mode]}
}

//append one architecture's ledger entry, written out by its container, to its library file at the root, after a blank line when the file holds entries already; called for arm64 then amd64 once both containers have finished, so the files read the same whichever leg landed first
function ledgerAppend(architecture) {
	let entry = readFileSync(join(release, `library.${architecture}.txt`), 'utf8')
	let ledger = join(root, ledgers[architecture])
	appendFileSync(ledger, (existsSync(ledger) && statSync(ledger).size > 0 ? '\n' : '') + entry)
	say(`    ${ledgers[architecture]} has the entry`)
}

//--privileged is for one narrow thing: build-export validates the icon inside bubblewrap, which needs a namespace an ordinary container cannot make. inside-flatpak.py carries the whole account, including why flatpak-builder is not what runs here
function flatpak() {
	let deb = newestIn(release, '.deb', 'amd64')
	if (!deb) throw new Error('the flatpak wraps the x86_64 deb and there is not one in release/ yet — run pnpm build-distro first')
	say(`==> ${brandStem}-flatpak:amd64  wrapping ${deb}`)
	run(['run', '--rm', '--platform', 'linux/amd64', '--privileged',
		...inside('inside-flatpak.py'),
		'-v', `${release}:/out`,
		`${brandStem}-flatpak:amd64`, '/build.py', deb, brandName, brandStem, identifier])
}

/*
find a package in release/ by extension and architecture token. the filenames are tauri's own, which is deliberate — naming them for publication is the delivery half and belongs elsewhere.

That is also why only a name beginning with brandName and an underscore counts. `pnpm hash` stages into this same folder, so after a release has been staged once, fuji.amd64.deb sits beside Fuji_0.1.0_amd64.deb and both end in the same three characters. The staged copy is the wrong input here — inside-flatpak.py reads the version and the architecture straight out of a build's filename and a published name carries neither — so a build output is what this looks for. Tauri names a build brandName, an underscore and the version, where a staged copy is brandStem and a dot, and the underscore tells them apart even for a product whose two names are the same string, where the capital that separates Fuji from fuji would not
*/
function newestIn(folder, extension, token) {
	if (!existsSync(folder)) return ''
	let names = readdirSync(folder)
		.filter(name => name.startsWith(`${brandName}_`) && name.endsWith(extension) && name.includes(token))
		.map(name => ({name, when: statSync(join(folder, name)).mtimeMs}))
		.sort((a, b) => b.when - a.when)
	return names.length ? names[0].name : ''
}

/*
The two steps `build` runs, in the order it runs them. They are grouped by container rather than by package format, and that grouping is forced rather than chosen: the rpm comes out of the same tauri run as the amd64 deb, which is the slow emulated one, so asking for it on its own would mean paying for that build twice.

`distro` is the word for those two together — a .deb and an .rpm are the distributions' own formats, installed into the system and leaning on its libraries, against flatpak's sandboxed bundle that carries its own.
*/
const steps = {
	//both tauri containers at once, the native arm64 leg finishing first and stopping the emulated one if it fails, which the essay above says more about
	distro:  async () => { stageSource(); await together(await tauri('arm64', 'deb'), await tauri('amd64', 'deb,rpm')) },
	flatpak: async () => flatpak(),
	library: async () => { stageSource(); await together(await tauri('arm64', 'library'), await tauri('amd64', 'library')); ledgerAppend('arm64'); ledgerAppend('amd64') },
}

/*
Build the packages, after making sure the toolchains they need are current.

The images are rebuilt every time rather than being something a person has to remember. Docker's layer cache is what makes that free: with nothing changed it is about three seconds, and when a version inside a Dockerfile has moved — because somebody bumped rust and you pulled it — the affected image rebuilds itself and this build uses the new toolchain. Nobody has to know that happened, and there is no way they could have.
*/
async function build(only) {
	//the name is checked before anything runs, so a typo costs nothing rather than an image pass
	if (only && !steps[only]) throw new Error('no such step: ' + only + ' — say one of ' + Object.keys(steps).join(', '))

	buildImages()
	if (only) {
		await steps[only]()
	} else {
		//order matters: the flatpak wraps the amd64 deb that distro makes
		await steps.distro(); await steps.flatpak()
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
The verbs package.json passes. build makes the packages from the library in the cache, and library compiles the library into the cache, which is the twenty-minute run and the one that writes the ledger. build-images and stage are here for factoring rather than for typing — build and library run the images themselves, and stage is a window into what a container is handed, useful when something about the whitelist needs looking at.

Not named `prepare`, which was the first choice and is a trap: npm and pnpm treat prepare as a lifecycle script and run it on every install. A `prepare` here would mean that cloning this repository and running pnpm install tried to build ten gigabytes of docker images, and failed the install outright on a machine where docker was not running.
*/
const commands = {
	'build':        () => buildStep('linux', process.argv[3] || '-', '', async () => { await build(process.argv[3]); return {value: '-', words: 'the packages in linux/release'} }),
	'library':      () => buildStep('library', 'linux', '', async () => { buildImages(); await steps.library(); return {value: '-', words: 'both libraries in linux/cache, with their entries in the ledger'} }),
	'build-images': () => buildImages(),
	'stage':        () => stageSource(),
}

async function main() {
	let command = commands[process.argv[2]]
	if (!command) throw new Error('say which: ' + Object.keys(commands).join(', '))
	await command()
}
main().catch(e => { console.error('🚧 Error:', e.message || e); process.exitCode = 1 })
