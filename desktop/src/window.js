import {invoke} from '@tauri-apps/api/core'

//where to put this window, as the user sees its frame, in css pixels on every platform; window.rs says why tauri's own outer rectangle is not that on windows

export function windowFrameSet(frame) { return invoke('window_frame_set', {frame}) }//put the visible frame exactly there, whatever title bar or borders the window has
