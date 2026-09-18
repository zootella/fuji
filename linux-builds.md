# Linux builds

What Fuji builds and publishes for Linux, and the research that decided it — two populations of Linux desktop users, sized and sorted by the package format each one would reach for, running from the two rooms through to a combined leaderboard of the builds those rooms vote for.

**The decision has landed and the builds exist.** Fuji makes the top five of that leaderboard and nothing below them, the `linux` workspace makes them on the Mac in Docker containers, and `linux/README.md` is how you use it. This document is in its third act: what is still open is at the end, and the research in the middle is a candidate for a page on the site rather than for deletion. **That research chapter is left exactly as it was written**, so a caution or an open note inside it is dated rather than current — what became of each one is under "What the build answered" below.

The research chapter — everything from here down to the leaderboards — was read from the web on 2026-09-16. None of it is a measurement taken on hardware, so no machine is implicated and the usual rule about saying which computer a number came from does not bite; what matters instead is which survey a number came from, and every table says. The sections after it describe what was then built, and those are read from the repository.

## Why there are two rooms and not one leaderboard

**There is no single ranking of desktop Linux, because the two large samples that exist measure different crowds and give nearly opposite answers.** Any attempt to average them into one list hides the disagreement, which is the most useful thing about them.

The Stack Overflow Developer Survey is self-reported, 49,000-odd respondents, and its Linux population is dominated by Ubuntu and Debian. The Steam Hardware Survey is real telemetry on a large sample, and its Linux population is dominated by Arch derivatives and by immutable systems that cannot install a conventional package at all. Ubuntu is first in one and seventh in the other.

So this document builds **two rooms of 100 people each**, and names them for the crowd each survey describes:

- **Sysadmins** — people whose Linux is work. Developers, operators, the industry half. Modelled on Stack Overflow.
- **Gamers** — people whose Linux is a machine they chose and tune. The enthusiast and Reddit half. Modelled on Steam.

**The rooms disagree about distributions and agree about hardware.** They also overlap heavily once you stop asking what someone runs and start asking what they can install: a room can be scattered across four distributions that share nothing and still be served by one file. That is why the third column of each table is the package format rather than the distribution, and it is the column the eventual decision turns on. The section below says exactly what that column is claiming.

Each table is ordered by size and stops at 90 people, with the remainder described in prose beneath it rather than tabulated, because a tail of ones and twos invites false precision. **The leaderboards at the end go to full depth anyway**, all 200 people, because the tail rows turn out to matter — they vote almost entirely for formats the larger rows already chose, and they are enough to change which build is first.

## What the third column means

**It is what that row would reach for, not the only thing that would work.** Most people in both rooms can install three or four of these formats. The column names the one they would pick if Fuji offered all of them, which is a deliberately narrower question than coverage and the one that predicts whether somebody says "support X and then I would use it."

**The question is asked about a third-party application, not about software in the distribution's own archives.** Fuji will not be in Debian's or Ubuntu's repositories for the foreseeable future, so the relevant question is: an application your distribution does not package offers you a `.deb`, an AppImage and a Flatpak — which do you take? That gives different answers from "what is your distribution's native format," because on a machine where Flathub is already enabled and the default software centre leads with it, a Flatpak is what people reach for when the software is not in the archive.

**So a row's favorite is Flatpak where Flathub ships enabled and the default software centre surfaces it first** — Fedora and its spins, Linux Mint, Pop!\_OS, Zorin OS, Nobara, and the immutable systems. It stays the native format where Flathub is absent by default and the package manager is the cultural instinct: Ubuntu and Debian with `apt`, RHEL and openSUSE with `dnf` and `zypper`, Arch with an AUR helper.

**This rule is judgement rather than data, and it is the most consequential assumption in the document.** No survey has asked Linux users which format they prefer for third-party software. Applying it moves eight rows across the two tables and changes which build finishes first, so a reader who rejects it should expect a different leaderboard rather than a slightly different one.

## Room 1 — 100 sysadmins

| Count | What they run | Package they want | Running total |
|---|---|---|---|
| 38 | Ubuntu LTS 22.04 / 24.04 / 26.04, x86_64 | `.deb` x86_64 | 38 |
| 16 | Debian stable 12 / 13, x86_64 | `.deb` x86_64 | 54 |
| 11 | Arch / EndeavourOS / Manjaro, x86_64 | AUR PKGBUILD | 65 |
| 8 | Fedora Workstation, x86_64 | Flatpak x86_64 | 73 |
| 6 | RHEL / Rocky Linux / AlmaLinux, x86_64 | `.rpm` x86_64 | 79 |
| 6 | Linux Mint / Pop!\_OS / Zorin OS, x86_64 | Flatpak x86_64 | 85 |
| 3 | openSUSE Leap / Tumbleweed, x86_64 | `.rpm` x86_64 | 88 |
| 3 | NixOS, x86_64 | Flatpak x86_64 | **91** |

**The remaining 9**, with the same column applied: Raspberry Pi OS arm64 used as a real desktop, wanting a `.deb` arm64 (3); Gentoo, Void or Alpine, each wanting a source recipe of its own (2); Fedora Silverblue or Kinoite, wanting a Flatpak (2); and people whose only Linux is a server they reach over SSH, who would install no desktop application at all (2).

**By favorite, over all 100**: `.deb` x86_64 54, Flatpak x86_64 19, AUR PKGBUILD 11, `.rpm` x86_64 9, `.deb` arm64 3, a source recipe 2, and 2 who install nothing.

### How this room was built

The backbone is the Stack Overflow Developer Survey 2025, personal-use figures: Ubuntu 27.8%, Debian 11.4%, Arch 9.7%, Fedora 5.8%, NixOS 3.4%, Pop!\_OS 2.3%, Red Hat 1.8%. Those are multi-select shares of *all* respondents, so they neither sum to 100 nor describe Linux-desktop users specifically; renormalizing over the 62.2% they total gives Ubuntu 44.7%, Debian 18.3%, Arch 15.6%, Fedora 9.3%, NixOS 5.5%, Pop!\_OS 3.7%, Red Hat 2.9%.

**Three adjustments were then made on judgement, and they are the soft part of this table.** The survey names no distribution that most sysadmins would place in the room — Linux Mint, openSUSE, Rocky Linux and AlmaLinux are all absent — so weight was moved into them. Arch and NixOS were cut from their survey shares on the grounds that a room of working operators skews further toward Ubuntu and RHEL than the general developer population does. The net effect is that the `.rpm` family rises from a renormalized 12.2 to 17 and Arch falls from 15.6 to 11. A reader who disagrees with those two moves should adjust them; nothing else in the table depends on them.

**One number in the source is contested.** Two readings of the same 2025 survey gave Debian as 9.8% and as 11.4% for personal use. The higher figure is used above. Substituting the lower one changes the `.deb` family from 66.7% to 65.8% of named distributions, so the shape survives either way, but **the survey has not been read directly and should be** before anything load-bearing rests on this row.

## Room 2 — 100 gamers

| Count | What they run | Package they want | Running total |
|---|---|---|---|
| 21 | SteamOS Holo (Steam Deck), x86_64 | **Flatpak only** — read-only root | 21 |
| 15 | CachyOS, x86_64 | AUR PKGBUILD | 36 |
| 9 | Arch Linux, x86_64 | AUR PKGBUILD | 45 |
| 8 | Linux Mint 22.x, x86_64 | Flatpak x86_64 | 53 |
| 8 | Bazzite, x86_64 | **Flatpak only** — atomic, rpm-ostree | 61 |
| 6 | Manjaro / EndeavourOS / Garuda, x86_64 | AUR PKGBUILD | 67 |
| 4 | Nobara, x86_64 | Flatpak x86_64 | 71 |
| 3 | Fedora 44 KDE Spin, x86_64 | Flatpak x86_64 | 74 |
| 3 | Ubuntu 26.04 LTS, x86_64 | `.deb` x86_64 | 77 |
| 3 | Ubuntu Core 24, x86_64 | **Snap only** | 80 |
| 3 | Pop!\_OS, x86_64 | Flatpak x86_64 | 83 |
| 3 | Older Ubuntu and Mint releases, x86_64 | `.deb` x86_64 | 86 |
| 2 | Debian 13 trixie, x86_64 | `.deb` x86_64 | 88 |
| 2 | Fedora 44 Workstation, x86_64 | Flatpak x86_64 | **90** |

**The remaining 10**, with the same column applied: other immutable systems — Silverblue, Kinoite, ChimeraOS — wanting a Flatpak (3); unidentified or custom distributions, taken to want an AppImage (3); openSUSE, wanting an `.rpm` (2); NixOS, wanting a Flatpak (1); Gentoo, wanting an ebuild (1).

**By favorite, over all 100**: Flatpak x86_64 53, AUR PKGBUILD 30, `.deb` x86_64 8, Snap x86_64 3, AppImage x86_64 3, `.rpm` x86_64 2, a source recipe 1.

### How this room was built

Ten of these fourteen rows are the published Steam Hardware & Software Survey for August 2026, taken almost at face value. Steam reports each distribution as a share *within* its Linux population, so the percentages are already out of 100 and need no renormalizing: SteamOS Holo 21.07%, CachyOS 15.47%, Arch Linux 8.78%, Linux Mint 22.3 8.38%, Bazzite 7.76%, Fedora 44 KDE 3.17%, Ubuntu 26.04 LTS 2.86%, Ubuntu Core 24 2.75%, Debian 13 1.98%, Fedora 44 Workstation 1.94%. Those ten total 74.16%, and they are rows 1 through 5, 8 through 10, and 13 through 14 above.

**The other four rows are constructed** — Manjaro / EndeavourOS / Garuda at 6, Nobara at 4, Pop!\_OS at 3, and older Ubuntu and Mint releases at 3. Steam publishes only a top ten, leaving 25.84% unaccounted for, and these sixteen people are that tail distributed by family in the proportions the published rows imply. They are the softest numbers in either table.

**Two properties of this sample matter more than any single row.** Steam's Linux figure is roughly 4 to 5% of all Steam users, so this is a large sample of a small population. And the Steam Deck is one hardware product sold in the millions, which is why a single distribution holds 21% here and would hold nothing like that in any other survey — SteamOS Holo's share is a hardware fact wearing a distribution's name.

## The leaderboards

Each room's own ranking, then the two together. **These go to full depth** — all 100 in each room, tail included — because the tail is where the ranking changes.

### Room 1 — sysadmins, all 100

| # | Build | Favorite of |
|---|---|---|
| 1 | `.deb` x86_64 | 54 |
| 2 | Flatpak x86_64 | 19 |
| 3 | AUR PKGBUILD | 11 |
| 4 | `.rpm` x86_64 | 9 |
| 5 | `.deb` arm64 | 3 |
| 6 | Source recipe — Gentoo, Void, Alpine | 2 |
| — | Install nothing at all | 2 |

### Room 2 — gamers, all 100

| # | Build | Favorite of |
|---|---|---|
| 1 | Flatpak x86_64 | 53 |
| 2 | AUR PKGBUILD | 30 |
| 3 | `.deb` x86_64 | 8 |
| 4 | Snap x86_64 | 3 |
| 5 | AppImage x86_64 | 3 |
| 6 | `.rpm` x86_64 | 2 |
| 7 | Source recipe — Gentoo | 1 |

### Both rooms — the top ten of 200

| # | Build | Sysadmins | Gamers | Total |
|---|---|---|---|---|
| 1 | **Flatpak x86_64** | 19 | 53 | **72** |
| 2 | **`.deb` x86_64** | 54 | 8 | **62** |
| 3 | **AUR PKGBUILD** | 11 | 30 | **41** |
| 4 | `.rpm` x86_64 | 9 | 2 | 11 |
| 5 | `.deb` arm64 | 3 | 0 | 3 |
| 6 | Snap x86_64 | 0 | 3 | 3 |
| 7 | AppImage x86_64 | 0 | 3 | 3 |
| 8 | Gentoo ebuild | ~1 | ~1 | ~2 |
| 9 | Void template | ~1 | 0 | ~1 |
| 10 | Alpine APKBUILD | ~1 | 0 | ~1 |

Two people in Room 1 appear on no row at all: their only Linux is a server reached over SSH, and they would install no desktop application of any kind.

### What the leaderboards say

**No build leads both rooms.** `.deb` x86_64 is first in Room 1 and third in Room 2; Flatpak is first in Room 2 and second in Room 1; the AUR is second in Room 2 and third in Room 1. Every one of the top three is somebody's answer and nobody's universal answer, which is the same disagreement the two surveys started with, surviving all the way to the end.

**The tail decides first place.** Counting only rows of five people or more, the order is `.deb` 54 against Flatpak 51. Counting everybody, it is Flatpak 72 against `.deb` 62. Flatpak gains 21 going to full depth where `.deb` gains 8, so the lead of 3 becomes a deficit of 10 and first place changes hands. The small rows introduce almost no new formats; they pile onto formats the large rows already chose, and they pile onto Flatpak hardest, because the rows between one and four people are Fedora spins, Nobara, Pop!\_OS, the smaller immutables and NixOS. `.rpm` is the other beneficiary, nearly doubling from 6 to 11 once openSUSE is counted in both rooms.

**The top three hold 175 of 200.** Everything from rank 4 down is 11 people or fewer, and everything from rank 5 down is 3 or fewer.

**Ranks 8 through 10 are below the resolution of this exercise.** They come from splitting a three-person Gentoo / Void / Alpine bucket that was constructed rather than read from a survey. They are listed for completeness, not because the data distinguishes them.

**One fact the favorite column structurally cannot show.** 31 of the 200 could not install a `.deb` or an `.rpm` at all — SteamOS Holo and Bazzite at 29 tabulated, with 3 more immutable systems in Room 2's tail. Their favorite reads as Flatpak, which makes them look like a preference when they are a constraint. Every other Flatpak row in these tables is a genuine preference; those 31 are not.

## Architecture, which is the one thing both rooms agree about

**Every tabulated row in both tables is x86_64.** Not one of the 181 is ARM.

- **Debian's popularity-contest**, generated 2026-09-16 13:54:34 UTC, over 289,370 submissions: amd64 281,513 (97.3%), i386 4,833 (1.7%), **arm64 2,061 (0.71%)**, armhf 394 (0.14%), everything else below 200 each. It is opt-in, weekly, Debian proper rather than its derivatives, and it counts servers alongside desktops — so the desktop arm64 fraction is lower than 0.71%, not higher. Raspberry Pi OS does not report here at all.
- **Steam, August 2026** lists every Linux distribution as "64 bit" and breaks processors down only as AMD 66.87% against Intel 33.04%. No ARM vendor appears at any depth. The Steam Deck is an x86_64 machine, which is why a handheld contributes nothing to an ARM count.
- **Raspberry Pi** is the only meaningful ARM desktop population. Over 75 million units sold as of March 2026, 7.6 million shipped in 2025. One industry tally puts desktop use at about 14% of Pi 4 units, the rest headless — **that figure is blog-quality and should be held loosely**, though its direction agrees with everything else here.

**Across 200 people, ARM desktops are 3, all of them in Room 1's untabulated tail.** That number is the single largest correction this research makes to the assumption it started from.

## What Fuji builds today, as a matter of record

Against the combined leaderboard above, Fuji makes ranks 1 through 5 and nothing below them. Four are packages somebody downloads; rank 3 is a recipe an Arch user's own machine follows.

| Rank | Build | Made by | Published as |
|---|---|---|---|
| 1 | Flatpak x86_64 | `inside-flatpak.sh`, wrapping the amd64 `.deb` | `fuji.x86_64.flatpak` |
| 2 | `.deb` x86_64 | the amd64 container, `--bundles deb,rpm` | `fuji.amd64.deb` |
| 3 | AUR PKGBUILD | `linux/aur/PKGBUILD`, proved with `makepkg` | not published yet |
| 4 | `.rpm` x86_64 | the same container and the same run as rank 2 | `fuji.x86_64.rpm` |
| 5 | `.deb` arm64 | the arm64 container, `--bundles deb` | `fuji.arm64.deb` |

Snap and AppImage, ranks 6 and 7 at three people each, are not built and were not attempted.

**The Linux release machine is the Mac, and no Linux box is one.** That was the open logistical question here and it has an answer that removes it: all five are built together in containers against one base image and one lockfile, and `scripts.js` refuses to stage or upload from Linux while saying why. A Linux machine can still clone this repository and build Fuji for itself, which is the `desktop` workspace rather than this subject.

The engineering that came out of the build has homes of its own and is not repeated here. `linux/README.md` is the guide and carries the `debian:12-slim` floor; `linux/build.js` carries the whitelist and the two-layer image-and-container argument; `inside-flatpak.sh` carries why the Flatpak is assembled by hand; `scripts.js` owns the published names.

## What the build answered

Five of this document's open questions were closed by building the thing, and they are recorded closed rather than deleted, because each was a real doubt and a reader arriving at the research above will have the same one.

**What Tauri's bundler makes natively.** `.deb` and `.rpm` are native and come out of one `tauri build --bundles deb,rpm`. The Flatpak is not: `flatpak-builder` cannot run here at all, because bubblewrap installs a seccomp filter that Rosetta rejects, so `inside-flatpak.sh` assembles the bundle with `flatpak build-init` and ordinary shell instead. The AUR is a recipe rather than a bundler output, and `makepkg` in a container proves it works.

**The WebKitGTK and glibc floor, and which machine sets it.** The base image is `debian:12-slim`, so glibc 2.36 is the floor, and it reaches Ubuntu 24.04 LTS, Mint 22.x, Fedora 40 and up, and both generations of Raspberry Pi OS — verified by installing the built packages on clean Debian, Ubuntu 24.04 and Fedora images. Debian 13 as a base would have moved that floor to 2.41 and shut out the current Ubuntu LTS. The build host does set the floor, and choosing the image is how that is controlled rather than suffered.

**A Flatpak needs `--filesystem=host`, and it is flagged.** Confirmed and acted on. Fuji is handed a folder and reads what is in it, so the portal model — one file at a time, through a dialog the application cannot see past — describes a different program. `inside-flatpak.sh` says so where the permission is granted, and the download page tells the reader before they install.

**Two questions the decision outran.** Reading the Stack Overflow survey directly, and sizing Flatpak's reach rather than its exclusive reach, would both sharpen the leaderboard — and neither can now change what is built, because the top five are all built and ranks 6 and 7 are three people each. They are interesting rather than load-bearing, and the AppImage `.desktop` question went the same way when AppImage was not built.

## Open questions

**Publish to the AUR.** `linux/aur/PKGBUILD` is validated on every `pnpm build` and has never been pushed. The download page already tells Arch users Fuji will be there as `fuji-bin`, so this is a promise outstanding rather than an idea. It also wants a decision about who owns the AUR account and what happens at the next version, since `pkgver` is bumped by hand.

**Submit to Flathub.** The bundle Fuji publishes installs from a file; Flathub wants a manifest its own build farm reads, and that manifest does not exist. `inside-flatpak.sh` was written to be the specification for one — its permissions are exactly what a manifest's `finish-args` would say — so the work is transcription plus submission rather than research.

**What a Pi would actually do with Fuji.** `thumbnail.rs` rejects on Linux, so every raster file there takes the page route — the slower of the two by a wide margin according to `canvas.md`. The 3 ARM people in Room 1 are also the people on the weakest hardware taking the most expensive path. Nobody has run Fuji on a Pi, and the arm64 `.deb` is built and published for a machine nothing here has measured.

## Sources

Read 2026-09-16.

- [Steam Hardware & Software Survey, Linux, August 2026](https://store.steampowered.com/hwsurvey/Steam-Hardware-Software-Survey-Welcome-to-Steam?platform=linux) — telemetry, large sample, gaming-skewed. Per-distribution shares within the Linux population, top ten only.
- [Debian popularity-contest](https://popcon.debian.org/) — opt-in, weekly, Debian proper, servers included. The architecture table is the best public number on ARM's share of a deb-installing population.
- [Stack Overflow Developer Survey 2025](https://survey.stackoverflow.co/2025/) — self-reported, 49,000+ respondents, 177 countries. Multi-select, so shares overlap and do not sum. **Not read directly; taken from summaries**, including [FOSS Post's breakdown](https://fosspost.org/developer-os-preference-stack-overflow-survey/) dated 2026-07-10.
- [Flathub: over one million active users](https://docs.flathub.org/blog/over-one-million-active-users-and-growing) — January 2024, runtime-update estimate, self-described as conservative.
- [Tauri 2 distribution documentation](https://v2.tauri.app/distribute/) — the list of supported routes.
- [Tauri issue 9662, libwebkit2gtk-4.0 on Ubuntu 24 and Debian 13](https://github.com/tauri-apps/tauri/issues/9662) — the WebKitGTK version split.
- [Raspberry Pi statistics 2026](https://voxbooster.com/blog/raspberry-pi-statistics-2026/) — unit sales and the desktop-use share. **Blog-quality; the 14% figure is the weakest number cited in this document.**
