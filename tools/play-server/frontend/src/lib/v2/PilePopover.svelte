<script>
  /**
   * PilePopover — the open view of a graveyard or exile pile: a fixed-size
   * panel, absolutely positioned over its owning region, listing the cards as
   * small images (top of the pile first). Offered cards are lit and click
   * through exactly like a card on the board. Closes on Escape, on the ×, or
   * on a click anywhere outside it.
   */
  import Card from './Card.svelte';

  const { title, cards = [], offeredIds = null, onCardClick = null, onClose = null } = $props();

  function offered(c) {
    return !!offeredIds && !c?.hidden && offeredIds.has(Number(c?.object_id));
  }

  function eachKey(card, i) {
    return card?.hidden ? `hidden-${i}` : card?.object_id;
  }

  let panel = $state(null);

  function onKey(e) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      onClose?.();
    }
  }

  function onDocClick(e) {
    if (panel && !panel.contains(e.target) && !e.target.closest?.('.pile-btn')) onClose?.();
  }

  $effect(() => {
    // Capture phase so the panel sees Escape before the prompt panel's own
    // Escape handler backs out of a picker stage.
    window.addEventListener('keydown', onKey, true);
    document.addEventListener('mousedown', onDocClick);
    return () => {
      window.removeEventListener('keydown', onKey, true);
      document.removeEventListener('mousedown', onDocClick);
    };
  });
</script>

<div class="popover" bind:this={panel} role="dialog" aria-label={title}>
  <div class="pop-head">
    <span class="pop-title">{title}</span>
    <span class="pop-n">{cards.length} card{cards.length === 1 ? '' : 's'}{cards.length > 0 ? ' · top first' : ''}</span>
    <button type="button" class="pop-close" onclick={() => onClose?.()} aria-label="Close">×</button>
  </div>
  <div class="pop-body">
    {#each cards as card, i (eachKey(card, i))}
      <Card {card} size="small" offered={offered(card)} onClick={onCardClick} />
    {/each}
    {#if cards.length === 0}<span class="pop-empty">empty</span>{/if}
  </div>
</div>

<style>
  .popover {
    position: absolute;
    z-index: 50;
    right: 0;
    bottom: calc(100% + 4px);
    width: 540px;
    height: 240px;
    display: flex;
    flex-direction: column;
    background: #0f0f1c;
    border: 1px solid #556;
    border-radius: 5px;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.8);
    font-family: monospace;
  }

  .pop-head {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid #2a2a40;
    flex-shrink: 0;
  }

  .pop-title { color: #dde; font-size: 0.7rem; letter-spacing: 0.06em; text-transform: uppercase; }
  .pop-n { color: #778; font-size: 0.62rem; }

  .pop-close {
    margin-left: auto;
    background: none;
    border: 1px solid #334;
    border-radius: 3px;
    color: #aab;
    font-size: 0.8rem;
    line-height: 1;
    padding: 0 0.3rem;
    cursor: pointer;
  }

  .pop-close:hover { border-color: #889; color: #fff; }

  .pop-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    align-content: flex-start;
    padding: 0.4rem 0.5rem;
  }

  .pop-empty { color: #445; font-size: 0.66rem; }
</style>
