//a list's three traits, after VS Code's list: the rows selected, the anchor a shift range starts from, and the focus where the keyboard is; each change returns a new value rather than editing the old, so a view watching one value sees every change

export function selectionEmpty() { return {selected: new Set(), anchor: -1, focus: -1} }

export function selectionClick(traits, row, shown, {range, toggle}) {//one click on a row, by the rule Finder and Explorer share: alone it replaces the selection, with the toggle key it flips the row, and with shift it selects from the anchor to the row, keeping the rest only when the toggle key is down too. Row is an index into the list's rows, and shown is the order they are on screen, which a range runs along
	if (range && traits.anchor != -1) {
		let from = shown.indexOf(traits.anchor), to = shown.indexOf(row)
		if (from != -1) {//an anchor the filter has hidden starts no range, so the click falls through to the rules below
			let selected = toggle ? new Set(traits.selected) : new Set()
			for (let k = Math.min(from, to); k <= Math.max(from, to); k++) selected.add(shown[k])
			return {selected, anchor: traits.anchor, focus: row}
		}
	}
	if (toggle) {
		let selected = new Set(traits.selected)
		if (selected.has(row)) selected.delete(row); else selected.add(row)
		return {selected, anchor: row, focus: row}
	}
	return {selected: new Set([row]), anchor: row, focus: row}
}
