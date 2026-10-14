import {invoke} from '@tauri-apps/api/core'

//the media library, libmpv, reached through mpv.rs: one function per command, and the page decides what plays and when. A player belongs to the window that started it, and the names in the calls below are mpv's own, from its manual: an option like vid or hwdec, a command like loadfile or seek, a property like time-pos

export function mpvVersion() { return invoke('mpv_version') }//{mpv, api, ffmpeg}: the library's own version text, which names itself first, the client api it was built against, and the FFmpeg beneath it; rejects where the library is not linked
export function mpvStart(options) { return invoke('mpv_start', {options}) }//a player for this window, from a list of [name, value] pairs set before it starts; replaces the one this window had
export function mpvSend(args) { return invoke('mpv_send', {args}) }//one mpv command and its arguments, a list of strings, like ['loadfile', path] or ['cycle', 'pause']; resolves when it has run
export function mpvRead(name) { return invoke('mpv_read', {name}) }//one property as text, blank where it has no value yet
export function mpvStop() { return invoke('mpv_stop') }//destroy this window's player; nothing if it has none
