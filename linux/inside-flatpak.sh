#!/bin/sh
# Wrap a .deb into a .flatpak bundle. Handed the deb's filename, which carries the version and the architecture, so nothing here has to be told either.
#
# ## Why this does not use flatpak-builder
#
# The usual way to make a flatpak is a YAML manifest handed to flatpak-builder, which runs every build step inside a bubblewrap sandbox. That is right when a manifest compiles something. Fuji's does not: the binary was already built by the tauri container, and every step here is unpacking an archive and putting files where a flatpak expects them. A sandbox around `install -Dm755` earns nothing.
#
# It also could not run. bubblewrap installs a seccomp filter, and under rosetta — which is how an amd64 container executes on an apple silicon mac — that call fails with EINVAL and the build stops. Native arm64 is fine, so the fork in the road was an aarch64 flatpak nobody asked for or an x86_64 one built another way. The commands below are that other way: build-init lays out a tree, ordinary shell puts files in it, and build-export and build-bundle do the ostree work. None of them needs a sandbox, so the same script runs native and emulated alike.
#
# The container still runs --privileged, for one narrow reason: build-export validates the icon it is exporting, and that check runs in bubblewrap, which needs a namespace an ordinary container cannot make. That bwrap call does not install a seccomp filter, which is why it survives rosetta where flatpak-builder's does not.
#
# Submitting to flathub would want a manifest, because that is what their build farm reads. It does not exist yet and is not needed to make the bundle; this script is the specification for it when the day comes — the permissions below are exactly what a manifest's finish-args would say.
set -eu

deb="${1:?say which deb to wrap}"
id=com.zootella.fuji

# Ask the image which runtime it has rather than naming one here. Both numbers have to agree — the image installs a runtime and build-init builds against one — and two places holding the same version is a drift waiting to happen: bump the Dockerfile alone and this fails at build-init with a runtime that is not installed. So the Dockerfile's ARG is the single say, and this reads what it did.
runtime_version=$(flatpak list --columns=application,branch | awk '$1 == "org.gnome.Platform" {print $2; exit}')
runtime_version="${runtime_version:?no org.gnome.Platform installed in this image — rebuild it with pnpm image}"
echo "==> building against the GNOME ${runtime_version} runtime"

# tauri writes Fuji_<version>_<arch>.deb, and flatpak names architectures differently from debian
version=$(echo "$deb" | sed -n 's/^Fuji_\([0-9][0-9.]*\)_.*/\1/p')
debian_arch=$(echo "$deb" | sed -n 's/^Fuji_[0-9][0-9.]*_\(.*\)\.deb$/\1/p')
case "$debian_arch" in
	amd64) flatpak_arch=x86_64 ;;
	arm64) flatpak_arch=aarch64 ;;
	*) echo "no flatpak architecture known for '$debian_arch'" >&2; exit 1 ;;
esac

rm -rf /work
mkdir -p /work
cp "/out/${deb}" /work/fuji.deb
cd /work

echo "==> unpacking the deb"
ar x fuji.deb
tar -xf data.tar.gz

echo "==> build-init"
flatpak build-init build "$id" org.gnome.Sdk org.gnome.Platform "$runtime_version"

# Everything a flatpak ships is named for the application id rather than for the binary, so the renames below are not tidying: a .desktop file or an icon under any other name is simply not found.
echo "==> laying out /app"
install -Dm755 usr/bin/fuji build/files/bin/fuji
install -Dm644 usr/share/applications/Fuji.desktop "build/files/share/applications/${id}.desktop"
sed -i "s/^Icon=.*/Icon=${id}/" "build/files/share/applications/${id}.desktop"
# tauri writes a 256x256@2 folder, which is not a size freedesktop recognises, so that one is installed as the 512x512 it actually is
install -Dm644 usr/share/icons/hicolor/32x32/apps/fuji.png   "build/files/share/icons/hicolor/32x32/apps/${id}.png"
install -Dm644 usr/share/icons/hicolor/128x128/apps/fuji.png "build/files/share/icons/hicolor/128x128/apps/${id}.png"
install -Dm644 "usr/share/icons/hicolor/256x256@2/apps/fuji.png" "build/files/share/icons/hicolor/512x512/apps/${id}.png"

# A sandboxed application starts with no way to draw and no way to reach a file. The first four are what any GUI needs — wayland with an x11 fallback covers both kinds of session, dri is what makes the web engine's compositing hardware accelerated rather than software.
#
# --filesystem=host is the one that is an argument rather than a detail. Fuji is a file manager: it is handed a folder and reads what is in it, so the portal model, where a user picks one file at a time through a dialog the application cannot see past, describes a different program than this one. A flatpak asking for the host filesystem is flagged as weakening its own sandbox, which is true, and is better said plainly here than worked around.
echo "==> build-finish"
flatpak build-finish build \
	--socket=wayland \
	--socket=fallback-x11 \
	--device=dri \
	--share=ipc \
	--filesystem=host \
	--command=fuji

echo "==> build-export"
# this repository is also, exactly, what a flatpak remote of our own would serve — so a single-file bundle and a self-hosted repo are the same build with a different last step
flatpak build-export repo build

echo "==> build-bundle"
out="/out/Fuji_${version}_${flatpak_arch}.flatpak"
flatpak build-bundle repo "$out" "$id"

ls -la "$out"
echo "==> bundled $out"
