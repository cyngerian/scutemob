<script>
  /**
   * PilePopover — the list of cards in an opened pile
   * (`docs/play-surface-v3-brief.md` §1 rule 4; the mockup's `pilePopover`).
   *
   * Rendered once at the table's root from `ui.popover` (see `ui.svelte.js`
   * for why it cannot live inside the board that opened it). It lists exactly
   * the entries it was given: a hidden one is a back.
   *
   * Placed at the pointer, then moved back inside the viewport using its own
   * measured size.
   */
  import CardFace from './CardFace.svelte';
  import { ui, closeFloating } from './ui.svelte.js';

  let node = $state(null);
  const cards = $derived(ui.popover?.cards ?? []);

  $effect(() => {
    const p = ui.popover;
    if (!p || !node) return;
    void cards.length;
    // measured at the origin: a fixed box near the right edge would otherwise be squeezed before it is moved
    node.style.left = '0px';
    node.style.top = '0px';
    const r = node.getBoundingClientRect();
    node.style.left = Math.max(8, Math.min(p.x, innerWidth - r.width - 8)) + 'px';
    node.style.top = Math.max(8, Math.min(p.y, innerHeight - r.height - 8)) + 'px';
  });
</script>

{#if ui.popover}
  <div class="popover" style="--card: 80px" bind:this={node}>
    <div class="panel-title">
      <span>{ui.popover.title}{cards.length > 0 || !ui.popover.note ? ` · ${cards.length}` : ''}{ui.popover.note ? ` · ${ui.popover.note}` : ''}</span>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="close" style="cursor: pointer" onclick={() => closeFloating()}>✕</span>
    </div>
    {#if cards.length}
      <div class="cards grid8">
        {#each cards as card, i (i)}
          <CardFace {card} size="perm" />
        {/each}
      </div>
    {:else}
      <div class="dim">nothing to show</div>
    {/if}
  </div>
{/if}
