//the fit table: fitSize from fit.js run over a grid of pictures, beams and screens, written as fit-table.txt beside this file for the test in src/fit.rs to read, so the two languages are checked across thousands of arguments rather than the dozens a person would type. node fit-table.js in this folder writes it; run it again when the arithmetic changes in both files at once, and read the diff
import {fitSize, fitNames} from '../desktop/src/fit.js'
import {writeFileSync} from 'node:fs'
import {fileURLToPath} from 'node:url'
import {dirname, join} from 'node:path'

const sizes = [1, 2, 3, 7, 16, 17, 20, 100, 119, 120, 121, 179, 180, 181, 240, 320, 479, 480, 481, 500, 640, 1080, 1920, 3000, 4032, 65535, 4294967295]//image pixels, each side: slivers, the sizes around a beam and three quarters of one where a rounding could go either way, the shapes a collection holds, and the largest a u32 takes
const beams = [120, 240, 360, 480]//css pixels, the four the user can choose
const screens = [[1920, 1080], [2560, 1440], [1440, 900], [3840, 2160], [1024, 768]]//css pixels, read by ScaleFit and LogFit alone

let lines = [
	'# fitSize from fit.js over a grid, written by fit-table.js; pnpm fit-table in the steel folder writes it again',
	'# a row is width height beam screen_width screen_height, then for each fit named on the next line its width height scale',
	'fits ' + fitNames.join(' '),
]
sizes.forEach((width, i) => {
	sizes.forEach((height, j) => {
		let beam = beams[(i + j) % beams.length]//one beam and one screen a pair, cycling, so every size meets every beam somewhere in the grid without the grid multiplying by both
		let [screenWidth, screenHeight] = screens[(i * 3 + j) % screens.length]
		let row = [width, height, beam, screenWidth, screenHeight]
		for (let fit of fitNames) {
			let r = fitSize({fit, width, height, beam, screenWidth, screenHeight})
			row.push(r.width, r.height, r.scale)//scale as node writes a double, the shortest text that reads back as the same double, which rust's parse reads back to the bit
		}
		lines.push(row.join(' '))
	})
})
writeFileSync(join(dirname(fileURLToPath(import.meta.url)), 'fit-table.txt'), lines.join('\n') + '\n')
console.log(`${lines.length - 3} rows of ${fitNames.length} fits each`)
