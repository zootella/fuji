import {invoke} from '@tauri-apps/api/core'

//where this window is and where to put it, as the user sees its frame, in css pixels on every platform; window.rs says why tauri's own outer rectangle is not that on windows

export function windowEngine() { return invoke('window_engine') }//the version of the web engine drawing this window, as the platform reports it; the page names the engine, which is fixed per platform
export function windowFrame() { return invoke('window_frame') }//{x, y, width, height} of the visible frame
export function windowFrameSet(frame) { return invoke('window_frame_set', {frame}) }//put the visible frame exactly there, whatever title bar or borders the window has
export function windowFullscreenLeave() { return invoke('window_fullscreen_leave') }//out of fullscreen without changing whether the window is showing, which tauri's own way does not promise on windows

//a native rectangle for a video: on windows a child window above the page, on the mac and linux a GL view of fuji's own above it that mpv draws into when asked, at a rectangle of the page, which the page measures in css pixels and hands over in backing ones, since a native child is placed in those; window.rs says why there is one and how the video gets into it on each
export function windowVideoMake({x, y, width, height}) { return invoke('window_video_make', {x: Math.round(x * devicePixelRatio), y: Math.round(y * devicePixelRatio), width: Math.round(width * devicePixelRatio), height: Math.round(height * devicePixelRatio)}) }//its handle as a decimal string, for mpv's wid option, which mpv.rs resolves off windows into the view it names
export function windowVideoDrop() { return invoke('window_video_drop') }//remove it, if there is one. The mouse over it comes to the page as the window event video-mouse, down, up or move with the point in its rectangle in backing pixels, which the panel listens for; a file dropped on it comes as the window event video-drop with its paths, which the shell listens for beside the web view's own drop
