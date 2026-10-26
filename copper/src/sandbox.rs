//the sandbox: how the main process starts the sandbox process, the walls an operating system offers a process, which the sandbox process puts around itself between reading its request and reading its first file, and a check that tries the walls from inside

/*
The sandbox process may read the folder it was given, and nothing else: no other file, no write anywhere, no network, no program started, no service of the system asked. Those are what code a hostile file runs reaches for, taking itself for a program running as the user, and the walls refuse them whatever code is asking. The process puts the walls on itself, once, after the request has told it the folder and before the walk opens a file, so no byte of a file arrives outside them; what the loader mapped and ran before main, the program's own libraries, came from the program and not from a file.

On the Mac the wall is Apple's sandbox, the kernel's own, given a profile in its language through sandbox_init_with_parameters, the call Firefox makes, outside what Apple supports and in Apple's own library. The profile denies everything and allows reads under the folder, and the folder's path reaches it as a parameter rather than spliced into the text, so no name can change what the profile says. On Linux the walls are two: Landlock allows reads under the folder and nothing else, and a seccomp filter refuses the rest, a socket, so neither the network nor the desktop's message bus, a program started, another process attached to or signaled. A kernel without Landlock, Raspberry Pi OS's among them, leaves the files open, and enter goes on rather than refusing, since seccomp's half still holds and the check says which walls did. Before its walls the process makes itself one that dumps no core, so a crash reaches no crash reporter through the kernel, apport's or ABRT's. On Windows the walls are a job object and the integrity level, which is how Chromium walls its renderers. The process makes a job that allows it one active process, so a program it starts fails to start, that ends it at an unhandled exception and when the job's last handle closes, with a limit on committed memory, and with the restrictions on the desktop Chromium puts on its job at lockdown, the clipboard, global atoms, the display settings and the desktop itself among them; then it lowers its own token to the Untrusted integrity level, below Low, which refuses every write to a file or a registry key the user owns and the reading or writing of another process's memory, while leaving reads of the user's files, since Windows has no wall around a folder a process can put on itself, the network, since the AppContainer that would close it cannot open the user's folder by path, and ending another process of the user's, which Windows counts as neither a read nor a write of it, measured in the check. Before either it sets its error mode so a crash puts up no dialog and reaches no Windows Error Reporting. The job's flag against an unhandled exception promises the same and misses a fast fail, which is how Rust aborts and how the C runtime's abort() ends FFmpeg's av_assert0, since a fast fail never reaches the place where the job's flag is read; the error mode is read before that place. Measured on the Windows 10 box on 2026-10-25, with the error mode set an access violation, a stack overflow, a Rust abort and a C abort each ended the process with its exit code reaching the parent and nothing written to the reporter's archive or the event log, and with the job's flag alone the two aborts were reported.

On the Mac the main process starts the sandbox process itself, with posix_spawn, and on Windows with CreateProcessW, where Linux uses Rust's Command. Command came first here too, and a crash of a sandbox process started that way put up macOS's "quit unexpectedly" dialog, whose Reopen would start fuji over, and wrote a crash report, though the main process had caught the crash and gone on; a night of crashes would leave a stack of dialogs. ReportCrash counts the sandbox process as the application, since it is the application's own executable, and since macOS 10.15 it hears of a crash from EXC_CORPSE_NOTIFY, which the kernel raises only when nothing handled the process's EXC_CRASH. Command cannot set a new process's exception ports, and posix_spawn can: the sandbox process starts with its EXC_CRASH and EXC_CORPSE_NOTIFY pointed at a port the main process holds, where a thread answers each one handled, as Crashpad's ExceptionSwallower keeps its test children's crashes from the system, and the main process learns of the crash as before, from how the process ended. A handler inside the sandbox process could not do this, since a thread that overflows its stack, as one of mpv's can on a hostile file, leaves a signal handler no stack to run on. Holding the spawn, the main process also starts the sandbox process with nothing open but its three standard streams, whatever WebKit and AppKit opened without asking for it to close, since the walls judge a file as it opens and a handle already open would pass them, and with an empty environment, which it needs nothing from. On Windows Command would start the new process holding every handle the main process has marked inheritable, since Rust asks CreateProcessW to inherit handles and names none, so the main process calls CreateProcessW itself with a list of the three streams, the only way Windows has to say which; the three are marked inheritable for the one call and no longer after. It starts the process with its own error mode rather than this process's, which a new process otherwise inherits, so the sandbox process's quiet is its own: node sets the same bit in every process it starts, and a sandbox process that inherited it from the walk passed the walk's crash step before any wall was built. And it starts it with an environment of SystemRoot, without which the system's own libraries fail to load, and the temporary folder, so the check tries the real one.

The check, fuji --sandbox-check with a folder and a file outside it, puts the same walls on itself and tries the reaches one by one, printing what each answered beside what it should, so each platform's walls are a run and not a belief.
*/

pub const CHECK_SWITCH: &str = "--sandbox-check";//the first argument that makes fuji's executable run the check

/// Start the sandbox process: program with the one argument, its stdout the file given, its stdin and stderr pipes handed back with the process
pub fn start(program: &std::path::Path, switch: &str, stdout: std::fs::File) -> std::io::Result<(Child, Box<dyn std::io::Write + Send>, Box<dyn std::io::Read + Send>)> { platform::start(program, switch, stdout) }
pub use platform::Child;//the sandbox process once started: kill it, and wait for how it ended

/// Wall this process in to reading the folder and nothing else, for good; an error when the platform refuses, which the sandbox process takes as a reason not to start
pub fn enter(folder: &str) -> Result<(), String> { platform::enter(folder) }

#[cfg(target_os = "macos")]
mod platform {
	use std::ffi::{c_char, c_int, c_void, CStr, CString};
	use std::os::fd::AsRawFd;
	use std::os::unix::ffi::OsStrExt;
	use std::os::unix::process::ExitStatusExt;

	pub struct Child { pid: libc::pid_t }
	impl Child {
		pub fn kill(&mut self) -> std::io::Result<()> { if unsafe { libc::kill(self.pid, libc::SIGKILL) } == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) } }
		pub fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
			let mut status = 0;
			loop {
				if unsafe { libc::waitpid(self.pid, &mut status, 0) } == self.pid { return Ok(std::process::ExitStatus::from_raw(status)) }
				let e = std::io::Error::last_os_error();
				if e.kind() != std::io::ErrorKind::Interrupted { return Err(e) }
			}
		}
	}

	extern "C" {//from the SDK's spawn.h and mach headers, which the libc crate does not declare
		fn posix_spawnattr_setexceptionports_np(attributes: *mut libc::posix_spawnattr_t, mask: u32, port: u32, behavior: c_int, flavor: c_int) -> c_int;//as if the new process had called task_set_exception_ports
		static mach_task_self_: u32;//this process's own task port
		fn mach_port_allocate(task: u32, right: u32, name: *mut u32) -> c_int;
		fn mach_port_insert_right(task: u32, name: u32, right: u32, kind: u32) -> c_int;
		fn mach_msg_destroy(message: *mut c_void);//every right and every piece of memory a received message carried, let go
		fn mach_msg(message: *mut c_void, option: c_int, send_size: u32, receive_size: u32, receive_port: u32, timeout: u32, notify: u32) -> c_int;
	}
	const EXC_MASK_CRASH: u32 = 1 << 10;//by their names in mach/exception_types.h
	const EXC_MASK_CORPSE_NOTIFY: u32 = 1 << 13;
	const EXCEPTION_DEFAULT: c_int = 1;
	#[cfg(target_arch = "aarch64")] const THREAD_STATE_NONE: c_int = 5;//mach/arm/thread_status.h
	#[cfg(target_arch = "x86_64")] const THREAD_STATE_NONE: c_int = 13;//mach/i386/thread_status.h
	const MACH_PORT_RIGHT_RECEIVE: u32 = 1;//and mach/port.h and mach/message.h
	const MACH_MSG_TYPE_MAKE_SEND: u32 = 20;
	const MACH_SEND_MSG: c_int = 1;
	const MACH_RCV_MSG: c_int = 2;
	const MACH_MSGH_BITS_REMOTE_MASK: u32 = 0x1f;

	pub fn start(program: &std::path::Path, switch: &str, stdout: std::fs::File) -> std::io::Result<(Child, Box<dyn std::io::Write + Send>, Box<dyn std::io::Read + Send>)> {
		let port = crash_port()?;
		let (stdin_read, stdin_write) = std::io::pipe()?;//both pipes close on exec, so a program started beside this one never holds them
		let (stderr_read, stderr_write) = std::io::pipe()?;
		let program = CString::new(program.as_os_str().as_bytes())?;
		let switch = CString::new(switch)?;
		let arguments = [program.as_ptr(), switch.as_ptr(), std::ptr::null()];
		let environment: [*const c_char; 1] = [std::ptr::null()];
		let mut pid = 0;
		let code = unsafe {
			let mut actions: libc::posix_spawn_file_actions_t = std::ptr::null_mut();
			let mut attributes: libc::posix_spawnattr_t = std::ptr::null_mut();
			libc::posix_spawn_file_actions_init(&mut actions);
			libc::posix_spawnattr_init(&mut attributes);
			let set = [//each a code, 0 for done
				libc::posix_spawn_file_actions_adddup2(&mut actions, stdin_read.as_raw_fd(), 0),
				libc::posix_spawn_file_actions_adddup2(&mut actions, stdout.as_raw_fd(), 1),
				libc::posix_spawn_file_actions_adddup2(&mut actions, stderr_write.as_raw_fd(), 2),
				libc::posix_spawnattr_setflags(&mut attributes, libc::POSIX_SPAWN_CLOEXEC_DEFAULT as libc::c_short),//every other file closed in the new process
				posix_spawnattr_setexceptionports_np(&mut attributes, EXC_MASK_CRASH | EXC_MASK_CORPSE_NOTIFY, port, EXCEPTION_DEFAULT, THREAD_STATE_NONE),
			].into_iter().find(|code| *code != 0).unwrap_or(0);
			let code = if set != 0 { set } else { libc::posix_spawn(&mut pid, program.as_ptr(), &actions, &attributes, arguments.as_ptr() as *const *mut c_char, environment.as_ptr() as *const *mut c_char) };
			libc::posix_spawn_file_actions_destroy(&mut actions);
			libc::posix_spawnattr_destroy(&mut attributes);
			code
		};
		if code != 0 { return Err(std::io::Error::from_raw_os_error(code)) }
		Ok((Child { pid }, Box::new(stdin_write), Box::new(stderr_read)))//the new process's ends of the pipes, and the file, close here, theirs now alone
	}

	fn crash_port() -> std::io::Result<u32> {//the main process's port for the sandbox processes' crashes, made once with the thread that answers them
		static PORT: std::sync::OnceLock<Result<u32, String>> = std::sync::OnceLock::new();
		PORT.get_or_init(|| {
			let task = unsafe { mach_task_self_ };
			let mut port = 0;
			if unsafe { mach_port_allocate(task, MACH_PORT_RIGHT_RECEIVE, &mut port) } != 0 { return Err("could not make a port for crashes".into()) }
			if unsafe { mach_port_insert_right(task, port, port, MACH_MSG_TYPE_MAKE_SEND) } != 0 { return Err("could not give the port for crashes a send right".into()) }
			std::thread::Builder::new().name("sandbox crashes".into()).spawn(move || crashes_answer(port)).map_err(|e| e.to_string())?;
			Ok(port)
		}).clone().map_err(std::io::Error::other)
	}

	#[repr(C)] struct Header { bits: u32, size: u32, remote: u32, local: u32, voucher: u32, id: i32 }//mach_msg_header_t
	#[repr(C)] struct Reply { header: Header, ndr: [u8; 8], code: c_int }//exception_raise's reply: the header, NDR_record, and the answer

	fn crashes_answer(port: u32) {//each crash a sandbox process sends here, answered handled, which ends its telling there; the sandbox process has ended either way
		let mut buffer = [0u64; 128];//1 KiB, sized here, and aligned for the header
		loop {
			if unsafe { mach_msg(buffer.as_mut_ptr().cast(), MACH_RCV_MSG, 0, std::mem::size_of_val(&buffer) as u32, port, 0, 0) } != 0 { continue }//one too large is thrown away by the kernel; wait for the next
			let request = unsafe { &mut *(buffer.as_mut_ptr() as *mut Header) };
			let mut reply = Reply { header: Header { bits: request.bits & MACH_MSGH_BITS_REMOTE_MASK, size: std::mem::size_of::<Reply>() as u32, remote: request.remote, local: 0, voucher: 0, id: request.id.wrapping_add(100) }, ndr: [0, 0, 0, 0, 1, 0, 0, 0], code: 0 };//as a MIG server answers: the reply right the request carried, its id plus 100, and KERN_SUCCESS
			unsafe { mach_msg((&mut reply as *mut Reply).cast(), MACH_SEND_MSG, reply.header.size, 0, 0, 0, 0) };
			request.remote = 0;//the reply right, which the reply just spent
			unsafe { mach_msg_destroy(buffer.as_mut_ptr().cast()) };//and the rest, the rights to the crashed thread and task among them
		}
	}

	extern "C" {//in libSystem, which every program links; the SDK's header declares neither any more, so these are as Firefox's Sandbox.mm declares them
		fn sandbox_init_with_parameters(profile: *const c_char, flags: u64, parameters: *const *const c_char, errorbuf: *mut *mut c_char) -> c_int;//0, or -1 with the reason in errorbuf
		fn sandbox_free_error(errorbuf: *mut c_char);
	}

	const PROFILE: &str = "(version 1)
(deny default)
(allow file-read* (subpath (param \"FOLDER\")))
";

	pub fn enter(folder: &str) -> Result<(), String> {
		let fail = |e: &dyn std::fmt::Display| format!("sandbox: {folder}: {e}");
		let real = std::fs::canonicalize(folder).map_err(|e| fail(&e))?;//before the wall, since resolving a path reads every folder above it, and the kernel judges a read by the path it resolves to, /private/tmp for /tmp
		let links = links_followed(folder);
		let mut profile = PROFILE.to_string();
		let mut named = vec![(CString::new("FOLDER").map_err(|e| fail(&e))?, CString::new(real.as_os_str().as_bytes()).map_err(|e| fail(&e))?)];
		for (i, link) in links.iter().enumerate() {
			profile.push_str(&format!("(allow file-read* (literal (param \"LINK{i}\")))\n"));//the link alone, never what is under it
			named.push((CString::new(format!("LINK{i}")).map_err(|e| fail(&e))?, CString::new(link.as_os_str().as_bytes()).map_err(|e| fail(&e))?));
		}
		let profile = CString::new(profile).map_err(|e| fail(&e))?;
		let mut parameters: Vec<*const c_char> = named.iter().flat_map(|(name, value)| [name.as_ptr(), value.as_ptr()]).collect();//names and values in turn
		parameters.push(std::ptr::null());//ended by a null
		let mut error: *mut c_char = std::ptr::null_mut();
		if unsafe { sandbox_init_with_parameters(profile.as_ptr(), 0, parameters.as_ptr(), &mut error) } == 0 { return Ok(()) }
		let words = if error.is_null() { "no reason given".to_string() } else { let words = unsafe { CStr::from_ptr(error) }.to_string_lossy().into_owned(); unsafe { sandbox_free_error(error) }; words };
		Err(format!("sandbox: {}", words.trim()))
	}

	fn links_followed(folder: &str) -> Vec<std::path::PathBuf> {//every link the kernel reads to resolve the folder as it was named, /var for a folder in /var/folders, each a read the walls judge on its own; at the path the kernel reaches it by, left to right, so a link under /var is /private/var's, and a link's own target followed the same way
		use std::path::{Component, Path, PathBuf};
		let mut links = Vec::new();
		let mut current = PathBuf::from("/");
		let mut rest: Vec<PathBuf> = Vec::new();//the parts still to walk, the next one last
		let queue = |rest: &mut Vec<PathBuf>, path: &Path| rest.extend(path.components().rev().map(|part| PathBuf::from(part.as_os_str())));
		queue(&mut rest, &std::path::absolute(folder).unwrap_or_else(|_| PathBuf::from(folder)));
		while let Some(next) = rest.pop() {
			match next.components().next() {
				Some(Component::RootDir) => current = PathBuf::from("/"),
				Some(Component::ParentDir) => { current.pop(); }
				Some(Component::Normal(name)) => {
					let candidate = current.join(name);
					match std::fs::read_link(&candidate) {
						Ok(target) if links.len() < 40 => { links.push(candidate); queue(&mut rest, &target) }//a relative target resolves from the folder the link sits in, which current still is; past 40 links the kernel gives up as well
						_ => current = candidate,
					}
				}
				_ => {}
			}
		}
		links
	}
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
	#[cfg(target_os = "linux")]
	pub use super::linux::enter;
	#[cfg(not(target_os = "linux"))]
	pub fn enter(_folder: &str) -> Result<(), String> { Ok(()) }//no wall on this platform yet

	pub struct Child(std::process::Child);
	impl Child {
		pub fn kill(&mut self) -> std::io::Result<()> { self.0.kill() }
		pub fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> { self.0.wait() }
	}
	pub fn start(program: &std::path::Path, switch: &str, stdout: std::fs::File) -> std::io::Result<(Child, Box<dyn std::io::Write + Send>, Box<dyn std::io::Read + Send>)> {
		use std::process::Stdio;
		let mut command = std::process::Command::new(program);
		command.arg(switch).stdin(Stdio::piped()).stdout(stdout).stderr(Stdio::piped());
		#[cfg(target_os = "linux")] {
			use std::os::unix::process::CommandExt;
			command.env_clear().envs(std::env::var_os("LD_LIBRARY_PATH").map(|path| ("LD_LIBRARY_PATH", path)));//nothing from the main process's environment but where a development build finds the media library, which a package finds by its rpath
			unsafe { command.pre_exec(|| { libc::syscall(libc::SYS_close_range, 3u32, u32::MAX, libc::CLOSE_RANGE_CLOEXEC); Ok(()) }) };//every file but the three streams closes as it starts, as on the Mac; marked rather than closed, so the pipe Rust reports a failed start through still works
		}
		let mut child = command.spawn()?;
		let (Some(stdin), Some(stderr)) = (child.stdin.take(), child.stderr.take()) else { let _ = child.kill(); let _ = child.wait(); return Err(std::io::Error::other("no pipes")) };
		Ok((Child(child), Box::new(stdin), Box::new(stderr)))
	}
}

#[cfg(target_os = "windows")]
mod platform {
	use std::ffi::OsStr;
	use std::os::windows::ffi::OsStrExt;
	use std::os::windows::io::{AsRawHandle, FromRawHandle};
	use std::os::windows::process::ExitStatusExt;
	use windows::core::{PCWSTR, PWSTR};
	use windows::Win32::Foundation::{CloseHandle, SetHandleInformation, HANDLE, HANDLE_FLAGS, HANDLE_FLAG_INHERIT, WAIT_OBJECT_0};
	use windows::Win32::Security::{CreateWellKnownSid, GetLengthSid, SetTokenInformation, TokenIntegrityLevel, WinUntrustedLabelSid, PSID, SID_AND_ATTRIBUTES, TOKEN_ADJUST_DEFAULT, TOKEN_MANDATORY_LABEL, TOKEN_QUERY};
	use windows::Win32::System::Diagnostics::Debug::{SetErrorMode, SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SEM_NOOPENFILEERRORBOX};
	use windows::Win32::System::JobObjects::{AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicUIRestrictions, JobObjectExtendedLimitInformation, SetInformationJobObject, JOBOBJECT_BASIC_UI_RESTRICTIONS, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_UILIMIT_DESKTOP, JOB_OBJECT_UILIMIT_DISPLAYSETTINGS, JOB_OBJECT_UILIMIT_EXITWINDOWS, JOB_OBJECT_UILIMIT_GLOBALATOMS, JOB_OBJECT_UILIMIT_HANDLES, JOB_OBJECT_UILIMIT_READCLIPBOARD, JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS, JOB_OBJECT_UILIMIT_WRITECLIPBOARD};
	use windows::Win32::System::Pipes::CreatePipe;
	use windows::Win32::System::Threading::{CreateProcessW, DeleteProcThreadAttributeList, GetCurrentProcess, GetExitCodeProcess, InitializeProcThreadAttributeList, OpenProcessToken, TerminateProcess, UpdateProcThreadAttribute, WaitForSingleObject, CREATE_DEFAULT_ERROR_MODE, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, INFINITE, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, STARTF_USESTDHANDLES, STARTUPINFOEXW};

	pub const MEMORY_LIMIT: usize = 1 << 30;//1 GiB of committed memory, past which the job refuses the sandbox process any more; the inventory reads a file a megabyte at a time, and the probe opens a video's demuxer and decodes nothing
	const SE_GROUP_INTEGRITY: u32 = 0x20;//winnt.h's, which the windows crate keeps behind a feature this crate does not otherwise need

	pub struct Child { process: HANDLE }
	unsafe impl Send for Child {}//a process handle is good from any thread, and the main process kills from its watcher and waits from its loop
	impl Child {
		pub fn kill(&mut self) -> std::io::Result<()> { unsafe { TerminateProcess(self.process, 1) }.map_err(std::io::Error::other) }
		pub fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
			if unsafe { WaitForSingleObject(self.process, INFINITE) } != WAIT_OBJECT_0 { return Err(std::io::Error::last_os_error()) }
			let mut code = 0;
			unsafe { GetExitCodeProcess(self.process, &mut code) }.map_err(std::io::Error::other)?;
			Ok(std::process::ExitStatus::from_raw(code))
		}
	}
	impl Drop for Child { fn drop(&mut self) { unsafe { let _ = CloseHandle(self.process); } } }

	pub fn start(program: &std::path::Path, switch: &str, stdout: std::fs::File) -> std::io::Result<(Child, Box<dyn std::io::Write + Send>, Box<dyn std::io::Read + Send>)> {
		let (stdin_read, stdin_write) = pipe()?;
		let (stderr_read, stderr_write) = pipe()?;
		let program_wide = wide(program.as_os_str());
		let mut line = std::ffi::OsString::from("\"");//the command line as the new process reads it back, the program quoted since its path may hold a space
		line.push(program);
		line.push("\" ");
		line.push(switch);
		let mut line = wide(&line);//which CreateProcessW may write into
		let environment = environment_block();
		let handles = [HANDLE(stdin_read.as_raw_handle()), HANDLE(stdout.as_raw_handle()), HANDLE(stderr_write.as_raw_handle())];//the three the new process gets, and the only three
		let mut size = 0;
		let _ = unsafe { InitializeProcThreadAttributeList(None, 1, None, &mut size) };//asked first for the size, which it answers with an error by design
		let mut list_memory = vec![0u64; size.div_ceil(8)];
		let list = LPPROC_THREAD_ATTRIBUTE_LIST(list_memory.as_mut_ptr().cast());
		unsafe { InitializeProcThreadAttributeList(Some(list), 1, None, &mut size) }.map_err(std::io::Error::other)?;
		let started = unsafe { UpdateProcThreadAttribute(list, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize, Some(handles.as_ptr().cast()), std::mem::size_of_val(&handles), None, None) }.map_err(std::io::Error::other).and_then(|()| {
			for handle in handles { unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAG_INHERIT) }.map_err(std::io::Error::other)?; }//inheritable for the one call below and no longer after; a Command started on another thread in these microseconds would take them too, which nothing in fuji does during an inventory
			let mut info = STARTUPINFOEXW::default();
			info.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
			info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
			info.StartupInfo.hStdInput = handles[0];
			info.StartupInfo.hStdOutput = handles[1];
			info.StartupInfo.hStdError = handles[2];
			info.lpAttributeList = list;
			let mut made = PROCESS_INFORMATION::default();
			let started = unsafe { CreateProcessW(PCWSTR(program_wide.as_ptr()), Some(PWSTR(line.as_mut_ptr())), None, None, true, EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT | CREATE_DEFAULT_ERROR_MODE, Some(environment.as_ptr().cast()), PCWSTR::null(), &info.StartupInfo, &mut made) };//its error mode its own rather than this process's, so the quiet the sandbox process puts on itself is the only quiet it has
			for handle in handles { let _ = unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }; }
			started.map_err(std::io::Error::other)?;
			unsafe { let _ = CloseHandle(made.hThread); }
			Ok(made.hProcess)
		});
		unsafe { DeleteProcThreadAttributeList(list) };
		Ok((Child { process: started? }, Box::new(stdin_write), Box::new(stderr_read)))//the new process's ends of the pipes, and the file, close here as start returns, theirs now alone
	}

	fn pipe() -> std::io::Result<(std::fs::File, std::fs::File)> {//a pipe's two ends as files, neither inheritable, read and written as Rust reads and writes any file
		let (mut read, mut write) = (HANDLE(std::ptr::null_mut()), HANDLE(std::ptr::null_mut()));
		unsafe { CreatePipe(&mut read, &mut write, None, 0) }.map_err(std::io::Error::other)?;
		Ok(unsafe { (std::fs::File::from_raw_handle(read.0), std::fs::File::from_raw_handle(write.0)) })
	}

	fn wide(text: &OsStr) -> Vec<u16> { text.encode_wide().chain(std::iter::once(0)).collect() }//as Windows reads text, ended by a null

	fn environment_block() -> Vec<u16> {//the new process's whole environment: SystemRoot, without which the system's own libraries fail to load, and the temporary folder, so the check tries the real one; nothing else of this process's
		let mut block: Vec<u16> = ["SystemRoot", "TEMP", "TMP"].into_iter().filter_map(|name| std::env::var_os(name).map(|value| (name, value))).flat_map(|(name, value)| OsStr::new(name).encode_wide().chain(std::iter::once('=' as u16)).chain(value.encode_wide()).chain(std::iter::once(0)).collect::<Vec<u16>>()).collect();
		block.push(0);//ended by an empty name after the last
		if block.len() == 1 { block.push(0) }//a block with no name is two nulls
		block
	}

	pub fn enter(_folder: &str) -> Result<(), String> {//the folder is not a wall here: nothing on Windows limits a process's reads to one folder, so the sandbox process reads what the user can, and the check says so
		let fail = |what: &str, e: &dyn std::fmt::Display| format!("sandbox: {what}: {e}");
		unsafe { SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX) };//first, and on its own rather than through the job's flag: a crash of this process puts up no dialog and reaches no Windows Error Reporting, a fast fail's included
		let job = unsafe { CreateJobObjectW(None, PCWSTR::null()) }.map_err(|e| fail("job", &e))?;//unnamed, and its handle kept open for the life of the process, since closing the last handle of a job that kills on close would end it here
		let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
		limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_ACTIVE_PROCESS | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
		limits.BasicLimitInformation.ActiveProcessLimit = 1;//this process, so a program it starts fails to start
		limits.ProcessMemoryLimit = MEMORY_LIMIT;
		unsafe { SetInformationJobObject(job, JobObjectExtendedLimitInformation, (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(), std::mem::size_of_val(&limits) as u32) }.map_err(|e| fail("job limits", &e))?;
		let desktop = JOBOBJECT_BASIC_UI_RESTRICTIONS { UIRestrictionsClass: JOB_OBJECT_UILIMIT_DESKTOP | JOB_OBJECT_UILIMIT_DISPLAYSETTINGS | JOB_OBJECT_UILIMIT_EXITWINDOWS | JOB_OBJECT_UILIMIT_GLOBALATOMS | JOB_OBJECT_UILIMIT_HANDLES | JOB_OBJECT_UILIMIT_READCLIPBOARD | JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS | JOB_OBJECT_UILIMIT_WRITECLIPBOARD };//the eight Chromium's sandbox puts on its job at lockdown: no clipboard, no window of another process's, no global atoms, no display settings, no desktop made or switched to, no logging off
		unsafe { SetInformationJobObject(job, JobObjectBasicUIRestrictions, (&desktop as *const JOBOBJECT_BASIC_UI_RESTRICTIONS).cast(), std::mem::size_of_val(&desktop) as u32) }.map_err(|e| fail("job desktop", &e))?;
		unsafe { AssignProcessToJobObject(job, GetCurrentProcess()) }.map_err(|e| fail("job", &e))?;
		let mut token = HANDLE(std::ptr::null_mut());
		unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_ADJUST_DEFAULT, &mut token) }.map_err(|e| fail("token", &e))?;
		let mut sid = [0u32; 17];//SECURITY_MAX_SID_SIZE, 68 bytes
		let mut sid_size = std::mem::size_of_val(&sid) as u32;
		let label_sid = PSID(sid.as_mut_ptr().cast());
		let lowered = unsafe { CreateWellKnownSid(WinUntrustedLabelSid, None, Some(label_sid), &mut sid_size) }.and_then(|()| {
			let label = TOKEN_MANDATORY_LABEL { Label: SID_AND_ATTRIBUTES { Sid: label_sid, Attributes: SE_GROUP_INTEGRITY } };
			let length = std::mem::size_of::<TOKEN_MANDATORY_LABEL>() as u32 + unsafe { GetLengthSid(label_sid) };
			unsafe { SetTokenInformation(token, TokenIntegrityLevel, (&label as *const TOKEN_MANDATORY_LABEL).cast(), length) }//the token this process already runs on, lowered in place; what it had open stays open, and every open from here is judged at the new level
		});
		unsafe { let _ = CloseHandle(token); }
		lowered.map_err(|e| fail("integrity", &e))
	}
}

#[cfg(target_os = "linux")]
mod linux {
	use landlock::{Access, AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr, ABI};
	use seccompiler::{SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter, SeccompRule};

	pub fn enter(folder: &str) -> Result<(), String> {//on a process of one thread, since Landlock walls the thread that asks and the threads it starts after
		let fail = |e: &dyn std::fmt::Display| format!("sandbox: {e}");
		unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) };//no core, and no process of the user's attaches to this one
		let abi = ABI::V5;//the last that adds a right on files
		let opened = PathFd::new(folder).map_err(|e| fail(&format!("{folder}: {e}")))?;//the folder itself, so one that cannot be opened is an error rather than a rule left out
		Ruleset::default().handle_access(AccessFs::from_all(abi)).map_err(|e| fail(&e))?
			.create().map_err(|e| fail(&e))?
			.add_rule(PathBeneath::new(opened, AccessFs::from_read(abi))).map_err(|e| fail(&e))?
			.restrict_self().map_err(|e| fail(&e))?;//as much as the kernel offers, and nothing on a kernel without Landlock
		let others = || SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Ne, std::process::id() as u64).and_then(|aimed| SeccompRule::new(vec![aimed]));//the first argument any process but this one, which is how these calls name their target
		let mut rules = std::collections::BTreeMap::new();
		for call in [libc::SYS_socket, libc::SYS_execve, libc::SYS_execveat, libc::SYS_ptrace, libc::SYS_process_vm_readv, libc::SYS_process_vm_writev, libc::SYS_pidfd_open, libc::SYS_tkill] { rules.insert(call, vec![]); }//refused whatever their arguments
		for call in [libc::SYS_io_uring_setup, libc::SYS_io_uring_enter, libc::SYS_io_uring_register] { rules.insert(call, vec![]); }//and io_uring, which opens a socket inside the kernel where this filter never sees the call
		for call in [libc::SYS_kill, libc::SYS_tgkill, libc::SYS_rt_sigqueueinfo, libc::SYS_rt_tgsigqueueinfo] { rules.insert(call, vec![others().map_err(|e| fail(&e))?]); }//refused when aimed past this process
		let arch = std::env::consts::ARCH.try_into().map_err(|e| fail(&e))?;
		let filter = SeccompFilter::new(rules, SeccompAction::Allow, SeccompAction::Errno(libc::EPERM as u32), arch).map_err(|e| fail(&e))?;
		let program: seccompiler::BpfProgram = filter.try_into().map_err(|e: seccompiler::BackendError| fail(&e))?;
		seccompiler::apply_filter_all_threads(&program).map_err(|e| fail(&e))
	}
}

/// The check, from brass's run(): this process walled in to the folder in the first argument after the switch, then every reach tried, a line each to stdout saying what it answered and whether that was what the walls should answer; exit status 1 if any was not
pub fn check_serve() -> ! {
	let arguments: Vec<String> = std::env::args().skip(2).collect();
	let [folder, outside] = arguments.as_slice() else { eprintln!("usage: fuji {CHECK_SWITCH} <folder> <a file outside it>"); std::process::exit(2) };
	let listener = std::net::TcpListener::bind("127.0.0.1:0");//before the wall, so a connection inside it has somewhere on this machine to reach and none leaves it
	let inside = std::fs::read_dir(folder).ok().and_then(|entries| entries.flatten().map(|entry| entry.path()).find(|path| path.is_file()));//a file in the folder to read, found before the wall too
	if let Err(e) = enter(folder) { eprintln!("{e}"); std::process::exit(3) }
	let mut wrong = 0;
	let mut say = |allowed_expected: bool, what: &str, answer: Result<String, String>| {
		let allowed = answer.is_ok();
		if allowed != allowed_expected { wrong += 1 }
		let words = match &answer { Ok(words) | Err(words) => words.as_str() };
		println!("{} {} {what}{}{words}", if allowed == allowed_expected { "ok" } else { "NO" }, if allowed { "allowed" } else { "refused" }, if words.is_empty() { "" } else { ": " });
	};
	let text = |e: std::io::Error| e.to_string();
	say(true, "list the folder", std::fs::read_dir(folder).map(|entries| format!("{} names", entries.count())).map_err(text));
	say(true, "read a file in it", match &inside { Some(path) => std::fs::read(path).map(|bytes| format!("{} bytes of {}", bytes.len(), path.display())).map_err(text), None => Err("the folder holds no file".into()) });
	say(false, "read a file outside it", std::fs::read(outside).map(|bytes| format!("{} bytes of {outside}", bytes.len())).map_err(text));
	say(false, "list the folder above it", std::path::Path::new(folder).parent().map_or(Err("no folder above".into()), |above| std::fs::read_dir(above).map(|entries| format!("{} names", entries.count())).map_err(text)));
	let made = std::path::Path::new(folder).join("sandbox-check.txt");
	say(false, "write a file in it", std::fs::write(&made, "written").map(|()| { let _ = std::fs::remove_file(&made); made.display().to_string() }).map_err(text));
	let elsewhere = std::env::temp_dir().join("sandbox-check.txt");
	say(false, "write a file in the temporary folder", std::fs::write(&elsewhere, "written").map(|()| { let _ = std::fs::remove_file(&elsewhere); elsewhere.display().to_string() }).map_err(text));
	say(false, "connect to this machine", match &listener { Ok(listener) => listener.local_addr().and_then(std::net::TcpStream::connect).map(|stream| format!("{:?}", stream.peer_addr())).map_err(text), Err(e) => Err(format!("no listener to reach: {e}")) });
	let program = if cfg!(windows) { std::process::Command::new(std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into())).join("System32").join("cmd.exe")).args(["/c", "exit"]).status() } else { std::process::Command::new("/usr/bin/true").status() };//a program every machine of its kind has, by its whole path so a refusal is the walls' and not a missing file's or an empty PATH's
	say(false, "start a program", program.map(|status| status.to_string()).map_err(text));
	#[cfg(any(target_os = "macos", target_os = "linux"))] say(false, SERVICE, service_reach());//a platform with no service this asks prints no line, rather than a pass nothing earned
	#[cfg(unix)] say(false, "signal the process that started it", if unsafe { libc::kill(libc::getppid(), 0) } == 0 { Ok("it answered".into()) } else { Err(std::io::Error::last_os_error().to_string()) });//signal 0, which only asks whether the signal could be sent
	#[cfg(target_os = "linux")] say(false, "set up io_uring", io_uring_setup());
	#[cfg(target_os = "linux")] say(false, "leave a core for a crash reporter", if unsafe { libc::prctl(libc::PR_GET_DUMPABLE) } == 1 { Ok("dumpable".into()) } else { Err("not dumpable".into()) });
	#[cfg(windows)] say(false, "set a value under the user's registry", windows_check::registry_write());
	#[cfg(windows)] say(false, "end the process that started it", windows_check::parent_end());
	#[cfg(windows)] say(false, "read or write the memory of the process that started it", windows_check::parent_memory());
	#[cfg(windows)] say(false, "take more memory than the limit", windows_check::memory_past_limit());
	#[cfg(windows)] say(false, "report a crash to Windows Error Reporting", windows_check::crash_report());
	std::process::exit(if wrong > 0 { 1 } else { 0 })
}

#[cfg(target_os = "macos")]
const SERVICE: &str = "look up a service of the system's";
#[cfg(target_os = "macos")]
fn service_reach() -> Result<String, String> {//Launch Services, the service that opens a program or an address for whoever asks, found by name the way any program finds it; a process with no network could otherwise have it reach out. A record the system's libraries fetched as they loaded, before main, the user's own among them, stays in hand inside the walls, so a fresh lookup is what this asks
	extern "C" {
		static bootstrap_port: u32;//mach_port_t, this process's way to the system's directory of services
		fn bootstrap_look_up(bootstrap: u32, name: *const std::ffi::c_char, service: *mut u32) -> std::ffi::c_int;//0, or why not, 1100 for a name this process may not look up
	}
	let mut service = 0u32;
	match unsafe { bootstrap_look_up(bootstrap_port, c"com.apple.coreservices.launchservicesd".as_ptr(), &mut service) } {
		0 => Ok("found launchservicesd".into()),
		code => Err(format!("launchservicesd, refused with {code}")),
	}
}
#[cfg(target_os = "linux")]
const SERVICE: &str = "reach the desktop's message bus";
#[cfg(target_os = "linux")]
fn service_reach() -> Result<String, String> {//D-Bus, where a desktop's services answer whoever connects, at the address every session has
	let bus = format!("/run/user/{}/bus", unsafe { libc::getuid() });
	std::os::unix::net::UnixStream::connect(&bus).map(|_| format!("connected to {bus}")).map_err(|e| format!("{bus}: {e}"))
}

#[cfg(target_os = "linux")]
fn io_uring_setup() -> Result<String, String> {//a ring of one entry, which a process with io_uring can open a socket through out of any filter's sight
	let mut parameters = [0u8; 120];//struct io_uring_params, zeroed, as a first setup takes it
	let ring = unsafe { libc::syscall(libc::SYS_io_uring_setup, 1u32, parameters.as_mut_ptr()) };
	if ring < 0 { return Err(std::io::Error::last_os_error().to_string()) }
	unsafe { libc::close(ring as i32) };
	Ok("a ring".into())
}

#[cfg(windows)]
mod windows_check {//the reaches a process on Windows has past the folder, each tried for the check and no further: a key opened to set a value and closed, a process opened and closed, memory asked for and let go
	pub fn registry_write() -> Result<String, String> {//the user's own half of the registry, where a program of the user's keeps its settings and the system keeps what runs at logon
		use windows::core::w;
		use windows::Win32::System::Registry::{RegCloseKey, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE};
		let mut key = HKEY::default();
		unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, w!("Software"), None, KEY_SET_VALUE, &mut key) }.ok().map_err(|e| e.to_string())?;//opened for setting a value, which is where the walls judge it; nothing is set
		unsafe { let _ = RegCloseKey(key); }
		Ok("opened HKEY_CURRENT_USER\\Software to set a value".into())
	}
	pub fn parent_end() -> Result<String, String> { parent_open(windows::Win32::System::Threading::PROCESS_TERMINATE) }//for the right to end it, which is what the signal asks on unix
	pub fn parent_memory() -> Result<String, String> { use windows::Win32::System::Threading::{PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE}; parent_open(PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION) }//for the rights to read and write its memory
	fn parent_open(rights: windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS) -> Result<String, String> {//the process that started this one, opened for the rights given and closed; its id from this process's own basic information, since the process list that Toolhelp reads comes back empty at the Untrusted level
		use windows::Wdk::System::Threading::{NtQueryInformationProcess, ProcessBasicInformation};
		use windows::Win32::Foundation::CloseHandle;
		use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcess};
		#[repr(C)] #[derive(Default)] struct Basic { exit_status: i32, peb: usize, affinity: usize, priority: i32, process: usize, parent: usize }//PROCESS_BASIC_INFORMATION, 48 bytes, laid out here since the crate's needs a feature of the kernel's for the PEB it points at
		let mut basic = Basic::default();
		let mut returned = 0;
		unsafe { NtQueryInformationProcess(GetCurrentProcess(), ProcessBasicInformation, (&mut basic as *mut Basic).cast(), std::mem::size_of::<Basic>() as u32, &mut returned) }.ok().map_err(|e| format!("this process's own information: {e}"))?;
		let parent = basic.parent as u32;
		let opened = unsafe { OpenProcess(rights, false, parent) }.map_err(|e| e.to_string())?;
		unsafe { let _ = CloseHandle(opened); }
		Ok(format!("opened process {parent}"))
	}
	pub fn memory_past_limit() -> Result<String, String> {//twice the job's limit asked for and let go, committed by the heap and never touched
		let asked = 2 * super::platform::MEMORY_LIMIT;
		let mut memory: Vec<u8> = Vec::new();
		memory.try_reserve_exact(asked).map(|()| format!("{} MiB committed", asked >> 20)).map_err(|e| e.to_string())
	}
	pub fn crash_report() -> Result<String, String> {//whether a crash of this process would go to Windows Error Reporting, read from the error mode, where the system looks before it reports or asks
		use windows::Win32::System::Diagnostics::Debug::{GetErrorMode, SEM_NOGPFAULTERRORBOX};
		let mode = unsafe { GetErrorMode() };
		if mode & SEM_NOGPFAULTERRORBOX.0 == 0 { Ok(format!("error mode {mode:#x} lets it")) } else { Err(format!("error mode {mode:#x} keeps it")) }
	}
}

#[cfg(test)]
mod tests {
	#[cfg(target_os = "macos")]
	use super::*;

	#[test]
	#[cfg(target_os = "macos")]
	fn a_folder_that_is_not_there_is_refused_before_any_wall_goes_up() {//the sandbox process then exits rather than walling itself in to nothing; the walls themselves are tried by fuji --sandbox-check, since a test that put them on would wall in every test running beside it
		let refused = enter("/no/such/folder/anywhere").unwrap_err();
		assert!(refused.starts_with("sandbox: /no/such/folder/anywhere: "), "{refused}");
	}
}
