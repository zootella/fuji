import {invoke} from '@tauri-apps/api/core'

//the windows registry, read and written for the page, all under HKEY_CURRENT_USER; registry.rs is the long version, and associate.js is the caller that knows which keys

export function registryGet(key, name)        { return invoke('registry_get',    {key, name})        }//one string value, a blank name being the key's default value; resolves blank when the key or the value is not there, and rejects off windows
export function registrySet(key, name, value) { return invoke('registry_set',    {key, name, value}) }//one string value, creating the key if it is missing; resolves true if the value changed and false if it already said this, and rejects off windows
export function registryDelete(key, name)     { return invoke('registry_delete', {key, name})        }//one value, a blank name being the default; resolves true if there was one to delete, and rejects off windows
export function registryNotify()              { return invoke('registry_notify')                     }//tell the shell that file associations changed, so explorer catches up without a sign-out
export function registryOpens(extension)      { return invoke('registry_opens',  {extension})        }//what windows would open a file type with right now, the type with its dot: {program, name, executable}, each blank where windows has none, all three blank when nothing is registered; rejects off windows
