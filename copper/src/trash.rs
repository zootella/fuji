//the trash: a file or a folder moved to the Trash or the Recycle Bin, where the user can take it back, through each platform's own call; brass's disk.rs holds the command

/*
On the Mac it is NSFileManager's trashItemAtURL rather than asking Finder through osascript, which would start a program and put up the system's prompt to let fuji control Finder. Finder offers Put Back for an item trashed this way, and not for one trashed through NSWorkspace's recycleURLs, Apple's other call for it, which records no original folder: seen in Finder's menu and in the Trash's .DS_Store on the Mac mini, macOS 27.0.1, on 2026-10-27. On Windows it is the shell's IFileOperation with undo allowed and no interface, with COM initialized around the one call as thumbnail.rs does, but single-threaded, since Microsoft says IFileOperation "cannot be used for a multithreaded apartment." On Linux it is GLib's own call, g_file_trash, the one Nautilus makes, which follows the freedesktop specification: the home trash for the home volume, a trash folder at the top of any other, and an error where neither can be used. The trash crate was tried first and found to copy a file to the home trash when the top of its volume refuses a trash folder, which GLib never does; a copy of a file the user asked to put away is a second copy to find, so where fuji cannot trash a file it refuses, and never copies. Inside a Flatpak GLib sends the call through the trash portal by itself; without it the home trash would resolve under the sandbox's own data folder, a trash the desktop never shows.
*/

/*
What the call does depends on the drive, and on Windows that includes deleting for good. Measured on Windows 10 on 2026-10-30: on the system drive the item goes to the Recycle Bin and Restore brings it back; on an exFAT USB stick, which Windows counts as removable and gives no Recycle Bin, the item is deleted for good, nothing comes to the screen, and the answer is still Ok, where Explorer's Delete asks first. Expected and untried there: a network share deletes for good as the stick does, and an external drive that reports itself as fixed recycles as the system drive does. On the Mac, Put Back was seen on the Mac mini's own volume, as above; expected and untried, a USB stick gets a .Trashes folder of its own and recycles, and a network share has no trash, so the call answers an error rather than deleting. On Linux GLib answers an error where it cannot trash, by its own documentation. So Ok means the user can take the item back everywhere except on a Windows drive with no Recycle Bin, and whether fuji deletes anyway there, asks first, or refuses is for the feature that calls this to decide; this decides nothing.
*/

/// Move the item at path to the Trash or the Recycle Bin; on Windows, a drive with no Recycle Bin deletes it for good, as the essay above says
pub fn trash(path: &str) -> Result<(), String> { platform::trash(path) }

#[cfg(target_os = "macos")]
mod platform {
	use objc2_foundation::{NSFileManager, NSString, NSURL};

	pub fn trash(path: &str) -> Result<(), String> {
		let url = NSURL::fileURLWithPath(&NSString::from_str(path));
		NSFileManager::defaultManager().trashItemAtURL_resultingItemURL_error(&url, None).map_err(|e| format!("trash: {path}: {}", e.localizedDescription()))
	}
}

#[cfg(target_os = "windows")]
mod platform {
	use windows::core::HSTRING;
	use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED};
	use windows::Win32::UI::Shell::{FileOperation, IFileOperation, IShellItem, SHCreateItemFromParsingName, FOF_ALLOWUNDO, FOF_NO_UI};

	struct Com;//com initialized on this thread for as long as this lives, as in thumbnail.rs
	impl Drop for Com { fn drop(&mut self) { unsafe { CoUninitialize() } } }

	/*
	Windows gives a drive a Recycle Bin by the kind of drive it reports itself as, not by its file system: a drive it counts as fixed keeps a hidden $RECYCLE.BIN folder at its top, and a delete with undo allowed is a move into it, while a drive it counts as removable, as most USB sticks and camera cards are, keeps none. There Explorer's Delete first asks "Are you sure you want to permanently delete this file?", and FOF_NO_UI holds FOF_NOCONFIRMATION, which answers that question yes for the user. Adding FOF_WANTNUKEWARNING, which Microsoft says "partially overrides FOF_NOCONFIRMATION", or FOFX_RECYCLEONDELETE changed nothing on the stick: each deleted its file for good without a word. Two ways for a feature to find out in time are untried: leaving FOF_NOCONFIRMATION out, so the shell asks its own question, which may then ask on the system drive too; and an IFileOperationProgressSink, whose PreDeleteItem may be told whether the item is about to be recycled, and can stop the delete before it happens.
	*/
	pub fn trash(path: &str) -> Result<(), String> {
		let fail = |e: &dyn std::fmt::Display| format!("trash: {path}: {e}");
		unsafe {
			let _com = if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok() { Some(Com) } else { None };//first, so it drops last, after every object below
			let operation: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL).map_err(|e| fail(&e))?;
			operation.SetOperationFlags(FOF_ALLOWUNDO | FOF_NO_UI).map_err(|e| fail(&e))?;//to the Recycle Bin, and nothing put on the screen
			let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path.replace('/', "\\")), None).map_err(|e| fail(&e))?;//the shell parses its own names with backslashes
			operation.DeleteItem(&item, None).map_err(|e| fail(&e))?;
			operation.PerformOperations().map_err(|e| fail(&e))?;
			if operation.GetAnyOperationsAborted().map_err(|e| fail(&e))?.as_bool() { return Err(fail(&"the shell stopped it")) }
		}
		Ok(())
	}
}

#[cfg(target_os = "linux")]
mod platform {
	use gio::prelude::FileExt;

	pub fn trash(path: &str) -> Result<(), String> { gio::File::for_path(path).trash(gio::Cancellable::NONE).map_err(|e| format!("trash: {path}: {e}")) }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod platform {
	pub fn trash(path: &str) -> Result<(), String> { Err(format!("trash: no trash on this platform: {path}")) }
}
