import {spawn, spawnSync} from 'node:child_process'
import {existsSync, readdirSync, readFileSync} from 'node:fs'
import {join} from 'node:path'
import {fileURLToPath} from 'node:url'

/*
The mac installer, signed with an Apple Developer ID and notarized, so a copy someone downloads opens like any other app from the Internet.

A browser marks every file it downloads, and Gatekeeper checks a marked app the first time it runs. pnpm installer signs fuji with a certificate of fuji's own, which scripts.js explains, and Gatekeeper trusts only certificates Apple issued: the dialog says Apple could not verify "Fuji" is free of malware, offers Done and Move to Trash, and the way past it runs through Privacy & Security in System Settings. An app passes once it has two things. A Developer ID certificate, which Apple issues to a member of its developer program, says who signed the app, and notarization, Apple's service that scans a submitted build and returns a ticket, says Apple looked at it. An app with both meets the same question every app from the Internet meets, with an Open button.

This file holds everything about a release that depends on Apple, and the rest of the pipeline knows nothing about it. tauri.conf.json still names Fuji Desktop and pnpm installer still signs with it, so a release that depends on no one stays one command away. If Apple revoked the certificate, macs online would refuse every copy signed with it, installed ones included, and the next release would go out from pnpm installer, with the download page describing the way past the dialog again. A lapsed membership is milder: copies already notarized keep working, and only new ones cannot be notarized. Each signature carries a secure timestamp, so a copy signed before the certificate expires stays valid after it does.

## What a run does

It runs pnpm installer, unchanged, with the credentials and one more variable in its environment. Tauri prefers APPLE_SIGNING_IDENTITY to the signingIdentity in tauri.conf.json, so this run alone signs with the Developer ID, and when it finds the credentials, Tauri zips the signed app, submits it, waits, and staples the ticket to the app. dmg.js then wraps the stapled app as it always does. The rest is Apple's rule for a container: sign the outermost one, notarize it, and staple its ticket to it. The dmg and the app inside then each carry a ticket of their own, and neither needs a network at first launch. Developers report Gatekeeper refusing a dmg that is signed and never notarized, so pnpm installer leaves its dmg unsigned, and only a run of this file signs one.

The identity is named by its kind, Developer ID Application, and codesign signs with the identity in the keychain whose name contains those words. The certificate's own name is its holder's legal name and team, so naming the kind keeps both out of the repository, and a renewed certificate needs no edit here. Every signed copy still carries that name and team, where anyone can read them with codesign -dvvv. macOS keeps a user's permissions against a rule naming the team rather than one certificate, so a renewal costs no one a permission; the move from Fuji Desktop costs each user one, once. While an old and a renewed certificate are both valid, the words match two identities and codesign refuses to choose, so the check below stops before the build; removing the older one from the login keychain settles it.

## The .env

Three values in .env at the monorepo root, passed by the package script with --env-file-if-exists, the way pnpm upload gets the deploy values:

	APPLE_API_KEY=ABC123DEFG
	APPLE_API_ISSUER=00000000-0000-0000-0000-000000000000
	APPLE_API_KEY_PATH=/Users/you/.appstoreconnect/private_keys/AuthKey_ABC123DEFG.p8

They describe an App Store Connect API key, made under Users and Access, then Integrations, with Developer access. The key ID and the issuer are identifiers. The secret is the .p8 file, which Apple lets its owner download only once, and which lives outside the repository, readable only by its owner, in the folder Apple's tools look in. A key lost or exposed is revoked on the same page and replaced, and nothing already shipped notices. The values reach every program the build starts, Vite among them, which exposes to the page only names beginning VITE_.

## When a run stops partway

Each wait asks Apple for the answer until it comes, with no time limit, and caffeinate keeps the mac from idle sleep while it does. Apple's side does not depend on this mac: a closed terminal or a restart loses the wait, not the submission, and notarytool history lists every submission with its status. stapler finds a ticket by the signed file's code hash rather than by the submission, so an accepted file can be stapled at any later time, as long as nothing has rebuilt or re-signed it. The simple recovery is to run pnpm notarize again, which builds, signs and submits afresh, at the cost of another wait.
*/

//every path is built from this file's own location, for the reason scripts.js gives at the root
const here    = fileURLToPath(new URL('.', import.meta.url))
const bundled = join(here, 'src-tauri/target/release/bundle')//where tauri leaves the app in macos/ and dmg.js leaves the dmg in dmg/

const identity    = 'Developer ID Application'//the kind of certificate rather than its holder's name, which the essay explains
const credentials = ['APPLE_API_KEY', 'APPLE_API_ISSUER', 'APPLE_API_KEY_PATH']//the three .env values, under the names tauri reads

function main() {
	if (process.platform != 'darwin') throw new Error('notarization is apple\'s, and only a mac submits to it; on this platform pnpm installer is the whole build')

	//check everything a run needs before building anything, since tauri only warns when the credentials are missing, and goes on to build an installer nobody notarized
	let missing = credentials.filter(name => !process.env[name])
	if (missing.length > 0) throw new Error(`.env at the monorepo root is missing ${missing.join(', ')} — see the essay atop notarize.js, and run this as pnpm notarize, so the env file is passed`)
	if (!existsSync(process.env.APPLE_API_KEY_PATH)) throw new Error('no api key at ' + process.env.APPLE_API_KEY_PATH)
	let found = findIdentities()
	if (found.length != 1) throw new Error(`expected one valid "${identity}" identity in the keychain, found ${found.length}${found.length > 0 ? ': ' + found.join(', ') : ''} — the essay atop notarize.js says what to do with two`)

	spawn('caffeinate', ['-i', '-w', String(process.pid)], {stdio: 'ignore'}).unref()//no idle sleep until this process ends, so a long wait on apple is never cut off; caffeinate watches this process and exits with it

	//the ordinary installer, signing with the developer id instead of Fuji Desktop, and with tauri notarizing and stapling the app on the way
	run('pnpm', ['installer'], {APPLE_SIGNING_IDENTITY: identity})

	let brandName = JSON.parse(readFileSync(join(here, 'src-tauri/tauri.conf.json'), 'utf8')).productName//the name tauri gave the .app
	let app = join(bundled, `macos/${brandName}.app`)
	run('xcrun', ['stapler', 'validate', app])//tauri staples without checking the result, so check it before the dmg holding this app goes to apple

	let folder = join(bundled, 'dmg')
	let names = readdirSync(folder).filter(name => name.endsWith('.dmg'))//dmg.js empties this folder before it writes, so there is one
	if (names.length != 1) throw new Error(`expected one dmg in ${folder}, found ${names.length}`)
	let dmg = join(folder, names[0])

	//the outermost container: sign it, notarize it, staple its ticket to it
	run('codesign', ['--sign', identity, '--timestamp', dmg])
	notarize(dmg)
	run('xcrun', ['stapler', 'staple', dmg])

	//what gatekeeper will say to a downloaded copy: each should be accepted, with source=Notarized Developer ID
	run('spctl', ['--assess', '-vv', '--type', 'execute', app])
	run('spctl', ['--assess', '-vv', '--type', 'open', '--context', 'context:primary-signature', dmg])
	console.log(`notarized ${dmg}; pnpm hash and pnpm upload are next`)
}

function findIdentities() {//the names of the valid code signing identities containing the words in identity, which are the ones codesign would choose among
	let listed = spawnSync('security', ['find-identity', '-v', '-p', 'codesigning'], {encoding: 'utf8'}).stdout
	return [...listed.matchAll(/\d+\) [0-9A-F]{40} "(.*)"/g)].map(match => match[1]).filter(name => name.includes(identity))
}

//submit one file to apple's notary service and wait for the answer, printing apple's log when the answer is no. the json is what tauri reads for the app, so the two submissions are judged alike
function notarize(file) {
	let key = ['--key', process.env.APPLE_API_KEY_PATH, '--key-id', process.env.APPLE_API_KEY, '--issuer', process.env.APPLE_API_ISSUER]
	console.log(`submitting ${file} and waiting for apple; if this run is interrupted, notarytool history lists the submission`)
	let submitted = spawnSync('xcrun', ['notarytool', 'submit', file, ...key, '--wait', '--output-format', 'json'], {encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit']})
	if (submitted.error) throw submitted.error
	if (!submitted.stdout.trim()) throw new Error(`notarytool submit exited ${submitted.status} with no answer`)//notarytool has said why above
	let answer = JSON.parse(submitted.stdout)
	if (!answer.id) throw new Error('notarytool answered without a submission: ' + submitted.stdout)
	console.log(`${answer.status}  ${answer.id}`)
	if (answer.status == 'Accepted') return
	run('xcrun', ['notarytool', 'log', answer.id, ...key])
	throw new Error(`apple answered ${answer.status} for ${file}; its log is above`)
}

function run(command, args, environment) {//one step, with its output on this terminal, and the whole run stops if it fails
	let result = spawnSync(command, args, {stdio: 'inherit', cwd: here, env: {...process.env, ...environment}})
	if (result.error) throw result.error
	if (result.status != 0) throw new Error(`${[command, ...args].join(' ')} exited ${result.status}`)
}

try { main() } catch (e) { console.error('🚧 Error:', e); process.exitCode = 1 }
