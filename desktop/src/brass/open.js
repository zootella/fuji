import {invoke} from '@tauri-apps/api/core'

//the pictures the operating system handed fuji, because the user double-clicked one rather than opening fuji and going looking; open.rs is the long version, and says why they wait in rust instead of arriving as an event the page could listen for

export function openFiles() { return invoke('open_files') }//the paths handed over since the page last asked, emptied as it answers; none on an ordinary launch, one or more when the user double-clicked. Only the window made for them receives them, and the shell asks once, at mount; a picture opened while fuji is already running gets a window of its own rather than an event here
