#!/usr/bin/env bash
# libmpv for windows, built from release tarballs: one dll with everything inside it, and nothing in it from anywhere but these sources.
#
# Eight packages, each a tagged release fetched from its own project and checked against the hash written here, built static in dependency order into a prefix under target, and the last of them, mpv, linked as the one shared library, libmpv-2.dll, which carries the other seven and the mingw runtime inside it. It imports Windows' own libraries and the universal C runtime and nothing else: no Vulkan, which a graphics driver installs and Windows does not, so no machine fails to start fuji for the lack of one. FFmpeg is built with every decoder and demuxer and without encoders, muxers, devices or network, since fuji only ever plays what is on the disk; mpv without the player, the scripting languages, the disc formats and the frame servers, with WASAPI for sound and OpenGL and Direct3D 9 for a picture; libplacebo without Vulkan and without Direct3D 11, which would need a shader compiler and its translator, two more packages, and can come when video does.
#
# The toolchain is MSYS2's ucrt64: gcc, meson, ninja, cmake, pkgconf and nasm, which libmpv.js beside this installs the first time and starts this script under. Only the toolchain comes from MSYS2; PKG_CONFIG_LIBDIR is pinned to the prefix so no MSYS2 package can be found by accident, and the dll's imports are the proof. The build is 14 minutes on the Windows box, FFmpeg nine of them, and a package once built is skipped on the next run, so a change to mpv's options alone costs a minute. The result is stripped and copied to src-tauri/binaries/libmpv-2.dll, untracked, where tauri.windows.conf.json names it and the installer picks it up. video-spike.md has the measurements and why the nightly this replaced was not enough.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
tauri=$here/../src-tauri
root=$tauri/target/libmpv                        # the sources, the build trees and the prefix, under tauri's own build output so a clean build clears them too
src=$root/src
prefix=$root/prefix
jobs=$(nproc)
export USERPROFILE=${USERPROFILE:-$(cygpath -w "$HOME")}  # meson's cmake lookups ask python for the home folder, and the windows python reads this
export PKG_CONFIG_PATH=$prefix/lib/pkgconfig
export PKG_CONFIG_LIBDIR=$prefix/lib/pkgconfig   # only our prefix, never msys2's own packages
export CFLAGS="-O2 -pipe -I$prefix/include" CXXFLAGS="-O2 -pipe -I$prefix/include" LDFLAGS="-L$prefix/lib"
export SOURCE_DATE_EPOCH=1791763200      # 2026-10-12, the day this recipe was written: the linker and strip stamp a time into every PE header, and with this one fixed the same sources make the same bytes, so the dll's hash names the recipe rather than the minute it ran
mkdir -p $src $prefix

# the sources: name, where its tarball is, and the sha256 of the tarball; libplacebo publishes no tarball with its submodules in it, so it is cloned at its tag and the commit checked instead
fetch() {
	local name=$1 url=$2 sha=$3
	local file=$src/${url##*/}
	if [ -d $src/$name ]; then return; fi
	echo "== fetching $name"
	curl -sSL -o $file $url
	echo "$sha *$file" | sha256sum -c --quiet || { echo "$file does not match the hash written in build.sh"; exit 1; }
	tar -xf $file -C $src
}
clone() {
	local name=$1 url=$2 tag=$3 commit=$4
	if [ -d $src/$name ]; then return; fi
	echo "== cloning $name"
	git clone -q --recursive --branch $tag --depth 1 $url $src/$name
	[ "$(git -C $src/$name rev-parse HEAD)" = "$commit" ] || { echo "$name at $tag is not commit $commit"; exit 1; }
}
fetch zlib-1.3.2       https://zlib.net/zlib-1.3.2.tar.xz                                                        d7a0654783a4da529d1bb793b7ad9c3318020af77667bcae35f95d0e42a792f3
fetch freetype-2.15.0  https://download.savannah.gnu.org/releases/freetype/freetype-2.15.0.tar.xz                93cd47eb719c325107f2d3cf087d06299e944c3b48a40b55537302304dea1dd5
fetch fribidi-1.0.17   https://github.com/fribidi/fribidi/releases/download/v1.0.17/fribidi-1.0.17.tar.xz        6949dcde27d41cebad1fd741fcafc36d55a1020d2d872d4a6eb3914caabbada2
fetch harfbuzz-14.6.0  https://github.com/harfbuzz/harfbuzz/releases/download/14.6.0/harfbuzz-14.6.0.tar.xz     d07a007327277708a2a73ae437887cdbaf282937f6d03ca5467723e9099af586
fetch libass-0.17.5    https://github.com/libass/libass/releases/download/0.17.5/libass-0.17.5.tar.xz            2dca25c0e0c837ddf00b52011b3f82cac1e4ddd3ad018227806b0c2288864acc
clone libplacebo-7.360.1 https://code.videolan.org/videolan/libplacebo.git v7.360.1 cee9b076f2c63104ccfd497fa79c39a867293ec4
fetch ffmpeg-9.0.2     https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz                                           8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e
fetch mpv-0.41.0       https://github.com/mpv-player/mpv/archive/refs/tags/v0.41.0.tar.gz                        ee21092a5ee427353392360929dc64645c54479aefdb5babc5cfbb5fad626209

# one package: skipped if its marker is in the prefix, otherwise built in its source folder and marked
step() { local name=$1; shift; if [ -f $prefix/.$name ]; then echo "== $name already built"; return; fi; echo "== $name"; local started=$(date +%s); ( cd $src/$name && "$@" ); touch $prefix/.$name; echo "== $name done in $(( $(date +%s) - started )) s"; }
mesonic() { rm -rf build; meson setup build --prefix=$prefix --buildtype=release "$@" && ninja -C build -j$jobs install; }

build_zlib() { ./configure --prefix=$prefix --static && make -j$jobs && make install; }
build_freetype() { mesonic --default-library=static -Dzlib=system -Dharfbuzz=disabled -Dpng=disabled -Dbrotli=disabled -Dbzip2=disabled -Dtests=disabled; }
build_fribidi() { mesonic --default-library=static -Ddocs=false -Dbin=false -Dtests=false; }
build_harfbuzz() { mesonic --default-library=static -Dfreetype=enabled -Dglib=disabled -Dgobject=disabled -Dcairo=disabled -Dchafa=disabled -Dicu=disabled -Dgdi=disabled -Ddirectwrite=disabled -Dtests=disabled -Ddocs=disabled -Dintrospection=disabled -Dutilities=disabled; }
build_libass() { ./configure --prefix=$prefix --disable-shared --enable-static --disable-fontconfig --enable-directwrite --disable-libunibreak && make -j$jobs && make install; }
build_libplacebo() { mesonic --default-library=static -Dvulkan=disabled -Dvk-proc-addr=disabled -Dopengl=enabled -Dd3d11=disabled -Dglslang=disabled -Dshaderc=disabled -Dlcms=disabled -Dlibdovi=disabled -Dunwind=disabled -Dxxhash=disabled -Ddemos=false -Dtests=false; }
build_ffmpeg() {
	./configure --prefix=$prefix --pkg-config-flags=--static --enable-static --disable-shared --disable-debug --disable-doc --disable-programs \
		--disable-autodetect --enable-zlib --enable-d3d11va --enable-dxva2 \
		--disable-encoders --disable-muxers --disable-devices --disable-avdevice --disable-network --disable-protocols --enable-protocol=file \
		&& make -j$jobs && make install
}
build_mpv() {
	mesonic --default-library=shared -Dprefer_static=true \
		-Dcplayer=false -Dlibmpv=true -Dgpl=true -Dbuild-date=false -Dmanpage-build=disabled -Dtests=false \
		-Dwin32-threads=enabled -Dwasapi=enabled -Dgl-win32=enabled -Ddirect3d=enabled -Dzlib=enabled \
		-Dlua=disabled -Djavascript=disabled -Diconv=disabled -Dcdda=disabled -Dcplugins=disabled -Ddvbin=disabled -Ddvdnav=disabled -Djpeg=disabled -Dlcms2=disabled -Dlibarchive=disabled -Dlibavdevice=disabled -Dlibbluray=disabled -Drubberband=disabled -Duchardet=disabled -Dvapoursynth=disabled -Dzimg=disabled -Dsdl2-gamepad=disabled -Dwin32-smtc=disabled \
		-Dvulkan=disabled -Dshaderc=disabled -Dspirv-cross=disabled -Dd3d11=disabled -Degl-angle=disabled -Degl-angle-lib=disabled -Degl-angle-win32=disabled -Dcuda-hwaccel=disabled -Dcuda-interop=disabled \
		-Dc_link_args="-static -static-libgcc" -Dcpp_link_args="-static -static-libgcc -static-libstdc++"
}

step zlib-1.3.2 build_zlib
step freetype-2.15.0 build_freetype
step fribidi-1.0.17 build_fribidi
step harfbuzz-14.6.0 build_harfbuzz
step libass-0.17.5 build_libass
step libplacebo-7.360.1 build_libplacebo
step ffmpeg-9.0.2 build_ffmpeg
step mpv-0.41.0 build_mpv

# the result: stripped of its symbols, which takes it from 40 MB to 34, and put where the configuration names it
mkdir -p $tauri/binaries
strip -o $tauri/binaries/libmpv-2.dll $prefix/bin/libmpv-2.dll
echo "== libmpv-2.dll $(stat -c %s $tauri/binaries/libmpv-2.dll) bytes, sha256 $(sha256sum $tauri/binaries/libmpv-2.dll | cut -c1-64), in src-tauri/binaries"
