import {invoke} from '@tauri-apps/api/core'

//the operating system's thumbnailer, ImageIO on the mac and the windows imaging component on windows; thumbnail.rs is the long version, SquareFlow is the caller and decides which files come here, and the thumbnail pipeline document on fuji's site says why

export function thumbnailRender({path, fit, beam, screenWidth, screenHeight, backingPerCss, gamut}) { return invoke('thumbnail_render', {path, fit, beam, screenWidth, screenHeight, backingPerCss, gamut}) }//one ArrayBuffer: twenty bytes of header, then rgba. fit, beam and the screen's size in css pixels are what fitSize in fit.js takes, and rust runs the same fit to choose how large to render; backingPerCss is devicePixelRatio; gamut is 'display-p3' or 'srgb'. Rejects on linux, for a file the operating system will not decode, for a picture whose size the library will not say, and for one claiming more than half the machine's memory

export function thumbnailUnpack(buffer) {//the thumbnail's width, height, gamut and pixels out of that buffer, shaped for new ImageData(pixels, width, height, {colorSpace: gamut}), and the picture's own size as it shows, in image pixels, which the page sizes the tile from
	let header = new DataView(buffer, 0, 20)
	let width = header.getUint32(0, true), height = header.getUint32(4, true)//little endian, as thumbnail.rs writes them
	let gamut = header.getUint32(8, true) ? 'display-p3' : 'srgb'//what the pixels are, which on windows is srgb whatever was asked
	let naturalWidth = header.getUint32(12, true), naturalHeight = header.getUint32(16, true)//after its orientation, so a portrait from a phone is tall
	return {width, height, gamut, naturalWidth, naturalHeight, pixels: new Uint8ClampedArray(buffer, 20, width * height * 4)}
}
