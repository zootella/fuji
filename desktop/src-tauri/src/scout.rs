//the scout's command: the tip of one file by its path, and the second slice when the tip asks for one, read in fuji's own process and handed to the scout, fuji's own reader of a file's first bytes in safe rust, a crate of its own in the scout folder at the repository root, where its essay says what it answers and what it promises

/// What the first bytes of this file say: its format and medium, the size a picture or a video shows at, the running time of a sound or a video, and the rest of the fields scout_bytes answers. The tip is the first 64 KiB, and when the tip says where the rest of its answer lies, one more 64 KiB slice at the offset it names, which is the last read, never a third. An error for a path that is not a regular file, since opening a FIFO waits for a writer, and for a file that cannot be opened or read
#[tauri::command]
pub async fn scout_tip(path: String) -> Result<::scout::Tip, String> {//::scout is the crate, where a bare scout could be read as this module
	crate::run_blocking(move || {
		let meta = std::fs::metadata(&path).map_err(|e| format!("scout: {path}: {e}"))?;//follows a link to what it names, since a link to a picture is a picture, and the kind refused below is the target's
		if !meta.is_file() { return Err(format!("scout: not a regular file: {path}")) }
		let mut file = open_file(&path).map_err(|e| format!("scout: {path}: {e}"))?;
		let mut tip = vec![0u8; ::scout::TIP_BYTES];//the allocations, this and the slice's below, are sized by the crate's constant and never by anything in the file
		let got = read_into(&mut file, &mut tip).map_err(|e| format!("scout: {path}: {e}"))?;
		tip.truncate(got);
		let mut t = ::scout::scout_bytes(&tip, meta.len());
		if t.slice > 0 {//the reader asks for the second slice: the same read again at the offset the answer names, and the reader over both; whatever the second answer asks for, this was the last read
			use std::io::Seek;
			file.seek(std::io::SeekFrom::Start(t.slice)).map_err(|e| format!("scout: {path}: {e}"))?;
			let mut slice = vec![0u8; ::scout::TIP_BYTES];
			let got = read_into(&mut file, &mut slice).map_err(|e| format!("scout: {path}: {e}"))?;
			slice.truncate(got);
			t = ::scout::scout_slices(&tip, &slice, t.slice, meta.len());
		}
		Ok(t)
	}).await
}
fn open_file(path: &str) -> std::io::Result<std::fs::File> {//the file for reading, opened without waiting on the mac and linux: an open of a FIFO otherwise blocks until something writes to it, and the kind check above can be raced. A read of a regular file still waits on the disk, which is the containment the scan process is for
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
	use std::io::Read;
	let mut got = 0;
	while got < buffer.len() {
		let Some(rest) = buffer.get_mut(got..) else { break };
		let n = file.read(rest)?;
		if n == 0 { break }
		got = got.saturating_add(n);
	}
	Ok(got)
}

/*
Hashing, to come. The whole-file hashes belong in this module beside the tip's command: SHA-1 and SHA-256, the names the rest of the world checks files by; BLAKE3, which hashes a file in pieces on every core; and fuji's own quick hash of a file's size and a few small stripes from its start, its middle and its end, which finds likely copies across millions of files for the cost of a few reads each. Each will be a command that takes one path and the hash's name and answers one hash, reading the file in a buffer fuji sized, on the blocking pool like the command above, with the same rule that nothing in the file sizes anything; nothing of them is built.
*/
