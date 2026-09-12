import {invoke} from '@tauri-apps/api/core'

//the pictures the operating system handed fuji, because the user double-clicked one rather than opening fuji and going looking; open.rs is the long version, and says why they wait in rust instead of arriving as an event the page could listen for

export function openFiles() { return invoke('open_files') }//the paths handed over since the page last asked, emptied as it answers; none on an ordinary launch, one when the user double-clicked a picture. The shell also drains this from the 'open' event, which macos raises when fuji is already running
