//copper: fuji's rust that needs neither tauri nor a window and cannot be safe rust, every call into c and into the operating system's own libraries away from a window, each a plain function a thin command in brass wraps, so that a search for unsafe here is the whole list of such calls fuji makes outside a window

pub mod media;//the media library, libmpv: its declarations, and the questions asked of a file without a window
