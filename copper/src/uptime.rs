//how long this process has been running, by the operating system's own record of when it started the process; brass's uptime.rs adds the moment main began and answers the page

use std::time::SystemTime;

/*
How long this process has been running, from two clocks. One is the operating system's record of when it started the process, which is before the loader mapped a single library, so a library added to the build shows its cost here before main ever runs. The other is main's own, the moment run() began, which window.rs keeps for its own reasons. The page reads both at its first frame, and the difference between them is what the loader spent, otherwise the one part of a launch nothing in the program can time.

Each platform keeps the start time somewhere of its own: the Mac in the process's bsd info through libproc, Windows in the creation time GetProcessTimes answers, and Linux in the process's stat line as clock ticks since boot, read against the uptime the kernel lists beside it.
*/

#[cfg(target_os = "macos")]
mod platform {
	use std::time::{Duration, SystemTime, UNIX_EPOCH};

	pub fn started() -> Option<SystemTime> {//the process's bsd info, whose start time is the kernel's own record
		let mut info: libc::proc_taskallinfo = unsafe { std::mem::zeroed() };
		let size = std::mem::size_of::<libc::proc_taskallinfo>() as libc::c_int;
		let got = unsafe { libc::proc_pidinfo(std::process::id() as libc::c_int, libc::PROC_PIDTASKALLINFO, 0, &mut info as *mut libc::proc_taskallinfo as *mut libc::c_void, size) };
		if got != size { return None }//libproc answers with the bytes it filled, and anything short is a refusal
		Some(UNIX_EPOCH + Duration::new(info.pbsd.pbi_start_tvsec, info.pbsd.pbi_start_tvusec as u32 * 1000))
	}
}

#[cfg(target_os = "windows")]
mod platform {
	use std::time::{Duration, SystemTime, UNIX_EPOCH};
	use windows::Win32::Foundation::FILETIME;
	use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

	pub fn started() -> Option<SystemTime> {//the creation time windows keeps for every process, in hundred-nanosecond ticks since 1601
		let (mut creation, mut exit, mut kernel, mut user) = (FILETIME::default(), FILETIME::default(), FILETIME::default(), FILETIME::default());
		unsafe { GetProcessTimes(GetCurrentProcess(), &mut creation, &mut exit, &mut kernel, &mut user).ok()? };
		let ticks = (creation.dwHighDateTime as u64) << 32 | creation.dwLowDateTime as u64;
		let since1601 = Duration::new(ticks / 10_000_000, (ticks % 10_000_000 * 100) as u32);//seconds and nanoseconds apart, since the ticks as nanoseconds would not fit a u64
		Some(UNIX_EPOCH + since1601 - Duration::from_secs(11_644_473_600))//the 369 years from windows' epoch to unix's
	}
}

#[cfg(target_os = "linux")]
mod platform {
	use std::time::{Duration, SystemTime};

	pub fn started() -> Option<SystemTime> {//the process's stat line gives its start as clock ticks since boot, and uptime says how long ago boot was
		let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
		let after = stat.rsplit(')').next()?;//the fields after the command name, which sits in parentheses and may hold a space itself
		let ticks: u64 = after.split_whitespace().nth(19)?.parse().ok()?;//starttime, field 22 of the line, with the two before the name and the name itself already gone
		let uptime: f64 = std::fs::read_to_string("/proc/uptime").ok()?.split_whitespace().next()?.parse().ok()?;
		let boot = SystemTime::now().checked_sub(Duration::from_secs_f64(uptime))?;
		Some(boot + Duration::from_millis(ticks * 10))//a tick in proc is a hundredth of a second on every linux, whatever the kernel's own rate: USER_HZ, fixed so the numbers it lists never change meaning
	}
}

/// When the operating system started this process, from its own record, which is before the loader mapped a single library; none when it will not say
pub fn started() -> Option<SystemTime> { platform::started() }

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::Duration;
	#[test]
	fn the_process_started_before_now_and_not_long_ago() {
		let started = started().expect("the operating system's record of when this process started");
		let ago = SystemTime::now().duration_since(started).expect("a start before now");
		assert!(ago < Duration::from_secs(3600), "{ago:?} ago");
	}
}
