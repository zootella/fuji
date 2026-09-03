
//  _____       _ _ 
// |  ___|   _ (_|_)
// | |_ | | | || | |
// |  _|| |_| || | |
// |_|   \__,_|/ |_|
//           |__/   

//keep, this is the new unifed library to keep components short and tell what's a pure function in here

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor, LogicalSize, PhysicalSize, PhysicalPosition} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {diskRead, diskReadDir} from '../disk.js'//our rust modules
import {panelResolution} from '../panel.js'

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

export const imageTypes = {
	'.bmp': 'image/bmp',//1986, Microsoft: Simple uncompressed raster format for Windows graphics, easy to decode
	'.gif': 'image/gif',//1987, CompuServe: 256-color palette with animation support, early web staple, now 😺🍔

	'.jpg':  'image/jpeg',//1992, Joint Photographic Experts Group: Lossy compression for photographs
	'.jpeg': 'image/jpeg',
	'.jpe':  'image/jpeg',
	'.jfif': 'image/jpeg',

	'.png':  'image/png',//1996, PNG Development Group/W3C: lossless compression and full alpha transparency
	'.svg':  'image/svg+xml',//2001, W3C: Scalable vector graphics for resolution-independent diagrams and icons
	'.avif': 'image/avif',//2019, Alliance for Open Media: from AV1 codec, supports HDR and wide color gamut
	'.webp': 'image/webp',//2010, Google: recent format for smaller file size
}
export async function listSiblings(path) {//given a path, return text all about it
	let folder = parse.dirname(path)
	let contents = await diskReadDir(folder)
	let files = contents.filter(f => f.is_file && !f.is_dir && !f.is_symlink)//only include files
	files = files.map(f => ({...f,
		path: parse.join(folder, f.name),
		extension: parse.extname(f.name).toLowerCase(),
	}))
	let images = files
		.filter(f => !f.name.startsWith('.'))//skip the .name.ext files macos makes for every file on a removable drive
		.filter(f => imageTypes[f.extension])//only include known extensions
		.map(f => ({
		...f,
		mime: imageTypes[f.extension],//include the mime type that goes with that extension
	}))

	let list = images.map(f => f.path).sort()
	let index = list.indexOf(path)
	if (index == -1) index = 0//ttd august
	return {index, list}
}

//images

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

const startingWindowSize = {widthFraction: 0.6, heightFraction: 0.8}//how much of the usable desktop the window takes when it first opens

/*
The window is created hidden — tauri.conf.json sets visible false — and this function sizes it to fit the desktop before revealing it, so it appears once already correct instead of flashing at one size and jumping to another.

Two things it deliberately does not do. It never chooses a position: where a window opens is the operating system's job, and leaving it there is what makes a second copy land beside the first rather than exactly on top of it, where the user could never find it — the one exception being a window the user has asked fuji to remember, handed in as a rect, which it puts back where they left it. And it never lets a sizing failure stop the reveal, which is what the finally is for — the window starts hidden, so an error on the way to show() would leave a process running with nothing on screen at all.

That difference is also why there is one try here rather than two. Failing to measure the desktop has a fallback: the window keeps the size tauri.conf.json gave it when it was created, 800 by 600, which is also Tauri's own default. Failing to show has no fallback, so show() sits outside the catch — if it rejects, the app is broken in a way no handling here improves.

The caller measures the viewport after this resolves, and must, because a hidden window is given no animation frames: nothing can await one until show() has happened.
*/

export async function revealWindow(rect) {//size the hidden window and show it; pass the window fuji recorded to return to it, or false to size to the desktop; call once, after the app has mounted and there's something to see
	let w = getCurrentWindow()
	try {
		if (await w.isVisible()) return//size once at startup only; a hot reload in development mounts the app again against a window that is already up
		if (rect) {//fuji is remembering the window and has one recorded, so put it back rather than measuring anything
			await w.setSize(new PhysicalSize(rect.width, rect.height))
			await w.setPosition(new PhysicalPosition(rect.x, rect.y))//physical at both ends, exactly as the window reported it, so nothing rounds on the way back
			return
		}
		let m = await currentMonitor()
		if (!m) return//tauri couldn't say which monitor we're on, so there's nothing to measure and the fallback size stands
		let area = xy(m.workArea.size.width, m.workArea.size.height)//the monitor minus the chrome the os keeps for itself: menu bars, docks, the windows taskbar; in backing pixels, like everything tauri measures
		let logical = xy(area, '/', m.scaleFactor)//the resize api speaks logical pixels
		await w.setSize(new LogicalSize(
			Math.round(logical.x * startingWindowSize.widthFraction),
			Math.round(logical.y * startingWindowSize.heightFraction)))
	} catch (e) {
		console.error('sizing the window:', e)//whatever went wrong measuring or resizing, the fallback size stands
	} finally {
		await w.show()//reveal whatever happened above, including the early returns
	}
}

export async function screenToViewport() {//arrow from the screen corner above the os menu to the viewport corner below the titlebar
	/*
	The Pixel Unit Challenge: there are multiple pixel units at play
	1. CSS pixels - What web APIs report
	2. Logical/Points pixels - macOS "looks like" resolution
	3. Backing store pixels - The large bitmap macOS renders to
	4. Physical/Hardware pixels - Actual screen pixels
	you're seeing that 1 and 2 are the same, and 3 is a macOS only thing

	also, tauri APIs are broken:
	- getCurrentWindow().outerSize works, but is in backing store pixels
	- and is the same as what .innerSize says
	- window.innerWidth and .innerHeight are nonsensical

	so the crazy workaround here assumes a border width that's the same all around
	and a title bar height that's only on the top
	and then we can do the math from there!

	on mac, border is 0, so all the extra height is title bar at the top
	on windows, crazily, position points to outside a 7 pixel all the way around shadow,
	except with the top shaved off--so the math still works
	*/
	let w = getCurrentWindow()
	let p = await w.outerPosition()
	let s = await w.outerSize()
	let m = await currentMonitor()

	//measurements from tauri are in macOS backing store pixels
	let backingPosition = xy(p.x, p.y)//screen corner to window outer corner
	let backingWindowOuter = xy(s.width, s.height)//outer window dimensions, including titlebar and borders
	let backingScreen = xy(m.size.width, m.size.height)//screen dimensions

	//measurements from HTML are in CSS pixels
	let cssScreen = xy(screen.width, screen.height)//screen dimensions
	let cssWindowInner = xy(window.innerWidth, window.innerHeight)//inner window dimensions, the renderer viewport which is the frame div

	//scale everything CSS pixels
	let scale = cssScreen.y / backingScreen.y
	let cssPosition = xy(backingPosition, '*', scale)
	let cssWindowOuter = xy(backingWindowOuter, '*', scale)

	//here, we assume there's a border all the way around, and a title bar only at the top
	let border = (cssWindowOuter.x - cssWindowInner.x) / 2
	let title = cssWindowOuter.y - border - cssWindowInner.y - border
	let cssScreenToViewport = xy(cssPosition.x + border, cssPosition.y + border + title)

if (false) console.log(`in backing units:
${backingScreen.x} × ${backingScreen.y} screen
${backingWindowOuter.x} × ${backingWindowOuter.y} outer window
${backingPosition.x} × ${backingPosition.y} position

all in css units from here:
${cssScreen.x} × ${cssScreen.y} screen
${cssWindowOuter.x} × ${cssWindowOuter.y} outer window (calculated, scale of ${scale})
${cssWindowInner.x} × ${cssWindowInner.y} inner window
${cssPosition.x} × ${cssPosition.y} position (calculated)

from that we assume ${border} border all the way around, and ${title} title bar on the top, and
${cssScreenToViewport.x} × ${cssScreenToViewport.y} screen to viewport
`)
	return cssScreenToViewport
}

export async function measureScreen() {//get the screen resolution as {x, y} in all the different real and fake pixel units
	const w = getCurrentWindow()
	const m = await currentMonitor()
	let q = {
		windowDevicePixelRatio: window.devicePixelRatio,
		tauriWindowScaleFactor: await w.scaleFactor(),
		tauriMonitorScaleFactor: m.scaleFactor,//these tend to all be the same, but go between CSS and backing, never to physical!

		cssScreen: xy(screen.width, screen.height),
		backingScreen: xy(m.size.width, m.size.height),
		physicalScreen: await panelResolution(),//custom Rust code we wrote to system APIs to get the real physical pixel counts
	}
	console.log(q)
	return q
}
let _screen//{when, physicalScreen} ttd august, save here if not 0,0; report from here not api call if within 50ms












export function sizeThumbnail(method, natural, h) {
	let aspect = natural.x / natural.y
	if (method == 'Flickr.') {//size to height, don't let get wider than 2x height, don't blow up icons
		let w = h * 2
		if (natural.x <= w && natural.y <= h) return natural//small enough to pass through
		let thumbnail = xy(h * aspect, h)//fit to height
		if (thumbnail.x > w) return xy(w, w / aspect)//too wide!--fit to width
		return thumbnail
	}//we might have additional alternative sizing strategies in the future
}










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






































