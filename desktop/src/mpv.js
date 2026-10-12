import {invoke} from '@tauri-apps/api/core'

//the media library, libmpv, reached through mpv.rs: one function per command, and the page decides what plays and when

export function mpvVersion() { return invoke('mpv_version') }//{mpv, api, ffmpeg}: the library's own version text, which names itself first, the client api it was built against, and the FFmpeg beneath it; rejects where the library is not linked
