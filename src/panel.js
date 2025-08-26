//./src/panel.js

import {invoke} from '@tauri-apps/api/core'

export function panelResolution() { return invoke('panel_resolution') }
