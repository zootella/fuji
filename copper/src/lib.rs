//copper: fuji's rust that needs neither tauri nor a window and cannot be safe rust, every call into c and into the operating system's own libraries away from a window, each a plain function a thin command in brass wraps, so that a search for unsafe here is the whole list of such calls fuji makes outside a window

pub mod media;//the media library, libmpv: its declarations, and the questions asked of a file without a window
pub mod inventory;//an inventory of a folder's whole tree, every file hashed and scouted and the videos asked of FFmpeg, a line each, by the loop brass's commands start and stop
pub mod sandbox;//how the sandbox process is started, the walls it puts around itself, and the check that tries them
pub mod uptime;//when the operating system started this process
pub mod panel;//how many pixels the main display really has
pub mod launch;//launch services, the mac's record of which application opens which kind of file
pub mod registry;//the windows registry, read and written for the page
pub mod memory;//the machine's memory and what fuji's processes take of it
pub mod thumbnail;//the operating system's thumbnailer, ImageIO and the Windows Imaging Component
