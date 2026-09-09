import {ref, shallowRef} from 'vue'
import parse from 'path-browserify'
import {listFolder} from './components/library.js'
import {settings, settingsChanged} from './settings.js'
import AlphabetSort from './AlphabetSort.js'

/*
The model holds what the user is looking at, and no view owns it: the folder, the order it is in, the images in that order, and which one the user is on. It sits here because the sheet and the tables are interchangeable views of the same thing — a user who presses c, or switches tables, expects the same folder in the same order. If the listing lived inside a table, the sheet would have to duplicate it or reach in for it, and reaching in is how two components stop being separable. architecture.md carries the longer argument.

Two things shape the rest. The position is a path rather than an index, so changing the sort leaves the user on the picture they were looking at instead of on whatever is now forty-seventh; flipping is find where I am, step one, take that path. And the listing is kept unsorted, because it is what every sort reads from — which is what makes changing the order cost no disk.
*/

const modelSorts = {//every order fuji can put a folder in, the way the shell keeps every table it can show
	Alphabet: AlphabetSort,
}

export const modelFolder = ref('')//the folder the user is in
export const modelList = shallowRef([])//its images as paths, in the current order; shallow because it is replaced whole, never edited
export const modelPath = ref('')//the image the user is on, and the durable answer to where they are
export const modelSort = ref('Alphabet')//chosen in the sheet, read by every table
export const modelShowing = ref('Table')//Sheet or Table: which kind of view is on screen. The shell writes it on every switch, and it is here rather than in the shell so a flow filling a hidden sheet can wait on it, doing none of its decoding behind the table's frames

let modelFiles = []//the listing every sort orders, as it came off the disk; nothing renders it, so it stays out of vue

export function modelStart() { modelSortSet(settings.sort.order) }//the shell calls this once, after the settings file is read

export async function modelOpen(path) {//list the folder this image sits in, put it in order, and stand on the image
	if (typeof path != 'string' || !path.trim()) throw new Error(`opened a path that is not one: ${path}`)
	let folder = parse.dirname(path)
	modelFiles = await listFolder(folder)
	modelFolder.value = folder
	modelList.value = modelOrder()

	modelPath.value = ''
	if (modelList.value.includes(path)) modelPath.value = path
	else if (modelList.value.length) modelPath.value = modelList.value[0]//the dropped file is not one fuji shows, so stand on the first that is; blank when the folder holds no images at all
}

export function modelIndex() { return modelList.value.indexOf(modelPath.value) }//or -1, when the current image is not in the list at all
export function modelStand(path) { modelPath.value = path }//what a flip and a click on a thumbnail both do

export function modelSortSet(name) {//choose an order; the list is rebuilt from the listing already in hand, and every view is reading that list
	if (!modelSorts[name]) {//a name settings cannot check, because the sorts fuji has are known here and not there
		console.log(`⭕ settings: no sort named ${name}, using Alphabet instead`)
		name = 'Alphabet'
	}
	modelSort.value = name
	modelList.value = modelOrder()
	if (settings.sort.order != name) { settings.sort.order = name; settingsChanged() }//repair the file, the way a bad value anywhere else in it is repaired
}

function modelOrder() { return modelSorts[modelSort.value](modelFiles) }//the only place a sort is called
