#![no_main]
//the scout under libFuzzer. pnpm fuzz in the scout folder runs it, seeded from the sample files, and keeps what it finds in fuzz/corpus; a panic anywhere in the reader is a finding, written to fuzz/artifacts

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
	let tip = data.get(..scout::TIP_BYTES).unwrap_or(data);//at most a tip's worth, as fuji's command reads
	let _ = scout::scout_bytes(tip, data.len() as u64);
});
