#!/usr/bin/env python3
"""
libmpv for fuji, built from release tarballs on the Mac, on Windows and in the Linux containers: one library with everything inside it, and nothing in it from anywhere but these sources. The same sources at the same versions on each platform, so the library the About section names is the recipe's and every package carries the same work.

On Windows, twelve packages, each a tagged release fetched from its own project and checked against the hash written here, built static in dependency order into a prefix under target, and the last of them, mpv, linked as the one shared library, libmpv-2.dll, which carries the other eleven and the mingw runtime inside it. It imports Windows' own libraries and the universal C runtime and nothing else: no Vulkan, which a graphics driver installs and Windows does not, so no machine fails to start fuji for the lack of one, and Direct3D is loaded at runtime by mpv itself, so neither is a Direct3D library in the import table.

What is in it, and why. FFmpeg with every decoder and demuxer, dav1d beside it for AV1, which FFmpeg decodes in software through no other library, and both of Windows' hardware decoding paths, D3D11VA and DXVA2; without encoders, muxers, devices or network, since fuji only ever plays what is on the disk. mpv without the player, the scripting languages, the disc formats and the frame servers; with WASAPI for sound, Direct3D 11 for a picture, which every Windows 10 draws with and which hardware decoding hands frames to without a copy, OpenGL and Direct3D 9 beneath it for the machines that want them, and the media keys through Windows' own transport controls. libplacebo, mpv's renderer, with Direct3D 11 and without Vulkan, which Direct3D covers on Windows. Direct3D takes shaders in HLSL and mpv writes them in GLSL, so between them sit shaderc, the GLSL compiler to SPIR-V, with glslang, SPIRV-Tools and SPIRV-Headers built inside it at the commits its DEPS file names, and SPIRV-Cross, the translator from SPIR-V to HLSL. lcms2 for ICC profiles, so a calibrated display is honored. libass for subtitles, over FreeType, HarfBuzz and FriBidi, with DirectWrite finding the fonts. zlib under FFmpeg and FreeType.

The Windows toolchain is MSYS2's ucrt64: gcc, meson, ninja, cmake, pkgconf, nasm and the C++/WinRT headers, which libmpv.js beside this installs the first time and starts this script under, with MSYS2's own Python, the one that spells a path /c/Users the way the twelve projects' configure scripts expect, rather than the native one meson brings. Only the toolchain comes from MSYS2; PKG_CONFIG_LIBDIR is pinned to the prefix so no MSYS2 package can be found by accident, and the dll's imports are the proof. The build is about twenty minutes on the Windows box, FFmpeg nine of them and shaderc four, and a package once built is skipped on the next run, so a change to mpv's options alone costs a minute. The result is stripped and copied to src-tauri/binaries/libmpv-2.dll, untracked, where tauri.windows.conf.json names it and the installer picks it up. It replaced a nightly from a build farm, a thousand commits past the release on an FFmpeg with no number, kept thirty days and never the same bytes twice, and the installer carrying it went from 38.6 MB to 20.8.

On the Mac the same recipe makes libmpv.2.dylib, arm64, for macOS 11 and later, which is every Mac with Apple silicon, from ten of the twelve: FFmpeg with VideoToolbox for the hardware decoder; mpv with CoreAudio for sound and the OpenGL output, with VideoToolbox's frames handed to it without a copy, which is the output mpv's own Mac window draws through and the one the render API offers; libplacebo with OpenGL alone, since OpenGL takes shaders as mpv writes them and wants no compiler, so the two Khronos packages are Windows' alone; and CoreText finding the fonts for libass. Vulkan over MoltenVK is left out until a reason appears. mpv's Cocoa code, which its OpenGL backend and so the render API's OpenGL path stand on, includes a header its Swift build generates, so the Swift bridge is built with Apple's own swiftc, and the pieces above it, mpv's own window, media keys and touch bar, which a library drawing into fuji's view never shows, are left out. The toolchain is Homebrew's, meson, ninja and pkgconf over Apple's clang and Swift, which libmpv.js beside this names when one is missing; cmake and nasm stay on Windows, since no package here is built with cmake and arm64 assembly goes through the compiler, and nothing of Homebrew's is in the result: PKG_CONFIG_LIBDIR is pinned to the prefix as on Windows, and the dylib names Apple's own frameworks and libraries and nothing else, which otool -L shows. The result is stripped, told that its name is @rpath/libmpv.2.dylib, which the rpath Tauri gives the app resolves to Contents/Frameworks in the bundle and to target/Frameworks beside a development build, signed ad hoc since stripping breaks the signature the linker gave it, and copied to src-tauri/binaries/libmpv.2.dylib, untracked, where tauri.macos.conf.json names it and Tauri carries it into the bundle; libmpv.dylib beside it is the unversioned name the linker looks for.

On Linux the same recipe makes libmpv.so.2 inside the containers the linux workspace runs, from the same ten packages as the Mac, over the toolchain the tauri image installs from Debian 12: FFmpeg decoding in software, since a hardware path there means VAAPI and a dependency on the machine's driver stack, which waits for a measurement; mpv with ALSA and PulseAudio for sound, the two every desktop has and PipeWire answers for, and the OpenGL output reached through X11, which is the lightest of mpv's ways to have OpenGL at all and is never used as a window, since the render API draws into fuji's own view; libplacebo with OpenGL alone; and fontconfig finding libass's fonts. The library names the distribution's X11, OpenGL, fontconfig and sound libraries and the C library, which the packages depend on by name, and nothing else. The result is stripped and copied to src-tauri/binaries/libmpv.so.2, where tauri.linux.conf.json names it as a resource the deb and the rpm install beside the application's other files and the flatpak carries into its sandbox, and the binary finds it by an rpath relative to itself.

Beside the library goes libmpv.json, the record of what was made and where it came from: the library's hash and size, the moment, the hash of this file, the tools, and every source with its hash. pnpm hash copies that record into the release's sidecar, which git tracks, so the repository says of every release which library it carried and where each part of it came from, long after this recipe has moved on; and it refuses a library this file as checked out did not build. So an edit here, to a comment as much as to a flag, means running this again before a release is staged, which costs seconds when nothing has to be rebuilt, since a package already built is skipped under a stamp of what built it.

The same record goes into the ledger: four files at the root of the repository, library-mac.txt, library-win.txt, library-amd.txt and library-arm.txt, one per library, each appended by every run of this that compiled anything and never otherwise edited, and tracked by git. An entry is the library's line, the recipe's, the tools', and one line per source, hashes whole, so a reader of the repository sees for each library when it was built, from what, checked against which hashes, with which compilers, and what came out; two entries with one hash on different days are the reproducibility proof, written where it belongs. pnpm hash refuses a library with no entry in its file, so no release carries a library the ledger does not know. On the Mac and Windows this file appends the entry itself; inside a Linux container it writes the entry beside the record and the driver on the Mac appends it, since a container never writes into the working tree.

This is Python rather than the shell script it replaced because the bookkeeping outgrew the shell: the sources are a table, the record is written by the JSON library, a stamp hashes a function's own source, and every command is started with its arguments as a list that no shell splits or quotes. Python is already on every build platform, since meson is written in it, so it adds nothing to install.
"""
import hashlib
import inspect
import json
import os
import platform
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

system = os.uname().sysname if hasattr(os, 'uname') else 'Windows'
if system == 'Darwin': host = 'mac'
elif system.startswith(('MINGW', 'MSYS')) or system == 'Windows': host = 'windows'
elif system == 'Linux': host = 'linux'
else: sys.exit(f'build.py: no section for {system}')
architecture = {'aarch64': 'arm64', 'amd64': 'x86_64'}.get(platform.machine().lower(), platform.machine().lower())  #arm64 or x86_64, by the two names the machines use for each
ledger = {'mac': 'mac', 'windows': 'win'}.get(host) or ('arm' if architecture == 'arm64' else 'amd')  #which of the four library files this library's entries go in
here = Path(__file__).resolve().parent
tauri = here.parent / 'src-tauri'
repository = here.parent.parent  #the monorepo root, where the four library files are; inside a container this is the staged copy, which dies with it, so there the driver carries the entry out
root = tauri / 'target' / 'libmpv'  #the sources, the build trees and the prefix, under tauri's own build output so a clean build clears them too
src = root / 'src'
prefix = root / 'prefix'
jobs = os.cpu_count() or 4

def say(text): print(text, flush=True)  #a line of progress, flushed so it lands in order with the output of the programs run between
def run(arguments, cwd=None, quiet=False):  #one program, with the environment built below, failing the build if it fails
	subprocess.run([str(a) for a in arguments], cwd=cwd, env=environment, check=True, stdout=subprocess.DEVNULL if quiet else None, stderr=subprocess.DEVNULL if quiet else None)
def read(arguments):  #one program's first line of output, for a version or a path
	return subprocess.run([str(a) for a in arguments], env=environment, check=True, capture_output=True, text=True).stdout.strip().splitlines()[0]
def sha256(path):  #the hash of a file
	digest = hashlib.sha256()
	with open(path, 'rb') as file:
		for block in iter(lambda: file.read(1 << 20), b''): digest.update(block)
	return digest.hexdigest()
def sha256_text(text): return hashlib.sha256(text.encode()).hexdigest()  #and of text

environment = dict(os.environ)
if host == 'windows' and not environment.get('USERPROFILE'): environment['USERPROFILE'] = read(['cygpath', '-w', environment['HOME']])  #meson's cmake lookups ask python for the home folder, and the windows python reads this
environment['PKG_CONFIG_PATH'] = f'{prefix}/lib/pkgconfig'
environment['PKG_CONFIG_LIBDIR'] = f'{prefix}/lib/pkgconfig'  #only our prefix, never msys2's or homebrew's own packages
if host == 'linux': environment['PKG_CONFIG_LIBDIR'] += f":/usr/lib/{read(['gcc', '-print-multiarch'])}/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig"  #and on linux the distribution's after it, for the handful the library names: fontconfig, x11, opengl and the two sound libraries, each asked for below
cflags = f'-O2 -pipe -I{prefix}/include'
ldflags = f'-L{prefix}/lib'
if host == 'linux': cflags += ' -fPIC'  #x86-64 code is not position independent unless asked, and every static library here ends up inside one shared one; arm64 and the other two platforms are already

"""
The same sources make the same bytes, so a library's hash names the recipe and the tools rather than the afternoon, and four settings are what make that true. SOURCE_DATE_EPOCH is the time Windows' linker and strip stamp into every PE header, pinned to the day this recipe was written; the Mac and Linux stamp no time and read it anyway. ZERO_AR_DATE keeps Apple's archiver from writing a timestamp into each member of a static library. The Swift compiler writes a hash of each compile into its object, for incremental builds of its own, and that hash takes in a temporary path that differs every run, so mpv's Swift bridge is compiled with it disabled, the -Xfrontend flag among mpv's options below. And Apple's linker writes local symbol names into the dylib in whichever order its threads finish, then hashes the whole file into the dylib's UUID before strip removes those names, so the link runs with -x, which puts no local symbols in at all; the linker's own -reproducible option did not settle the order. The last two were found on the Mac mini on 2026-10-15 by building twice and comparing, each as sixteen bytes of UUID and nothing else. What the hash still names is the folder the library was built in, since mpv and FFmpeg write their configure lines into the binary, prefix and all; the containers build at one path every time, and the Mac at this checkout's.
"""
if host == 'mac': environment.update(MACOSX_DEPLOYMENT_TARGET='11.0', ZERO_AR_DATE='1')  #the floor every Apple silicon Mac clears, and the archiver's timestamps off
if host == 'mac': ldflags += ' -Wl,-x'  #no local symbols in the dylib, which strip would take out anyway, so the linker's threads have nothing to order
environment.update(CFLAGS=cflags, CXXFLAGS=cflags, LDFLAGS=ldflags)
environment['SOURCE_DATE_EPOCH'] = '1791763200'  #2026-10-12

#the sources: name, where its tarball is, and the sha256 of the tarball; libplacebo publishes no tarball with its submodules in it, so it is cloned at its tag and the commit checked instead. Each is also written down for the record at the end, fetched this run or found from the last
sources = []
def fetch(name, url, sha):
	sources.append({'name': name, 'url': url, 'sha256': sha})
	file = src / url.rsplit('/', 1)[1]
	if (src / name).is_dir(): return
	say(f'== fetching {name}')
	run(['curl', '-fsSL', '--retry', '3', '-o', file, url])  #-f so a server's error page is never saved as the archive, and three tries, since the hash check below would refuse a short download anyway
	if sha256(file) != sha: sys.exit(f'{file} does not match the hash written in build.py')
	run(['tar', '-xf', file, '-C', src])
def clone(name, url, tag, commit):
	sources.append({'name': name, 'url': url, 'commit': commit})
	if (src / name).is_dir(): return
	say(f'== cloning {name}')
	run(['git', 'clone', '-q', '--recursive', '--branch', tag, '--depth', '1', url, src / name])
	if read(['git', '-C', src / name, 'rev-parse', 'HEAD']) != commit: sys.exit(f'{name} at {tag} is not commit {commit}')
def clone_commit(folder, url, commit):  #a repository at one commit rather than a tag, which is how shaderc's DEPS file names the three libraries it builds with; fetched into the folder shaderc's cmake looks in
	sources.append({'name': folder.name, 'url': url, 'commit': commit})
	if folder.is_dir(): return
	say(f'== cloning {folder.name} at {commit}')
	folder.mkdir(parents=True)
	run(['git', '-C', folder, 'init', '-q'])
	run(['git', '-C', folder, 'fetch', '-q', '--depth', '1', url, commit])
	run(['git', '-C', folder, 'checkout', '-q', 'FETCH_HEAD'])

#one package: skipped if its marker in the prefix holds this run's stamp, otherwise built in its source folder by its function and marked. The stamp is the hash of that function's source, the two build helpers', the flags every package gets, the ones named after it, which are the platform's for that package, and the stamp of the package before it; so a flag changed here rebuilds the package rather than leaving last run's in the prefix under a recipe that never made it, and rebuilds every package after it too, since each is linked into the next, and a run stopped partway leaves nothing marked current that was built against what came before. A new version is a new name, and so a new folder and marker
chain = ''
compiled = False  #whether this run compiled any package, which is what earns the library an entry in the ledger
def step(name, build, *flags):
	global chain, compiled
	stamp = sha256_text(f'{chain} {inspect.getsource(build)} {inspect.getsource(meson_build)} {inspect.getsource(cmake_build)} {cflags} {cflags} {ldflags} {" ".join(flags)}')
	chain = stamp
	marker = prefix / f'.{name}'
	if marker.is_file() and marker.read_text().strip() == stamp: say(f'== {name} already built'); return
	say(f'== {name}'); started = time.time()
	build(src / name, *flags)
	marker.write_text(stamp + '\n')
	compiled = True
	say(f'== {name} done in {time.time() - started:.0f} s')
def meson_build(folder, *options):  #libdir said outright, because debian's meson would put libraries and their pkg-config files under lib/<triplet>, where PKG_CONFIG_LIBDIR above never looks
	shutil.rmtree(folder / 'build', ignore_errors=True)
	run(['meson', 'setup', 'build', f'--prefix={prefix}', '--libdir=lib', '--buildtype=release', *options], cwd=folder)
	run(['ninja', '-C', 'build', f'-j{jobs}', 'install'], cwd=folder)
def cmake_build(folder, *options):
	shutil.rmtree(folder / 'build', ignore_errors=True)
	run(['cmake', '-G', 'Ninja', '-B', 'build', '-DCMAKE_BUILD_TYPE=Release', f'-DCMAKE_INSTALL_PREFIX={prefix}', '-DCMAKE_INSTALL_LIBDIR=lib', '-DBUILD_SHARED_LIBS=OFF', *options], cwd=folder)
	run(['ninja', '-C', 'build', f'-j{jobs}', 'install'], cwd=folder)
def clean(folder):  #the three built by configure and make start from a clean tree, as the meson and cmake builds do by removing their build folder, so a rebuild never keeps an object compiled under the flags before
	subprocess.run(['make', 'distclean'], cwd=folder, env=environment, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
def configure_make(folder, *options):  #the three autotools packages: their own configure script, run by sh as its first line asks, then make
	run(['sh', './configure', f'--prefix={prefix}', *options], cwd=folder)
	run(['make', f'-j{jobs}'], cwd=folder)
	run(['make', 'install'], cwd=folder)
def pc_rename(folder, old, new, libs):  #a pkg-config file under the name of a package's shared build, listing the static libraries instead: mpv and libplacebo ask for the two Khronos packages by their shared names
	lines = (prefix / 'lib/pkgconfig' / f'{old}.pc').read_text().splitlines()
	lines = [f'Name: {new}' if line == f'Name: {old}' else f'Libs: {libs}' if line.startswith('Libs: ') else line for line in lines]
	(prefix / 'lib/pkgconfig' / f'{new}.pc').write_text('\n'.join(lines) + '\n')

def build_zlib(folder): clean(folder); configure_make(folder, '--static')
def build_freetype(folder): meson_build(folder, '--default-library=static', '-Dzlib=system', '-Dharfbuzz=disabled', '-Dpng=disabled', '-Dbrotli=disabled', '-Dbzip2=disabled', '-Dtests=disabled')
def build_fribidi(folder): meson_build(folder, '--default-library=static', '-Ddocs=false', '-Dbin=false', '-Dtests=false')
def build_harfbuzz(folder): meson_build(folder, '--default-library=static', '-Dfreetype=enabled', '-Dglib=disabled', '-Dgobject=disabled', '-Dcairo=disabled', '-Dchafa=disabled', '-Dicu=disabled', '-Dgdi=disabled', '-Ddirectwrite=disabled', '-Dcoretext=disabled', '-Dpng=disabled', '-Dtests=disabled', '-Ddocs=disabled', '-Dintrospection=disabled', '-Dutilities=disabled')
def build_libass(folder, fonts):  #the platform's own font provider, turned on after the three are turned off, since a later flag wins
	clean(folder); configure_make(folder, '--disable-shared', '--enable-static', '--disable-directwrite', '--disable-coretext', '--disable-fontconfig', fonts, '--disable-libunibreak')
def build_spirv_cross(folder):  #the translator from SPIR-V to HLSL, which is how a GLSL shader reaches Direct3D; its file under the shared name lists the static libraries the C API is made of and the C++ runtime they need
	cmake_build(folder, '-DSPIRV_CROSS_STATIC=ON', '-DSPIRV_CROSS_SHARED=OFF', '-DSPIRV_CROSS_CLI=OFF', '-DSPIRV_CROSS_ENABLE_TESTS=OFF', '-DSPIRV_CROSS_ENABLE_MSL=OFF', '-DSPIRV_CROSS_ENABLE_CPP=OFF', '-DSPIRV_CROSS_ENABLE_REFLECT=OFF', '-DSPIRV_CROSS_ENABLE_UTIL=OFF')
	pc_rename(folder, 'spirv-cross-c', 'spirv-cross-c-shared', '-L${libdir} -lspirv-cross-c -lspirv-cross-hlsl -lspirv-cross-glsl -lspirv-cross-core -lstdc++')
def build_shaderc(folder):  #the GLSL compiler to SPIR-V, with glslang, SPIRV-Tools and SPIRV-Headers built inside it from the commits its DEPS file names; the combined static library's file is copied under shaderc's own name
	cmake_build(folder, '-DSHADERC_SKIP_TESTS=ON', '-DSHADERC_SKIP_EXAMPLES=ON', '-DSHADERC_SKIP_COPYRIGHT_CHECK=ON', '-DSHADERC_ENABLE_WGSL_OUTPUT=OFF', '-DSPIRV_SKIP_EXECUTABLES=ON', '-DENABLE_GLSLANG_BINARIES=OFF')
	pc_rename(folder, 'shaderc_combined', 'shaderc', '-L${libdir} -lshaderc_combined -lstdc++')
def build_lcms2(folder): meson_build(folder, '--default-library=static', '-Dtests=disabled', '-Djpeg=disabled', '-Dtiff=disabled', '-Dutils=false', '-Dfastfloat=false', '-Dthreaded=false')
def build_libplacebo(folder, *placebo): meson_build(folder, '--default-library=static', '-Dvulkan=disabled', '-Dvk-proc-addr=disabled', '-Dopengl=enabled', '-Dglslang=disabled', '-Dlcms=enabled', '-Dlibdovi=disabled', '-Dunwind=disabled', '-Dxxhash=disabled', '-Ddemos=false', '-Dtests=false', *placebo)
def build_dav1d(folder): meson_build(folder, '--default-library=static', '-Denable_tools=false', '-Denable_tests=false', '-Denable_examples=false', '-Denable_docs=false')
def build_ffmpeg(folder, *ffmpeg):
	clean(folder)
	configure_make(folder, '--pkg-config-flags=--static', '--enable-static', '--disable-shared', '--disable-debug', '--disable-doc', '--disable-programs',
		'--disable-autodetect', '--enable-zlib', '--enable-libdav1d', *ffmpeg,
		'--disable-encoders', '--disable-muxers', '--disable-devices', '--disable-avdevice', '--disable-network', '--disable-protocols', '--enable-protocol=file')
def fold_private():  #linux only: our own libraries are static and nothing else, so each one's private link line and requirements, which pkg-config gives out only when asked for a static link, become its public ones; then mpv's build, asking for a plain link, gets our ten whole and the distribution's fontconfig, X11, OpenGL and sound libraries as shared objects, where asking for static links throughout pulled the distribution's static fontconfig in and the link failed
	import re
	for path in sorted((prefix / 'lib/pkgconfig').glob('*.pc')):
		text = path.read_text()
		for key in ('Libs', 'Requires'):
			private = re.search(r'^%s\.private:(.*)$' % key, text, re.M)
			if not private: continue
			text = re.sub(r'^%s\.private:.*\n?' % key, '', text, flags=re.M)
			if re.search(r'^%s:' % key, text, re.M): text = re.sub(r'^(%s:.*)$' % key, lambda m: m.group(1) + ' ' + private.group(1).strip(), text, count=1, flags=re.M)
			else: text += '\n%s: %s\n' % (key, private.group(1).strip())
		path.write_text(text)
def build_mpv(folder, *mpv):
	meson_build(folder, '--default-library=shared',
		'-Dcplayer=false', '-Dlibmpv=true', '-Dgpl=true', '-Dbuild-date=false', '-Dmanpage-build=disabled', '-Dtests=false', '-Dzlib=enabled', '-Dlcms2=enabled',
		'-Dlua=disabled', '-Djavascript=disabled', '-Diconv=disabled', '-Dcdda=disabled', '-Dcplugins=disabled', '-Ddvbin=disabled', '-Ddvdnav=disabled', '-Djpeg=disabled', '-Dlibarchive=disabled', '-Dlibavdevice=disabled', '-Dlibbluray=disabled', '-Drubberband=disabled', '-Duchardet=disabled', '-Dvapoursynth=disabled', '-Dzimg=disabled', '-Dsdl2-gamepad=disabled',
		'-Dvulkan=disabled', '-Dcuda-hwaccel=disabled', '-Dcuda-interop=disabled',
		*mpv)

#the platform's flags for the four packages that differ: libass's font provider, libplacebo's and FFmpeg's hardware, and mpv's outputs
if host == 'windows':
	fonts = '--enable-directwrite'
	placebo = ['-Dd3d11=enabled', '-Dshaderc=enabled']
	ffmpeg = ['--enable-d3d11va', '--enable-dxva2']
	mpv = ['-Dprefer_static=true', '-Dwin32-threads=enabled', '-Dwasapi=enabled', '-Dgl-win32=enabled', '-Ddirect3d=enabled', '-Dd3d11=enabled', '-Dshaderc=enabled', '-Dspirv-cross=enabled', '-Dwin32-smtc=enabled',
		'-Degl-angle=disabled', '-Degl-angle-lib=disabled', '-Degl-angle-win32=disabled',
		'-Dc_link_args=-static -static-libgcc', '-Dcpp_link_args=-static -static-libgcc -static-libstdc++']
elif host == 'linux':
	fonts = '--enable-fontconfig'  #the one font provider linux has, which libass requires one of; the distribution's fontconfig, which every desktop carries
	placebo = ['-Dd3d11=disabled', '-Dshaderc=disabled', '-Dgl-proc-addr=disabled']
	ffmpeg = ['--enable-pic']  #its own switch for the same thing as -fPIC above, since its configure writes its own flags
	mpv = ['-Dprefer_static=false', '-Dx11=enabled', '-Dgl-x11=enabled', '-Degl=disabled', '-Degl-x11=disabled', '-Dwayland=disabled', '-Ddrm=disabled', '-Dgbm=disabled', '-Dxv=disabled', '-Dvaapi=disabled', '-Dvdpau=disabled', '-Dx11-clipboard=disabled',
		'-Dalsa=enabled', '-Dpulse=enabled', '-Dpipewire=disabled', '-Djack=disabled', '-Dsndio=disabled', '-Doss-audio=disabled', '-Dopenal=disabled', '-Dsdl2-audio=disabled',
		'-Dcocoa=disabled', '-Dgl-cocoa=disabled', '-Dcoreaudio=disabled', '-Dswift-build=disabled']
else:
	fonts = '--enable-coretext'
	placebo = ['-Dd3d11=disabled', '-Dshaderc=disabled', '-Dgl-proc-addr=disabled']  #no EGL or GLX to look a function up through; the host hands libplacebo the loader
	ffmpeg = ['--enable-videotoolbox']
	mpv = ['-Dprefer_static=true', '-Dcocoa=enabled', '-Dgl-cocoa=enabled', '-Dvideotoolbox-gl=enabled', '-Dvideotoolbox-pl=disabled', '-Dcoreaudio=enabled', '-Daudiounit=disabled', '-Davfoundation=disabled',
		'-Dswift-build=enabled', '-Dmacos-cocoa-cb=disabled', '-Dmacos-media-player=disabled', '-Dmacos-touchbar=disabled',
		'-Dswift-flags=-Xfrontend -disable-incremental-llvm-codegen',  #the per-compile hash left out of the Swift objects, as the essay on reproducibility above says; one argument with a space in it, which meson splits
		'-Dx11=disabled', '-Dwayland=disabled', '-Degl=disabled', '-Ddrm=disabled', '-Dvaapi=disabled', '-Dvdpau=disabled',
		'-Dalsa=disabled', '-Djack=disabled', '-Dpipewire=disabled', '-Dpulse=disabled', '-Dsndio=disabled', '-Doss-audio=disabled', '-Dopenal=disabled']

def main():
	src.mkdir(parents=True, exist_ok=True); prefix.mkdir(parents=True, exist_ok=True)
	tools = f"{read(['gcc', '--version'])}; meson {read(['meson', '--version'])}; ninja {read(['ninja', '--version'])}"  #the toolchain the package manager provided, said at the top of every run and written into the record at the end, since it is the one part of a build this file does not pin: a library hash that moved with nothing changed here is answered by this line
	tools += '; ' + (subprocess.run(['swiftc', '--version'], env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True).stdout.strip().splitlines()[0] if host == 'mac' else read(['nasm', '--version']))  #swiftc says its version on whichever stream it likes
	say(f'== tools: {tools}')

	fetch('zlib-1.3.2',       'https://github.com/madler/zlib/releases/download/v1.3.2/zlib-1.3.2.tar.xz',                 'd7a0654783a4da529d1bb793b7ad9c3318020af77667bcae35f95d0e42a792f3')  #the same bytes zlib.net serves, from the release page that keeps every version: zlib.net serves the current version alone and takes the one before down the day a new one ships, so 1.3.1 answered 404 there on 2026-10-15
	fetch('freetype-2.15.0',  'https://download-mirror.savannah.gnu.org/releases/freetype/freetype-2.15.0.tar.xz',         '93cd47eb719c325107f2d3cf087d06299e944c3b48a40b55537302304dea1dd5')  #savannah's own mirror by name, since its download address bounces to a random one, and one of those answered 404 and another with a page in place of the archive
	fetch('fribidi-1.0.17',   'https://github.com/fribidi/fribidi/releases/download/v1.0.17/fribidi-1.0.17.tar.xz',        '6949dcde27d41cebad1fd741fcafc36d55a1020d2d872d4a6eb3914caabbada2')
	fetch('harfbuzz-14.6.0',  'https://github.com/harfbuzz/harfbuzz/releases/download/14.6.0/harfbuzz-14.6.0.tar.xz',     'd07a007327277708a2a73ae437887cdbaf282937f6d03ca5467723e9099af586')
	fetch('libass-0.17.5',    'https://github.com/libass/libass/releases/download/0.17.5/libass-0.17.5.tar.xz',            '2dca25c0e0c837ddf00b52011b3f82cac1e4ddd3ad018227806b0c2288864acc')
	if host == 'windows':  #the shader compiler and translator, which Direct3D wants and OpenGL does not
		fetch('SPIRV-Cross-vulkan-sdk-1.4.363.0', 'https://github.com/KhronosGroup/SPIRV-Cross/archive/refs/tags/vulkan-sdk-1.4.363.0.tar.gz', 'f708982e88b763ef5b0394ed468fc2c3628f68cdab89f2e7a946cd628c04e721')
		fetch('shaderc-2026.4',   'https://github.com/google/shaderc/archive/refs/tags/v2026.4.tar.gz',                         'f06ce5bcca94e5df7f34e115743597d0ad2e13c5fe9213c67dc8c76031241947')
		clone_commit(src / 'shaderc-2026.4/third_party/glslang',       'https://github.com/KhronosGroup/glslang.git',       'e1b562a8bed273a02f30b59b66a5d499793cede5')
		clone_commit(src / 'shaderc-2026.4/third_party/spirv-tools',   'https://github.com/KhronosGroup/SPIRV-Tools.git',   'ef96ed763b43b59b33b31b362f09a02b729fa1c9')
		clone_commit(src / 'shaderc-2026.4/third_party/spirv-headers', 'https://github.com/KhronosGroup/SPIRV-Headers.git', '04fd3caa1e8267e4d95c806cad901181728e1006')
	fetch('lcms2-2.19',       'https://github.com/mm2/Little-CMS/releases/download/lcms2.19/lcms2-2.19.tar.gz',           '49e7e134e4299733dd0eda434fa468997a28ab3d33fa397c642b03644f552216')
	clone('libplacebo-7.360.1', 'https://code.videolan.org/videolan/libplacebo.git', 'v7.360.1', 'cee9b076f2c63104ccfd497fa79c39a867293ec4')
	fetch('dav1d-1.5.4',      'https://downloads.videolan.org/pub/videolan/dav1d/1.5.4/dav1d-1.5.4.tar.xz',                '686616b7c69eb88d44459391ab25cac13b6647a3b288835c5784e71c1514a5c5')
	fetch('ffmpeg-9.0.2',     'https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz',                                           '8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e')
	fetch('mpv-0.41.0',       'https://github.com/mpv-player/mpv/archive/refs/tags/v0.41.0.tar.gz',                        'ee21092a5ee427353392360929dc64645c54479aefdb5babc5cfbb5fad626209')

	step('zlib-1.3.2', build_zlib)
	step('freetype-2.15.0', build_freetype)
	step('fribidi-1.0.17', build_fribidi)
	step('harfbuzz-14.6.0', build_harfbuzz)
	step('libass-0.17.5', build_libass, fonts)  #the platform's flags after the function are for the stamp, and the function takes them
	if host == 'windows':
		step('SPIRV-Cross-vulkan-sdk-1.4.363.0', build_spirv_cross)
		step('shaderc-2026.4', build_shaderc)
	step('lcms2-2.19', build_lcms2)
	step('libplacebo-7.360.1', build_libplacebo, *placebo)
	step('dav1d-1.5.4', build_dav1d)
	step('ffmpeg-9.0.2', build_ffmpeg, *ffmpeg)
	if host == 'linux': fold_private()
	step('mpv-0.41.0', build_mpv, *mpv)

	#the result: stripped of its symbols, which on windows takes it from 40 MB to 34, and put where the platform's configuration names it, with the record described at the top beside it
	binaries = tauri / 'binaries'
	binaries.mkdir(parents=True, exist_ok=True)
	if host == 'windows':
		library = 'libmpv-2.dll'
		run(['strip', '-o', binaries / library, prefix / 'bin' / library])
	elif host == 'mac':
		library = 'libmpv.2.dylib'
		run(['strip', '-x', '-o', binaries / library, prefix / 'lib' / library])  #-x keeps the exported symbols a library is for, and drops the rest
		run(['install_name_tool', '-id', f'@rpath/{library}', binaries / library])
		run(['codesign', '--force', '--sign', '-', binaries / library])
		link(binaries / 'libmpv.dylib', library)
	else:
		library = 'libmpv.so.2'
		run(['strip', '-o', binaries / library, prefix / 'lib' / library])
		link(binaries / 'libmpv.so', library)
	record = {
		'file': library,
		'bytes': (binaries / library).stat().st_size,
		'sha256': sha256(binaries / library),
		'built': datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'),
		'recipe': sha256(__file__),
		'tools': tools,
		'sources': sources,
	}
	(binaries / 'libmpv.json').write_text(json.dumps(record, indent='\t') + '\n')
	entry = ledger_entry(record)
	(binaries / 'library.txt').write_text(entry)  #this build's entry alone, which the Linux driver carries out of the container into the ledger
	if compiled and host != 'linux': ledger_append(repository / f'library-{ledger}.txt', entry)
	say(f"== {library} {record['bytes']} bytes, sha256 {record['sha256']}, in src-tauri/binaries with libmpv.json beside it" + (f', and in library-{ledger}.txt' if compiled and host != 'linux' else ''))

def link(path, target):  #the unversioned name beside the library, which the linker looks for; made fresh each run
	if path.is_symlink() or path.exists(): path.unlink()
	path.symlink_to(target)
def ledger_entry(record):  #one entry of a library file: the library's line, the recipe's, the tools', then one per source with its hash or commit whole and where it came from, the names padded so the hashes line up
	width = max(len(source['name']) for source in record['sources'])
	lines = [
		f"{record['built'].replace('T', ' ').replace('Z', ' UTC')}  {host} {architecture}  {record['file']}  {record['bytes']} bytes  sha256 {record['sha256']}",
		f"recipe  desktop/libmpv/build.py  {record['recipe']}",
		f"tools   {record['tools']}",
	]
	for source in record['sources']: lines.append(f"{source['name'].ljust(width)}  {(source.get('sha256') or 'commit ' + source['commit']).ljust(64)}  {source['url']}")
	return '\n'.join(lines) + '\n'
def ledger_append(path, entry):  #onto the end of the file, after a blank line when it holds entries already, and never anything else
	with open(path, 'a') as file: file.write(('\n' if path.exists() and path.stat().st_size else '') + entry)

try: main()
except subprocess.CalledProcessError as error: sys.exit(f'build.py: {" ".join(error.cmd[:3])} failed with status {error.returncode}')
