#!/usr/bin/env python3
"""
What one container does, start to finish, in one of two modes. Handed the word library, it compiles the media library from its sources, keeps it in the cache, and writes its record and its ledger entry out, which is pnpm library in the linux workspace. Handed a bundle list, deb or deb,rpm, it takes the library from the cache and builds exactly those packages, which is pnpm build; without a library in the cache for the recipe it was handed it stops and says to run pnpm library, so every library that reaches a package was compiled by the command that logs it.

/src is the whitelist copy, mounted read only so a container can never write into the working tree on the mac. /out is linux/release, where the packages, the records and the ledger entries go. /cache is linux/cache, the one thing that outlives a container on purpose. Everything in between happens in /work, which dies with the container.

The cache holds the media library this container compiled, so a build skips the twelve minutes of FFmpeg under emulation, and the library is compiled a few times a year rather than a few times an hour. It is safe because the recipe writes libmpv.json beside the library it makes, with the library's hash and the hash of the recipe itself. A kept library is used only when its record names, by hash, the recipe this run was handed, and the library still hashes to what the record says; and the folder it is kept in is named for the architecture and a hash of the toolchain image's layers, so a Dockerfile that bumps a compiler is a miss. Those are the two things that can change the bytes, and both are in the key. The record travels with the library into the package's sidecar either way, and the ledger at the root says when the library was compiled, so pnpm hash and a reader of the repository see a library compiled last week by this recipe for exactly what it is. Deleting linux/cache by hand is how to compile fresh, which is pnpm library again.
"""
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

mode = sys.argv[1] if len(sys.argv) > 1 else sys.exit('say library, or which bundles to make, like deb or deb,rpm')
work = Path('/work')
out = Path('/out')
cache = Path('/cache')
desktop = work / 'desktop'
binaries = desktop / 'src-tauri/binaries'  #where the recipe leaves the library, its record and its ledger entry, and where tauri.linux.conf.json and build.rs look for the library
recipe = desktop / 'libmpv/build.py'

def say(text): print(text, flush=True)
def run(arguments, cwd=work): subprocess.run([str(a) for a in arguments], cwd=cwd, check=True)
def read(arguments): return subprocess.run([str(a) for a in arguments], check=True, capture_output=True, text=True).stdout.strip()
def sha256(path):
	digest = hashlib.sha256()
	with open(path, 'rb') as file:
		for block in iter(lambda: file.read(1 << 20), b''): digest.update(block)
	return digest.hexdigest()

say('==> copying source out of the read-only mount')
shutil.rmtree(work, ignore_errors=True)
shutil.copytree('/src', work, symlinks=True)

#--frozen-lockfile is the whole reason pnpm-lock.yaml is in the whitelist. It refuses to resolve anything the lockfile does not already name, so this build installs the same versions the mac and windows box installed rather than whatever the registry is serving today. Every dependency in desktop/package.json is a caret range, so without this the packages would float on every run
say('==> pnpm install')
run(['corepack', 'enable'])
run(['pnpm', 'install', '--frozen-lockfile'])

architecture = read(['dpkg', '--print-architecture'])
image = os.environ.get('IMAGE', '')  #the toolchain image as a hash of its layers, handed in by build.js, which says why it is not the image's id
kept = cache / f'{architecture}-{image}' if cache.is_dir() and image else None  #this architecture's shelf in the cache, or none when no cache is mounted
out.mkdir(parents=True, exist_ok=True)

def library_kept():  #the library and its record from the cache, into binaries, if the record names this recipe and describes that library; false otherwise
	if not kept or not (kept / 'libmpv.json').is_file(): return False
	record = json.loads((kept / 'libmpv.json').read_text())
	library = kept / record['file']
	if record['recipe'] != sha256(recipe) or not library.is_file() or sha256(library) != record['sha256']: return False
	binaries.mkdir(parents=True, exist_ok=True)
	shutil.copy2(library, binaries / record['file'])
	shutil.copy2(kept / 'libmpv.json', binaries / 'libmpv.json')
	unversioned = binaries / 'libmpv.so'  #the name the linker looks for, which the recipe makes beside the library and the cache does not keep
	unversioned.unlink(missing_ok=True)
	unversioned.symlink_to(record['file'])
	say(f"==> libmpv from the cache, compiled by this recipe at {record['built']}")
	return True
def library_keep():  #the library and its record just made, into the cache for the builds after, and this architecture's shelves for earlier images cleared, since a library the current image did not compile is never used again
	if not kept: return
	record = json.loads((binaries / 'libmpv.json').read_text())
	for shelf in cache.glob(f'{architecture}-*'):
		if shelf != kept: shutil.rmtree(shelf)
	kept.mkdir(parents=True, exist_ok=True)
	shutil.copy2(binaries / record['file'], kept / record['file'])
	shutil.copy2(binaries / 'libmpv.json', kept / 'libmpv.json')

if mode == 'library':
	say('==> libmpv from its release sources')
	run(['python3', recipe])
	library_keep()
	shutil.copy2(binaries / 'libmpv.json', out / f'libmpv.{architecture}.json')  #the record under this architecture's name, since the other container writes one too; pnpm hash puts it in each package's sidecar, the rpm and the flatpak taking amd64's, which is the library they carry
	shutil.copy2(binaries / 'library.txt', out / f'library.{architecture}.txt')  #and the ledger entry, for build.js to append to the architecture's library file at the root
	say(f'==> libmpv for {architecture} in the cache, with its record and its ledger entry in /out')
	sys.exit()

if not library_kept(): sys.exit(f'no media library in the cache for this recipe and this image; run pnpm library first, then pnpm build')
say(f'==> tauri build --bundles {mode}')
run(['pnpm', 'tauri', 'build', '--bundles', mode], cwd=desktop)  #--bundles overrides tauri.conf.json's targets for this run only, which is what keeps the linux specifics in this workspace instead of editing what the mac and windows builds are told to make

say('==> collecting packages')
found = sorted(path for path in (desktop / 'src-tauri/target/release/bundle').rglob('*') if path.is_file() and path.suffix in ('.deb', '.rpm'))
if not found: sys.exit('nothing was bundled, look above for what tauri said')
shutil.copy2(binaries / 'libmpv.json', out / f'libmpv.{architecture}.json')  #the record of the library these packages carry, which pnpm hash reads; before the packages, since hash refuses a package older than its record, which is how a library compiled after the packages were made is kept out of a release
for package in found:
	shutil.copy2(package, out / package.name)
	say(f'    {package.name}')
say(f'==> {len(found)} package(s) in /out')
