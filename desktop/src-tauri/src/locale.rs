//the locale command: how the user's system writes numbers, from copper, on the blocking pool

use crate::run_blocking;

/// The separator the user's system puts between groups of digits, like the comma in 16,444,365, with the user's own choice in the system's settings; blank when the system will not say
#[tauri::command]//its body on the blocking pool because linux answers by reading the locale's file from the disk
pub async fn locale_grouping() -> String {
	run_blocking(|| Ok(copper::locale::grouping())).await.unwrap_or_default()//blank for a panic in the body too
}
