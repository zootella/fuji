//the trash: a file or a folder moved to the Trash or the Recycle Bin, where the user can take it back, through each platform's own call; brass's disk.rs holds the command

/*
On the Mac it is NSFileManager's trashItemAtURL rather than asking Finder through osascript, which would start a program and put up the system's prompt to let fuji control Finder. Finder offers Put Back for an item trashed this way, and not for one trashed through NSWorkspace's recycleURLs, Apple's other call for it, which records no original folder: seen in Finder's menu and in the Trash's .DS_Store on the Mac mini, macOS 27.0.1, on 2026-10-27. On Windows it is the shell's IFileOperation with undo allowed and no interface, with COM initialized around the one call as thumbnail.rs does, but single-threaded, since Microsoft says IFileOperation "cannot be used for a multithreaded apartment." On Linux it is the trash crate, which follows the freedesktop specification: the home trash for the home volume, and a trash folder at the top of any other.

Two cases are likely wrong and untried. On Windows a drive with a $RECYCLE.BIN, external ones included, takes the item as Explorer's Delete does; a drive with none, a network share or a drive Windows counts as removable, is where Explorer asks before deleting permanently, and with no interface the shell may delete outright instead, so a Move to the Recycle Bin there would be a plain delete the user never chose. In the Flatpak, --filesystem=host reaches the home trash at ~/.local/share/Trash, but Flatpak points XDG_DATA_HOME at the application's own folder under ~/.var/app, and the trash crate takes the home trash from XDG_DATA_HOME first, so an item would land in a trash the desktop never shows; the trash portal is the way out the sandbox offers.
*/

/// Move the item at path to the Trash or the Recycle Bin
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
	pub fn trash(path: &str) -> Result<(), String> { ::trash::delete(path).map_err(|e| format!("trash: {path}: {e}")) }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod platform {
	pub fn trash(path: &str) -> Result<(), String> { Err(format!("trash: no trash on this platform: {path}")) }
}
