#!/usr/bin/env bash
# libmpv for windows, built from release tarballs: one dll with everything inside it, and nothing in it from anywhere but these sources.
#
# Twelve packages, each a tagged release fetched from its own project and checked against the hash written here, built static in dependency order into a prefix under target, and the last of them, mpv, linked as the one shared library, libmpv-2.dll, which carries the other eleven and the mingw runtime inside it. It imports Windows' own libraries and the universal C runtime and nothing else: no Vulkan, which a graphics driver installs and Windows does not, so no machine fails to start fuji for the lack of one, and Direct3D is loaded at runtime by mpv itself, so neither is a Direct3D library in the import table.
#
# What is in it, and why. FFmpeg with every decoder and demuxer, dav1d beside it for AV1, which FFmpeg decodes in software through no other library, and both of Windows' hardware decoding paths, D3D11VA and DXVA2; without encoders, muxers, devices or network, since fuji only ever plays what is on the disk. mpv without the player, the scripting languages, the disc formats and the frame servers; with WASAPI for sound, Direct3D 11 for a picture, which every Windows 10 draws with and which hardware decoding hands frames to without a copy, OpenGL and Direct3D 9 beneath it for the machines that want them, and the media keys through Windows' own transport controls. libplacebo, mpv's renderer, with Direct3D 11 and without Vulkan, which Direct3D covers on Windows. Direct3D takes shaders in HLSL and mpv writes them in GLSL, so between them sit shaderc, the GLSL compiler to SPIR-V, with glslang, SPIRV-Tools and SPIRV-Headers built inside it at the commits its DEPS file names, and SPIRV-Cross, the translator from SPIR-V to HLSL. lcms2 for ICC profiles, so a calibrated display is honored. libass for subtitles, over FreeType, HarfBuzz and FriBidi, with DirectWrite finding the fonts. zlib under FFmpeg and FreeType.
#
# The toolchain is MSYS2's ucrt64: gcc, meson, ninja, cmake, pkgconf, nasm and the C++/WinRT headers, which libmpv.js beside this installs the first time and starts this script under. Only the toolchain comes from MSYS2; PKG_CONFIG_LIBDIR is pinned to the prefix so no MSYS2 package can be found by accident, and the dll's imports are the proof. The build is about twenty minutes on the Windows box, FFmpeg nine of them and shaderc four, and a package once built is skipped on the next run, so a change to mpv's options alone costs a minute. The result is stripped and copied to src-tauri/binaries/libmpv-2.dll, untracked, where tauri.windows.conf.json names it and the installer picks it up. video-spike.md has the measurements and why the nightly this replaced was not enough.
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
clone_commit() {                                  # a repository at one commit rather than a tag, which is how shaderc's DEPS file names the three libraries it builds with; fetched into the folder shaderc's cmake looks in
	local folder=$1 url=$2 commit=$3
	if [ -d $folder ]; then return; fi
	echo "== cloning ${folder##*/} at $commit"
	mkdir -p $folder && git -C $folder init -q && git -C $folder fetch -q --depth 1 $url $commit && git -C $folder checkout -q FETCH_HEAD
}
fetch zlib-1.3.2       https://zlib.net/zlib-1.3.2.tar.xz                                                        d7a0654783a4da529d1bb793b7ad9c3318020af77667bcae35f95d0e42a792f3
fetch freetype-2.15.0  https://download.savannah.gnu.org/releases/freetype/freetype-2.15.0.tar.xz                93cd47eb719c325107f2d3cf087d06299e944c3b48a40b55537302304dea1dd5
fetch fribidi-1.0.17   https://github.com/fribidi/fribidi/releases/download/v1.0.17/fribidi-1.0.17.tar.xz        6949dcde27d41cebad1fd741fcafc36d55a1020d2d872d4a6eb3914caabbada2
fetch harfbuzz-14.6.0  https://github.com/harfbuzz/harfbuzz/releases/download/14.6.0/harfbuzz-14.6.0.tar.xz     d07a007327277708a2a73ae437887cdbaf282937f6d03ca5467723e9099af586
fetch libass-0.17.5    https://github.com/libass/libass/releases/download/0.17.5/libass-0.17.5.tar.xz            2dca25c0e0c837ddf00b52011b3f82cac1e4ddd3ad018227806b0c2288864acc
fetch SPIRV-Cross-vulkan-sdk-1.4.363.0 https://github.com/KhronosGroup/SPIRV-Cross/archive/refs/tags/vulkan-sdk-1.4.363.0.tar.gz f708982e88b763ef5b0394ed468fc2c3628f68cdab89f2e7a946cd628c04e721
fetch shaderc-2026.4   https://github.com/google/shaderc/archive/refs/tags/v2026.4.tar.gz                         f06ce5bcca94e5df7f34e115743597d0ad2e13c5fe9213c67dc8c76031241947
clone_commit $src/shaderc-2026.4/third_party/glslang       https://github.com/KhronosGroup/glslang.git       e1b562a8bed273a02f30b59b66a5d499793cede5
clone_commit $src/shaderc-2026.4/third_party/spirv-tools   https://github.com/KhronosGroup/SPIRV-Tools.git   ef96ed763b43b59b33b31b362f09a02b729fa1c9
clone_commit $src/shaderc-2026.4/third_party/spirv-headers https://github.com/KhronosGroup/SPIRV-Headers.git 04fd3caa1e8267e4d95c806cad901181728e1006
fetch lcms2-2.19       https://github.com/mm2/Little-CMS/releases/download/lcms2.19/lcms2-2.19.tar.gz           49e7e134e4299733dd0eda434fa468997a28ab3d33fa397c642b03644f552216
clone libplacebo-7.360.1 https://code.videolan.org/videolan/libplacebo.git v7.360.1 cee9b076f2c63104ccfd497fa79c39a867293ec4
fetch dav1d-1.5.4      https://downloads.videolan.org/pub/videolan/dav1d/1.5.4/dav1d-1.5.4.tar.xz                686616b7c69eb88d44459391ab25cac13b6647a3b288835c5784e71c1514a5c5
fetch ffmpeg-9.0.2     https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz                                           8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e
fetch mpv-0.41.0       https://github.com/mpv-player/mpv/archive/refs/tags/v0.41.0.tar.gz                        ee21092a5ee427353392360929dc64645c54479aefdb5babc5cfbb5fad626209

# one package: skipped if its marker is in the prefix, otherwise built in its source folder and marked
step() { local name=$1; shift; if [ -f $prefix/.$name ]; then echo "== $name already built"; return; fi; echo "== $name"; local started=$(date +%s); ( cd $src/$name && "$@" ); touch $prefix/.$name; echo "== $name done in $(( $(date +%s) - started )) s"; }
meson_build() { rm -rf build; meson setup build --prefix=$prefix --buildtype=release "$@" && ninja -C build -j$jobs install; }
cmake_build() { rm -rf build; cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=$prefix -DBUILD_SHARED_LIBS=OFF "$@" && ninja -C build -j$jobs install; }

build_zlib() { ./configure --prefix=$prefix --static && make -j$jobs && make install; }
build_freetype() { meson_build --default-library=static -Dzlib=system -Dharfbuzz=disabled -Dpng=disabled -Dbrotli=disabled -Dbzip2=disabled -Dtests=disabled; }
build_fribidi() { meson_build --default-library=static -Ddocs=false -Dbin=false -Dtests=false; }
build_harfbuzz() { meson_build --default-library=static -Dfreetype=enabled -Dglib=disabled -Dgobject=disabled -Dcairo=disabled -Dchafa=disabled -Dicu=disabled -Dgdi=disabled -Ddirectwrite=disabled -Dtests=disabled -Ddocs=disabled -Dintrospection=disabled -Dutilities=disabled; }
build_libass() { ./configure --prefix=$prefix --disable-shared --enable-static --disable-fontconfig --enable-directwrite --disable-libunibreak && make -j$jobs && make install; }
build_spirv_cross() {                           # the translator from SPIR-V to HLSL, which is how a GLSL shader reaches Direct3D; mpv and libplacebo ask pkg-config for it under the name of its shared build, so the static build gets a file under that name, listing the static libraries the C API is made of and the C++ runtime they need
	cmake_build -DSPIRV_CROSS_STATIC=ON -DSPIRV_CROSS_SHARED=OFF -DSPIRV_CROSS_CLI=OFF -DSPIRV_CROSS_ENABLE_TESTS=OFF -DSPIRV_CROSS_ENABLE_MSL=OFF -DSPIRV_CROSS_ENABLE_CPP=OFF -DSPIRV_CROSS_ENABLE_REFLECT=OFF -DSPIRV_CROSS_ENABLE_UTIL=OFF
	sed -e 's/^Name: spirv-cross-c$/Name: spirv-cross-c-shared/' -e 's/^Libs: .*/Libs: -L${libdir} -lspirv-cross-c -lspirv-cross-hlsl -lspirv-cross-glsl -lspirv-cross-core -lstdc++/' $prefix/lib/pkgconfig/spirv-cross-c.pc > $prefix/lib/pkgconfig/spirv-cross-c-shared.pc
}
build_shaderc() {                               # the GLSL compiler to SPIR-V, with glslang, SPIRV-Tools and SPIRV-Headers built inside it from the commits its DEPS file names; mpv and libplacebo ask for shaderc by its shared name, so the combined static library's file is copied over it
	cmake_build -DSHADERC_SKIP_TESTS=ON -DSHADERC_SKIP_EXAMPLES=ON -DSHADERC_SKIP_COPYRIGHT_CHECK=ON -DSHADERC_ENABLE_WGSL_OUTPUT=OFF -DSPIRV_SKIP_EXECUTABLES=ON -DENABLE_GLSLANG_BINARIES=OFF
	sed -e 's/^Name: shaderc_combined$/Name: shaderc/' -e 's/^Libs: .*/Libs: -L${libdir} -lshaderc_combined -lstdc++/' $prefix/lib/pkgconfig/shaderc_combined.pc > $prefix/lib/pkgconfig/shaderc.pc
}
build_lcms2() { meson_build --default-library=static -Dtests=disabled -Djpeg=disabled -Dtiff=disabled -Dutils=false -Dfastfloat=false -Dthreaded=false; }
build_libplacebo() { meson_build --default-library=static -Dvulkan=disabled -Dvk-proc-addr=disabled -Dopengl=enabled -Dd3d11=enabled -Dshaderc=enabled -Dglslang=disabled -Dlcms=enabled -Dlibdovi=disabled -Dunwind=disabled -Dxxhash=disabled -Ddemos=false -Dtests=false; }
build_dav1d() { meson_build --default-library=static -Denable_tools=false -Denable_tests=false -Denable_examples=false -Denable_docs=false; }
build_ffmpeg() {
	./configure --prefix=$prefix --pkg-config-flags=--static --enable-static --disable-shared --disable-debug --disable-doc --disable-programs \
		--disable-autodetect --enable-zlib --enable-libdav1d --enable-d3d11va --enable-dxva2 \
		--disable-encoders --disable-muxers --disable-devices --disable-avdevice --disable-network --disable-protocols --enable-protocol=file \
		&& make -j$jobs && make install
}
build_mpv() {
	meson_build --default-library=shared -Dprefer_static=true \
		-Dcplayer=false -Dlibmpv=true -Dgpl=true -Dbuild-date=false -Dmanpage-build=disabled -Dtests=false \
		-Dwin32-threads=enabled -Dwasapi=enabled -Dgl-win32=enabled -Ddirect3d=enabled -Dzlib=enabled \
		-Dlcms2=enabled -Dd3d11=enabled -Dshaderc=enabled -Dspirv-cross=enabled -Dwin32-smtc=enabled \
		-Dlua=disabled -Djavascript=disabled -Diconv=disabled -Dcdda=disabled -Dcplugins=disabled -Ddvbin=disabled -Ddvdnav=disabled -Djpeg=disabled -Dlibarchive=disabled -Dlibavdevice=disabled -Dlibbluray=disabled -Drubberband=disabled -Duchardet=disabled -Dvapoursynth=disabled -Dzimg=disabled -Dsdl2-gamepad=disabled \
		-Dvulkan=disabled -Degl-angle=disabled -Degl-angle-lib=disabled -Degl-angle-win32=disabled -Dcuda-hwaccel=disabled -Dcuda-interop=disabled \
		-Dc_link_args="-static -static-libgcc" -Dcpp_link_args="-static -static-libgcc -static-libstdc++"
}

step zlib-1.3.2 build_zlib
step freetype-2.15.0 build_freetype
step fribidi-1.0.17 build_fribidi
step harfbuzz-14.6.0 build_harfbuzz
step libass-0.17.5 build_libass
step SPIRV-Cross-vulkan-sdk-1.4.363.0 build_spirv_cross
step shaderc-2026.4 build_shaderc
step lcms2-2.19 build_lcms2
step libplacebo-7.360.1 build_libplacebo
step dav1d-1.5.4 build_dav1d
step ffmpeg-9.0.2 build_ffmpeg
step mpv-0.41.0 build_mpv

# the result: stripped of its symbols, which takes it from 40 MB to 34, and put where the configuration names it
mkdir -p $tauri/binaries
strip -o $tauri/binaries/libmpv-2.dll $prefix/bin/libmpv-2.dll
echo "== libmpv-2.dll $(stat -c %s $tauri/binaries/libmpv-2.dll) bytes, sha256 $(sha256sum $tauri/binaries/libmpv-2.dll | cut -c1-64), in src-tauri/binaries"
