//./scripts.js

import {execFile, execFileSync} from 'node:child_process'
import {createHash} from 'node:crypto'
import {copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync} from 'node:fs'
import {dirname, join} from 'node:path'
import {fileURLToPath} from 'node:url'

/*
The publishing pipeline for all three workspaces, in one file at the monorepo root. Everything the package.json scripts do beyond calling tauri, vitepress or docker is here, reached by a verb: reveal, hash, upload-installer, upload-site, icons-collect.

**This file publishes; it does not build.** The desktop workspace builds with tauri, the linux workspace builds in containers, and both then call in here to stage, hash and send — which is why hashing and uploading exist once rather than once per workspace. `linux/build.js` is the other half of that split and knows nothing about publishing.

Two machines publish fuji. Windows sends the exe. The mac sends the dmg it built natively and the four linux packages it built in docker. So a command means the same thing everywhere while doing different work underneath: `pnpm hash` is one package in desktop on windows and four in linux on the mac, and nobody has to remember which computer they are sitting at. What differs is passed as --source by the workspace that asked.

One file rather than one per workspace, because the facts worth keeping in one place cross the workspace boundary. The published name is the clearest case: tauri writes Fuji_0.1.0_aarch64.dmg, which is right for a build directory and wrong for a download link, so hash stages it as fuji.dmg and a link in a page survives the next version. That name is written by the build side and shipped by the site side, and when these were separate scripts each spelled it out for itself. The targets table below is now the only place any of it is said.

Living at the root also settles the working-directory question by force rather than by discipline. Both workspaces call this file, each from its own folder, so nothing here can be relative to wherever node started; every path is built from this file's own location. The scripts this replaced each had their own answer to that, every one of them correct only because pnpm happened to run it from the right place.

It imports node builtins and nothing else, and has to: the root package.json has no dependencies, and node_modules belongs to the workspaces below it.
*/

/*
Every artifact fuji publishes, and the one place any of it is said.

This was a table of three platforms keyed by process.platform until the linux workspace arrived, and the change is worth understanding rather than skimming. A platform table quietly assumed that the machine running a command is the machine that made the file — true when each operating system built its own installer, and false the moment a mac started building linux packages in docker. So the key is now the artifact rather than the computer, and which machine can make a thing is a separate question asked by whatMachineMakes below.

source says where the built file is found: 'bundle' is tauri's own output under the desktop workspace, 'linux' is what the containers left in linux/release.

publish is the name the file takes on the server, and sidecar is the name of the json beside it. They are the same string throughout, deliberately.

The rule for a published name: **every linux package states its architecture, and none carries a version.** macOS and Windows ship one architecture each by decision rather than by accident — there is no intel build and no ARM windows build — so fuji.dmg and fuji.exe need no token, and both names are already published and linked. Linux is where architectures multiply, so every name there says which machine it is for, including the formats that have only one build today. That costs a few characters and means no name ever has to change when an aarch64 flatpak or an ARM rpm turns up.

That is a decision worth not relitigating. A versioned filename says what it is when it is sitting in somebody's downloads folder, and costs more than it is worth: a link anyone posts pins that version forever, so it either serves stale software or 404s on the next release, and neither failure announces itself. A stable name is overwritten in place, so every link ever shared keeps handing people the current build. What version a download is belongs on the page, which reads it from the sidecar.
*/
const targets = {
	//the two an operating system builds for itself
	'dmg':     {source: 'bundle', folder: 'dmg',  suffix: '.dmg',       publish: 'fuji.dmg', sidecar: 'fuji.dmg'},
	'exe':     {source: 'bundle', folder: 'nsis', suffix: '-setup.exe', publish: 'fuji.exe', sidecar: 'fuji.exe'},

	//the four the linux workspace builds in containers, and every one names its architecture. giving the ARM deb the bare name fuji.deb is the tempting mistake here, since it is the raspberry pi link: beside fuji.amd64.deb an unadorned name reads like the ordinary choice while being the rarer one, which is a trap laid for the majority. no bare linux name also means none has to be renamed, and no link broken, when a second architecture of some format turns up
	'deb-arm64':   {source: 'linux', match: /_(arm64)\.deb$/,      publish: 'fuji.arm64.deb',      sidecar: 'fuji.arm64.deb'},
	'deb-x64':     {source: 'linux', match: /_(amd64)\.deb$/,      publish: 'fuji.amd64.deb',      sidecar: 'fuji.amd64.deb'},
	'rpm-x64':     {source: 'linux', match: /\.(x86_64)\.rpm$/,    publish: 'fuji.x86_64.rpm',     sidecar: 'fuji.x86_64.rpm'},
	'flatpak-x64': {source: 'linux', match: /_(x86_64)\.flatpak$/, publish: 'fuji.x86_64.flatpak', sidecar: 'fuji.x86_64.flatpak'},
}

/*
Which targets this computer stages and sends. Two machines publish fuji and no others: the mac makes its own dmg and, through docker, every linux package; windows makes the exe.

Linux is deliberately absent, and its absence is the simpler answer rather than an omission. Somebody can clone this repository on ubuntu or raspberry pi os and run pnpm installer in the desktop workspace, and they will get a package built for the machine they are sitting at — that is development, and it works. What they cannot do is stage and upload it, because a published package comes from the mac where all four are built together against one base image and one lockfile. Teaching this file a third place to look for a built file, so that a borrowed linux box could publish one package out of four, would buy a case nobody has and cost a branch in every function below.
*/
const machines = {
	darwin: ['dmg', 'deb-arm64', 'deb-x64', 'rpm-x64', 'flatpak-x64'],
	win32:  ['exe'],
}

function whatMachineMakes() {
	let found = machines[process.platform]
	if (!found) throw new Error(
		`fuji does not publish from ${process.platform}. The dmg and every linux package are staged on the mac, ` +
		`the exe on windows. Building here for your own use is a different thing and works: pnpm installer in desktop.`)
	return found
}

function readTarget(name) {
	let found = targets[name]
	if (!found) throw new Error(`no such target: ${name} — say one of ${Object.keys(targets).join(', ')}`)
	return found
}

/*
Which targets a command acts on, decided in one place because hash and upload must agree.

Named targets win and are an instruction: asking for one that is not built is an error. With none named it is everything this machine makes, narrowed by --source to the workspace that asked — which is what lets `pnpm hash` mean "the dmg" in desktop and "the four packages" in linux while being the same word and the same code. Run from the root with neither, it means everything this computer makes.
*/
function chosenTargets() {
	let args = process.argv.slice(3)
	let named = args.filter(a => !a.startsWith('-'))
	if (named.length) return {names: named, demanded: true}

	let source = (args.find(a => a.startsWith('--source=')) || '').split('=')[1]
	let names = whatMachineMakes()
	if (source) names = names.filter(name => readTarget(name).source == source)
	return {names, demanded: false}
}

//where a graphical file manager gets pointed, per platform rather than per target
const openers = {darwin: 'open', win32: 'explorer', linux: 'xdg-open'}

//every path is built from this file's own location, never from the working directory, because both workspaces call this file and each calls it from its own folder
const root = fileURLToPath(new URL('.', import.meta.url))
const configurationFile = join(root, 'desktop/src-tauri/tauri.conf.json')//the file that named the bundle
const bundled = join(root, 'desktop/src-tauri/target/release/bundle')    //where tauri leaves what it built
const staged = {                                                         //where hash puts a package and its sidecar, and where upload looks for them
	bundle: join(root, 'desktop/release'),                                //the dmg and the exe, copied out from under tauri's versioned name
	linux:  join(root, 'linux/release'),                                 //the four the containers made, already sitting where they were written
}
const icons   = join(root, 'desktop/src-tauri/icons')                    //committed artwork, generated rather than drawn
const built   = join(root, 'site/docs/.vitepress/dist')                  //what vitepress builds

let server//the destination, filled by readServer before either upload runs

//open the graphical file manager on this platform's finished installer, so it can be double-clicked the way a person who downloaded it would. that is a different and stronger test than starting a built binary in place: an installer has a first-run experience — the publisher warning, the wizard, where the application ends up — and none of that happens otherwise
function reveal() {
	let name = process.argv[3] || whatMachineMakes()[0]
	let target = readTarget(name)
	//the folder, not the file: a filename carries the version and would need editing every release
	let folder = target.source == 'bundle' ? join(bundled, target.folder) : staged.linux
	if (!existsSync(folder)) throw new Error('nothing built yet at ' + folder + ', run pnpm installer first')

	console.log('opening  ' + folder)
	//an absolute path, because explorer resolves a relative one against its own working directory rather than ours. and explorer answers 1 even when it did open the window, so its exit means nothing
	execFile(openers[process.platform], [folder], error => {
		if (error && process.platform != 'win32') console.error('could not open the file manager: ' + error.message)
	})
}

function readVersion() {//the version comes from the same file that named the bundle, so the two cannot disagree
	let version = JSON.parse(readFileSync(configurationFile, 'utf8')).version
	if (!version) throw new Error('tauri.conf.json has no version')
	return version
}

//find the one file a target describes, and say which architecture it turned out to be. read from the filename rather than from the machine running this, so every number describes the file that exists
function findBuilt(target, version) {
	if (target.source == 'bundle') {
		let folder = join(bundled, target.folder)
		if (!existsSync(folder)) return false
		let prefix = `Fuji_${version}_`
		let names = readdirSync(folder).filter(n => n.startsWith(prefix) && n.endsWith(target.suffix))
		if (names.length > 1) throw new Error(`expected one ${prefix}*${target.suffix} in ${folder}, found ${names.length}: ${names.join(', ')}`)
		if (!names.length) return false
		return {folder, file: names[0], arch: names[0].slice(prefix.length, names[0].length - target.suffix.length)}
	}

	//a container wrote this one, under whatever name tauri or flatpak chose. the regex does two jobs and the parentheses are not decoration: the group is where the architecture comes from, so a sidecar describes the file that exists rather than repeating something this table already believed
	let folder = staged.linux
	if (!existsSync(folder)) return false
	//the containers write here and hash stages here too, so by the second run a published copy is sitting beside the build it came from — and fuji.x86_64.rpm matches the same pattern Fuji-0.1.0-1.x86_64.rpm does. Skipping every published name is what makes hash repeatable rather than a once-per-clean thing
	let published = new Set(Object.values(targets).map(t => t.publish))
	let names = readdirSync(folder).filter(n => target.match.test(n) && !published.has(n))
	if (names.length > 1) throw new Error(`expected one ${target.match} in ${folder}, found ${names.length}: ${names.join(', ')}`)
	if (!names.length) return false
	return {folder, file: names[0], arch: names[0].match(target.match)[1]}
}

/*
Stage what this machine built and write a sidecar beside each, building nothing.

Two things a sidecar must not lie about, and each is read rather than assumed: the version comes from tauri.conf.json, the same file that named the bundle; the architecture comes out of the built file's own name, so it describes the file that exists rather than the computer that ran this.

Named with no arguments this means "everything this machine makes", which is one thing on windows and five on the mac. A target that has not been built yet is reported and skipped rather than thrown on, because staging a dmg should not fail merely because nobody has run the linux containers today — but a target asked for by name is an instruction, so that one throws.
*/
function hash() {
	let {names, demanded} = chosenTargets()
	let version = readVersion()
	for (let name of names) hashOne(name, version, demanded)
}

function hashOne(name, version, demanded) {
	let target = readTarget(name)
	let built = findBuilt(target, version)
	if (!built) {
		if (demanded) throw new Error(`${name}: nothing built to stage — run the build that makes it first`)
		console.log(`skipped ${name} — nothing built yet`)
		return false
	}

	let stage = staged[target.source]
	mkdirSync(stage, {recursive: true})
	let published = target.publish
	let destination = join(stage, published)

	//copy first, then measure what landed, so every number describes the file the site will ship. a container's output is already sitting in the staging folder under its published name, and copying a file onto itself is both pointless and an error
	if (join(built.folder, built.file) != destination) copyFileSync(join(built.folder, built.file), destination)
	let bytes = readFileSync(destination)//whole file into memory; a package is a few megabytes and streaming would buy nothing

	let sidecar = {
		file: published,
		version,
		arch: built.arch,
		bytes: bytes.length,
		sha256: createHash('sha256').update(bytes).digest('hex'),
		date: new Date().toISOString().slice(0, 10),//iso, because the home page sorts these as text to find the earliest build
	}
	writeFileSync(join(stage, target.sidecar + '.json'), JSON.stringify(sidecar, null, '\t') + '\n')

	/*
	hash, size, filename — and the hash whole, because a cropped hash cannot check a download. No leading verb: the command is called hash, so saying "staged" on every line is a word that carries nothing, and the lines can be counted by looking at them rather than being totalled underneath. Nothing is padded: every package fuji builds is a few megabytes, so the byte counts are the same width and the columns line up on their own. If that ever stops being true the columns drift a little, which costs less than machinery to prevent it.
	*/
	console.log(`${sidecar.sha256}  ${sidecar.bytes} bytes  ${published}`)
	return true
}

/*
Both uploads ship finished files and do no thinking about them: the site was built by vitepress, the installer and its sidecar by hash above, on the machine that could build them.

Two destinations, and the difference between them is the point. The site goes as an account that administers the server, over rsync, which mirrors with --delete so a file dropped from the build leaves the server too. The installers go as a second account with no shell at all: chrooted to the hostname's directory and able to speak only the SFTP file protocol. That way a compromise of the machine that builds an installer could overwrite the installers and their sidecars and nothing else. rsync is unavailable to such an account by construction, since rsync works by running a program on the far side — scp is what remains, and since OpenSSH 9.0 scp transfers over SFTP anyway, so the two agree on the wire.

The server answers one hostname from two directories: a request that misses the site's directory falls through to the downloads directory. That is why an installer sits at the apex, like fujidesktop.app/fuji.exe, rather than under a downloads path, and why a site deploy has no way to reach an installer.

## Running this against your own server

The destination lives in .env at the monorepo root, which is gitignored, so a fresh clone will not have one and readServer will say which values are missing and stop. Write it yourself — seven values and no logic:

	DEPLOY_HOST=files.example.com
	DEPLOY_PORT=22
	DEPLOY_SITE_USER=deploy
	DEPLOY_SITE_PATH=/var/www/example.com/site/
	DEPLOY_FILES_USER=upload
	DEPLOY_FILES_PATH=/downloads/
	DEPLOY_FILES_KEY=/home/you/.ssh/upload_ed25519

On an EC2 instance or a DigitalOcean droplet the ordinary account is something like ubuntu or deploy, the host can be an IP like 203.0.113.10, and the port is 22.

The two paths look nothing alike, which is the one thing here that surprises people. DEPLOY_SITE_PATH is absolute because that account sees the whole filesystem. DEPLOY_FILES_PATH is short because the account that uses it is chrooted by sshd, so its own directory is its filesystem root and the path has to be written as that account sees it. Give it the full path `ls` would show you elsewhere and scp fails with no such file or directory — correctly, since inside the chroot there is no such path, and it costs a confusing half hour to work out why. Using one ordinary unchrooted account for both works and is the simplest thing that runs; splitting them, so the machine that builds an installer holds credentials that cannot administer anything, is worth the extra setup once a release is public.

Keep the trailing slashes: rsync ignores one on a destination, but scp given a path that does not exist yet will write a file by that name, and the slash turns that into an error instead. DEPLOY_FILES_KEY names the installer account's own key rather than letting ssh offer whatever it finds, which matters with two accounts, since the default identity is the administrative one.

The file sits at the monorepo root rather than inside a workspace on purpose. Vite reads a .env inside the workspace it builds and copies any VITE_-prefixed value into the client bundle, which would publish these values on the website itself. At the root it is outside what Vite looks at, so that hazard stops existing rather than becoming something to remember — which is also the reason not to move it down.

Node does not read .env on its own; the package.json scripts pass --env-file-if-exists, which is why these run as pnpm upload rather than node scripts.js. Run it the second way, or leave the file unwritten, and the values arrive empty, which readServer catches and explains. The tolerant spelling of the flag is deliberate: plain --env-file makes node refuse to start when the file is absent, printing one line and never reaching that message.
*/

function readServer() {//gather the destination from the environment, naming whatever the scripts did not fill
	let required = [
		'DEPLOY_HOST',       'DEPLOY_PORT',
		'DEPLOY_SITE_USER',  'DEPLOY_SITE_PATH',
		'DEPLOY_FILES_USER', 'DEPLOY_FILES_PATH', 'DEPLOY_FILES_KEY',
	]
	let missing = required.filter(name => !process.env[name])
	if (missing.length > 0) throw new Error(`.env at the monorepo root is missing ${missing.join(', ')} — see the essay in scripts.js, and run this as pnpm rather than node, so the env file is passed`)

	return {
		host:     process.env.DEPLOY_HOST,
		port:     process.env.DEPLOY_PORT,
		siteUser: process.env.DEPLOY_SITE_USER,//administers the server; ships the site over rsync
		sitePath: process.env.DEPLOY_SITE_PATH,
		fujiUser: process.env.DEPLOY_FILES_USER,//no shell, chrooted, sftp only; ships installers over scp
		fujiPath: process.env.DEPLOY_FILES_PATH,
		fujiKey:  process.env.DEPLOY_FILES_KEY,//that account's own key, so an installer upload never offers the admin one
	}
}

function uploadSite() {//ship the built site, mirroring so a file dropped from the build leaves the server too
	if (process.platform == 'win32') throw new Error('the site ships from the Mac — windows has no rsync, and this would fail with a spawn error rather than a sentence')
	if (!existsSync(built)) throw new Error('no build to upload, run pnpm build first: ' + built)

	//refuse to ship a sidecar from inside the site. the server answers one hostname from two directories and checks the site's first, so a fuji.*.json in this build would shadow the real one in downloads and pin the download page to whatever hash it holds until the next deploy. one gets here by sitting in docs/public, which a retired fixtures script used to copy them into for local development — the files are gitignored, so a machine that ran it still has them and no other machine can tell
	let shadowing = readdirSync(built).filter(name => name.startsWith('fuji.') && name.endsWith('.json'))
	if (shadowing.length > 0) throw new Error(`${shadowing.join(', ')} would ship inside the site and shadow the real sidecar on the server — delete site/docs/public/fuji.*.json on this machine, then build again`)

	server = readServer()

	//--delete is load-bearing: without it every previous deploy's hashed assets accumulate on the server forever. it is also the reason the installers live in a directory this cannot reach
	execFileSync('rsync', [
		'-avz', '--delete',
		built + '/',//trailing slash: copy the contents, not the directory itself
		`${server.siteUser}@${server.host}:${server.sitePath}`,
		'-e', `ssh -p ${server.port}`,//rsync spells the port inside the ssh command, lowercase -p
	], {stdio: 'inherit'})
}

//Ship what this machine staged: every package it makes, and the sidecar beside each. Named with no arguments this is one file on windows and five on the mac, and the same word means "this machine's work" in both places — which is the whole reason a command is not named per platform.
function uploadInstaller() {
	let {names, demanded} = chosenTargets()
	let version = readVersion()

	//gather and check everything before asking for credentials, so a missing .env is never what hides a stale sidecar, and a half-finished release is never half uploaded
	let sending = []
	for (let name of names) {
		let target = readTarget(name)
		let stage = staged[target.source]
		let sidecarName = target.sidecar + '.json'
		if (!existsSync(join(stage, sidecarName))) {
			if (demanded) throw new Error(`${name}: no sidecar staged — run pnpm hash on this machine first`)
			console.log(`skipped  ${name.padEnd(12)} not staged`)
			continue
		}
		let sidecar = JSON.parse(readFileSync(join(stage, sidecarName), 'utf8'))
		checkSidecar(name, stage, sidecar, sidecarName, version)
		sending.push({name, stage, file: sidecar.file, sidecarName})
	}
	if (!sending.length) throw new Error('nothing staged to upload — run pnpm hash first')

	server = readServer()
	reportTransport()
	//the package before its sidecar, every time, so the page never fetches a hash for a file still arriving
	for (let one of sending) { send(one.stage, one.file); send(one.stage, one.sidecarName) }
	console.log(`sent     ${sending.length} package${sending.length == 1 ? '' : 's'} and ${sending.length} sidecar${sending.length == 1 ? '' : 's'}`)
}

function send(folder, name) {//copy one file into the downloads directory as the restricted account
	//run from the staging directory and name the file bare. a windows absolute path contains the colon scp uses to split host from path, and a bare filename makes that question stop existing
	execFileSync('scp', [
		'-P', server.port,//scp spells the port capital -P, unlike ssh and rsync
		'-o', 'IdentitiesOnly=yes',//offer only the key named below; without this, keys loaded in ssh-agent are offered too, and can go first, so the admin key could be tried
		'-i', server.fujiKey,//with IdentitiesOnly above, the admin key is never tried
		name,
		`${server.fujiUser}@${server.host}:${server.fujiPath}`,
	], {stdio: 'inherit', cwd: folder})
}

//catch a sidecar left from a previous release, which would otherwise publish a hash describing a file nobody can download. it cannot see a stale build — a sidecar truthfully describing a package compiled before the last source change passes — and it defends against nobody who can write here
function checkSidecar(name, stage, sidecar, sidecarName, version) {
	if (!existsSync(join(stage, sidecar.file))) throw new Error(`${name}: ${sidecarName} names ${sidecar.file}, which is not beside it`)
	let bytes = readFileSync(join(stage, sidecar.file))
	let sha256 = createHash('sha256').update(bytes).digest('hex')

	if (sidecar.version != version)    throw new Error(`${name}: sidecar says version ${sidecar.version}, tauri.conf.json says ${version}; re-run pnpm hash`)
	if (sidecar.bytes != bytes.length) throw new Error(`${name}: sidecar says ${sidecar.bytes} bytes, the file is ${bytes.length}; re-run pnpm hash`)
	if (sidecar.sha256 != sha256)      throw new Error(`${name}: sidecar hash does not describe this file; re-run pnpm hash`)

	console.log(`checked  ${sidecar.sha256}  ${bytes.length} bytes  ${sidecar.file}  ${sidecar.version} ${sidecar.arch}`)
}

//say which scp is about to run. windows carries two openssh installs, and which one a script gets depends on the shell it was launched from; they differ in how they judge a private key's permissions, so this is the first thing anyone will want to know when authentication fails. diagnostic only, so it never throws — a missing scp should be reported by the transfer, not by this
function reportTransport() {
	try {
		let found = execFileSync(process.platform == 'win32' ? 'where.exe' : 'which', ['scp'], {encoding: 'utf8'}).trim().split('\n')[0]
		console.log('using    ' + found)
	} catch (e) {
		console.log('using    could not locate scp')//say so and carry on; the transfer below reports the real failure
	}
}

//the three tauri icon runs stay in package.json, where pnpm is what puts the tauri cli on the path; this is the part after them. each run writes a folder of its own, and these are the files that have to come out from under a generated name and sit where tauri.conf.json and the windows manifest expect them. icon.md is the whole subject, including why these are committed artifacts
function iconsCollect() {
	let copies = [
		['.mac/icon.icns',              'mac/icon.icns'],        //the dock icon, inset to apple's grid, kept where the shared run cannot overwrite it
		['.tile/Square284x284Logo.png', 'tile/tile-medium.png'], //the start menu tile, renamed because the manifest attribute names the size and the file is just an asset
		['.tile/Square142x142Logo.png', 'tile/tile-small.png'],
	]
	for (let [from, to] of copies) {
		mkdirSync(dirname(join(icons, to)), {recursive: true})
		copyFileSync(join(icons, from), join(icons, to))
		console.log('copied   ' + to)
	}
}

//the verb package.json passes. these are spelled out rather than shared, because the script names a person types differ between the two workspaces on purpose — pnpm upload means the installer in desktop and the site in site — and this file should never have to guess which one called it
const commands = {
	'reveal':           reveal,
	'hash':             hash,
	'upload-installer': uploadInstaller,
	'upload-site':      uploadSite,
	'icons-collect':    iconsCollect,
}

function main() {//one gate in, one gate out
	let command = commands[process.argv[2]]
	if (!command) throw new Error('say which: ' + Object.keys(commands).join(', '))
	command()
}
try { main() } catch (e) { console.error('🚧 Error:', e); process.exitCode = 1 }
