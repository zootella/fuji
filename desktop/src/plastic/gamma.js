import {ref} from 'vue'
import {settings} from './settings.js'

/*
The gamma fuji is showing every picture through, as one number in the viewer's convention: 1 changes nothing and a bigger number lifts the shadows further, and the filter the shell draws with applies a power of 1 over it. Four things change it, and they are built to be used together. The g key brightens to the gentle lift fuji.toml names when the gamma is exactly 1, and returns it to exactly 1 from anywhere else, so it is the way in and always the way back out. Shift with plus or minus steps it up or down by a fixed amount, from wherever it is, and so does shift with the wheel on a table, by a smaller step a notch, since a wheel is easier to flick than a key is to press. And a shift and right drag on a table sets it to anything, live, the way a slider would. The steps and the drag share one floor. The number is never written anywhere, so fuji starts at 1 on every launch.

It lives in a module of its own because it is the shell's to draw and a table's to drag, and neither should have to reach into the other for it.
*/

export const gamma = ref(1)//exactly 1 when off, which is when the shell draws no filter at all

export function gammaToggle() {//the g key: brighten from normal, and return to normal from anything else
	gamma.value = gamma.value == 1 ? settings.gamma.key : 1//read every time, so a number changed in fuji.toml means what it says
}

export function gammaStep(change) {//shift with plus or minus, or a notch of the wheel: add change, negative to darken, from wherever the gamma is, and never below gamma.floor. The caller says how much, because the keys and the wheel step by different settings
	gamma.value = Math.max(settings.gamma.floor, Math.round((gamma.value + change) * 1e6) / 1e6)//rounded to a millionth, because a step like 0.2 is not exact in binary and would drift, 1.6 arriving as 1.5999999999999999; without it a walk up and back down could miss 1 by a hair, and g would read that as not normal and leave the filter on
}

export function gammaDrag(start, height, frame) {//a drag height above where it began, in a frame of that height: linear from the gamma the drag found, like a slider laid up the frame, gamma.drag over the whole height and never below gamma.floor, which keeps it from 0, a power of infinity
	gamma.value = Math.max(settings.gamma.floor, start + settings.gamma.drag * height / frame)
}
