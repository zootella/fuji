import {diskStat} from './disk.js'
import {registryGet, registrySet, registryNotify} from './registry.js'
import {pathsExecutable} from './paths.js'
import {brandName, brandDescription} from './brand.js'
import {imageTypes, backize, forwardize, platform} from './components/library.js'

/*
What fuji has told the operating system it can open: every extension in imageTypes, with the name it should carry. The shell runs this once at startup, and it does something only on Windows, and only for the copy the installer put there; everywhere else it does nothing and says nothing. The policy is all here, in plain JavaScript, over general commands in registry.rs that read a value, write one if it would change, and tell the shell, and know nothing about what they are reading or writing.

macOS needs no code, because its declaration is not code. CFBundleDocumentTypes sits in Info.plist inside the .app, LaunchServices reads it when it first sees the bundle, and dragging Fuji.app into Applications is the whole of the registration. Windows has no such file, so an application registers itself.

Two things have to be said clearly about how, because the subject is thick with folklore. The first is what fuji writes, all under HKEY_CURRENT_USER: a ProgID per extension naming the type and the icon and the command that opens it, that ProgID added to the extension's OpenWithProgids list, the executable's own key with the extensions it supports, and a Capabilities block registered so the Settings app lists fuji by name with its types. The second is what fuji does not write: the extension's own default value, the single line that says .webp means fuji from now on. That line is the one an installer from 1999 would write, it is the one Tauri's bundled NSIS macro still writes, and it is the one Microsoft's own current API — ActivationRegistrationManager, whose implementation is open — deliberately does not.

**Which is why `bundle.fileAssociations` is absent from tauri.conf.json and must stay absent.** It is Tauri's own feature for this and it looks like the obvious thing to turn on. On macOS it would be fine; on Windows it inserts that NSIS macro, which takes each extension's default value at install time, silently, without asking — the one thing this whole design refuses to do. Everything here makes fuji available. Nothing here makes fuji default, because since Windows 8 the default lives in a hash-protected key that only the user, through the system's own interface, can set. So after this runs, fuji is in Explorer's Open with menu and listed in Settings under Default apps, and every file on the machine still opens with whatever opened it before. associations.md carries the reasoning, the alternatives, and what each key is for.

The icon those types wear is a file rather than the application, because an application icon is full bleed and unmistakable, which is exactly wrong on a document: a folder of pictures would be a folder of identical mint discs. So document-image.ico ships beside the executable through bundle.resources and DefaultIcon names it, with fuji's own icon as the fallback if it is not there. One icon for all ten types today; the ProgIDs are per extension, so a different icon per format costs nothing later. icon.md owns the artwork.

Two practical notes. It runs on every launch, which is cheap because each value is read before it is written and an unchanged value is not touched; the shell is only notified if something actually moved. And it has one gate, which is the whole of how it tells an installed copy from anything else: the running executable has to be in the folder the installer recorded. Every install writes an entry under Software\Microsoft\Windows\CurrentVersion\Uninstall named for the product, the one the Settings app lists, and its InstallLocation value is the folder the user chose on the installer's folder page, in quotes; the uninstaller removes the entry, and an update keeps it. So a copy installed anywhere registers, and nothing else does. The command paths come from the executable, so registering any other copy would point the registry at a file that may move or vanish, a debug or release build in target/ that gets rebuilt above all. None of them registers, and none needs to say why. The entry is under HKEY_CURRENT_USER because nsis.installMode is currentUser in tauri.conf.json; a per-machine install would write it under the machine's hive instead, where registry.rs does not look.
*/

const documentIcon = 'document-image.ico'//beside the executable, put there by bundle.resources

export async function associateRegister() {//tell windows what an installed copy can open, claiming none of it; called once at startup, and answers a line for the log, blank where there was nothing to do
	if (platform() != 'windows') return ''//there is no registry to ask or write: macos declares its types in Info.plist, and linux declares none yet, as associations.md records
	let running = await pathsExecutable()
	let folder = running.slice(0, running.lastIndexOf('/'))//the folder the program sits in, forwardized like everything the page holds
	let recorded = await registryGet(`Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${brandName}`, 'InstallLocation')//where the installer put fuji, in quotes, or blank when fuji is not installed
	recorded = forwardize(recorded.replace(/^"|"$/g, ''))//the quotes off, and the path spelled the way the page spells every other
	if (!recorded || folder.toLowerCase() != recorded.toLowerCase()) return ''//not the copy the installer put there, whatever else it is; nothing to say, since the reason does not matter

	let executable = backize(running)//the registry wants windows' own spelling of a path, which is what backize turns a forwardized one back into
	let file = executable.split('\\').pop()//fuji.exe, which is the key windows expects under Applications
	let command = `"${executable}" "%1"`//quoted, because a picture's path will contain spaces; %1 is where windows puts the file
	let icon = `${executable},0`//the executable's own icon, its first, which is still better than none
	let beside = `${folder}/${documentIcon}`
	try { await diskStat(beside); icon = `${backize(beside)},0` } catch {}//the document icon when it is there, and a stat that rejects means it is not; neither location is quoted, which is safe because windows reads an icon location by splitting at the last comma rather than at a space

	let application = `Software\\Classes\\Applications\\${file}`
	let capabilities = `Software\\${brandName}\\Capabilities`
	let changed = 0
	let set = async (key, name, value) => { if (await registrySet(key, name, value)) changed++ }//each value read first and written only if it would change, so a launch that changed nothing knows it

	let types = Object.entries(imageTypes)
	for (let [extension, type] of types) {
		let program = `${brandName}${extension}`//the progid, so .webp becomes Fuji.webp: one per extension rather than one for all of them, so a document icon per format is possible later without stranding the choices users have already made against a progid fuji stopped writing
		await set(`Software\\Classes\\${program}`, '', type.name)//the progid: what this kind of file is called, which explorer prints in its type column
		await set(`Software\\Classes\\${program}\\DefaultIcon`, '', icon)//what explorer draws on one
		await set(`Software\\Classes\\${program}\\shell\\open\\command`, '', command)//and what opens it
		await set(`Software\\Classes\\${extension}\\OpenWithProgids`, program, '')//fuji joins the list of what could open this extension, which is the offer; the value is empty and only the name matters
		await set(`${application}\\SupportedTypes`, extension, '')//so fuji is offered for these and not for everything else
		await set(`${capabilities}\\FileAssociations`, extension, program)//and so the settings app can list fuji's types
	}
	await set(application, 'FriendlyAppName', brandName)
	await set(`${application}\\shell\\open\\command`, '', command)
	await set(capabilities, 'ApplicationName', brandName)
	await set(capabilities, 'ApplicationDescription', brandDescription)
	await set('Software\\RegisteredApplications', brandName, capabilities)//the line that puts fuji in the settings app by name, and last on purpose: any write above can reject and stop the whole run, so publishing fuji to Settings is the step that only happens once everything it points at is there. The next launch starts again from the top and finishes the job

	if (changed > 0) await registryNotify()//only when something moved, because this runs on every launch and almost always writes nothing
	return `associate: ${types.length} types registered, ${changed} values written`
}
