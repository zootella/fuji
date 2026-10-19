#![no_main]
//the scout under libFuzzer. pnpm fuzz in the scout folder runs it, seeded from the sample files, and keeps what it finds in fuzz/corpus; a panic anywhere in the reader is a finding, written to fuzz/artifacts. Each input is read twice over: as the command reads a file, a tip of at most 64 KiB and then the slice the first answer asks for, and again through a tip cut to half the input, so the second read is exercised by inputs far smaller than a tip

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
	scout(data, scout::tip::TIP_BYTES);
	scout(data, data.len() / 2);
});

fn scout(data: &[u8], cut: usize) {//the command's two reads over the input as a file: the first cut bytes, then 64 KiB at the offset the first answer comes with, when it comes with one
	let tip = data.get(..cut.min(scout::tip::TIP_BYTES)).unwrap_or(data);
	let (_, wish) = scout::tip::scout_bytes(tip, data.len() as u64);
	let Some(at) = wish else { return };
	let from = usize::try_from(at).unwrap_or(usize::MAX);
	let slice = from.checked_add(scout::tip::TIP_BYTES).and_then(|end| data.get(from..end.min(data.len()))).unwrap_or(&[]);
	let _ = scout::tip::scout_slices(tip, slice, at, data.len() as u64);
}
