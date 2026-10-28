# Bike Lock

***Bike Lock*** locks files with *dice words*, a passphrase of words picked by rolling dice, which the user chooses and memorizes, and it opens them again when the user types those words. It works like the lock on a bike: the combination is the user's own, and nobody else holds a copy. Underneath sit five published designs, AES-256-GCM, HKDF, Argon2id, Padmé and the dice words themselves, configured so that guessing six dice words costs a sophisticated adversary five quadrillion dollars, while opening a 3 MB file costs the user one millisecond.

## Locking and opening

The user chooses files to lock and types their dice words. Fuji replaces each file with its locked copy, under its own name with a `.bl` mark before the extension, so `file-name.ext` becomes `file-name.bl.ext`, erasing the original as [Fuji's Delete](/erase) does.

When Fuji meets a locked file, it asks for dice words once. A pause of around 25 milliseconds on a modest Mac or PC turns them into a key, and from then on every file locked with those words opens as fast as Fuji can read it, on the contact sheet and on the light table. Fuji decrypts in memory, and the file on disk stays locked.

A user with two sets of dice words, one for some files and one for others, types each set once. Fuji keeps every key that has opened a file for as long as it runs, tries them in turn starting with the one that worked last, and asks again only when none of them opens the file in front of it. Dice words that open nothing are discarded at once. Fuji never writes dice words or keys to the disk, and they leave memory when Fuji quits.

A locked file can be renamed, moved and copied like any other, as long as its name keeps the `.bl` mark. Other programs see a `.ext` file they cannot read.

## Designing a good cryptographic system

Cryptographers assemble a system from *primitives*, small published algorithms that each do one job, such as turning a passphrase into a key, encrypting bytes or sealing them, and that the field has studied for years. A primitive earns its strength from everyone who tried to break it and failed, so a good design invents none of its own. Bruce Schneier put the reason in [a memo to amateur cipher designers](https://www.schneier.com/crypto-gram/archives/1998/1015.html) in 1998: "Anyone, from the most clueless amateur to the best cryptographer, can create an algorithm that he himself can't break." ***Bike Lock*** invents no cryptography.

Every designer chooses from the same shelf of published primitives, and the shelf holds designs of every age, many of them chosen in open contests. The U.S. National Institute of Standards and Technology, NIST, chose AES from fifteen designs in a process that ended in October 2000, and the Password Hashing Competition chose Argon2 from 24 candidates in July 2015. A primitive with decades of failed attempts behind it is the strongest thing on the shelf, and a newer one earns its place by doing a job the older ones cannot. ***Bike Lock*** takes from across those years: AES from 2001, HKDF from 2010, Argon2 from 2015 and Padmé from 2019.

The joins between primitives are where systems fail, so a good design keeps them few and plain. "The worst enemy of security is complexity," Schneier wrote in [A Plea for Simplicity](https://www.schneier.com/essays/archives/1999/11/a_plea_for_simplicit.html) in 1999. GCM, the mode of encryption ***Bike Lock*** uses, shows what a careless join costs. It takes a *nonce*, a number used once, that must never repeat under the same key, and in 2016 Hanno Böck and colleagues [found 184 HTTPS servers repeating nonces](https://eprint.iacr.org/2016/475), which "fully breaks the authenticity of the connections." ***Bike Lock*** uses each primitive the standard way its designers describe, adds no part that does not answer a need, and gives every file a key of its own, so no key ever meets a second nonce.

A good design also keeps nothing secret but the key. Auguste Kerckhoffs set that rule down in 1883, second in a list of six principles in [La cryptographie militaire](https://www.petitcolas.net/kerckhoffs/), here in his French and then in English:

> 1° Le système doit être matériellement, sinon mathématiquement, indéchiffrable ;\
> 2° Il faut qu’il n’exige pas le secret, et qu’il puisse sans inconvénient tomber entre les mains de l’ennemi ;\
> 3° La clef doit pouvoir en être communiquée et retenue sans le secours de notes écrites, et être changée ou modifiée au gré des correspondants ;\
> 4° Il faut qu’il soit applicable à la correspondance télégraphique ;\
> 5° Il faut qu’il soit portatif, et que son maniement ou son fonctionnement n’exige pas le concours de plusieurs personnes ;\
> 6° Enfin, il est nécessaire, vu les circonstances qui en commandent l’application, que le système soit d’un usage facile, ne demandant ni tension d’esprit, ni la connaissance d’une longue série de règles à observer.
>
> 1\. The system must be undecipherable in practice, if not mathematically;\
> 2\. It must not require secrecy, and it must be able to fall into the enemy's hands without harm;\
> 3\. The key must be one that can be shared and remembered without the help of written notes, and changed or replaced whenever the correspondents choose;\
> 4\. It must work over the telegraph;\
> 5\. It must be portable, and handling or running it must not take more than one person;\
> 6\. Finally, given the circumstances that call for it, the system must be easy to use, demanding neither mental strain nor knowledge of a long series of rules to follow.
>
> Auguste Kerckhoffs, [La cryptographie militaire](https://www.petitcolas.net/kerckhoffs/), *Journal des sciences militaires*, January 1883, page 12.

A design whose strength depends on staying hidden fails the day someone reads it. This article describes ***Bike Lock*** to the byte, Fuji's source code is public, and the one secret in the whole system is the user's dice words, a key remembered, as the third principle asks, without the help of written notes.

And a good design gets tested against an opponent before anyone trusts it. A *tabletop exercise* plays out a scenario on paper: in the words of [NIST's glossary](https://csrc.nist.gov/glossary/term/tabletop_exercise), "a facilitator initiates the discussion by presenting a scenario and asking questions based on the scenario." For a cryptographic system, the scenario is a sophisticated adversary holding the encrypted files, with essentially unlimited time, money, knowledge and expertise, and the questions are what it can do, what that costs, and how long it takes. This article ends with one.

## Five building blocks

***Bike Lock*** fits its five building blocks together in three steps. The dice words go in at the top, and a sealed file comes out at the bottom:

```
dice words + bike lock salt  →  Argon2id  →  dice key            once a session    26 ms
dice key   + file salt       →  HKDF      →  file key            once a file       0.27 µs
file key   + nonce           →  AES-GCM   →  ciphertext and tag  for 3 MB          1 ms
```

The times are medians on an Apple M2 Mac mini, from the RustCrypto project's crates that Fuji builds on. The first step is slow by design and runs once a session. The other two are fast and run for every file. The slow step stands between the sophisticated adversary and every guess, and the fast steps are all the user waits for after it.

The sections below take the five building blocks from the inside out, starting with the cipher at the core and working outward from the oldest design to the newest, and ending with the dice words, the one part the user holds. Padmé, which pads the file before the cipher seals it, is the one block the diagram leaves out.

### AES-256-GCM, 2001 and 2007

At the core, AES-256-GCM encrypts the file and seals it. [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard), the Advanced Encryption Standard, is Rijndael, by Joan Daemen and Vincent Rijmen, which NIST published as FIPS 197 in November 2001, here with its longest key, 256 bits. It runs in [*Galois/Counter Mode*](https://en.wikipedia.org/wiki/Galois/Counter_Mode), designed by David McGrew and John Viega and specified by NIST in [SP 800-38D](https://csrc.nist.gov/pubs/sp/800/38/d/final) in 2007. Counter mode turns a 256-bit key and a 12-byte nonce into a stream that encrypts the file into *ciphertext*, and a hash over a Galois field turns the ciphertext into a 16-byte *tag*. Each file gets a key of its own, which the next section makes, and 12 new random bytes for its nonce, which Fuji stores at the front of the locked file.

The tag is the seal. Only the right key over unchanged ciphertext reproduces it, so one comparison answers both of Fuji's questions when it opens a file: were these the right dice words, and has any byte changed? A wrong key passes by chance once in 2^128 tries. Fuji checks the tag over the whole ciphertext before it decrypts a single byte, so nothing from a wrong key or an altered file ever reaches the screen.

With the AES instructions built into current processors, which the crates find and use on their own, a 3 MB file opens in a millisecond, and a wrong key on the same file fails in under half of one, in the same measurements as the diagram above.

### HKDF, 2010

[*HKDF*](https://en.wikipedia.org/wiki/HKDF), the HMAC-based key derivation function, turns one key into many. Hugo Krawczyk set out its design in [Cryptographic Extraction and Key Derivation: The HKDF Scheme](https://eprint.iacr.org/2010/264) in 2010, and published it with Pasi Eronen as [RFC 5869](https://www.rfc-editor.org/rfc/rfc5869.html) in May of that year. Fuji gives it the dice key, which the next section makes from the dice words, and the file's own 16 random bytes of *file salt*, and it returns a 32-byte file key in about a quarter of a microsecond. Every file gets a key of its own: the same file locked twice comes out as two unrelated runs of bytes, and nothing in two locked files shows that the same dice words locked them. The file salt is not a secret, and it sits in the clear at the front of the locked file.

### Argon2id, 2015

Turning dice words into a key takes a hash built to be expensive. [*Argon2*](https://en.wikipedia.org/wiki/Argon2), by Alex Biryukov, Daniel Dinu and Dmitry Khovratovich of the University of Luxembourg, won the [Password Hashing Competition](https://www.password-hashing.net/), an open contest that ran from 2013 to 2015, and became [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html) in September 2021.

Argon2 is *memory-hard*: every guess has to fill a large table in memory and move it through the processor, each block mixed from the one before it and one chosen from earlier in the table, and no shortcut skips the table. A graphics card that runs billions of fast hashes a second can hold only as many Argon2 guesses as its memory has room for tables, and runs them only as fast as its memory can move them. ***Bike Lock*** writes the table's settings down once, at the top of [`steel/src/lock.rs`](https://github.com/zootella/fuji/blob/main/steel/src/lock.rs), and hands them to RustCrypto's `argon2` crate:

```rust
//rfc 9106's second recommended option, from its section 4, as the rfc gives it
pub const MEMORY: u32 = (64 * size::MB / size::KB) as u32;//64 MiB in the KiB argon2 counts in: a table of 65,536 blocks of 1 KiB
pub const PASSES: u32 = 3;//fill the table, then rewrite every block twice more
pub const LANES: u32 = 4;//four rows of the table, filled side by side on four threads
pub const KEY: usize = 32;//the key argon2id makes, the option's 256-bit tag size
pub const SALT: usize = 16;//the option's 128-bit salt, and the size of each file's salt too

pub fn key(passphrase: &str, label: &str) -> Option<Data> {
	let salt = sha256(&Data::from_text(label)).clip_view(0, SALT);
	let settings = Params::new(MEMORY, PASSES, LANES, Some(KEY)).ok()?;
	let mut out = [0u8; KEY];
	Argon2::new(Algorithm::Argon2id, Version::V0x13, settings).hash_password_into(Data::from_text(passphrase).array(), salt.array(), &mut out).ok()?;
	Some(Data::from_array(&out))
}
```

`Data::from_text` turns the dice words into UTF-8 in Unicode Normalization Form C, so the same words typed two ways, an accent as one character or as a letter and a combining mark, make the same dice key.

Argon2 comes in three variants. *Argon2d* chooses which earlier block to mix from by the data itself, which gives graphics cards the least to work with. *Argon2i* chooses by position alone, so the pattern of its memory reads reveals nothing about the passphrase to anyone timing them. *Argon2id* runs the first half of its first pass as Argon2i and the rest as Argon2d, and both of the options RFC 9106 recommends name it. ***Bike Lock*** runs the second option exactly as the RFC gives it in section 4:

> If much less memory is available, a uniformly safe option is Argon2id with t=3 iterations, p=4 lanes, m=2^(16) (64 MiB of RAM), 128-bit salt, and 256-bit tag size. This is the SECOND RECOMMENDED option.
>
> [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html), section 4, September 2021.

The settings are part of the key: the same dice words under different settings make a different key, so every machine has to run the same ones. The first recommended option needs 2 GB of memory free; the second needs 64 MB, which fits every machine Fuji runs on, a Mac, a Windows gaming PC or a Linux desktop, down to a Raspberry Pi. Its four lanes run on four cores, which every one of them has.

Argon2id takes a *salt* beside the dice words, a value mixed in so that a table of guesses computed for some other system is useless against this one. The *bike lock salt* is the first 16 bytes of the [SHA-256](https://en.wikipedia.org/wiki/SHA-2) hash of the nine ASCII bytes "bike lock", which Fuji passes as the label in the code above, and which make the 128 bits the RFC's option names. One salt for every user means the same dice words make the same dice key on every machine and in every year, so a single short pause serves every locked file in every folder. RFC 9106 asks for a salt unique to each password, and the tabletop below counts what one shared salt costs against dice words: nothing.

### Padmé, 2019

Encryption hides what a file holds and not how big it is: the size is one of the few things a locked file leaves in view, and an exact byte count says more about a file than it needs to. So ***Bike Lock*** pads every file with *Padmé*, from "[Reducing Metadata Leakage from Encrypted Files and Communication with PURBs](https://petsymposium.org/popets/2019/popets-2019-0056.php)", by Kirill Nikitin, Ludovic Barman, Wouter Lueks, Matthew Underwood, Jean-Pierre Hubaux and Bryan Ford, in the *Proceedings on Privacy Enhancing Technologies* in 2019.

Padmé treats a length like a floating-point number and allows its mantissa no more bits than its exponent takes to write, rounding everything below them up. The paper's Algorithm 1 is five lines, and the Rust follows it line for line, with every step checked:

```rust
pub fn padme(length: u64) -> Option<u64> {
	if length < 2 { return Some(length) }//no exponent to speak of; a sealed file's length field alone makes 8
	let e = length.ilog2();//the length's exponent
	let s = e.ilog2().checked_add(1)?;//the bits it takes to write e
	let z = e.checked_sub(s)?;//the low bits to round away
	let m = 1u64.checked_shl(z)?.checked_sub(1)?;//a mask of z ones
	Some(length.checked_add(m)? & !m)
}
```

***Bike Lock*** runs it on L, the file's length plus the 8 bytes that record that length. A file of 3,000,000 bytes has an L of 3,000,008, which rounds up to 3,014,656, the next multiple of 64 KB: 14,648 bytes of zeros, about half a percent more. Every file from 2,949,113 to 3,014,648 bytes, 65,536 sizes in all, locks to that same size.

The paper bounds the cost as "always less than 12%," and Fuji's tests find the worst case exactly: a length of 129, a 121-byte file and its length field, rounds up to 144, 11.6% more, a little past the 9 bytes to 10 that the paper's text names as its worst. The cost falls as files grow, to under 3.2% for anything from 64 KB to 4 GB. What a padded size still shows is the order of magnitude and a few leading bits, which the paper bounds at O(log log M) bits for files up to M bytes. The zeros go inside the ciphertext, and the tag covers them.

### Dice words, 1995

Dice words are a passphrase chosen by dice instead of by a person. Roll five dice, read them as a five-digit number, and find that number in the Electronic Frontier Foundation's [long wordlist](https://www.eff.org/dice) of 7,776 words; repeat for each word. Every word adds 12.9 bits, and six words come to 77.5 bits, one of the 221,073,919,720,733,357,899,776 six-word passphrases the EFF's page counts.

Arnold Reinhold published the method as [Diceware](https://theworld.com/~reinhold/diceware.html) on Usenet's *sci.crypt.research* on August 1, 1995, and Joseph Bonneau of the EFF [published the long list](https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases) in July 2016, the same size as Reinhold's. The EFF's advice is "For most applications, we suggest making a six-word passphrase," and Reinhold recommends "a minimum of six words for use with GPG, wireless security and file encryption programs," with seven or more for the highest-value uses.

Fuji recommends six dice words, and seven for files meant to stay locked for decades. Rolling the dice matters as much as the number of words. Dice make every word on the list equally likely, so a guesser has no better place to start than anywhere else. A person choosing words favors familiar ones, and guessing programs try familiar words first.

## The locked file

```
file-name.bl.ext

16 bytes    file salt
12 bytes    nonce
            ┌─ ciphertext ─────────────────────┐
 8 bytes    │ the file's length, n             │
 n bytes    │ the file                         │
 p bytes    │ zeros, to the size Padmé chooses │
            └──────────────────────────────────┘
16 bytes    tag
```

A locked file is Padmé's padded size plus 44 bytes. The file salt and the nonce are random, the tag reveals nothing without the key, and everything between them is ciphertext indistinguishable from random bytes, the true length and the padding included. The `.bl` mark in the name tells Fuji the file is locked, so nothing inside the file says so.

The length is *little-endian*, an unsigned 64-bit number with its lowest byte first. Every processor Fuji runs on, Apple silicon, Intel and AMD, and the ARM chips in a Raspberry Pi, keeps numbers in that order, so writing and reading the length is a straight copy with no bytes to swap.

Opening reads those bytes as bytes nobody vouched for, and checks every split before it trusts one:

```rust
pub fn open(key: &Data, sealed: &Data) -> Option<Data> {
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
```

A wrong key, a changed byte, and bytes too short to be a locked file all come back as the same nothing.

## Where the code lives

***Bike Lock*** is written in Rust, in [`steel/src/lock.rs`](https://github.com/zootella/fuji/blob/main/steel/src/lock.rs), in steel, the crate of Fuji's code that the compiler holds to safe Rust: it forbids unsafe code, and denies any index that could fall outside its slice, any unwrap or panic, and any sum that could overflow, as [Security](/security#four-places) describes. Opening a locked file starts by reading bytes nobody vouched for, the file salt, the nonce and the stored length, and reading such bytes is the job steel exists for. Every step after that is arithmetic on bytes in memory.

The primitives come from the RustCrypto project's crates, `argon2`, `hkdf`, `sha2` and `aes-gcm`, and the AES-GCM crate has had a security audit, [by NCC Group in 2020](https://www.nccgroup.com/research/public-report-rustcrypto-aesgcm-and-chacha20pluspoly1305-implementation-review/), which [the crate's documentation](https://docs.rs/aes-gcm/latest/aes_gcm/) reports had no significant findings. Steel holds only general primitives that any program could use. Fuji's page, the JavaScript that runs its windows and decides everything, holds what makes them ***Bike Lock***, the `.bl` mark and the text of the salt, and decides which files to lock and which keys to try.

The tests start from what the user will do. A file locked today has to open on another machine and in every later version of Fuji, so the tests open locked files that a separate implementation made, in Python, with the dice key from [the reference implementation](https://github.com/P-H-C/phc-winner-argon2) of Argon2 and the rest from the [cryptography](https://cryptography.io/) package, and a change to the format that would lock anyone out fails a test first. They lock a folder under two sets of dice words and open each file with whichever key fits, while a typo opens nothing. They put locked files through what a disk or a download does to files, cut short, filled with zeros where the rest never arrived, a flipped bit, bytes appended, and every one opens to nothing rather than to something wrong. They give the mark to files that were never locked, every format among the samples Fuji's file reader is tested on, and those open to nothing too. And they hold Padmé to the paper, seal and open files of every size from empty to 3 MB, and open many at once under one key.

## The tabletop

The user locks their files with six dice words, and the files then leave the user's hands. The sophisticated adversary holds every locked file, knows ***Bike Lock*** down to the byte from this page and Fuji's public source, and has essentially unlimited time, money, knowledge and expertise. The questions are what the sophisticated adversary can do with all that, what it costs, and how long it takes.

### Guessing

Through the cryptography, the only way to a file is to guess the dice words, and every guess pays for a full Argon2id run. A run fills its 64 MB table and passes over it three times, reading and writing about half a gigabyte. NVIDIA's H100, a data-center graphics card, [moves 3.35 terabytes a second](https://www.nvidia.com/en-us/data-center/h100/) through its memory, so a card that did nothing but feed Argon2id would top out near six thousand guesses a second. The tabletop grants the sophisticated adversary a million such cards, every one at that ceiling, at a dollar per card-hour. On average, finding the dice words takes:

| Dice words | Bits | Average cost | Average time on a million cards |
|---|---|---|---|
| 4 | 51.7 | $85 million | 3½ days |
| 5 | 64.6 | $660 billion | 75 years |
| 6 | 77.5 | $5 quadrillion | 580,000 years |
| 7 | 90.5 | $40 quintillion | 4.5 billion years |

Seven dice words take about as long as the Earth has existed, which the U.S. Geological Survey puts at [4.54 billion years](https://pubs.usgs.gov/gip/geotime/age.html).

### Waiting

The sophisticated adversary can keep the files and wait for cheaper hardware. Grant a thousandfold drop in the cost of a guess, from custom chips or from decades of progress, and five dice words fall in 27 days for $660 million, six still cost $5 trillion and 580 years, and seven stay out of reach at $40 quadrillion. Six dice words hold today with room to spare, and seven hold for the long term.

### What Argon2id buys

A single RTX 4090 graphics card runs [22.1 billion SHA-256 guesses a second](https://hashcat.net/forum/post-63595.html) in hashcat, the password-guessing program. Behind a plain hash like that, the same million cards would find six dice words in about two months, for $1.4 billion. Argon2id's memory turns that into 580,000 years and $5 quadrillion, the strength of nearly two more dice words on every passphrase.

### What holds

- **The file key** is 256 bits, and no search reaches it; the only road to it runs through the dice words.
- **The bike lock salt** lets a table of guesses computed once serve against every ***Bike Lock*** user at once, which is the price of one salt for everyone. A table covers only dice words that people chose themselves, and a random six from the EFF's list is one of 221 sextillion.
- **The tag** makes a locked file with even one changed byte refuse to open.
- **Padmé** leaves a locked file's size showing only its order of magnitude and a few leading bits.

Against six dice words, no budget wins by guessing: five quadrillion dollars and 580,000 years. Four dice words fall in days, five hold today but not for decades, six hold, and seven hold for about as long as the Earth has existed.

## Sources

Every address below was fetched on October 28, 2026.

- Biryukov, Alex, Daniel Dinu, Dmitry Khovratovich, and Simon Josefsson. "[Argon2 Memory-Hard Function for Password Hashing and Proof-of-Work Applications](https://www.rfc-editor.org/rfc/rfc9106.html)." RFC 9106, September 2021.
- Böck, Hanno, Aaron Zauner, Sean Devlin, Juraj Somorovsky, and Philipp Jovanovic. "[Nonce-Disrespecting Adversaries: Practical Forgery Attacks on GCM in TLS](https://eprint.iacr.org/2016/475)." Cryptology ePrint Archive, Paper 2016/475, 2016.
- Bonneau, Joseph. "[Deep Dive: EFF's New Wordlists for Random Passphrases](https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases)." *Deeplinks*, Electronic Frontier Foundation, July 19, 2016.
- Dworkin, Morris. [*Recommendation for Block Cipher Modes of Operation: Galois/Counter Mode (GCM) and GMAC*](https://csrc.nist.gov/pubs/sp/800/38/d/final). NIST Special Publication 800-38D. Gaithersburg, MD: National Institute of Standards and Technology, November 2007.
- Electronic Frontier Foundation. "[EFF Dice-Generated Passphrases](https://www.eff.org/dice)."
- Fernick, Jennifer. "[Public Report – RustCrypto AES/GCM and ChaCha20+Poly1305 Implementation Review](https://www.nccgroup.com/research/public-report-rustcrypto-aesgcm-and-chacha20pluspoly1305-implementation-review/)." NCC Group, February 26, 2020.
- Fuji. [*Fuji*](https://github.com/zootella/fuji). Source repository on GitHub, from which this page quotes `steel/src/lock.rs`.
- Kerckhoffs, Auguste. "[La cryptographie militaire](https://www.petitcolas.net/kerckhoffs/)." *Journal des sciences militaires* 9 (January 1883): 5–38, and (February 1883): 161–191. Transcribed at petitcolas.net; the six principles are on page 12, and the English of them on this page is Fuji's own translation.
- Krawczyk, Hugo. "[Cryptographic Extraction and Key Derivation: The HKDF Scheme](https://eprint.iacr.org/2010/264)." Cryptology ePrint Archive, Paper 2010/264, 2010.
- Krawczyk, Hugo, and Pasi Eronen. "[HMAC-based Extract-and-Expand Key Derivation Function (HKDF)](https://www.rfc-editor.org/rfc/rfc5869.html)." RFC 5869, May 2010.
- National Institute of Standards and Technology. "[Tabletop Exercise](https://csrc.nist.gov/glossary/term/tabletop_exercise)." Computer Security Resource Center glossary, from NIST SP 800-84.
- Nikitin, Kirill, Ludovic Barman, Wouter Lueks, Matthew Underwood, Jean-Pierre Hubaux, and Bryan Ford. "[Reducing Metadata Leakage from Encrypted Files and Communication with PURBs](https://petsymposium.org/popets/2019/popets-2019-0056.php)." *Proceedings on Privacy Enhancing Technologies* 2019, no. 4: 6–33. Also [on arXiv as 1806.03160](https://arxiv.org/abs/1806.03160).
- NVIDIA. "[H100 Tensor Core GPU](https://www.nvidia.com/en-us/data-center/h100/)." nvidia.com.
- Password Hashing Competition. "[Password Hashing Competition](https://www.password-hashing.net/)." password-hashing.net.
- Password Hashing Competition. "[phc-winner-argon2](https://github.com/P-H-C/phc-winner-argon2)." The reference C implementation of Argon2, on GitHub.
- Python Cryptographic Authority. "[Welcome to pyca/cryptography](https://cryptography.io/)." Documentation of the cryptography package.
- Reinhold, Arnold G. "[The Diceware Passphrase Home Page](https://theworld.com/~reinhold/diceware.html)."
- RustCrypto. "[aes-gcm](https://docs.rs/aes-gcm/latest/aes_gcm/)." Crate documentation, docs.rs.
- Schneier, Bruce. "[A Plea for Simplicity](https://www.schneier.com/essays/archives/1999/11/a_plea_for_simplicit.html)." schneier.com, November 19, 1999.
- Schneier, Bruce. "[Memo to the Amateur Cipher Designer](https://www.schneier.com/crypto-gram/archives/1998/1015.html)." *Crypto-Gram*, October 15, 1998.
- slyexe. [Benchmark of an RTX 4090 with hashcat 7.0.0](https://hashcat.net/forum/post-63595.html). hashcat forum, August 6, 2025.
- U.S. Geological Survey. "[Geologic Time: Age of the Earth](https://pubs.usgs.gov/gip/geotime/age.html)."
- Wikipedia. "[Advanced Encryption Standard](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard)."
- Wikipedia. "[Argon2](https://en.wikipedia.org/wiki/Argon2)."
- Wikipedia. "[Galois/Counter Mode](https://en.wikipedia.org/wiki/Galois/Counter_Mode)."
- Wikipedia. "[HKDF](https://en.wikipedia.org/wiki/HKDF)."
- Wikipedia. "[SHA-2](https://en.wikipedia.org/wiki/SHA-2)."
