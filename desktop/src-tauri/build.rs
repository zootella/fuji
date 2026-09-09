//tauri_build embeds tauri.conf.json in the binary and writes gen/schemas
//it checks every capability permission against the plugins present, so a typo fails the build
//on windows it compiles the icon and manifest too, which is why a --target x86_64-pc-windows-msvc check dies here
fn main() {//cargo runs this once before the crate compiles, never at runtime
	tauri_build::build()
}
