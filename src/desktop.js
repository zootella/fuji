//./src/desktop.js

import {invoke} from '@tauri-apps/api/core'

export function desktopExitHold(path, text) { return invoke('desktop_exit_hold', {path, text}) }//update the text rust will write to path when fuji exits, or pass blank to forget the path
