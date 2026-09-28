import tauriConfiguration from '../src-tauri/tauri.conf.json' with {type: 'json'}//the standard way to import json as a module, which vite and node both read

//the product's name and one-line description, read from tauri.conf.json when vite builds the page, so the one place they are written is the file every build of the application already reads. Rename the application there and the page and the rust follow: rust reads the same file through its package info

export const brandName        = tauriConfiguration.productName            //Fuji, as it is shown: the registry's names for it, and a menu or a window title
export const brandFile        = brandName.toLowerCase()                   //fuji, as it is spelled in a file or folder name, which is lowercase on every platform
export const brandDescription = tauriConfiguration.bundle.shortDescription//the one line the installers and the settings app show beside the name
