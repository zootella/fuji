//./src/main.js
import {createApp} from 'vue'
import App from './App.vue'
import './index.css'

createApp(App).mount('#app')//fuji opens a single window and never a second, which is why nothing here has to keep per-instance state apart
