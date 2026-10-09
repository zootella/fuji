import {invoke} from '@tauri-apps/api/core'

//other programs; process.rs is the long version

export function processOpen(target) { return invoke('process_open', {target}) }//open this file, folder or address with the program the system has for it, the way a double-click would, and let it go: an https address in the browser, an ms-settings address in Windows Settings
