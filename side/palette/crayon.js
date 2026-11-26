/*
A box of crayons picked from a palette, for the palette's Crayon Boxes tab.

A palette with a ladder under every hue is a grid: one hue at many lightnesses, where nobody has checked whether one hue's fourth step can be told from its neighbor's fifth. What fuji wants is a box: a flat set of crayons, several reds and blues and browns, uneven if the colors fall that way, where any few taken out and drawn beside each other can be seen and told apart. This module picks such a box from whatever crayons a palette has, by a few measurements and no taste, so the same rule is put to every palette and the boxes can be compared. It is a sample, a first pass for a person to change, and it holds no color of its own.

The measurements. A crayon can be seen when it contrasts with the ground it is drawn on, by the ratio the accessibility guidelines define, at least the floor below. Two crayons can be told apart when they are far enough from each other in OKLab, the space style.css measures fuji's grays in, where a distance is a straight line between two colors and equal distances look about equal: at least apart below. And a crayon is a color at all when it has enough chroma, since a gray is fuji's own and never a crayon. It also has to stand off the text's color, by ink below, more than it must off another crayon.

The picking is farthest first. It starts from the two colors a drawing must never be taken for, the ground and the text, which a drawing with no color of its own is in, and takes the crayon farthest from everything taken so far, weighing in how strong its color is, again and again, until every crayon left is nearer than apart to something in the box, or the box is full. So the first crayons out are the palette's strongest and most different, and each one after is the strongest and most different from all of those, which fills in a second blue or a third brown only where there is room for one. Every pair in a box is at least apart, which is more than fuji asked for; the limit is loose enough that a box still comes out with close neighbors inside a family.

A crayon's family is the rainbow's word for it, red, orange, brown and so on, worked out from its hue and lightness by the ranges below, which are this page's own and rough: a brown is an orange or a yellow gone dark, which is why the warm colors make so many crayons on white and red makes few.
*/

export const crayonLimits = {
	floor: 1.6,//the least contrast with the ground a crayon can have and be drawn, as a ratio to 1. The guidelines ask 3 of a drawing that has to be understood, which no yellow passes on white until it is a brown; 1.6 lets a strong amber in, like Radix's amber 10, so a box can have a yellow at all, and the page says each crayon's ratio and marks the ones under 3, so the pale ones can be seen for what they are
	apart: 0.09,//the least distance in OKLab between any two crayons in a box. About what separates Tailwind's blue 500 from its sky 500, two colors of one family a person names differently
	chroma: 0.04,//the least chroma in OKLab a crayon can have, under which it is a gray
	ink: 0.16,//the least distance between a crayon and the text's color, wider than apart, since a thin line in a navy or a maroon that dark reads as a line in the text's own color
	strong: 1,//how much a strong color is preferred to a far one in the picking: at 0 the box takes the farthest crayon whatever its chroma, and at 1 a crayon twice as strong is wanted as much as one twice as far, so the box fills with a palette's vivid crayons before its dull ones. Every pair is still at least apart; this changes which crayons are taken, and matters most in a palette with more crayons than a box holds
	most: 24,//the most crayons in a box, the size of a box of crayons a child is given
}

export const crayonFamilies = ['red', 'orange', 'brown', 'yellow', 'green', 'cyan', 'blue', 'purple', 'pink']//the families in the order a box lists them, around the color wheel from red, with brown beside the orange it is a dark shade of

function linear(v) { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }//one channel of sRGB, 0 to 255, as linear light

export function crayonLab([r, g, b]) {//a color in OKLab, from its red, green and blue in sRGB, 0 to 255, with its chroma and its hue in degrees beside the three coordinates; the matrices are Björn Ottosson's, who defined the space
	r = linear(r); g = linear(g); b = linear(b)
	let l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
	let m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
	let s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
	let lab = {l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s, a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s, b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s}
	lab.chroma = Math.hypot(lab.a, lab.b)
	lab.hue = (Math.atan2(lab.b, lab.a) * 180 / Math.PI + 360) % 360
	return lab
}

export function crayonApart(p, q) { return Math.hypot(p.l - q.l, p.a - q.a, p.b - q.b) }//how far two colors in OKLab are from each other

export function crayonContrast(p, q) {//the contrast between two colors as the accessibility guidelines define it, from 1, the same, to 21, black on white
	let luminance = ([r, g, b]) => 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
	let [high, low] = [luminance(p), luminance(q)].sort((x, y) => y - x)
	return (high + 0.05) / (low + 0.05)
}

export function crayonFamily(lab) {//the rainbow's word for a color, by its hue in OKLab and, for brown, its lightness
	let h = lab.hue
	if (lab.l < 0.58 && ((h >= 38 && h < 110) || (h >= 25 && h < 38 && lab.chroma < 0.12))) return 'brown'//an orange or a yellow gone dark, and a red gone dark and dull with it, where a dark red still strong is a red
	if (h >= 10 && h < 38) return 'red'
	if (h >= 38 && h < 65) return 'orange'
	if (h >= 65 && h < 110) return 'yellow'
	if (h >= 110 && h < 170) return 'green'
	if (h >= 170 && h < 225) return 'cyan'
	if (h >= 225 && h < 285) return 'blue'
	if (h >= 285 && h < 335) return 'purple'
	return 'pink'
}

export function crayonBox(crayons, ground, text, limits = crayonLimits) {//a box picked from crayons, each a name, a color as its maker writes it, and rgb, its red, green and blue; ground and text are rgb too. Answers the box, each crayon with its family, its contrast with the ground and its nearest neighbor in the box, ordered by family and then from light to dark, and beside it how many crayons were too pale, too gray or too near the text to be picked from
	let groundLab = crayonLab(ground), textLab = crayonLab(text)
	let seen = new Set(), able = [], pale = 0, gray = 0, dark = 0
	for (let c of crayons) {
		let key = c.rgb.join()
		if (seen.has(key)) continue//the same color under two names is one crayon, the first
		seen.add(key)
		let lab = crayonLab(c.rgb), contrast = crayonContrast(c.rgb, ground)
		if (lab.chroma < limits.chroma) { gray++; continue }
		if (contrast < limits.floor) { pale++; continue }
		if (crayonApart(lab, textLab) < limits.ink) { dark++; continue }
		able.push({...c, lab, contrast, family: crayonFamily(lab)})
	}
	let box = [], taken = [groundLab, textLab]
	let reach = able.map(c => Math.min(...taken.map(t => crayonApart(c.lab, t))))//each crayon's distance from the nearest color taken so far
	let score = i => reach[i] * able[i].lab.chroma ** limits.strong//how much a crayon is wanted next: its distance from everything taken, times its chroma to the power strong
	while (box.length < limits.most) {
		let best = -1
		for (let i = 0; i < able.length; i++) if (reach[i] >= limits.apart && (best < 0 || score(i) > score(best))) best = i
		if (best < 0) break
		box.push(able[best])
		for (let i = 0; i < able.length; i++) reach[i] = Math.min(reach[i], crayonApart(able[i].lab, able[best].lab))
		reach[best] = -1
	}
	for (let c of box) {
		c.nearest = box.filter(o => o != c).map(o => ({name: o.name, apart: crayonApart(c.lab, o.lab)})).sort((x, y) => x.apart - y.apart)[0] || null
	}
	box.sort((x, y) => crayonFamilies.indexOf(x.family) - crayonFamilies.indexOf(y.family) || y.lab.l - x.lab.l)
	return {box, from: able.length, pale, gray, dark}
}
