import {invoke} from '@tauri-apps/api/core'
import {ref} from 'vue'
import {documentDir} from '@tauri-apps/api/path'
import parse from 'path-browserify'
import {Time, hasInt, saySize4, sayDuration, tickToText} from '@zootella/ergo'//small helpers from the ergo project, the same in every language fuji is written in
import {diskAppend} from './disk.js'
import {log, logTrouble} from './log.js'
import {brandStem} from './brand.js'
import {forwardize} from './components/library.js'

//an inventory of a folder's whole tree; inventory.rs is the long version

export function inventoryTake(folder, output, options) { return invoke('inventory_take', {folder, output, options}) }//resolves once it ends with its counts, {folders, files, bytes, size, read, path, stopped}
export function inventoryProgress(output) { return invoke('inventory_progress', {output}) }//the same counts so far, or null once it has ended
export function inventoryStop(output) { return invoke('inventory_stop', {output}) }//stops at its next read

//the inventory the panel starts and shows, kept here so it outlives the panel; one at a time

export const inventoryOptions = ref({sha1: false, sha256: false, blake3: false, indent: true, dated: false})//the panel's checkboxes: the whole-file hashes, each a read of every byte, JSON across lines, and a file named for the moment the inventory began rather than one written over each time
export const inventoryFolder  = ref('')//blank before the first
export const inventoryOutput  = ref('')//also how Rust names it
export const inventoryRunning = ref(false)
export const inventoryTrouble = ref('')//why it could not run
export const inventoryCounts  = ref(inventoryCountsNone())//the panel's latest read while it shows, then the final count
export const inventoryTime    = ref(0)//kept current by the panel while it shows, and set at the end
let inventoryBegan = 0

export function inventoryCountsNone() { return {folders: 0, files: 0, bytes: 0, size: 0, read: 0, path: '', stopped: false} }
export function inventorySince() { return Date.now() - inventoryBegan }

export function sayCount(n) { return n.toLocaleString('en-US') }//with commas, as American English writes a number, whatever the system's language
export function sayRate(count, time, say) {//the average over the whole run, not the speed of the moment; blank past the largest exact whole number, which saySize4 refuses
	let n = Math.floor(count / time * Time.hour)
	return hasInt(n) ? ` (${say(n)}/hr)` : ''
}
function inventorySay(folder, counts, time) {//how an inventory went, as the panel shows it, for the end of its file and the log
	let {folders, files, bytes, stopped} = counts
	let rate = (count, say) => time > 0 ? sayRate(count, time, say) : ''
	return `${folder}, ${stopped ? 'stopped' : 'done'} in ${sayDuration(time, '#h #m #s')}: ${sayCount(folders)} folders, ${sayCount(files)} files${rate(files, sayCount)}, ${saySize4(bytes)}${rate(bytes, saySize4)}`
}

export async function inventoryRun(folder) {//resolves once the inventory ends
	if (inventoryRunning.value) return
	inventoryRunning.value = true//at once, so a second press finds one under way
	inventoryFolder.value = folder; inventoryOutput.value = ''
	inventoryCounts.value = inventoryCountsNone(); inventoryTrouble.value = ''
	inventoryBegan = Date.now(); inventoryTime.value = 0
	try {
		let {sha1, sha256, blake3, indent, dated} = inventoryOptions.value
		let name = dated ? `${brandStem}-inventory-${tickToText(inventoryBegan)}.txt` : `${brandStem}-inventory.txt`
		let output = parse.join(forwardize(await documentDir()), name)
		inventoryOutput.value = output
		log(`inventory: taking ${folder} into ${output}, ${JSON.stringify(inventoryOptions.value)}`)
		inventoryCounts.value = await inventoryTake(folder, output, {sha1, sha256, blake3, indent})
		inventoryTime.value = inventorySince()
		let said = inventorySay(folder, inventoryCounts.value, inventoryTime.value)
		await diskAppend(output, `# ${said}\n`)//a footer no entry can be mistaken for, since an entry's second field is always a quoted path
		log(`inventory: ${said}`)
	} catch (error) {
		inventoryTrouble.value = String(error)
		logTrouble('inventory: taking', error)
	} finally {
		inventoryTime.value = inventorySince()
		inventoryRunning.value = false
	}
}
