<script setup>//./src/App.vue

import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

import Viewer from './components/Viewer.vue'
import Lister from './components/Lister.vue'

const greetMsg = ref('')
const name = ref('')

</script>
<template>

<Viewer />

</template>
