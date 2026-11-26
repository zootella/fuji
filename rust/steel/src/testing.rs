//what steel's tests share: a folder of one test's own under the system's temporary folder, filled with small files and folders and removed when the test ends. The name is what keeps tests apart, since the tests of one binary run as threads of one process and share its id, so a name a second test takes fails that test here rather than letting the two meet in one folder
#![allow(clippy::expect_used, clippy::panic)]//a helper for tests may expect and assert: a mistake in it fails the test, which is what a test is for

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Tree(pub PathBuf);
impl Tree {
	pub fn new(name: &str) -> Tree {
		static NAMES: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
		assert!(NAMES.lock().expect("the lock on the names").insert(name.to_string()), "two tests named their tree {name}");
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
