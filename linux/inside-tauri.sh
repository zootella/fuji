#!/bin/sh
# What one container does, start to finish. It is handed a bundle list — deb, or deb,rpm — and it builds exactly that and nothing else.
#
# /src is the whitelist copy, mounted read only so a container can never write into the working tree on the mac. /out is linux/release, the only writable thing here. Everything in between happens in /work, which dies with the container.
set -eu

bundles="${1:?say which bundles to make, like deb or deb,rpm}"

echo "==> copying source out of the read-only mount"
rm -rf /work
mkdir -p /work
cp -a /src/. /work/
cd /work

# --frozen-lockfile is the whole reason pnpm-lock.yaml is in the whitelist. It refuses to resolve anything the lockfile does not already name, so this build installs the same versions the mac and windows box installed rather than whatever the registry is serving today. Every dependency in desktop/package.json is a caret range, so without this the packages would float on every run.
echo "==> pnpm install"
corepack enable
pnpm install --frozen-lockfile

# the media library first, from the recipe the mac and windows build with, since a container starts empty and the binary links against it; desktop/libmpv/build.sh says what goes in
echo "==> libmpv from its release sources"
bash desktop/libmpv/build.sh
echo "==> tauri build --bundles ${bundles}"
cd desktop
# --bundles overrides tauri.conf.json's targets for this run only, which is what keeps the linux specifics in this workspace instead of editing what the mac and windows builds are told to make.
pnpm tauri build --bundles "${bundles}"

echo "==> collecting packages"
mkdir -p /out
found=0
for f in $(find src-tauri/target/release/bundle -type f \( -name '*.deb' -o -name '*.rpm' \) | sort); do
	cp -v "$f" /out/
	found=$((found + 1))
done
[ "$found" -gt 0 ] || { echo "nothing was bundled — look above for what tauri said" >&2; exit 1; }
# and the record the recipe wrote beside the library, under this architecture's name, since the other container writes one too; pnpm hash puts it in each package's sidecar, the rpm and the flatpak taking amd64's, which is the library they carry
cp -v src-tauri/binaries/libmpv.json "/out/libmpv.$(dpkg --print-architecture).json"
echo "==> $found package(s) in /out"
