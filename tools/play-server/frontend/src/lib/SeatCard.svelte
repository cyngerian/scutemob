<script>
  /**
   * SeatCard — one seat's whole standing in two lines, plus that seat's command
   * zone and an expandable drawer for everything else this seat is *entitled*
   * to know about them.
   *
   * UI-3 (`scutemob-180`, AC 6008 + 6008b), from the first-human-playtest notes:
   *   - "command zone could just be in player card"
   *   - "player card should be expandable to view cards in hand you know about"
   *   - "commanders dont show card on hover"
   *
   * Re-laid-out by `scutemob-261` for 1080p: it used to wrap
   * `$viewer/PlayerPanel` — a 1.6rem centred life total over three rows,
   * ~145px per card — and four of those plus the phase bar and action bar left
   * the battlefield less than half the screen. The panel's fields are rendered
   * here directly in two rows (name + badges + life; zone counts + pool +
   * poison + commander damage). **`PlayerPanel.svelte` is no longer imported by
   * this surface and is unmodified**; the replay viewer still composes it.
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
   *   offeredIds (Set<number>|null) — see `ZoneHand`; marks a castable commander
   */
  import { cardTooltip, zoneCaption } from '$viewer/cardTooltip.js';

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
    offeredIds = null,
  } = $props();

  function isOffered(card) {
    return !!offeredIds && !card?.hidden && offeredIds.has(Number(card?.object_id));
  }

  let expanded = $state(false);

  /** CR 104.2/104.3a — out of the game, but still shown (greyed) so the table reads. */
  const eliminated = $derived(!!(player?.has_lost || player?.has_conceded));

  const MANA_COLORS = [
    { key: 'white', label: 'W', cls: 'mana-w' },
    { key: 'blue', label: 'U', cls: 'mana-u' },
    { key: 'black', label: 'B', cls: 'mana-b' },
    { key: 'red', label: 'R', cls: 'mana-r' },
    { key: 'green', label: 'G', cls: 'mana-g' },
    { key: 'colorless', label: 'C', cls: 'mana-c' },
  ];

  const pool = $derived(
    MANA_COLORS.filter((m) => (player?.mana_pool?.[m.key] ?? 0) > 0).map((m) => ({
      ...m,
      n: player.mana_pool[m.key],
    })),
  );

  /** Commander damage received, flattened — lethal at 21 (CR 903.10a). */
  const cmdDamage = $derived.by(() => {
    const out = [];
    for (const [opp, byCard] of Object.entries(player?.commander_damage_received ?? {})) {
      for (const [card, dmg] of Object.entries(byCard)) {
        if (dmg > 0) out.push({ opp, card, dmg });
      }
    }
    return out;
  });

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
   * in production as well as in dev.
   */
  function eachKey(card, i) {
    return card?.hidden ? `hidden-${i}` : card?.object_id;
  }

  /**
   * A hidden card has no identity to preview — `cardTooltip` skips a null name.
   * G11 (`scutemob-190`): the caption rides inside the floating div, never a
   * native `title` on the anchor.
   */
  function tooltipArg(card) {
    return { name: card?.hidden ? null : card?.name, caption: zoneCaption(card) };
  }
</script>

<div
  class="seat-card"
  class:eliminated
  class:is-human={isHuman}
  class:is-active={isActive}
  class:has-priority={hasPriority}
>
  <div class="row1">
    <span class="name">{playerName}</span>
    {#if isActive}<span class="badge b-active">ACTIVE</span>{/if}
    {#if hasPriority}<span class="badge b-prio">PRIO</span>{/if}
    {#if player?.has_lost}<span class="badge b-lost">LOST</span>{/if}
    {#if player?.has_conceded}<span class="badge b-lost">CONCEDED</span>{/if}
    <span class="life" class:low={(player?.life ?? 20) <= 5}>{player?.life ?? '?'}</span>
  </div>

  <div class="row2">
    <span>H {player?.hand_size ?? 0}</span>
    <span>L {player?.library_size ?? 0}</span>
    {#if (player?.graveyard_size ?? 0) > 0}<span>GY {player.graveyard_size}</span>{/if}
    {#if (player?.land_plays_remaining ?? 0) > 0}
      <span class="drops">drop {player.land_plays_remaining}</span>
    {/if}
    {#if (player?.poison ?? 0) > 0}
      <span class="poison" class:danger={player.poison >= 8}>☠ {player.poison}</span>
    {/if}
    {#each pool as m (m.key)}
      <span class="pip {m.cls}">{m.label}{m.n > 1 ? m.n : ''}</span>
    {/each}
    {#each cmdDamage as d (d.opp + d.card)}
      <span class="cmd-dmg" class:lethal={d.dmg >= 21}>⚔ {d.card.split(',')[0]} {d.dmg}</span>
    {/each}
  </div>

  <!--
    Command zone, folded into the seat card (playtest note: "command zone could
    just be in player card"). CR 903.6 — a commander in the command zone is
    public information, which is why this can be rendered for every seat.
  -->
  <div class="row3">
    {#each commanders as card, i (eachKey(card, i))}
      <!--
        A real `<button>`: since SIM-1 (`scutemob-175`) a commander in the
        command zone is **castable** (CR 903.8), so this chip is a primary game
        action rather than an inspect affordance, and must be keyboard-reachable.
      -->
      <button
        type="button"
        class="cmd-card"
        class:clickable={onCardClick !== null}
        class:offered={isOffered(card)}
        disabled={onCardClick === null}
        onclick={() => onCardClick?.(card)}
        use:cardTooltip={tooltipArg(card)}
      >
        ★ {card.name}
      </button>
    {/each}
    <button
      class="drawer-toggle"
      aria-expanded={expanded}
      onclick={() => (expanded = !expanded)}
      title={expanded ? 'Collapse' : 'Show what this seat knows about this player'}
    >
      {expanded ? '▾' : '▸'}
    </button>
  </div>

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
                use:cardTooltip={tooltipArg(card)}
              >
                {card.name}
              </span>
            {/each}
          </div>
        {/if}
        {#if hiddenHandCount > 0}
          <!--
            Said plainly rather than shown as a row of blanks: the server does
            not send these identities at all (Architecture Invariant 7).
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
                use:cardTooltip={tooltipArg(card)}
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
    gap: 0.1rem;
    min-width: 12rem;
    max-width: 19rem;
    padding: 0.2rem 0.4rem;
    background: #141428;
    border: 1px solid #2a2a44;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.72rem;
  }

  .seat-card.is-active {
    border-color: #4a7ac0;
    background: #161636;
  }

  .seat-card.has-priority {
    border-color: #fa0;
    box-shadow: 0 0 5px #fa04;
  }

  .seat-card.is-human {
    /* The seat you are sitting in should be findable at a glance. */
    outline: 1px solid #2a4a6a;
    outline-offset: 1px;
  }

  /* CR 104.2 — a player who has left the game stays visible but recedes. */
  .seat-card.eliminated {
    opacity: 0.45;
    filter: grayscale(0.6);
  }

  .row1,
  .row2,
  .row3 {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    flex-wrap: wrap;
    min-height: 1.1rem;
  }

  .name {
    font-weight: bold;
    color: #ddd;
    font-size: 0.8rem;
  }

  .badge {
    font-size: 0.56rem;
    padding: 0 0.25rem;
    border-radius: 2px;
    font-weight: bold;
    letter-spacing: 0.04em;
  }

  .b-active {
    background: #1a4a8a;
    color: #9cf;
  }

  .b-prio {
    background: #4a3a0a;
    color: #fa0;
  }

  .b-lost {
    background: #4a1a1a;
    color: #f66;
  }

  /* The life total: one bold number at the row's end, not a 1.6rem banner. */
  .life {
    margin-left: auto;
    font-size: 1rem;
    font-weight: bold;
    color: #aef;
    line-height: 1;
  }

  .life.low {
    color: #f44;
  }

  .row2 {
    color: #778;
    font-size: 0.66rem;
    gap: 0.45rem;
  }

  .drops {
    color: #8a4;
  }

  .poison {
    color: #9f6;
    font-weight: bold;
  }

  .poison.danger {
    color: #f66;
  }

  .pip {
    font-size: 0.6rem;
    font-weight: bold;
    padding: 0 0.25rem;
    border-radius: 2px;
    min-width: 1rem;
    text-align: center;
  }

  .mana-w { background: #6a6030; color: #ffe; }
  .mana-u { background: #1a3a6a; color: #adf; }
  .mana-b { background: #2a1a4a; color: #d9f; }
  .mana-r { background: #5a2a1a; color: #fba; }
  .mana-g { background: #1a4a2a; color: #afa; }
  .mana-c { background: #3a3a3a; color: #ccc; }

  .cmd-dmg {
    color: #c88;
    background: #2a1a1a;
    border: 1px solid #4a2a2a;
    border-radius: 2px;
    padding: 0 0.25rem;
    font-size: 0.62rem;
  }

  .cmd-dmg.lethal {
    color: #f44;
  }

  /* Reset the button back to a chip — see the markup note on why it is a button. */
  .cmd-card {
    background: #2a1a06;
    border: 1px solid #6a4a10;
    color: #ca8;
    padding: 0 0.3rem;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.66rem;
    line-height: 1.3;
    cursor: default;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cmd-card.clickable {
    cursor: pointer;
  }

  .cmd-card:hover:not(:disabled) {
    background: #3a2a10;
  }

  .cmd-card:disabled {
    /* Inspect-only (no click handler): still readable, not dimmed. */
    opacity: 1;
  }

  /* CR 903.8 — castable from the command zone right now (`scutemob-260`). */
  .cmd-card.offered {
    border-color: #5c5;
    box-shadow: 0 0 0 1px #2a5a2a, 0 0 7px rgba(110, 230, 130, 0.35);
    color: #efe;
  }

  .drawer-toggle {
    margin-left: auto;
    background: transparent;
    border: none;
    color: #667;
    font-family: monospace;
    font-size: 0.7rem;
    padding: 0 0.15rem;
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
