<script>
  /**
   * ZoneExile — flat exile pile (shared zone).
   *
   * Props:
   *   cards (CardInZoneView[]) — exiled cards
   */
  const { cards = [] } = $props();
</script>

<div class="zone-exile">
  <div class="zone-header">
    <span class="zone-label">Exile</span>
    <span class="zone-count muted">{cards.length}</span>
  </div>

  {#if cards.length === 0}
    <div class="empty-zone muted">— empty —</div>
  {:else}
    <div class="exile-list">
      {#each cards as card (card.object_id)}
        <div class="exile-card" title="{card.name} ({(card.card_types ?? []).join(', ')})">
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
  .zone-exile {
    background: #181820;
    border: 1px solid #303050;
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
    border-bottom: 1px solid #282840;
    padding-bottom: 0.2rem;
  }

  .zone-label {
    color: #88c;
    font-weight: bold;
    font-size: 0.78rem;
  }

  .muted {
    color: #446;
    font-size: 0.7rem;
  }

  .empty-zone {
    padding: 0.25rem 0;
    font-size: 0.72rem;
  }

  .exile-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.2rem;
    max-height: 100px;
    overflow-y: auto;
  }

  .exile-card {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.1rem 0.3rem;
    background: #20203a;
    border: 1px solid #30305a;
    border-radius: 3px;
    font-size: 0.7rem;
  }

  .exile-card:hover {
    background: #2a2a4a;
  }

  .card-name {
    color: #aac;
  }

  .card-type {
    font-size: 0.62rem;
  }
</style>
