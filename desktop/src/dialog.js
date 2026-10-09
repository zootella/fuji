import {invoke} from '@tauri-apps/api/core'
import {forwardize} from './components/library.js'

//the operating system's own dialog boxes, which only rust can put up; dialog.rs is the long version

export async function dialogOpen() {//the system's Open box over this window, for one file of any kind, or on the Mac one file or folder: the path chosen, forwardized here, at the boundary, like every other path entering fuji, or blank when the user cancels
	return forwardize(await invoke('dialog_open'))
}
