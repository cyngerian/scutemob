<script>
  /**
   * PlayerPanel — per-player info bar: life, mana pool, poison, commander damage.
   *
   * Props:
   *   player (PlayerView) — player state from the view model
   *   playerName (string) — display name for this player
   *   isActive (boolean) — true when this player is the active player
   *   hasPriority (boolean) — true when this player holds priority
   */
  const { player, playerName, isActive = false, hasPriority = false } = $props();

  // Mana pip color classes
  const MANA_COLORS = [
    { key: 'white',     label: 'W', cls: 'mana-w' },
    { key: 'blue',      label: 'U', cls: 'mana-u' },
    { key: 'black',     label: 'B', cls: 'mana-b' },
    { key: 'red',       label: 'R', cls: 'mana-r' },
    { key: 'green',     label: 'G', cls: 'mana-g' },
    { key: 'colorless', label: 'C', cls: 'mana-c' },
  ];

  // Compute total mana in pool
  const totalMana = $derived(
    MANA_COLORS.reduce((sum, m) => sum + (player?.mana_pool?.[m.key] ?? 0), 0)
  );

  // Flatten commander damage into a list for display
  const commanderDamageList = $derived(() => {
    if (!player?.commander_damage_received) return [];
    const entries = [];
    for (const [oppName, byCard] of Object.entries(player.commander_damage_received)) {
      for (const [cardName, dmg] of Object.entries(byCard)) {
        if (dmg > 0) entries.push({ oppName, cardName, dmg });
      }
    }
    return entries;
  });
</script>

<div
  class="player-panel"
  class:is-active={isActive}
  class:has-priority={hasPriority}
  class:has-lost={player?.has_lost}
  class:has-conceded={player?.has_conceded}
>
  <!-- Player name + status badges -->
  <div class="player-header">
    <span class="player-name">{playerName}</span>
    {#if isActive}
      <span class="badge badge-active" title="Active player">ACTIVE</span>
    {/if}
    {#if hasPriority}
      <span class="badge badge-priority" title="Has priority">PRIORITY</span>
    {/if}
    {#if player?.has_lost}
      <span class="badge badge-lost">LOST</span>
    {/if}
    {#if player?.has_conceded}
      <span class="badge badge-conceded">CONCEDED</span>
    {/if}
  </div>

  <!-- Life total -->
  <div class="life-total" class:life-low={(player?.life ?? 20) <= 5}>
    {player?.life ?? '?'}
  </div>

  <!-- Mana pool -->
  {#if totalMana > 0}
    <div class="mana-pool" title="Mana pool">
      {#each MANA_COLORS as m}
        {#if (player?.mana_pool?.[m.key] ?? 0) > 0}
          <span class="mana-pip {m.cls}" title="{m.label}: {player.mana_pool[m.key]}">
            {m.label}{player.mana_pool[m.key] > 1 ? player.mana_pool[m.key] : ''}
          </span>
        {/if}
      {/each}
    </div>
  {/if}

  <!-- Poison counters -->
  {#if (player?.poison ?? 0) > 0}
    <div class="poison-counters" title="Poison counters">
      <span class="poison-label">Poison:</span>
      <span class="poison-value" class:poison-danger={(player?.poison ?? 0) >= 8}>
        {player.poison}
      </span>
    </div>
  {/if}

  <!-- Zone sizes -->
  <div class="zone-sizes">
    <span title="Hand size">Hand: {player?.hand_size ?? 0}</span>
    <span title="Library size">Lib: {player?.library_size ?? 0}</span>
    {#if (player?.graveyard_size ?? 0) > 0}
      <span title="Graveyard size">GY: {player.graveyard_size}</span>
    {/if}
    {#if (player?.land_plays_remaining ?? 0) > 0}
      <span title="Land plays remaining" class="land-plays">
        Lands: {player.land_plays_remaining}
      </span>
    {/if}
  </div>

  <!-- Commander damage received -->
  {#each commanderDamageList() as { oppName, cardName, dmg }}
    <div class="cmd-damage" title="Commander damage from {cardName} ({oppName})">
      <span class="cmd-dmg-source">{cardName}</span>
      <span class="cmd-dmg-value" class:cmd-dmg-lethal={dmg >= 21}>{dmg}</span>
    </div>
  {/each}
</div>

<style>
  .player-panel {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.5rem 0.6rem;
    background: #141428;
    border: 1px solid #333;
    border-radius: 4px;
    min-width: 120px;
    font-family: monospace;
    font-size: 0.8rem;
  }

  .player-panel.is-active {
    border-color: #6af;
    background: #161636;
  }

  .player-panel.has-priority {
    border-color: #fa0;
    box-shadow: 0 0 6px #fa04;
  }

  .player-panel.is-active.has-priority {
    border-color: #fa0;
  }

  .player-panel.has-lost {
    opacity: 0.5;
    border-color: #622;
  }

  .player-panel.has-conceded {
    opacity: 0.5;
    border-color: #422;
  }

  .player-header {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    flex-wrap: wrap;
  }

  .player-name {
    font-weight: bold;
    color: #ddd;
    font-size: 0.85rem;
  }

  .badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.3rem;
    border-radius: 2px;
    font-weight: bold;
  }

  .badge-active {
    background: #1a4a8a;
    color: #6af;
    border: 1px solid #2a5aaa;
  }

  .badge-priority {
    background: #4a3a0a;
    color: #fa0;
    border: 1px solid #8a6a1a;
  }

  .badge-lost {
    background: #4a1a1a;
    color: #f66;
  }

  .badge-conceded {
    background: #3a1a1a;
    color: #c44;
  }

  .life-total {
    font-size: 1.6rem;
    font-weight: bold;
    color: #aef;
    line-height: 1;
    text-align: center;
  }

  .life-total.life-low {
    color: #f44;
  }

  .mana-pool {
    display: flex;
    gap: 0.2rem;
    flex-wrap: wrap;
  }

  .mana-pip {
    font-size: 0.7rem;
    font-weight: bold;
    padding: 0.1rem 0.3rem;
    border-radius: 2px;
    min-width: 1.2rem;
    text-align: center;
  }

  .mana-w { background: #6a6030; color: #ffe; }
  .mana-u { background: #1a3a6a; color: #adf; }
  .mana-b { background: #2a1a4a; color: #d9f; }
  .mana-r { background: #5a2a1a; color: #fba; }
  .mana-g { background: #1a4a2a; color: #afa; }
  .mana-c { background: #3a3a3a; color: #ccc; }

  .poison-counters {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .poison-label {
    color: #888;
    font-size: 0.75rem;
  }

  .poison-value {
    color: #9f6;
    font-weight: bold;
  }

  .poison-value.poison-danger {
    color: #f66;
  }

  .zone-sizes {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    color: #777;
    font-size: 0.75rem;
  }

  .land-plays {
    color: #8a4;
  }

  .cmd-damage {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.3rem;
    background: #2a1a1a;
    border: 1px solid #4a2a2a;
    padding: 0.1rem 0.3rem;
    border-radius: 2px;
    font-size: 0.7rem;
  }

  .cmd-dmg-source {
    color: #c88;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 80px;
  }

  .cmd-dmg-value {
    color: #f88;
    font-weight: bold;
    flex-shrink: 0;
  }

  .cmd-dmg-value.cmd-dmg-lethal {
    color: #f44;
    text-shadow: 0 0 4px #f44;
  }
</style>
