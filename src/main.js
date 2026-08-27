//./src/main.js
import {createApp} from 'vue'
import {createPinia} from 'pinia'
import App from './App.vue'
import router from './router/index.js'
import './index.css'

createApp(App).use(createPinia()).use(router).mount('#app')//one pinia for the life of the process: fuji opens a single window once, so there's never a second store instance to keep in step
