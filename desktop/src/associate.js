import {ref} from 'vue'
import {openUrl} from '@tauri-apps/plugin-opener'//granted for ms-settings:defaultapps and nothing else, in capabilities/default.json
import {diskStat} from './disk.js'
import {registryGet, registrySet, registryDelete, registryNotify, registryOpens} from './registry.js'
import {pathsExecutable} from './paths.js'
import {brandName, brandDescription} from './brand.js'
import {settings, settingsChanged} from './settings.js'
import {log} from './log.js'
import {imageTypes, backize, forwardize, platform} from './components/library.js'

/*
Fuji offers itself to macOS and Windows as a program that can open pictures, and it sets out to be simple, modern, polite, and assertive about it, and to keep the user in control. Point for point, that differs from how most programs have handled file types for twenty-five years, and from image viewers most of all, which have more formats to claim than almost anything else on a machine. The usual way made sense when it began: before Windows 8 the default for a file type was a registry value any program could write, so a program that wanted its files took them at install and checked them again at every launch. Windows 8 sealed the default with a hash that only the system's own screens write, and much association code in the wild is that old habit, carried past the system it was built for.

Simple and modern are how it is built. Simple means one plain pass, the same writes in the same order every time, over general commands in registry.rs that know nothing about pictures, with the whole policy here in plain JavaScript. It never checks the Windows version, never tries one method with another behind it, and never computes the hash; and since each value is read before it is written, a pass that changed nothing knows it and leaves the shell alone. On the Mac simple goes all the way to no code at all for the offer: CFBundleDocumentTypes in Info.plist is the declaration, Launch Services reads it when it first sees the bundle, and dragging Fuji.app into Applications is the whole of the registration. Modern means the program registers itself, per user, in the shape Microsoft's own current API for unpackaged apps writes — ActivationRegistrationManager, whose implementation is open — rather than an installer script writing defaults, often for the whole machine. So the installer does nothing about file types, and the way to a saved default runs through the system's own screens, where Microsoft says it belongs.

Polite, assertive, and the user in control are how it behaves. Polite means fuji offers itself as one more program that can open each kind of picture and, until the user says yes to that kind, leaves whatever opens it now exactly as it is: every type in Info.plist is ranked Alternate, which says fuji can open it and is not claiming it, and on Windows fuji writes nothing that decides a type. Assertive means the offer is complete and keeps itself whole — every format fuji can show, everywhere the system looks for a program to open one, the Open With menu on both and Settings under Default apps by name on Windows, rewritten at every launch so anything missing comes back — and that after a yes, fuji claims the type at every launch in the one way that never overrides a choice the user saved. The user in control means installing or running fuji is never taken as permission. The user answers in fuji's settings, and the answers live in fuji.toml. The system keeps the last word, so fuji reads it and follows: a choice of fuji the user already made in the system's own screens counts as a yes, and where an answer and the system disagree, the settings show the difference and the way to the screen that can settle it. Fuji asks the system what opens a type only while the settings are showing, never at launch, since checking at launch is what grows into the banner a browser shows. Polite and assertive pull against each other, and the design is in holding both: fuji takes nothing that belongs to another program or to the user's choice, and lets go of nothing that belongs to it.

One thing makes fuji's case its own: it opens many kinds of file rather than one or two, and a person may well want it for some and another program for the rest, fuji for .webp and the editor they already use for .jpg. So everything here is per extension. Each one gets its own ProgID and its own name, which Explorer prints in its Type column, so a folder sorted by type keeps its .jpe files apart from its .jpg ones, and Info.plist keeps the same shape, an entry per extension. A ProgID per extension also means a different icon per format costs nothing later, and never strands a choice a user made against a ProgID fuji stopped writing. And the answer is per extension, one of yes, no, or ask, which means undecided and is where every extension starts; fuji.toml keeps them as three lists of extensions, three lines however many kinds of file fuji learns, and an extension a later fuji adds starts as ask rather than inheriting anyone's yes.

The mechanics, in brief. The formats are imageTypes in components/library.js, and Info.plist lists the same extensions by hand, since the bundle is made before any of fuji's code runs; Linux, where a desktop file would declare them, declares none yet. A Windows program an installer places has no such file, so it registers itself, and three layers decide what opens a type. The offer, which fuji writes whatever the answer, all under HKEY_CURRENT_USER: a ProgID per extension naming the type, its icon, and the command that opens it; that ProgID in the extension's OpenWithProgids list; the executable's own key under Applications, with the extensions it supports; and a Capabilities block named in RegisteredApplications, which lists fuji in Settings. The fallback, the extension's own default value, which Windows uses where the user has saved no choice: the single line that says .webp means fuji from now on, which an installer from 1999 writes at install, Tauri's NSIS macro still writes, and ActivationRegistrationManager deliberately does not. Fuji writes it for a yes, rewrites it at every launch whoever wrote it last, and takes it back for any other answer, but only while it still names fuji. And above both, the user's saved choice, UserChoice, sealed by the hash, which fuji never writes and only reads, by asking the shell what it would open each type with, the same lookup Explorer makes. On the Mac there is no fallback: the default a program can set there is the user's saved choice itself, so a yes will set it once, at the moment of the yes, and a no will set nothing, since taking it back would mean choosing another program for the user. That half is the Mac's to build.

Only the copy the installer put there registers or acts on an answer. Everything registered names the running executable's path, so a build in target/ or a copy on the Desktop would point Windows at a file that may move or vanish; the test is that the executable sits in the folder InstallLocation names under Software\Microsoft\Windows\CurrentVersion\Uninstall\Fuji, which the installer writes, an update keeps, and the uninstaller removes. That key is under HKEY_CURRENT_USER because nsis.installMode is currentUser in tauri.conf.json. Every copy shares fuji.toml in the home folder, which is why a copy that fails the test cannot change an answer either: it would be changing the installed copy's answers without being able to carry them out.

Two things elsewhere have to stay in step with this. bundle.fileAssociations stays absent from tauri.conf.json: it looks like the obvious switch, and on Windows it inserts that NSIS macro, which takes each type's default at install, silently. And the uninstall hook in src-tauri/windows/hooks.nsh takes all of it back, since these are the running program's writes and not the installer's. It learns what to take from Capabilities\FileAssociations, which this writes for every extension it offers, so the hook never needs the list; an extension offered here keeps its entry there, or the uninstaller will not know to remove it.
*/

const documentIcon = 'document-image.ico'//what a picture wears in Explorer once fuji opens its type: a file of its own, beside the executable where bundle.resources puts it, rather than fuji's own icon, which is full bleed and made to stand out in a taskbar, the wrong thing for a document to do, since a folder of pictures would become a folder of identical mint discs. One icon for every type, for now
const answerNames = ['yes', 'no', 'ask']//the three answers, which are also the three lists in fuji.toml, in the order the file shows them

export const associateAnswers = ref({})//extension to yes, no, or ask, for every extension fuji opens, in imageTypes order; filled at startup, and changed only by associateChoose and by following the system
export const associateOpens   = ref({})//extension to what windows would open it with, {program, name, executable} as registry_opens answers, name blank when nothing is registered; empty until the settings look, since fuji asks only while the user is looking at the answer
export const associateActive  = ref(false)//this copy acts on the answers: the copy the installer put there, and on windows alone until the mac has its half

let executable = ''//this copy's program file, forwardized, found at startup on windows

export async function associateStart() {//at startup, in every copy: read the answers and repair the three lists, then on the installed copy tell windows what fuji can open and claim what the user said yes to. Answers a line for the log, blank where there was nothing to say
	let {answers, problems} = answersRead()
	for (let problem of problems) log(`⭕ settings: associations, ${problem}`)
	associateAnswers.value = answers
	answersWrite()//every extension in exactly one list, which repairs a file with a mistake in it and adds an extension this fuji has and the last did not
	if (platform() != 'windows') return ''//no registry: macos declares its types in Info.plist, and linux declares none yet
	executable = await pathsExecutable()
	associateActive.value = await installed()
	if (!associateActive.value) return ''//not the copy the installer put there, whatever else it is; nothing to say, since the reason does not matter
	let changed = await queue(register)
	return `associate: ${Object.keys(answers).length} types registered, ${changed} values written`
}

export function associateChoose(extension, answer) {//the user's answer for one extension, from the settings: recorded, carried out, and looked at again, so the settings show where windows stands afterwards
	if (!associateActive.value || !imageTypes[extension] || !answerNames.includes(answer)) throw new Error(`cannot answer ${answer} for ${extension} here`)//the settings offer neither choice, so reaching this is a mistake in the code asking
	associateAnswers.value = {...associateAnswers.value, [extension]: answer}//a new object rather than an edit, so the settings see the change
	answersWrite()
	return queue(async () => { await register(); await look() })
}

export function associateLook() { return queue(look) }//ask windows what opens every extension, and follow a choice of fuji the user made there; the settings call this when they appear and whenever the window comes back into focus, as it does on the user's return from windows' own settings

export function associateOurs(extension) {//windows would open this extension with this very copy of fuji, as last looked
	let opener = associateOpens.value[extension]
	return !!opener?.executable && forwardize(opener.executable).toLowerCase() == executable.toLowerCase()
}

export function associateProgram(extension) {//the program windows would open this extension with, by the name a person knows it by: this copy's own name when it is fuji, whatever windows calls it, and blank when nothing opens it or the settings have not looked
	let opener = associateOpens.value[extension]
	if (!opener) return ''
	if (associateOurs(extension)) return brandName
	return opener.name
}

export function associateDiffers(extension) {//the answer and windows tell different stories, which only the user can settle, in windows' own settings: a yes windows opens with another program, or a no it opens with fuji. Ask never differs, being no answer at all
	if (!associateActive.value || !associateOpens.value[extension]) return false//nothing to compare until the settings have looked
	let answer = associateAnswers.value[extension]
	if (answer == 'yes') return !associateOurs(extension)
	if (answer == 'no')  return associateOurs(extension)
	return false
}

export async function associateFinish() {//windows' own Default apps, where a choice saved for another program can change: fuji's page there on windows 11, and on windows 10, which does not know the parameter, the list where set defaults by app leads to the same page
	await openUrl(`ms-settings:defaultapps?registeredAppUser=${encodeURIComponent(brandName)}`)//the name fuji registered under, escaped as Microsoft asks
}

let running = Promise.resolve()//the last pass queued, so the next waits for it: a focus can arrive while a choice is still writing
function queue(work) { running = running.then(work, work); return running }//after the pass before, whether it worked or not; the caller of that one has already heard how it went

function answersRead() {//each extension's answer from the three lists, with a line for each thing the lists got wrong. An extension is read the way a person might write it by hand, so JPG and jpg both mean .jpg
	let lists = {}//extension to the answers that list it
	let problems = []
	for (let answer of answerNames) {
		for (let item of settings.associations[answer]) {
			let extension = item.trim().toLowerCase()
			if (!extension.startsWith('.')) extension = `.${extension}`
			if (!imageTypes[extension]) { problems.push(`${item} is not a kind of file fuji opens, so it was dropped from ${answer}`); continue }
			(lists[extension] ??= new Set()).add(answer)//a set, so an extension listed twice in the same list is only there once
		}
	}
	let answers = {}
	for (let extension of Object.keys(imageTypes)) {
		let found = [...(lists[extension] ?? [])]
		answers[extension] = found.length == 1 ? found[0] : 'ask'//in none of the lists is ask, which is how a new extension arrives
		if (found.length > 1) problems.push(`${extension} is in ${found.join(' and ')}, so it counts as ask`)
	}
	return {answers, problems}
}

function answersWrite() {//the three lists as the answers say: every extension in exactly one, in imageTypes order, so the file reads the same way every time. Written to fuji.toml when fuji closes, like every setting
	for (let answer of answerNames) settings.associations[answer] = Object.keys(associateAnswers.value).filter(extension => associateAnswers.value[extension] == answer)
	settingsChanged()
}

async function installed() {//whether this is the copy the installer put there, the one gate on everything this writes
	let folder = executable.slice(0, executable.lastIndexOf('/'))//the folder the program sits in, forwardized like everything the page holds
	let recorded = await registryGet(`Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${brandName}`, 'InstallLocation')//where the installer put fuji, in quotes, or blank when fuji is not installed
	recorded = forwardize(recorded.replace(/^"|"$/g, ''))//the quotes off, and the path spelled the way the page spells every other
	return !!recorded && folder.toLowerCase() == recorded.toLowerCase()
}

async function register() {//tell windows what this copy can open, claim the fallback for each yes, and take back each other fallback while it still names fuji; answers how many values changed
	let windowsExecutable = backize(executable)//the registry wants windows' own spelling of a path, which is what backize turns a forwardized one back into
	let file = windowsExecutable.split('\\').pop()//fuji.exe, which is the key windows expects under Applications
	let command = `"${windowsExecutable}" "%1"`//quoted, because a picture's path will contain spaces; %1 is where windows puts the file
	let icon = `${windowsExecutable},0`//the executable's own icon, its first, which is still better than none
	let beside = `${executable.slice(0, executable.lastIndexOf('/'))}/${documentIcon}`
	try { await diskStat(beside); icon = `${backize(beside)},0` } catch {}//the document icon when it is there, and a stat that rejects means it is not; neither location is quoted, which is safe because windows reads an icon location by splitting at the last comma rather than at a space

	let application = `Software\\Classes\\Applications\\${file}`
	let capabilities = `Software\\${brandName}\\Capabilities`
	let changed = 0
	let set   = async (key, name, value) => { if (await registrySet(key, name, value)) changed++ }//each value read first and written only if it would change, so a pass that changed nothing knows it
	let unset = async (key, name)        => { if (await registryDelete(key, name))    changed++ }//and taking back something already gone is no change either

	for (let [extension, type] of Object.entries(imageTypes)) {
		let program = `${brandName}${extension}`//the progid, so .webp becomes Fuji.webp; the uninstall hook takes back only a progid of exactly this shape, so a change here is a change there
		await set(`Software\\Classes\\${program}`, '', type.name)//the progid: what this kind of file is called, which explorer prints in its type column
		await set(`Software\\Classes\\${program}\\DefaultIcon`, '', icon)//what explorer draws on one
		await set(`Software\\Classes\\${program}\\shell\\open\\command`, '', command)//and what opens it
		await set(`Software\\Classes\\${extension}\\OpenWithProgids`, program, '')//fuji joins the list of what could open this extension, which is the offer; the value is empty and only the name matters
		await set(`${application}\\SupportedTypes`, extension, '')//so fuji is offered for these and not for everything else
		await set(`${capabilities}\\FileAssociations`, extension, program)//so the settings app can list fuji's types, and so the uninstaller can find everything this wrote

		let fallback = `Software\\Classes\\${extension}`//the extension's own key, whose default value names the progid windows uses where the user has saved no choice
		if (associateAnswers.value[extension] == 'yes') await set(fallback, '', program)//claimed, whoever wrote it last, since the user said yes; it decides only where no choice is saved, so it never takes a type from the user
		else if (await registryGet(fallback, '') == program) await unset(fallback, '')//given back, but only while it still names fuji; what another program had there before the yes is gone, and the machine's own classes decide until the next program writes one
	}
	await set(application, 'FriendlyAppName', brandName)
	await set(`${application}\\shell\\open\\command`, '', command)
	await set(capabilities, 'ApplicationName', brandName)
	await set(capabilities, 'ApplicationDescription', brandDescription)
	await set('Software\\RegisteredApplications', brandName, capabilities)//the line that puts fuji in the settings app by name, and last on purpose: any write above can reject and stop the whole pass, so publishing fuji to Settings is the step that only happens once everything it points at is there. The next pass starts again from the top and finishes the job

	if (changed > 0) await registryNotify()//only when something moved, because this runs on every launch and almost always writes nothing
	return changed
}

async function look() {//what windows would open every extension with, and a choice of fuji already saved there, followed: an extension still at ask that windows opens with this copy becomes yes
	if (platform() != 'windows') return//the mac's half, a command asking launch services, is still to build
	let opens = {}
	for (let extension of Object.keys(imageTypes)) opens[extension] = await registryOpens(extension)//one call each, since a command takes one thing
	associateOpens.value = opens
	if (!associateActive.value) return//a copy that cannot act on an answer shows what windows says and follows nothing
	let followed = Object.keys(opens).filter(extension => associateAnswers.value[extension] == 'ask' && associateOurs(extension))//never a no, which the user said on purpose, and never a yes, which already agrees
	if (followed.length == 0) return
	associateAnswers.value = {...associateAnswers.value, ...Object.fromEntries(followed.map(extension => [extension, 'yes']))}
	answersWrite()
	log(`associate: windows opens ${followed.join(' ')} with fuji, so each is now yes`)
	await register()//and the fallbacks that go with a yes
}
