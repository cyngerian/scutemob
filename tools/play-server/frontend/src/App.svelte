<script>
  /**
   * App — the mount point's thin shell.
   *
   * M11-local Session 6 (`memory/m11-session-plan.md` §4, item 4).
   *
   * Everything lives in `lib/PlayApp.svelte` so the whole play surface is one
   * importable component: `src/main.js` owns mounting and nothing else, exactly
   * as `tools/replay-viewer/frontend/src/main.js` does.
   */
  import PlayApp from './lib/PlayApp.svelte';
</script>

<PlayApp />
