import {invoke} from '@tauri-apps/api/core'

//the system's icon for a kind of file, which iconCache.js asks for and keeps, and on windows where the system keeps it, which associate.js registers; icon.rs in copper says how each platform is asked

export function iconRender({kind, extension, css, backingPerCss}) { return invoke('icon_render', {kind, extension, css, backingPerCss}) }//png bytes in an ArrayBuffer: the system's icon for a kind, css pixels square at backingPerCss backing pixels each. The kind is folder; image, video or audio, the medium's own icon; or file, the icon for a file of this extension, lowercased and without its dot. Rejects where the system gives no icon, which on linux today is every kind
export function iconLocation(kind) { return invoke('icon_location', {kind}) }//where windows keeps its own icon for a kind, folder, image, video, audio, or file for the blank page, written the way a DefaultIcon in the registry takes one, which associate.js registers; rejects off windows
