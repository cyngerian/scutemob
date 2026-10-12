import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import PlayAppV2 from './lib/v2/PlayAppV2.svelte'
import TableV3 from './lib/v3/TableV3.svelte'
import CardWindow from './lib/v3/CardWindow.svelte'
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
const query = new URLSearchParams(window.location.search)
const useV2 = query.has('v2')

// `scutemob-266`: the alpha surface (`docs/play-surface-v3-brief.md`) mounts
// behind `?v3`, and `?card-window` is its popped-out card view for a second
// monitor (fed by a `storage` event, so it needs no game of its own).
// The two earlier surfaces are untouched.
const useV3 = query.has('v3')
const useCardWindow = query.has('card-window')

const root = useCardWindow ? CardWindow : useV3 ? TableV3 : useV2 ? PlayAppV2 : App

const app = mount(root, {
  target: document.getElementById('app'),
})

export default app
