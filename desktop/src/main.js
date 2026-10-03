import {createApp} from 'vue'
import App from './App.vue'
import {brandName} from './brand.js'
import './index.css'

document.title = brandName//the page's own title, which no window shows, since rust and the shell title the window; set here so index.html never spells the name
createApp(App).mount('#app')//one app per window: every window fuji makes loads its own page, on the mac as much as anywhere, so nothing here has to keep one window's state apart from another's
