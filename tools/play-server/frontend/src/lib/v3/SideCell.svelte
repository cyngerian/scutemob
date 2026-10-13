<script>
  /**
   * SideCell — an opponent's fixed seat in a side column
   * (`docs/play-surface-v3-brief.md` §1 rules 3 and 14; the mockup's `sideCell` and `openPeek`).
   *
   * Every opponent keeps a cell, the focused one included (greyed out, in
   * addition to its board across the table). The cell is mirrored like that
   * board: under the name line and the 1×5 strip come lands, then other
   * permanents, then creatures last. Clicking the cell brings the seat across;
   * a card in it takes the click only while the current decision offers it
   * (`offeredIds`), so every other click still means "focus".
   *
   * Peek: the pointer resting on the cell itself for a second — never while a
   * card in it is hovered — asks the table to draw this seat again at double
   * size (`openPeek` in `ui.svelte.js`). That copy is this component with
   * `pop`: fixed, positioned by `popStyle`, labelled rows, fitted to its own
   * box, closed when the pointer leaves.
   *
   * Props:
   *   name, player, zones — the seat, `state.players[name]`, `state.zones`
   *   isActive, hasPrio   — border state (rule 8)
   *   focused             — this seat is the one across the table (greyed out)
   *   lift                — the grey-out is lifted because its cards can be chosen
   *   pop, popStyle, fromRight — the peeked copy; `peekStyle(rect)` in `zones.js` gives the last two
   *   offeredIds          — `Set<number>` of object ids the decision offers
   *   onCardClick         — `(card, group)`; `group` is a land stack's members or null
   *   onFocus             — `(name)`
   *   onSeatMenu          — `(event, name)` on a right-click
   */
  import CardFace from './CardFace.svelte';
  import ZoneStrip from './ZoneStrip.svelte';
  import { splitBattlefield, landStacks } from './layout.js';
  import { settings, openPeek, closePeek } from './ui.svelte.js';
  import { fitBoard } from './fit.js';
  import { poolText, seatOut } from './zones.js';
  import { stackTargets } from './stackTargets.svelte.js';

  let {
    name,
    player,
    zones,
    isActive = false,
    hasPrio = false,
    focused = false,
    lift = false,
    pop = false,
    popStyle = '',
    fromRight = false,
    offeredIds = new Set(),
    onCardClick = null,
    onFocus = null,
    onSeatMenu = null,
  } = $props();

  const rows = $derived(splitBattlefield(zones?.battlefield?.[name] ?? []));
  const stacks = $derived(landStacks(rows.lands));
  const untapped = $derived(rows.lands.filter((p) => !p.tapped).length);
  const life = $derived(player?.life ?? 0);
  const out = $derived(seatOut(player));
  const pool = $derived(poolText(player));
  const dataRows = $derived(JSON.stringify([[stacks.length, 0.7], [rows.others.length, 1], [rows.creatures.length, 1]]));

  const isLit = (card) => offeredIds.has(Number(card.object_id));

  let node = $state(null);

  // peek arms only while the pointer rests on the cell itself, never on a card (a card hover is its own thing)
  let peekTimer = null;
  function disarm() {
    clearTimeout(peekTimer);
    peekTimer = null;
  }
  function arm() {
    if (pop || peekTimer !== null) return;
    peekTimer = setTimeout(() => {
      peekTimer = null;
      if (node) openPeek(name, node.getBoundingClientRect());
    }, 1000);
  }
  $effect(() => disarm);

  // the peeked copy fits its cards to its own, bigger box
  $effect(() => {
    if (!pop || !node) return;
    void dataRows;
    void popStyle;
    fitBoard(node, { cap: settings.cap, auto: settings.auto, oppAuto: settings.oppAuto });
  });

  function click() {
    disarm();
    onFocus?.(name);
    closePeek();
  }
  function menu(event) {
    event.preventDefault();
    disarm();
    onSeatMenu?.(event, name);
  }
</script>

{#snippet face(card, group)}
  {@const lit = group ? group.some(isLit) : isLit(card)}
  <CardFace
    {card}
    count={group ? group.length : 0}
    {lit}
    onclick={lit
      ? (c, event) => {
          event.stopPropagation();
          onCardClick?.(c, group);
        }
      : null}
  />
{/snippet}

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={node}
  class="cell"
  class:cell-pop={pop}
  class:from-right={pop && fromRight}
  class:active={isActive}
  class:is-active={isActive}
  class:has-prio={hasPrio}
  class:focused-cell={focused && !pop}
  class:seat-out={!!out}
  class:seat-targeted={stackTargets.seats.has(name)}
  class:lift={lift && !pop}
  style={pop ? popStyle : undefined}
  data-seat={name}
  data-rows={dataRows}
  data-overhead={pop ? '9' : '8'}
  data-label={pop ? '4.6' : '0'}
  onclick={click}
  oncontextmenu={menu}
  onmousemove={pop ? undefined : (event) => (event.target.closest('.card') ? disarm() : arm())}
  onmouseleave={pop ? () => closePeek() : disarm}
>
  <div class="top">
    <div class="who">
      <div><span class="seat-name">{name}</span></div>
      <div class="life" class:low={life < 20}>{life}</div>
      {#if out}<div class="seat-flag out">{out}</div>{/if}
      {#if player?.poison > 0}<div class="seat-flag poison">poison {player.poison}</div>{/if}
      {#if pool}<div class="seat-flag pool">{pool}</div>{/if}
    </div>
    <ZoneStrip {name} {player} {zones} />
  </div>
  <!-- mirrored like the board across the table: lands nearest the strip, then other, creatures last -->
  <div class="zones">
    <div class="zone lands">
      <div class="row-label">lands {untapped}/{rows.lands.length}</div>
      <div class="cards">
        {#each stacks as stack (stack.key)}
          {@render face(stack.members[0], stack.members)}
        {/each}
      </div>
    </div>
    <div class="zone others">
      {#if pop}<div class="row-label">other {rows.others.length}</div>{/if}
      <div class="cards grid8">
        {#each rows.others as card (card.object_id)}
          {@render face(card, null)}
        {/each}
      </div>
    </div>
    <div class="zone creatures">
      {#if pop}<div class="row-label">creatures {rows.creatures.length}</div>{/if}
      <div class="cards grid8">
        {#each rows.creatures as card (card.object_id)}
          {@render face(card, null)}
        {/each}
      </div>
    </div>
  </div>
  {#if pop}<div class="hint">peek — move the pointer away to close · click to bring across the table</div>{/if}
</div>
