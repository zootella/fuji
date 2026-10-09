import {invoke} from '@tauri-apps/api/core'
import {forwardize} from './components/library.js'

//the operating system's own dialog boxes, which only rust can put up; dialog.rs is the long version

export async function dialogOpen({files, folders}) {//the system's Open box over this window, for a file, a folder, or on the Mac either: the path chosen, forwardized here, at the boundary, like every other path entering fuji, or blank when the user cancels
	return forwardize(await invoke('dialog_open', {files, folders}))
}
