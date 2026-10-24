//the scout's command: the tip of one file by its path, and the second slice when the tip asks for one, read in fuji's own process and handed to the scout, fuji's own reader of a file's first bytes in safe rust, in steel, the safe crate at the repository root, where its essay says what it answers and what it promises. The reading is crate-visible, because inventory.rs reads every file of a folder the same way

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// What the first bytes of this file say: its format and kind, the stored size of a picture or a video and the size to show it at, the running time of a sound or a video, whether the file is corrupted or chopped, and the rest of the fields scout_bytes answers, with the scout's report on its own reading beside them. The tip is the first 64 KiB, and when the tip says where the rest of its answer lies, one more 64 KiB slice at the offset it names, which is the last read, never a third. An error for a path that is not a regular file, since opening a FIFO waits for a writer, and for a file that cannot be opened or read
#[tauri::command]
pub async fn scout_tip(path: String) -> Result<steel::tip::Tip, String> {
	crate::run_blocking(move || {
		let meta = std::fs::metadata(&path).map_err(|e| format!("scout: {path}: {e}"))?;//follows a link to what it names, since a link to a picture is a picture, and the kind refused below is the target's
		if !meta.is_file() { return Err(format!("scout: not a regular file: {path}")) }
		let mut file = open_file(&path).map_err(|e| format!("scout: {path}: {e}"))?;
		tip_read(&mut file, meta.len()).map_err(|e| format!("scout: {path}: {e}"))
	}).await
}

pub(crate) fn tip_read(file: &mut std::fs::File, size: u64) -> std::io::Result<steel::tip::Tip> {//what the reader says of a file just opened: the tip from its start, and the reader over both when it asks for the second slice, which is the same read again at the offset that came back beside the answer, and the last read
	let mut tip = vec![0u8; steel::tip::TIP_BYTES];//the allocations, this and the slice's below, are sized by the crate's constant and never by anything in the file
	let got = read_into(file, &mut tip)?;
	tip.truncate(got);
	let (mut t, wish) = steel::tip::scout_bytes(&tip, size);
	if let Some(at) = wish {
		file.seek(SeekFrom::Start(at))?;
		let mut slice = vec![0u8; steel::tip::TIP_BYTES];
		let got = read_into(file, &mut slice)?;
		slice.truncate(got);
		t = steel::tip::scout_slices(&tip, &slice, at, size);
	}
	Ok(t)
}
pub(crate) fn open_file(path: impl AsRef<Path>) -> std::io::Result<std::fs::File> {//the file for reading, opened without waiting on the mac and linux: an open of a FIFO otherwise blocks until something writes to it, and the kind check above can be raced. A read of a regular file still waits on the disk, which is the containment the scan process is for
	let mut options = std::fs::OpenOptions::new();
	options.read(true);
	#[cfg(unix)]
	{
		use std::os::unix::fs::OpenOptionsExt;
		options.custom_flags(libc::O_NONBLOCK);
	}
	options.open(path)
}
fn read_into(file: &mut std::fs::File, buffer: &mut [u8]) -> std::io::Result<usize> {//fill a buffer from where the file stands, in as many reads as it takes, answering how many bytes arrived; fewer than the buffer where the file ends first
	let mut got = 0;
	while got < buffer.len() {
		let Some(rest) = buffer.get_mut(got..) else { break };
		let n = file.read(rest)?;
		if n == 0 { break }
		got = got.saturating_add(n);
	}
	Ok(got)
}
