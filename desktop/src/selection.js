//a list's traits, after VS Code's list: the rows selected, the anchor a shift range starts from, and the focus where the keyboard is; each change returns a new value rather than editing the old, so the list can compare the two and touch only the rows that differ

export function selectionEmpty() { return {selected: new Set(), anchor: -1, focus: -1} }

export function selectionClick(traits, row, shown, {range, toggle}) {//one click on a row, by the rule Finder and Explorer share: alone it replaces the selection, with the toggle key it flips the row, and with shift it selects from the anchor to the row, keeping the rest only when the toggle key is down too. Row is an index into the list's rows, and shown is the order they are on screen, which a range runs along
	if (range && shown.includes(traits.anchor)) {//an anchor the filter has hidden, or none yet, starts no range, so the click falls through to the rules below
		let selected = toggle ? new Set(traits.selected) : new Set()
		for (let r of _selectionRange(shown, traits.anchor, row)) selected.add(r)
		return {selected, anchor: traits.anchor, focus: row}
	}
	if (toggle) {
		let selected = new Set(traits.selected)
		if (selected.has(row)) selected.delete(row); else selected.add(row)
		return {selected, anchor: row, focus: row}
	}
	return {selected: new Set([row]), anchor: row, focus: row}
}

export function selectionMove(traits, row, shown, {range}) {//the keyboard moving to a row, as an arrow does in Finder and Explorer: alone it selects just that row, and with shift it selects from the anchor to it
	if (range && shown.includes(traits.anchor)) return {selected: new Set(_selectionRange(shown, traits.anchor, row)), anchor: traits.anchor, focus: row}
	return {selected: new Set([row]), anchor: row, focus: row}
}

export function selectionFocus(traits, row) {//the keyboard moving to a row and leaving the selection and the anchor as they were, as control and an arrow do in Explorer
	return {selected: traits.selected, anchor: traits.anchor, focus: row}
}

export function selectionSmear(base, from, to, shown) {//a drag across the rows from one to the other, added to the rows selected in base
	let selected = new Set(base.selected)
	for (let r of _selectionRange(shown, from, to)) selected.add(r)
	return {selected, anchor: from, focus: to}
}

export function selectionAll(traits, shown) {//every row on screen, with the focus left where it was
	return {selected: new Set(shown), anchor: shown.length ? shown[0] : -1, focus: traits.focus}
}

function _selectionRange(shown, from, to) {//the rows from one to the other in the order they are on screen, both included
	let a = shown.indexOf(from), b = shown.indexOf(to)
	return shown.slice(Math.min(a, b), Math.max(a, b) + 1)
}
