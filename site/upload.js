//./site/upload.js

import {execFileSync} from 'node:child_process'
import {createHash} from 'node:crypto'
import {existsSync} from 'node:fs'
import {readFile} from 'node:fs/promises'

/*
The publishing half of the two-part arrangement release.js describes. Everything this ships was made somewhere else: the site by vitepress, the installers and their sidecars by the desktop workspace on the machine that could build them. This side copies finished files and does no thinking about them.

Four modes in one file — site, dmg, exe, deb — because a release goes out from several computers. Only the machine that can build a platform's installer can publish it, so the same script runs on macOS, Windows and Linux, doing one platform's work each time.

Two destinations, and the difference between them is the point. The site goes as an account that administers the server, over rsync, which mirrors with --delete so a file dropped from the build leaves the server too. The installers go as a second account that has no shell at all: chrooted to the hostname's directory and able to speak only the SFTP file protocol. That way a compromise of the machine that builds an installer could overwrite the installers and their sidecars and could do nothing else on the server. rsync is unavailable to such an account by construction, since rsync works by running a program on the far side — scp is what remains, and since OpenSSH 9.0 scp transfers over SFTP anyway, so the two agree on the wire.

The server serves two directories at one hostname: a request that misses the site's directory falls through to the downloads directory. That is why an installer answers at the apex, like fujidesktop.app/fuji.exe, rather than under a downloads path — and why a site deploy has no way to reach an installer.

## Running this against your own server

The destination lives in .env at the monorepo root, which is gitignored, so a fresh clone will not have one and this script will say which values are missing and stop. Write it yourself — six values and no logic:

	DEPLOY_HOST=files.example.com
	DEPLOY_PORT=22
	DEPLOY_SITE_USER=deploy
	DEPLOY_SITE_PATH=/var/www/example.com/site/
	DEPLOY_FILES_USER=deploy
	DEPLOY_FILES_PATH=/var/www/example.com/downloads/
	DEPLOY_FILES_KEY=/home/you/.ssh/deploy_ed25519

On an EC2 instance or a DigitalOcean droplet the ordinary account is something like ubuntu or deploy, the host can be an IP like 203.0.113.10, the port is 22, and both paths are wherever your web server has its roots. Keep the trailing slashes: rsync ignores one on a destination, but scp given a path that does not exist yet will write a file by that name, and the slash turns that into an error instead. DEPLOY_FILES_KEY names the private key for the installer account specifically, rather than letting ssh offer whatever it finds — with two accounts that matters, since the default identity is the administrative one. Using one account for both user fields works and is the simplest thing that runs; splitting them, so the machine that builds an installer holds credentials that cannot administer anything, is the arrangement described above and is worth the extra setup once a release is public.

It sits at the monorepo root rather than in this workspace on purpose. Vite reads a .env inside the workspace it builds and copies any VITE_-prefixed value into the client bundle, which would publish these values on the website itself. At the root the file is outside what Vite looks at, so that hazard stops existing rather than becoming something to remember — which is also the reason not to move it back down here.

Node does not read .env on its own — the package.json scripts pass --env-file-if-exists, which is why these run as pnpm upload-exe rather than node upload.js exe. Run it the second way, or leave the file unwritten, and the values arrive empty, which readServer catches and explains. The tolerant spelling of the flag is deliberate: plain --env-file makes node refuse to start when the file is absent, printing one line and never reaching that message.
*/

//what each installer mode ships, by the stable published names release.js writes into desktop/release/
const installers = {
	dmg: 'fuji.dmg',
	exe: 'fuji.exe',
	deb: 'fuji.deb',
}

const staging = '../desktop/release'//where the desktop workspace leaves a finished release

let mode = process.argv[2]//site, dmg, exe, or deb
let server//filled by readServer below, before any mode runs

function readServer() {//gather the destination from the environment, naming whatever the scripts did not fill
	let required = [
		'DEPLOY_HOST',       'DEPLOY_PORT',
		'DEPLOY_SITE_USER',  'DEPLOY_SITE_PATH',
		'DEPLOY_FILES_USER', 'DEPLOY_FILES_PATH', 'DEPLOY_FILES_KEY',
	]
	let missing = required.filter(name => !process.env[name])
	if (missing.length > 0) throw new Error(`.env at the monorepo root is missing ${missing.join(', ')} — see the comment at the top of upload.js, and run this as pnpm rather than node, so the env file is passed`)

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
	let source = 'docs/.vitepress/dist/'//trailing slash: copy the contents, not the directory itself
	if (!existsSync(source)) throw new Error('no build to upload, run vitepress build first: ' + source)
	if (process.platform == 'win32') throw new Error('the site ships from the Mac — windows has no rsync, and this would fail with a spawn error rather than a sentence')

	//--delete is load-bearing: without it every previous deploy's hashed assets accumulate on the
	//server forever. it is also the reason the installers live in a directory this cannot reach
	execFileSync('rsync', [
		'-avz', '--delete',
		source,
		`${server.siteUser}@${server.host}:${server.sitePath}`,
		'-e', `ssh -p ${server.port}`,//rsync spells the port inside the ssh command, lowercase -p
	], {stdio: 'inherit'})
}

async function uploadInstaller(platform) {//ship one platform's installer and its sidecar
	let name = installers[platform]
	let sidecarName = name + '.json'
	if (!existsSync(`${staging}/${name}`))        throw new Error('no installer staged, run pnpm release on this machine first: ' + name)
	if (!existsSync(`${staging}/${sidecarName}`)) throw new Error('no sidecar staged beside the installer: ' + sidecarName)

	await checkSidecar(name, sidecarName)
	reportTransport()

	//installer first, sidecar second, so the page never fetches a hash for a file still arriving
	send(name)
	send(sidecarName)
}

function send(name) {//copy one file into the downloads directory as the restricted account
	//run from the staging directory and name the file bare. a windows absolute path contains the
	//colon scp uses to split host from path, and a bare filename makes that question stop existing
	execFileSync('scp', [
		'-P', server.port,//scp spells the port capital -P, unlike ssh and rsync
		'-i', server.fujiKey,//this account's own key; an explicit -i is offered before any default, so the admin key is never tried
		name,
		`${server.fujiUser}@${server.host}:${server.fujiPath}`,
	], {stdio: 'inherit', cwd: staging})
}

//catch a sidecar left from the previous release, which would otherwise publish a hash describing a
//file nobody can download. it cannot see a stale build — a sidecar that truthfully describes an
//installer compiled before the last source change passes — and it defends against nobody who can write here
async function checkSidecar(name, sidecarName) {
	let sidecar = JSON.parse(await readFile(`${staging}/${sidecarName}`, 'utf8'))
	let bytes = await readFile(`${staging}/${name}`)//whole file into memory; an installer is a few megabytes
	let sha256 = createHash('sha256').update(bytes).digest('hex')

	if (sidecar.file != name)          throw new Error(`sidecar names ${sidecar.file} rather than ${name}`)
	if (sidecar.bytes != bytes.length) throw new Error(`sidecar says ${sidecar.bytes} bytes, the file is ${bytes.length}; re-run pnpm release`)
	if (sidecar.sha256 != sha256)      throw new Error('sidecar hash does not describe this file; re-run pnpm release')

	console.log(`checked  ${name}  ${sidecar.version}  ${sidecar.arch}  ${bytes.length} bytes`)
}

//say which scp is about to run. windows carries two openssh installs and which one a script gets
//depends on the shell it was launched from; they differ in how they judge a private key's
//permissions, so this is the first thing anyone will want to know when authentication fails.
//diagnostic only, so it never throws — a missing scp should be reported by the transfer, not by this
function reportTransport() {
	let locate = process.platform == 'win32' ? 'where.exe' : 'which'
	try {
		let found = execFileSync(locate, ['scp'], {encoding: 'utf8'}).trim().split('\n')[0]
		console.log(`using    ${found}`)
	} catch (e) {
		console.log('using    could not locate scp')//say so and carry on; the transfer below reports the real failure
	}
}

async function main() {//ship the site, or one platform's installer and sidecar, to the server
	if (mode != 'site' && !installers[mode]) throw new Error('say which: site, dmg, exe, deb')
	server = readServer()

	if (mode == 'site') uploadSite()
	else await uploadInstaller(mode)
}
main().catch(e => { console.error('🚧 Error:', e); process.exitCode = 1 })
