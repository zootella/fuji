//./scripts.js

import {execFile, execFileSync} from 'node:child_process'
import {createHash} from 'node:crypto'
import {copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync} from 'node:fs'
import {dirname, join} from 'node:path'
import {fileURLToPath} from 'node:url'

/*
The build pipeline for both workspaces, in one file at the monorepo root. Everything the package.json scripts do beyond calling tauri or vitepress is here, reached by a verb: reveal, hash, upload-installer, upload-site, icons-collect.

Fuji is developed on several computers, and each one builds, stages and publishes only the installer it can make. That is why a command means the same thing everywhere while doing different work underneath — `installer` produces a .dmg, a .exe or a .deb, and `upload` sends whichever of those this machine is the one that can build. Nobody has to remember which computer they are sitting at.

One file rather than one per workspace, because the facts worth keeping in one place cross the workspace boundary. The stable publishing name is the clearest case: tauri writes Fuji_0.1.0_aarch64.dmg, which is right for a build directory and wrong for a download link, so hash copies it to fuji.dmg and a link in a page survives the next version. That name is written by the desktop side and shipped by the site side, and when these were separate scripts each spelled it out for itself. The platforms table below is now the only place any of it is said.

Living at the root also settles the working-directory question by force rather than by discipline. Both workspaces call this file, each from its own folder, so nothing here can be relative to wherever node started; every path is built from this file's own location. The scripts this replaced each had their own answer to that, every one of them correct only because pnpm happened to run it from the right place.

It imports node builtins and nothing else, and has to: the root package.json has no dependencies, and node_modules belongs to the workspaces below it.
*/

//everything that differs between the three computers, in one table. folder and suffix are how tauri
//names what it built, published is the stable name a download link can keep, and opener is the
//command that shows a person a folder
const platforms = {
	darwin: {folder: 'dmg',  suffix: '.dmg',       published: 'fuji.dmg', opener: 'open'},
	win32:  {folder: 'nsis', suffix: '-setup.exe', published: 'fuji.exe', opener: 'explorer'},
	linux:  {folder: 'deb',  suffix: '.deb',       published: 'fuji.deb', opener: 'xdg-open'},
}

function thisMachine() {//the row for the computer we are running on, and the one place anything below asks
	let found = platforms[process.platform]
	if (!found) throw new Error('fuji has no build for this platform: ' + process.platform)
	return found
}

//every path is built from this file's own location, never from the working directory, because both
//workspaces call this file and each calls it from its own folder
const root = fileURLToPath(new URL('.', import.meta.url))
const configurationFile = join(root, 'desktop/src-tauri/tauri.conf.json')//the file that named the bundle
const bundled = join(root, 'desktop/src-tauri/target/release/bundle')    //where tauri leaves what it built
const staging = join(root, 'desktop/release')                            //where hash puts the installer and its sidecar, and where upload looks for them
const icons   = join(root, 'desktop/src-tauri/icons')                    //committed artwork, generated rather than drawn
const built   = join(root, 'site/docs/.vitepress/dist')                  //what vitepress builds

let server//the destination, filled by readServer before either upload runs

//open the graphical file manager on this platform's finished installer, so it can be double-clicked
//the way a person who downloaded it would. that is a different and stronger test than starting a
//built binary in place: an installer has a first-run experience — the publisher warning, the wizard,
//where the application ends up — and none of that happens otherwise
function reveal() {
	let machine = thisMachine()
	let folder = join(bundled, machine.folder)//the folder, not the file: its name carries the version and would need editing every release, and the folder holds one file anyway
	if (!existsSync(folder)) throw new Error('nothing built yet at ' + folder + ', run pnpm installer first')

	console.log('opening  ' + folder)
	//an absolute path, because explorer resolves a relative one against its own working directory
	//rather than ours. and explorer answers 1 even when it did open the window, so its exit means nothing
	execFile(machine.opener, [folder], error => {
		if (error && process.platform != 'win32') console.error('could not open the file manager: ' + error.message)
	})
}

//stage this platform's installer under its publishing name and write the sidecar beside it, building
//nothing. two things the sidecar must not lie about, and each is read rather than assumed: the version
//comes from tauri.conf.json, the same file that named the bundle, so the two cannot disagree; the
//architecture comes out of the bundle's own filename, so it describes the file that exists rather than
//the machine that ran this
function hash() {
	let machine = thisMachine()
	let version = JSON.parse(readFileSync(configurationFile, 'utf8')).version
	if (!version) throw new Error('tauri.conf.json has no version')

	//find this version's bundle and only this version's; an old one left beside it would be a coin flip
	let folder = join(bundled, machine.folder)
	let prefix = `Fuji_${version}_`
	let names = readdirSync(folder).filter(name => name.startsWith(prefix) && name.endsWith(machine.suffix))
	if (names.length != 1) throw new Error(`expected one ${prefix}*${machine.suffix} in ${folder}, found ${names.length} of them: ${readdirSync(folder).join(', ')}`)

	let name = names[0]
	let architecture = name.slice(prefix.length, name.length - machine.suffix.length)//what tauri called it, between the version and the extension

	//copy first, then measure what landed, so every number describes the file the site will actually ship
	mkdirSync(staging, {recursive: true})
	let destination = join(staging, machine.published)
	copyFileSync(join(folder, name), destination)
	let bytes = readFileSync(destination)//whole file into memory; an installer is a few megabytes, and streaming would buy nothing

	let sidecar = {
		file: machine.published,
		version,
		arch: architecture,
		bytes: bytes.length,
		sha256: createHash('sha256').update(bytes).digest('hex'),
		date: new Date().toISOString().slice(0, 10),//iso, because the home page sorts three of these as text to find the earliest build
	}
	writeFileSync(destination + '.json', JSON.stringify(sidecar, null, '\t') + '\n')

	console.log('staged   ' + name)
	console.log('      -> ' + destination + '  ' + sidecar.bytes + ' bytes')
	console.log('         ' + destination + '.json  ' + sidecar.sha256)
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

	//refuse to ship a sidecar from inside the site. the server answers one hostname from two directories
	//and checks the site's first, so a fuji.*.json in this build would shadow the real one in downloads
	//and pin the download page to whatever hash it holds until the next deploy. one gets here by sitting
	//in docs/public, which a retired fixtures script used to copy them into for local development — the
	//files are gitignored, so a machine that ran it still has them and no other machine can tell
	let shadowing = readdirSync(built).filter(name => name.startsWith('fuji.') && name.endsWith('.json'))
	if (shadowing.length > 0) throw new Error(`${shadowing.join(', ')} would ship inside the site and shadow the real sidecar on the server — delete site/docs/public/fuji.*.json on this machine, then build again`)

	server = readServer()

	//--delete is load-bearing: without it every previous deploy's hashed assets accumulate on the
	//server forever. it is also the reason the installers live in a directory this cannot reach
	execFileSync('rsync', [
		'-avz', '--delete',
		built + '/',//trailing slash: copy the contents, not the directory itself
		`${server.siteUser}@${server.host}:${server.sitePath}`,
		'-e', `ssh -p ${server.port}`,//rsync spells the port inside the ssh command, lowercase -p
	], {stdio: 'inherit'})
}

function uploadInstaller() {//ship this machine's installer and its sidecar
	let name = thisMachine().published
	let sidecarName = name + '.json'
	if (!existsSync(join(staging, name)))        throw new Error('no installer staged, run pnpm installer and then pnpm hash on this machine first: ' + name)
	if (!existsSync(join(staging, sidecarName))) throw new Error('no sidecar staged beside the installer: ' + sidecarName)
	checkSidecar(name, sidecarName)//check what is here before asking for credentials, so a missing .env is never what hides a stale sidecar
	server = readServer()
	reportTransport()

	send(name); send(sidecarName)//installer first, so the page never fetches a hash for a file still arriving
}

function send(name) {//copy one file into the downloads directory as the restricted account
	//run from the staging directory and name the file bare. a windows absolute path contains the colon
	//scp uses to split host from path, and a bare filename makes that question stop existing
	execFileSync('scp', [
		'-P', server.port,//scp spells the port capital -P, unlike ssh and rsync
		'-i', server.fujiKey,//an explicit -i is offered before any default, so the admin key is never tried
		name,
		`${server.fujiUser}@${server.host}:${server.fujiPath}`,
	], {stdio: 'inherit', cwd: staging})
}

//catch a sidecar left from the previous release, which would otherwise publish a hash describing a
//file nobody can download. it cannot see a stale build — a sidecar that truthfully describes an
//installer compiled before the last source change passes — and it defends against nobody who can write here
function checkSidecar(name, sidecarName) {
	let sidecar = JSON.parse(readFileSync(join(staging, sidecarName), 'utf8'))
	let bytes = readFileSync(join(staging, name))
	let sha256 = createHash('sha256').update(bytes).digest('hex')

	if (sidecar.file != name)          throw new Error(`sidecar names ${sidecar.file} rather than ${name}`)
	if (sidecar.bytes != bytes.length) throw new Error(`sidecar says ${sidecar.bytes} bytes, the file is ${bytes.length}; re-run pnpm hash`)
	if (sidecar.sha256 != sha256)      throw new Error('sidecar hash does not describe this file; re-run pnpm hash')

	console.log(`checked  ${name}  ${sidecar.version}  ${sidecar.arch}  ${bytes.length} bytes`)
}

//say which scp is about to run. windows carries two openssh installs, and which one a script gets
//depends on the shell it was launched from; they differ in how they judge a private key's permissions,
//so this is the first thing anyone will want to know when authentication fails. diagnostic only, so it
//never throws — a missing scp should be reported by the transfer, not by this
function reportTransport() {
	try {
		let found = execFileSync(process.platform == 'win32' ? 'where.exe' : 'which', ['scp'], {encoding: 'utf8'}).trim().split('\n')[0]
		console.log('using    ' + found)
	} catch (e) {
		console.log('using    could not locate scp')//say so and carry on; the transfer below reports the real failure
	}
}

//the three tauri icon runs stay in package.json, where pnpm is what puts the tauri cli on the path;
//this is the part after them. each run writes a folder of its own, and these are the files that have to
//come out from under a generated name and sit where tauri.conf.json and the windows manifest expect
//them. icon.md is the whole subject, including why these are committed artifacts
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

//the verb package.json passes. these are spelled out rather than shared, because the script names a
//person types differ between the two workspaces on purpose — pnpm upload means the installer in
//desktop and the site in site — and this file should never have to guess which one called it
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
