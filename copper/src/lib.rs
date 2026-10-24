//copper: fuji's rust that needs neither tauri nor a window and cannot be safe rust, every call into c and into the operating system's own libraries away from a window, each a plain function a thin command in brass wraps, so that a search for unsafe here is the whole list of such calls fuji makes outside a window

pub mod media;//the media library, libmpv: its declarations, and the questions asked of a file without a window
pub mod inventory;//an inventory of a folder's whole tree, every file hashed and scouted and the videos asked of FFmpeg, a line each, by the loop brass's commands start and stop
