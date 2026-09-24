use tauri::command;

/*
A trackpad or a Magic Mouse scrolls by sliding a finger, and the page cannot tell that from a wheel. Both reach it as wheel events, and a swipe is dozens of them with a momentum tail after the fingers lift, where the table reads each one as a command: one swipe flips through a dozen pictures. This module is the first answer, on the Mac alone: tell touch from a wheel where it can be told reliably, and drop it before the page sees it, for the window whose page asked.

The reliable signal is AppKit's. Every scroll event macOS delivers says whether its deltas are precise, which is true for every trackpad and the Magic Mouse and false for a notched wheel. WebKit reads that flag to choose pixel or line scrolling and never exposes it to the DOM, nor the gesture's phase nor its momentum; what the page sees is a delta macOS has already accelerated, so one slow notch of a mouse arrives as a few pixels and looks like the first event of a swipe. So the question is asked here, once, where the event is made, and never in the page.

A local event monitor sees every scroll wheel event dispatched to this process before the window it is aimed at does, and a monitor that answers nil has dropped the event. Fuji installs one at setup and keeps it for the life of the process. The page's half is one command per window, on or off: the shell turns it on while a table is showing and off while the sheet is, because the sheet scrolls by exactly these events. Windows are told apart by the address of the NSWindow, which is what tauri hands over and what the event names. An address a closed window leaves behind is harmless, because a window later made at that address states its own wish before it is revealed.

What this accepts as wrong: smooth-scrolling utilities such as Mos, Mac Mouse Fix and Logitech's Options+ re-post a mouse's notches as precise events, which is how they make scrolling smooth, so a mouse behind one reads as a trackpad and its owner has the keyboard until touch has commands of its own. Windows and Linux do nothing here, and the command is a no-op there: a trackpad is rarer on Windows, and Chromium's deltas are legible enough that the page may one day tell a notch from a swipe by itself.
*/

/// Watch scroll wheel events from the moment fuji starts, so a page can ask below; nothing to install off the mac
pub fn touch_start() {
	#[cfg(target_os = "macos")]
	platform::touch_start();
}

/// Drop every trackpad and magic mouse scroll aimed at this window before the page sees it, or stop dropping them; a no-op off the mac
#[command]
pub fn touch_block(window: tauri::WebviewWindow, on: bool) -> Result<(), String> {
	#[cfg(target_os = "macos")]
	return platform::touch_block(&window, on);

	#[cfg(not(target_os = "macos"))]
	{
		let _ = (window, on);//there is nothing to do with them here; the same reasoning as associate.rs
		Ok(())
	}
}

#[cfg(target_os = "macos")]
mod platform {
	use std::ptr::NonNull;
	use std::sync::Mutex;
	use std::sync::atomic::{AtomicBool, Ordering};
	use block2::RcBlock;
	use objc2::rc::Retained;
	use objc2::MainThreadMarker;
	use objc2_app_kit::{NSEvent, NSEventMask};
	use crate::log;

	static BLOCKED: Mutex<Vec<usize>> = Mutex::new(Vec::new());//the addresses of the windows whose pages asked for precise scrolls to be dropped; a handful at most, so a list rather than a set. A Mutex because the command arrives on tauri's pool threads and the monitor runs on the main one
	static SEEN: AtomicBool = AtomicBool::new(false);//whether the monitor has fired yet, so the log can say once a run that it is live

	pub fn touch_start() {
		let block = RcBlock::new(|event: NonNull<NSEvent>| -> *mut NSEvent {//appkit calls this on the main thread for every scroll wheel event, before the window sees it; answer the event to pass it on, or null to drop it
			let e = unsafe { event.as_ref() };//appkit promises a live event for the length of the call
			let precise = e.hasPreciseScrollingDeltas();//true for a trackpad or a magic mouse, false for a notched wheel
			if !SEEN.swap(true, Ordering::Relaxed) { log::log(&format!("⭕ touch: the monitor saw its first scroll event, precise {precise}")) }//once, so a log from any machine says whether the hook is live and what its first device was
			if precise {
				if let Some(window) = MainThreadMarker::new().and_then(|marker| e.window(marker)) {//the window the event is aimed at; the marker is a formality, since the monitor only ever runs on the main thread
					let address = Retained::as_ptr(&window) as usize;
					if blocked().contains(&address) { return std::ptr::null_mut() }//dropped: the page never hears of it
				}
			}
			event.as_ptr()//passed on unchanged
		});
		let token = unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::ScrollWheel, &block) };
		std::mem::forget(token); std::mem::forget(block);//the monitor lives as long as the process, so neither is ever released: the token is what removeMonitor would want and nothing calls it, and the block is what appkit calls
	}

	pub fn touch_block(window: &tauri::WebviewWindow, on: bool) -> Result<(), String> {
		let address = objc2::rc::autoreleasepool(|_| window.ns_window()).map_err(|e| format!("touch: {e}"))? as usize;//the NSWindow tauri built, which is the same object the monitor's event names. Tauri autoreleases the pointer it answers, and which thread runs a command is tauri's choice rather than fuji's: on the main thread the run loop's own pool would drain it, and this pool does the same wherever the command lands, rather than leaking a retain of the window on a thread with no pool. Only the address survives the drain, and it is never dereferenced
		let mut list = blocked();
		list.retain(|a| *a != address);//off; and on says it again below, so a window asking twice is in the list once
		if on { list.push(address) }
		Ok(())
	}

	fn blocked() -> std::sync::MutexGuard<'static, Vec<usize>> { BLOCKED.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }//take the list even if a previous holder panicked, as log.rs does
}
