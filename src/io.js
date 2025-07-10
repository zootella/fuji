//./src/io.js

import {invoke} from '@tauri-apps/api/core'

export function ioRead(path)                { return invoke('io_read',    {path})                }
export function ioReadDir(path)             { return invoke('io_readdir', {path})                }
export function ioStat(path)                { return invoke('io_stat',    {path})                }
export function ioCopy(source, destination) { return invoke('io_copy',    {source, destination}) }
