#!/bin/sh
# Run makepkg against the PKGBUILD, the way an Arch user's machine would.
#
# The deb is copied in beside the recipe under a fixed name, because a PKGBUILD validated here has to read its input from somewhere and the published one will read it from a URL that does not exist yet. aur/PKGBUILD says what changes when it does.
set -eu

deb="${1:?say which deb to package}"

# makepkg checks that every name in source() is present before it runs any of the PKGBUILD's own functions, so the deb has to arrive already carrying the name the recipe asks for. That name is built from the version in the deb's own filename rather than hardcoded, so bumping the version in one place — tauri.conf.json — keeps working.
version=$(echo "$deb" | sed -n 's/^Fuji_\([0-9][0-9.]*\)_.*/\1/p')
version="${version:?could not read a version out of '$deb'}"

# A PKGBUILD carries its own pkgver, and the AUR expects it bumped by hand each release, so it can fall behind tauri.conf.json. makepkg would report that as a missing source file and leave whoever is reading it hunting for a copy step that worked fine — so the disagreement is named here instead.
pkgver=$(sed -n 's/^pkgver=//p' /aur/PKGBUILD)
if [ "$version" != "$pkgver" ]; then
	echo "version mismatch: the deb is ${version}, aur/PKGBUILD says pkgver=${pkgver}" >&2
	echo "bump pkgver in aur/PKGBUILD to match the version in desktop/src-tauri/tauri.conf.json" >&2
	exit 1
fi

rm -rf /work
mkdir -p /work
cp /aur/PKGBUILD /work/
cp "/out/${deb}" "/work/fuji-${version}.deb"
chown -R builder:builder /work
cd /work

echo "==> makepkg"
# --syncdeps installs what the PKGBUILD declares, which is how a missing or misspelled dependency name is caught. --noconfirm because nobody is here to answer pacman.
su builder -c 'makepkg --syncdeps --noconfirm --cleanbuild'

echo "==> what makepkg produced"
ls -la /work/*.pkg.tar.* 2>/dev/null || { echo "makepkg made no package" >&2; exit 1; }
for p in /work/*.pkg.tar.*; do
	echo "--- $(basename "$p") ---"
	tar -tf "$p" | grep -vE '^\.' | head -20
done

# the PKGBUILD itself is what would be published, so that is what goes to /out — alongside the .SRCINFO the AUR requires beside every recipe and generates from it
su builder -c 'makepkg --printsrcinfo' > /work/.SRCINFO
mkdir -p /out/aur
cp /work/PKGBUILD /work/.SRCINFO /out/aur/
echo "==> PKGBUILD and .SRCINFO in /out/aur"
