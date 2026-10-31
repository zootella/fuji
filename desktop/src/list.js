import {Time} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in
import {isCommandOrControl} from './keyboard.js'
import {selectionEmpty, selectionClick, selectionMove, selectionFocus, selectionSmear, selectionAll} from './selection.js'

/*
Fuji's own list, drawn with the page's own elements rather than through Vue. A list is many rows whose state changes a little at a time, a click marking one row or a keystroke in the filter hiding a few, and Vue answers every change by running its template over all of them to find the few that differ. Here each row is one element, made once and kept for the life of the list, and a change touches only the rows it changes: the filter sets hidden on the rows whose answer flipped, and a press or a key marks the rows whose selection or focus did. Every row is in the page, with no windowing and no virtualizer: this plain list built, scrolled and filtered ten thousand rows without trouble on the Mac mini and the Windows box, scrolls on the engine's compositor with no script running, and leaves scrolling a row into view to the browser. FujiUserInterfaceSpike.vue is the bench that times it. FujiList.vue is the thin layer that lets the rest of fuji place it as a component.
*/

export function listMake(frame, {columns, onSelect, onSort, onTrouble}) {//the list inside this element, which it owns from here on: a row of headers, and below it the rows, which scroll. columns says what each row shows and what its header says, onSelect hears what each press, key or drag did, onSort each sort, and onTrouble anything that throws where the page hands the list an event
	let header = document.createElement('div'), root = document.createElement('div')
	header.className = 'myHeader'
	root.className = 'myList'
	root.tabIndex = 0//so the list can take the keyboard
	frame.replaceChildren(header, root)
	for (let c of columns) if (typeof c.width != 'number' || c.width <= 0) throw new Error(`expected a width in css pixels for column ${c.key}: ${c.width}`)
	let o = {frame, header, root, columns, onSelect, onSort, widths: columns.map(c => c.width), rows: [], lower: [], elements: [], order: [], shown: [], filter: '', sort: {column: 0, descending: false}, traits: selectionEmpty(), typed: '', typedAt: 0}//sorted by the first column from the start, the name, as Finder and Explorer open a folder
	for (let [i, column] of columns.entries()) {
		let cell = document.createElement('span')
		cell.className = 'myListCell'
		cell.dataset.column = i//how a click finds its column
		if (column.align) cell.dataset.align = column.align
		cell.textContent = column.title
		if (i == o.sort.column) cell.dataset.sort = 'up'
		let grip = document.createElement('span')
		grip.className = 'myGrip'//the divider at the column's right edge, which a drag resizes it by and a double-click fits
		cell.append(grip)
		header.append(cell)
	}
	_listGrid(o)
	let gate = handle => e => { try { handle(o, e) } catch (error) { onTrouble(error) } }
	root.addEventListener('pointerdown', gate(_listPress))
	header.addEventListener('click', gate(_listHeaderClick))
	header.addEventListener('pointerdown', gate(_listGripPress))
	header.addEventListener('dblclick', gate(_listGripFit))
	root.addEventListener('scroll', () => { header.scrollLeft = root.scrollLeft }, {passive: true})//the headers follow the rows sideways, since they sit outside what scrolls
	root.addEventListener('focus', () => { root.dataset.active = 'true' })//an attribute for the stylesheet rather than :focus, because WebKit keeps a row's style from while the list had the keyboard after its window goes behind another, though :focus itself stops matching; a change of attribute always restyles
	root.addEventListener('blur', () => { delete root.dataset.active })
	/*
	A press in the list has its default action cancelled, and the list depends on it. A press in a page starts the browser's own text selection, and as the pointer moves afterward the engine keeps working out where that selection would end. The rows carry user-select: none, so no row is a place a selection can begin or end, and the engine searches for one by walking past every row in the list; WebKit's stack showed EventHandler::handleMousePressEvent and updateSelectionForMouseDrag down into PositionIterator::isCandidate. At two thousand rows on the Mac mini that search held the page for up to six seconds a press, and every click made meanwhile waited in line behind it. user-select: none keeps a selection from appearing, but only cancelling the mousedown keeps the engine from looking for one. The list's own selection rides the pointerdown above, which this leaves alone.

	The cost is the rest of that default: a press no longer moves the keyboard's focus, so when the list takes keys, its press focuses the list itself.

	The search ran in the browser's own process, outside any code of fuji's, where a timer in the page sees only how long a press waited in line. Sampling that process while it was busy is what named it.
	*/
	frame.addEventListener('mousedown', e => e.preventDefault())
	return o
}

export function listRows(o, rows) {//replace every row, as for a new folder; a row is a record holding a value for each column's key
	o.rows = rows
	let first = o.columns[0].key
	o.lower = rows.map(row => String(row[first]).toLowerCase())//the first column, the name, is what the filter matches, as Finder's search does; lowered once per list rather than once per keystroke
	o.elements = rows.map((row, i) => {
		let e = document.createElement('div')
		e.className = 'myRow'
		e.dataset.row = i//how a press finds its row
		for (let column of o.columns) {
			let cell = document.createElement('span')
			cell.className = 'myListCell'
			if (column.align) cell.dataset.align = column.align
			/*
			A cell's text is a file's name or something said about the file, and a file's name is text nobody vouched for. On the Mac and on Linux a name can hold angle brackets and quotes, and be written to look like markup that loads a picture and runs a script when the picture fails; a torrent's list of files carries whatever its maker typed. Assigned to innerHTML, the engine would parse that name into elements and run it. Assigned to textContent, the DOM replaces the element's children with a single text node holding the string exactly as it is, and nothing in it is ever parsed. That is how the rest of fuji is safe without saying so: Vue's double curly braces reach the page through the same doors, its runtime making each interpolation a node with createTextNode, updating it through nodeValue, and setting an element's whole text with textContent. OWASP's guide to preventing DOM-based cross-site scripting names textContent the safe assignment property. So nothing in the list writes markup from a string: a row that grows an icon builds it with createElement and hands every part it did not write itself to textContent.
			*/
			if (column == o.columns[0]) {//the name in a span as wide as its text, so a press on the name can be told from a press in the space beside it
				let name = document.createElement('span')
				name.className = 'myName'
				name.textContent = _listSay(column, row)
				cell.append(name)
			} else {
				cell.textContent = _listSay(column, row)
			}
			e.append(cell)
		}
		return e
	})
	o.root.replaceChildren()
	o.index = new Map(rows.map((row, i) => [row, i]))//each row's place in rows, for a column's sort, which hands rows back rather than their places
	o.order = rows.map((row, i) => i)
	o.order = _listSorted(o)//in the sort the list already has, so new rows keep the one the user chose for the old ones
	_listPlace(o)
	o.traits = selectionEmpty()
	_listShow(o)
}

function _listSay(column, row) { return column.say ? column.say(row[column.key]) : String(row[column.key]) }//the text a cell shows for a row's value

const listNarrowest = 24//css pixels, the narrowest a drag leaves a column, which still leaves its divider to grab
function _listGrid(o) { o.frame.style.setProperty('--list-columns', o.widths.map(w => `${w}px`).join(' ')) }//the widths onto the one grid the headers and rows share

function _listGripPress(o, e) {//a press on a divider resizes the column to its left as the pointer moves; the columns to its right move over, and past the list's edge the list scrolls sideways
	let grip = e.target.closest('.myGrip')
	if (!grip || e.button != 0) return
	let column = Number(grip.parentElement.dataset.column), startX = e.clientX, startWidth = o.widths[column]
	grip.setPointerCapture(e.pointerId)//so the drag keeps the pointer outside the header and outside the window
	let move = m => { o.widths[column] = Math.max(listNarrowest, Math.round(startWidth + m.clientX - startX)); _listGrid(o) }
	let end = () => { grip.removeEventListener('pointermove', move); grip.removeEventListener('pointerup', end); grip.removeEventListener('pointercancel', end) }
	grip.addEventListener('pointermove', move)
	grip.addEventListener('pointerup', end)
	grip.addEventListener('pointercancel', end)
}

function _listGripFit(o, e) {//a double-click on a divider fits its column to the widest text in it, the header's included, as Finder and Explorer do
	let grip = e.target.closest('.myGrip')
	if (!grip) return
	let column = Number(grip.parentElement.dataset.column), c = o.columns[column]
	let measure = document.createElement('canvas').getContext('2d')
	let style = getComputedStyle(grip.parentElement)
	measure.font = `${style.fontStyle} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`//the list's own font, so text is measured without drawing any of it, quickly enough for ten thousand rows at once
	let widest = measure.measureText(`${c.title} ▲ `).width//room for the header and the sort arrow beside it
	for (let row of o.rows) widest = Math.max(widest, measure.measureText(_listSay(c, row)).width)
	o.widths[column] = Math.ceil(widest) + 16 + 2//the cell's padding at either end, and a pixel each side so the last letter is never cut
	_listGrid(o)
}

function _listHeaderClick(o, e) {//a click on a header sorts by its column, and another on the same header reverses it, as Explorer and Finder do
	if (e.target.closest('.myGrip')) return//a press on a divider resizes, and the click it ends with sorts nothing
	let cell = e.target.closest('[data-column]')
	if (!cell) return
	let start = performance.now()
	o.root.focus({preventScroll: true})
	let column = Number(cell.dataset.column)
	o.sort = {column, descending: o.sort.column == column ? !o.sort.descending : false}
	o.order = _listSorted(o)
	_listPlace(o)
	_listShow(o)
	for (let h of o.header.children) delete h.dataset.sort
	cell.dataset.sort = o.sort.descending ? 'down' : 'up'//the arrow the stylesheet draws beside the sorted header
	if (o.traits.focus != -1) o.elements[o.traits.focus].scrollIntoView({block: 'nearest'})//the row the keyboard is on stays in view wherever the sort took it
	o.onSort({key: o.columns[column].key, descending: o.sort.descending, listMs: performance.now() - start})
}

function _listSorted(o) {//the rows' order sorted by the chosen column, from the order they are in now; a stable sort leaves rows that tie in the order they had, so the column sorted before becomes the second key, a spreadsheet's way and what Explorer documents
	let {key, sort} = o.columns[o.sort.column]
	if (sort) {//a column that brings a sort of its own, which takes rows and hands them back in order, like the name column running fuji's one sort setting
		let order = sort(o.order.map(i => o.rows[i])).map(row => o.index.get(row))
		return o.sort.descending ? order.reverse() : order
	}
	let direction = o.sort.descending ? -1 : 1
	return [...o.order].sort((i, j) => direction * _listCompare(o.rows[i][key], o.rows[j][key]))//and the rest by their raw values, numbers like a size in bytes or a date in milliseconds
}
function _listCompare(a, b) { return a < b ? -1 : a > b ? 1 : 0 }

function _listPlace(o) {//the row elements into the list in the order, moving rather than making them, so each keeps its marks
	let fragment = document.createDocumentFragment()
	for (let i of o.order) fragment.append(o.elements[i])
	o.root.append(fragment)
}

export function listFilter(o, text) {//show only the rows whose first column holds this text anywhere, ignoring case; a change of filter clears the selection, like a press below the rows, so a row the filter hides is never one a command could act on
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
	let command = isCommandOrControl(e)
	if (e.altKey || ((e.ctrlKey || e.metaKey) && !command)) return false//a chord with alt, or with the modifier that is not the command key, like control on the mac, is the system's
	if (command && !e.shiftKey && e.key.toLowerCase() == 'a') {
		o.traits = selectionAll(o.traits, o.shown)
		kind = 'select all'
	} else {
		let to = _listTarget(o, e.key)
		if (to != -1) {
			o.typed = ''//a movement key ends a name being typed
			if (command && e.shiftKey) o.traits = selectionClick(o.traits, to, o.shown, {range: true, toggle: true})//with the command key and shift, the rows from the anchor join the selection, as a click with both keys adds them
			else if (command) o.traits = selectionFocus(o.traits, to)//with the command key alone only the keyboard moves, as control and an arrow move it in Explorer, so space can pick rows one at a time
			else o.traits = selectionMove(o.traits, to, o.shown, {range: e.shiftKey})
			kind = `${command ? 'command ' : ''}${e.shiftKey ? 'shift ' : ''}${e.key}`
		} else if (e.key == ' ' && (command || !_listTyping(o, e))) {//space toggles the row the keyboard is on, as control and space pick rows one at a time in Explorer, with the command key still held from the arrows or without it, and is a letter only inside a name being typed
			to = o.traits.focus
			if (to == -1) { e.preventDefault(); return true }//no row has the keyboard yet, and the space still must not scroll the list
			o.traits = selectionClick(o.traits, to, o.shown, {range: false, toggle: true})
			kind = 'space'
		} else {
			if (command) return false//a chord the list has no use for, like copy, which belongs to the page around it or to the system
			to = _listFind(o, e)
			if (to == -1) return false
			o.traits = selectionMove(o.traits, to, o.shown, {range: false})
			kind = 'typed'//and not what was typed, which is the start of a file's name, so a log the list's reports reach names no file
		}
		o.elements[to].scrollIntoView({block: 'nearest'})
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

const listTypedRest = 1*Time.second//how long without a key before the next one starts a new name rather than adding to the one being typed; a second, the same as Chromium's own type-ahead in a select
function _listTyping(o, e) { return o.typed != '' && e.timeStamp - o.typedAt <= listTypedRest }//whether this key comes while a name is being typed
function _listFind(o, e) {//the row a typed character takes the focus to: the next shown, in the list's order and wrapping at the end, whose name starts with what has been typed so far, ignoring case, as VS Code's list jumps; or -1
	if ([...e.key].length != 1) return -1//a key named rather than typed, like Tab or Enter
	let typing = _listTyping(o, e)
	if (!typing) o.typed = ''
	o.typed += e.key.toLowerCase()
	o.typedAt = e.timeStamp
	let shown = o.shown, at = shown.indexOf(o.traits.focus)//-1 before any row has the keyboard, which starts the search at the top
	let start = typing ? Math.max(at, 0) : at + 1//a new name starts after the row the keyboard is on, so the same letter again moves on; a name still being typed starts on it, so another letter stays put while that row matches
	return [...shown.slice(start), ...shown.slice(0, start)].find(i => o.lower[i].startsWith(o.typed)) ?? -1
}

function _listShow(o) {
	let shown = []
	for (let i of o.order) {//in the order the rows are on screen, sorted or as given
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
	let toggle = isCommandOrControl(e)
	let row = e.target.closest('[data-row]')
	let before = o.traits
	if (!row) {
		if (e.offsetX >= o.root.clientWidth || e.offsetY >= o.root.clientHeight) return//a scrollbar, down the side or across the bottom, which a press on clears nothing
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
	if (!e.shiftKey && !e.target.closest('.myName') && o.shown.length > 0) _listSmear(o, e, row ? Number(row.dataset.row) : -1, toggle ? before : selectionEmpty())//a press on a name is left for dragging the file, once the list drags files
}

/*
A press anywhere in a row but on its name, dragged up or down, selects the rows from the one pressed to the one level with the pointer, as Finder's list does from beside a name. A press below the last row does the same from the bottom up, with the last row as the start, and nothing is drawn for the drag itself. With the toggle key down at the press, the rows join the selection the list had before it, which is how Finder adds a second run. The pointer is held to the rows in view, so a drag past the list's edge reaches the last row showing rather than rows scrolled out of sight. Until the pointer reaches a row other than the one it started on, the press's own selection stands, so a toggle click that drifts a pixel is still a toggle click.
*/
function _listSmear(o, e, from, base) {//from is the row pressed, or -1 below the last row; base is what the drag adds to
	let height = o.elements[o.shown[0]].offsetHeight//every row is one line of the same height
	let last = o.shown.length - 1
	let start = from == -1 ? last + 1 : o.shown.indexOf(from)//places in shown, where last + 1 is below the rows
	let at = start, changed = 0, ms = 0, pointerY = e.clientY//where the pointer was last heard, which a scroll under a still pointer measures again
	o.root.setPointerCapture(e.pointerId)//so the drag keeps the pointer outside the list and outside the window
	let step = () => {//after the pointer moves or the list scrolls under it
		let box = o.root.getBoundingClientRect()
		let y = Math.min(Math.max(pointerY, box.top), box.top + o.root.clientHeight - 1)
		let k = Math.min(Math.floor((y - box.top + o.root.scrollTop) / height), last + 1)
		if (k == at) return
		at = k
		let t = performance.now(), before = o.traits
		if (start > last && k > last) o.traits = base//begun below the rows and still there
		else o.traits = selectionSmear(base, o.shown[Math.min(start, last)], o.shown[Math.min(k, last)], o.shown)
		changed += _listDraw(o, before)
		ms += performance.now() - t
	}
	let move = m => { pointerY = m.clientY; step() }
	let end = () => {
		o.root.removeEventListener('pointermove', move); o.root.removeEventListener('scroll', step); o.root.removeEventListener('pointerup', end); o.root.removeEventListener('pointercancel', end)
		if (at != start) o.onSelect({selected: o.traits.selected.size, changed, kind: 'smear', listMs: ms})
	}
	o.root.addEventListener('pointermove', move)
	o.root.addEventListener('scroll', step, {passive: true})
	o.root.addEventListener('pointerup', end)
	o.root.addEventListener('pointercancel', end)
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
