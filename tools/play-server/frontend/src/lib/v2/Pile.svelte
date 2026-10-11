<script>
  /**
   * Pile — a graveyard or exile pile as one fixed button: the top card's image
   * (or chip) with the pile's count, opening a `PilePopover` that lists it.
   * The popover is absolutely positioned inside the owning region, so opening
   * it moves nothing (brief principle 1).
   */
  import Card from './Card.svelte';
  import PilePopover from './PilePopover.svelte';

  const {
    title,
    cards = [],
    size = 'small',
    offeredIds = null,
    onCardClick = null,
    open = false,
    onToggle = null,
    side = 'right',
  } = $props();

  const top = $derived(cards.length > 0 ? cards[0] : null);
  const litCount = $derived(offeredIds ? cards.filter((c) => !c?.hidden && offeredIds.has(Number(c?.object_id))).length : 0);
</script>

<div class="pile pile-{side}" class:open>
  <button
    type="button"
    class="pile-btn"
    class:lit={litCount > 0}
    onclick={() => onToggle?.()}
    aria-expanded={open}
    title="{title}: {cards.length} card{cards.length === 1 ? '' : 's'}"
  >
    <span class="pile-head">
      <span class="pile-title">{title}</span>
      <span class="pile-n">{cards.length}</span>
    </span>
    <span class="pile-top size-{size}">
      {#if top}
        <Card card={top} {size} />
      {:else}
        <span class="pile-empty">—</span>
      {/if}
    </span>
  </button>
  {#if open}
    <PilePopover {title} {cards} {offeredIds} {onCardClick} onClose={() => onToggle?.()} />
  {/if}
</div>

<style>
  .pile {
    position: relative;
    flex: 0 0 auto;
  }

  .pile-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 2px 4px;
    background: #141420;
    border: 1px solid #2a2a40;
    border-radius: 4px;
    color: #aab;
    font-family: monospace;
    cursor: pointer;
  }

  .pile-btn:hover { border-color: #667; }
  .pile.open .pile-btn { border-color: #99a; background: #1c1c2e; }
  .pile-btn.lit { border-color: #5f5; box-shadow: 0 0 6px rgba(110, 230, 130, 0.5); }

  .pile-head {
    display: flex;
    gap: 0.3rem;
    align-items: baseline;
    font-size: 0.58rem;
    letter-spacing: 0.05em;
  }

  .pile-title { color: #889; text-transform: uppercase; }
  .pile-n { color: #ccd; font-weight: bold; }

  .pile-top { display: block; pointer-events: none; }
  .pile-top.size-small { width: 48px; height: 67px; }

  .pile-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: #445;
    border: 1px dashed #2a2a3a;
    border-radius: 4px;
    box-sizing: border-box;
  }
</style>
