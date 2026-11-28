//how a file becomes a tile on the sheet: every thumbnail sized by a fit, its pixels from the operating system or from the page, decided per file; TestFlow.vue places and draws what this decides, and nothing here knows which framework that is

import {cacheNeed, cacheRelease} from './cache.js'
import {governorRun} from './governor.js'
import {settings, settingsThumbnailBeam} from './settings.js'
import {fitSize} from './fit.js'
import {thumbnailRender, thumbnailUnpack} from '../brass/thumbnail.js'
import {logTrouble, logThumbnail, logBucket} from '../brass/log.js'//the log, off unless fuji.toml says otherwise; every thumbnail and every bucket is a row in it
import {xy, platform, sayCaptionDay, sayCaptionSize, sayDimensions, middleDot} from './library.js'
import {fileTypesSheet} from './fileTypes.js'

/*
The one flow, and the whole of how a file becomes a tile. A bucket hands this its files, the listing's entries, which carry the name, the size and the modified time the caption reads. fileTypesSheet in fileTypes.js says from the extension what kind of tile each gets on this platform: one whose contactSheet is img, a GIF or an SVG, is an img, so a GIF animates and an SVG is painted by the engine inside the sandbox an img is; everything else is a canvas fuji sized, which is memory the sheet can count, and a file nothing on this platform can draw is left out, counted in the bucket's caption. A canvas gets its pixels one of two ways. A file this platform's imaging library reads goes down to Rust, and the operating system's thumbnail comes back small and goes on with one putImageData; the store never hears about the file. Anything else, and everything on linux, the store reads and decodes and the page halves down into the canvas, at a cost to the main thread.

Every tile is sized the same way whatever its route: from the picture's own size, by the fit thumbnail.fit names, which the settings panel sets, through flowFit and fit.js. A page tile has that size once its decode resolves, an img tile once the engine has loaded it, and a native tile from Rust, which sends it back beside the pixels after running the same fit to choose how large to render. The flow itself is wrap, which places tiles left to right like words, and its name names no fit, since the fit is the user's to choose.

Nothing is read ahead of its thumbnail. A tile takes up room when its pixels arrive, so the rows reflow as a bucket fills, and a file that cannot be shown leaves the bucket when its render or its decode fails, counted in the caption rather than shown as anything; the bucket's numbers are the listing's, so a failure never renumbers what is around it. The check on the size a file's header claims is the render's own, in thumbnail.rs, where the decoder runs unsandboxed.

Every tile in a bucket is asked for at once, and each waits in the line of the resource it taxes, which governor.js lets through four at a time in the order asked, so a bucket fills from the top. A native tile waits under rust computation, since its decode runs on a Rust pool thread and never touches the page. A page tile waits under web computation, since it is a full decode held in the store and a draw on the main thread. An img tile has no line of its own; it waits only for the store's read, which every view's reads share under disk. A bucket that goes away leaves its tiles in line, and each returns the moment it is let in, since a call already made cannot be called back. Nothing pauses a bucket, so a sheet hidden behind the table goes on filling.

Two flows came before this one and are gone, and both of their lessons are in this file. TagFlow handed the engine full-size originals in plain img tags and let it decide everything, which is why every raster tile here is a canvas instead: the engine's thumbnail is smaller than a full decode but it is the engine's to keep or drop, and a canvas is a number of bytes fuji owns and can total. CanvasFlow painted each picture down into a canvas by hand, which is the page route below, and the halving in flowShrink is the part of it that had to be got right. The Image Fidelity page on fuji's site is the long version, with the measurements that chose each path.
*/

const flowHolder = 'TestFlow'//on every reference a flow takes, so a leak has a name

export function flowMake({files, watched, onBytes, onRefused}) {//a flow over one bucket's files, the listing's entries in the pager's order and never from two folders; the component that draws it makes one as it is set up, fills it once it has mounted, and closes it as it goes away
	let flow = {
		beam: settingsThumbnailBeam(),//the length every thumbnail is measured against, in css pixels; read once: every tile is sized to it, and a change means making them all again
		fitName: settings.thumbnail.fit,//how every tile is sized against the beam, one of fitNames in fit.js; read once like the beam
		screen: xy(screen.width, screen.height),//the screen's size in css pixels, which ScaleFit and LogFit measure against; read once, and stale on a change of monitor exactly as devicePixelRatio is
		gamut: matchMedia('(color-gamut: p3)').matches ? 'display-p3' : 'srgb',//the color space every canvas is made in, read once like the beam. A canvas is sRGB unless asked, and drawing a Display P3 photograph into an sRGB canvas clamps its most saturated colors away for good, so the thumbnail would come out duller than a table shows the same file. Asking the screen what it can show, rather than asking the engine whether it knows the name, is what keeps this from being a feature check: webkitgtk has no display-p3 value and throws when handed one, and is never handed one, because the query is always false there. Stale on a change of monitor, exactly as devicePixelRatio is
		platform: platform(),//mac, windows or linux, read once, which fileTypesSheet chooses every tile's kind for
		canvases: new Map(),//path to its canvas element, which the component hands over through flowCanvas as it makes each one
		closed: false,//the bucket is going away, so every tile still in line returns as it is let in
		held: [],//the paths this flow holds a store reference on, which are the img tiles; released as it closes, since an img needs its url as long as it shows
		bytes: 0,//what this bucket's canvases cost; the imgs are the engine's and small
		refused: 0,
		onBytes, onRefused,//what the canvases cost so far, called as each one is sized, and how many files could not be shown, called as each is refused, both for the bucket's caption
	}
	flow.tiles = watched(files.map(file => tileFor(flow, file)))//one small object per path, built once; a load or a refusal changes one tile. watched is the component's framework wrapping them so it hears each change, and every change below is made through what it handed back
	return flow
}

function tileFor(flow, file) {//the kind of tile a file gets on this platform, from its extension alone
	let kind = fileTypesSheet(file.extension, flow.platform) || 'refused'//img, native or page, by the extension the listing lowercased; refused where nothing on this platform can draw it, which flowFill counts with the files that fail
	return {path: file.path, kind, url: '', css: null, file, natural: null}//file is the listing's entry, with the name, the size and the modified time, which the caption reads; natural is the picture's own size, once a route reports it. kind turns to refused when a render or a decode fails; url is an img tile's, once the store has it, and css its size once the engine has loaded it
}

export function flowCanvas(flow, path, canvas) {//a tile's canvas as the component makes it, or nothing as it takes one away
	if (canvas) flow.canvases.set(path, canvas)
	else flow.canvases.delete(path)
}

export function flowClose(flow) {//the bucket is going away
	flow.closed = true
	for (let path of flow.held) cacheRelease(path, flowHolder)
}

export async function flowFill(flow) {//ask for every tile at once, each in the line of the resource it taxes, and log the bucket when all of them are done
	let began = performance.now()
	let ofKind = kind => flow.tiles.filter(tile => tile.kind == kind)
	let imgs = ofKind('img'), native = ofKind('native'), page = ofKind('page')
	for (let tile of ofKind('refused')) flowRefuse(flow, tile, 'nothing on this platform can draw it')//refused before anything was tried; this counts it and writes its row in the log
	await Promise.all([
		...imgs.map(tile =>                                       flowImg(flow, tile)),//not governed here: the store's read is, under disk, and governing it again would wait in that line behind itself
		...native.map(tile => governorRun('rust computation', () => flowNative1(flow, tile))),
		...page.map(tile =>   governorRun('web computation',  () => flowPage1(flow, tile))),//keeps its place through the store's read under disk, so at most four page tiles hold a whole file and its decode at once; governor.js says why this one caller holds two lines
	])
	if (!flow.closed) logBucket({index: flow.tiles.length, render: Math.round(performance.now() - began), bytes: flow.bytes, note: `${native.length} native, ${page.length} page, ${imgs.length} img, ${flow.refused} refused`})
}

export function flowRefuse(flow, tile, why) {//the file leaves the bucket, its caption counts one more, and a row says which file and why; nothing is tried twice
	tile.kind = 'refused'
	flow.refused++; flow.onRefused(flow.refused)
	logThumbnail({hit: 'refused', path: tile.path, note: why})
}

async function flowNative1(flow, tile) {//one thumbnail from the operating system, onto its canvas
	if (flow.closed) return//a bucket that went away while this tile waited in line asks Rust for nothing
	let began = performance.now()
	try {
		let buffer = await thumbnailRender({path: tile.path, fit: flow.fitName, beam: flow.beam, screenWidth: flow.screen.x, screenHeight: flow.screen.y, backingPerCss: window.devicePixelRatio, gamut: flow.gamut})//rust runs the same fit on the picture's size to choose how large to render; never enlarged, so a small picture comes back at its own size
		let {width, height, gamut, naturalWidth, naturalHeight, pixels} = thumbnailUnpack(buffer)
		let canvas = flow.canvases.get(tile.path)
		if (flow.closed || !canvas) return//and one that went away while Rust worked throws the thumbnail away
		tile.natural = xy(naturalWidth, naturalHeight)
		flowSize(flow, canvas, flowFit(flow, tile.natural).css, xy(width, height))//the tile's size from the picture's own, which rust sends back for this, as every route works it out
		let context = canvas.getContext('2d', {colorSpace: flow.gamut})
		context.putImageData(new ImageData(pixels, width, height, {colorSpace: gamut}), 0, 0)//tagged with what the pixels are, so windows' srgb pixels are right on a wide-gamut canvas; anything past the canvas is clipped
		flowEdge(context, canvas, width, height)//the sliver flowSnap may have added, if this thumbnail came back a backing pixel short of its box
		logThumbnail({hit: 'native', render: Math.round(performance.now() - began), bytes: canvas.width * canvas.height * 4, natural: `${width}x${height}`, path: tile.path})
	} catch (error) {
		flowRefuse(flow, tile, String(error))
	}
}

async function flowPage1(flow, tile) {//one thumbnail made by the page from the store's decoded element, halved down; the essay above flowShrink says why halving rather than one draw
	if (flow.closed) return//before the store is asked, so a bucket that went away takes no reference it would have to give back
	let began = performance.now()
	let promise = cacheNeed(tile.path, flowHolder)//the reference is taken before any await, so the release below is owed from this line on
	try {
		let entry = await promise
		let canvas = flow.canvases.get(tile.path)
		if (flow.closed || !canvas) return
		if (entry.error) { flowRefuse(flow, tile, String(entry.error)); return }
		let natural = entry.natural
		if (!(natural.x > 0 && natural.y > 0)) { flowRefuse(flow, tile, 'the picture has no size'); return }
		tile.natural = natural

		let {scale, css} = flowFit(flow, natural)//the size it will show at, and the css pixels per image pixel that got it there
		let detail = Math.min(window.devicePixelRatio, 1 / scale)//canvas pixels per css pixel: devicePixelRatio, but never more than the file has; from the scale rather than the sizes, because a sliver rounds up to one css pixel
		let backing = xy(Math.max(1, Math.round(css.x * detail)), Math.max(1, Math.round(css.y * detail)))
		flowSize(flow, canvas, css, backing)
		flowShrink(canvas.getContext('2d', {colorSpace: flow.gamut}), entry.element, natural, xy(canvas.width, canvas.height), flow.gamut)//the canvas rather than the ask, so this route fills whatever flowSnap sized it to and never leaves an edge
		logThumbnail({hit: 'page', render: Math.round(performance.now() - began), bytes: canvas.width * canvas.height * 4, natural: `${canvas.width}x${canvas.height}`, path: tile.path})
	} catch (error) {
		flowRefuse(flow, tile, String(error))
	} finally {
		cacheRelease(tile.path, flowHolder)//the canvas holds these pixels now; the store may drop the bytes, the url, and the full-size decode
	}
}

function flowImg(flow, tile) {//a gif or an svg: the store's url, no decode; flowImgSize sizes it once the engine has it. Resolves once the url is set or refused
	let began = performance.now()
	flow.held.push(tile.path)
	return cacheNeed(tile.path, flowHolder, {decode: false})
		.then(entry => {
			if (entry.error) { flowRefuse(flow, tile, String(entry.error)); return }
			tile.url = entry.url
			logThumbnail({hit: 'img', render: Math.round(performance.now() - began), path: tile.path})//the read only; what the engine then spends showing it, nothing here can see
		})
		.catch(error => logTrouble('TestFlow: loading an img thumbnail', error))//the store answers a bad file with entry.error, so this catches only a store that broke
}

export function flowImgSize(flow, tile, natural) {//an img tile, sized by the fit once the engine knows its picture's size: a gif's own, an svg's width and height, or an svg's viewBox when it has no width and height, which is what WebKit reports then; and the beam's square should it report no size at all, which nothing has yet
	if (natural.x > 0 && natural.y > 0) { tile.natural = natural; tile.css = flowFit(flow, natural).css }
	else tile.css = xy(flow.beam, flow.beam)
}

export function flowDetails(tile) {//the caption's second line as runs: the day the file was last modified, the date Finder and Explorer show by default and the one a copy keeps, where Explorer gives a copy a new created time; its size, and the size the picture shows at, each left out until it is known, so a missing one never leaves a divider standing alone
	let parts = [sayCaptionDay(tile.file.mtime, settings), sayCaptionSize(tile.file.size, settings)]
	if (tile.natural) parts.push([sayDimensions(tile.natural)])//known once the thumbnail lands, and never for a file fuji refused
	return parts.filter(part => part.length).flatMap((part, i) => i == 0 ? part : [` ${middleDot} `, ...part])//the dot between the parts, chosen 2026-10-06 over a fullwidth slash, two spaces and a plain slash
}

function flowSize(flow, canvas, css, backing) {//size a canvas to the css size its fit chose and the backing pixels it has, which is also what gives its tile room in the flow; assigning width or height also clears it and resets its context, so it comes before any drawing
	canvas.width = flowSnap(css.x, backing.x); canvas.height = flowSnap(css.y, backing.y)
	canvas.style.width = css.x + 'px'; canvas.style.height = css.y + 'px'//the only place a canvas gets its css size, so the component binds none and never overwrites it
	flow.bytes += canvas.width * canvas.height * 4; flow.onBytes(flow.bytes)//the running total, which the bucket shows in its caption
}
function flowSnap(side, have) {//how many pixels a canvas gets for one axis: the css box in backing pixels, or the pixels in hand where those cannot reach it
	/*
	A canvas is laid out on whole css pixels and painted in backing pixels, so its box is a whole number of css pixels times devicePixelRatio however the fit above rounded. A bitmap that is not exactly that many backing pixels is not blitted one to one, it is resampled, and the phase of that resample walks a full pixel across the picture: on a retina panel a thumbnail one backing row short of its box comes back sharp at both ends and flat grey through the middle. Half of the thumbnails the operating system makes have an odd short side, so half of them land in that state, and none of it was visible on a machine where a css pixel and a backing pixel were the same thing.

	So the canvas is sized to the box rather than to the picture, and the picture is put in the corner of it. Two cases, and the constant tells them apart. A thumbnail shrunk to fit misses its box by at most one backing pixel of rounding, and that sliver is worth taking, because filling it buys a one to one blit for every row; flowEdge repeats the last row and column into it so the seam is the picture's own color. A picture smaller than the box misses it by far more than that and is meant to, since the fit leaves such a picture at its own size and the engine enlarges it the way an img tag would, so that one keeps the pixels it has.

	Nearest rather than down, and flooring was tried and measured and is worse. On a mac the choice is invisible: devicePixelRatio is 1 or 2, so the box is always a whole number of backing pixels and the two agree exactly. Windows scales at 125, 150 and 175 percent, where a box on whole css pixels lands on a quarter, a half or three quarters of a backing pixel and no canvas can sit on it either way. There the two differ on half of all sizes, and the reasoning that said flooring should win was this: across three scales, every canvas that came out larger than its box and whose row had drifted off the backing grid was resampled, while every canvas smaller than its box was clean. Never overshoot, and the compositor has nothing to resample.

	It does not hold. Every one of those clean undershoots was short by exactly a quarter of a backing pixel, which is what rounding happens to produce; flooring makes them short by a half or three quarters, and those resample as readily as an overshoot does — measured at 175 percent, where flooring fixed two tiles and broke three, including two in a row sitting squarely on the grid. A canvas cannot be made to fit a box that is not a whole number of backing pixels, and how far it misses by matters more than which side it misses on. So this rounds, the miss is at most half a backing pixel in either direction, and the tiles that still resample are left to the layout question that owns them.

	No specification promises any of this: it is how WebKit and Chromium lay out and composite a canvas today, so an engine update is a reason to measure it again, with one-pixel black and white rows authored at exactly the size the thumbnailer returns and a count of the pixels that come back neither black nor white, which takes minutes and answers in one percentage. The Image Fidelity page on fuji's site has the tiles all of this was read from.
	*/
	let backingPerCss = window.devicePixelRatio
	let want = Math.round(side * backingPerCss)//nearest, because the size of the miss matters more than its direction; flooring was measured and was worse
	return want > have + backingPerCss ? have : want
}
function flowEdge(context, canvas, width, height) {//fill whatever flowSnap left over by repeating the picture's last row and column into it; the row goes first, so the column carries it into the corner
	if (canvas.height > height) context.drawImage(canvas, 0, height - 1, width, 1, 0, height, width, canvas.height - height)
	if (canvas.width > width) context.drawImage(canvas, width - 1, 0, 1, canvas.height, width, 0, canvas.width - width, canvas.height)
}
function flowFit(flow, natural) {//the css size a picture of natural image pixels shows at under this flow's fit, and the css pixels per image pixel that got it there; every route sizes its tile here, from the picture's own size, and fit.js has the fits themselves
	let fitted = fitSize({fit: flow.fitName, width: natural.x, height: natural.y, beam: flow.beam, screenWidth: flow.screen.x, screenHeight: flow.screen.y})
	return {scale: fitted.scale, css: xy(fitted.width, fitted.height)}//whole css pixels, because that is the grid the engine lays a box out on; flowSize is where the pixels are then made to match it
}

/*
Why this halves rather than drawing once. Fuji's first thumbnails aliased on the Mac — the roof tiles of a 26-megapixel photograph turned to jaggies at 240 css pixels, while Safari showed the same file smooth as an img — and two things caused it. CoreGraphics' high interpolation reads a fixed footprint of source pixels around each output pixel, so at 26 to 1 most of the picture is never read, and pixels that are never read alias. And WebKit hands drawImage a subsampled frame only when it has to decode one: a frame already decoded at full size counts as good enough for any smaller request, and the store's decode() makes exactly that frame, so drawImage was given all 26 megapixels, where Safari's img, which never called decode(), was given a frame a quarter as wide and a quarter as tall. Halving until the last draw is within two to one puts every source pixel into the average. Chromium's high quality is already a chain of halvings under a cubic filter, so on Windows this is work the engine would have done anyway. Every step draws into a canvas, which both engines keep in premultiplied pixels, so a PNG's clear pixels bring the color hidden under them into no edge; that is read from how the engines store a canvas, and not counted as the native routes' edges were.

createImageBitmap looks like the purpose-built tool here, and on Chromium it is: ask it for resizeWidth and resizeQuality and it decodes straight to the size wanted. WebKit has the code and does not ship the options, so on the Mac it handed back a full-size bitmap — a whole second copy of a large photograph, allocated for nothing, then scaled by the same drawImage that could have done the job alone. It was tried and taken back out.
*/
function flowShrink(context, source, size, target, flowGamut) {//draw source, of size pixels, into context at target pixels, halving through scratch canvases until the last draw is within two to one; a fixed-footprint filter is only an honest average at that ratio. flowGamut is the flow's, the color space the context was made in
	let scratch = [document.createElement('canvas'), document.createElement('canvas')]//two, taken in turn, because a canvas cannot be shrunk into itself
	let step = 0
	while (size.x > target.x * 2) {//one axis is enough, because the fit kept the aspect; strictly more than double, so the last draw below is between one to one and two to one
		let half = xy(Math.ceil(size.x / 2), Math.ceil(size.y / 2))
		let canvas = scratch[step % 2]//never the one that is the current source
		canvas.width = half.x; canvas.height = half.y//sizing clears it and resets its context, so the quality is set again below
		let c = canvas.getContext('2d', {colorSpace: flowGamut})//the same gamut all the way down, so nothing is converted twice
		c.imageSmoothingQuality = 'high'
		c.drawImage(source, 0, 0, half.x, half.y)//the first step reads the store's element, and the engine applies the file's orientation there; every later step reads a canvas
		source = canvas; size = half; step++
	}
	context.imageSmoothingQuality = 'high'//rather than the default low, which reads only the four nearest source pixels per output pixel
	context.drawImage(source, 0, 0, target.x, target.y)
	for (let canvas of scratch) { canvas.width = 0; canvas.height = 0 }//give the backing stores back now; the first is a quarter of the picture's own size
}
