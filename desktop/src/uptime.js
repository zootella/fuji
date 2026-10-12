import {invoke} from '@tauri-apps/api/core'

//how long this process has been running, from the operating system's record of when it started and from main's own clock; the difference is what the loader spent before main, which nothing in the program can time for itself. uptime.rs says where each platform keeps the start

export function uptimeProcess() { return invoke('uptime_process') }//{process, main}: milliseconds since the operating system started this process, 0 when it would not say, and milliseconds since main began
