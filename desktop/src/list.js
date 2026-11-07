import {Time} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in
import {isCommandOrControl} from './keyboard.js'
import {selectionEmpty, selectionClick, selectionMove, selectionFocus, selectionSmear, selectionAll} from './selection.js'
import {labelPut} from './label.js'

/*
Fuji's own list, drawn with the page's own elements rather than through Vue. A list is many rows whose state changes a little at a time, a click marking one row or a keystroke in the filter hiding a few, and Vue answers every change by running its template over all of them to find the few that differ. Here each row is one element, made once and kept for the life of the list, and a change touches only the rows it changes: the filter sets hidden on the rows whose answer flipped, and a press or a key marks the rows whose selection or focus did. Every row is in the page, with no windowing and no virtualizer: this plain list built, scrolled and filtered ten thousand rows without trouble on the Mac mini and the Windows box, scrolls on the engine's compositor with no script running, and leaves scrolling a row into view to the browser. FujiUserInterfaceSpike.vue is the bench that times it.

App code makes the list and keeps it, rather than a component making it and the app reaching in through Vue. listMake answers a plain object, the app calls the functions here on it, and the list reports what the user did through the callbacks it was made with, asking the app for nothing. So the list lives as long as the app wants it, a plain module can hold one and fill it with no framework in between, and FujiList.vue does nothing but put its element on screen. Each row carries its identity in a field the app names, a path for a folder on disk or a tag for something with no path, so a record that arrives later as a fresh object, from Rust or from another process, can name the row it describes.
*/

export function listMake({identity, columns, onSelect, onSort, onMenu, onTrouble}) {//a list of its own element, a row of headers and below it the rows, which scroll. identity names the field in each row that tells it from every other. columns says what each row shows, left to right: each has a key into the row, the title its header says, a say function that makes its value text, or runs that mix in small capitals as label.js takes them, a compare function that says which of two rows comes first, which a column of raw numbers leaves out, an align, tabular to draw every digit one width so the digits line up down the column, and a width in css pixels. onSelect hears what each press, key or drag did, with how many rows it left selected, how many changed, its kind and the list's own milliseconds; onSort each sort, by which column and which way; onMenu where in the window a right click asks for a menu; and onTrouble anything that throws where the page hands the list an event
	if (typeof identity != 'string' || !identity) throw new Error(`expected the name of the field that identifies a row: ${identity}`)
	let element = document.createElement('div'), header = document.createElement('div'), root = document.createElement('div')
	element.className = 'myListFrame'
	header.className = 'myHeader'
	root.className = 'myList'
	root.tabIndex = 0//so the list can take the keyboard
	element.append(header, root)
	for (let c of columns) if (typeof c.width != 'number' || c.width <= 0) throw new Error(`expected a width in css pixels for column ${c.key}: ${c.width}`)
	let o = {element, header, root, identity, columns, onSelect, onSort, onMenu, widths: columns.map(c => c.width), rows: [], lower: [], elements: [], textWidths: [], beside: [], cutLines: [], order: [], shown: [], at: [], filter: '', sort: {column: 0, descending: false}, traits: selectionEmpty(), typed: '', typedAt: 0}//sorted by the first column from the start, the name, as Finder and Explorer open a folder
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
	element.addEventListener('contextmenu', gate(_listMenu))
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
	element.addEventListener('mousedown', e => e.preventDefault())
	return o
}

export function listRows(o, rows) {//replace every row, as for a new folder; a row is a record holding a value for each column's key, and its identity
	let seen = new Set()
	for (let row of rows) {
		let id = row[o.identity]
		if (typeof id != 'string' || !id || seen.has(id)) throw new Error(`expected a unique identity in ${o.identity}: ${id}`)
		seen.add(id)
	}
	o.rows = rows
	let first = o.columns[0].key
	o.lower = rows.map(row => String(row[first]).toLowerCase())//the first column, the name, is what the filter matches, as Finder's search does; lowered once per list rather than once per keystroke
	o.elements = rows.map((row, i) => {
		let e = document.createElement('div')
		e.className = 'myRow'
		e.dataset.row = i//how a press finds its row
		for (let [k, column] of o.columns.entries()) {
			let cell = document.createElement('span')
			cell.className = 'myListCell'
			if (column.align) cell.dataset.align = column.align
			if (column.tabular) cell.dataset.tabular = 'true'
			if (k == 0) {//the icon beside the name, as Finder and Explorer draw one: a placeholder square for now, showing where the icons will go
				let icon = document.createElement('span')
				icon.className = 'myListIcon'
				cell.append(icon)
			}
			let text = document.createElement('span')//the text in a box of its own, beside the icon in the first column
			labelPut(text, _listSay(column, row))//a file's name is text nobody vouched for, which label.js puts in without ever parsing it
			cell.append(text)
			e.append(cell)
		}
		return e
	})
	o.root.replaceChildren()
	o.order = rows.map((row, i) => i)
	o.order = _listSorted(o)//in the sort the list already has, so new rows keep the one the user chose for the old ones
	_listPlace(o)
	_listMeasure(o)//before the filter hides any row, since a hidden row lays out zero wide
	o.traits = selectionEmpty()
	_listShow(o)
}
function _listMeasure(o) {//each cell's text width, read from the layout once as the rows are made, so telling a cut cell is arithmetic from then on: a span in the cell's flex row never shrinks below its text, so its width is the whole text as drawn, small capitals and tabular digits included, even where the cell cuts it off. Read on screen, so a list fills its rows once it is placed, and a change of faces, which changes every width, needs the rows made again
	o.textWidths = o.elements.map(e => o.columns.map((c, k) => e.children[k].lastChild.getBoundingClientRect().width))
	o.beside = o.columns.map((c, k) => _listBeside(o, k))
	o.cutLines = o.elements.map(() => o.columns.map(() => false))//each cell's line, or false where its text fits
	_listCutLines(o, o.columns.map((c, k) => k))
}
function _listCutLines(o, columns) {//the line at the edge of each cell in these columns whose text is wider than its column leaves room for, drawn or taken away only where the answer changed, so a drag on a divider touches only the cells it cuts or uncovers
	for (let [i, e] of o.elements.entries()) for (let k of columns) {
		let cut = o.textWidths[i][k] > o.widths[k] - o.beside[k] + 0.01//the hundredth so a text that exactly fits, measured in fractions, is never called cut
		if (cut == Boolean(o.cutLines[i][k])) continue
		if (!cut) { o.cutLines[i][k].remove(); o.cutLines[i][k] = false; continue }
		let line = document.createElement('span')//beside the cells rather than in one, since a cell clips what is inside it to its own box, and the line reaches past the row
		line.className = 'myCutLine'
		line.style.gridColumn = `${k + 1} / ${k + 2}`//placed against its column's area of the row's grid, without taking a place in the grid; both lines named, since a positioned child's missing end line is the grid's padding edge rather than the next line
		if (o.columns[k].align) line.dataset.align = o.columns[k].align
		e.append(line)
		o.cutLines[i][k] = line
	}
}
function _listBeside(o, column) {//the room a column's cells keep beside their text
	let style = getComputedStyle(o.header.children[column])
	let beside = parseFloat(style.paddingLeft) + parseFloat(style.paddingRight)//the cell's padding at either end, read from the header's cell, whose padding the rows' cells share
	let icon = o.elements[0]?.children[column].querySelector('.myListIcon')//and in a column that shows an icon, the icon and the gap after it, read from a row
	if (icon) beside += parseFloat(getComputedStyle(icon).width) + parseFloat(getComputedStyle(icon.parentElement).columnGap)//from the style rather than the layout, since the filter may have hidden that row, and a hidden row's icon lays out zero wide
	return beside
}

function _listSay(column, row) { return column.say ? column.say(row[column.key]) : String(row[column.key]) }//the text a cell shows for a row's value

const listNarrowest = 24//css pixels, the narrowest a drag leaves a column, which still leaves its divider to grab
function _listGrid(o) { o.element.style.setProperty('--list-columns', o.widths.map(w => `${w}px`).join(' ')) }//the widths onto the one grid the headers and rows share

function _listGripPress(o, e) {//a press on a divider resizes the column to its left as the pointer moves; the columns to its right move over, and past the list's edge the list scrolls sideways
	let grip = e.target.closest('.myGrip')
	if (!grip || e.button != 0) return
	let column = Number(grip.parentElement.dataset.column), startX = e.clientX, startWidth = o.widths[column]
	grip.setPointerCapture(e.pointerId)//so the drag keeps the pointer outside the header and outside the window
	let move = m => { o.widths[column] = Math.max(listNarrowest, Math.round(startWidth + m.clientX - startX)); _listGrid(o); _listCutLines(o, [column]) }
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
	measure.font = `${style.fontStyle} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`//the list's own font, so the header is measured without drawing it
	let widest = measure.measureText(`${c.title} ▲ `).width//room for the header and the sort arrow beside it
	for (let row of o.textWidths) widest = Math.max(widest, row[column])//and every row's text, as the rows measured it when they were made, filtered or not
	o.widths[column] = Math.ceil(widest + o.beside[column]) + 2//and a pixel each side so the last letter is never cut
	_listGrid(o)
	_listCutLines(o, [column])
}

function _listHeaderClick(o, e) {//a click on a header sorts by its column, and another on the same header reverses it, as Explorer and Finder do
	if (e.target.closest('.myGrip')) return//a press on a divider resizes, and the click it ends with sorts nothing
	let cell = e.target.closest('[data-column]')
	if (!cell) return
	let start = performance.now()
	_listPointerFocus(o)
	let column = Number(cell.dataset.column)
	o.sort = {column, descending: o.sort.column == column ? !o.sort.descending : false}
	o.order = _listSorted(o)
	_listPlace(o)
	_listShow(o)
	_listJoin(o, o.traits.selected)//the selected rows have new neighbors in the new order
	for (let h of o.header.children) delete h.dataset.sort
	cell.dataset.sort = o.sort.descending ? 'down' : 'up'//the arrow the stylesheet draws beside the sorted header
	if (o.traits.focus != -1) o.elements[o.traits.focus].scrollIntoView({block: 'nearest'})//the row the keyboard is on stays in view wherever the sort took it
	o.onSort({key: o.columns[column].key, descending: o.sort.descending, listMs: performance.now() - start})
}

function _listSorted(o) {//the rows' order sorted by the chosen column, from the order they are in now; a stable sort leaves rows that tie in the order they had, so the column sorted before becomes the second key, a spreadsheet's way and what Explorer documents
	let {key, compare} = o.columns[o.sort.column]
	if (!compare) compare = (a, b) => _listCompare(a[key], b[key])//a column with no comparator of its own sorts by its raw values, numbers like a size in bytes or a date in milliseconds
	let order = [...o.order].sort((i, j) => compare(o.rows[i], o.rows[j]))
	return o.sort.descending ? order.reverse() : order//reversed rather than compared backward, so rows that tie turn over with the rest
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

export function listSelected(o) { return o.shown.filter(i => o.traits.selected.has(i)).map(i => o.rows[i]) }//the rows selected, in the order they are on screen, made when the app asks rather than on every change

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
	o.root.dataset.keyboard = 'true'//the focus mark shows from the first key on, as Windows shows its focus rectangle only once the keyboard has been used, and keeps showing it through clicks inside the list, until a press brings the keyboard back into the list from somewhere else
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
	let shown = [], at = []
	for (let i of o.order) {//in the order the rows are on screen, sorted or as given
		let match = !o.filter || o.lower[i].includes(o.filter)
		if (o.elements[i].hidden == match) o.elements[i].hidden = !match//written only where the answer changed
		at[i] = match ? shown.length : -1
		if (match) shown.push(i)
	}
	o.shown = shown//the rows on screen, in order, which a shift range runs along
	o.at = at//and each row's place among them, or -1 for a row the filter hides, so a row finds its neighbors on screen without a search
}

function _listPointerFocus(o) {//the keyboard to the list for a press, which the press's cancelled default would otherwise have done; a press that brings the keyboard in from somewhere else, like the tree, starts it without the focus mark, which the next key the list uses shows again, as the browser's own focus-visible does, so a key pressed once by accident does not leave the mark for the life of the list. Switching windows and back brings no press, so a keyboard user keeps the mark through it
	if (document.activeElement != o.root) delete o.root.dataset.keyboard
	o.root.focus({preventScroll: true})
}

function _listPress(o, e) {//on the press rather than the click, as Finder and Explorer select
	if (e.button != 0) return
	if ((e.ctrlKey || e.metaKey) && !isCommandOrControl(e)) return//control on the mac, where a control-click is a right click, which _listMenu answers
	let start = performance.now()
	_listPointerFocus(o)
	let toggle = isCommandOrControl(e)
	let row = e.target.closest('[data-row]')
	if (row && !toggle && !e.shiftKey && o.traits.selected.has(Number(row.dataset.row))) return _listCarry(o, e, Number(row.dataset.row))//a plain press on a row already selected keeps the selection, which a drag from here carries
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
	if (!e.shiftKey && o.shown.length > 0) _listSmear(o, e, row ? Number(row.dataset.row) : -1, toggle ? before : selectionEmpty())
}

/*
A press on a row that is not selected, anywhere in it, dragged up or down, selects the rows from the one pressed to the one level with the pointer, as Finder's list does from beside a name. A press below the last row does the same from the bottom up, with the last row as the start, and nothing is drawn for the drag itself. With the toggle key down at the press, the rows join the selection the list had before it, which is how Finder adds a second run. The pointer is held to the rows in view, so a drag past the list's edge reaches the last row showing rather than rows scrolled out of sight. Until the pointer reaches a row other than the one it started on, the press's own selection stands, so a toggle click that drifts a pixel is still a toggle click.

A press on a row that is selected carries the selection instead, so a file is dragged by selecting it first. Finder and Explorer drag a file pressed on its name, but where a name ends cannot be seen, and that would make the place to press grow and shrink with the length of each name; here every part of a row does the same thing.
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

function _listMenu(o, e) {//a right click anywhere in the list turns away the web view's own menu, which the list does itself so it is right in any page that places it, and asks for a menu at the pointer that acts on what is highlighted, as Explorer's does: a row not yet selected is selected alone first, and a click below the rows clears the selection, as a press there does, so a menu with nothing selected is the one for the folder itself
	e.preventDefault()
	if (!o.root.contains(e.target)) return//the headers, which have no menu
	let row = e.target.closest('[data-row]')
	if (!row && (e.offsetX >= o.root.clientWidth || e.offsetY >= o.root.clientHeight)) return//a scrollbar
	_listPointerFocus(o)
	let start = performance.now(), before = o.traits
	if (!row) o.traits = selectionEmpty()
	else if (!o.traits.selected.has(Number(row.dataset.row))) o.traits = selectionClick(o.traits, Number(row.dataset.row), o.shown, {range: false, toggle: false})
	let changed = _listDraw(o, before)
	if (o.traits != before) o.onSelect({selected: o.traits.selected.size, changed, kind: row ? 'right click' : 'right click below the rows', listMs: performance.now() - start})
	o.onMenu({x: e.clientX, y: e.clientY})
}

/*
A plain press on a row that is already selected leaves the selection as it is, because the user may be about to drag it somewhere, as Finder and Explorer both do. Once the pointer moves a few pixels from the press, the press is a drag carrying the selection, and the pointer changes over the whole window to say so. Released without moving, it was a click after all, and the selection becomes that one row, on the release rather than the press. Nothing takes a drop yet; the pointer is the whole of it.
*/
const listDragSlop = 2//css pixels the pointer moves from a press, either way, before the press is a drag: half the 4-pixel square Windows' DragDetect centers on the press, SM_CXDRAG and SM_CYDRAG at the factory, from memory
function _listCarry(o, e, from) {//from is the row pressed, which is selected
	let before = o.traits
	o.traits = selectionFocus(o.traits, from)//the keyboard to the row pressed, with the selection left alone
	_listDraw(o, before)
	let startX = e.clientX, startY = e.clientY, carrying = false
	o.root.setPointerCapture(e.pointerId)//so the drag keeps the pointer outside the list and outside the window
	let move = m => {
		if (carrying || (Math.abs(m.clientX - startX) <= listDragSlop && Math.abs(m.clientY - startY) <= listDragSlop)) return
		carrying = true
		document.documentElement.dataset.carry = 'true'//on the root rather than the list, so the rule in FujiList.vue reaches every element the pointer crosses
	}
	let end = u => {
		o.root.removeEventListener('pointermove', move); o.root.removeEventListener('pointerup', end); o.root.removeEventListener('pointercancel', end)
		delete document.documentElement.dataset.carry
		let start = performance.now(), before = o.traits
		if (!carrying && u.type == 'pointerup') o.traits = selectionClick(o.traits, from, o.shown, {range: false, toggle: false})//a click after all
		let changed = _listDraw(o, before)
		o.onSelect({selected: o.traits.selected.size, changed, kind: carrying ? 'carry' : 'plain', listMs: performance.now() - start})
	}
	o.root.addEventListener('pointermove', move)
	o.root.addEventListener('pointerup', end)
	o.root.addEventListener('pointercancel', end)
}

/*
The selection drawn by difference: the rows that left it lose their mark, the rows that joined it gain one, the focus mark moves from the row that had it to the row that has it, and only the rows beside those are looked at again, for whether a run of selected rows still joins them. This is why selection.js returns a new value on every change instead of changing the one it was given, because the list needs the old selection and the new one side by side to compare them. A click then costs the one or two rows it changes, however long the list, a shift range costs the rows in it, and clearing costs the rows that were selected. Drawing a selection the other way, by visiting every row to ask whether it is selected, costs the whole list every time: that was the Vue version, which reran its template over every row and lagged at ten thousand. With 1,944 of 5,000 rows selected, a command-click marked its one row in well under a millisecond of the list's own time, on the Mac mini.
*/
function _listDraw(o, before) {//the marks onto the rows whose selection or focus changed since the traits before, and how many rows changed selection
	let after = o.traits, changed = []
	for (let i of before.selected) if (!after.selected.has(i)) { delete o.elements[i].dataset.selected; changed.push(i) }
	for (let i of after.selected) if (!before.selected.has(i)) { o.elements[i].dataset.selected = 'true'; changed.push(i) }
	if (before.focus != after.focus) {
		if (before.focus != -1) delete o.elements[before.focus].dataset.focus
		if (after.focus != -1) o.elements[after.focus].dataset.focus = 'true'
	}
	let near = new Set()//the rows that changed and the rows beside them on screen, since a change can move no other row's joins
	for (let i of changed) {
		near.add(i)
		let p = o.at[i]
		if (p > 0) near.add(o.shown[p - 1])
		if (p != -1 && p < o.shown.length - 1) near.add(o.shown[p + 1])
	}
	_listJoin(o, near)
	return changed.length
}

function _listJoin(o, rows) {//mark each of these rows joined to the selected row above or below it on screen, so a run of selected rows draws as one rounded block, its corners rounded only at the run's top and bottom; a row not selected joins nothing
	let s = o.traits.selected
	for (let i of rows) {
		let e = o.elements[i], p = o.at[i]
		let up = s.has(i) && p > 0 && s.has(o.shown[p - 1])
		let down = s.has(i) && p != -1 && p < o.shown.length - 1 && s.has(o.shown[p + 1])
		if (up != ('joinUp' in e.dataset)) { if (up) e.dataset.joinUp = 'true'; else delete e.dataset.joinUp }
		if (down != ('joinDown' in e.dataset)) { if (down) e.dataset.joinDown = 'true'; else delete e.dataset.joinDown }
	}
}
