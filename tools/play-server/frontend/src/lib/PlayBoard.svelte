<script>
  /**
   * PlayBoard — the play surface's board: battlefields in a reflowing grid, then
   * the shared graveyard/exile row.
   *
   * UI-3 (`scutemob-180`, AC 6008), from the first-human-playtest notes:
   *   - "battlefields should be 2x2: tons of empty space on the right of the board"
   *   - "players battlefield should disapear when they die / row of battlefields
   *      goes away when 2 players are gone, so remaining battlefields render larger"
   *
   * # Why this is a play-local component and not an edit to `$viewer/StateView.svelte`
   *
   * `StateView` is imported **in place** from the replay viewer, which is a
   * step-debugger for a whole game: it wants every seat's hand in a row, one
   * battlefield per row, and it must keep showing a dead player's board because
   * stepping *backwards* past their elimination is the normal thing to do there.
   * Every requirement above is the opposite. Editing `StateView` would have
   * forced the two surfaces to share a layout that neither wants; `PlayApp` uses
   * this instead and `StateView` is untouched, so the replay viewer is
   * byte-for-byte what it was.
   *
   * The *leaf* components are still shared and imported through `$viewer` —
   * `ZoneBattlefield`, `ZoneGraveyard`, `ZoneExile`. Only the arrangement is
   * local, which is exactly the split `docs/mtg-engine-replay-viewer.md`
   * §"Shared Component Strategy" describes.
   *
   * Props:
   *   state (StateViewModel) — ALREADY seat-redacted by the server
   *   humanName (string|null) — display name of the seat this payload is for
   *   onCardClick (fn|null)
   */
  import ZoneBattlefield from '$viewer/ZoneBattlefield.svelte';
  import ZoneGraveyard from '$viewer/ZoneGraveyard.svelte';
  import ZoneExile from '$viewer/ZoneExile.svelte';

  const { state, humanName = null, onCardClick = null } = $props();

  const playerNames = $derived(state?.players ? Object.keys(state.players).sort() : []);

  /** CR 104.2/104.3a: a player who has lost or conceded is out of the game. */
  function isEliminated(pname) {
    const p = state?.players?.[pname];
    return !!(p?.has_lost || p?.has_conceded);
  }

  /**
   * Whose battlefield gets a cell.
   *
   * Eliminated seats are dropped entirely rather than rendered empty, and the
   * grid is `auto-fit` over a minimum column width, so four boards lay out 2×2
   * and two boards lay out 2×1 at **double the width** with no code branch on
   * the count. That is the playtest note's "remaining battlefields render
   * larger" without a hardcoded 2.
   *
   * CR 800.4a empties a departing player's battlefield anyway (their permanents
   * cease to exist / are exiled), so an eliminated seat's cell would be an empty
   * box occupying a quarter of the board — which is the dead space the note is
   * about, in its worst form.
   */
  const livingNames = $derived(playerNames.filter((p) => !isEliminated(p)));
  const eliminatedNames = $derived(playerNames.filter((p) => isEliminated(p)));

  /**
   * An eliminated seat is only worth a line if it still holds something. It
   * normally does not (CR 800.4a), so this is usually empty — but a token that
   * ceased to exist and a permanent that did not are different facts, and
   * silently dropping the second would be a lie rather than a simplification.
   */
  const eliminatedWithBoard = $derived(
    eliminatedNames.filter((p) => (state?.zones?.battlefield?.[p]?.length ?? 0) > 0),
  );

  const graveyardNames = $derived(
    playerNames.filter((p) => (state?.zones?.graveyard?.[p]?.length ?? 0) > 0),
  );
  const anyExile = $derived((state?.zones?.exile?.length ?? 0) > 0);
</script>

<div class="play-board">
  <section class="battlefield-grid" style="--cells: {livingNames.length}">
    {#each livingNames as pname (pname)}
      {@const permanents = state?.zones?.battlefield?.[pname] ?? []}
      <div class="bf-cell" class:is-human={pname === humanName}>
        <div class="bf-label">
          {pname}
          {#if pname === humanName}<span class="you">you</span>{/if}
          {#if state?.turn?.active_player === pname}<span class="active">active</span>{/if}
        </div>
        <ZoneBattlefield {permanents} playerName={pname} {onCardClick} />
      </div>
    {/each}
  </section>

  {#if eliminatedWithBoard.length > 0}
    <section class="eliminated-boards">
      {#each eliminatedWithBoard as pname (pname)}
        <div class="bf-cell eliminated">
          <div class="bf-label">{pname} <span class="out">out of the game</span></div>
          <ZoneBattlefield
            permanents={state.zones.battlefield[pname]}
            playerName={pname}
            {onCardClick}
          />
        </div>
      {/each}
    </section>
  {/if}

  {#if graveyardNames.length > 0 || anyExile}
    <section class="gy-exile-row">
      {#each graveyardNames as pname (pname)}
        <div class="gy-cell">
          <div class="gy-label">{pname}</div>
          <ZoneGraveyard cards={state.zones.graveyard[pname]} playerName={pname} {onCardClick} />
        </div>
      {/each}
      {#if anyExile}
        <div class="gy-cell">
          <ZoneExile cards={state.zones.exile} {onCardClick} />
        </div>
      {/if}
    </section>
  {/if}
</div>

<style>
  .play-board {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    font-family: monospace;
  }

  /*
    `auto-fit` + `minmax` is what makes the reflow automatic: with four living
    boards and a wide viewport this is 2×2; when two players die the two
    survivors each take a full half. No JS decides the column count.
  */
  .battlefield-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(22rem, 1fr));
    gap: 0.4rem;
    align-items: start;
  }

  /* One living board should not stretch to a single absurdly wide column. */
  .battlefield-grid:has(> .bf-cell:only-child) {
    grid-template-columns: minmax(22rem, 48rem);
  }

  .bf-cell {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .bf-cell.is-human {
    outline: 1px solid #2a4a6a;
    outline-offset: 2px;
    border-radius: 4px;
  }

  .bf-cell.eliminated {
    opacity: 0.45;
    filter: grayscale(0.6);
  }

  .bf-label {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
    font-size: 0.72rem;
    color: #668;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .you {
    color: #6af;
    font-size: 0.62rem;
    border: 1px solid #2a4a7a;
    border-radius: 2px;
    padding: 0 0.2rem;
  }

  .active {
    color: #fa0;
    font-size: 0.62rem;
    border: 1px solid #7a5a10;
    border-radius: 2px;
    padding: 0 0.2rem;
  }

  .out {
    color: #a66;
    font-size: 0.62rem;
  }

  .eliminated-boards {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .gy-exile-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: flex-start;
  }

  .gy-cell {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .gy-label {
    font-size: 0.68rem;
    color: #556;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
</style>
