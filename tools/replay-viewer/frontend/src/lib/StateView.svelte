<script>
  /**
   * StateView — main content area displaying the full game state.
   *
   * Replaces the raw JSON dump from Session 2.
   * Props-based for M11 Tauri app reuse.
   *
   * Props:
   *   state (StateViewModel) — the game state from the view model
   *   diff (Set<string>) — set of changed path strings (from diff.js) for highlighting
   *   onCardClick (function|null) — called with a card object when user clicks a card
   */
  import PlayerPanel from './PlayerPanel.svelte';
  import ZoneBattlefield from './ZoneBattlefield.svelte';
  import ZoneStack from './ZoneStack.svelte';
  import ZoneHand from './ZoneHand.svelte';
  import ZoneGraveyard from './ZoneGraveyard.svelte';
  import ZoneExile from './ZoneExile.svelte';

  const { state, diff = null, onCardClick = null } = $props();

  // Sorted player names for consistent ordering
  const playerNames = $derived(state?.players ? Object.keys(state.players).sort() : []);

  // Command zone cards (merge all players)
  const allCommandZoneCards = $derived(() => {
    if (!state?.zones?.command_zone) return [];
    return Object.entries(state.zones.command_zone)
      .filter(([, cards]) => cards.length > 0)
      .map(([pname, cards]) => ({ pname, cards }));
  });

  // Any graveyard has cards?
  const anyGraveyard = $derived(
    playerNames.some((p) => (state?.zones?.graveyard?.[p]?.length ?? 0) > 0)
  );

  // Any exile?
  const anyExile = $derived((state?.zones?.exile?.length ?? 0) > 0);
</script>

{#if !state}
  <div class="no-state muted">No state loaded.</div>
{:else}
  <div class="state-view">

    <!-- Player panels row -->
    <section class="player-panels-row">
      {#each playerNames as pname (pname)}
        <PlayerPanel
          player={state.players[pname]}
          playerName={pname}
          isActive={state.turn.active_player === pname}
          hasPriority={state.turn.priority === pname}
          lifeChanged={diff?.has(`players.${pname}.life`) ?? false}
          manaChanged={diff?.has(`players.${pname}.mana_pool`) ?? false}
        />
      {/each}
    </section>

    <!-- Stack (if non-empty, shown prominently) -->
    {#if state.zones.stack?.length > 0}
      <section class="stack-section" class:changed={diff?.has('zones.stack') ?? false}>
        <ZoneStack items={state.zones.stack} onCardClick={onCardClick} />
      </section>
    {/if}

    <!-- Battlefield per player -->
    <section class="battlefield-section">
      {#each playerNames as pname (pname)}
        {@const permanents = state.zones.battlefield?.[pname] ?? []}
        {#if permanents.length > 0}
          <div
            class="player-battlefield"
            class:changed={diff?.has(`zones.battlefield.${pname}`) ?? false}
          >
            <div class="player-bf-label">{pname}</div>
            <ZoneBattlefield {permanents} playerName={pname} onCardClick={onCardClick} />
          </div>
        {/if}
      {/each}
    </section>

    <!-- Hands -->
    <section class="hands-section">
      {#each playerNames as pname (pname)}
        <div
          class="player-hand-wrapper"
          class:changed={diff?.has(`zones.hand.${pname}`) ?? false}
        >
          <div class="zone-player-label muted">{pname}</div>
          <ZoneHand
            cards={state.zones.hand?.[pname] ?? []}
            playerName={pname}
            onCardClick={onCardClick}
          />
        </div>
      {/each}
    </section>

    <!-- Graveyards + Exile (shown only if non-empty) -->
    {#if anyGraveyard || anyExile}
      <section class="gy-exile-section">
        {#each playerNames as pname (pname)}
          {#if (state.zones.graveyard?.[pname]?.length ?? 0) > 0}
            <div
              class="player-gy-wrapper"
              class:changed={diff?.has(`zones.graveyard.${pname}`) ?? false}
            >
              <div class="zone-player-label muted">{pname}</div>
              <ZoneGraveyard
                cards={state.zones.graveyard[pname]}
                playerName={pname}
                onCardClick={onCardClick}
              />
            </div>
          {/if}
        {/each}
        {#if anyExile}
          <div
            class="exile-wrapper"
            class:changed={diff?.has('zones.exile') ?? false}
          >
            <ZoneExile cards={state.zones.exile} onCardClick={onCardClick} />
          </div>
        {/if}
      </section>
    {/if}

    <!-- Command Zone (shown only if any commander is there) -->
    {#if allCommandZoneCards().length > 0}
      <section class="command-section">
        <div class="command-label">Command Zone</div>
        <div class="command-list">
          {#each allCommandZoneCards() as { pname, cards }}
            <div class="cmd-player">
              <span class="cmd-player-name">{pname}:</span>
              {#each cards as card (card.object_id)}
                <span class="cmd-card" title="{card.name} — {(card.card_types ?? []).join(', ')}">
                  {card.name}
                </span>
              {/each}
            </div>
          {/each}
        </div>
      </section>
    {/if}

  </div>
{/if}

<style>
  .state-view {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    font-family: monospace;
  }

  .no-state {
    padding: 1rem;
    text-align: center;
    font-family: monospace;
  }

  .muted {
    color: #556;
    font-size: 0.75rem;
  }

  /* Player panels: horizontal row */
  .player-panels-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: flex-start;
  }

  /* Stack section */
  .stack-section {
    display: flex;
    justify-content: flex-start;
  }

  /* Battlefield: one row per player */
  .battlefield-section {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .player-battlefield {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .player-bf-label {
    font-size: 0.72rem;
    color: #668;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* Hands row */
  .hands-section {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: flex-start;
  }

  .player-hand-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 150px;
  }

  .zone-player-label {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  /* Graveyards + exile row */
  .gy-exile-section {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: flex-start;
  }

  .player-gy-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .exile-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  /* Command zone */
  .command-section {
    background: #1a140a;
    border: 1px solid #4a3010;
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    font-size: 0.78rem;
  }

  .command-label {
    color: #a80;
    font-weight: bold;
    font-size: 0.78rem;
    margin-bottom: 0.25rem;
  }

  .command-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .cmd-player {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .cmd-player-name {
    color: #886;
    font-size: 0.72rem;
  }

  .cmd-card {
    background: #2a1a06;
    border: 1px solid #6a4a10;
    color: #ca8;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    font-size: 0.72rem;
    cursor: default;
  }

  .cmd-card:hover {
    background: #3a2a10;
  }
</style>
