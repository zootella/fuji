import {invoke} from '@tauri-apps/api/core'

//the folder tree, many folders read in one call; find.rs says why a call reads a stretch rather than one folder

export function findFolders(from, forward, extensions, want, examine) { return invoke('find_folders', {from, forward, extensions, want, examine}) }//the next folders in sorted preorder after this one, or before it, holding a file with one of the extensions: {folders: [{path, files}], stopped, done, examined, refused}, up to want of them from at most examine folders read; continue from stopped in the same direction until done. The glance's rule for visible names, symlinks and extensions throughout
