//what steel's tests share: a folder of one test's own under the system's temporary folder, filled with small files and folders and removed when the test ends, so tests run side by side without meeting. The walker's tests keep a tree of their own, with a listing method the others have no use for
#![allow(clippy::expect_used)]//a helper for tests may expect: a mistake in it fails the test, which is what a test is for

use std::fs;
use std::path::{Path, PathBuf};

pub struct Tree(pub PathBuf);
impl Tree {
	pub fn new(name: &str) -> Tree {
		let path = std::env::temp_dir().join(format!("steel-{name}-{}", std::process::id()));
		let _ = fs::remove_dir_all(&path);
		fs::create_dir_all(&path).expect("a folder under the temporary folder");
		Tree(path)
	}
	pub fn path(&self, relative: &str) -> PathBuf { self.0.join(relative) }
	pub fn text(&self, relative: &str) -> String { self.path(relative).to_string_lossy().replace('\\', "/") }//the path as a command takes it, with forward slashes
	pub fn file(&self, relative: &str) {//a small file whose contents are its own name, so its size is the name's length
		let path = self.path(relative);
		fs::create_dir_all(path.parent().expect("a parent")).expect("the parent made");
		fs::write(path, relative).expect("the file written");
	}
	pub fn folder(&self, relative: &str) { fs::create_dir_all(self.path(relative)).expect("the folder made") }
	pub fn relative(&self, path: &Path) -> String { path.strip_prefix(&self.0).expect("a path under the tree").to_string_lossy().replace('\\', "/") }
}
impl Drop for Tree { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
