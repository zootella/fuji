import {invoke} from '@tauri-apps/api/core'
import {imageTypes} from './components/library.js'

//what fuji tells the operating system it can open; associate.rs is the long version and associations.md is the whole subject, including why fuji offers every one of these and claims none of them

export function associateRegister() {//called once at startup; writes the registry on windows, and does nothing on mac, on linux, or in a debug build
	let types = Object.entries(imageTypes).map(([extension, type]) => ({
		extension,//with its leading dot, as the registry wants it
		program: `Fuji${extension}`,//the progid, so ".webp" becomes "Fuji.webp": one per extension rather than one for all of them, so a document icon per format is possible later without stranding the choices users have already made against a progid fuji stopped writing
		name: type.name,//what explorer prints in its type column
	}))
	return invoke('associate_register', {types})
}
