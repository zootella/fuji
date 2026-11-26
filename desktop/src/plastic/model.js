import {ref, shallowRef} from 'vue'
import parse from 'path-browserify'
import {listFolder} from './library.js'
import {settings, settingsChanged} from './settings.js'
import StrcmpSort from './sort/StrcmpSort.js'
import WindowsSort from './sort/WindowsSort.js'
import MacSort from './sort/MacSort.js'
import {log} from '../brass/log.js'//the shell starts the log before it starts the model, so a line from here always has somewhere to land

/*
The model holds what the user is looking at, and no view owns it: the folder, the order it is in, the images in that order, and which one the user is on. It sits here because the sheet and the tables are interchangeable views of the same thing — a user who presses c, or switches tables, expects the same folder in the same order. If the listing lived inside a table, the sheet would have to duplicate it or reach in for it, and reaching in is how two components stop being separable.

Two things shape the rest. The position is a path rather than an index, so changing the sort leaves the user on the picture they were looking at instead of on whatever is now forty-seventh; flipping is find where I am, step one, take that path. And the listing is kept unsorted, because it is what every sort reads from — which is what makes changing the order cost no disk.
*/

/*
Why fuji carries Explorer's order and Finder's, both on every platform, written in the page.

The user fuji imagines started on a Mac or on Windows and now uses both. The order their folders come in is one of the things their hands know, so fuji gives them the one they know as a choice, the same choice on either machine: a person who thinks in Explorer's order can have it on the Mac, and Finder's on Windows.

The other design is for fuji to sort like the Mac on the Mac and like Windows on Windows, asking each system's own function, StrCmpLogicalW there and localizedStandardCompare here. It would be less code and exact by construction, and it is not what fuji does, on purpose: the same folder would then read two ways on a user's two machines, and neither machine could give the other's order at all, since each function exists on one platform only. Fuji is meant to be simple, uniform and familiar, and uniform is the one that decides this.

So each order is fuji's own code, WindowsSort.js and MacSort.js, plain functions of two names that run the same in any web engine, and a sort is a decision about what the user sees, which is plastic's to make and nothing the Rust beneath it needs to know. Each file's essay says what it follows and how it was held against the real thing. side/sort holds that test, and WindowsTableSort.js beside it, an exact Windows sort built from Microsoft's table that could take WindowsSort.js's place here.
*/
const modelSorts = {//every order fuji can put a folder in, the way the shell keeps every table it can show; each is named as its file is, with Sort on the end as a fit ends in Fit, so the word in fuji.toml cannot be read as anything but a sort
	StrcmpSort,
	WindowsSort,
	MacSort,
}
export const modelSortNames = Object.keys(modelSorts)//for the settings panel, which offers them in this order
export const modelSortLabels = {//and what it calls each, since the name the code knows a sort by is not always the word for a user
	StrcmpSort:  'Simple lexicographic',
	WindowsSort: 'Windows',
	MacSort:     'Mac',
}

export const modelFolder = ref('')//the folder the user is in
export const modelList = shallowRef([])//its images as paths, in the current order; shallow because it is replaced whole, never edited
export const modelPath = ref('')//the image the user is on, and the durable answer to where they are
export const modelSort = ref('StrcmpSort')//chosen in the settings panel, read by every table

let modelFiles = []//the listing every sort orders, as it came off the disk; nothing renders it, so it stays out of vue
let modelFilesByPath = new Map()//the same entries by path, for a view that holds a path and wants what the listing knows about it

export function modelStart() { modelSortSet(settings.sort.order) }//the shell calls this once, after the settings file is read

export async function modelOpen(path) {//list the folder this image sits in, put it in order, and stand on the image
	if (typeof path != 'string' || !path.trim()) throw new Error(`opened a path that is not one: ${path}`)
	await _modelList(parse.dirname(path))
	modelPath.value = ''
	if (modelList.value.includes(path)) modelPath.value = path
	else if (modelList.value.length) modelPath.value = modelList.value[0]//the dropped file is not one fuji shows, so stand on the first that is; blank when the folder holds no images at all
}
export async function modelOpenFolder(folder) {//the same for a folder itself, with no picture in particular to stand on, so stand on the first
	if (typeof folder != 'string' || !folder.trim()) throw new Error(`opened a folder that is not one: ${folder}`)
	await _modelList(folder)
	modelPath.value = modelList.value[0] || ''//blank when the folder holds no images at all
}
async function _modelList(folder) {//list a folder and put it in order, which every way of opening one does first
	modelFiles = await listFolder(folder)
	modelFilesByPath = new Map(modelFiles.map(f => [f.path, f]))
	modelFolder.value = folder
	modelList.value = modelOrder()
}

export function modelIndex() { return modelList.value.indexOf(modelPath.value) }//or -1, when the current image is not in the list at all
export function modelStand(path) { modelPath.value = path }//what a flip and a click on a thumbnail both do
export function modelFile(path) { return modelFilesByPath.get(path) || false }//the listing's entry for a path in the current folder, with its name, size in bytes and modified time, or false for one the listing does not hold

export function modelSortSet(name) {//choose an order; the list is rebuilt from the listing already in hand, and every view is reading that list
	if (!modelSorts[name]) {//a name settings cannot check, because the sorts fuji has are known here and not there
		log(`settings: no sort named ${name}, using StrcmpSort instead`)//which is also how a file from when this sort was called Alphabet comes right
		name = 'StrcmpSort'
	}
	modelSort.value = name
	modelList.value = modelOrder()
	if (settings.sort.order != name) { settings.sort.order = name; settingsChanged() }//repair the file, the way a bad value anywhere else in it is repaired
}

export function modelCompare(a, b) { return modelSorts[modelSort.value](a, b) }//which of two entries, each with a name, comes first in the order fuji's one sort setting names, which every list in fuji runs its names through
function modelOrder() { return [...modelFiles].sort(modelCompare).map(file => file.path) }//the folder's paths in that order
