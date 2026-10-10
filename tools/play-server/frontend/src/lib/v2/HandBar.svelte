<script>
  /**
   * HandBar — the viewer seat's hand along the bottom (brief §3). Fixed height,
   * scrolls horizontally past ~14 cards. Playable cards are lit; click is the
   * click-through the app owns.
   */
  import Card from './Card.svelte';

  const { cards = [], offeredIds = null, onCardClick = null } = $props();

  function offered(c) {
    return !!offeredIds && !c?.hidden && offeredIds.has(Number(c?.object_id));
  }

  function eachKey(card, i) {
    return card?.hidden ? `hidden-${i}` : card?.object_id;
  }

  const litCount = $derived(cards.filter(offered).length);
</script>

<div class="hand">
  <div class="label">
    <span class="title">HAND</span>
    <span class="count">{cards.length}</span>
    {#if litCount > 0}<span class="lit">{litCount} playable</span>{/if}
  </div>
  <div class="cards">
    {#each cards as card, i (eachKey(card, i))}
      <Card {card} size="hand" offered={offered(card)} onClick={onCardClick} />
    {/each}
    {#if cards.length === 0}<span class="empty">— empty —</span>{/if}
  </div>
</div>

<style>
  .hand {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding: 0.25rem 0.5rem;
    background: #0d0d1a;
    border-top: 1px solid #22223a;
    font-family: monospace;
  }

  .label {
    display: flex;
    gap: 0.4rem;
    align-items: baseline;
    flex-shrink: 0;
    font-size: 0.64rem;
    color: #778;
  }

  .title {
    color: #88a;
    letter-spacing: 0.06em;
  }

  .lit {
    color: #7c7;
  }

  .cards {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 0.35rem;
    align-items: center;
    overflow-x: auto;
    overflow-y: hidden;
    padding: 0.2rem 0;
  }

  .empty {
    color: #445;
    font-size: 0.7rem;
  }
</style>
