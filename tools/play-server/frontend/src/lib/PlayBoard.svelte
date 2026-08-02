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
   * this instead and `StateView` is untouched.
   *
   * Precisely: **`StateView.svelte` is byte-for-byte what it was**, so the
   * replay viewer's own composition of it is unaffected. That is not a claim
   * that this batch changed nothing under `$viewer` — it changed exactly one
   * file, `CombatView.svelte`, deliberately and in place, because the replay
   * viewer had the same planeswalker-label defect. See that file's
   * `formatTarget`.
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
   * Eliminated seats are dropped entirely rather than rendered empty. CR 800.4a
   * empties a departing player's battlefield anyway (their permanents cease to
   * exist or are exiled), so an eliminated seat's cell would be an empty box
   * occupying a quarter of the board — which is the dead space the playtest note
   * is about, in its worst form.
   */
  const livingNames = $derived(playerNames.filter((p) => !isEliminated(p)));

  /**
   * How many columns the battlefield grid gets.
   *
   * **Computed, not `auto-fit`,** and that is a correction rather than a
   * preference. The first version used
   * `repeat(auto-fit, minmax(22rem, 1fr))`, which packs as many tracks as will
   * fit: four boards need only ~88rem, so any display wider than that laid them
   * out **1×4** — a single row with the boards squeezed left, which is *exactly*
   * the "tons of empty space on the right of the board" the note complained
   * about. `auto-fit` gave the dead-player reflow for free and quietly failed
   * the headline requirement on the machines most likely to run this.
   *
   * The rule: at most two columns up to four seats, three from five (CR 903.1
   * tables run to six here — `session.rs::MAX_PLAYERS`), so four seats are 2×2,
   * two survivors are 2×1 at full width each, one survivor is a single column,
   * and six seats are 3×2. The narrow-window collapse to one column stays, but
   * as an explicit media query rather than as a side effect of track packing.
   */
  const columns = $derived(
    livingNames.length <= 1 ? 1 : livingNames.length <= 4 ? 2 : 3,
  );
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
  <section class="battlefield-grid" style="--cols: {columns}">
    {#each livingNames as pname (pname)}
      {@const permanents = state?.zones?.battlefield?.[pname] ?? []}
      <div class="bf-cell" class:is-human={pname === humanName}>
        <div class="bf-label">
          {pname}
          {#if pname === humanName}<span class="you">you</span>{/if}
          {#if state?.turn?.active_player === pname}<span class="active">active</span>{/if}
        </div>
        <!--
          `stackLands` — G13 (`scutemob-190`). Opted into here and NOT in the
          replay viewer: see `ZoneBattlefield`'s module doc, which carries the
          whole reasoning (this is a play surface; that one is a step debugger
          whose job is per-object identity).
        -->
        <ZoneBattlefield {permanents} playerName={pname} {onCardClick} stackLands />
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
            stackLands
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
    `--cols` is set inline from `columns` — see its doc for why the column count
    is computed rather than left to `auto-fit`, which laid four boards out 1×4 on
    any display wider than ~88rem and reproduced the very complaint this grid
    exists to answer.

    `minmax(0, 1fr)` rather than `minmax(22rem, 1fr)`: a `1fr` track's implicit
    minimum is `auto`, so a battlefield with a long card name would refuse to
    shrink and overflow the row. The narrow-viewport floor is the media query
    below, which is a statement about the *viewport* — the right place for it —
    instead of a per-track minimum that silently changes the column count.
  */
  .battlefield-grid {
    display: grid;
    grid-template-columns: repeat(var(--cols, 2), minmax(0, 1fr));
    gap: 0.4rem;
    align-items: start;
  }

  /* One living board should not stretch to a single absurdly wide column. */
  .battlefield-grid:has(> .bf-cell:only-child) {
    grid-template-columns: minmax(0, 48rem);
  }

  /* Below two comfortable board widths, stack them however many seats survive. */
  @media (max-width: 60rem) {
    .battlefield-grid {
      grid-template-columns: minmax(0, 1fr);
    }
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
