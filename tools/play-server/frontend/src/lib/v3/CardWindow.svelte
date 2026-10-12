<script>
  /**
   * CardWindow — the popped-out card window for the second monitor
   * (`docs/play-surface-v3-brief.md` §1 rule 7; a port of `mockups/card-window.html`).
   *
   * Mounted on its own at `?card-window`. It shows the last card hovered on
   * the table at full size (Scryfall's `large` printing), fed by the
   * `storage` event of the key the in-page hover writes (`hover.js`). It holds no game state and asks the server for nothing.
   */
  import { scryfallImageUrl } from '../v2/images.js';
  import { CARD_KEY } from './hover.js';

  let name = $state(null);
  let failedName = $state(null);
  const url = $derived(name ? scryfallImageUrl(name, 'large') : null);

  function read(raw) {
    try {
      const n = JSON.parse(raw ?? 'null')?.name;
      if (n) name = n;
    } catch {
      // not ours
    }
  }

  $effect(() => {
    const onStorage = (ev) => {
      if (ev.key === CARD_KEY) read(ev.newValue);
    };
    window.addEventListener('storage', onStorage);
    return () => window.removeEventListener('storage', onStorage);
  });
</script>

<div class="v3 v3-cardwin">
  {#if !name}
    <div class="idle">
      card window<br />
      <span class="small">hover a card on the table — it shows here.<br />Drag this window to your second monitor.</span>
    </div>
  {:else}
    {#if failedName !== name}
      {#key url}
        <img src={url} alt="" loading="lazy" decoding="async" draggable="false" onerror={() => (failedName = name)} />
      {/key}
    {/if}
    <div class="name">{name}</div>
  {/if}
</div>

<style>
  .v3-cardwin {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    background: #08080f;
    color: #ccd;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 13px;
  }
  img {
    max-width: 96vw;
    max-height: 86vh;
    border-radius: 4.5% / 3.2%;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.9);
  }
  .name {
    font-size: 1.1rem;
    color: #ccd;
  }
  .idle {
    color: #556;
    font-size: 0.9rem;
    text-align: center;
    line-height: 1.6;
  }
  .small {
    font-size: 0.8rem;
  }
</style>
