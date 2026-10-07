import {invoke} from '@tauri-apps/api/core'

//every one of these rejects with a message rather than throwing a typed error, and none of them makes a parent folder for you

export function diskRead(path)                { return invoke('disk_read',    {path})                }//the whole file at once, as an ArrayBuffer; wrap it in a Uint8Array to use it
export function diskWrite(path, data)         { return invoke('disk_write',   {path, data})          }//creates the file, or truncates an existing one to nothing first; data is an array of byte values
export function diskReadDir(path)             { return invoke('disk_readdir', {path})                }//one folder, not its subfolders: name, is_file, is_dir, is_symlink, size, and mtime as milliseconds since 1970, 0 where the filesystem has no answer. Skips any entry it cannot stat rather than failing the whole listing
export function diskPeek(path, extensions)    { return invoke('disk_peek',    {path, extensions})    }//one folder at a glance: files, how many visible regular files have one of the extensions given with their dots, and folders, the names of its visible subfolders with symlinks left out; from the directory read alone, no stat per entry, so a folder of fifty thousand files answers in a moment
export function diskStat(path)                { return invoke('disk_stat',    {path})                }//one path's metadata: the same three kind flags, size, and atime, mtime, birthtime as milliseconds since 1970, each 0 where the filesystem has no answer. Describes a symlink itself rather than following it
export function diskCopy(source, destination) { return invoke('disk_copy',    {source, destination}) }//files only, overwrites the destination without asking, and cannot report progress or be cancelled
