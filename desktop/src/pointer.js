import {invoke} from '@tauri-apps/api/core'

//the pointers Windows shows while a drag holds something, which css has no keyword for, read out of ole32.dll by pointer.rs and made into a value for the cursor property

export function pointerRead(name, backingPerCss) { return invoke('pointer_read', {name, backingPerCss}) }//one ArrayBuffer: sixteen bytes of header, then rgba. name is move, copy, link or none; backingPerCss is devicePixelRatio. Rejects off windows

export async function pointerCursor(name) {//a value for the cursor property showing that pointer at the display's scale, falling back to the arrow
	let backingPerCss = window.devicePixelRatio
	let buffer = await pointerRead(name, backingPerCss)
	let header = new DataView(buffer, 0, 16)
	let width = header.getUint32(0, true), height = header.getUint32(4, true), x = header.getUint32(8, true), y = header.getUint32(12, true)//little endian, as pointer.rs writes them
	let canvas = document.createElement('canvas')
	canvas.width = width; canvas.height = height
	canvas.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(buffer, 16, width * height * 4), width, height), 0, 0)
	return `image-set(url("${canvas.toDataURL('image/png')}") ${backingPerCss}x) ${x / backingPerCss} ${y / backingPerCss}, default`//the picture marked with its own scale, so the engine shows its backing pixels one to one rather than taking them for css pixels and stretching them; the hotspot is in css pixels
}
