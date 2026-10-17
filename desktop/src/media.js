import {invoke} from '@tauri-apps/api/core'

//the media library, libmpv, reached through media.rs: one function per command, and the page decides what plays and when. A player belongs to the window that started it, and the names in the calls below are mpv's own, from its manual: an option like vid or hwdec, a command like loadfile or seek, a property like time-pos

export function mediaVersion() { return invoke('media_version') }//{mpv, api, ffmpeg}: the library's own version text, which names itself first, the client api it was built against, and the FFmpeg beneath it; rejects where the library is not linked
export function mediaStart(options) { return invoke('media_start', {options}) }//a player for this window, from a list of [name, value] pairs set before it starts; replaces the one this window had
export function mediaSend(args) { return invoke('media_send', {args}) }//one mpv command and its arguments, a list of strings, like ['loadfile', path] or ['cycle', 'pause']; resolves when it has run
export function mediaRead(name) { return invoke('media_read', {name}) }//one property as text, blank where it has no value yet
export function mediaStop() { return invoke('media_stop') }//destroy this window's player; nothing if it has none

//asking a file a question rather than playing it, each call from a player of its own that never touches the window's; seconds is how long the page will wait for the answer

export function mediaProbe(path, seconds) { return invoke('media_probe', {path, seconds}) }//{duration, format, tracks}: the running time in seconds as text, blank where the file has none; the demuxer's name for its format; and the track list as mpv's own JSON, each video track with demux-w, demux-h, demux-par and demux-rotation, read from the headers with nothing decoded, and demux-par absent where the headers do not say, as in a DVD's MPEG-2. Rejects for a file the library will not open, and for one that gives no answer within seconds
export function mediaFrame(path, at, seconds) { return invoke('media_frame', {path, at, seconds}) }//one ArrayBuffer: the frame at at seconds, for mediaFrameUnpack below. Rejects for a file with no video, and for one that gives no answer within seconds

export function mediaFrameUnpack(buffer) {//the frame's width, height and pixels out of that buffer, shaped for new ImageData(pixels, width, height); the frame is already at the shape the video shows at, turned and with its pixels made square
	let header = new DataView(buffer, 0, 8)
	let width = header.getUint32(0, true), height = header.getUint32(4, true)//little endian, as media.rs writes them
	return {width, height, pixels: new Uint8ClampedArray(buffer, 8, width * height * 4)}
}
