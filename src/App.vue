<script setup>//./src/App.vue

import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

import Lister1 from './components/Lister1.vue'

import Viewer1 from './components/Viewer1.vue'
import Viewer2 from './components/Viewer2.vue'
import Viewer3 from './components/Viewer3.vue'
import Viewer4 from './components/Viewer4.vue'
import Viewer5 from './components/Viewer5.vue'//img tag with gamma and pixelated
import Viewer6 from './components/Viewer6.vue'//decode img in script
import Viewer7 from './components/Viewer7.vue'//img tag in template
import Viewer8 from './components/Viewer8.vue'//img triad

import Pixel1 from './components/Pixel1.vue'

import Pan1 from './components/Pan1.vue'
import Table1 from './components/Table1.vue'
import Table2 from './components/Table2.vue'
import Marble3 from './components/Marble3.vue'
import Space1 from './components/Space1.vue'//pink polka dots

</script>
<template>

<Viewer8 />

</template>
