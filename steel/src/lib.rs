//steel, fuji's safe rust, a crate of its own so it builds, tests and fuzzes with nothing of fuji around it: the scout, fuji's own reader of the formats of the files a collection holds, in tip.rs, and the walker of the storage they sit on, in walk.rs. Both are knowledge of an outside world fuji does not control, gathered case by case, and both are safe rust under the two rules below
#![forbid(unsafe_code)]//the whole crate is safe rust, checked by the compiler and refused if a line ever says otherwise; tip.rs's essay says what that promise covers for a reader of hostile bytes, and the rules below add what it does not
#![deny(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::arithmetic_side_effects)]//the rules tip.rs's essay states, held by clippy wherever it runs, over the walker as much as the reader: no index that could be out of range, no unwrap, no panic, no arithmetic that could overflow; rustc leaves lints in clippy's namespace alone, so this costs an ordinary build nothing

pub mod tip;//the reader, which says what a file's first bytes are and what the common formats keep in their header
pub mod walk;//the walker, which lists every name in a folder tree with what kind of thing it is
pub mod read;//the reads: a file opened without waiting, and its tip read for the scout
pub mod fit;//the fits: how big a thumbnail is from a picture's size and the beam, the arithmetic fit.js has too
