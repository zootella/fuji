#!/usr/bin/env bash
# libmpv for fuji, built from release tarballs on the Mac, on Windows and in the Linux containers: one library with everything inside it, and nothing in it from anywhere but these sources. The same sources at the same versions on each platform, so the library the About section names is the recipe's and every package carries the same work.
#
# On Windows, twelve packages, each a tagged release fetched from its own project and checked against the hash written here, built static in dependency order into a prefix under target, and the last of them, mpv, linked as the one shared library, libmpv-2.dll, which carries the other eleven and the mingw runtime inside it. It imports Windows' own libraries and the universal C runtime and nothing else: no Vulkan, which a graphics driver installs and Windows does not, so no machine fails to start fuji for the lack of one, and Direct3D is loaded at runtime by mpv itself, so neither is a Direct3D library in the import table.
#
# What is in it, and why. FFmpeg with every decoder and demuxer, dav1d beside it for AV1, which FFmpeg decodes in software through no other library, and both of Windows' hardware decoding paths, D3D11VA and DXVA2; without encoders, muxers, devices or network, since fuji only ever plays what is on the disk. mpv without the player, the scripting languages, the disc formats and the frame servers; with WASAPI for sound, Direct3D 11 for a picture, which every Windows 10 draws with and which hardware decoding hands frames to without a copy, OpenGL and Direct3D 9 beneath it for the machines that want them, and the media keys through Windows' own transport controls. libplacebo, mpv's renderer, with Direct3D 11 and without Vulkan, which Direct3D covers on Windows. Direct3D takes shaders in HLSL and mpv writes them in GLSL, so between them sit shaderc, the GLSL compiler to SPIR-V, with glslang, SPIRV-Tools and SPIRV-Headers built inside it at the commits its DEPS file names, and SPIRV-Cross, the translator from SPIR-V to HLSL. lcms2 for ICC profiles, so a calibrated display is honored. libass for subtitles, over FreeType, HarfBuzz and FriBidi, with DirectWrite finding the fonts. zlib under FFmpeg and FreeType.
#
# The Windows toolchain is MSYS2's ucrt64: gcc, meson, ninja, cmake, pkgconf, nasm and the C++/WinRT headers, which libmpv.js beside this installs the first time and starts this script under. Only the toolchain comes from MSYS2; PKG_CONFIG_LIBDIR is pinned to the prefix so no MSYS2 package can be found by accident, and the dll's imports are the proof. The build is about twenty minutes on the Windows box, FFmpeg nine of them and shaderc four, and a package once built is skipped on the next run, so a change to mpv's options alone costs a minute. The result is stripped and copied to src-tauri/binaries/libmpv-2.dll, untracked, where tauri.windows.conf.json names it and the installer picks it up. It replaced a nightly from a build farm, a thousand commits past the release on an FFmpeg with no number, kept thirty days and never the same bytes twice, and the installer carrying it went from 38.6 MB to 20.8.
#
# On the Mac the same recipe makes libmpv.2.dylib, arm64, for macOS 11 and later, which is every Mac with Apple silicon, from ten of the twelve: FFmpeg with VideoToolbox for the hardware decoder; mpv with CoreAudio for sound and the OpenGL output, with VideoToolbox's frames handed to it without a copy, which is the output mpv's own Mac window draws through and the one the render API offers; libplacebo with OpenGL alone, since OpenGL takes shaders as mpv writes them and wants no compiler, so the two Khronos packages are Windows' alone; and CoreText finding the fonts for libass. Vulkan over MoltenVK is left out until a reason appears. mpv's Cocoa code, which its OpenGL backend and so the render API's OpenGL path stand on, includes a header its Swift build generates, so the Swift bridge is built with Apple's own swiftc, and the pieces above it, mpv's own window, media keys and touch bar, which a library drawing into fuji's view never shows, are left out. The toolchain is Homebrew's, meson, ninja and pkgconf over Apple's clang and Swift, which libmpv.js beside this names when one is missing; cmake and nasm stay on Windows, since no package here is built with cmake and arm64 assembly goes through the compiler, and nothing of Homebrew's is in the result: PKG_CONFIG_LIBDIR is pinned to the prefix as on Windows, and the dylib names Apple's own frameworks and libraries and nothing else, which otool -L shows. The result is stripped, told that its name is @rpath/libmpv.2.dylib, which the rpath Tauri gives the app resolves to Contents/Frameworks in the bundle and to target/Frameworks beside a development build, signed ad hoc since stripping breaks the signature the linker gave it, and copied to src-tauri/binaries/libmpv.2.dylib, untracked, where tauri.macos.conf.json names it and Tauri carries it into the bundle; libmpv.dylib beside it is the unversioned name the linker looks for.
#
# On Linux the same recipe makes libmpv.so.2 inside the containers the linux workspace runs, from the same ten packages as the Mac, over the toolchain the tauri image installs from Debian 12: FFmpeg decoding in software, since a hardware path there means VAAPI and a dependency on the machine's driver stack, which waits for a measurement; mpv with ALSA and PulseAudio for sound, the two every desktop has and PipeWire answers for, and the OpenGL output reached through X11, which is the lightest of mpv's ways to have OpenGL at all and is never used as a window, since the render API draws into fuji's own view; libplacebo with OpenGL alone; and fontconfig finding libass's fonts. The library names the distribution's X11, OpenGL, fontconfig and sound libraries and the C library, which the packages depend on by name, and nothing else. The result is stripped and copied to src-tauri/binaries/libmpv.so.2, where tauri.linux.conf.json names it as a resource the deb and the rpm install beside the application's other files and the flatpak carries into its sandbox, and the binary finds it by an rpath relative to itself.
#
# Beside the library goes libmpv.json, the record of what was made and where it came from: the library's hash and size, the day, the hash of this file, the tools, and every source with its hash. pnpm hash copies that record into the release's sidecar, which git tracks, so the repository says of every release which library it carried and where each part of it came from, long after this recipe has moved on; and it refuses a library this file as checked out did not build. So an edit here, to a comment as much as to a flag, means running this again before a release is staged, which costs seconds when nothing has to be rebuilt, since a package already built is skipped under a stamp of what built it.
set -euo pipefail
case "$(uname -s)" in Darwin) platform=mac;; MINGW*|MSYS*) platform=windows;; Linux) platform=linux;; *) echo "build.sh: no section for $(uname -s)"; exit 1;; esac
here=$(cd "$(dirname "$0")" && pwd)
tauri=$here/../src-tauri
root=$tauri/target/libmpv                        # the sources, the build trees and the prefix, under tauri's own build output so a clean build clears them too
src=$root/src
prefix=$root/prefix
jobs=$(nproc 2>/dev/null || sysctl -n hw.ncpu)   # linux and msys2 have nproc, as the windows build proved; the mac asks sysctl
sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -c1-64; }  # the hash of a file, by whichever tool the platform has
sha256_text() { if command -v sha256sum >/dev/null; then sha256sum; else shasum -a 256; fi | cut -c1-64; }      # and of what is piped in
bytes() { stat -c %s "$1" 2>/dev/null || stat -f %z "$1"; }                                                      # the size of a file, by whichever stat the platform has
if [ $platform = windows ]; then export USERPROFILE=${USERPROFILE:-$(cygpath -w "$HOME")}; fi  # meson's cmake lookups ask python for the home folder, and the windows python reads this
if [ $platform = mac ]; then export MACOSX_DEPLOYMENT_TARGET=11.0 ZERO_AR_DATE=1; fi           # the floor every Apple silicon Mac clears, and archives without timestamps, for the same bytes from the same sources
export PKG_CONFIG_PATH=$prefix/lib/pkgconfig
export PKG_CONFIG_LIBDIR=$prefix/lib/pkgconfig   # only our prefix, never msys2's or homebrew's own packages
if [ $platform = linux ]; then PKG_CONFIG_LIBDIR=$PKG_CONFIG_LIBDIR:/usr/lib/$(gcc -print-multiarch)/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig; fi  # and on linux the distribution's after it, for the handful the library names on purpose: fontconfig, x11, opengl and the two sound libraries, each asked for by name below
export CFLAGS="-O2 -pipe -I$prefix/include" CXXFLAGS="-O2 -pipe -I$prefix/include" LDFLAGS="-L$prefix/lib"
if [ $platform = linux ]; then CFLAGS="$CFLAGS -fPIC" CXXFLAGS="$CXXFLAGS -fPIC"; fi           # x86-64 code is not position independent unless asked, and every static library here ends up inside one shared one; arm64 and the other two platforms are already
if [ $platform = mac ]; then LDFLAGS="$LDFLAGS -Wl,-x"; fi                                     # no local symbols in the dylib's symbol table, which strip takes out after anyway: Apple's linker writes them in an order that differs from one link to the next, by whichever of its threads finishes first, and hashes the whole file into the dylib's UUID before anything is stripped, so two links of the same objects on the Mac mini on 2026-10-15 differed in those sixteen bytes and nothing else, and the linker's -reproducible option, tried first, did not settle the order. With no local symbols to order, the link is the same every time
export SOURCE_DATE_EPOCH=1791763200      # 2026-10-12, the day this recipe was written: on windows the linker and strip stamp a time into every PE header, and with this one fixed the same sources make the same bytes, so the dll's hash names the recipe rather than the minute it ran; the mac and linux stamp no time and read it anyway
mkdir -p $src $prefix
tools="$(gcc --version | head -1); meson $(meson --version); ninja $(ninja --version)"   # the toolchain the package manager provided, said at the top of every run and written into the record at the end, since it is the one part of a build this file does not pin: a library hash that moved with nothing changed here is answered by this line
if [ $platform = mac ]; then tools="$tools; $(swiftc --version 2>&1 | head -1)"; else tools="$tools; $(nasm --version | head -1)"; fi
echo "== tools: $tools"

# the sources: name, where its tarball is, and the sha256 of the tarball; libplacebo publishes no tarball with its submodules in it, so it is cloned at its tag and the commit checked instead. Each is also written down for the record at the end, fetched this run or found from the last
sources=
source_record() { sources="$sources"$'\n\t\t'"{\"name\": \"$1\", \"url\": \"$2\", \"$3\": \"$4\"},"; }   # one line of the record: the name, where it came from, and its hash or its commit
fetch() {
	local name=$1 url=$2 sha=$3
	local file=$src/${url##*/}
	source_record $name $url sha256 $sha
	if [ -d $src/$name ]; then return; fi
	echo "== fetching $name"
	curl -fsSL --retry 3 -o $file $url   # -f so a server's error page is never saved as the archive, and three tries, since the hash check below would refuse a short download anyway
	[ "$(sha256 $file)" = "$sha" ] || { echo "$file does not match the hash written in build.sh"; exit 1; }
	tar -xf $file -C $src
}
clone() {
	local name=$1 url=$2 tag=$3 commit=$4
	source_record $name $url commit $commit
	if [ -d $src/$name ]; then return; fi
	echo "== cloning $name"
	git clone -q --recursive --branch $tag --depth 1 $url $src/$name
	[ "$(git -C $src/$name rev-parse HEAD)" = "$commit" ] || { echo "$name at $tag is not commit $commit"; exit 1; }
}
clone_commit() {                                  # a repository at one commit rather than a tag, which is how shaderc's DEPS file names the three libraries it builds with; fetched into the folder shaderc's cmake looks in
	local folder=$1 url=$2 commit=$3
	source_record ${folder##*/} $url commit $commit
	if [ -d $folder ]; then return; fi
	echo "== cloning ${folder##*/} at $commit"
	mkdir -p $folder && git -C $folder init -q && git -C $folder fetch -q --depth 1 $url $commit && git -C $folder checkout -q FETCH_HEAD
}
fetch zlib-1.3.2       https://github.com/madler/zlib/releases/download/v1.3.2/zlib-1.3.2.tar.xz                 d7a0654783a4da529d1bb793b7ad9c3318020af77667bcae35f95d0e42a792f3  # the same bytes zlib.net serves, from the release page that keeps every version: zlib.net serves the current version alone and takes the one before down the day a new one ships, so 1.3.1 answered 404 there on 2026-10-15
fetch freetype-2.15.0  https://download-mirror.savannah.gnu.org/releases/freetype/freetype-2.15.0.tar.xz         93cd47eb719c325107f2d3cf087d06299e944c3b48a40b55537302304dea1dd5  # savannah's own mirror by name, since its download address bounces to a random one, and one of those answered 404 and another with a page in place of the archive
fetch fribidi-1.0.17   https://github.com/fribidi/fribidi/releases/download/v1.0.17/fribidi-1.0.17.tar.xz        6949dcde27d41cebad1fd741fcafc36d55a1020d2d872d4a6eb3914caabbada2
fetch harfbuzz-14.6.0  https://github.com/harfbuzz/harfbuzz/releases/download/14.6.0/harfbuzz-14.6.0.tar.xz     d07a007327277708a2a73ae437887cdbaf282937f6d03ca5467723e9099af586
fetch libass-0.17.5    https://github.com/libass/libass/releases/download/0.17.5/libass-0.17.5.tar.xz            2dca25c0e0c837ddf00b52011b3f82cac1e4ddd3ad018227806b0c2288864acc
if [ $platform = windows ]; then                 # the shader compiler and translator, which Direct3D wants and OpenGL does not
fetch SPIRV-Cross-vulkan-sdk-1.4.363.0 https://github.com/KhronosGroup/SPIRV-Cross/archive/refs/tags/vulkan-sdk-1.4.363.0.tar.gz f708982e88b763ef5b0394ed468fc2c3628f68cdab89f2e7a946cd628c04e721
fetch shaderc-2026.4   https://github.com/google/shaderc/archive/refs/tags/v2026.4.tar.gz                         f06ce5bcca94e5df7f34e115743597d0ad2e13c5fe9213c67dc8c76031241947
clone_commit $src/shaderc-2026.4/third_party/glslang       https://github.com/KhronosGroup/glslang.git       e1b562a8bed273a02f30b59b66a5d499793cede5
clone_commit $src/shaderc-2026.4/third_party/spirv-tools   https://github.com/KhronosGroup/SPIRV-Tools.git   ef96ed763b43b59b33b31b362f09a02b729fa1c9
clone_commit $src/shaderc-2026.4/third_party/spirv-headers https://github.com/KhronosGroup/SPIRV-Headers.git 04fd3caa1e8267e4d95c806cad901181728e1006
fi
fetch lcms2-2.19       https://github.com/mm2/Little-CMS/releases/download/lcms2.19/lcms2-2.19.tar.gz           49e7e134e4299733dd0eda434fa468997a28ab3d33fa397c642b03644f552216
clone libplacebo-7.360.1 https://code.videolan.org/videolan/libplacebo.git v7.360.1 cee9b076f2c63104ccfd497fa79c39a867293ec4
fetch dav1d-1.5.4      https://downloads.videolan.org/pub/videolan/dav1d/1.5.4/dav1d-1.5.4.tar.xz                686616b7c69eb88d44459391ab25cac13b6647a3b288835c5784e71c1514a5c5
fetch ffmpeg-9.0.2     https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz                                           8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e
fetch mpv-0.41.0       https://github.com/mpv-player/mpv/archive/refs/tags/v0.41.0.tar.gz                        ee21092a5ee427353392360929dc64645c54479aefdb5babc5cfbb5fad626209

# one package: skipped if its marker in the prefix holds this run's stamp, otherwise built in its source folder by its function and marked. The stamp is the hash of that function, the two build helpers, the flags every package gets, the ones named after it, which are the platform's for that package, and the stamp of the package before it; so a flag changed here rebuilds the package rather than leaving last run's in the prefix under a recipe that never made it, and rebuilds every package after it too, since each is linked into the next, and a run stopped partway leaves nothing marked current that was built against what came before. A new version is a new name, and so a new folder and marker
chain=
step() {
	local name=$1 build=$2; shift 2
	local stamp=$(printf '%s' "$chain $(declare -f $build meson_build cmake_build) $CFLAGS $CXXFLAGS $LDFLAGS $*" | sha256_text)
	chain=$stamp
	if [ "$(cat $prefix/.$name 2>/dev/null)" = "$stamp" ]; then echo "== $name already built"; return; fi
	echo "== $name"; local started=$(date +%s)
	( cd $src/$name && $build )
	echo "$stamp" > $prefix/.$name
	echo "== $name done in $(( $(date +%s) - started )) s"
}
meson_build() { rm -rf build; meson setup build --prefix=$prefix --libdir=lib --buildtype=release "$@" && ninja -C build -j$jobs install; }  # libdir said outright, because debian's meson would put libraries and their pkg-config files under lib/<triplet>, where PKG_CONFIG_LIBDIR above never looks
cmake_build() { rm -rf build; cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=$prefix -DCMAKE_INSTALL_LIBDIR=lib -DBUILD_SHARED_LIBS=OFF "$@" && ninja -C build -j$jobs install; }

clean() { make distclean >/dev/null 2>&1 || true; }   # the three built by configure and make start from a clean tree, as the meson and cmake builds do by removing their build folder, so a rebuild never keeps an object compiled under the flags before
build_zlib() { clean; ./configure --prefix=$prefix --static && make -j$jobs && make install; }
build_freetype() { meson_build --default-library=static -Dzlib=system -Dharfbuzz=disabled -Dpng=disabled -Dbrotli=disabled -Dbzip2=disabled -Dtests=disabled; }
build_fribidi() { meson_build --default-library=static -Ddocs=false -Dbin=false -Dtests=false; }
build_harfbuzz() { meson_build --default-library=static -Dfreetype=enabled -Dglib=disabled -Dgobject=disabled -Dcairo=disabled -Dchafa=disabled -Dicu=disabled -Dgdi=disabled -Ddirectwrite=disabled -Dcoretext=disabled -Dpng=disabled -Dtests=disabled -Ddocs=disabled -Dintrospection=disabled -Dutilities=disabled; }
build_libass() { clean; ./configure --prefix=$prefix --disable-shared --enable-static --disable-directwrite --disable-coretext --disable-fontconfig $fonts --disable-libunibreak && make -j$jobs && make install; }  # the platform's own font provider, below, turned on after the three are turned off, since a later flag wins
build_spirv_cross() {                           # the translator from SPIR-V to HLSL, which is how a GLSL shader reaches Direct3D; mpv and libplacebo ask pkg-config for it under the name of its shared build, so the static build gets a file under that name, listing the static libraries the C API is made of and the C++ runtime they need
	cmake_build -DSPIRV_CROSS_STATIC=ON -DSPIRV_CROSS_SHARED=OFF -DSPIRV_CROSS_CLI=OFF -DSPIRV_CROSS_ENABLE_TESTS=OFF -DSPIRV_CROSS_ENABLE_MSL=OFF -DSPIRV_CROSS_ENABLE_CPP=OFF -DSPIRV_CROSS_ENABLE_REFLECT=OFF -DSPIRV_CROSS_ENABLE_UTIL=OFF
	sed -e 's/^Name: spirv-cross-c$/Name: spirv-cross-c-shared/' -e 's/^Libs: .*/Libs: -L${libdir} -lspirv-cross-c -lspirv-cross-hlsl -lspirv-cross-glsl -lspirv-cross-core -lstdc++/' $prefix/lib/pkgconfig/spirv-cross-c.pc > $prefix/lib/pkgconfig/spirv-cross-c-shared.pc
}
build_shaderc() {                               # the GLSL compiler to SPIR-V, with glslang, SPIRV-Tools and SPIRV-Headers built inside it from the commits its DEPS file names; mpv and libplacebo ask for shaderc by its shared name, so the combined static library's file is copied over it
	cmake_build -DSHADERC_SKIP_TESTS=ON -DSHADERC_SKIP_EXAMPLES=ON -DSHADERC_SKIP_COPYRIGHT_CHECK=ON -DSHADERC_ENABLE_WGSL_OUTPUT=OFF -DSPIRV_SKIP_EXECUTABLES=ON -DENABLE_GLSLANG_BINARIES=OFF
	sed -e 's/^Name: shaderc_combined$/Name: shaderc/' -e 's/^Libs: .*/Libs: -L${libdir} -lshaderc_combined -lstdc++/' $prefix/lib/pkgconfig/shaderc_combined.pc > $prefix/lib/pkgconfig/shaderc.pc
}
build_lcms2() { meson_build --default-library=static -Dtests=disabled -Djpeg=disabled -Dtiff=disabled -Dutils=false -Dfastfloat=false -Dthreaded=false; }
build_libplacebo() { meson_build --default-library=static -Dvulkan=disabled -Dvk-proc-addr=disabled -Dopengl=enabled -Dglslang=disabled -Dlcms=enabled -Dlibdovi=disabled -Dunwind=disabled -Dxxhash=disabled -Ddemos=false -Dtests=false $placebo; }
build_dav1d() { meson_build --default-library=static -Denable_tools=false -Denable_tests=false -Denable_examples=false -Denable_docs=false; }
build_ffmpeg() {
	clean
	./configure --prefix=$prefix --pkg-config-flags=--static --enable-static --disable-shared --disable-debug --disable-doc --disable-programs \
		--disable-autodetect --enable-zlib --enable-libdav1d $ffmpeg \
		--disable-encoders --disable-muxers --disable-devices --disable-avdevice --disable-network --disable-protocols --enable-protocol=file \
		&& make -j$jobs && make install
}
fold_private() {                                # linux only: our own libraries are static and nothing else, so each one's private link line and requirements, which pkg-config gives out only when asked for a static link, become its public ones; then mpv's build, asking for a plain link, gets our ten whole and the distribution's fontconfig, X11, OpenGL and sound libraries as shared objects, where asking for static links throughout pulled the distribution's static fontconfig in and the link failed
	python3 - $prefix/lib/pkgconfig/*.pc <<'PY'
import re, sys
for path in sys.argv[1:]:
	text = open(path).read()
	for key in ('Libs', 'Requires'):
		private = re.search(r'^%s\.private:(.*)$' % key, text, re.M)
		if not private: continue
		text = re.sub(r'^%s\.private:.*\n?' % key, '', text, flags=re.M)
		if re.search(r'^%s:' % key, text, re.M): text = re.sub(r'^(%s:.*)$' % key, lambda m: m.group(1) + ' ' + private.group(1).strip(), text, count=1, flags=re.M)
		else: text += '\n%s: %s\n' % (key, private.group(1).strip())
	open(path, 'w').write(text)
PY
}
build_mpv() {
	meson_build --default-library=shared \
		-Dcplayer=false -Dlibmpv=true -Dgpl=true -Dbuild-date=false -Dmanpage-build=disabled -Dtests=false -Dzlib=enabled -Dlcms2=enabled \
		-Dlua=disabled -Djavascript=disabled -Diconv=disabled -Dcdda=disabled -Dcplugins=disabled -Ddvbin=disabled -Ddvdnav=disabled -Djpeg=disabled -Dlibarchive=disabled -Dlibavdevice=disabled -Dlibbluray=disabled -Drubberband=disabled -Duchardet=disabled -Dvapoursynth=disabled -Dzimg=disabled -Dsdl2-gamepad=disabled \
		-Dvulkan=disabled -Dcuda-hwaccel=disabled -Dcuda-interop=disabled \
		"${mpv[@]}"
}
case $platform in
windows)
	fonts=--enable-directwrite
	placebo="-Dd3d11=enabled -Dshaderc=enabled"
	ffmpeg="--enable-d3d11va --enable-dxva2"
	mpv=(-Dprefer_static=true -Dwin32-threads=enabled -Dwasapi=enabled -Dgl-win32=enabled -Ddirect3d=enabled -Dd3d11=enabled -Dshaderc=enabled -Dspirv-cross=enabled -Dwin32-smtc=enabled
	     -Degl-angle=disabled -Degl-angle-lib=disabled -Degl-angle-win32=disabled
	     -Dc_link_args="-static -static-libgcc" -Dcpp_link_args="-static -static-libgcc -static-libstdc++");;
linux)
	fonts=--enable-fontconfig                 # the one font provider linux has, which libass requires one of; the distribution's fontconfig, which every desktop carries
	placebo="-Dd3d11=disabled -Dshaderc=disabled -Dgl-proc-addr=disabled"
	ffmpeg="--enable-pic"                     # its own switch for the same thing as -fPIC above, since its configure writes its own flags
	mpv=(-Dprefer_static=false -Dx11=enabled -Dgl-x11=enabled -Degl=disabled -Degl-x11=disabled -Dwayland=disabled -Ddrm=disabled -Dgbm=disabled -Dxv=disabled -Dvaapi=disabled -Dvdpau=disabled -Dx11-clipboard=disabled
	     -Dalsa=enabled -Dpulse=enabled -Dpipewire=disabled -Djack=disabled -Dsndio=disabled -Doss-audio=disabled -Dopenal=disabled -Dsdl2-audio=disabled
	     -Dcocoa=disabled -Dgl-cocoa=disabled -Dcoreaudio=disabled -Dswift-build=disabled);;
mac)
	fonts=--enable-coretext
	placebo="-Dd3d11=disabled -Dshaderc=disabled -Dgl-proc-addr=disabled"  # no EGL or GLX to look a function up through; the host hands libplacebo the loader
	ffmpeg="--enable-videotoolbox"
	mpv=(-Dprefer_static=true -Dcocoa=enabled -Dgl-cocoa=enabled -Dvideotoolbox-gl=enabled -Dvideotoolbox-pl=disabled -Dcoreaudio=enabled -Daudiounit=disabled -Davfoundation=disabled
	     -Dswift-build=enabled -Dmacos-cocoa-cb=disabled -Dmacos-media-player=disabled -Dmacos-touchbar=disabled
	     "-Dswift-flags=-Xfrontend -disable-incremental-llvm-codegen"   # the Swift compiler writes a hash of each compile into its object, for incremental builds of its own, and the hash takes in a temporary path that differs every run; the linker folds it into the dylib's UUID, and with nothing else different two builds disagreed in those sixteen bytes alone, found on the Mac mini on 2026-10-15. This leaves the hash constant, and two builds give one dylib
	     -Dx11=disabled -Dwayland=disabled -Degl=disabled -Ddrm=disabled -Dvaapi=disabled -Dvdpau=disabled
	     -Dalsa=disabled -Djack=disabled -Dpipewire=disabled -Dpulse=disabled -Dsndio=disabled -Doss-audio=disabled -Dopenal=disabled);;
esac

step zlib-1.3.2 build_zlib
step freetype-2.15.0 build_freetype
step fribidi-1.0.17 build_fribidi
step harfbuzz-14.6.0 build_harfbuzz
step libass-0.17.5 build_libass $fonts                   # the platform's flags after the function are for the stamp; the function reads them itself
if [ $platform = windows ]; then
step SPIRV-Cross-vulkan-sdk-1.4.363.0 build_spirv_cross
step shaderc-2026.4 build_shaderc
fi
step lcms2-2.19 build_lcms2
step libplacebo-7.360.1 build_libplacebo $placebo
step dav1d-1.5.4 build_dav1d
step ffmpeg-9.0.2 build_ffmpeg $ffmpeg
if [ $platform = linux ]; then fold_private; fi
step mpv-0.41.0 build_mpv "${mpv[@]}"

# the result: stripped of its symbols, which on windows takes it from 40 MB to 34, and put where the platform's configuration names it, with the record described at the top beside it
mkdir -p $tauri/binaries
case $platform in
windows)
	strip -o $tauri/binaries/libmpv-2.dll $prefix/bin/libmpv-2.dll
	library=libmpv-2.dll;;
mac)
	strip -x -o $tauri/binaries/libmpv.2.dylib $prefix/lib/libmpv.2.dylib   # -x keeps the exported symbols a library is for, and drops the rest
	install_name_tool -id @rpath/libmpv.2.dylib $tauri/binaries/libmpv.2.dylib
	codesign --force --sign - $tauri/binaries/libmpv.2.dylib
	ln -sf libmpv.2.dylib $tauri/binaries/libmpv.dylib
	library=libmpv.2.dylib;;
linux)
	strip -o $tauri/binaries/libmpv.so.2 $prefix/lib/libmpv.so.2
	ln -sf libmpv.so.2 $tauri/binaries/libmpv.so
	library=libmpv.so.2;;
esac
tools_json=${tools//\\/\\\\}; tools_json=${tools_json//\"/\\\"}   # the one value a compiler could put a quote in
cat > $tauri/binaries/libmpv.json <<EOF
{
	"file": "$library",
	"bytes": $(bytes $tauri/binaries/$library),
	"sha256": "$(sha256 $tauri/binaries/$library)",
	"built": "$(date -u +%Y-%m-%d)",
	"recipe": "$(sha256 $here/build.sh)",
	"tools": "$tools_json",
	"sources": [${sources%,}
	]
}
EOF
echo "== $library $(bytes $tauri/binaries/$library) bytes, sha256 $(sha256 $tauri/binaries/$library), in src-tauri/binaries with libmpv.json beside it"
