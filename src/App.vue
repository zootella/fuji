<script setup>//./src/App.vue

import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

import MyFlip from './components/MyFlip.vue'//data url path and img triad
import MyLens from './components/MyLens.vue'//img tag with gamma and pixelated
import MyList from './components/MyList.vue'//list the images in the folder from a drag
import MySpace from './components/MySpace.vue'//pink polka dots

import LightTable from './components/LightTable.vue'

</script>
<template>

<LightTable />

</template>
