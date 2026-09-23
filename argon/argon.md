# Argon2id speed test

A letter to the Claude Code session on the Raspberry Pi 4B, asking it to run one small program and record what it measures. The program is `argon.rs`, in this folder beside this letter, with the `Cargo.toml` that builds it. Written on the Mac mini, 2026-09-23. Please add your results to the table at the end, and hand the file back to the user to commit.

## What this measures, and why

**How long Argon2id takes on this machine, at settings fuji may use on every machine it runs on.** Argon2id is a password hash: it turns a passphrase into a key, and it is slow on purpose, so that guessing passphrases is expensive. Its three settings — memory, passes and lanes — all feed into the key it derives, so they cannot vary from one machine to another. Every machine has to use exactly the same values and wait whatever that costs it.

**So the slowest machine fuji supports decides the settings, and the Pi 4B is that machine.** The memory is fixed at 64 MiB, which fits any Raspberry Pi, and the lanes at 4, which matches the Pi 4B's four cores. Both come from RFC 9106's second recommendation, for where memory is tight. The open question is the number of passes. The RFC recommends 3, and fuji is considering many more, because a pass is a linear dial: twice the passes means twice the wait and twice the cost of every guess. The time the Pi takes per pass is what chooses the number.

**The program runs three settings:** the RFC's 64 MiB, 3 passes and 4 lanes, then the same with 30 passes, then with 300. It times each several times and prints the median and the fastest.

**It also checks that machines agree.** The passphrase and salt are fixed, so the same settings must derive the same key on every machine. Each line ends with the first 8 bytes of the key, and the Pi's should match the Mac mini's below exactly. A mismatch would be a finding worth more than any timing.

## How to run it

The Pi already builds fuji, so its Rust toolchain is in place and nothing needs installing.

1. **Build and run it from this folder:**

   ```
   cd argon
   cargo run --release
   ```

   `Cargo.toml` names the one dependency, the `argon2` crate, and `Cargo.lock` pins it to the version the Mac mini used, so both machines run the same code. The build goes into `target/`, which the repository ignores. **The `parallel` feature, which `Cargo.toml` turns on, matters**: it makes the crate compute the four lanes on four threads; it is off by default, and without it four lanes are no faster than one. It changes only the speed, never the key.
2. **Run it on a cool, idle machine**, plugged into its proper power supply. A Pi slows itself down when it gets hot or when its power dips, which would make it look slower than it is. `vcgencmd measure_temp` gives the temperature, and `vcgencmd get_throttled` should answer `throttled=0x0` after the run; if not, record that.
3. **Record the machine**: `cat /proc/device-tree/model` for the model, `free -h` for its memory, `uname -a` for the OS, and `rustc --version`.

The 300-pass setting runs three times, so the whole program takes about a minute on the Mac mini and longer on the Pi.

## Results

**Mac mini** — Mac14,3, Apple M2 (8 cores: 4 performance and 4 efficiency), 16 GB, macOS 15.7.4, rustc 1.98.0, `argon2` 0.6.0 with `parallel`, 2026-09-23:

```
64 MiB, 3 passes, 4 lanes: median 30 ms, fastest 28 ms, 7 runs, key e7e1e4c8edf91ed1
64 MiB, 30 passes, 4 lanes: median 273 ms, fastest 261 ms, 5 runs, key 34b9b86b616f7761
64 MiB, 300 passes, 4 lanes: median 2615 ms, fastest 2603 ms, 3 runs, key efc45fda0a9bc513
```

About 9 ms a pass once the passes dominate. Run to run, the medians move by 10 to 20% — an earlier run of the same program measured 38, 326 and 2968 ms. The keys never move.

**Guesses for the Pi 4B with 4 GB, made on the Mac mini before any run**, to be scored against what it measures:

```
64 MiB, 3 passes, 4 lanes:   about 250 ms, somewhere from 150 to 400
64 MiB, 30 passes, 4 lanes:  about 2.5 s, somewhere from 1.8 to 4
64 MiB, 300 passes, 4 lanes: about 25 s, somewhere from 18 to 40
keys: the same three as the Mac mini
```

The reasoning, about ten times slower per pass: the Pi 4B's four Cortex-A72 cores run at 1.5 GHz, or 1.8 on later boards, and are each several times slower than an M2 performance core at the hashing inside each block. And Argon2id is bound by memory as much as by arithmetic — every block reads another from somewhere in a 64 MiB table far bigger than any cache — where the M2's memory is many times faster than the Pi's LPDDR4. The 4 GB does not matter, since 64 MiB fits any Pi. The one thing that could push the 300-pass run past the range is heat: half a minute with all four cores busy can reach the temperature at which a Pi without a heatsink slows itself down, which `vcgencmd get_throttled` would show.

**Raspberry Pi 4B** — to be filled in: the model, memory, OS, rustc version, temperature and throttling, and the program's three lines.
