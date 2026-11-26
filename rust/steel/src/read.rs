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

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects, clippy::panic)]//a test may index, unwrap and add: a mistake in one fails the test, which is what a test is for
mod tests {
	use super::*;

	fn sample(name: &str) -> String { format!("{}/samples/{name}", env!("CARGO_MANIFEST_DIR")) }
	fn read_sample(name: &str) -> crate::tip::Tip {
		let mut file = open_file(sample(name)).unwrap();
		let size = file.metadata().unwrap().len();
		tip_read(&mut file, size).unwrap()
	}

	#[test]
	fn the_tip_is_read_from_a_file_and_the_second_slice_when_the_tip_asks() {
		let t = read_sample("picture.png");
		assert_eq!((t.format, t.kind, (t.raster.width, t.raster.height), t.reading.reads), ("png", "image", (32, 20), 1));
		let t = read_sample("moov-last.mp4");//the movie box after the data, but the sample is small enough that the tip holds the whole file, so no second read is needed
		assert_eq!((t.format, t.seconds, t.reading.reads, t.reading.details), ("mp4", 1.0, 1, ""));
		let path = std::env::temp_dir().join(format!("steel-read-wav-{}", std::process::id()));//a WAV too long for the tip: a 70,000-byte LIST chunk before the sound, so the data chunk the running time needs is past the first 64 KiB, which the tip asks for and the second read reaches
		let mut v = b"RIFF".to_vec(); v.extend(102044u32.to_le_bytes()); v.extend(b"WAVEfmt "); v.extend(16u32.to_le_bytes()); v.extend([1, 0, 1, 0]); v.extend(8000u32.to_le_bytes()); v.extend(16000u32.to_le_bytes()); v.extend([2, 0, 16, 0]);
		v.extend(b"LIST"); v.extend(70000u32.to_le_bytes()); v.resize(v.len() + 70000, 0);
		v.extend(b"data"); v.extend(32000u32.to_le_bytes()); v.resize(v.len() + 32000, 0);
		std::fs::write(&path, &v).unwrap();
		let mut file = open_file(&path).unwrap();
		let t = tip_read(&mut file, v.len() as u64).unwrap();
		let _ = std::fs::remove_file(&path);
		assert_eq!((t.format, t.seconds, t.reading.reads, t.reading.details), ("wav", 2.0, 2, ""));
		let t = read_sample("empty");
		assert_eq!((t.format, t.reading.details), ("", "empty"));
		assert!(open_file(sample("not-there")).is_err());
	}

	#[cfg(unix)]
	#[test]
	fn a_pipe_opens_without_waiting_for_a_writer() {//the reason the open is made without blocking: an ordinary open of a named pipe waits until something writes to it, which on a drive of unknown files is a wait with no end
		let path = std::env::temp_dir().join(format!("steel-read-pipe-{}", std::process::id()));
		let _ = std::fs::remove_file(&path);
		assert!(std::process::Command::new("mkfifo").arg(&path).status().unwrap().success());
		let (sender, receiver) = std::sync::mpsc::channel();
		let at = path.clone();
		std::thread::spawn(move || { let _ = sender.send(open_file(&at).and_then(|mut file| tip_read(&mut file, 0))); });
		let answer = receiver.recv_timeout(std::time::Duration::from_secs(5)).expect("the open and the read answered within five seconds, rather than waiting for a writer that never comes");
		let _ = std::fs::remove_file(&path);
		assert_eq!(answer.unwrap().reading.details, "empty");//a pipe with no writer reads as a file with nothing in it
	}
}
