//the scout's command: the tip of one file by its path, and the second slice when the tip asks for one, read in fuji's own process and handed to the scout, fuji's own reader of a file's first bytes in safe rust, in steel, the safe crate at the repository root, where its essay says what it answers and what it promises; the reading itself is steel's too, in its read.rs, which the inventory in copper uses as this does

/// What the first bytes of this file say: its format and kind, the stored size of a picture or a video and the size to show it at, the running time of a sound or a video, whether the file is corrupted or chopped, and the rest of the fields scout_bytes answers, with the scout's report on its own reading beside them. The tip is the first 64 KiB, and when the tip says where the rest of its answer lies, one more 64 KiB slice at the offset it names, which is the last read, never a third. An error for a path that is not a regular file, since opening a FIFO waits for a writer, and for a file that cannot be opened or read
#[tauri::command]
pub async fn scout_tip(path: String) -> Result<steel::tip::Tip, String> {
	crate::run_blocking(move || {
		let meta = std::fs::metadata(&path).map_err(|e| format!("scout: {path}: {e}"))?;//follows a link to what it names, since a link to a picture is a picture, and the kind refused below is the target's
		if !meta.is_file() { return Err(format!("scout: not a regular file: {path}")) }
		let mut file = steel::read::open_file(&path).map_err(|e| format!("scout: {path}: {e}"))?;
		steel::read::tip_read(&mut file, meta.len()).map_err(|e| format!("scout: {path}: {e}"))
	}).await
}
