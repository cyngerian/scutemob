import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { installGlobalErrorReporting } from './lib/stores.js'

// UI-4 (`scutemob-185`): installed before mount so the listeners are armed for
// every handler the app will ever attach. Stated precisely, because the obvious
// stronger claim is false: the strip lives inside `ActionBar`, so a throw during
// the very first render sets the store and renders nothing. Installing early is
// still right — it costs nothing and closes the window between mount and first
// paint. See `stores.js` for why `window` is the only thing that sees a DOM
// handler's exception at all.
installGlobalErrorReporting()

const app = mount(App, {
  target: document.getElementById('app'),
})

export default app
