//./src/experiment.js - TEMPORARY, delete when the question below is answered

import {homeDir} from '@tauri-apps/api/path'
import parse from 'path-browserify'
import {diskRead, diskWrite} from './disk.js'
import {forwardize} from './components/library.js'

/*
Does an <img> that is not in the document keep its decoded pixels?

The cache means to hold decoded images in a Map, in nothing. The diamond table's triad holds its three in the document — two of them display none, but present — so it proves nothing about a detached one. If the engine only retains decodes for what it is painting, a cached image loses its bitmap and showing it again costs the full decode, silently, which is the cache failing at the one thing it exists for.

img.decode() is the probe: it resolves at once for an image already decoded, and takes the full time for one that is not. The six timings below each answer something different; read the report they write.

This runs in the tauri webview on purpose. On macOS that is WKWebView and on Windows it is Chromium, and they may not answer the same way.
*/

const subject = 'Documents/here/colors for fuji/1red.jpg'//a 6240 x 4160 progressive jpeg that takes about a second to decode, which is what makes it a good subject
const reportName = 'fuji-experiment.txt'

export async function experimentRun() {
	let home = forwardize(await homeDir())
	let lines = []
	let say = line => { lines.push(line); console.log('🧪 '+line) }

	let bytes = new Uint8Array(await diskRead(parse.join(home, subject)))
	let blob = new Blob([bytes.buffer], {type: 'image/jpeg'})
	say(`subject ${subject}`)
	say(`file ${bytes.length.toLocaleString()} bytes`)

	let url1 = URL.createObjectURL(blob)

	let a = new Image()//1: cold, detached. the baseline every other number is read against
	say(`1 cold decode, detached element, url1 .......... ${await timeDecode(a, url1)}ms`)
	say(`   natural ${a.naturalWidth} x ${a.naturalHeight}, ${(a.naturalWidth*a.naturalHeight*4/1048576).toFixed(0)} MiB decoded`)

	say(`2 same element decoded again ................... ${await timeAgain(a)}ms`)//does this element keep its own decode

	let b = new Image()//3: a different element, same url, while the first is still referenced
	say(`3 new element, same url, first still alive ..... ${await timeDecode(b, url1)}ms`)

	a.src = ''; b.src = ''//4: nothing references the decode any more, though javascript cannot force collection, so a fast number here may only mean it has not been collected yet
	let c = new Image()
	say(`4 new element, same url, others released ....... ${await timeDecode(c, url1)}ms`)

	let url2 = URL.createObjectURL(blob)//5: same bytes, different url. tells us whether decodes are keyed by url or by content
	let d = new Image()
	say(`5 new element, new url over the same blob ...... ${await timeDecode(d, url2)}ms`)

	let url3 = URL.createObjectURL(blob)//6: the triad's situation, an element that is in the document
	let e = new Image()
	e.style.position = 'fixed'; e.style.width = '1px'; e.style.height = '1px'; e.style.opacity = '0'
	document.body.appendChild(e)
	say(`6 attached element, fresh url .................. ${await timeDecode(e, url3)}ms`)
	let f = new Image()
	say(`  detached element, same url as the attached one ${await timeDecode(f, url3)}ms`)

	let text = lines.join('\n')+'\n'
	await diskWrite(parse.join(home, reportName), Array.from(new TextEncoder().encode(text)))
	console.log('🧪 wrote '+parse.join(home, reportName))
}

async function timeDecode(img, url) {//set a source and wait for pixels, which is the whole measurement
	let t = performance.now()
	img.src = url
	await img.decode()
	return Math.round(performance.now() - t)
}
async function timeAgain(img) {//decode an element that has already decoded; instant means it kept its pixels
	let t = performance.now()
	await img.decode()
	return Math.round(performance.now() - t)
}
