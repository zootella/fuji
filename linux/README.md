# Linux builds

Fuji's four Linux packages, built on a Mac through Docker.

Fuji is developed on a Mac and its Linux users are somebody else's machines, so the Linux packages have to come from somewhere. This workspace is that somewhere: a few containers, no Linux computer, and nothing to check out on a second machine and remember to keep current. `build.js` carries the reasoning; this file is how you use it.

## Before you start

**Install Docker Desktop and leave it running.** The `docker` command is only a client, so if Docker Desktop is quit everything here fails with "cannot connect to the Docker daemon". `docker desktop status` says whether it is up and `docker desktop start` starts it without opening the window.

In its settings, turn on **Use Rosetta for x86/amd64 emulation**. The x86-64 packages build in an emulated container, and Rosetta is several times faster than the alternative.

Then, from the repository root, `pnpm install` once — it installs every workspace. This one has no dependencies of its own, because everything it does is drive containers whose toolchains live inside them.

## The commands

```
pnpm build        the four packages
pnpm hash         stage them and write the sidecars
pnpm upload       send them to the server
```

That is the whole of it. Publishing a release from this workspace is those three and then a commit, because the sidecars are tracked.

**Three steps rather than one, because each leaves behind a different kind of thing.** `build` writes packages, which are gitignored and disposable. `hash` writes sidecars, which are committed. `upload` puts files on a server, which is the only one of the three you cannot take back — and that is why it stays a separate, deliberate act rather than the tail of something else.

**The first `build` on a new machine takes far longer than the rest**, because it builds the three toolchain images before it builds anything else: Debian carrying Node and Rust, a second copy of it for the other architecture, and a Flatpak image holding the GNOME runtime. The three come to about 11 GB on disk once built. Every build after that is under eight minutes on the Mac mini — 8m12s was measured while a fifth step, since removed, was still part of it — of which the image step is three seconds.

### The four commands underneath

These exist so that `build` is factored rather than one long function. You would rarely type one.

```
pnpm build-images     the three toolchain images
pnpm build-distro     the two .deb files and the .rpm, out of one Tauri run
pnpm build-flatpak    the .flatpak, which needs the x86-64 .deb to exist first
pnpm stage            copy the source a container would get, and stop
```

**`build` runs `build-images` itself, every time**, and that is the answer to a question the old design could not answer: how would you know the toolchain had moved? You could not. Somebody bumps Rust in a `Dockerfile`, you pull it, and nothing tells you. Docker's layer cache makes checking free — three seconds when nothing changed — so the build simply brings its own images up to date and uses them. The only thing you notice is that a build right after such a pull takes longer than usual, with Docker's own output showing layers being rebuilt where they are normally reused.

It is not called `prepare`, which was the first choice and is a trap: npm and pnpm treat `prepare` as a lifecycle script and run it on every install. A `prepare` in this workspace would mean that cloning Fuji and running `pnpm install` tried to build ten gigabytes of Docker images — and failed the install outright on a machine where Docker was not running.

## Where everything goes

This is the part worth reading once. Four directories, and a file changes its name twice on the way through.

**Source goes in.** Staging copies four things out of the repository root into `linux/.stage/`:

```
linux/.stage/package.json
linux/.stage/pnpm-workspace.yaml
linux/.stage/pnpm-lock.yaml
linux/.stage/desktop/
```

Nothing else. Not `.git`, not the root `.env`, not `node_modules`, not `src-tauri/target`. That comes to about 2 MB against 15 GB for the real `desktop/` folder. The list is stated positively — a whitelist, not a set of exclusions — because a whitelist fails closed, and the thing it must never let through is the `.env` holding the deploy credentials.

`.stage` is wiped and rebuilt by every run, and it is mounted **read-only** into the container, so a build can never write back into your working tree.

**Packages come out.** Each container writes to `linux/release/`, under the filename its own build chose:

```
linux/release/Fuji_0.1.0_arm64.deb          from  build-distro   (arm64 container, runs native on Apple silicon)
linux/release/Fuji_0.1.0_amd64.deb          from  build-distro   (amd64 container, emulated)
linux/release/Fuji-0.1.0-1.x86_64.rpm       from  build-distro   (same amd64 container, same run)
linux/release/Fuji_0.1.0_x86_64.flatpak     from  build-flatpak  (wraps the amd64 .deb above)
```

The `0.1.0` in those names is read from `desktop/src-tauri/tauri.conf.json`, which is the one place Fuji's version is written.

**`pnpm hash` stages them under their published names**, and writes a sidecar beside each — a small JSON file holding the filename, version, architecture, byte count, SHA-256 and build date:

```
linux/release/fuji.arm64.deb            ← Fuji_0.1.0_arm64.deb        + fuji.arm64.deb.json
linux/release/fuji.amd64.deb            ← Fuji_0.1.0_amd64.deb        + fuji.amd64.deb.json
linux/release/fuji.x86_64.rpm           ← Fuji-0.1.0-1.x86_64.rpm     + fuji.x86_64.rpm.json
linux/release/fuji.x86_64.flatpak       ← Fuji_0.1.0_x86_64.flatpak   + fuji.x86_64.flatpak.json
```

**A published name never carries a version, and carries an architecture wherever Fuji ships more than one.** That is what makes a link permanent: the same URL is overwritten on every release, so anything anyone posts keeps handing people the current build. A versioned filename would instead pin whatever version was current the day it was shared, and go quietly stale. Which version a file is belongs on the download page, which reads it from the sidecar.

The architecture token is each ecosystem's own word — `amd64` for Debian, `x86_64` for RPM and Flatpak — rather than one spelling normalized across all of them, because a Debian user and a Fedora user each expect their own.

**Every Linux name states its architecture, including the two formats with only one build today.** Calling the ARM one `fuji.deb` is the tempting mistake: beside `fuji.amd64.deb` a bare name reads like the ordinary choice while being the rarer one, which trips up the majority. Keeping every name explicit also means none has to be renamed, and no link broken, when an aarch64 Flatpak or an ARM rpm turns up. `fuji.dmg` and `fuji.exe` keep bare names because Fuji ships one architecture each there by decision, and because both are already published and linked.

**`pnpm upload` sends the pairs** — each package followed by its sidecar, package first so the page never fetches a hash for a file still arriving. It is `scp` as a restricted account that can write the downloads directory and nothing else; `../.env` holds the address and the key path.

**What git keeps.** The packages are gitignored and the sidecars are committed, so the repository keeps a dated record of what hash each release had. `.stage/` is ignored entirely.

## The four packages, and what each is for

| What | For | Notes |
|---|---|---|
| `.deb` arm64 | Raspberry Pi and other ARM Debian machines | published as `fuji.arm64.deb` |
| `.deb` amd64 | Debian, Ubuntu, Mint, Pop!_OS, Zorin on x86-64 | the largest single audience |
| `.rpm` x86_64 | Fedora, RHEL, Rocky, AlmaLinux | same build run as the amd64 deb |
| `.flatpak` x86_64 | any distribution, sandboxed, and what Arch users take | the only one that works on SteamOS and Bazzite |

## Why it is built this way

**The pipe flushes clean.** An image is the toolchain and holds no source and no secrets. A container is one build: it is handed the source, makes a package, and is destroyed. Nothing carries between builds — no warm `target/`, no cargo registry, no `node_modules` from last time. These packages are made a few times a year, not in a daily loop, so a known starting state is worth more than speed.

**`pnpm-lock.yaml` and `Cargo.lock` are what make "clean" mean something.** Both are in the whitelist and both are enforced, so the versions that land are the ones the Mac and the Windows box already build with. Without them a fresh container would resolve every caret range against whatever the registry served that morning.

**The base image is `debian:12-slim`, and that is a decision rather than a default.** A binary is compatible with its build machine's glibc and every later one, never an earlier one, so the base sets a floor on who can run the result. Debian 12's glibc 2.36 reaches Debian 12 and 13, Ubuntu 24.04 LTS and 26.04, Mint 22.x, Fedora 40 and up, and both the bookworm and trixie generations of Raspberry Pi OS. Debian 13 would have moved that floor to 2.41 and shut out the current Ubuntu LTS. Reaching back costs nothing at the newer end.

**Rust is pinned** in `desktop/src-tauri/rust-toolchain.toml`, which governs this container and the Mac and Windows builds alike, so the three cannot drift apart.

## Things that will confuse you once

**Editing an `inside-*.sh` script takes effect immediately.** Those are mounted into the container at run time, not baked into the image, so there is no image to rebuild. Editing a `Dockerfile.*` needs no action either: the next `build` rebuilds that image on its own.

**The amd64 build is the slow one** — 4m54s of Rust against 1m40s for arm64 in the same run, because Apple silicon runs arm64 natively and emulates x86-64. `build` runs arm64 first on purpose, so a mistake surfaces in two minutes rather than twenty.

**The Flatpak is not built with `flatpak-builder`.** Bubblewrap installs a seccomp filter, and Rosetta rejects that call, so the usual manifest route cannot build an x86-64 Flatpak on this Mac at all. Since the work is unpacking a `.deb` and placing files rather than compiling, `inside-flatpak.sh` uses `flatpak build-init` and ordinary shell instead, which needs no sandbox. Its comments carry the whole account.

**The Flatpak container runs `--privileged`** for one narrow reason: `flatpak build-export` validates the icon inside bubblewrap, and that needs a namespace an ordinary container cannot make.

**There is no `pnpm reveal` here.** That is a Mac convenience in the `desktop` workspace; everything this one produces is in `linux/release/`, named above.

## Building on Linux instead

You can clone this repository on Ubuntu or Raspberry Pi OS and build Fuji for the machine you are sitting at — that is the `desktop` workspace, not this one:

```bash
pnpm install
cd desktop
pnpm installer
```

What you cannot do there is stage or upload, and `scripts.js` will say so. A published package comes from the Mac, where all of them are built together against one base image and one lockfile.
