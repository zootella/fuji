import {platform} from './components/library.js'
import {selectionEmpty, selectionClick} from './selection.js'

/*
Fuji's own list, drawn with the page's own elements rather than through Vue. A list is many rows whose state changes a little at a time, a click marking one row or a keystroke in the filter hiding a few, and Vue answers every change by running its template over all of them to find the few that differ. Here each row is one element, made once and kept for the life of the list, and a change touches only the rows it changes: the filter sets hidden on the rows whose answer flipped, and a press marks the rows whose selection did. FujiList.vue is the thin layer that lets the rest of fuji place it as a component.
*/

export function listMake(root, {onSelect, onTrouble}) {//the list inside this element, which it owns from here on; onSelect hears what each press did, and onTrouble anything that throws where the page hands the list an event
	let o = {root, onSelect, rows: [], lower: [], elements: [], shown: [], filter: '', traits: selectionEmpty()}
	root.addEventListener('pointerdown', e => { try { _listPress(o, e) } catch (error) { onTrouble(error) } })
	/*
	A press in the list has its default action cancelled, and the list depends on it. A press in a page starts the browser's own text selection, and as the pointer moves afterward the engine keeps working out where that selection would end. The rows carry user-select: none, so no row is a place a selection can begin or end, and the engine searches for one by walking past every row in the list; WebKit's stack showed EventHandler::handleMousePressEvent and updateSelectionForMouseDrag down into PositionIterator::isCandidate. At two thousand rows on the Mac mini that search held the page for up to six seconds a press, and every click made meanwhile waited in line behind it. user-select: none keeps a selection from appearing, but only cancelling the mousedown keeps the engine from looking for one. The list's own selection rides the pointerdown above, which this leaves alone.

	The cost is the rest of that default: a press no longer moves the keyboard's focus, so when the list takes keys, its press focuses the list itself.

	The search ran in the browser's own process, outside any code of fuji's, where a timer in the page sees only how long a press waited in line. Sampling that process while it was busy is what named it.
	*/
	root.addEventListener('mousedown', e => e.preventDefault())
	return o
}

export function listRows(o, rows) {//replace every row, as for a new folder
	o.rows = rows
	o.lower = rows.map(row => row.toLowerCase())//once per list rather than once per keystroke
	o.elements = rows.map((text, i) => {
		let e = document.createElement('div')
		e.className = 'myRow'
		e.dataset.row = i//how a press finds its row
		/*
		A row's text is a file's name, and a file's name is text nobody vouched for. On the Mac and on Linux a name can hold angle brackets and quotes, and be written to look like markup that loads a picture and runs a script when the picture fails; a torrent's list of files carries whatever its maker typed. Assigned to innerHTML, the engine would parse that name into elements and run it. Assigned to textContent, the DOM replaces the row's children with a single text node holding the string exactly as it is, and nothing in it is ever parsed. That is how the rest of fuji is safe without saying so: Vue's double curly braces reach the page through the same doors, its runtime making each interpolation a node with createTextNode, updating it through nodeValue, and setting an element's whole text with textContent. OWASP's guide to preventing DOM-based cross-site scripting names textContent the safe assignment property. So nothing in the list writes markup from a string: a row that grows an icon or a column builds it with createElement and hands every part it did not write itself to textContent.
		*/
		e.textContent = text
		return e
	})
	let fragment = document.createDocumentFragment()
	for (let e of o.elements) fragment.append(e)
	o.root.replaceChildren(fragment)
	o.traits = selectionEmpty()
	_listShow(o)
}

export function listFilter(o, text) {//show only the rows that hold this text anywhere, ignoring case
	o.filter = text.toLowerCase()
	_listShow(o)
}

function _listShow(o) {
	let shown = []
	for (let i = 0; i < o.rows.length; i++) {
		let match = !o.filter || o.lower[i].includes(o.filter)
		if (o.elements[i].hidden == match) o.elements[i].hidden = !match//written only where the answer changed
		if (match) shown.push(i)
	}
	o.shown = shown//the rows on screen, in order, which a shift range runs along
}

function _listPress(o, e) {//on the press rather than the click, as Finder and Explorer select
	if (e.button != 0) return
	let start = performance.now()
	let toggle = platform() == 'mac' ? e.metaKey : e.ctrlKey//command on the mac, where control and a click is a right click
	let row = e.target.closest('[data-row]')
	let before = o.traits.selected
	if (!row) {
		if (e.offsetX >= o.root.clientWidth) return//the scrollbar, which a press on clears nothing
		if (!toggle && !e.shiftKey) o.traits = selectionEmpty()//a press below the last row clears, as it does in Finder and Explorer
	} else {
		o.traits = selectionClick(o.traits, Number(row.dataset.row), o.shown, {range: e.shiftKey, toggle})
	}
	/*
	The selection drawn by difference: the rows that left it lose their mark, the rows that joined it gain one, and no other row is touched. This is why selection.js returns a new value on every press instead of changing the one it was given, because the list needs the old selection and the new one side by side to compare them. A click then costs the one or two rows it changes, however long the list, and a shift range costs the rows in it. Drawing a selection the other way, by visiting every row to ask whether it is selected, costs the whole list on every press: that was the Vue version, which reran its template over every row and lagged at ten thousand. With 1,944 of 5,000 rows selected, a command-click marked its one row in well under a millisecond of the list's own time, on the Mac mini.
	*/
	let after = o.traits.selected, changed = 0
	for (let i of before) if (!after.has(i)) { delete o.elements[i].dataset.selected; changed++ }
	for (let i of after) if (!before.has(i)) { o.elements[i].dataset.selected = 'true'; changed++ }
	o.onSelect({
		selected: after.size,
		changed,
		kind: !row ? 'below the rows' : e.shiftKey ? (toggle ? 'shift and toggle' : 'shift') : toggle ? 'toggle' : 'plain',
		listMs: performance.now() - start,
	})
}
