//how the user's system writes numbers, asked of each operating system its own way; brass's locale.rs holds the command over this. The page's own Intl formats from a locale tag and the standard data for it, and never sees a symbol the user chose in the system's settings, so it can write a number differently from Explorer and Finder

#[cfg(target_os = "windows")]
mod platform {
	use windows::core::PCWSTR;
	use windows::Win32::Globalization::{GetLocaleInfoEx, LOCALE_STHOUSAND};

	pub fn grouping() -> String {//the user's default locale, which a null name asks for, with whatever symbol the user typed over its own on the Region page
		let mut buffer = [0u16; 8];//room past the four characters, its null included, that windows allows this value
		let length = unsafe { GetLocaleInfoEx(PCWSTR::null(), LOCALE_STHOUSAND, Some(&mut buffer)) };//the characters written with the null after them, or 0 when windows will not say
		if length <= 0 { return String::new() }
		String::from_utf16_lossy(&buffer[..length as usize - 1])
	}
}

#[cfg(target_os = "macos")]
mod platform {
	use core_foundation::base::TCFType;
	use core_foundation::string::{CFString, CFStringRef};
	use core_foundation_sys::base::CFRelease;
	use core_foundation_sys::locale::{CFLocaleCopyCurrent, CFLocaleGetValue, kCFLocaleGroupingSeparator};

	pub fn grouping() -> String {//the current locale, which carries the user's choices in Language & Region, the number format among them
		unsafe {
			let locale = CFLocaleCopyCurrent();//Copy in the name, so ours to release below
			if locale.is_null() { return String::new() }
			let value = CFLocaleGetValue(locale, kCFLocaleGroupingSeparator);//Get in the name, so only lent; wrapping it under the get rule retains it for as long as the CFString lives
			let separator = if value.is_null() { String::new() } else { CFString::wrap_under_get_rule(value as CFStringRef).to_string() };
			CFRelease(locale as _);
			separator
		}
	}
}

#[cfg(target_os = "linux")]
mod platform {
	use std::ffi::CStr;

	pub fn grouping() -> String {//the numeric locale the environment names, as the desktop set it from its region, read through a locale object of its own: setlocale would change the whole process's, which media.rs keeps at C for mpv
		unsafe {
			let locale = libc::newlocale(libc::LC_NUMERIC_MASK, c"".as_ptr(), std::ptr::null_mut());//the empty name means the environment's, as setlocale takes it
			if locale.is_null() { return String::new() }//the environment names a locale this system does not have installed
			let separator = CStr::from_ptr(libc::nl_langinfo_l(libc::THOUSEP, locale)).to_string_lossy().into_owned();//copied, since the text belongs to the locale object freed below; blank in the C locale, which groups nothing
			libc::freelocale(locale);
			separator
		}
	}
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod platform {
	pub fn grouping() -> String { String::new() }
}

/// The separator the user's system puts between groups of digits in a number, like the comma in 16,444,365, with the user's own choice in the system's settings; blank when the system will not say
pub fn grouping() -> String { platform::grouping() }

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn the_system_says_how_it_groups_digits() {
		let separator = grouping();
		assert!(separator.chars().count() <= 3, "{separator:?}");//a comma, a period, an apostrophe or a space of some width, and never a word
		#[cfg(any(target_os = "windows", target_os = "macos"))]
		assert!(!separator.is_empty(), "every locale on these two has a separator");
	}
}
