import {invoke} from '@tauri-apps/api/core'

//the mac's menu bar, which menu.rs builds and which the page hears through the window's menu event; these are the things the page asks of it, and off the mac, where there is no menu bar, they answer at once and change nothing

export function menuText(id, text) { return invoke('menu_text', {id, text}) }//retitle one of fuji's own items by its id: the View item, which says where a window goes next, Show Light Table from the sheet and Show Contact Sheet from the table
export function menuEnable(menu, item, enabled) { return invoke('menu_enable', {menu, item, enabled}) }//light or gray one item, by the words the menu bar shows for its menu and for it
export function menuWaiting() { return invoke('menu_waiting') }//the item this window was made to answer, Settings…, About or Fuji Help chosen while fuji had no window, taken away as it answers; blank for every other window, and the shell asks once, at mount
