import {invoke} from '@tauri-apps/api/core'

//the operating system's thumbnailer, ImageIO on the mac and the windows imaging component on windows; thumbnail.rs is the long version, SquareFlow is the caller and decides which files come here, and the thumbnail pipeline document on fuji's site says why

export function thumbnailProbe(paths)                          { return invoke('thumbnail_probe',  {paths})                        }//one call for a card's paths, answered with {format, width, height, problem} for each, from the first bytes and the header alone; problem says why a file will not be shown, and is blank otherwise. Rejects only when the bridge itself failed
export function thumbnailRender(path, format, maximum, gamut) { return invoke('thumbnail_render', {path, format, maximum, gamut}) }//one ArrayBuffer: twelve bytes of header, then rgba. format is what the probe said the bytes are, and the render refuses a file that disagrees; maximum is the longer side in backing pixels, never enlarged; gamut is 'display-p3' or 'srgb'. Rejects on linux, for a file the operating system will not decode, and for a header claiming more than half the machine's memory

export function thumbnailUnpack(buffer) {//the width, height, gamut and pixels out of that buffer, shaped for new ImageData(pixels, width, height, {colorSpace: gamut})
	let header = new DataView(buffer, 0, 12)
	let width = header.getUint32(0, true), height = header.getUint32(4, true)//little endian, as thumbnail.rs writes them
	let gamut = header.getUint32(8, true) ? 'display-p3' : 'srgb'//what the pixels are, which on windows is srgb whatever was asked
	return {width, height, gamut, pixels: new Uint8ClampedArray(buffer, 12, width * height * 4)}
}
