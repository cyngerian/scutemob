<script>
  /**
   * Pile — one zone drawn as a pile with its top card showing
   * (`docs/play-surface-v3-brief.md` §1 rule 4; the mockup's `pile()`).
   *
   * Fixed size: the stylesheet sizes piles per container (55px in a zone box,
   * `--pile` in a strip) and autosize never touches them. An empty pile is a
   * dashed frame; the command slot (`cz`) has its own tint and a star.
   *
   * Props:
   *   label   — 'CMD' | 'HAND' | 'LIB' | 'GY' | 'EX*'
   *   top     — the card on top (`{ hidden: true }` is a back), or null for the empty frame
   *   count   — shown after the label; null for none
   *   cz      — the command slot
   *   onclick — `(event)`; the click never reaches the cell underneath
   */
  import CardFace from './CardFace.svelte';

  let { label, top = null, count = null, cz = false, onclick = null } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="pile"
  class:cz
  onclick={(event) => {
    event.stopPropagation();
    onclick?.(event);
  }}
>
  {#if top}
    <CardFace card={top} size="perm" />
  {:else}
    <div class="empty"></div>
  {/if}
  <div class="lbl">{count == null ? label : `${label} ${count}`}</div>
</div>
