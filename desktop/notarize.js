import {execFileSync, spawnSync} from 'node:child_process'
import {readdirSync, readFileSync} from 'node:fs'
import {basename, join} from 'node:path'
import {fileURLToPath} from 'node:url'
import {buildStep, readSha256} from '../build-log.js'

/*
The Mac release, signed with a Developer ID and notarized by Apple, in one run: pnpm installer with Apple in it, leaving its dmg in the bundle folder where pnpm installer leaves its own and pnpm hash looks. pnpm installer stays the build that needs no Apple account.

It needs two things from Apple's developer program, both kept outside this repository. A Developer ID Application certificate in the login keychain, which codesign finds by that kind rather than by its holder's name, so no tracked file carries a legal name. And an App Store Connect API key, its three values in .env at the repository root under the names tauri reads. Without them this stops before building and says which is missing.

Tauri does the app: given the identity and the key, its build signs the app with the hardened runtime on, which notarization requires, sends it to Apple, waits, and staples the ticket to it. This does the same for the dmg around it, then asks Gatekeeper what a downloaded copy of each will meet, and any step that fails stops the run. Apple answered each part in about half a minute through October 2026, so nothing is kept between runs, and one cut short is run again.
*/

const desktop     = fileURLToPath(new URL('.', import.meta.url))//every path from this file's own location, for the reason scripts.js gives at the root
const bundled     = join(desktop, 'src-tauri/target/release/bundle')
const brandName   = JSON.parse(readFileSync(join(desktop, 'src-tauri/tauri.conf.json'), 'utf8')).productName//the name tauri gives the .app
const identity    = 'Developer ID Application'//the kind of certificate rather than its holder's name
const credentials = ['APPLE_API_KEY', 'APPLE_API_ISSUER', 'APPLE_API_KEY_PATH']//the api key's three values, under the names tauri reads

async function main() {
	if (process.platform != 'darwin') throw new Error('notarizing is the mac\'s alone; pnpm installer builds the installer here')
	let missing = credentials.filter(name => !process.env[name])
	if (missing.length) throw new Error(`.env at the repository root has no ${missing.join(', ')}, the App Store Connect API key notarizing sends with; pnpm installer builds without one`)
	if (!execFileSync('security', ['find-identity', '-v', '-p', 'codesigning'], {encoding: 'utf8'}).includes(`"${identity}`)) throw new Error(`no ${identity} certificate in the login keychain to sign with; pnpm installer signs with the self-signed identity tauri.conf.json names instead`)

	run('node', ['build.js', '--config', JSON.stringify({bundle: {macOS: {hardenedRuntime: true}}})], {APPLE_SIGNING_IDENTITY: identity})//tauri prefers this identity to the one in tauri.conf.json, and with the key in the environment notarizes and staples the app
	let app = join(bundled, `macos/${brandName}.app`)
	run('xcrun', ['stapler', 'validate', app])//the app's ticket, there before the dmg wraps it
	run('node', ['dmg.js'])
	let dmg = join(bundled, 'dmg', readdirSync(join(bundled, 'dmg')).find(name => name.endsWith('.dmg')))//the one dmg.js just made, in the folder it empties first

	await buildStep('notarize', basename(dmg), `signed with the ${identity}`, async () => {
		run('codesign', ['--sign', identity, '--timestamp', dmg])
		let sent = spawnSync('xcrun', ['notarytool', 'submit', dmg, '--wait', '--output-format', 'json', '--key', process.env.APPLE_API_KEY_PATH, '--key-id', process.env.APPLE_API_KEY, '--issuer', process.env.APPLE_API_ISSUER], {encoding: 'utf8'})
		let answer = sent.status == 0 ? JSON.parse(sent.stdout) : {}
		if (answer.status != 'Accepted') throw new Error(`Apple did not accept the dmg: ${answer.status ? `${answer.status}, and xcrun notarytool log ${answer.id} says why` : (sent.stderr || sent.stdout).trim()}`)
		run('xcrun', ['stapler', 'staple', dmg])
		run('xcrun', ['stapler', 'validate', dmg])
		run('spctl', ['--assess', '-vv', '--type', 'execute', app])//accepted, with source=Notarized Developer ID
		run('spctl', ['--assess', '-vv', '--type', 'open', '--context', 'context:primary-signature', dmg])
		return {value: await readSha256(dmg), words: 'the app and the dmg each carry a ticket, and Gatekeeper accepts both'}
	})
}

function run(command, args, env = {}) {//one program to its end, its output in this terminal, ending the run if it fails
	let ran = spawnSync(command, args, {cwd: desktop, stdio: 'inherit', env: {...process.env, ...env}})
	if (ran.error) throw ran.error
	if (ran.status != 0) throw new Error(`${command} ${args.join(' ')} exited ${ran.status}`)
}

main().catch(e => { console.error('🚧 Error:', e); process.exitCode = 1 })
