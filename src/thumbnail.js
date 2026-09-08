import {invoke} from '@tauri-apps/api/core'

//the operating system's thumbnailer, on the two platforms fuji calls it on: ImageIO on the mac and the windows imaging component on windows; thumbnail.rs is the long version. Nothing calls this yet: CanvasFlow makes thumbnails in the page on every platform, and canvas.md says what would earn this the call

export function thumbnailRender(path, maximum, gamut) { return invoke('thumbnail_render', {path, maximum, gamut}) }//one ArrayBuffer: eight bytes of header, then rgba. maximum is the longer side in backing pixels, never enlarged; gamut is 'display-p3' or 'srgb', and windows answers srgb regardless. Rejects with a message on linux, and for a file the operating system cannot decode

export function thumbnailUnpack(buffer) {//the width, height and pixels out of that buffer, shaped for new ImageData(pixels, width, height)
	let header = new DataView(buffer, 0, 8)
	let width = header.getUint32(0, true), height = header.getUint32(4, true)//little endian, as thumbnail.rs writes them
	return {width, height, pixels: new Uint8ClampedArray(buffer, 8, width * height * 4)}
}
