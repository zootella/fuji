import {createApp} from 'vue'
import App from './App.vue'
import './index.css'

createApp(App).mount('#app')//one app per window: every window fuji makes loads its own page, on the mac as much as anywhere, so nothing here has to keep one window's state apart from another's
