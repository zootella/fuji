//the modifier a component's chords are held with, like a toggle click or select all, which is control unless the page that places the components says otherwise once as it starts; the components ask isCommandOrControl, named after Tauri's CmdOrCtrl, and never which platform they are on

let commandKey = 'ctrlKey'//the property of a key or pointer event that says whether that modifier is down

export function setCommandKey(property) {//metaKey for a platform whose command key is command, as the mac's is
	if (property != 'ctrlKey' && property != 'metaKey') throw new Error(`expected ctrlKey or metaKey: ${property}`)
	commandKey = property
}

export function isCommandOrControl(e) { return e[commandKey] }
