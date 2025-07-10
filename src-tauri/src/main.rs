//./src-tauri/src/main.rs

/*
this is the single entrypoint of our Rust binary
on Windows, the line below supresses a console window which would otherwise pop up
all it does is call lib.rs's run() function
*/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
	fuji_lib::run()
}
