//the lock: a key from a passphrase, and a file sealed under it and opened again, from four published designs, argon2id, hkdf-sha256, aes-256-gcm and padmé; the Bike Lock page on fuji's site gives the account of each and draws the sealed file byte by byte

use aes_gcm::aead::{Nonce, Tag};
use aes_gcm::{AeadInOut, Aes256Gcm, KeyInit};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;
use zootella_ergo::{sha256, size, Data};

//rfc 9106's second recommended option, from its section 4, as the rfc gives it
pub const MEMORY: u32 = (64 * size::MB / size::KB) as u32;//64 MiB in the KiB argon2 counts in: a table of 65,536 blocks of 1 KiB
pub const PASSES: u32 = 3;//fill the table, then rewrite every block twice more
pub const LANES: u32 = 4;//four rows of the table, filled side by side on four threads
pub const KEY: usize = 32;//the key argon2id makes, the option's 256-bit tag size
pub const SALT: usize = 16;//the option's 128-bit salt, and the size of each file's salt too

//a sealed file: the file salt and the nonce in the clear, then the length, the file and padmé's zeros encrypted together, then the tag
pub const NONCE: usize = 12;
pub const LENGTH: usize = 8;//the file's length, a u64, little-endian, the order of every processor fuji runs on
pub const TAG: usize = 16;
const HEADER: usize = SALT + NONCE;

pub fn key(passphrase: &str, label: &str) -> Option<Data> {//the key a passphrase makes under a label, the slow step. The salt is the label's sha-256 cut to the option's 128 bits, the same on every machine and in every year, so one run serves every file sealed under that label
	let salt = sha256(&Data::from_text(label)).clip_view(0, SALT);
	let settings = Params::new(MEMORY, PASSES, LANES, Some(KEY)).ok()?;
	let mut out = [0u8; KEY];
	Argon2::new(Algorithm::Argon2id, Version::V0x13, settings).hash_password_into(Data::from_text(passphrase).array(), salt.array(), &mut out).ok()?;//from_text makes the passphrase utf-8 in normalization form c, so the same characters typed two ways make the same key
	Some(Data::from_array(&out))
}

pub fn padme(length: u64) -> Option<u64> {//the size padmé rounds a length up to, algorithm 1 of nikitin and colleagues, 2019: keep as many leading bits as it takes to write the length's exponent, and round away the rest
	if length < 2 { return Some(length) }//no exponent to speak of; a sealed file's length field alone makes 8
	let e = length.ilog2();//the length's exponent
	let s = e.ilog2().checked_add(1)?;//the bits it takes to write e
	let z = e.checked_sub(s)?;//the low bits to round away
	let m = 1u64.checked_shl(z)?.checked_sub(1)?;//a mask of z ones
	Some(length.checked_add(m)? & !m)
}

pub fn seal(key: &Data, file: &Data) -> Option<Data> {//the file sealed under a key from key(), in one allocation sized by padmé, encrypted where it lies
	let n = u64::try_from(file.size()).ok()?;//the file's length, as the length field holds it
	let padded = usize::try_from(padme(n.checked_add(LENGTH as u64)?)?).ok()?;//the length field and the file, rounded up
	let salt = Data::random(SALT);
	let nonce = Data::random(NONCE);//random, though no file key ever seals twice: a salt repeated by mistake still never repeats a key and a nonce together
	let mut out = Vec::with_capacity(HEADER.checked_add(padded)?.checked_add(TAG)?);
	out.extend_from_slice(salt.array());
	out.extend_from_slice(nonce.array());
	out.extend_from_slice(&n.to_le_bytes());
	out.extend_from_slice(file.array());
	out.resize(HEADER.checked_add(padded)?, 0);//padmé's zeros
	let tag = file_cipher(key, salt.array())?.encrypt_inout_detached(&Nonce::<Aes256Gcm>::try_from(nonce.array()).ok()?, &[], out.get_mut(HEADER..)?.into()).ok()?;
	out.extend_from_slice(&tag);
	Some(Data::from(out))
}

pub fn open(key: &Data, sealed: &Data) -> Option<Data> {//the file a sealed file holds, or none for a wrong key, a changed byte, or bytes too short to be a sealed file at all, which the tag does not tell apart. These are bytes nobody vouched for, so every split is checked
	let (salt, rest) = sealed.array().split_at_checked(SALT)?;
	let (nonce, rest) = rest.split_at_checked(NONCE)?;
	let (ciphertext, tag) = rest.split_at_checked(rest.len().checked_sub(TAG)?)?;
	let mut plain = ciphertext.to_vec();//the one copy, since the cipher decrypts where the bytes lie
	file_cipher(key, salt)?.decrypt_inout_detached(&Nonce::<Aes256Gcm>::try_from(nonce).ok()?, &[], plain.as_mut_slice().into(), &Tag::<Aes256Gcm>::try_from(tag).ok()?).ok()?;//checks the tag over every byte before it decrypts one
	let n = usize::try_from(u64::from_le_bytes(plain.get(..LENGTH)?.try_into().ok()?)).ok()?;
	let end = LENGTH.checked_add(n)?;
	if end > plain.len() { return None }
	plain.truncate(end);
	plain.drain(..LENGTH);
	Some(Data::from(plain))
}

fn file_cipher(key: &Data, salt: &[u8]) -> Option<Aes256Gcm> {//the cipher for one file, under the file's own key, which hkdf-sha256 makes from the passphrase's key and the file's salt
	let mut file_key = [0u8; KEY];
	Hkdf::<Sha256>::new(Some(salt), key.array()).expand(&[], &mut file_key).ok()?;
	Aes256Gcm::new_from_slice(&file_key).ok()
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects, clippy::panic)]//a test may index, unwrap and add: a mistake in one fails the test, which is what a test is for
mod tests {
	use super::*;
	use std::sync::OnceLock;

	const LABEL: &str = "bike lock";//the label fuji's page will pass
	const WORDS: &str = "anchor velvet quarry tundra mosaic thimble";//dice words whose key another implementation computed, in the first test

	fn pinned_key() -> &'static Data {//the key WORDS make, derived once for every test that needs a key and not a particular one, since argon2id is slow in a test build
		static KEY: OnceLock<Data> = OnceLock::new();
		KEY.get_or_init(|| key(WORDS, LABEL).unwrap())
	}
	fn dice_words() -> String {//six words drawn at random, as dice would draw them from a longer list
		let list = ["acorn", "badge", "cello", "dingo", "ember", "fjord", "gusto", "haiku", "igloo", "jumbo", "kayak", "lemur", "mango", "nylon", "otter", "plaza"];
		Data::random(6).array().iter().map(|b| list[usize::from(*b) % list.len()]).collect::<Vec<_>>().join(" ")
	}

	#[test]
	fn a_file_locked_by_another_implementation_opens_here() {//a file locked today has to open on another machine and in every later version of fuji, and a change to the format that seal and open both made would pass every round trip below while locking out every file locked before it. So these answers were made apart from this code, in python: the dice key by argon2-cffi 25.1.0, which vendors the reference c implementation from the password hashing competition, and the locked files by cryptography 50.0.2's hkdf and aes-gcm, with padmé written again from the paper
		assert_eq!(pinned_key().base16(), "12c7129b5472d39e70df6483cd911a78cc5d625d2f36693a2556a5249649757d");
		let locked = [//a file salt of the bytes 0 through 15 and a nonce of 16 through 27, so the part in the clear is easy to read
			("", "000102030405060708090a0b0c0d0e0f101112131415161718191a1b9f6e518ed2d8b32942753f2ead751ed100173c9b209db0e7"),
			("a file locked by an independent implementation", "000102030405060708090a0b0c0d0e0f101112131415161718191a1bb16e518ed2d8b329fbc76417b2fdcc5270e1241c1fcc4956d056d4b037c42a7f069b257b062cf9ca6be55c0375a4fa7e96d69d8855e0d16e71dbddf7a7d0dc7f5dac32d5dcb7f33b"),
		];
		for (file, sealed) in locked { assert_eq!(open(pinned_key(), &Data::from_base16(sealed).unwrap()), Some(Data::from_text(file)), "{file:?}") }
	}

	#[test]
	fn the_same_words_typed_two_ways_make_the_key_another_implementation_makes() {//an accent typed as one character or as a letter and a combining mark, as keyboards and systems differ; python normalized the words to form c before argon2-cffi hashed them
		let pinned = "318f1d846864f95913a4b1dff00de198b2950e7ef3ffc2ae5d9a86938ef64887";
		assert_eq!(key("cr\u{e8}me br\u{fb}l\u{e9}e tundra mosaic thimble", LABEL).unwrap().base16(), pinned);
		assert_eq!(key("cre\u{300}me bru\u{302}le\u{301}e tundra mosaic thimble", LABEL).unwrap().base16(), pinned);
	}

	#[test]
	fn a_folder_locked_under_two_sets_of_dice_words_opens_with_whichever_key_fits() {//the user locked some files with one set of words and some with another, and fuji tries every key it holds on each file; a near miss, and the right words under another label, open nothing
		let (first, second) = (pinned_key(), key(&dice_words(), LABEL).unwrap());
		let near_misses = [key("anchor velvet quarry tundra mosaic thimbles", LABEL).unwrap(), key(WORDS, "bike lock 2").unwrap()];//a typo, and the same words under another label
		for i in 0..12 {
			let file = Data::random(i * 1000);
			let locker = if i % 3 == 0 { first } else { &second };
			let sealed = seal(locker, &file).unwrap();
			let opened: Vec<Data> = [first, &second, &near_misses[0], &near_misses[1]].into_iter().filter_map(|k| open(k, &sealed)).collect();
			assert_eq!(opened, [file], "file {i} opens with the one key that locked it, and with no other");
		}
	}

	#[test]
	fn damage_a_disk_or_a_download_can_do_opens_nothing() {//a file cut short by an interrupted copy, a download with zeros where the rest never arrived, a sector a dying disk zeroed, a bit flipped in storage, and bytes some tool appended: each opens to nothing rather than to something wrong
		let k = pinned_key();
		let sealed = seal(k, &Data::random(600)).unwrap();
		let a = sealed.array();
		let damaged = |d: Vec<u8>| open(k, &Data::from(d));
		for n in 0..a.len() { assert_eq!(open(k, &sealed.clip_view(0, n)), None, "cut to {n} bytes") }
		for i in 0..a.len() {
			for bit in [0x01, 0x80] { let mut d = a.to_vec(); d[i] ^= bit; assert_eq!(damaged(d), None, "bit {bit:#x} of byte {i} flipped") }
		}
		let mut d = a.to_vec(); d[a.len() / 2..].fill(0); assert_eq!(damaged(d), None, "the second half zeros");
		let mut d = a.to_vec(); d[64..320].fill(0); assert_eq!(damaged(d), None, "a run of zeros in the middle");
		for extra in [1, 16, 4096] { let mut d = a.to_vec(); d.extend_from_slice(Data::random(extra).array()); assert_eq!(damaged(d), None, "{extra} bytes appended") }
	}

	#[test]
	fn a_file_that_was_never_locked_opens_nothing() {//a picture, a song or an archive given the mark by hand: every format among the scout's samples, and random bytes of every short length
		let k = pinned_key();
		let mut count = 0;
		for entry in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/samples")).unwrap() {
			let path = entry.unwrap().path();
			if !path.is_file() { continue }
			assert_eq!(open(k, &Data::from(std::fs::read(&path).unwrap())), None, "{}", path.display());
			count += 1;
		}
		assert!(count >= 70, "{count} samples, when the folder holds more than 70");//the folder is there and full, since an empty one would prove nothing
		for n in 0..100 { assert_eq!(open(k, &Data::random(n)), None, "{n} random bytes") }
	}

	#[test]
	fn a_file_opens_to_exactly_what_was_sealed_at_every_size() {//every length up to a few hundred bytes, the 121 bytes padmé pads the most, the edges of a block and of 64 KiB, and a phone's picture, each sealed to padmé's size plus the 44 bytes around it
		let k = pinned_key();
		for n in (0..=300).chain([4095, 4096, 4097, 65_535, 65_536, 65_537, 3 * size::MB as usize]) {
			let file = Data::random(n);
			let sealed = seal(k, &file).unwrap();
			assert_eq!(sealed.size() as u64, padme(LENGTH as u64 + n as u64).unwrap() + (HEADER + TAG) as u64, "the sealed size of {n} bytes");
			assert_eq!(open(k, &sealed), Some(file), "{n} bytes");
		}
	}

	#[test]
	fn sealing_writes_the_length_little_endian_and_pads_with_zeros() {//the writer's half of the format, read here past open, which takes the length and stops: a reader elsewhere that checks the padding, as the python in the first test did, has to find zeros
		let k = pinned_key();
		let file = Data::random(1000);
		let sealed = seal(k, &file).unwrap();
		let a = sealed.array();
		let mut plain = a[HEADER..a.len() - TAG].to_vec();
		let tag = Tag::<Aes256Gcm>::try_from(&a[a.len() - TAG..]).unwrap();
		file_cipher(k, &a[..SALT]).unwrap().decrypt_inout_detached(&Nonce::<Aes256Gcm>::try_from(&a[SALT..HEADER]).unwrap(), &[], plain.as_mut_slice().into(), &tag).unwrap();
		assert_eq!(&plain[..LENGTH], &[0xe8, 0x03, 0, 0, 0, 0, 0, 0]);//1000, lowest byte first
		assert_eq!(&plain[LENGTH..LENGTH + 1000], file.array());
		assert!(plain[LENGTH + 1000..].iter().all(|b| *b == 0) && plain.len() == 1024, "padmé takes 1008 to 1024, with zeros");
	}

	#[test]
	fn the_same_file_sealed_twice_shares_nothing() {//locking a file again, or a copy of it elsewhere, gives no sign the two hold the same file
		let k = pinned_key();
		let file = Data::random(5000);
		let (a, b) = (seal(k, &file).unwrap(), seal(k, &file).unwrap());
		let (a, b) = (a.array(), b.array());
		assert!(a[..SALT] != b[..SALT] && a[SALT..HEADER] != b[SALT..HEADER] && a[HEADER..] != b[HEADER..]);//the file salt, the nonce, and the ciphertext with its tag
	}

	#[test]
	fn files_of_nearby_sizes_lock_to_one_size() {//the Bike Lock page's example: every file from 2,949,113 to 3,014,648 bytes locks to the same size, and the sizes just outside do not
		let locked = |n: u64| padme(n + LENGTH as u64).unwrap() + (HEADER + TAG) as u64;
		let one = locked(3_000_000);
		assert!((2_949_113..=3_014_648).all(|n| locked(n) == one));
		assert!(locked(2_949_112) != one && locked(3_014_649) != one);
	}

	#[test]
	fn padme_rounds_as_the_paper_does() {
		assert_eq!(padme(9), Some(10));//the paper's own example
		let over = |(length, p): (u64, u64)| (p - length) as f64 / length as f64;//the overhead, as a fraction
		let mut worst = (2, 2);
		for length in (2..200_000).chain([u64::from(u32::MAX), 1 << 40, (1 << 40) + 1, u64::MAX / 3]) {
			let p = padme(length).unwrap();
			assert!(p >= length && padme(p) == Some(p), "{length} to {p}");//never smaller, and a padded size pads to itself
			assert!(u128::from(p - length) * 100 < u128::from(length) * 12, "{length} to {p} is 12% or more");//the bound the paper states, always less than 12%
			if over((length, p)) > over(worst) { worst = (length, p) }
		}
		assert_eq!(worst, (129, 144));//the worst case, 11.6%: lengths from 128 to 255 round to multiples of 16, though the paper's text names 9 to 10, 11.1%, as its worst
		assert_eq!(padme(u64::MAX), None);
	}

	#[test]
	fn many_files_open_at_once_under_one_key() {//fuji opens a folder's locked files on several threads at once, every one with the same key
		let k = pinned_key();
		let files: Vec<Data> = (0..16).map(|i| Data::random(10_000 + i)).collect();
		let sealed: Vec<Data> = files.iter().map(|f| seal(k, f).unwrap()).collect();
		std::thread::scope(|scope| {
			for (f, s) in files.iter().zip(&sealed) { scope.spawn(move || assert_eq!(open(k, s).as_ref(), Some(f))); }
		});
	}
}
