# Testing in Rust

How fuji's Rust is tested: what a test is for, where one lives, what shape it takes, what it should reach for, and the tools in the order they earn their place. The constitution's code chapter governs the code a test is written in; this document governs the tests, and it is written for a reader who knows testing from JavaScript and is new to Rust.

## What a test is for

A test suite is good when it makes the software easy to change and hard to break, in that order. Every other property follows from those two, and most bad suites fail the first while appearing to serve the second: they are long, present and complete, they break on every refactor, and they would pass against a program that gave the wrong answers.

The test for a test is the one [Alex Kladov](https://matklad.github.io/2021/05/31/how-to-test.html) gives, which he calls the neural network test. Replace the code under test with an opaque box that produces the right answers. Would the suite still pass? If it would, the suite tests what the code does. If it would not, the suite tests how the code is shaped, and it will have to be rewritten with every change to that shape while catching nothing a user would notice. A test that asserts a function was called, or that a private helper returns a particular structure, or that a list has the length the current loop happens to produce, fails this test. A test that feeds a file and checks what comes out passes it.

[Kent Beck's twelve properties](https://testdesiderata.com/) of a good test are sliders rather than a checklist, and three of them carry the weight here. A test should be *behavioral*: sensitive to changes in what the code does. It should be *structure-insensitive*: unchanged when the code is rearranged. And it should be *predictive*: when the suite passes, the code is fit to ship. The others, isolated, deterministic, fast, writable, readable, automated, specific, composable and inspiring, are the conditions under which the first three stay true over years.

So a suite is seldom one to one with the code. A few lines that decide something subtle, the order a tree is walked in or the byte a hash starts at, earn a large and careful suite, and a hundred lines of wiring earn none. What decides the weight is what the code could encounter and what it would cost to get wrong, never how much code there is.

## Where a test lives

Rust gives three places, and each has a job.

**A test module in the file**, marked so it compiles only under `cargo test`, is the home of most tests. It sits at the bottom of the file it tests, and `use super::*` gives it the file's private items, which is how a pure core can be tested without being made public for the purpose.

```rust
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_zero_size_in_the_header_is_corrupted() {
		let t = scout(&png_header(0, 20));
		assert_eq!((t.corrupted, t.reading.details), (true, "a size outside the format's range"));
	}
}
```

**The `tests` folder beside `src`** holds tests that use the crate only through its public interface, the way another crate would. Each file there is compiled as a crate of its own. Fuji has few of these, because the crates' public surfaces are small and thin, and the interesting behavior is a function or two below them, which the module test reaches.

**A doc test** is an example in a `///` comment that `cargo test` compiles and runs. Fuji does not write them: the comments here are prose at the line's own altitude, not examples, and an example that must compile pulls the comment's attention toward the mechanics of calling.

When a file's test module grows past its code, which the scout's did, it can move to a file of its own beside it, `tip/tests.rs` declared by `mod tests;` under the same `cfg`, keeping its private access. A shared helper for several files' tests, a tree a test builds, a sample's path, goes in one module declared under `cfg(test)` from the crate's root, as steel's `testing.rs` is.

The safe crate's lints apply to its tests, so a test module there opens with the one line that lets a test index, unwrap, add and panic, since a mistake in a test fails the test, which is what a test is for. Nothing lets a test write unsafe code.

## The shape of a good test

**A test is data against the code.** The strongest tests in this repository are tables: a sample file beside the answer its maker's own tools give, a built tree beside the order it should list in, a set of arguments beside the numbers a twin in another language produced for them. One check function runs the table, and a new case is a new row. This is Kladov's second rule, make adding a test trivial, and it is what lets property tests, exhaustive tests and a fuzzer grow from the same soil later: once an input is data, anything can produce it.

```rust
#[test]
fn samples_answer_as_their_makers_made_them() {
	let rows: &[(&str, &str, (u32, u32), f64)] = &[
		("picture.png", "png", (32, 20), 0.0),
		("video.mkv",   "matroska", (32, 20), 1.0),
	];
	let mut wrong = vec![];//every row that disagrees, so one run shows them all
	for &(name, format, raster, seconds) in rows {
		let t = scout_sample(name);
		if (t.format, (t.raster.width, t.raster.height)) != (format, raster) { wrong.push(format!("{name}: {:?} against {:?}", t.format, format)) }
		if (t.seconds - seconds).abs() >= 0.05 { wrong.push(format!("{name}: {} seconds against {seconds}", t.seconds)) }
	}
	assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
```

Collect every disagreement before failing. A table that stops at its first wrong row makes the reader run it ten times to learn ten facts. A table is one test to cargo: its rows are not in the list `cargo test` prints, one row cannot be run by name, and a row that panics rather than disagrees stops the table there. [rstest](https://docs.rs/rstest) and [test-case](https://docs.rs/test-case) make each row a test of its own with a name in the list; fuji takes the loop for how little a new row costs to add and read, knowing the trade.

**A test builds its own world and takes it down.** A folder under the system's temporary directory, named for the test, filled with small files whose contents are their own names so their sizes are known, and removed when the test ends through `Drop`, which runs whether the test passed or panicked. The name is what keeps tests apart: cargo runs the tests of one binary as threads of one process, so they share its id, and two tests that chose one name would meet, each wiping the other's folder as it started or finished. The process id in the name separates two runs in two terminals and nothing more, so a name is unique or the test is wrong. [tempfile](https://docs.rs/tempfile)'s `TempDir` is the crate the world uses for this, picking a name nothing else has and removing it in `Drop`, and it is the move when the hand-built helper is outgrown. Nothing is read from the developer's home folder, nothing depends on what another test left behind, and the pinned sample files under `samples/` are read and never written.

**A test's name is a sentence about the behavior**, in the lowercase words the code uses, so the list `cargo test` prints reads as the specification: `a_link_is_listed_and_never_followed`, `the_pipe_opens_without_waiting_for_a_writer`, `forward_from_a_folder_finds_the_folders_after_it_in_sorted_preorder`. A name that says what function is called, `test_walk_2`, says nothing when it fails.

**An assertion says what was seen.** `assert_eq!` on a tuple compares several facts in one line and prints both sides on failure. Where a bare `assert!` must do, its message carries the values, `"{} by {}", r.x, r.y`, so the failure is specific, in Beck's word: when a test fails, the cause is obvious. The comment on an assertion says why the expected value is what it is, when that is not plain from the arrangement, "by bytes, so capitals before lowercase".

**No test waits on the clock and no test shares state.** Rust runs tests in parallel threads of one process, so a test that changes the current directory, an environment variable or a global changes it for every test running beside it. Time-based tests are flaky by construction. The one place a thread and a timeout belong is proving that a call *does not* block, the open of a named pipe, say: the call runs on a thread of its own, the test waits on a channel for a bounded time, and a late answer is the failure. Kladov's rule for concurrency in tests is "don't do this", and that exception is the shape it takes when the thing under test is the not-blocking.

## What to reach for

**Test at the boundary where behavior is stable.** For a library crate, the functions a caller uses. For a function that walks a tree, the sequence it produces. Never the private helper's intermediate value, which is structure. If a private function needs a test of its own, it is a pure core that deserves one, like the arithmetic of a fit, and the test treats it as a boundary of its own: arguments in, answers out.

**Test what the code could encounter, not what it is.** The questions to ask of any function, in the order they find bugs:

- *Nothing.* An empty file, an empty folder, an empty string, a zero size, a zero deadline.
- *Too much.* A header that claims a billion by a billion, a chunk longer than the file, a tree deeper than a thread's stack, a count that would overflow a `u32`.
- *Missing or refused.* A path that is not there, a folder the user may not read, a file that vanishes between the listing and the open.
- *Lying.* A file whose name says one format and whose bytes say another, a size field that points past the end, a structure that loops back on itself.
- *Odd names.* Quotes and newlines in a file name, a name that is not valid UTF-8, a trailing dot or space on Windows, a dot file.
- *The platform.* A symbolic link, a named pipe, a socket, a device, a cloud placeholder, a reparse point, a case-insensitive file system, the other slash.
- *Stopping.* A stop flag set before the first item, and set from another thread partway through.

Each of these is a row in a table or a tree built for it, and the code's answer to each is a behavior a user will meet. One row meets the harness itself: a test runs on a thread whose stack is 2 MiB, as a command body on the blocking pool does, where a program's main thread has 8, so a test of depth either runs its walk on a thread whose stack size it sets to the one fuji's code will have, or says in its comment which thread it answers for.

**Test contracts across calls.** The most valuable tests are often about a sequence rather than a call. A walk that answers where it stopped promises that continuing from there produces the rest with nothing skipped and nothing repeated: the test walks in steps of one and compares the sequence with one walk in a single step. A reader that answers in two slices promises the same answer as one that saw the whole file: the test cuts the input at several places and compares. These are the tests a refactor is most likely to break and least likely to be caught by anything else.

**Errors are values here, so test them as values.** A command answers `Err` with words, never a panic, and a test checks the words where they carry meaning, `refused.contains("could not open")`, and the kind of failure where the words are the platform's own. In the safe crate nothing may panic, and three things hold that between them, since no one of them can. Clippy's lints refuse the panic sites they know, an index or a slice that could be out of range, an unwrap, an expect, a panic and the arithmetic operators, and they hold only while `cargo clippy` runs: rustc leaves clippy's namespace alone and `cargo test` never runs it, so a change to the safe crate is checked with `cargo clippy` in its folder before the commit. Safe Rust panics in places no lint sees, `split_at` past the end, `copy_from_slice` on two lengths that differ, a capacity that overflows in `with_capacity` or `repeat`, a sum that overflows in a debug build, and a stack overflow or a failed allocation aborts the process without a panic at all; the code holds those by taking no buffer's size and no loop's length from a number the file gave, and the tests and the fuzzer hold them where the code is wrong. So a test there is never written with `#[should_panic]`, and a panic found by the fuzzer is a bug.

**Never call a function that changes the user's machine.** A test may ask the Mac which application opens a kind of file; it never sets one. A test may read the registry under a key of its own and must delete what it wrote; it never touches a key the user owns. A test may read the display's size and the machine's memory; it never changes a setting. Where a function's whole job is such a change, it is tested by the user, by hand, on each machine, and the test suite says so by leaving it out.

**What the user tests, the suite leaves alone.** The constitution is firm that a visual outcome is checked by eyes on the running program and never by a harness: a thumbnail's look, a window's placement, a panel's text. A test may check that a thumbnail came back at 16 by 10 with 640 bytes of pixels; whether it looks right is not a question a test can answer, and a test that pretended to would be the vanity kind.

## The tools, in the order they earn their place

**Example tests** come first and stay. They hold the known cases, the samples, the past bugs with the input that found them, and they are the specification a reader learns the code from.

**Tables** are example tests that stopped repeating themselves. When the third test of a function looks like the first two with different numbers, it becomes a row.

**An expect test** is a table whose answers the tool writes. [expect-test](https://github.com/rust-analyzer/expect-test) and [insta](https://insta.rs/) run the code, compare what came out with the answer kept in the test or in a file beside it, and on request rewrite the kept answer to what came out, so a person reviews a diff rather than typing a dozen fields by hand; Kladov's own suites are built on them. They fit where an answer is a structure with many fields, as the tip's for a sample is, and the typed table stays where the answer is a few numbers a person should derive, since an answer accepted from the code is only as right as the eyes that read the diff.

**A twin in another language** is a table generated rather than typed. Where the same arithmetic is written in Rust and in JavaScript, as the fits are, the strongest test runs the JavaScript over a grid of inputs and writes a file both sides read, so the two are checked across thousands of arguments rather than the dozens a person pins by hand. The file is data, regenerated by hand when the arithmetic changes on both sides at once, and a divergence anywhere in it is a failure that names its row.

**Property tests** generate inputs. [proptest](https://proptest-rs.github.io/proptest/intro.html) runs an invariant over hundreds of generated values and, on a failure, shrinks the input to the smallest case that still fails, which is the part that makes them usable: the counterexample arrives small enough to read. They fit where a function has a contract that holds for every input, round trips above all, a line written then parsed back, a walk in steps against a walk in one, a size turned twice coming back as it was. They complement example tests and never replace them: the examples hold the cases a person knows matter, and the properties search for the ones nobody imagined. A property test that only restates the implementation, computing the expected value with the same formula, proves nothing, which is the same failure as a vanity example test.

**Fuzzing** is property testing with the property "does not crash" and an engine that learns from coverage which inputs reach new code. For anything that reads bytes nobody vouched for it is the strongest tool there is, and the safe crate runs [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html) over its reader, seeded from the samples. A finding is one of three things: a panic, an input that runs past the engine's timeout, and an input that pushes memory past its limit, both limits set on the run with `-timeout` and `-rss_limit_mb`. For a reader of headers the last two matter as much as the first, since a structure that loops back on itself hangs and a length that sizes an allocation takes the machine's memory, and the reader is built so neither can happen; the fuzzer is what shows it. A target may also assert a property rather than only survive, the same answer from two slices as from the whole file, say, and does for hours what the sweep test does for a bounded run. A fuzzer needs a pure function with bytes in and an answer out, which is one more reason to keep compute away from input and output.

**Mutation testing grades the suite.** [cargo-mutants](https://mutants.rs/) copies the crate, plants one small bug at a time, an operator swapped, a function returning a default, and runs the tests expecting them to fail. A mutant that survives is a place where the code can be broken and no test notices: code that is buggy, unnecessary, underspecified, or hard to test, as its author puts it. It is slow, an incremental build and a test run per mutant, so it runs on a crate with a fast suite and no flakes, when the suite is in question rather than on every change. Its report is the list of what to test next, and it is a better list than any person's guess.

**Coverage is a map, not a score.** A coverage tool shows which lines ran during the tests, and that is useful for finding a branch no test reaches. It cannot show whether any test checked what those lines did; a test that calls a function and asserts the result is not empty covers every line and verifies nothing. The number is not a goal, and a suite tuned to raise it is the junior suite from the first section.

## Speed

A suite is fast when input and output stay in the caller and compute in the callee, so that most tests exercise compute on data already in memory. The slow tests are the few that must touch the world: a real folder, a real decoder, a real player. They stay in the suite, since they are the ones that find what the pure tests cannot, and they are kept few and small, a 32 by 20 sample rather than a photograph.

A test that takes seconds is marked `#[ignore = "..."]` with its reason in the attribute, which `cargo test` prints beside the word ignored, and run by asking for it with `cargo test -- --ignored`, never gated behind conditional compilation, which hides it from the type checker as well. `cargo test` prints a line per test, ok or FAILED, and swallows what a passing test printed; `cargo test -- --nocapture` shows it, and `cargo test name` runs the tests whose names contain the word. A test that failed on one platform's run prints its left and right values, and reading them before touching anything is the first move.

## Debug and release

`cargo test` builds the debug profile, where an integer overflow panics, and a release build wraps it, so a sum in copper can pass every test and wrap in the shipped binary. Steel is immune, since its lints forbid the operators and every sum there is a checked call that answers an error. Copper takes the same shape wherever a number came from outside the program, a checked operation rather than an operator, and `cargo test --release` runs the suite once as the shipped binary would when arithmetic there has changed.

## Platforms

Fuji runs on three operating systems and is developed on five machines, and a test may be true on one and meaningless on another. A test of platform code is gated with the same `#[cfg]` as the code, so the Mac's launch services are tested on the Mac, the registry on Windows, and each has a counterpart proving the other platforms answer that the facility is not there. A crate without Tauri in it can be type-checked for Windows from the Mac, tests included, with `cargo check --tests --target x86_64-pc-windows-msvc`, which catches the compile errors a Windows-only test would otherwise meet on its first visit there; running it is still the Windows box's job. The check runs every dependency's build script for the target too, and today blake3's, beneath ergo in both crates, stops it: for the Windows target it assembles its x86-64 code with Windows' own ml64.exe, which no Mac has. Until blake3 is built another way, its pure or prefer_intrinsics feature, a Windows-only test is first compiled on the Windows box.

## When a test is wrong

A test that fails against correct code is wrong about the world, and the fix is to the test, with the lesson in its comment. A sample that was expected to need a second read turned out to fit in the first; the test now says so and builds a file that does not. The code is never bent to make a test pass, and a test is never deleted to make a run green: each is reread, and the one that misstates reality is corrected to state it.

The reverse case is what every test above is written for. A test that fails against changed code and names the behavior that changed has done its job, and the question is whether the change was meant. If it was, the test moves with the behavior, in the same commit, and its name and comment say the new truth.

## The vanity test

The failure to watch for in a growing suite is the test that proves the call ran. The display has at least 640 pixels, the machine has at least a gigabyte, the process started within the hour. Each executes the code, each would pass against almost anything, and together they raise the coverage number while telling nothing a user would feel. Three things keep such a test from misleading: it is called a smoke test in its comment, so nobody mistakes it for behavior; it checks the one fact that would differ if the call were wrong, the pid is this process's own rather than some pid; or it is deleted. A suite is measured by the bugs it would catch, and a test that catches none is a cost in reading time and in the false confidence a long green list gives.
