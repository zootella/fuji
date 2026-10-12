//tauri_build embeds tauri.conf.json in the binary and writes gen/schemas
//it checks every capability permission against the plugins present, so a typo fails the build
//on windows it compiles the icon and manifest too, which is why a --target x86_64-pc-windows-msvc check dies here
fn main() {//cargo runs this once before the crate compiles, never at runtime
	if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") { println!("cargo:rustc-link-search=native={}/target/gathered", std::env::var("CARGO_MANIFEST_DIR").unwrap()) }//the libraries frameworks.js gathered, which mpv.rs links against so the binary names each by where it will sit inside the app; the package scripts gather before every build, and a cargo run on its own fails to link until one has
	tauri_build::build()
}
