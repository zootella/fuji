//tauri_build embeds tauri.conf.json in the binary and writes gen/schemas
//it checks every capability permission against the plugins present, so a typo fails the build
//on windows it compiles the icon and manifest too, which is why a --target x86_64-pc-windows-msvc check dies here
fn product_name() -> String {//the productName in tauri.conf.json, which names the resources folder the bundlers make; read by its key rather than parsed, since this script has no json library and the file is fuji's own
	let text = std::fs::read_to_string("tauri.conf.json").expect("tauri.conf.json beside Cargo.toml");
	let after = text.split("\"productName\"").nth(1).expect("productName in tauri.conf.json");
	after.split('"').nth(1).expect("a quoted productName").to_string()
}
fn main() {//cargo runs this once before the crate compiles, never at runtime
	if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {//the same library as libmpv.so.2, which the deb and the rpm install as a resource beside the application's other files, under usr/lib and the product's name, and the flatpak lays out the same way under /app; the rpath is relative to the binary so both find it
		println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/{}", product_name());
	}
	tauri_build::build()
}
