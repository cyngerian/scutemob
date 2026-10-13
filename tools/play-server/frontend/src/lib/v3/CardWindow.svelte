<script>
  /**
   * CardWindow — the popped-out card window for the second monitor
   * (`docs/play-surface-v3-brief.md` §1 rule 7; a port of `mockups/card-window.html`).
   *
   * Mounted on its own at `?card-window`. It shows the last card hovered on
   * the table at full size (the `large` printing from the server's local image
   * store), fed by the `storage` event of the key the in-page hover writes
   * (`hover.js`). It holds no game state; the one thing it asks the server for is
   * that picture. When the store does not hold it (a 404) the window shows a
   * card-shaped text chip instead: the name, and the type line and P/T the hover
   * passed along.
   */
  import { cardImageUrl } from '../v2/images.js';
  import { CARD_KEY } from './hover.js';

  let name = $state(null);
  let failedName = $state(null);
  let typeLine = $state(null);
  let pt = $state(null);
  const url = $derived(name ? cardImageUrl(name, 'large') : null);

  function read(raw) {
    try {
      const msg = JSON.parse(raw ?? 'null');
      if (msg?.name) {
        failedName = null; // every hover asks the store again: a prefetch may have landed since
        name = msg.name;
        typeLine = msg.type ?? null;
        pt = msg.pt ?? null;
      }
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
      <div class="name">{name}</div>
    {:else}
      <div class="chip">
        <div class="chip-name">{name}</div>
        {#if typeLine}<div class="chip-type">{typeLine}</div>{/if}
        {#if pt}<div class="chip-pt">{pt}</div>{/if}
      </div>
    {/if}
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
  .chip {
    box-sizing: border-box;
    height: min(86vh, 134vw);
    aspect-ratio: 5 / 7;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 6%;
    text-align: center;
    background: #1b1b2a;
    border: 1px solid #556;
    border-radius: 4.5% / 3.2%;
    font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
    overflow: hidden;
  }
  .chip-name {
    font-size: 2rem;
    font-weight: 700;
    color: #eef;
    overflow-wrap: anywhere;
  }
  .chip-type {
    font-size: 1.2rem;
    color: #99a;
    overflow-wrap: anywhere;
  }
  .chip-pt {
    font-size: 1.6rem;
    font-weight: 800;
    color: #fff;
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
