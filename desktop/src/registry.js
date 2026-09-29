import {invoke} from '@tauri-apps/api/core'

//the windows registry, read and written for the page, all under HKEY_CURRENT_USER, and every call rejects off windows; registry.rs is the long version, and associate.js is the caller that knows which keys

export function registryGet(key, name)        { return invoke('registry_get',    {key, name})        }//one string value, a blank name meaning the key's default; blank when missing
export function registrySet(key, name, value) { return invoke('registry_set',    {key, name, value}) }//one string value, creating the key; true if it changed, false if it already said this
export function registryDelete(key, name)     { return invoke('registry_delete', {key, name})        }//one value; true if there was one to delete
export function registryNotify()              { return invoke('registry_notify')                     }//tell the shell that file associations changed, so explorer catches up without a sign-out
export function registryOpens(extension)      { return invoke('registry_opens',  {extension})        }//what windows would open a type like .png with now, as {name, executable}, each blank where it has none
