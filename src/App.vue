<script setup>//./src/App.vue

import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

import Lister1 from './components/Lister1.vue'

import Viewer1 from './components/Viewer1.vue'
import Viewer2 from './components/Viewer2.vue'
import Viewer3 from './components/Viewer3.vue'
import Viewer4 from './components/Viewer4.vue'

const greetMsg = ref('')
const name = ref('')

</script>
<template>

<Viewer4 />

</template>
