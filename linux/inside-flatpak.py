#!/usr/bin/env python3
"""
Wrap a .deb into a .flatpak bundle. Handed the deb's filename, which carries the version and the architecture, so nothing here has to be told either, and the three names it was built under, which nothing here could learn.

Why this does not use flatpak-builder. The usual way to make a flatpak is a YAML manifest handed to flatpak-builder, which runs every build step inside a bubblewrap sandbox. That is right when a manifest compiles something. Fuji's does not: the binary was already built by the tauri container, and every step here is unpacking an archive and putting files where a flatpak expects them. A sandbox around copying a file earns nothing.

It also could not run. bubblewrap installs a seccomp filter, and under rosetta, which is how an amd64 container executes on an apple silicon mac, that call fails with EINVAL and the build stops. Native arm64 is fine, so the fork in the road was an aarch64 flatpak nobody asked for or an x86_64 one built another way. The steps below are that other way: build-init lays out a tree, this script puts files in it, and build-export and build-bundle do the ostree work. None of them needs a sandbox, so the same script runs native and emulated alike.

The container still runs --privileged, for one narrow reason: build-export validates the icon it is exporting, and that check runs in bubblewrap, which needs a namespace an ordinary container cannot make. That bwrap call does not install a seccomp filter, which is why it survives rosetta where flatpak-builder's does not.

Submitting to flathub would want a manifest, because that is what their build farm reads. It does not exist yet and is not needed to make the bundle; this script is the specification for it when the day comes, since the permissions below are exactly what a manifest's finish-args would say.
"""
import re
import shutil
import subprocess
import sys
from pathlib import Path

#the deb, then the names it was built under. build.js reads all three from tauri.conf.json and Cargo.toml, since this container never sees either file, and the README's section The two names says where each is set. brand_name is productName, which named the deb and its desktop entry; brand_stem is the crate's name, which named the executable and its icons; and identifier is the name everything in a flatpak is filed under. Handing the identifier in rather than writing it here a second time is what keeps a flatpak and its desktop entry from ever naming two different ones, which is a bundle that installs and does not appear. Its form is what flathub wants: three components, a domain we control reversed, and the application's own name last
if len(sys.argv) != 5: sys.exit('say the deb to wrap, the product name it was built under, the stem of the executable inside it, and the application identifier')
deb, brand_name, brand_stem, identifier = sys.argv[1:5]
work = Path('/work')
out = Path('/out')

def say(text): print(text, flush=True)
def run(arguments): subprocess.run([str(a) for a in arguments], cwd=work, check=True)
def read(arguments): return subprocess.run([str(a) for a in arguments], check=True, capture_output=True, text=True).stdout
def install(source, target, mode):  #one file into the tree, its parents made and its mode set, as install -D does
	target.parent.mkdir(parents=True, exist_ok=True)
	shutil.copyfile(source, target)
	target.chmod(mode)

#ask the image which runtime it has rather than naming one here. Both numbers have to agree, the image installs a runtime and build-init builds against one, and two places holding the same version is a drift waiting to happen: bump the Dockerfile alone and this fails at build-init with a runtime that is not installed. So the Dockerfile's ARG is the single say, and this reads what it did
installed = [line.split() for line in read(['flatpak', 'list', '--columns=application,branch']).splitlines()]
runtime_version = next((branch for application, branch in installed if application == 'org.gnome.Platform'), '')
if not runtime_version: sys.exit('no org.gnome.Platform installed in this image, rebuild it with pnpm build-images')
say(f'==> building against the GNOME {runtime_version} runtime')

#tauri writes brand_name, then _<version>_<arch>.deb, and flatpak names architectures differently from debian
named = re.fullmatch(re.escape(brand_name) + r'_([0-9][0-9.]*)_(.*)\.deb', deb)
if not named: sys.exit(f'{deb} is not named the way tauri names a deb')
version, debian_arch = named.groups()
flatpak_arch = {'amd64': 'x86_64', 'arm64': 'aarch64'}.get(debian_arch)
if not flatpak_arch: sys.exit(f"no flatpak architecture known for '{debian_arch}'")

shutil.rmtree(work, ignore_errors=True)
work.mkdir()
shutil.copy2(out / deb, work / deb)

say('==> unpacking the deb')
run(['ar', 'x', deb])
run(['tar', '-xf', 'data.tar.gz'])

say('==> build-init')
run(['flatpak', 'build-init', 'build', identifier, 'org.gnome.Sdk', 'org.gnome.Platform', runtime_version])

#everything a flatpak ships is named for the identifier rather than for the binary, so the renames below are not tidying: a .desktop file or an icon under any other name is simply not found
say('==> laying out /app')
files = work / 'build/files'
install(work / f'usr/bin/{brand_stem}', files / f'bin/{brand_stem}', 0o755)
entry = files / f'share/applications/{identifier}.desktop'
install(work / f'usr/share/applications/{brand_name}.desktop', entry, 0o644)
entry.write_text(re.sub(r'^Icon=.*$', f'Icon={identifier}', entry.read_text(), flags=re.M))
for made, size in (('32x32', '32x32'), ('128x128', '128x128'), ('256x256@2', '512x512')):  #tauri writes a 256x256@2 folder, which is not a size freedesktop recognises, so that one is installed as the 512x512 it actually is
	install(work / f'usr/share/icons/hicolor/{made}/apps/{brand_stem}.png', files / f'share/icons/hicolor/{size}/apps/{identifier}.png', 0o644)
shutil.copytree(work / f'usr/lib/{brand_name}', files / f'lib/{brand_name}', symlinks=True)  #the resources the deb installs under usr/lib and the product's name, the media library among them, laid out under /app the same way, where the binary's rpath, ../lib and the product's name relative to itself, finds them

#a sandboxed application starts with no way to draw and no way to reach a file. The first four are what any GUI needs: wayland with an x11 fallback covers both kinds of session, dri is what makes the web engine's compositing hardware accelerated rather than software.
#
#--filesystem=host is the one that is an argument rather than a detail. Fuji is a file manager: it is handed a folder and reads what is in it, so the portal model, where a user picks one file at a time through a dialog the application cannot see past, describes a different program than this one. A flatpak asking for the host filesystem is flagged as weakening its own sandbox, which is true, and is better said plainly here than worked around
say('==> build-finish')
run(['flatpak', 'build-finish', 'build', '--socket=wayland', '--socket=fallback-x11', '--device=dri', '--socket=pulseaudio', '--share=ipc', '--filesystem=host', f'--command={brand_stem}'])

say('==> build-export')
run(['flatpak', 'build-export', 'repo', 'build'])  #this repository is also, exactly, what a flatpak remote of our own would serve, so a single-file bundle and a self-hosted repo are the same build with a different last step

say('==> build-bundle')
bundle = out / f'{brand_name}_{version}_{flatpak_arch}.flatpak'
run(['flatpak', 'build-bundle', 'repo', bundle, identifier])
say(f'==> bundled {bundle}, {bundle.stat().st_size} bytes')
