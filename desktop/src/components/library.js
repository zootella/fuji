
//  _____       _ _ 
// |  ___|   _ (_|_)
// | |_ | | | || | |
// |  _|| |_| || | |
// |_|   \__,_|/ |_|
//           |__/   

//keep, this is the new unifed library to keep components short and tell what's a pure function in here

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor, primaryMonitor, cursorPosition} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {diskRead, diskReadDir} from '../disk.js'//our rust modules
import {panelResolution} from '../panel.js'
import {brandName} from '../brand.js'
import {log} from '../log.js'//log.js imports forwardize from here in return, which is fine: neither file calls the other while the modules are loading, only later from inside a function

//promises

export const raf = () => new Promise(resolve => requestAnimationFrame(resolve))//before next paint, synchronized with display refresh (~16ms)
export function blobToDataUrl(blob) {//promisifed wrapper of FileReader's .readAsDataURL method
	let reader = new FileReader()
	let p = new Promise((resolve, reject) => {
		reader.onload  = () => resolve(reader.result)
		reader.onerror = () => reject(reader.error)
	})
	reader.readAsDataURL(blob)
	return p
}

//arrows

export function xy(a, o, b) {//use like xy(x, y) to set or xy(a, '+', b) to compute
	if      (o == '+') { return {x: a.x + b.x, y: a.y + b.y} }//use with two {x, y} objects
	else if (o == '-') { return {x: a.x - b.x, y: a.y - b.y} }
	else if (o == '*') { return {x: a.x * b,   y: a.y * b  } }//use with ane xy object and a number, like 2
	else if (o == '/') { return {x: a.x / b,   y: a.y / b  } }
	else if (o == '==') { return   a.x == b.x && a.y == b.y  }//equals
	else if (o == '!=') { return !(a.x == b.x && a.y == b.y) }
	else { return {x: a, y: o} }
}
export function xySnap(a, backingPerCss) { return xy(Math.round(a.x * backingPerCss) / backingPerCss, Math.round(a.y * backingPerCss) / backingPerCss) }//an arrow in css pixels snapped to the backing grid, at backingPerCss backing pixels to the css pixel: whole numbers at 1, halves at 2, and always a whole number of backing pixels, for the moment real numbers become pixels

//paths

//forwardize all new paths that come into the system, then backize to show on the page
export function forwardize(path) {
	//rotate backslashes forward given what looks like a windows drive letter path; the forwardized path will still work with path-browserify and our rust io module code
	return /^[a-zA-Z]:[\\/]/.test(path) ? path.replace(/\\/g, '/') : path
}
export function backize(path) {
	//but will look weird on windows, so use this in template code before showing to a Windows user
	return /^[a-zA-Z]:[\\/]/.test(path) ? path.replace(/\//g, '\\') : path
}

//every kind of picture fuji can show: the folder listing filters by it, the flow routes by it, the store types its blobs from it, and associate.js hands it to the operating system on windows as what fuji is offering to open
//**this list exists twice, and the other copy is CFBundleDocumentTypes in src-tauri/Info.plist.** Add or remove an extension here and change that file to match, in the same commit. macOS reads the plist out of the bundle before any of fuji's code has run, so nothing here can reach it; generating one from the other was considered and declined, because the cost lands on the build and this list changes about never
//the name is what windows prints in explorer's type column, and it carries the extension rather than the format on purpose. All four jpeg spellings are honestly one format, and Finder's kind column calls them all JPEG image, but that column is also the only way to sort a folder by extension, and a shared name scatters the three .jpe files through the sort instead of grouping them. WebP keeps its own capitalization, being the one extension whose real name is not simply its letters in capitals
export const imageTypes = {
	'.bmp':  {mime: 'image/bmp',     name: 'BMP Image'},//1986, Microsoft: Simple uncompressed raster format for Windows graphics, easy to decode
	'.gif':  {mime: 'image/gif',     name: 'GIF Image'},//1987, CompuServe: 256-color palette with animation support, early web staple, now 😺🍔

	'.jpg':  {mime: 'image/jpeg',    name: 'JPG Image'},//1992, Joint Photographic Experts Group: Lossy compression for photographs
	'.jpeg': {mime: 'image/jpeg',    name: 'JPEG Image'},
	'.jpe':  {mime: 'image/jpeg',    name: 'JPE Image'},
	'.jfif': {mime: 'image/jpeg',    name: 'JFIF Image'},

	'.png':  {mime: 'image/png',     name: 'PNG Image'},//1996, PNG Development Group/W3C: lossless compression and full alpha transparency
	'.svg':  {mime: 'image/svg+xml', name: 'SVG Image'},//2001, W3C: Scalable vector graphics for resolution-independent diagrams and icons
	'.avif': {mime: 'image/avif',    name: 'AVIF Image'},//2019, Alliance for Open Media: from AV1 codec, supports HDR and wide color gamut
	'.webp': {mime: 'image/webp',    name: 'WebP Image'},//2010, Google: recent format for smaller file size
}
export async function listFolder(folder) {//the image files in one folder, in whatever order the disk handed them over; a sort is what puts them in one
	let contents = await diskReadDir(folder)
	let files = contents.filter(f => f.is_file && !f.is_dir && !f.is_symlink)//only include files
	files = files.map(f => ({...f,
		path: parse.join(folder, f.name),
		extension: parse.extname(f.name).toLowerCase(),
	}))
	return files
		.filter(f => !f.name.startsWith('.'))//skip the .name.ext files macos makes for every file on a removable drive
		.filter(f => imageTypes[f.extension])//only include known extensions
		.map(f => ({
		...f,
		mime: imageTypes[f.extension].mime,//include the mime type that goes with that extension; the filter above has already dropped anything the table does not name
	}))
}
export async function listSiblings(path) {//the same listing, ordered and with the given path found in it; the retired experiments are the only callers left, because the model lists and sorts for itself
	let images = await listFolder(parse.dirname(path))
	let list = images.map(f => f.path).sort()
	let index = list.indexOf(path)
	if (index == -1) index = 0//ttd august
	return {index, list}
}

//images

export const errorImageData = `data:image/svg+xml;base64,${btoa(`
	<svg width="300" height="300" xmlns="http://www.w3.org/2000/svg">
		<rect width="300" height="300" fill="none" stroke="#444" stroke-width="1" stroke-dasharray="2,1"/>
		<line x1="140" y1="140" x2="160" y2="160" stroke="#444" stroke-width="1"/>
		<line x1="160" y1="140" x2="140" y2="160" stroke="#444" stroke-width="1"/>
	</svg>
`)}`//a dashed box with a cross: what every view shows in place of a picture it could not read

export async function readAndRenderImage(img, path) {
	let details = await readImage(path)
	return await renderImage(img, details)
}
export async function readImage(path) {//read the file at path and get a data url string ready to render
	let details = {}
	details.t1 = performance.now()//start time
	details.path = path

	//read file and convert to data url
	let bytes = new Uint8Array(await diskRead(path))
	details.t2 = performance.now()//time spent in io from disk
	let blob = new Blob([bytes.buffer], {type: 'image/png'})
	let data = await blobToDataUrl(blob)//alternatively, URL.createObjectURL saves memory, but creates a resource that could leak
	details.t3 = performance.now()//time converting formats in memory
	details.size = bytes.length//byte size of file
	details.data = data//keep a reference to the data url even though we don't use it yet
	return details
}
export async function renderImage(img, details) {//render the data url string details.data into the given hidden img tag

	//load the data url into the given img tag and decode it
	img.src = details.data//setting this should cause an earlier call awaiting decode to throw, and this new call to work fine
	await img.decode()//throws on problem with the image data

	//success if there wasn't an exception from that
	details.t4 = performance.now()//time rendering image to bitmap
	details.natural = xy(img.naturalWidth, img.naturalHeight)//and now we can get its pixel dimensions
	details.note = `${Math.round(details.t2 - details.t1)}ms disk + ${Math.round(details.t3 - details.t2)}ms memory + ${Math.round(details.t4 - details.t3)}ms render`
	return details
}

//resolution

export function platform() {//mac, windows or linux
	let p = navigator.platform//MacIntel on every mac, apple silicon included; Win32 on every windows
	if (p.startsWith('Mac')) return 'mac'
	if (p.startsWith('Win')) return 'windows'
	return 'linux'
}

export function windowTitle(showing, path, folder) {//what the title bar says: the picture a table is showing, the folder the sheet is showing, Settings on the settings panel, and fuji's own name when there is none of those
	/*
	The name alone, never a path: a title bar is narrow and a taskbar button narrower, and the leading half of a path is the half nobody needs.

	The suffix is where the platforms genuinely differ, so this is one of the few places fuji does something different on each. Windows spells a document window 'name - App', which Notepad and Paint still do, and a taskbar button carries that string. macOS spells it just the name, because the application's own name is already in the menu bar an inch away and repeating it there reads as a mistake — Preview and TextEdit both show the bare filename. GNOME agrees with macOS and its file manager shows a bare folder name. KDE would rather have 'name — App' with an em dash, which is a third form and is not followed here.
	*/
	let name = path && parse.basename(path)//the picture, on a table
	if (showing == 'Sheet')    name = folder && parse.basename(folder)
	if (showing == 'Settings') name = 'Settings'//about fuji rather than about a folder or a picture
	if (!name) return brandName//nothing open yet, or a path with no last segment like a bare root; either way the application's own name and nothing else
	return platform() == 'windows' ? `${name} - ${brandName}` : name
}

const sheetPreset = {width: 0.6, height: 0.85}//how big the sheet's window opens when settings have no size that fits, as portions of the work area's width and height. The window is the frame the user sees, which on windows leaves out the invisible resize borders; window.rs says how
const sheetWidest = 16 / 9//but the preset is never wider than this for its height, so a super wide monitor gets a sheet rather than a banner. The portions above make about 1.2 to 1 on a 16:10 laptop, 1.3 on a 16:9 screen and 1.7 on a 21:9 one, so only the 32:9 screens meet this, which would otherwise open a sheet 2.6 to 1 and over three thousand css pixels wide. A size the user gave a sheet is theirs, and is never capped

export async function screenAreas() {//the screen this window is on, whole and less the menu bar, dock or taskbar, each as {x, y, width, height} in css pixels; false when there is no monitor to ask
	let m = await currentMonitor()
	if (!m) return false
	let backingPerCss = m.scaleFactor//tauri answers in backing pixels, which it calls physical, and every rectangle the page handles is css
	let rect = (at, size) => ({x: at.x / backingPerCss, y: at.y / backingPerCss, width: size.width / backingPerCss, height: size.height / backingPerCss})
	return {screen: rect(m.position, m.size), work: rect(m.workArea.position, m.workArea.size)}
}
export function rectSheet(work, saved) {//where the contact sheet's window goes, in css pixels: the size the user last gave a sheet if it fits the work area, or sheetPreset of the work area if not, and a random place inside it either way, so two sheets opened at once almost never land on each other. saved is the size from settings, a width of 0 meaning none
	let height = Math.round(work.height * sheetPreset.height)
	let width = Math.min(Math.round(work.width * sheetPreset.width), Math.round(height * sheetWidest))
	if (saved.width > 0 && saved.width <= Math.round(work.width) && saved.height <= Math.round(work.height)) { width = saved.width; height = saved.height }//a size from a bigger desktop, or from before the dock moved, gives way to the preset. Compared in whole css pixels, because settings keep whole css pixels and a work area at a fractional scale, 150 percent on windows, is fractional in css pixels, so a sheet stretched across the whole work area, saved at 1707, would otherwise fail to fit one 1706.67 wide
	return {x: Math.round(work.x + Math.random() * (work.width - width)), y: Math.round(work.y + Math.random() * (work.height - height)), width, height}//the room left over along each axis, rolled evenly
}
export async function pointerPosition() {//where the pointer is, in the same css pixels as screenAreas, or false when the platform will not say
	try {
		let [p, m] = await Promise.all([cursorPosition(), platform() == 'mac' ? primaryMonitor() : currentMonitor()])//on the mac tao multiplies the pointer by the primary display's scale factor, in util::cursor_position, where every monitor and work area uses that monitor's own, so it comes back to css pixels, the unit screenAreas answers in there, only divided by the primary's. Windows reports the pointer in backing pixels, and linux on x11 has one scale factor for every monitor, so the window's own monitor is right for both
		if (!m) return false
		return {x: p.x / m.scaleFactor, y: p.y / m.scaleFactor}
	} catch (error) {
		return false//the platform would not say, and the preview centers instead. Linux on wayland does not land here: tao answers (0, 0) there rather than failing, which reads as the work area's corner, where the larger side is all of it, so the preview centers there too
	}
}

const previewZoomMost = 2//the most a preview enlarges a small picture, in css pixels per raster pixel, the unit the table's number keys use, so a capped preview hands over a card the 2 key would give. A constant rather than a setting

/*
Where the preview goes, which is a picture sized to the work area and placed out from under the pointer.

The size is the picture's own shape, as large as the work area allows, so a portrait on a wide screen meets the menu bar and the dock, and a landscape meets the sides. A small picture stops at previewZoomMost rather than filling the screen with enlarged pixels.

The place is chosen one axis at a time, by one rule: the pointer splits the work area in two, and the picture centers in the larger side. A pointer over the menu bar, the dock or anywhere else past the work area's edge counts as at that edge, where the larger side is the whole work area and the picture simply centers. A picture too big for the larger side slides toward the far edge until it fits, and no further. Since the pointer is uncovered if the picture clears it on either axis, and the larger side is the only one worth trying on each, this leaves the pointer uncovered whenever any placement inside the work area could.
*/
export function rectPreview(natural, work, pointer) {//the preview's frame in css pixels: natural is the picture's raster size, work the work area, and pointer where the pointer is, or false to center
	let scale = Math.min(work.width / natural.x, work.height / natural.y, previewZoomMost)//css pixels per image pixel: the tighter side meets the work area's edge, unless the picture would pass the most a preview enlarges first
	let width = Math.round(natural.x * scale), height = Math.round(natural.y * scale)
	return {x: _away(work.x, work.width, width, pointer ? pointer.x : false), y: _away(work.y, work.height, height, pointer ? pointer.y : false), width, height}
}
function _away(start, length, size, pointer) {//one axis of rectPreview: where along it a size goes, inside start to start plus length, and away from pointer
	let end = start + length
	let at = start + (length - size) / 2//centered, which is where a picture goes when the platform gives no pointer
	if (pointer !== false) {//strict, because zero is a position
		let p = Math.min(Math.max(pointer, start), end)//past either edge counts as at it
		let [from, to] = end - p >= p - start ? [p, end] : [start, p]//the larger side, and on an exact tie the right or lower one
		at = (from + to) / 2 - size / 2//centered in that side
	}
	return Math.round(Math.max(start, Math.min(at, end - size)))//kept inside the work area, sliding toward the far edge when the side is too small
}

export async function revealWindow() {//show the window, which rust built hidden and the shell has placed
	await getCurrentWindow().show()
}

export async function measureScreen() {//get the screen resolution as {x, y} in css, backing and panel pixels
	const w = getCurrentWindow()
	const m = await currentMonitor()
	let q = {
		windowDevicePixelRatio: window.devicePixelRatio,
		tauriWindowScaleFactor: await w.scaleFactor(),
		tauriMonitorScaleFactor: m.scaleFactor,//these tend to all be the same, and each is backing per css, never anything to do with the panel!

		cssScreen: xy(screen.width, screen.height),
		backingScreen: xy(m.size.width, m.size.height),
		panelScreen: await panelResolution(),//custom Rust code we wrote to system APIs to get the panel's own pixel count
	}
	log(`⭕ library: measured the screen ${JSON.stringify(q)}`)
	return q
}
let _screen//{when, panelScreen} ttd august, save here if not 0,0; report from here not api call if within 50ms






















//group digits like "12,345"
export function sayGroupDigits(s, thousandsSeparator = ',') {//pass comma, period, or leave out to get international ready thin space
	if (typeof s != 'string') s += ''

	let minus = ''
	if (s.startsWith('-')) { minus = '-'; s = s.slice(1) }//deal with negative numbers
	if (s.length > 4) {//let a group of four through
		s = s.split('').reverse().join('')//reversed
		s = s.match(/.{1,3}/g).join(thousandsSeparator)//grouped reverse
		s = s.split('').reverse().join('')//forward again
	}
	return minus+s
}

// Describe big sizes and counts in four digits or less
export function saySize4(n)   { return _number4(n, 1024, [' bytes', ' KB', ' MB', ' GB', ' TB', ' PB', ' EB', ' ZB', ' YB']) }
export function sayNumber4(n) { return _number4(n, 1000, ['',       ' K',  ' M',  ' B',  ' T',  ' P',  ' E',  ' Z',  ' Y'])  }
function _number4(n, power, units) {
	var u = 0 // Start on the first unit
	var d = 1 // Which has a value of 1 each
	while (u < units.length) { // Loop to larger units until we can say n in four digits or less

		var w = Math.floor(n / d) // Find out how many of the current unit we have
		if (w <= 9999) return w + units[u] // Four digits or less, use this unit

		u++ // Move to the next larger unit
		d *= power
	}
	return n+'' // We ran out of units
}
