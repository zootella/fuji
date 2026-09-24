import {invoke} from '@tauri-apps/api/core'

//a trackpad or a magic mouse reaches the page as wheel events, dozens a swipe, and a table reads each one as a command; touch.rs is the long version, and says why the mac alone can tell them from a wheel and why rust drops them there rather than the page

export function touchBlock(on) { return invoke('touch_block', {on}) }//on for a view a swipe would misread, off for one that scrolls by it; this window only, and nothing at all off the mac
