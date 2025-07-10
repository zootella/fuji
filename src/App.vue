<script setup>//./src/App.vue

import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

import Lister from './components/Lister.vue'
import Viewer from './components/Viewer.vue'
import Viewer2 from './components/Viewer2.vue'

const greetMsg = ref('')
const name = ref('')

</script>
<template>

<Viewer2 />

</template>
