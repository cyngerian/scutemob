<script>
  /**
   * ZoneGraveyard — ordered list of cards in a player's graveyard.
   * Top of graveyard (most recently placed) is shown first.
   *
   * Props:
   *   cards (CardInZoneView[]) — graveyard contents, top first
   *   playerName (string) — player label for the zone header
   */
  const { cards = [], playerName, onCardClick = null } = $props();
</script>

<div class="zone-graveyard">
  <div class="zone-header">
    <span class="zone-label">Graveyard</span>
    <span class="zone-count muted">{cards.length}</span>
  </div>

  {#if cards.length === 0}
    <div class="empty-zone muted">— empty —</div>
  {:else}
    <div class="gy-list">
      {#each cards as card, i (card.object_id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="gy-card"
          class:clickable={onCardClick !== null}
          title="{card.name} ({(card.card_types ?? []).join(', ')})"
          onclick={() => onCardClick?.(card)}
        >
          <span class="gy-index muted">{i + 1}.</span>
          <span class="card-name">{card.name}</span>
          {#if card.card_types?.length}
            <span class="card-type muted">{card.card_types[0]}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .zone-graveyard {
    background: #1a1010;
    border: 1px solid #3a2020;
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    font-family: monospace;
    font-size: 0.78rem;
    min-width: 130px;
  }

  .zone-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.3rem;
    border-bottom: 1px solid #2a1818;
    padding-bottom: 0.2rem;
  }

  .zone-label {
    color: #a66;
    font-weight: bold;
    font-size: 0.78rem;
  }

  .muted {
    color: #554;
    font-size: 0.7rem;
  }

  .empty-zone {
    padding: 0.25rem 0;
    font-size: 0.72rem;
  }

  .gy-list {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    max-height: 120px;
    overflow-y: auto;
  }

  .gy-card {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.05rem 0.2rem;
    border-radius: 2px;
    font-size: 0.72rem;
  }

  .gy-card:hover {
    background: #2a1818;
  }

  .gy-card.clickable {
    cursor: pointer;
  }

  .gy-card.clickable:hover {
    background: #341a1a;
  }

  .gy-index {
    min-width: 1.2rem;
    text-align: right;
  }

  .card-name {
    color: #c88;
    flex: 1;
  }

  .card-type {
    font-size: 0.65rem;
  }
</style>
