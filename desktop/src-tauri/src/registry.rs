use tauri::command;

/*
The Windows registry, offered to the page the way disk.rs offers the disk: general commands any Windows application could use as they are, knowing nothing about which keys are written or why. Which keys, which values, and in what order is the page's; associate.js is the one caller today, and it is where the whole policy of offering file types without claiming them lives.

Three commands, all under HKEY_CURRENT_USER. registry_get reads one string value, answering blank when the key or the value is not there, so a value that is present and empty reads the same as a missing one; no caller needs to tell those apart, and the registry holds plenty of empty values, OpenWithProgids among them, whose names are the whole of what they say. registry_set writes one, creating the key and any missing parents; it reads first and writes only when the value would change, and answers whether it did, so a caller that runs on every launch can tell the shell only when something actually moved. registry_notify is that telling: it announces that file associations changed, so Explorer's menus and icons catch up without a sign-out.

A blank value name means the key's own default value, which is how the registry spells "the value of this key itself". The commands take any key under the current user and hold no guard, the same as disk.rs, and they never touch the machine-wide hive. On macOS and Linux there is no registry, and each command answers so.
*/

/// Read a string value under the current user; answers blank when the key or the value is not there
#[command]
pub fn registry_get(key: String, name: String) -> Result<String, String> {
	platform::get(&key, &name)
}

/// Write a string value under the current user, creating the key if it is missing, and only if it would change; answers whether it changed
#[command]
pub fn registry_set(key: String, name: String, value: String) -> Result<bool, String> {
	platform::set(&key, &name, &value)
}

/// Tell the shell that file associations changed
#[command]
pub fn registry_notify() -> Result<(), String> {
	platform::notify()
}

#[cfg(target_os = "windows")]
mod platform {
	use windows::core::PCWSTR;
	use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
	use windows::Win32::System::Registry::{RegCloseKey, RegCreateKeyExW, RegGetValueW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RRF_NOEXPAND, RRF_RT_REG_EXPAND_SZ, RRF_RT_REG_SZ};
	use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};

	//utf-16 with the terminating zero windows wants; bind the result to a variable before handing windows a pointer into it, because a pointer into a temporary would dangle
	fn wide(s: &str) -> Vec<u16> {
		s.encode_utf16().chain(std::iter::once(0)).collect()
	}

	pub fn get(path: &str, name: &str) -> Result<String, String> {
		let wide_path = wide(path);
		let wide_name = wide(name);
		let mut buffer = [0u16; 2048];//longer than any value fuji reads; a longer one is reported rather than cut short
		let mut size = std::mem::size_of_val(&buffer) as u32;//the registry counts in bytes
		let named = if name.is_empty() { PCWSTR::null() } else { PCWSTR(wide_name.as_ptr()) };//a null name is the key's default value
		let read = unsafe { RegGetValueW(HKEY_CURRENT_USER, PCWSTR(wide_path.as_ptr()), named, RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ | RRF_NOEXPAND, None, Some(buffer.as_mut_ptr().cast()), Some(&mut size)) };//a string, exactly as written; unlike RegQueryValueExW, this always ends one with its terminating zero
		if read == ERROR_FILE_NOT_FOUND { return Ok(String::new()) }//no key, or a key without this value
		if read.0 != 0 { return Err(format!("registry: could not read {path}, windows error {}", read.0)) }
		Ok(String::from_utf16_lossy(&buffer[..(size as usize / 2).saturating_sub(1)]))//characters, less the terminating zero; on success size never exceeds the buffer
	}

	pub fn set(path: &str, name: &str, value: &str) -> Result<bool, String> {
		let wide_path = wide(path);
		let wide_name = wide(name);
		let wide_value = wide(value);

		unsafe {
			let mut key = HKEY::default();
			let opened = RegCreateKeyExW(HKEY_CURRENT_USER, PCWSTR(wide_path.as_ptr()), None, PCWSTR::null(), REG_OPTION_NON_VOLATILE, KEY_QUERY_VALUE | KEY_SET_VALUE, None, &mut key, None);
			if opened.0 != 0 { return Err(format!("registry: could not open {path}, windows error {}", opened.0)) }

			let bytes = wide_value.iter().flat_map(|u| u.to_le_bytes()).collect::<Vec<u8>>();//REG_SZ is utf-16 little endian including its terminating zero, handed to windows as bytes

			//read what is there now; a value that already says this is left alone, so a launch that changed nothing does not go on to tell the shell that something did
			let mut buffer = [0u8; 2048];
			let mut size = buffer.len() as u32;
			let read = RegQueryValueExW(key, PCWSTR(wide_name.as_ptr()), None, None, Some(buffer.as_mut_ptr()), Some(&mut size));
			let same = read.0 == 0 && size as usize == bytes.len() && buffer[..size as usize] == bytes[..];//the length test sits before the slice on purpose, and is load bearing: it is what holds size down to something short before it is used as an index, so a longer value already in the registry cannot run this past the end of the buffer. Only the bytes are compared and not the type, so a value some other program wrote as REG_NONE is rewritten once and matches from then on

			let mut answer = Ok(false);
			if !same {
				let written = RegSetValueExW(key, PCWSTR(wide_name.as_ptr()), None, REG_SZ, Some(&bytes));
				answer = if written.0 == 0 { Ok(true) } else { Err(format!("registry: could not write {path}, windows error {}", written.0)) };
			}
			let _ = RegCloseKey(key);
			answer
		}
	}

	pub fn notify() -> Result<(), String> {
		unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) }
		Ok(())
	}
}

#[cfg(not(target_os = "windows"))]
mod platform {
	const NONE: &str = "registry: there is no registry on this platform";
	pub fn get(_path: &str, _name: &str) -> Result<String, String> { Err(NONE.to_string()) }
	pub fn set(_path: &str, _name: &str, _value: &str) -> Result<bool, String> { Err(NONE.to_string()) }
	pub fn notify() -> Result<(), String> { Err(NONE.to_string()) }
}
