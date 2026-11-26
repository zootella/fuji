import {invoke} from '@tauri-apps/api/core'

//launch services, the mac's record of which application opens which kind of file, and every call rejects off the mac; launch.rs is the long version, and associate.js is the caller that knows which extensions

export function launchOpens(extension) { return invoke('launch_opens', {extension}) }//what the mac would open a type like .png with now, as {name, executable, kind}, each blank where it has none: the shape registryOpens answers on windows, and the kind of file, like public.jpeg, which windows has no answer for
export function launchSet(extension)   { return invoke('launch_set',   {extension}) }//make this running copy the application that opens a type, and every type sharing its kind, with no dialog
