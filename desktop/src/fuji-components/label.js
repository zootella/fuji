/*
A label's text into its box, for the labels fuji draws, like a cell of its list, a name in its tree and a line of a thumbnail's caption, which all hand their text here so it means the same in each and goes in one way.

A label is a file's name or something said about the file, and a file's name is text nobody vouched for. On the Mac and on Linux a name can hold angle brackets and quotes, and be written to look like markup that loads a picture and runs a script when the picture fails; a torrent's list of files carries whatever its maker typed. Assigned to innerHTML, the engine would parse that name into elements and run it. Assigned to textContent, the DOM replaces the element's children with a single text node holding the string exactly as it is, and nothing in it is ever parsed. That is how the rest of fuji is safe without saying so: Vue's double curly braces reach the page through the same doors, its runtime making each interpolation a node with createTextNode, updating it through nodeValue, and setting an element's whole text with textContent. OWASP's guide to preventing DOM-based cross-site scripting names textContent the safe assignment property. So nothing here writes markup from a string, and the list and the tree build every other part of a row with createElement. Text that mixes in small capitals is the same: each plain run becomes a text node, which is never parsed either, and each small run a span of fuji's own making, whose text goes to textContent and whose class names come from here, never from the run.
*/

export function labelPut(element, said) {//said is a string, put in as it is, or a list of runs for text that mixes in small capitals: each run a string, or {small} holding capitals to draw small, with fixed true to draw them in the fixed-width face as well and pad true for a blank letter's room after them
	if (typeof said == 'string') { element.textContent = said; return }
	element.replaceChildren(...said.map(run => {
		if (typeof run == 'string') return run//replaceChildren makes a string a text node, never markup
		let small = document.createElement('span')
		small.className = 'mySmallCaps'
		if (run.fixed) small.classList.add('myFixedWidth')//the run only says yes or no
		if (run.pad) small.classList.add('myPadCell')
		small.textContent = run.small
		return small
	}))
}
