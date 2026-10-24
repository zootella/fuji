//the reads for the scout: a file opened without waiting, and its tip and second slice read and handed to the reader in tip.rs. The command in brass and the inventory in copper both read a file this way

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn tip_read(file: &mut std::fs::File, size: u64) -> std::io::Result<crate::tip::Tip> {//what the reader says of a file just opened: the tip from its start, and the reader over both when it asks for the second slice, which is the same read again at the offset that came back beside the answer, and the last read
	let mut tip = vec![0u8; crate::tip::TIP_BYTES];//the allocations, this and the slice's below, are sized by the crate's constant and never by anything in the file
	let got = read_into(file, &mut tip)?;
	tip.truncate(got);
	let (mut t, wish) = crate::tip::scout_bytes(&tip, size);
	if let Some(at) = wish {
		file.seek(SeekFrom::Start(at))?;
		let mut slice = vec![0u8; crate::tip::TIP_BYTES];
		let got = read_into(file, &mut slice)?;
		slice.truncate(got);
		t = crate::tip::scout_slices(&tip, &slice, at, size);
	}
	Ok(t)
}
pub fn open_file(path: impl AsRef<Path>) -> std::io::Result<std::fs::File> {//the file for reading, opened without waiting on the mac and linux: an open of a FIFO otherwise blocks until something writes to it, and a kind check made before the open can be raced. A read of a regular file still waits on the disk, which is the containment the scan process is for
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
