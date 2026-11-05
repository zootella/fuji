import {isCommandOrControl} from './keyboard.js'

/*
Fuji's own tree, drawn with the page's own elements as list.js draws the list, and written beside it rather than out of it. The two are built the same way and share none of their code, since the tree selects one folder and highlights only its name where the list selects many rows across their whole width, and a row that could be either would carry the other's features everywhere.

The tree is flat. Every folder is one row element in one container, made once, in depth-first order, so the folders inside a folder are the run of rows right after it, and closing a folder hides that run rather than removing anything. Which rows show is a pass over every row in _treeShow, the way the list's filter is: a row shows when every folder above it is open. Each row draws the lines down its left for itself, a cell for each level of depth, worked out from the data as the row is made: a line where a folder above still has a folder after it, and at the row's own level a tee, or an elbow for a folder's last child, with the twistie on the line for a folder that has folders inside.

App code makes the tree and keeps it, as it does a list: treeMake answers a plain object, the app calls the functions here on it, and the tree reports what the user did through the callbacks it was made with. Each folder is the app's own record, kept and handed back rather than copied, and each level is put in order by the comparator the app gives.
*/

export function treeMake({identity, compare, onSelect = () => {}, onOpen = () => {}, onClose = () => {}, onTrouble}) {//a tree of its own element, which scrolls both ways. identity names the field in each folder that tells it from every other, compare puts each level in order, onSelect hears the folder the user selected, onOpen and onClose each folder opened or closed, each of these optional, and onTrouble anything that throws where the page hands the tree an event
	if (typeof identity != 'string' || !identity) throw new Error(`expected the name of the field that identifies a folder: ${identity}`)
	if (typeof compare != 'function') throw new Error(`expected a comparator to put each level in order: ${compare}`)
	let element = document.createElement('div')
	element.className = 'myTree'
	element.tabIndex = 0//so the tree can take the keyboard
	let o = {element, identity, compare, onSelect, onOpen, onClose, rows: [], open: new Set(), shown: [], selected: -1}
	let gate = handle => e => { try { handle(o, e) } catch (error) { onTrouble(error) } }
	element.addEventListener('pointerdown', gate(_treePress))
	element.addEventListener('dblclick', gate(_treeDoubleClick))
	element.addEventListener('contextmenu', e => e.preventDefault())//the web view's own menu turned away in every build, as the list turns it away, so a built fuji and a development one behave alike
	element.addEventListener('mousedown', e => e.preventDefault())//the press's default cancelled, for the reason list.js gives: the engine's search for where a text selection would end walks every row
	element.addEventListener('focus', () => { element.dataset.active = 'true' })//an attribute for the stylesheet rather than :focus, for the reason list.js gives
	element.addEventListener('blur', () => { delete element.dataset.active })
	return o
}

export function treeRoots(o, folders) {//replace every folder, the roots and all they hold; a folder is a record with a name, an icon, its identity, and children, a list of the folders inside it, each a record like this one
	o.rows = []
	o.open = new Set()
	o.selected = -1
	_treeAdd(o, folders, -1, [], new Set())
	let fragment = document.createDocumentFragment()
	for (let row of o.rows) fragment.append(row.element)
	o.element.replaceChildren(fragment)
	_treeShow(o)
}

function _treeAdd(o, folders, parent, lines, seen) {//these folders under the row parent, or at the top for -1, in the order compare puts them, each followed by everything it holds; lines says, for each level above, whether that level's line runs on below
	let sorted = [...folders].sort(o.compare)
	for (let [n, node] of sorted.entries()) {
		let id = node[o.identity]
		if (typeof id != 'string' || !id || seen.has(id)) throw new Error(`expected a unique identity in ${o.identity}: ${id}`)
		seen.add(id)
		let held = node.children && node.children.length > 0
		let above = parent != -1 || n > 0, below = n < sorted.length - 1//whether the line at this folder's level comes down to it, from the folder it is in or a root before it, and whether it runs on to another folder after it
		let i = o.rows.length
		o.rows.push(_treeRow({node, i, parent, lines, above, below, opens: held}))
		if (parent != -1) o.rows[parent].kids.push(i)
		if (held) _treeAdd(o, node.children, i, [...lines, below], seen)
	}
}

function _treeRow({node, i, parent, lines, above, below, opens}) {//one folder's row, i in rows: a guide cell for each level above, its own cell with a twistie on its line when it opens, and the icon and name a click selects
	let element = document.createElement('div')
	element.className = 'myTreeRow'
	element.dataset.row = i//how a press finds its row
	for (let runs of lines) {
		let cell = document.createElement('span')
		cell.className = 'myGuide'
		if (runs) cell.dataset.guide = 'line'//a folder above has another after it, so its line runs past this row
		element.append(cell)
	}
	let own = document.createElement('span')
	own.className = 'myGuide'
	let guide = 'alone'//the line at the row's own level: up to it and on down, a tee; up only, an elbow, for a last folder; down only, for the first of several roots; neither, for a root by itself
	if (above && below) guide = 'tee'
	else if (above) guide = 'elbow'
	else if (below) guide = 'first'
	own.dataset.guide = guide
	let twistie = false
	if (opens) {
		twistie = document.createElement('span')
		twistie.className = 'myTwistie'
		own.className = 'myGuide myTwistieCell'//the whole square is what a press opens or closes, rather than only the box drawn in it
		own.append(twistie)
	}
	let item = document.createElement('span'), icon = document.createElement('span'), label = document.createElement('span')
	item.className = 'myTreeItem'
	icon.className = 'myTreeIcon'
	label.className = 'myTreeLabel'
	icon.textContent = node.icon//a letter the app chooses, standing in for an icon
	label.textContent = node.name//a folder's name is text nobody vouched for, so it goes in as text and is never parsed, as list.js explains
	item.append(icon, label)
	element.append(own, item)
	return {node, parent, element, item, twistie, kids: []}
}

function _treeShow(o) {//every row shows whose folders above are all open, and the rest hide, written only where the answer changed; one pass in order is enough, since a folder's row always comes before the rows of what it holds
	let shown = [], showing = []
	for (let [i, row] of o.rows.entries()) {
		showing[i] = row.parent == -1 || (showing[row.parent] && o.open.has(row.parent))
		if (row.element.hidden == showing[i]) row.element.hidden = !showing[i]
		if (showing[i]) shown.push(i)
	}
	o.shown = shown//the rows on screen, in order, which the keys move along
}

function _treeSelect(o, i) {//the folder in row i, alone, kept in view
	if (i == o.selected) return
	if (o.selected != -1) delete o.rows[o.selected].item.dataset.selected
	o.selected = i
	o.rows[i].item.dataset.selected = 'true'

	//up or down just enough to show the whole row, and never sideways, so the names stay put as the selection moves; scrollIntoView would scroll sideways too, to a long name's start
	let row = o.rows[i].element.getBoundingClientRect(), top = o.element.getBoundingClientRect().top, bottom = top + o.element.clientHeight
	if (row.top < top) o.element.scrollTop -= top - row.top
	else if (row.bottom > bottom) o.element.scrollTop += row.bottom - bottom

	o.onSelect(o.rows[i].node)
}

function _treeOpen(o, i) {
	if (o.open.has(i)) return
	o.open.add(i)
	o.rows[i].twistie.dataset.open = 'true'
	_treeShow(o)
	o.onOpen(o.rows[i].node)
}
function _treeClose(o, i) {
	if (!o.open.has(i)) return
	o.open.delete(i)
	delete o.rows[i].twistie.dataset.open
	if (_treeHolds(o, i, o.selected)) _treeSelect(o, i)//the selection was inside, so it comes up to the folder closing, as the classic Windows tree does, from memory
	_treeShow(o)
	o.onClose(o.rows[i].node)
}
function _treeToggle(o, i) { if (o.open.has(i)) _treeClose(o, i); else _treeOpen(o, i) }
function _treeHolds(o, folder, i) {//whether row i is somewhere inside the folder in row folder
	if (i == -1) return false
	for (let p = o.rows[i].parent; p != -1; p = o.rows[p].parent) if (p == folder) return true
	return false
}

function _treePress(o, e) {//on the press, as the list selects: a twistie opens or closes its folder and leaves the selection alone, the icon or the name selects, and anywhere else only takes the keyboard, as the classic Windows tree does, from memory
	if (e.button != 0) return
	if ((e.ctrlKey || e.metaKey) && !isCommandOrControl(e)) return//control on the mac, where a control-click is a right click
	o.element.focus({preventScroll: true})//the keyboard to the tree, which the press's cancelled default would otherwise have done
	let row = e.target.closest('[data-row]')
	if (!row) return
	let i = Number(row.dataset.row)
	if (e.target.closest('.myTwistieCell')) _treeToggle(o, i)
	else if (e.target.closest('.myTreeItem')) _treeSelect(o, i)
}
function _treeDoubleClick(o, e) {//a double-click on a folder's icon or name opens or closes it, as a click on its twistie does, as the classic Windows tree does, from memory
	let item = e.target.closest('.myTreeItem')
	if (!item) return
	let i = Number(item.closest('[data-row]').dataset.row)
	if (o.rows[i].twistie) _treeToggle(o, i)
}

const treeKeys = ['ArrowDown', 'ArrowUp', 'ArrowRight', 'ArrowLeft', 'Home', 'End', 'PageDown', 'PageUp']
export function treeKey(o, e) {//one key, acted on only while the tree has the keyboard, since every key in the view reaches here; true when the tree used it. The arrows move the selection itself, since the tree selects one folder and has no keyboard focus apart from it, as Explorer's folder pane does
	if (document.activeElement != o.element) return false
	if (e.altKey || e.ctrlKey || e.metaKey || !treeKeys.includes(e.key)) return false//a chord, or a key the tree has no use for, which belongs to the page around it or to the system
	e.preventDefault()//or the focused tree would also scroll itself by the key
	let shown = o.shown, i = o.selected
	if (!shown.length) return true
	if (e.key == 'ArrowRight') {//into the outline: a closed folder opens, and an open one gives the selection to the first folder inside
		if (i != -1 && o.rows[i].twistie) {
			if (!o.open.has(i)) _treeOpen(o, i)
			else if (o.rows[i].kids.length > 0) _treeSelect(o, o.rows[i].kids[0])
		}
	} else if (e.key == 'ArrowLeft') {//out of it: an open folder closes, and any other gives the selection to the folder it is in
		if (i != -1) {
			if (o.open.has(i)) _treeClose(o, i)
			else if (o.rows[i].parent != -1) _treeSelect(o, o.rows[i].parent)
		}
	} else {
		let at = shown.indexOf(i)//-1 before any folder is selected, when a key starts from the end it points away from
		let page = Math.max(1, Math.floor(o.element.clientHeight / o.rows[shown[0]].element.offsetHeight) - 1)//a screen of rows less one, so the row the page key left stays in view
		let k
		if      (e.key == 'ArrowDown') k = at == -1 ? 0 : at + 1
		else if (e.key == 'ArrowUp')   k = at == -1 ? shown.length - 1 : at - 1
		else if (e.key == 'PageDown')  k = at == -1 ? 0 : at + page
		else if (e.key == 'PageUp')    k = at == -1 ? shown.length - 1 : at - page
		else if (e.key == 'Home')      k = 0
		else                           k = shown.length - 1//End
		_treeSelect(o, shown[Math.min(Math.max(k, 0), shown.length - 1)])
	}
	return true
}
