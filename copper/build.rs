//the media library's folder for the linker on the mac and linux, where pnpm library leaves libmpv beside brass's sources; windows links the dll by name through the raw-dylib attribute on the declarations and needs no path. The rpath that finds the library when fuji launches stays in brass's build.rs, because a link argument from a dependency's build script never reaches the final binary; the one here reaches copper's own test binary and nothing else, so cargo test in this folder finds the library too; cargo 1.99 refuses the tests-only form of the instruction
fn main() {
	let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
	if os == "macos" || os == "linux" {
		let binaries = format!("{}/../desktop/src-tauri/binaries", std::env::var("CARGO_MANIFEST_DIR").unwrap());
		println!("cargo::rustc-link-search=native={binaries}");
		println!("cargo::rustc-link-arg=-Wl,-rpath,{binaries}");
	}
	println!("cargo::rerun-if-changed=build.rs");
}
