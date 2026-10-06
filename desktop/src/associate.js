import {ref} from 'vue'
import parse from 'path-browserify'
import {openUrl} from '@tauri-apps/plugin-opener'//granted for ms-settings:defaultapps and nothing else, in capabilities/default.json
import {diskStat} from './disk.js'
import {registryGet, registrySet, registryDelete, registryNotify, registryOpens} from './registry.js'
import {launchOpens, launchSet} from './launch.js'
import {pathsExecutable} from './paths.js'
import {brandName, brandDescription} from './brand.js'
import {settings, settingsChanged} from './settings.js'
import {log} from './log.js'
import {backize, forwardize, platform} from './components/library.js'
import {fileTypesEnabled} from './fileTypes.js'

/*
Fuji offers itself to macOS and Windows as a program that can open pictures, and sets out to be simple and modern in how it does that, polite and assertive in how it behaves, and to leave the user in control. That differs from how most programs handle file types, image viewers above all. Before Windows 8 the default for a type was a registry value any program could write, so a program took its types at install and again at every launch; Windows 8 sealed the default with a hash only the system's own screens write, and much association code is still that old habit.

Simple means one plain pass, the same writes in the same order at every launch, over general commands in registry.rs, with the whole policy here. It never checks the Windows version, never falls back from one method to another, and never computes the hash, and it reads each value before writing it, so a pass that changed nothing tells the shell nothing. Modern means registering per user, in the shape Microsoft's ActivationRegistrationManager writes for an unpackaged app, so the installer does nothing about file types. On the Mac the offer needs no code: CFBundleDocumentTypes in Info.plist declares it, ranking every type Alternate, which offers without claiming, and Launch Services reads it the first time it sees the bundle, so dragging Fuji.app into Applications is the whole of the registration.

Assertive means an offer that is complete and keeps itself whole, every type fuji opens, in Open with on both systems and in Windows' Default apps by name, rewritten at every launch; and after a yes, the fallback below, the one claim a program may still make. Polite and the user in control are the limits on that. Installing or running fuji is never permission: the user answers yes, no or ask in fuji's settings, Windows keeps the last word, and where the two disagree the settings show it, with the way to Windows' own screen. Fuji follows Windows in one direction only. An ask that Windows opens with this copy becomes a yes, whether the user chose fuji there or fuji is the only program offering the type, which Windows then opens with fuji on the offer alone; fuji never follows a no, which the user said on purpose. And fuji asks Windows what opens a type only while the settings are showing, never at launch, since a check at launch is what grows into the banner a browser shows.

Everything is per extension, since a person may want fuji for .webp and the editor they already use for .jpg. Each extension has its own ProgID and name, which Explorer prints in its Type column, so a folder sorted by type keeps its .jpe files apart from its .jpg; a different icon per format would cost nothing, and fuji never strands a choice saved against a ProgID by dropping it. Each has its own answer too, kept in fuji.toml as three lists, and a type a later fuji adds starts at ask.

The types are the enabled entries of fileTypes.js, and Info.plist and the Windows uninstaller's list in win-setup/registry.js name them again by hand, since neither can read that table; Linux declares none yet. On Windows three layers decide what opens a type, and everything fuji writes is under HKEY_CURRENT_USER. The offer, written whatever the answer: a ProgID per extension, with its name, icon and command; that ProgID in the extension's OpenWithProgids; the executable's key under Applications, with the types it supports; and a Capabilities block named in RegisteredApplications, which lists fuji in Settings. The fallback, the extension's own default value, which decides where no choice is saved: the line an installer from 1999 writes at install and ActivationRegistrationManager leaves alone. Fuji writes it for a yes and takes it back for any other answer, but only while it still names fuji. Above both, the user's saved choice, UserChoice, which fuji only reads, by asking the shell what it would open, the lookup Explorer makes. The Mac has no fallback, since the only default a program can set there is the saved choice itself.

The Mac is simpler, because there any application can set the user's saved choice itself, through Launch Services and with no dialog, which launch.rs has. Fuji can, so a choice made in fuji's settings is carried out at the click that makes it, once, and never again. Every other application can too, so what the Mac opens a type with is the whole answer, and fuji keeps none: the three lists in fuji.toml go unused there, the settings sort each type under whatever the Mac opens it with now, and Choose Fuji takes it. Taking a type away again is Get Info's, since fuji never chooses another application for the user. With no answer kept, none can disagree with the system, so nothing in the settings that helps the user finish a choice in the system's own screens is ever reached on the Mac. The code is the same on both; the table named system below is the one place that asks which platform it is on. And the Mac files its record by kind of file rather than by extension, so .jpg, .jpeg and .jpe, all public.jpeg, move together.

Only the installed copy registers or acts on an answer, since everything registered names the running executable, and a build in target/ or a copy on the Desktop may move or vanish. On the Mac the installed copy is one in an Applications folder; on Windows it is the copy the installer recorded. The test is that the executable sits in the folder InstallLocation names under Software\Microsoft\Windows\CurrentVersion\Uninstall\Fuji, which every install writes and the uninstaller removes, under HKEY_CURRENT_USER because the installer installs for the user alone. Every copy shares fuji.toml, so a copy that fails the test cannot change an answer either.

What the platforms do, seen rather than read. Windows shows a one-time chooser on the first double-click of a type after a new program registers for it, whatever the fallback says, and Set defaults by app can label a type Choose a default while it holds a saved choice; neither is fuji's doing. Explorer keeps drawing thumbnails for a type fuji owns, since fuji writes no ShellEx key and the thumbnail lookup never consults UserChoice. On macOS bare CFBundleTypeExtensions are enough, since Launch Services makes the system's own types from them, public.jpeg and public.svg-image, except for .jfif, which gets a dynamic type of its own, so Change All on a .jpg in Get Info misses it. And the Mac needs no document icon, since Quick Look's preview wins over a handler's icon, checked on the Mac mini 2026-09-14.

Two things elsewhere stay in step with this. bundle.fileAssociations stays out of tauri.conf.json, since Info.plist declares the Mac's types by hand, this file registers them with Windows at every launch, and Linux declares none yet. And the uninstaller takes back everything this writes, for the extensions win-setup/registry.js names and under the names this file gives them, Fuji.webp for a ProgID, Software\Fuji for the Capabilities block and fuji.exe under Applications, so a change to how this file names a key is a change to the rules there too. The installer never uninstalls first, so an install over an existing copy leaves every registration, and every choice the user saved in Windows' own screens, as it was.
*/

const documentIcon = 'document-image.ico'//what a picture of every type fuji opens wears in Explorer, beside the executable; fuji's own icon would make a folder of pictures a folder of cyan discs
const answerNames = ['yes', 'no', 'ask']//the three answers, which are also the three lists in fuji.toml, in the order the file shows them

export const associateAnswers = ref({})//extension to yes, no, or ask; changed only by associateChoose and by following windows
export const associateOpens   = ref({})//extension to {name, executable}, what the system would open it with; empty until the settings look
export const associateActive  = ref(false)//this copy acts on the answers: the installed copy, on windows and the mac

let executable = ''//this copy's program file, forwardized, found at startup

const system = {//the one place this asks which platform it is on: how each looks, offers, takes and tells the installed copy, and whether fuji keeps answers there at all. Everything below runs the same on both, and linux, with none of these yet, gets undefined
	windows: {keeps: true,  look: registryOpens, offer: offerWindows, take: () => register(), installed: installedWindows},//keeps, because windows may be waiting on a step only the user can finish; take writes the fallback for every yes, so it needs no list
	mac:     {keeps: false, look: launchOpens,   offer: async () => '', take: takeMac,        installed: installedMac},//keeps nothing, because what the mac opens a type with is the whole answer; Info.plist makes the offer before any of fuji's code runs
}[platform()]

export async function associateStart() {//at startup, in every copy: read and repair the three lists, then on the installed copy offer every type and claim each yes; answers a line for the log, or blank
	let {answers, problems} = answersRead()
	for (let problem of problems) log(`settings: associations, ${problem}`)
	associateAnswers.value = answers
	answersWrite()//every extension in exactly one list, which repairs a hand edit and adds a type this fuji has and the last did not
	if (!system) return ''//linux declares no types yet
	executable = await pathsExecutable()
	associateActive.value = await system.installed()
	if (!associateActive.value) return ''//not the installed copy, which is nothing worth a log line
	return queue(system.offer)//the offer, made again at every launch where fuji has to make it
}

export function associateChoose(extensions, answer) {//the user's answer for one extension or several: recorded where fuji keeps answers, carried out in one pass, and looked at again, so the settings show where the system lands
	if (!associateActive.value || !answerNames.includes(answer) || !extensions.every(extension => fileTypesEnabled[extension])) throw new Error(`cannot answer ${answer} for ${extensions.join(' ')} here`)//the settings offer no such choice, so reaching this is a mistake in the code asking
	if (system.keeps) {
		associateAnswers.value = {...associateAnswers.value, ...Object.fromEntries(extensions.map(extension => [extension, answer]))}
		answersWrite()
	}
	return queue(async () => { await system.take(extensions, answer); await look() })//try to take, then see whether it took: always on the mac, and on windows only where the user has saved no choice
}

export function associateLook() { return queue(look) }//see what the system opens every extension with, and follow it as look does; the settings call this on appearing and at every return of focus

export function associateProgram(extension) {//the program the system would open this extension with, by the name a person knows: Fuji for any copy of fuji, a nameless one by its file's, blank for nothing or not yet looked
	let opener = associateOpens.value[extension]
	if (!opener) return ''
	if (ours(extension)) return brandName
	return opener.name || parse.basename(forwardize(opener.executable))
}

export function associateDiffers(extension) {//a yes windows opens with another program, or a no it opens with fuji; ask never differs, being no answer at all
	if (!associateActive.value || !associateOpens.value[extension]) return false//nothing to compare until the settings have looked
	let answer = associateAnswers.value[extension]
	if (answer == 'yes') return !ours(extension)
	if (answer == 'no')  return ours(extension)
	return false
}

export function associateSiblings(extension) {//the extensions fuji opens that the system files under the same kind as this one, this one included and alphabetical, which therefore open with the same program; empty where it files none together, which is every extension on windows, since the registry answers no kind
	let kind = associateOpens.value[extension]?.kind
	if (!kind) return []//windows, or nothing looked at yet
	let siblings = Object.keys(associateOpens.value).filter(other => associateOpens.value[other].kind == kind).sort()
	if (siblings.length < 2) return []//a kind of its own, like .png's
	return siblings
}

function ours(extension) {//the system would open this extension with this very copy of fuji, as last looked
	let opener = associateOpens.value[extension]
	return !!opener?.executable && forwardize(opener.executable).toLowerCase() == executable.toLowerCase()
}

export async function associateFinish() {//open windows' Default apps, the one place a saved choice can change: at fuji's own page on windows 11, and at its first page on windows 10, which ignores the name
	await openUrl(`ms-settings:defaultapps?registeredAppUser=${encodeURIComponent(brandName)}`)//the name fuji registered under, escaped as Microsoft asks
}

let running = Promise.resolve()//the last pass queued, so the next waits for it: a focus can arrive while a choice is still writing
function queue(work) { running = running.then(work, work); return running }//after the pass before, whether it worked or not; the caller of that one has already heard how it went

function answersRead() {//each extension's answer from the three lists, and a line for each thing wrong with them; read as a person writes by hand, so JPG means .jpg
	let lists = {}//extension to the answers that list it
	let problems = []
	for (let answer of answerNames) {
		for (let item of settings.associations[answer]) {
			let extension = item.trim().toLowerCase()
			if (!extension.startsWith('.')) extension = `.${extension}`
			if (!fileTypesEnabled[extension]) { problems.push(`${item} is not a kind of file ${brandName} opens, so it was dropped from ${answer}`); continue }
			(lists[extension] ??= new Set()).add(answer)//a set, so an extension listed twice in the same list is only there once
		}
	}
	let answers = {}
	for (let extension of Object.keys(fileTypesEnabled)) {
		let found = [...(lists[extension] ?? [])]
		answers[extension] = found.length == 1 ? found[0] : 'ask'//in none of the lists is ask, which is how a new extension arrives
		if (found.length > 1) problems.push(`${extension} is in ${found.join(' and ')}, so it counts as ask`)
	}
	return {answers, problems}
}

function answersWrite() {//the three lists from the answers, in the table's order so the file reads the same every time; fuji.toml gets them at close, like every setting
	for (let answer of answerNames) settings.associations[answer] = Object.keys(associateAnswers.value).filter(extension => associateAnswers.value[extension] == answer)
	settingsChanged()
}

async function installedWindows() {//whether this is the copy the installer put there, the one gate on everything this writes
	let folder = parse.dirname(executable)//the folder the program sits in, forwardized like everything the page holds
	let recorded = forwardize(await registryGet(`Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${brandName}`, 'InstallLocation'))//where the installer put fuji, spelled the way the page spells every path, or blank when fuji is not installed
	return !!recorded && folder.toLowerCase() == recorded.toLowerCase()
}

async function installedMac() {//whether this is a copy in an Applications folder, the system's or the user's, which is where a mac user puts an application to keep it; a build in target/ or a copy on the Desktop may move or vanish, and the system would go on opening pictures with whatever took its place
	let bundle = parse.dirname(parse.dirname(parse.dirname(executable)))//Fuji.app, from Fuji.app/Contents/MacOS/fuji
	return bundle.endsWith('.app') && parse.basename(parse.dirname(bundle)) == 'Applications'
}

async function offerWindows() {//tell windows what this copy can open, at every launch, and answer a line for the log
	let changed = await register()
	return `associate: ${Object.keys(associateAnswers.value).length} types registered, ${changed} values written`
}

async function takeMac(extensions, answer) {//make this copy the application for each extension answered yes, which the mac carries out at once; a no takes nothing, since fuji never chooses another application for the user
	if (answer != 'yes') return
	for (let extension of extensions) await launchSet(extension)//one call each, since a command takes one thing
}

async function register() {//tell windows what this copy can open, claim the fallback for each yes, and take back each other fallback while it still names fuji; answers how many values changed
	let windowsExecutable = backize(executable)//windows' own spelling of the path, which the registry wants
	let file = parse.basename(executable)//fuji.exe, which is the key windows expects under Applications
	let command = `"${windowsExecutable}" "%1"`//quoted, because a picture's path will contain spaces; %1 is where windows puts the file
	let icon = `${windowsExecutable},0`//the executable's own icon, its first, which is still better than none
	let beside = `${parse.dirname(executable)}/${documentIcon}`//joined by hand, since parse.join would fold a network path's leading // into one
	try { await diskStat(beside); icon = `${backize(beside)},0` } catch {}//the document icon where it exists; unquoted is safe, since windows splits an icon location at its last comma

	let application = `Software\\Classes\\Applications\\${file}`
	let capabilities = `Software\\${brandName}\\Capabilities`
	let changed = 0
	let set   = async (key, name, value) => { if (await registrySet(key, name, value)) changed++ }//each value read first and written only if it would change, so a pass that changed nothing knows it
	let unset = async (key, name)        => { if (await registryDelete(key, name))    changed++ }//and taking back something already gone is no change either

	for (let [extension, entry] of Object.entries(fileTypesEnabled)) {
		let program = `${brandName}${extension}`//the progid, so .webp becomes Fuji.webp; the uninstaller's rules in win-setup/registry.js take back only this shape, so change both together
		await set(`Software\\Classes\\${program}`, '', entry.type)//the progid: what this kind of file is called, which explorer prints in its type column
		await set(`Software\\Classes\\${program}\\DefaultIcon`, '', icon)//what explorer draws on one
		await set(`Software\\Classes\\${program}\\shell\\open\\command`, '', command)//and what opens it
		await set(`Software\\Classes\\${extension}\\OpenWithProgids`, program, '')//the offer: fuji joins the list of what could open this extension, by name alone
		await set(`${application}\\SupportedTypes`, extension, '')//so fuji is offered for these and not for everything else
		await set(`${capabilities}\\FileAssociations`, extension, program)//so Settings can list fuji's types

		let fallback = `Software\\Classes\\${extension}`//the extension's own key, whose default value names the progid windows uses where the user has saved no choice
		if (associateAnswers.value[extension] == 'yes') await set(fallback, '', program)//claimed for a yes, whoever wrote it last; it never outranks a choice the user saved
		else if (await registryGet(fallback, '') == program) await unset(fallback, '')//given back, but only while it still names fuji; whatever another program had there before stays gone
	}
	await set(application, 'FriendlyAppName', brandName)
	await set(`${application}\\shell\\open\\command`, '', command)
	await set(capabilities, 'ApplicationName', brandName)
	await set(capabilities, 'ApplicationDescription', brandDescription)
	await set('Software\\RegisteredApplications', brandName, capabilities)//what lists fuji in Settings, last so it appears only once everything it points at is there; a pass that stopped early finishes on the next

	if (changed > 0) await registryNotify()//only when something moved, because this runs on every launch and almost always writes nothing
	return changed
}

async function look() {//what the system would open every extension with, and, where fuji keeps answers, every ask it opens with this copy turned to yes, whether the user chose fuji there or fuji is the only program offered
	if (!system) return//linux has nothing to ask yet
	let opens = {}
	for (let extension of Object.keys(fileTypesEnabled)) opens[extension] = await system.look(extension)//one call each, since a command takes one thing
	associateOpens.value = opens
	if (!associateActive.value || !system.keeps) return//a copy that cannot act on an answer shows what the system says and follows nothing, and nor does a system whose own record is the whole answer
	let followed = Object.keys(opens).filter(extension => associateAnswers.value[extension] == 'ask' && ours(extension))//never a no, which the user said on purpose, and never a yes, which already agrees
	if (followed.length == 0) return
	associateAnswers.value = {...associateAnswers.value, ...Object.fromEntries(followed.map(extension => [extension, 'yes']))}
	answersWrite()
	log(`associate: windows opens ${followed.join(' ')} with ${brandName}, so each is now yes`)
	await register()//and the fallbacks that go with a yes, which only windows has, being the only system where fuji keeps answers
}
