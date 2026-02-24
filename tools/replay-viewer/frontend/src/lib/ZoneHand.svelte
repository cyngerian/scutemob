<script>
  /**
   * ZoneHand — horizontal list of cards in a player's hand.
   *
   * This is a dev tool, so all hands are visible.
   *
   * Props:
   *   cards (CardInZoneView[]) — cards in this player's hand
   *   playerName (string) — player label for the zone header
   */
  const { cards = [], playerName } = $props();

  function primaryType(cardTypes) {
    if (!cardTypes?.length) return 'unknown';
    if (cardTypes.includes('Creature')) return 'creature';
    if (cardTypes.includes('Instant')) return 'instant';
    if (cardTypes.includes('Sorcery')) return 'sorcery';
    if (cardTypes.includes('Enchantment')) return 'enchantment';
    if (cardTypes.includes('Artifact')) return 'artifact';
    if (cardTypes.includes('Land')) return 'land';
    if (cardTypes.includes('Planeswalker')) return 'planeswalker';
    return 'other';
  }
</script>

<div class="zone-hand">
  <div class="zone-header">
    <span class="zone-label">Hand</span>
    <span class="zone-count muted">{cards.length} card{cards.length !== 1 ? 's' : ''}</span>
  </div>

  {#if cards.length === 0}
    <div class="empty-zone muted">— empty —</div>
  {:else}
    <div class="hand-cards">
      {#each cards as card (card.object_id)}
        <div
          class="hand-card card-type-{primaryType(card.card_types)}"
          title="{card.name} ({(card.card_types ?? []).join(', ')})"
        >
          <span class="card-name">{card.name}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .zone-hand {
    background: #141420;
    border: 1px solid #222238;
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    font-family: monospace;
    font-size: 0.78rem;
  }

  .zone-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.3rem;
  }

  .zone-label {
    color: #88a;
    font-weight: bold;
    font-size: 0.78rem;
  }

  .muted {
    color: #445;
    font-size: 0.72rem;
  }

  .empty-zone {
    padding: 0.25rem 0;
    font-size: 0.72rem;
  }

  .hand-cards {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .hand-card {
    padding: 0.15rem 0.35rem;
    border-radius: 3px;
    border: 1px solid #333;
    cursor: default;
    font-size: 0.72rem;
    background: #1a1a30;
    transition: border-color 0.1s;
  }

  .hand-card:hover {
    border-color: #556;
  }

  .card-name {
    color: #ccce;
    white-space: nowrap;
  }

  /* Type-based background tints */
  .card-type-creature    { border-color: #2a4a30; background: #131d18; }
  .card-type-instant     { border-color: #1a3a6a; background: #10182a; }
  .card-type-sorcery     { border-color: #3a1a6a; background: #181028; }
  .card-type-enchantment { border-color: #3a2a5a; background: #1a1428; }
  .card-type-artifact    { border-color: #3a3a4a; background: #18181e; }
  .card-type-land        { border-color: #2a3a1a; background: #141a10; }
  .card-type-planeswalker{ border-color: #5a2a2a; background: #241414; }
</style>
