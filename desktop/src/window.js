import {invoke} from '@tauri-apps/api/core'

//where this window is and where to put it, as the user sees its frame, in css pixels on every platform; window.rs says why tauri's own outer rectangle is not that on windows

export function windowEngine() { return invoke('window_engine') }//the version of the web engine drawing this window, as the platform reports it; the page names the engine, which is fixed per platform
export function windowNew(note) { return invoke('window_new', {note}) }//another window of this application, made hidden, whose page takes this note as it mounts; window.rs holds the word and never reads it
export function windowNote() { return invoke('window_note') }//the note this window was made with, taken away as it answers; blank for a window the operating system, the dock or the menu bar asked for
export function windowDrag() { return invoke('window_drag') }//move this window with the pointer until the left button comes up, placed by fuji on every move on the mac and windows, so it goes past the screen's edges and the menu bar where the platform's own drag would stop it; on linux, the desktop's own drag
export function windowConstraintLift() { return invoke('window_constraint_lift') }//let this window go anywhere and be any size, past the screen's edges and over the menu bar, with the mac's constraint lifted for it alone. Nothing elsewhere
export function windowTitleHide() { return invoke('window_title_hide') }//stop drawing this window's title in its title bar, on the mac, where the bar keeps its buttons; the title stays set for the menus and taskbars that list windows. Nothing elsewhere
export function windowFrame() { return invoke('window_frame') }//{x, y, width, height} of the visible frame
export function windowFrameSet(frame) { return invoke('window_frame_set', {frame}) }//put the visible frame exactly there, whatever title bar or borders the window has
export function windowFullscreenEnter() { return invoke('window_fullscreen_enter') }//into fuji's own fullscreen, which on the mac hides the menu bar and the dock until the last window in it comes out
export function windowFullscreenLeave() { return invoke('window_fullscreen_leave') }//out of fullscreen without changing whether the window is showing, which tauri's own way does not promise on windows

//a native rectangle for a video: on windows a child window above the page, on the mac and linux a GL view of fuji's own above it that mpv draws into when asked, at a rectangle of the page, which the page measures in css pixels and hands over in backing ones, since a native child is placed in those; window.rs says why there is one and how the video gets into it on each
export function windowVideoMake({x, y, width, height}) { return invoke('window_video_make', {x: Math.round(x * devicePixelRatio), y: Math.round(y * devicePixelRatio), width: Math.round(width * devicePixelRatio), height: Math.round(height * devicePixelRatio)}) }//its handle as a decimal string, for mpv's wid option, which media.rs resolves off windows into the view it names
export function windowVideoPlace({x, y, width, height}) { return invoke('window_video_place', {x, y, width, height}) }//move and resize it to a rectangle already in whole backing pixels, which the page works out itself so the css box can be set to the same pixels, and show it; a rectangle with no area hides it
export function windowVideoDrop() { return invoke('window_video_drop') }//remove it, if there is one. The mouse over it comes to the page as the window event video-mouse, down, up or move with the point in its rectangle in backing pixels, which the panel listens for; a file dropped on it comes as the window event video-drop with its paths, which the shell listens for beside the web view's own drop
