use tauri::command;
use crate::run_blocking;

/*
Launch Services, the Mac's record of which application opens which kind of file, offered to the page the way registry.rs offers the Windows registry: two general commands any Mac application could use as they are. Which extensions to ask about, when, and what to do with the answers is the page's; associate.js is the one caller.

The Mac files that record by kind of file rather than by extension, so both commands turn the extension into its UTType first, with typeWithFilenameExtension. Extensions that share a kind share an answer: .jpg, .jpeg and .jpe are all public.jpeg, so setting one sets all three. An extension the system has no kind for still gets one, a dynamic type made up from the extension, like .jfif's dyn.ah62d4rv4ge80y3xmq2, and the system keeps a choice for that like any other: seen on the Mac mini 2026-10-01, where Get Info had already given .jfif's to an application and launch_opens named it.

launch_opens answers in the shape registry_opens does on Windows, the application's name and its program file, so the page compares the two platforms' answers the same way: the program file inside the bundle, like /Applications/Preview.app/Contents/MacOS/Preview, rather than the bundle itself, because that is what the page already knows about the copy it is running in. It also answers the kind, the UTType's identifier, which Windows has no answer for: an extension's kind alone says nothing of which others share it, but the page asks about every extension it opens, and those whose kinds match are the ones that move together.

launch_set makes the running application the one that opens a kind, through NSWorkspace's setDefaultApplicationAtURL, which arrived in macOS 12. It changes the user's own saved choice, the same one Get Info's Change All writes, and the system asks nobody: seen on the Mac mini 2026-10-01, it answered at once with no dialog, and Finder and Get Info both followed. That is the difference from Windows, where only the system's own screens can write a saved choice, and it is why associate.js can carry out a choice on the Mac at the click that makes it.

Both are async and run their bodies through run_blocking, since each waits on another program, the Launch Services daemon. Off the Mac there is no Launch Services, and each command answers so.
*/

/// What opens a kind of file, each part blank where nothing does
#[derive(serde::Serialize)]
pub struct Opener {
	name: String,//what a person calls it, like Preview, as the Finder shows it
	executable: String,//the program file inside its bundle, as a full path
	kind: String,//the kind of file the mac files the extension under, like public.jpeg for .jpg, .jpeg and .jpe alike
}

/// Which application the Mac would open a file type with, the type written with its dot, like .png
#[command]
pub async fn launch_opens(extension: String) -> Result<Opener, String> {
	run_blocking(move || platform::opens(&extension)).await
}

/// Make the running application the one that opens a file type, and every type sharing its kind
#[command]
pub async fn launch_set(extension: String) -> Result<(), String> {
	run_blocking(move || platform::set(&extension)).await
}

#[cfg(target_os = "macos")]
mod platform {
	use std::time::Duration;
	use objc2::rc::Retained;
	use objc2_app_kit::NSWorkspace;
	use objc2_foundation::{NSBundle, NSError, NSFileManager, NSString};
	use objc2_uniform_type_identifiers::UTType;
	use super::Opener;

	const SET_WAIT: Duration = Duration::from_secs(10);//how long launch_set waits for the system's answer; it came at once on the Mac mini, so this only keeps a lost answer from holding a thread forever

	//the kind of file an extension names, with or without its dot
	fn kind(extension: &str) -> Result<Retained<UTType>, String> {
		UTType::typeWithFilenameExtension(&NSString::from_str(extension.trim_start_matches('.'))).ok_or_else(|| format!("launch: no kind of file for {extension}"))
	}

	pub fn opens(extension: &str) -> Result<Opener, String> {
		objc2::rc::autoreleasepool(|_| {//a pool for what these calls autorelease: this runs on a blocking pool thread, which has none of its own, and without one the objects wait for the thread to retire
			let kind = kind(extension)?;
			let identifier = kind.identifier().to_string();
			let Some(application) = NSWorkspace::sharedWorkspace().URLForApplicationToOpenContentType(&kind) else { return Ok(Opener { name: String::new(), executable: String::new(), kind: identifier }) };//nothing opens this kind
			let name = application.path().map(|path| NSFileManager::defaultManager().displayNameAtPath(&path).to_string()).unwrap_or_default();//the name the Finder shows, without .app
			let executable = NSBundle::bundleWithURL(&application).and_then(|bundle| bundle.executableURL()).and_then(|file| file.path()).map(|path| path.to_string()).unwrap_or_default();
			Ok(Opener { name, executable, kind: identifier })
		})
	}

	pub fn set(extension: &str) -> Result<(), String> {
		objc2::rc::autoreleasepool(|_| {//the same pool as opens, for the same reason
			let kind = kind(extension)?;
			let application = NSBundle::mainBundle().bundleURL();//the running application's bundle; a program run outside one gets its folder here, which the system would refuse, and the page never asks from one
			let (sender, receiver) = std::sync::mpsc::channel();
			let named = extension.to_string();//the block outlives this call's borrow, so it carries its own copy for the message
			let done = block2::RcBlock::new(move |error: *mut NSError| {//called once, on a queue of the system's choosing, with null for success
				let answer = if error.is_null() { Ok(()) } else { Err(format!("launch: could not set the application for {named}, {}", unsafe { &*error }.localizedDescription())) };
				let _ = sender.send(answer);//nobody listening means the wait below already gave up
			});
			NSWorkspace::sharedWorkspace().setDefaultApplicationAtURL_toOpenContentType_completionHandler(&application, &kind, Some(&done));
			receiver.recv_timeout(SET_WAIT).map_err(|_| format!("launch: no answer setting the application for {extension}"))?
		})
	}
}

#[cfg(not(target_os = "macos"))]
mod platform {
	use super::Opener;
	const NONE: &str = "launch: there is no launch services on this platform";
	pub fn opens(_extension: &str) -> Result<Opener, String> { Err(NONE.to_string()) }
	pub fn set(_extension: &str) -> Result<(), String> { Err(NONE.to_string()) }
}
