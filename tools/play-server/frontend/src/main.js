import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { installGlobalErrorReporting } from './lib/stores.js'

// UI-4 (`scutemob-185`): installed BEFORE the app mounts, so a throw during the
// very first render is surfaced too. See `stores.js` for why `window` is the only
// thing that sees a DOM handler's exception.
installGlobalErrorReporting()

const app = mount(App, {
  target: document.getElementById('app'),
})

export default app
