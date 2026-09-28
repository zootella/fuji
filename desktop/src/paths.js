import {invoke} from '@tauri-apps/api/core'
import {forwardize} from './components/library.js'

//where this copy of the program is; paths.rs is the long version

export async function pathsExecutable() {//the running program file, forwardized here, at the boundary, like every other path entering fuji; rejects only when the platform cannot say
	return forwardize(await invoke('paths_executable'))
}
