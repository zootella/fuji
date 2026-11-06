import {invoke} from '@tauri-apps/api/core'

export function localeGrouping() { return invoke('locale_grouping') }//the separator the user's system puts between groups of digits, like the comma in 16,444,365, with the user's own choice in the system's settings; blank when the system will not say
