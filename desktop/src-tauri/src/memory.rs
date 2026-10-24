//the memory command: the engine's process ids asked of this window's webview, which only this crate can reach, and the report itself from copper

use std::sync::mpsc;
use std::time::Duration;
use tauri::{command, WebviewWindow};
use crate::run_blocking;
use copper::memory::Report;

const WEBVIEW_WAIT: Duration = Duration::from_secs(2);//how long memory_report waits for the webview's process ids before answering without them; they come at once, since the closure only reads a few numbers, so this only keeps a lost answer from holding a report

/// The machine's memory, what is in use, and what this process and its web engine's processes take; the essay above says what each number is
#[command]
pub async fn memory_report(window: WebviewWindow) -> Result<Report, String> {
	let (sender, receiver) = mpsc::channel();
	let _ = window.with_webview(move |webview| { let _ = sender.send(platform::engine(webview)); });//runs on the main thread, later; an error here means no webview to ask, and the wait below gives up at its deadline
	run_blocking(move || Ok(copper::memory::report(receiver.recv_timeout(WEBVIEW_WAIT).unwrap_or_default()))).await//the engine's processes as (kind, pid), or none when the webview never answered
}

#[cfg(target_os = "macos")]
mod platform {
	use objc2::runtime::AnyObject;
	use objc2::{msg_send, sel};

	pub fn engine(webview: tauri::webview::PlatformWebview) -> Vec<(String, u32)> {//the web content and gpu processes behind this window's WKWebView, asked of the view itself. The two selectors are WebKit's private interface, the only road to a pid: read-only, there since 10.11, and each asked only after the view says it answers to it, so a WebKit without one leaves its row out rather than crashing
		let view: &AnyObject = unsafe { &*(webview.inner() as *const AnyObject) };
		let mut found = Vec::new();
		let answers: bool = unsafe { msg_send![view, respondsToSelector: sel!(_webProcessIdentifier)] };
		if answers { let pid: i32 = unsafe { msg_send![view, _webProcessIdentifier] }; if pid > 0 { found.push(("web content".to_string(), pid as u32)) } }
		let answers: bool = unsafe { msg_send![view, respondsToSelector: sel!(_gpuProcessIdentifier)] };
		if answers { let pid: i32 = unsafe { msg_send![view, _gpuProcessIdentifier] }; if pid > 0 { found.push(("gpu".to_string(), pid as u32)) } }//one per application, shared by every window, and where recent macOS draws canvases, so a thumbnail's pixels may be charged here rather than to web content
		found
	}
}

#[cfg(target_os = "windows")]
mod platform {
	use webview2_com::Microsoft::Web::WebView2::Win32::*;
	use webview2_core::Interface;//for cast, from the windows-core webview2-com is built on rather than fuji's own, under the name Cargo.toml gives it and explains

	pub fn engine(webview: tauri::webview::PlatformWebview) -> Vec<(String, u32)> { unsafe { engine_of(&webview.controller()) } }
	unsafe fn engine_of(controller: &ICoreWebView2Controller) -> Vec<(String, u32)> {//every process of the WebView2 environment this window's view runs in, with its kind, from webview2's own list; the browser and gpu processes serve every window of every copy of the program using this data folder, and the kind says which is which
		let mut found = Vec::new();
		let Ok(core) = controller.CoreWebView2() else { return found };
		let Ok(core2) = core.cast::<ICoreWebView2_2>() else { return found };
		let Ok(environment) = core2.Environment() else { return found };
		let Ok(environment8) = environment.cast::<ICoreWebView2Environment8>() else { return found };//the process list arrived with webview2 1.0.1108, and an older runtime answers no rows
		let Ok(infos) = environment8.GetProcessInfos() else { return found };
		let mut count = 0u32;
		if infos.Count(&mut count).is_err() { return found }
		for i in 0..count {
			let Ok(info) = infos.GetValueAtIndex(i) else { continue };
			let (mut pid, mut kind) = (0i32, COREWEBVIEW2_PROCESS_KIND::default());
			if info.ProcessId(&mut pid).is_err() || info.Kind(&mut kind).is_err() || pid <= 0 { continue }
			let name = match kind {
				COREWEBVIEW2_PROCESS_KIND_BROWSER => "browser",
				COREWEBVIEW2_PROCESS_KIND_RENDERER => "renderer",
				COREWEBVIEW2_PROCESS_KIND_GPU => "gpu",
				COREWEBVIEW2_PROCESS_KIND_UTILITY => "utility",
				COREWEBVIEW2_PROCESS_KIND_SANDBOX_HELPER => "sandbox helper",
				COREWEBVIEW2_PROCESS_KIND_PPAPI_PLUGIN => "plugin",
				COREWEBVIEW2_PROCESS_KIND_PPAPI_BROKER => "plugin broker",
				_ => "other",
			};
			found.push((name.to_string(), pid as u32));
		}
		found
	}
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
	pub fn engine(_webview: tauri::webview::PlatformWebview) -> Vec<(String, u32)> { vec![] }//no engine rows here, as the essay says
}
