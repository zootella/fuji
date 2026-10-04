/*
A governor lets a fixed number of calls through at a time to one part of the computer, and holds the rest in line until there is room. The caller asks for everything it wants at once, as greedily as it likes; the governor starts the first four, and each time one finishes it lets the next one in. That is the whole of it: a width, a line, and a count of what is running.

Why fuji needs one. The page is the application, and it is greedy by nature: a card of two hundred thumbnails is two hundred things it wants now. Handing all two hundred to the computer at once looks like the efficient choice, on the theory that the schedulers beneath fuji — the operating system's, the disk's, the processor's — know their hardware better than fuji does. Sometimes that is right. But asking for everything at once costs things no scheduler below can give back. The order is lost, so a card fills in scattered pieces rather than top to bottom. Nothing can be called back, so a card the user has already left keeps working for minutes. And the cost is paid all at once: the web renderer has the processor to decode two hundred large photographs, but not the memory to hold two hundred full-size decodes while it shrinks each one down, so asking it to is not efficient but ruinous. A width is the page saying how much of a resource it will take at once, and a governor is where that is said. Every governor is four wide for now, a number a present-day desktop handles for any of these resources, including four full-size decodes in the web renderer at once; a caller names a governor and says nothing else.

A resource is a name, and the governor does not know what it means. Callers name the part of the computer a call will tax — "rust computation", "web computation", "disk" — and every call made under the same name shares one line, wherever in fuji it comes from. Two parts of fuji that know nothing of each other, the contact sheet and some later drive scanner, govern their use of the same disk by saying the same word, with no object passed between them. The names are the individual hardware and software components of the machine fuji is running on, each of which runs out of something different: the processor, as the Rust threads that decode, which come from a pool that starts a thread for every call that waits, so a governor's width is the only thing deciding how many decode at once; the web renderer, which shares the processor but works on its own threads with its own memory and its own frames to keep; and storage, which on one machine can be a fast internal drive that takes many reads at once and a free USB stick from a conference two years ago, found under a car mat, that takes one at a time and slowly. Today every disk is "disk". Telling them apart later is a matter of naming them apart — "disk /", "disk /Volumes/Archive" — and nothing here changes when that happens, because a governor is made the first time anyone says its name.

A call is governed by the resource it is most likely to exhaust. Most calls use several: an operating system thumbnail reads the disk and then decodes on a Rust thread. Putting such a call through two governors at once would have it hold a place in one line while it waits in the other, so a caller picks the one that matters, the scarcer of the two for that kind of work, and says that name. One caller does hold a place while it waits in a second line, on purpose. A thumbnail the page makes is let in under web computation and keeps its place while the store reads the file under disk, because that place stands for memory as much as for the processor: from the read until the release, the tile holds the whole file and then its full-size decode, and four of those in the page at once is what the width promises. Freeing the place for the read would let every page tile on a card start reading, and whole files would pile up in the page faster than the decodes could take them.

The governor is as dumb as the store. cache.js keeps no queue of its own because a queue in the middle has to be told whose request matters, and that is knowledge only the views have. This is a queue, and it keeps the same discipline: it never decides whose call matters. Calls go in the order they were asked for, every caller is equal, and the only number is the width. The ordering decisions stay in the views that make the calls, which is where they were before: a view that wants its top row first asks for its top row first.

What a governor cannot do is stop a call that has started. Once work is let through it belongs to the layers beneath — a read blocked in the kernel, a decoder running on a pool thread, an invoke Tauri has already sent — and nothing in the page can reach down and halt it. So a call that has been let in runs to its end and holds its place in the width until it does, which is honest, because the resource really is busy until then. A caller that no longer wants an answer, because the user has moved on, ignores it when it arrives. A call still waiting in line has not started, and a caller that might stop wanting it should have its work check, first thing, whether it is still wanted, and return at once if not: let in, it costs a turn of the line and nothing more.

A pause stops the line, not the calls. Paused, a governor lets nothing more through; whatever is already running finishes, and the line resumes in order when the pause lifts. A pause belongs to the name, so it holds the resource still for every caller at once. That is right when the reason is the resource — a drive that has gone to sleep, a machine on battery — and wrong when the reason is one caller, like a sheet hidden behind the table, which would stop the table's reads along with its own.

A call never waits in the line of a name it is already holding a place in. Work let through under disk that then asks disk for a read waits behind itself, and four such calls hold all four places while each waits for a fifth, and nothing moves again. So a resource is governed at one depth: the store reads through disk, and a caller of the store does not wrap the store in disk too.

A failure passes straight through. If work rejects or throws, the caller's promise rejects with the same error, and the place in the width is freed either way, so a broken call never leaves the line one short.

One governor per name per window. Each window is its own webview with its own copy of this module, exactly as each has its own store, so two windows on the Mac each get the full width of every resource. Fuji has one window most of the time, and the day several windows working at once is a real case, the width is worth sharing across them through Rust.

Where this goes next, none of it built. The first is Fuzzy Logic, named for the rice cookers, washing machines and subway trains of late-eighties Japan that advertised it on the box: each governor timing every call in and out, learning how much a resource actually gives at each width, and widening or narrowing itself from four to match, so a fast internal drive is driven harder than a conference thumb drive without anyone typing a number for either. The second is a deadline, which is how a governor answers a call that never ends: past it, the governor frees the place and fails the caller's promise, while the stuck work finishes or does not on a thread nothing else is waiting for. security.md calls that containment, and says why a page cannot do better than it. The third is priority, letting a tile the user can see go ahead of one they cannot. Each attaches to the governor object without changing a single caller.
*/

const governorWidth = 4//how many calls every governor lets through at once, until Fuzzy Logic gives each one its own
const governors = new Map()//resource name to its governor, made the first time anyone says the name

export function governorRun(name, work) {//run work when name has room, in the order asked; resolves or rejects with what work did. work is a function returning a promise, called only once it is let in
	if (typeof work != 'function') throw new Error(`governor: expected a function of work for ${name}: ${work}`)
	let g = _governorFind(name)
	return new Promise((resolve, reject) => {
		g.waiting.push({work, resolve, reject})
		_governorAdmit(g)
	})
}

export function governorPause(name) { _governorFind(name).paused = true }//hold the line for every caller of this name; what is running finishes
export function governorResume(name) {//and let the line move again, from where it stopped
	let g = _governorFind(name)
	g.paused = false
	_governorAdmit(g)
}

function _governorFind(name) {//the governor for a name, made the first time any caller says it
	if (typeof name != 'string' || !name.trim()) throw new Error(`governor: expected the name of a resource: ${name}`)
	let g = governors.get(name)
	if (!g) {
		g = {
			paused: false,
			running: 0,//let through and not yet settled
			waiting: [],//{work, resolve, reject} for each call still in line, first asked at the front
		}
		governors.set(name, g)
	}
	return g
}

function _governorAdmit(g) {//let in as many as there is room for, from the front of the line
	while (!g.paused && g.running < governorWidth && g.waiting.length > 0) {
		let {work, resolve, reject} = g.waiting.shift()
		g.running++
		Promise.resolve()
			.then(work)//called inside the chain rather than directly, so work that throws before it returns a promise rejects like work that rejects
			.then(resolve, reject)//the caller's promise settles with whatever work did
			.finally(() => { g.running--; _governorAdmit(g) })//the place is free however work ended, and the next in line takes it
	}
}
