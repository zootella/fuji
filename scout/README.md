# Scout

Fuji's own reader of a file's first bytes. Given the first 64 KiB of a file and the file's size, it says what format the bytes are and what the common formats keep in their header: the size a picture or a video shows at, its turn, the running time of a sound or a video. When the first bytes say where the rest of an answer lies, a phone's video index after its data or an Ogg's last page, it asks for one more 64 KiB slice at that offset and answers again from both: two reads at most, and never a search of the file. It reads no pixels and no samples.

It is a Rust crate of its own, so it builds and tests here with nothing of Fuji around it. Fuji's Rust depends on it by path, and nothing publishes it.

```
pnpm test    run the reader's tests, pinned to the files in samples/
pnpm fuzz    run libFuzzer on the reader, seeded from samples/, until stopped
```

`pnpm fuzz -- -max_total_time=60` runs it for a minute. Fuzzing needs the nightly toolchain and cargo-fuzz, `rustup toolchain install nightly` and `cargo install cargo-fuzz`.

## The files

```
src/lib.rs    the reader, with its essay at the top and its tests at the end
samples/      small files made by real encoders, each pinned by a test to the answer it must give
fuzz/         the crate cargo-fuzz runs, which keeps what it finds in fuzz/corpus and fuzz/artifacts, both gitignored
```

## How Fuji uses it

`desktop/src-tauri/src/scout.rs` is the command the page calls with a path. It reads the start of the file, and the second slice when the reader asks for one, and hands them to the reader here; `desktop/src/scout.js` wraps the command for the page.
