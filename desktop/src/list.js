import {platform} from './components/library.js'
import {selectionEmpty, selectionClick, selectionMove, selectionAll} from './selection.js'

/*
Fuji's own list, drawn with the page's own elements rather than through Vue. A list is many rows whose state changes a little at a time, a click marking one row or a keystroke in the filter hiding a few, and Vue answers every change by running its template over all of them to find the few that differ. Here each row is one element, made once and kept for the life of the list, and a change touches only the rows it changes: the filter sets hidden on the rows whose answer flipped, and a press or a key marks the rows whose selection or focus did. FujiList.vue is the thin layer that lets the rest of fuji place it as a component.
*/

export function listMake(root, {onSelect, onTrouble}) {//the list inside this element, which it owns from here on; onSelect hears what each press or key did, and onTrouble anything that throws where the page hands the list an event
	let o = {root, onSelect, rows: [], lower: [], elements: [], shown: [], filter: '', traits: selectionEmpty()}
	root.addEventListener('pointerdown', e => { try { _listPress(o, e) } catch (error) { onTrouble(error) } })
	root.addEventListener('focus', () => { root.dataset.active = 'true' })//an attribute for the stylesheet rather than :focus, because WebKit keeps a row's style from while the list had the keyboard after its window goes behind another, though :focus itself stops matching; a change of attribute always restyles
	root.addEventListener('blur', () => { delete root.dataset.active })
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

export function listFilter(o, text) {//show only the rows that hold this text anywhere, ignoring case; a change of filter clears the selection, like a press below the rows, so a row the filter hides is never one a command could act on
	o.filter = text.toLowerCase()
	let before = o.traits
	o.traits = selectionEmpty()
	_listDraw(o, before)
	_listShow(o)
}

export function listKey(o, e) {//one key, acted on only while the list has the keyboard, since every key in the view reaches here; true when the list used it
	if (document.activeElement != o.root) return false
	let start = performance.now()
	let before = o.traits, kind
	let selectAll = (platform() == 'mac' ? e.metaKey : e.ctrlKey) && !e.shiftKey && !e.altKey && e.key.toLowerCase() == 'a'
	if (selectAll) {
		o.traits = selectionAll(o.traits, o.shown)
		kind = 'select all'
	} else {
		if (e.ctrlKey || e.metaKey || e.altKey) return false//a movement key with any of these is the system's or another command's, like command and down to open on the mac
		let to = _listTarget(o, e.key)
		if (to == -1) return false
		o.traits = selectionMove(o.traits, to, o.shown, {range: e.shiftKey})
		o.elements[to].scrollIntoView({block: 'nearest'})
		kind = e.shiftKey ? `shift ${e.key}` : e.key
	}
	e.preventDefault()//or the focused list would also scroll itself by the key
	o.root.dataset.keyboard = 'true'//the focus mark shows from the first key on, as Windows shows its focus rectangle only once the keyboard has been used, and keeps showing it
	let changed = _listDraw(o, before)
	o.onSelect({selected: o.traits.selected.size, changed, kind, listMs: performance.now() - start})
	return true
}

function _listTarget(o, key) {//the row a movement key takes the focus to, or -1 for a key that moves nothing
	let shown = o.shown
	if (!shown.length) return -1
	let at = shown.indexOf(o.traits.focus)//-1 before any row has the focus, when a key starts from the end it points away from
	let page = Math.max(1, Math.floor(o.root.clientHeight / o.elements[shown[0]].offsetHeight) - 1)//a screen of rows less one, so the row the page key left stays in view
	let k
	if      (key == 'ArrowDown') k = at == -1 ? 0 : at + 1
	else if (key == 'ArrowUp')   k = at == -1 ? shown.length - 1 : at - 1
	else if (key == 'PageDown')  k = at == -1 ? 0 : at + page
	else if (key == 'PageUp')    k = at == -1 ? shown.length - 1 : at - page
	else if (key == 'Home')      k = 0
	else if (key == 'End')       k = shown.length - 1
	else return -1
	return shown[Math.min(Math.max(k, 0), shown.length - 1)]
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
	o.root.focus({preventScroll: true})//the keyboard to the list, which the press's cancelled default would otherwise have done
	let toggle = platform() == 'mac' ? e.metaKey : e.ctrlKey//command on the mac, where control and a click is a right click
	let row = e.target.closest('[data-row]')
	let before = o.traits
	if (!row) {
		if (e.offsetX >= o.root.clientWidth) return//the scrollbar, which a press on clears nothing
		if (!toggle && !e.shiftKey) o.traits = selectionEmpty()//a press below the last row clears, as it does in Finder and Explorer
	} else {
		o.traits = selectionClick(o.traits, Number(row.dataset.row), o.shown, {range: e.shiftKey, toggle})
	}
	let changed = _listDraw(o, before)
	o.onSelect({
		selected: o.traits.selected.size,
		changed,
		kind: !row ? 'below the rows' : e.shiftKey ? (toggle ? 'shift and toggle' : 'shift') : toggle ? 'toggle' : 'plain',
		listMs: performance.now() - start,
	})
}

/*
The selection drawn by difference: the rows that left it lose their mark, the rows that joined it gain one, the focus mark moves from the row that had it to the row that has it, and no other row is touched. This is why selection.js returns a new value on every change instead of changing the one it was given, because the list needs the old selection and the new one side by side to compare them. A click then costs the one or two rows it changes, however long the list, a shift range costs the rows in it, and clearing costs the rows that were selected. Drawing a selection the other way, by visiting every row to ask whether it is selected, costs the whole list every time: that was the Vue version, which reran its template over every row and lagged at ten thousand. With 1,944 of 5,000 rows selected, a command-click marked its one row in well under a millisecond of the list's own time, on the Mac mini.
*/
function _listDraw(o, before) {//the marks onto the rows whose selection or focus changed since the traits before, and how many rows changed selection
	let after = o.traits, changed = 0
	for (let i of before.selected) if (!after.selected.has(i)) { delete o.elements[i].dataset.selected; changed++ }
	for (let i of after.selected) if (!before.selected.has(i)) { o.elements[i].dataset.selected = 'true'; changed++ }
	if (before.focus != after.focus) {
		if (before.focus != -1) delete o.elements[before.focus].dataset.focus
		if (after.focus != -1) o.elements[after.focus].dataset.focus = 'true'
	}
	return changed
}
