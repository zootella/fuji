//an argon2id speed test: the settings rfc 9106 recommends where memory is tight, then the same with ten and a hundred times the passes; argon.md says how to build and run it, and holds the results from each machine
use argon2::{Algorithm, Argon2, Params, Version};
use std::time::Instant;
use zootella_ergo::{Data, size, time};

const MEMORY: u32 = (64 * size::MB / size::KB) as u32;//64 MiB, the rfc's second recommendation, in the KiB argon2 counts memory in
const LANES: u32 = 4;//also the rfc's, and part of the result: a different number of lanes derives a different key
const PLAN: [(u32, u32); 3] = [(3, 7), (30, 5), (300, 3)];//passes, and how many times to time each; 3 is the rfc's

fn derive(passes: u32) -> [u8; 32] {//one run: a fixed passphrase and salt, so every machine derives the same key for the same settings
	let params = Params::new(MEMORY, passes, LANES, Some(32)).expect("the settings are valid");
	let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
	let mut key = [0u8; 32];
	argon.hash_password_into(b"correct horse battery staple", b"Fuji.Argon2id.SpeedTest.", &mut key).expect("the derivation runs");
	key
}

fn main() {
	for (passes, runs) in PLAN {
		let mut times = Vec::new();
		let mut key = [0u8; 32];
		for _ in 0..runs {
			let began = Instant::now();
			key = derive(passes);
			times.push(began.elapsed().as_secs_f64() * time::SECOND as f64);
		}
		times.sort_by(|a, b| a.total_cmp(b));
		let hex = Data::from_array(&key[..8]).base16();//the first 8 bytes, to check that machines agree
		println!("{} MiB, {passes} passes, {LANES} lanes: median {:.0} ms, fastest {:.0} ms, {runs} runs, key {hex}", u64::from(MEMORY) * size::KB / size::MB, times[times.len() / 2], times[0]);
	}
}
