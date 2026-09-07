import {invoke} from '@tauri-apps/api/core'

export function desktopExitHold(path, text)   { return invoke('desktop_exit_hold',   {path, text}) }//update the text rust will write to path when fuji exits, or pass blank to forget the path
export function desktopExitAppend(path, text) { return invoke('desktop_exit_append', {path, text}) }//add to the end of it instead, for a caller that produces its file a line at a time
