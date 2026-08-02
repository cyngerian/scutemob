<script>
  /**
   * SeatCard — one seat's whole standing: the shared `PlayerPanel`, that seat's
   * command zone, and an expandable drawer for everything else this seat is
   * *entitled* to know about them.
   *
   * UI-3 (`scutemob-180`, AC 6008 + 6008b), from the first-human-playtest notes:
   *   - "command zone could just be in player card"
   *   - "player card should be expandable to view cards in hand you know about"
   *   - "commanders dont show card on hover"
   *
   * # Why this wraps `PlayerPanel` instead of editing it
   *
   * `$viewer/PlayerPanel.svelte` is imported **in place** from the replay
   * viewer (`vite.config.js`), which renders it in an omniscient dev tool with
   * no command zone, no drawer and no notion of a "seat". Everything this
   * component adds is play-surface-specific, so it is added *around* the shared
   * component rather than inside it — **`PlayerPanel.svelte` is byte-for-byte
   * what it was.** (A statement about this component's dependency, not about the
   * whole batch: UI-3 changed one `$viewer` file deliberately and in place,
   * `CombatView.svelte`, because the replay viewer had the same defect.)
   *
   * Props:
   *   player (PlayerView)     — `state.players[playerName]`
   *   playerName (string)
   *   isActive, hasPriority (bool)
   *   isHuman (bool)          — this is the seat the payload is redacted for
   *   commanders (CardInZoneView[]) — `state.zones.command_zone[playerName] ?? []`
   *   hand (CardInZoneView[])       — `state.zones.hand[playerName] ?? []`
   *   graveyard (CardInZoneView[])  — `state.zones.graveyard[playerName] ?? []`
   *   onCardClick (fn|null)
   */
  import PlayerPanel from '$viewer/PlayerPanel.svelte';
  import { cardTooltip } from '$viewer/cardTooltip.js';

  const {
    player,
    playerName,
    isActive = false,
    hasPriority = false,
    isHuman = false,
    commanders = [],
    hand = [],
    graveyard = [],
    onCardClick = null,
  } = $props();

  let expanded = $state(false);

  /** CR 104.2/104.3a — out of the game, but still shown (greyed) so the table reads. */
  const eliminated = $derived(!!(player?.has_lost || player?.has_conceded));

  /**
   * Hand entries this seat may actually identify.
   *
   * `redact::redact_hands` replaces every card of a hand the viewer may not read
   * with `hidden_placeholder()` — `{hidden: true, name: "Hidden card",
   * object_id: 0}` — so "cards in hand you know about" is exactly the non-hidden
   * subset, and it is the *server* that decided which those are. Nothing here
   * re-derives entitlement; a client that filtered on anything but the flag the
   * redactor sets would be a second opinion about Architecture Invariant 7.
   *
   * For the human's own seat this is the whole hand. For an opponent it is
   * normally EMPTY today — the view model has no channel for a revealed card, so
   * a reveal effect does not un-hide an opponent's hand entry. That is stated in
   * the drawer rather than papered over.
   */
  const knownHand = $derived((hand ?? []).filter((c) => c?.hidden !== true));
  const hiddenHandCount = $derived((hand ?? []).length - knownHand.length);

  /**
   * Keyed-`#each` key, same hazard and same fix as `$viewer/ZoneHand.svelte`:
   * every redacted entry carries `object_id: 0`, so keying on the id alone gives
   * a keyed block duplicate keys and Svelte 5 **throws** (`each_key_duplicate`),
   * in production as well as in dev. `knownHand` excludes hidden entries so the
   * collision cannot arise there, but `commanders`/`graveyard` are keyed with
   * the same helper so the pattern does not have to be re-derived if a future
   * redaction rule starts blanking one of them.
   */
  function eachKey(card, i) {
    return card?.hidden ? `hidden-${i}` : card?.object_id;
  }

  /** A hidden card has no identity to preview — `cardTooltip` skips a null name. */
  function previewName(card) {
    return card?.hidden ? null : card?.name;
  }
</script>

<div class="seat-card" class:eliminated class:is-human={isHuman}>
  <PlayerPanel {player} {playerName} {isActive} {hasPriority} />

  <!--
    Command zone, folded into the seat card (playtest note: "command zone could
    just be in player card"). CR 903.6 — a commander in the command zone is
    public information, which is why this can be rendered for every seat and not
    just the human's.
  -->
  {#if commanders.length > 0}
    <div class="cmd-zone">
      <span class="cmd-label">Command</span>
      <div class="cmd-cards">
        {#each commanders as card, i (eachKey(card, i))}
          <!--
            A real `<button>`, unlike the `<span onclick>` the sibling `$viewer`
            zone components use. Not a style preference: since SIM-1
            (`scutemob-175`) a commander in the command zone is **castable** (CR
            903.8), so this is the one card chip on the surface that is a primary
            game action rather than an inspect affordance, and it should not be
            the one you cannot reach from the keyboard. The type is reset in CSS
            so it still reads as a chip.
          -->
          <button
            type="button"
            class="cmd-card"
            class:clickable={onCardClick !== null}
            disabled={onCardClick === null}
            title="{card.name} — {(card.card_types ?? []).join(', ')}"
            onclick={() => onCardClick?.(card)}
            use:cardTooltip={previewName(card)}
          >
            {card.name}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <button
    class="drawer-toggle"
    aria-expanded={expanded}
    onclick={() => (expanded = !expanded)}
    title={expanded ? 'Collapse' : 'Show what this seat knows about this player'}
  >
    {expanded ? '▾' : '▸'} details
  </button>

  {#if expanded}
    <div class="drawer">
      <div class="drawer-section">
        <span class="drawer-label">
          Hand
          <span class="drawer-count">{(hand ?? []).length}</span>
        </span>
        {#if knownHand.length > 0}
          <div class="chip-row">
            {#each knownHand as card, i (eachKey(card, i))}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span
                class="chip"
                class:clickable={onCardClick !== null}
                onclick={() => onCardClick?.(card)}
                use:cardTooltip={previewName(card)}
              >
                {card.name}
              </span>
            {/each}
          </div>
        {/if}
        {#if hiddenHandCount > 0}
          <!--
            Said plainly rather than shown as a row of blanks: the server does
            not send these identities at all (Architecture Invariant 7), so
            there is nothing here that a better client could reveal.
          -->
          <div class="drawer-note">
            {hiddenHandCount} card{hiddenHandCount === 1 ? '' : 's'} this seat may not identify.
          </div>
        {/if}
      </div>

      {#if (graveyard ?? []).length > 0}
        <div class="drawer-section">
          <span class="drawer-label">
            Graveyard
            <span class="drawer-count">{graveyard.length}</span>
          </span>
          <div class="chip-row">
            {#each graveyard as card, i (eachKey(card, i))}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span
                class="chip gy"
                class:clickable={onCardClick !== null}
                onclick={() => onCardClick?.(card)}
                use:cardTooltip={previewName(card)}
              >
                {card.name}
              </span>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .seat-card {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 11rem;
    max-width: 18rem;
    font-family: monospace;
  }

  .seat-card.is-human {
    /* The seat you are sitting in should be findable at a glance. */
    outline: 1px solid #2a4a6a;
    outline-offset: 2px;
    border-radius: 4px;
  }

  /* CR 104.2 — a player who has left the game stays visible but recedes. */
  .seat-card.eliminated {
    opacity: 0.45;
    filter: grayscale(0.6);
  }

  .cmd-zone {
    display: flex;
    align-items: baseline;
    gap: 0.3rem;
    flex-wrap: wrap;
    background: #1a140a;
    border: 1px solid #4a3010;
    border-radius: 3px;
    padding: 0.15rem 0.3rem;
  }

  .cmd-label {
    color: #a80;
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .cmd-cards {
    display: flex;
    flex-wrap: wrap;
    gap: 0.2rem;
  }

  /* Reset the button back to a chip — see the markup note on why it is a button. */
  .cmd-card {
    background: #2a1a06;
    border: 1px solid #6a4a10;
    color: #ca8;
    padding: 0.05rem 0.3rem;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.7rem;
    line-height: inherit;
    cursor: default;
  }

  .cmd-card.clickable {
    cursor: pointer;
  }

  .cmd-card:hover:not(:disabled) {
    background: #3a2a10;
  }

  .cmd-card:disabled {
    /* Inspect-only (no click handler): still readable, not dimmed like a
       disabled control, because nothing is unavailable — there is simply no
       action wired on this surface. */
    opacity: 1;
  }

  .drawer-toggle {
    align-self: flex-start;
    background: transparent;
    border: none;
    color: #667;
    font-family: monospace;
    font-size: 0.65rem;
    padding: 0.05rem 0.1rem;
    cursor: pointer;
  }

  .drawer-toggle:hover {
    color: #99a;
  }

  .drawer {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    background: #10101e;
    border: 1px solid #22223a;
    border-radius: 3px;
    padding: 0.25rem 0.35rem;
  }

  .drawer-section {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .drawer-label {
    color: #667;
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .drawer-count {
    color: #445;
  }

  .drawer-note {
    color: #556;
    font-size: 0.66rem;
    font-style: italic;
  }

  .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.15rem;
  }

  .chip {
    background: #1a1a30;
    border: 1px solid #2a2a45;
    color: #bbc;
    border-radius: 3px;
    padding: 0.05rem 0.25rem;
    font-size: 0.68rem;
    cursor: default;
  }

  .chip.clickable {
    cursor: pointer;
  }

  .chip:hover {
    border-color: #556;
  }

  .chip.gy {
    background: #1a1618;
    border-color: #3a2a2a;
    color: #b9a;
  }
</style>
