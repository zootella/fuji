/*
A thin main.rs over a lib.rs holding the whole application. That is Tauri's shape rather than fuji's taste: mobile has no main() to be the start of anything, since an Android or iOS shell links the application in as a library and calls into it, so keeping everything in the library is what makes that possible later without moving code. On the desktop it costs one file that does nothing but call across.

The attribute below is conditional on not being a debug build, which is worth remembering next to the eprintln! in desktop.rs: a debug build keeps its console, and a shipped Windows build is the one where that line has nowhere to print.
*/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]//release only: no console window behind the app; #! applies to the crate

fn main() {
	fuji_lib::run()//everything is over there; Cargo.toml says why the library is fuji_lib rather than fuji
}
