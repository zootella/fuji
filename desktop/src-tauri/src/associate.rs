use tauri::command;

/*
What fuji has told the operating system it can open. One command, called once at startup by the page, which hands down every extension in imageTypes with the name it should carry. Windows writes them into the registry; macOS and Linux do nothing here at all.

macOS needs nothing because its declaration is not code. CFBundleDocumentTypes sits in Info.plist inside the .app, LaunchServices reads it when it first sees the bundle, and dragging Fuji.app into Applications is the whole of the registration. There is no runtime call and nothing this module could add.

Windows has no such file, so an application registers itself. Two things have to be said clearly about how, because the subject is thick with folklore. The first is what fuji writes: a ProgID per extension naming the type and the icon and the command that opens it, that ProgID added to the extension's OpenWithProgids list, the executable's own key with the extensions it supports, and a Capabilities block registered so the Settings app lists fuji by name with its types. The second is what fuji does not write: the extension's own default value, the single line that says .webp means fuji from now on. That line is the one an installer from 1999 would write, it is the one Tauri's bundled NSIS macro still writes, and it is the one Microsoft's own current API — ActivationRegistrationManager, whose implementation is open — deliberately does not. Everything here makes fuji available. Nothing here makes fuji default, because since Windows 8 the default lives in a hash-protected key that only the user, through the system's own interface, can set.

So after this runs, fuji is in Explorer's Open with menu and listed in Settings under Default apps, and every file on the machine still opens with whatever opened it before. Windows offers the user the new choice the next time they open one of those types. associations.md carries the reasoning, the alternatives, and what each key is for.

Two practical notes. It runs on every launch, which is cheap because each value is read before it is written and an unchanged value is not touched; the shell is only notified if something actually moved. And it does nothing in a debug build, because the command paths come from current_exe() and a debug build's executable lives in target/debug, where it will be replaced and eventually deleted — registering it would leave the user's registry pointing at a moving target.
*/

/// One file type fuji tells the operating system it can open
#[derive(serde::Deserialize)]
#[allow(dead_code)]//these three are read only by the windows release path below, so every other build is right that nothing touches them
pub struct AssociateType {
	pub extension: String,//with its leading dot, like ".webp"
	pub program: String,//the progid naming this type, like "Fuji.webp"
	pub name: String,//what explorer prints in its type column, like "WebP Image"
}

/// Tell the operating system fuji can open these types, claiming none of them; answers a line for the log
#[command]
pub fn associate_register(types: Vec<AssociateType>) -> Result<String, String> {
	#[cfg(all(target_os = "windows", not(debug_assertions)))]
	return windows_register(&types);

	#[cfg(not(all(target_os = "windows", not(debug_assertions))))]
	{
		let _ = types;//there is nothing to do with them here; spelling the parameter _types instead would hide a genuinely unused one on windows
		Ok(String::new())//blank, so the shell logs nothing: a mac launch that was never going to register anything is not news
	}
}

#[cfg(all(target_os = "windows", not(debug_assertions)))]
fn windows_register(types: &[AssociateType]) -> Result<String, String> {
	let executable = std::env::current_exe().map_err(|e| format!("associate: {e}"))?;
	let file = executable.file_name().and_then(|n| n.to_str()).ok_or("associate: fuji's own file name is not utf-8")?.to_string();//"fuji.exe", which is the key windows expects under Applications
	let path = executable.to_str().ok_or("associate: the path to fuji is not utf-8")?.to_string();
	let command = format!("\"{path}\" \"%1\"");//quoted, because a picture's path will contain spaces
	let icon = format!("{path},0");//the first icon in the executable, which is fuji's own; a document icon of fuji's own is not built yet, and associations.md says why that matters

	let application = format!("Software\\Classes\\Applications\\{file}");
	let mut changed = 0;
	for t in types {
		changed += windows_set(&format!("Software\\Classes\\{}", t.program), "", &t.name)?;//the progid: what this kind of file is called
		changed += windows_set(&format!("Software\\Classes\\{}\\DefaultIcon", t.program), "", &icon)?;//what explorer draws on one
		changed += windows_set(&format!("Software\\Classes\\{}\\shell\\open\\command", t.program), "", &command)?;//and what opens it
		changed += windows_set(&format!("Software\\Classes\\{}\\OpenWithProgids", t.extension), &t.program, "")?;//fuji joins the list of what could open this extension, which is the offer; the value is empty and only the name matters
		changed += windows_set(&format!("{application}\\SupportedTypes"), &t.extension, "")?;//so fuji is offered for these and not for everything else
		changed += windows_set("Software\\Fuji\\Capabilities\\FileAssociations", &t.extension, &t.program)?;//and so the settings app can list fuji's types
	}
	changed += windows_set(&application, "FriendlyAppName", "Fuji")?;
	changed += windows_set(&format!("{application}\\shell\\open\\command"), "", &command)?;
	changed += windows_set("Software\\Fuji\\Capabilities", "ApplicationName", "Fuji")?;
	changed += windows_set("Software\\Fuji\\Capabilities", "ApplicationDescription", "A multimedia file manager designed with privacy and precision in mind")?;
	changed += windows_set("Software\\RegisteredApplications", "Fuji", "Software\\Fuji\\Capabilities")?;//the line that puts fuji in the settings app by name, and last on purpose: any write above can fail and take the whole call with it, so publishing fuji to Settings is the step that only happens once everything it points at is there. The next launch starts again from the top and finishes the job

	if changed > 0 { windows_notify() }//only when something moved, because this runs on every launch and almost always writes nothing
	Ok(format!("associate: {} types registered, {changed} values written", types.len()))
}

//create a key under HKEY_CURRENT_USER if it is not there and set one of its values, answering 1 if that changed anything and 0 if the value already said this
//a blank name means the key's own default value, which is how the registry spells "the value of this key itself"
#[cfg(all(target_os = "windows", not(debug_assertions)))]
fn windows_set(path: &str, name: &str, value: &str) -> Result<u32, String> {
	use windows::core::PCWSTR;
	use windows::Win32::System::Registry::{RegCloseKey, RegCreateKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ};

	let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();//utf-16 with the terminating zero windows wants; bound to a variable at every call site below, because a pointer into a temporary would dangle
	let wide_path = wide(path);
	let wide_name = wide(name);
	let wide_value = wide(value);

	unsafe {
		let mut key = HKEY::default();
		let opened = RegCreateKeyExW(HKEY_CURRENT_USER, PCWSTR(wide_path.as_ptr()), None, PCWSTR::null(), REG_OPTION_NON_VOLATILE, KEY_QUERY_VALUE | KEY_SET_VALUE, None, &mut key, None);
		if opened.0 != 0 { return Err(format!("associate: could not open {path}, windows error {}", opened.0)) }

		let bytes = wide_value.iter().flat_map(|u| u.to_le_bytes()).collect::<Vec<u8>>();//REG_SZ is utf-16 little endian including its terminating zero, handed to windows as bytes

		//read what is there now; a value that already says this is left alone, so a launch that changed nothing does not go on to tell the shell that something did
		let mut buffer = [0u8; 2048];
		let mut size = buffer.len() as u32;
		let read = RegQueryValueExW(key, PCWSTR(wide_name.as_ptr()), None, None, Some(buffer.as_mut_ptr()), Some(&mut size));
		let same = read.0 == 0 && size as usize == bytes.len() && buffer[..size as usize] == bytes[..];//the length test sits before the slice on purpose, and is load bearing: it is what holds size down to something short before it is used as an index, so a longer value already in the registry cannot run this past the end of the buffer. Only the bytes are compared and not the type, so a value some other program wrote as REG_NONE is rewritten once and matches from then on

		let mut answer = Ok(0);
		if !same {
			let written = RegSetValueExW(key, PCWSTR(wide_name.as_ptr()), None, REG_SZ, Some(&bytes));
			answer = if written.0 == 0 { Ok(1) } else { Err(format!("associate: could not write {path}, windows error {}", written.0)) };
		}
		let _ = RegCloseKey(key);
		answer
	}
}

//tell the shell that associations have changed, so explorer's menus and icons catch up without a sign-out
#[cfg(all(target_os = "windows", not(debug_assertions)))]
fn windows_notify() {
	use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
	unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) }
}
