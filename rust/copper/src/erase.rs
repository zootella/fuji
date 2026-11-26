//erasing a file: its contents written over once with random bytes where they sit, flushed to the device, its name written over, and the file removed; and moving one, which erases the original when the move has to copy it to another volume; brass's disk.rs holds the commands

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use zootella_ergo::{size, Data};

/*
One pass of random bytes, since one is all any drive since the 1990s needs, and random is what Gutmann's paper recommends for a drive whose encoding nobody can aim at; the Erase page on fuji's site has that history, and what no overwrite reaches: the old pages inside flash, a copy-on-write file system's old blocks, a snapshot's. The bytes are BLAKE3's stream under a key of 32 bytes from the operating system, since a keyed BLAKE3 is a pseudorandom function and the processor fills a gigabyte with it in a third of the time the operating system's own source takes, measured on the Mac mini. The flush comes before the removal, since a removed file's pages in the cache need never be written at all.

A symbolic link, a file with another hard link and a file a cloud service keeps are removed without the overwrite, as a plain delete removes them: a link's bytes are the file it names, a hard link's are still the other name's, and writing to a cloud file reaches through the service, which would upload the random bytes as a new version. The answer says which happened.

A file the user may not write is refused with the system's own error. Writing a file takes permission on the file, and removing it takes permission only on its folder on the Mac and Linux, while on Windows the standard library's remove_file clears the read-only attribute on its way, so on every platform a plain delete removes a read-only file that erase cannot write over. What a delete should do then, ask, clear the attribute, or remove the file without the overwrite, is the page's decision, and nothing makes it yet. A move across volumes meets the same refusal after its copy has landed, and says so.

Each call runs to its end on its thread, as long as writing the file once takes, with no count of its progress and no way to stop it; brass's disk essay says what the commands that change the disk still need there.
*/

const PIECE: usize = size::MB as usize;//written a megabyte at a time

/// Erase the file at path, and answer how: Erased when its contents were written over and it was removed, or Symlink, Hardlink or Cloud when it was only removed, as a plain delete removes it
pub fn erase(path: &str) -> std::io::Result<&'static str> {
	let found = std::fs::symlink_metadata(path)?;//the path itself, never what a link names
	if found.file_type().is_symlink() { std::fs::remove_file(path)?; return Ok("Symlink") }
	if !found.is_file() { return Err(std::io::Error::other(format!("erase: not a regular file: {path}"))) }
	if cloud(&found) { std::fs::remove_file(path)?; return Ok("Cloud") }

	let mut file = open(path)?;
	let held = file.metadata()?;//the file now open, in case another took its place since the look above
	if !held.is_file() { return Err(std::io::Error::other(format!("erase: not a regular file: {path}"))) }
	if crate::inventory::links(&file, &held) > 1 { drop(file); std::fs::remove_file(path)?; return Ok("Hardlink") }

	let allocated = allocation(&file, &held);
	overwrite(&mut file, held.len(), allocated)?;
	file.sync_all()?;//to the device before anything removes the file; on the Mac the standard library flushes through F_FULLFSYNC, past the drive's own cache
	let named = rename(Path::new(path));
	let _ = file.sync_all();//the new name too, which on Windows sits in the record the handle flushes
	#[cfg(unix)]
	if let Some(folder) = named.parent() { let _ = File::open(folder).and_then(|folder| folder.sync_all()); }//and on the Mac and Linux in the folder, flushed as a file of its own
	drop(file);
	std::fs::remove_file(&named).map_err(|e| std::io::Error::new(e.kind(), format!("erase: {path}: written over but not removed: {e}")))?;
	Ok("Erased")
}

/// Move the file at source to destination, and answer how: Renamed when one volume holds both, or, when the move had to copy it to another volume, how the original went, as erase answers
pub fn move_file(source: &str, destination: &str) -> std::io::Result<&'static str> {
	match std::fs::rename(source, destination) {
		Ok(()) => return Ok("Renamed"),
		Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {}//another volume, which no rename reaches
		Err(e) => return Err(e),
	}
	let found = std::fs::symlink_metadata(source)?;
	if !found.is_file() { return Err(std::io::Error::other(format!("move: only a file moves to another volume, and {source} is not one"))) }//a folder would need its whole tree carried, and a link would be copied as the file it names
	let made = std::fs::symlink_metadata(destination).is_err();//whether the copy makes the destination, rather than writing over a file the user had there
	if let Err(e) = carry(source, destination, &found) { if made { let _ = std::fs::remove_file(destination); } return Err(e) }//no half a copy left behind
	erase(source).map_err(|e| std::io::Error::new(e.kind(), format!("move: {destination} written, but {e}")))
}

fn carry(source: &str, destination: &str, found: &std::fs::Metadata) -> std::io::Result<()> {//the copy a move across volumes makes: the bytes, the times the file had, and all of it on the device before the original is erased
	std::fs::copy(source, destination)?;
	let copy = std::fs::OpenOptions::new().write(true).open(destination)?;
	let mut times = std::fs::FileTimes::new();//each the platform keeps, since a collection sorts by them
	if let Ok(time) = found.modified() { times = times.set_modified(time) }
	if let Ok(time) = found.accessed() { times = times.set_accessed(time) }
	#[cfg(target_os = "macos")]
	if let Ok(time) = found.created() { use std::os::macos::fs::FileTimesExt; times = times.set_created(time) }
	#[cfg(windows)]
	if let Ok(time) = found.created() { use std::os::windows::fs::FileTimesExt; times = times.set_created(time) }
	copy.set_times(times)?;
	copy.sync_all()
}

fn open(path: &str) -> std::io::Result<File> {//for writing in place, so never truncated, and never through a link
	let mut options = std::fs::OpenOptions::new();
	options.read(true).write(true);//read as well, since Windows answers a handle's metadata only to one allowed to read its attributes
	#[cfg(any(target_os = "macos", target_os = "linux"))]
	{
		use std::os::unix::fs::OpenOptionsExt;
		options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);//a link refused rather than followed, and a pipe answering at once rather than waiting for a reader
	}
	#[cfg(windows)]
	{
		use std::os::windows::fs::OpenOptionsExt;
		options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
	}
	options.open(path)
}

fn overwrite(file: &mut File, length: u64, allocated: u64) -> std::io::Result<()> {//every byte of the file once, from its start, with bytes no other erasure shares, and then on to the end of its last cluster
	let mut key = [0u8; 32];
	key.copy_from_slice(Data::random(32).array());
	let mut stream = blake3::Hasher::new_keyed(&key).finalize_xof();
	let mut buffer = vec![0u8; PIECE];
	let mut fill = |mut left: u64| -> std::io::Result<()> {//that many bytes of the stream into the file, a piece at a time
		while left > 0 {
			let n = usize::try_from(left).map_or(PIECE, |left| left.min(PIECE));
			let piece = &mut buffer[..n];
			stream.fill(piece);
			file.write_all(piece)?;
			left -= n as u64;
		}
		Ok(())
	};
	fill(length)?;
	let _ = fill(allocated.saturating_sub(length));//the slack between the file's end and its last cluster's, which keeps whatever was there before the file shrank or the cluster was reused, and which shred covers by rounding up to the block; let go when a volume with no block left refuses it, since ext4 counts a block of its own among a fragmented file's and so sends this a block past the cluster
	Ok(())
}

#[cfg(unix)]
fn allocation(_: &File, meta: &std::fs::Metadata) -> u64 { use std::os::unix::fs::MetadataExt; meta.blocks().saturating_mul(512) }//st_blocks counts 512-byte units on the Mac and Linux whatever st_blksize says, and Linux's fat and exfat drivers count whole clusters in it while reporting the sector as the block size, which is why the allocation and not shred's block size
#[cfg(windows)]
fn allocation(file: &File, _: &std::fs::Metadata) -> u64 {
	use windows::Win32::Storage::FileSystem::{FileStandardInfo, FILE_STANDARD_INFO};
	crate::inventory::handle_info::<FILE_STANDARD_INFO>(file, FileStandardInfo).map_or(0, |standard| u64::try_from(standard.AllocationSize).unwrap_or(0))//the clusters the file holds, Explorer's size on disk; for a small file NTFS keeps inside its own record, its length rounded up to eight bytes, so the overwrite runs at most seven bytes past the end, read on the Windows box on 2026-10-30 as 304 for a file of 300; on exFAT the whole cluster, 131072 for a file of 26874 on a USB stick, so the overwrite there runs to the cluster's end
}
#[cfg(not(any(unix, windows)))]
fn allocation(_: &File, _: &std::fs::Metadata) -> u64 { 0 }

/*
A file's name lives apart from its bytes, in the folder's own records, and deleting the file marks its record free and leaves the name's characters there until something reuses the space, where an undelete program reads them. The rename to random letters is an attempt to write over them first. It is safe: the new name is lowercase letters, never one of Windows' device names, and never a name the folder already holds, since a rename replaces a file at its destination; a rename that fails leaves the file under its own name, and the erase goes on.

Whether it works depends on how each file system renames. Read in Linux's source in October 2026: exFAT rewrites a name in place when the new one needs no more directory entries, which a name of the same length never does, so there it works; FAT32 writes a long name into new entries and marks the old ones deleted, which leaves the old characters behind. APFS and Btrfs copy on write and put the new name in a new node. Neither read nor measured: Windows' and the Mac's own exFAT and FAT drivers, ext4, and NTFS, which keeps a name both in the file's own record and in the folder's index, and logs a rename in its journal.
*/
fn rename(path: &Path) -> PathBuf {//the file under random lowercase letters as many as its name has, or under its own name where no such rename went through
	let (Some(folder), Some(name)) = (path.parent(), path.file_name()) else { return path.to_path_buf() };
	let length = name.to_string_lossy().encode_utf16().count();//the length FAT, exFAT and NTFS store a name at
	for _ in 0..4 {//a few tries, past a name that is taken
		let letters: String = Data::random(length).array().iter().map(|b| char::from(b'a' + b % 26)).collect();
		if ["con", "prn", "aux", "nul"].contains(&letters.as_str()) { continue }//names Windows keeps for devices
		let target = folder.join(&letters);
		if !matches!(std::fs::symlink_metadata(&target), Err(e) if e.kind() == std::io::ErrorKind::NotFound) { continue }//taken, and a rename would write over it
		if std::fs::rename(path, &target).is_ok() { return target }
	}
	path.to_path_buf()
}

#[cfg(target_os = "macos")]
const SF_DATALESS: u32 = 0x4000_0000;//by its name in sys/stat.h: a file whose bytes a provider like iCloud Drive keeps elsewhere until it is opened
#[cfg(target_os = "macos")]
fn cloud(meta: &std::fs::Metadata) -> bool { use std::os::macos::fs::MetadataExt; meta.st_flags() & SF_DATALESS != 0 }
#[cfg(windows)]
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;//by their names in winnt.h
#[cfg(windows)]
const CLOUD_ATTRIBUTES: u32 = 0x0000_0400 | 0x0000_1000 | 0x0004_0000 | 0x0040_0000;//FILE_ATTRIBUTE_REPARSE_POINT, OFFLINE, RECALL_ON_OPEN and RECALL_ON_DATA_ACCESS
#[cfg(windows)]
fn cloud(meta: &std::fs::Metadata) -> bool { use std::os::windows::fs::MetadataExt; meta.file_attributes() & CLOUD_ATTRIBUTES != 0 }//a reparse point that is not a link, which erase has already answered: OneDrive marks every file it keeps with one, downloaded or not
#[cfg(not(any(target_os = "macos", windows)))]
fn cloud(_: &std::fs::Metadata) -> bool { false }
