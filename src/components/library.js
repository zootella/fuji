
//  _____       _ _ 
// |  ___|   _ (_|_)
// | |_ | | | || | |
// |  _|| |_| || | |
// |_|   \__,_|/ |_|
//           |__/   

//keep, this is the new unifed library to keep components short and tell what's a pure function in here

import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow, currentMonitor} from '@tauri-apps/api/window'
import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module




//promise helpers
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
	let contents = await ioReadDir(folder)
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




export async function readAndRenderImage(img, path) {
	let details = await readImage(path)
	return await renderImage(img, details)
}
export async function readImage(path) {//read the file at path and get a data url string ready to render
	let details = {}
	details.t1 = performance.now()//start time
	details.path = path

	//read file and convert to data url
	let bytes = new Uint8Array(await ioRead(path))
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
	details.span = xy(img.naturalWidth, img.naturalHeight)//and now we can get its pixel dimensions
	details.note = `${Math.round(details.t2 - details.t1)}ms disk + ${Math.round(details.t3 - details.t2)}ms memory + ${Math.round(details.t4 - details.t3)}ms render`
	return details
}







export const hardVerticals = [
	480,  // Legacy 640×480 VGA; still seen in embedded systems and some virtual modes
	600,  // SVGA (800×600); common in late '90s multimedia PCs
	720,  // HD (1280×720); entry-level TVs, older budget laptops
	768,  // WXGA (1366×768); dominant in budget laptops for over a decade [mainstream]
	864,  // FWXGA variant (1536×864); occasional in mid-range laptops
	900,  // 1440×900 / 1600×900; mid-range panels, especially Dell and HP [mainstream]
	1024, // SXGA (1280×1024); popular 4:3 office monitors in the 2000s
	1050, // WSXGA+ (1680×1050); mid-to-high-end 16:10 panels in the late 2000s
	1080, // Full HD (1920×1080); the most common resolution today [mainstream]
	1152, // QWXGA (2048×1152); uncommon wide format, some niche monitors
	1200, // UXGA (1600×1200) / WUXGA (1920×1200); pro and workstation panels [mainstream]
	1440, // QHD (2560×1440); high-end monitors, gaming setups, premium laptops [mainstream]
	1536, // QXGA (2048×1536); uncommon high-res 4:3, some tablets
	1600, // WQXGA (2560×1600); premium 16:10 laptops and monitors [mainstream]
	1664, // MacBook Air M2 native (2560×1664)
	1964, // MacBook Pro 14" native (3024×1964)
	2160, // 4K UHD (3840×2160); widespread in premium laptops and monitors [mainstream]
	2234, // MacBook Pro 16" native (3456×2234)
	2520, // iMac 24" 4.5K Retina (4480×2520)
	2880, // 5K (5120×2880); iMacs and ultra-premium displays
	3200, // 6K (6016×3384); Apple Pro Display XDR and similar
	4320, // 8K UHD (7680×4320); bleeding-edge professional monitors
]




export function xy(a, o, b) {//use like xy(x, y) to set or xy(a, '+', b) to compute
	if      (o == '+') { return {x: a.x + b.x, y: a.y + b.y} }//use with two {x, y} objects
	else if (o == '-') { return {x: a.x - b.x, y: a.y - b.y} }
	else if (o == '*') { return {x: a.x * b,   y: a.y * b  } }//use with ane xy object and a number, like 2
	else if (o == '/') { return {x: a.x / b,   y: a.y / b  } }
	else if (o == '==') { return   a.x == b.x && a.y == b.y  }//equals
	else if (o == '!=') { return !(a.x == b.x && a.y == b.y) }
	else { return {x: a, y: o} }
}





