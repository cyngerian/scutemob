<script>
  /**
   * ZoneStrip — the 1×5 strip of piles on a side cell's name line
   * (`docs/play-surface-v3-brief.md` §1 rule 4): command zone, hand, library,
   * graveyard, exile. Sized once from the cell's width (`sizeStrips` in
   * `fit.js` sets `--pile`), never by autosize.
   *
   * Props: `name`, `player` (`state.players[name]`), `zones` (`state.zones`).
   */
  import Pile from './Pile.svelte';
  import { seatPiles } from './zones.js';
  import { openPopover } from './ui.svelte.js';

  let { name, player, zones } = $props();

  const piles = $derived(seatPiles(name, player, zones));

  function open(pile, event) {
    openPopover({ title: pile.title, note: pile.note, cards: pile.cards, x: event.clientX, y: event.clientY });
  }
</script>

<div class="zones5">
  <Pile label={piles.cmd.label} top={piles.cmd.top} cz onclick={(e) => open(piles.cmd, e)} />
  <Pile label={piles.hand.label} top={piles.hand.top} count={piles.hand.count} onclick={(e) => open(piles.hand, e)} />
  <div class="piles">
    {#each [piles.lib, piles.gy, piles.ex] as pile (pile.label)}
      <Pile label={pile.label} top={pile.top} count={pile.count} onclick={(e) => open(pile, e)} />
    {/each}
  </div>
</div>
