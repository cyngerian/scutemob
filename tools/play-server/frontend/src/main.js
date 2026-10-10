import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import PlayAppV2 from './lib/v2/PlayAppV2.svelte'
import { installGlobalErrorReporting } from './lib/stores.js'

// UI-4 (`scutemob-185`): installed before mount so the listeners are armed for
// every handler the app will ever attach. Stated precisely, because the obvious
// stronger claim is false: the strip lives inside `ActionBar`, so a throw during
// the very first render sets the store and renders nothing. Installing early is
// still right — it costs nothing and closes the window between mount and first
// paint. See `stores.js` for why `window` is the only thing that sees a DOM
// handler's exception at all.
installGlobalErrorReporting()

// `scutemob-263`: the second surface (`docs/play-surface-v2-brief.md`) mounts
// behind `?v2` on the same bundle and the same server, so the two can be
// compared on one seed. The first surface stays the default until the owner
// says otherwise (brief §7).
const useV2 = new URLSearchParams(window.location.search).has('v2')

const app = mount(useV2 ? PlayAppV2 : App, {
  target: document.getElementById('app'),
})

export default app
