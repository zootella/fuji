import {invoke} from '@tauri-apps/api/core'
import {pictureDir} from '@tauri-apps/api/path'
import {forwardize} from './components/library.js'

//where this copy of the program is; paths.rs is the long version

export async function pathsExecutable() {//the running program file, forwardized here, at the boundary, like every other path entering fuji; rejects only when the platform cannot say
	return forwardize(await invoke('paths_executable'))
}

//and the user's Pictures folder, where the system keeps it: Windows' known folder, which a user can move and OneDrive often does, the Mac's, and XDG's on Linux, rather than a guess under the home folder; through Tauri's path functions, which the core set in capabilities/default.json already grants. Blank when the system cannot say, which leaves a box's folder to the system
export async function picturesFolder() { try { return forwardize(await pictureDir()) } catch { return '' } }
