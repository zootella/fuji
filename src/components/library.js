
//  _____       _ _ 
// |  ___|   _ (_|_)
// | |_ | | | || | |
// |  _|| |_| || | |
// |_|   \__,_|/ |_|
//           |__/   

//keep, this is the new unifed library to keep components short and tell what's a pure function in here

import parse from 'path-browserify'//naming this parse instead of path so we can have variables named path
import {ioRead, ioReadDir} from '../io.js'//our rust module



//promise helpers
export const raf = () => new Promise(r => requestAnimationFrame(r))
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
export async function lookPath(path) {//given a path, return text all about it

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

	console.log(images)
	return `
${path} <- path
${folder} <- folder
found ${images.length} images
`




}










