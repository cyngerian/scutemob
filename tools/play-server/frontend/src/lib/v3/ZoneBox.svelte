<script>
  /**
   * ZoneBox — the 2×2 box of piles pinned to a centre board's corner
   * (`docs/play-surface-v3-brief.md` §1 rule 4): a title row with the seat's
   * name and life, then command zone, library, graveyard, exile. Bottom-right
   * on your board, top-right on the board across the table (the stylesheet
   * places it from its parent). Active and priority recolour its own border
   * (rule 8).
   *
   * Props:
   *   name, player, zones — the seat, `state.players[name]`, `state.zones`
   *   isActive, hasPrio   — border state
   *   onSeatMenu          — `(event, name)` on a right-click of the name plate
   */
  import Pile from './Pile.svelte';
  import { poolText, seatOut, seatPiles } from './zones.js';
  import { openPopover } from './ui.svelte.js';

  let { name, player, zones, isActive = false, hasPrio = false, onSeatMenu = null } = $props();

  const piles = $derived(seatPiles(name, player, zones));
  const life = $derived(player?.life ?? 0);
  const out = $derived(seatOut(player));
  const pool = $derived(poolText(player));

  function open(pile, event) {
    openPopover({ title: pile.title, note: pile.note, cards: pile.cards, x: event.clientX, y: event.clientY });
  }
</script>

<div class="zonebox" class:is-active={isActive} class:has-prio={hasPrio}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="nameplate"
    oncontextmenu={(event) => {
      event.preventDefault();
      event.stopPropagation();
      onSeatMenu?.(event, name);
    }}
  >
    <span class="seat-name">{name}</span><span class="life" class:low={life < 20}>{life}</span>
  </div>
  {#if out || pool || player?.poison > 0}
    <div class="seat-flags">
      {#if out}<span class="seat-flag out">{out}</span>{/if}
      {#if player?.poison > 0}<span class="seat-flag poison">poison {player.poison}</span>{/if}
      {#if pool}<span class="seat-flag pool">{pool}</span>{/if}
    </div>
  {/if}
  <Pile label={piles.cmd.label} top={piles.cmd.top} taxes={piles.cmd.taxes} cz onclick={(e) => open(piles.cmd, e)} />
  <div class="piles">
    {#each [piles.lib, piles.gy, piles.ex] as pile (pile.label)}
      <Pile label={pile.label} top={pile.top} count={pile.count} onclick={(e) => open(pile, e)} />
    {/each}
  </div>
</div>
