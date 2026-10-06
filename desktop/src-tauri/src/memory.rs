use serde::Serialize;
use std::sync::{mpsc, OnceLock};
use std::time::Duration;
use tauri::{command, WebviewWindow};
use crate::run_blocking;

/*
How much memory the machine has, how much of it is in use, and what this process and its web engine's processes take: the numbers Activity Monitor and Task Manager show, asked of the same places those tools ask, so a reading here matches what the user sees there. One command, memory_report, answers all of it at once, and the page does every sum and every judgment; nothing here knows what a bucket is. The machine's total is also what the size wall in thumbnail.rs measures a header against, through memory_total below, asked once and kept.

The page is not where the page's memory is. A webview is a view in this process onto a page rendered in the engine's own processes: WebKit's Web Content and GPU processes on the Mac, and WebView2's browser, renderer, GPU and utility processes on Windows. Every canvas, every decoded picture and the JavaScript heap live there, and this process holds the Rust, the window, and whatever thumbnails are in flight. So a report that stopped at this process would miss the thing a user watching the contact sheet fill is watching. Each platform finds those processes its own way, and each body says how. The GPU process on either platform serves every window of the program, one per application on the Mac and one per browser instance on Windows, and the browser process on Windows serves every copy of the program using the same data folder, so a row is labeled by kind and never claimed as this window's alone.

The pids come from the webview itself, which Tauri hands over only on the main thread, later, through with_webview. So memory_report sends them back on a channel and the body waits for them on the blocking pool, the shape launch_set in launch.rs uses for the Mac's answer. A webview that never answers within WEBVIEW_WAIT leaves the report with this process and the machine's rows, rather than failing it.

Linux answers the machine's rows from the kernel's listing and this process from its own status file, and lists no engine processes: nobody publishes fuji from Linux, and the report is for the two platforms that are published.
*/

const WEBVIEW_WAIT: Duration = Duration::from_secs(2);//how long memory_report waits for the webview's process ids before answering without them; they come at once, since the closure only reads a few numbers, so this only keeps a lost answer from holding a report

#[derive(Serialize)]
pub struct Row { name: String, bytes: u64 }//one of the platform's own breakdown rows, named as its tool names it, like App Memory on the Mac or Committed on Windows

#[derive(Serialize)]
pub struct Process { kind: String, pid: u32, bytes: u64 }//one process and what it takes by the platform's own measure, phys_footprint on the Mac and the private working set on Windows, which are Activity Monitor's and Task Manager's columns; kind is host for this process, which the page names, and the engine's own words for the rest, like web content, gpu, renderer

#[derive(Serialize)]
pub struct Report {
	installed: u64,//the memory the machine was bought with, in bytes
	total: u64,//the memory the operating system can use, which on Windows is a little less than installed
	used: u64,//in use right now, as the platform's own tool counts it
	available: u64,//what the platform says is free for an application to take
	details: Vec<Row>,//the rest of the platform's own breakdown
	processes: Vec<Process>,//this process first, then the web engine's
}

/// The machine's memory, what is in use, and what this process and its web engine's processes take; the essay above says what each number is
#[command]
pub async fn memory_report(window: WebviewWindow) -> Result<Report, String> {
	let (sender, receiver) = mpsc::channel();
	let _ = window.with_webview(move |webview| { let _ = sender.send(platform::engine(webview)); });//runs on the main thread, later; an error here means no webview to ask, and the wait below gives up at its deadline
	run_blocking(move || {
		let engine = receiver.recv_timeout(WEBVIEW_WAIT).unwrap_or_default();//the engine's processes as (kind, pid), or none when the webview never answered
		let total = memory_total();
		let (used, available, details) = platform::machine(total);
		let host = std::process::id();
		let mut processes = vec![Process { kind: "host".into(), pid: host, bytes: platform::footprint(host) }];
		processes.extend(engine.into_iter().map(|(kind, pid)| Process { kind, pid, bytes: platform::footprint(pid) }));
		Ok(Report { installed: platform::installed(), total, used, available, details, processes })
	}).await
}

/// The machine's physical memory in bytes, asked once; 8 GB when the platform will not say, which is a floor rather than a guess about any real machine
pub fn memory_total() -> u64 {
	static MEMORY: OnceLock<u64> = OnceLock::new();
	*MEMORY.get_or_init(|| { let m = platform::total(); if m > 0 { m } else { 8 << 30 } })
}

fn row(name: &str, bytes: u64) -> Row { Row { name: name.to_string(), bytes } }

#[cfg(target_os = "macos")]
mod platform {
	use std::ffi::{c_void, CStr};
	use objc2::runtime::AnyObject;
	use objc2::{msg_send, sel};
	use super::{row, Row};

	extern "C" {//declared by hand because libc stops short of two of these and marks the third deprecated in favor of a crate fuji does not carry; the linker finds all three in the system
		fn mach_host_self() -> libc::mach_port_t;
		static mach_task_self_: libc::mach_port_t;
		fn mach_port_deallocate(task: libc::mach_port_t, name: libc::mach_port_t) -> libc::kern_return_t;
	}

	fn sysctl<T>(name: &CStr) -> Option<T> {//one value from sysctlbyname, of whatever plain type the name answers in
		let mut value: T = unsafe { std::mem::zeroed() };
		let mut length = std::mem::size_of::<T>();
		let status = unsafe { libc::sysctlbyname(name.as_ptr(), &mut value as *mut T as *mut c_void, &mut length, std::ptr::null_mut(), 0) };
		if status == 0 { Some(value) } else { None }
	}

	pub fn total() -> u64 { sysctl::<u64>(c"hw.memsize").unwrap_or(0) }//how the mac says how much memory it has
	pub fn installed() -> u64 { total() }//the same number: the mac reports the memory it was built with

	pub fn machine(total: u64) -> (u64, u64, Vec<Row>) {//used, available, and activity monitor's memory tab, App Memory, Wired Memory, Compressed, Cached Files and Swap Used, from the page counts it reads and by the sums the tools that match it use
		let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) }.max(0) as u64;
		let mut stats: libc::vm_statistics64 = unsafe { std::mem::zeroed() };
		let mut count = libc::HOST_VM_INFO64_COUNT;
		let host = unsafe { mach_host_self() };//a send right this process now holds, given back below
		let status = unsafe { libc::host_statistics64(host, libc::HOST_VM_INFO64, &mut stats as *mut libc::vm_statistics64 as libc::host_info64_t, &mut count) };
		unsafe { mach_port_deallocate(mach_task_self_, host) };//every mach_host_self adds a reference to the port's name, and a report every two seconds for hours would run it up
		if status != 0 { return (0, 0, vec![]) }
		let pages = |n: u32| n as u64 * page;
		let app = pages(stats.internal_page_count).saturating_sub(pages(stats.purgeable_count));//what applications hold and cannot give back
		let wired = pages(stats.wire_count);//the kernel's, never paged out
		let compressed = pages(stats.compressor_page_count);
		let cached = pages(stats.external_page_count) + pages(stats.purgeable_count);//file pages and purgeable memory, which the system takes back on demand
		let used = app + wired + compressed;//activity monitor's Memory Used
		let swap = sysctl::<libc::xsw_usage>(c"vm.swapusage").map(|s| s.xsu_used).unwrap_or(0);
		let details = vec![row("App Memory", app), row("Wired Memory", wired), row("Compressed", compressed), row("Cached Files", cached), row("Swap Used", swap)];
		(used, total.saturating_sub(used), details)
	}

	pub fn footprint(pid: u32) -> u64 {//phys_footprint, activity monitor's Memory column: the process's own pages, compressed or not, and the iokit memory it maps, which is where an IOSurface lands
		let mut info: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
		let status = unsafe { libc::proc_pid_rusage(pid as i32, libc::RUSAGE_INFO_V4, &mut info as *mut libc::rusage_info_v4 as *mut libc::rusage_info_t) };//libc types the buffer as a pointer to the void pointer, as the header does, so the struct's address is cast one step further than it looks

		if status == 0 { info.ri_phys_footprint } else { 0 }//0 for a process this one may not ask about, which another user's would be and the engine's are not
	}

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
	use windows::Win32::Foundation::{CloseHandle, HANDLE};
	use windows::Win32::System::ProcessStatus::{GetPerformanceInfo, GetProcessMemoryInfo, PERFORMANCE_INFORMATION, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX, PROCESS_MEMORY_COUNTERS_EX2};
	use windows::Win32::System::SystemInformation::{GetPhysicallyInstalledSystemMemory, GlobalMemoryStatusEx, MEMORYSTATUSEX};
	use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
	use windows_core::Interface;//for cast, from the version webview2-com is built on rather than fuji's own, which Cargo.toml explains
	use super::{row, Row};

	fn status() -> Option<MEMORYSTATUSEX> {//the one call task manager's memory panel is mostly made of
		let mut status = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };//the length field is how windows knows which version of the struct it was handed
		unsafe { GlobalMemoryStatusEx(&mut status).ok().map(|_| status) }
	}

	pub fn total() -> u64 { status().map(|s| s.ullTotalPhys).unwrap_or(0) }//what windows can use, after the hardware's reservations
	pub fn installed() -> u64 {//what the box was bought with, from the firmware's own table, so 16 GB reads as 16 GB rather than the 15.9 windows has the use of
		let mut kilobytes = 0u64;
		unsafe { GetPhysicallyInstalledSystemMemory(&mut kilobytes).ok().map(|_| kilobytes * 1024).unwrap_or_else(|| total()) }
	}

	pub fn machine(total: u64) -> (u64, u64, Vec<Row>) {//used, available, and the rest of task manager's memory panel: Committed against its limit, and Cached
		let Some(s) = status() else { return (0, 0, vec![]) };
		let available = s.ullAvailPhys;
		let mut details = vec![row("Committed", s.ullTotalPageFile.saturating_sub(s.ullAvailPageFile)), row("Commit limit", s.ullTotalPageFile)];
		let mut performance = PERFORMANCE_INFORMATION { cb: std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32, ..Default::default() };
		if unsafe { GetPerformanceInfo(&mut performance, performance.cb) }.is_ok() { details.push(row("Cached", performance.SystemCache as u64 * performance.PageSize as u64)) }
		(total.saturating_sub(available), available, details)
	}

	pub fn footprint(pid: u32) -> u64 {//the private working set, task manager's Memory column, where this windows offers it, and the working set where it does not: the counters come in two sizes, and a windows before 10's 2004 release knows only the older
		let Ok(handle) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else { return 0 };//0 for a process this one may not open, which another user's would be and the engine's are not
		let bytes = unsafe { counters(handle) };
		let _ = unsafe { CloseHandle(handle) };
		bytes
	}
	unsafe fn counters(handle: HANDLE) -> u64 {
		let mut newer = PROCESS_MEMORY_COUNTERS_EX2 { cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX2>() as u32, ..Default::default() };
		if GetProcessMemoryInfo(handle, &mut newer as *mut PROCESS_MEMORY_COUNTERS_EX2 as *mut PROCESS_MEMORY_COUNTERS, newer.cb).is_ok() && newer.PrivateWorkingSetSize > 0 { return newer.PrivateWorkingSetSize as u64 }
		let mut older = PROCESS_MEMORY_COUNTERS_EX { cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32, ..Default::default() };
		if GetProcessMemoryInfo(handle, &mut older as *mut PROCESS_MEMORY_COUNTERS_EX as *mut PROCESS_MEMORY_COUNTERS, older.cb).is_ok() { return older.WorkingSetSize as u64 }
		0
	}

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
	use super::{row, Row};

	fn kilobytes(text: &str, key: &str) -> u64 {//one line of a kernel listing, in kilobytes there, as bytes; 0 for a key it lacks
		let line = text.lines().find(|line| line.starts_with(key)).unwrap_or("");
		line.split_whitespace().nth(1).and_then(|kb| kb.parse::<u64>().ok()).map(|kb| kb * 1024).unwrap_or(0)
	}
	fn meminfo(key: &str) -> u64 { kilobytes(&std::fs::read_to_string("/proc/meminfo").unwrap_or_default(), key) }

	pub fn total() -> u64 { meminfo("MemTotal:") }
	pub fn installed() -> u64 { total() }//a little under what the box was bought with, since the kernel keeps some back before it counts
	pub fn machine(total: u64) -> (u64, u64, Vec<Row>) {
		let available = meminfo("MemAvailable:");
		(total.saturating_sub(available), available, vec![row("Cached", meminfo("Cached:")), row("Swap Used", meminfo("SwapTotal:").saturating_sub(meminfo("SwapFree:")))])
	}
	pub fn footprint(pid: u32) -> u64 { kilobytes(&std::fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default(), "VmRSS:") }//resident, which is the nearest the kernel's listing comes to the other two platforms' columns
	pub fn engine(_webview: tauri::webview::PlatformWebview) -> Vec<(String, u32)> { vec![] }//no engine rows here, as the essay says
}
