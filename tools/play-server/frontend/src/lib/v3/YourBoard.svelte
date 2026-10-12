<script>
  /**
   * YourBoard — your battlefield, the centre of the screen
   * (`docs/play-surface-v3-brief.md` §1 rule 1; the mockup's `yourBoard`).
   *
   * Three rows — creatures, other permanents, lands (smaller, same-name lands
   * stacked with a count) — and your 2×2 zone box bottom-right. Every card
   * takes the click, offered or not: an unlit one is answered upstream with
   * "no legal action", which is more use than silence on your own board.
   *
   * Props:
   *   name, player, zones — your seat, `state.players[name]`, `state.zones`
   *   isActive, hasPrio   — border state (rule 8)
   *   offeredIds          — `Set<number>` of object ids the decision offers
   *   onCardClick         — `(card, group)`; `group` is a land stack's members or null
   *   onSeatMenu          — `(event, name)` on a right-click of the name plate
   */
  import CardFace from './CardFace.svelte';
  import ZoneBox from './ZoneBox.svelte';
  import { splitBattlefield, landStacks } from './layout.js';

  let { name, player, zones, isActive = false, hasPrio = false, offeredIds = new Set(), onCardClick = null, onSeatMenu = null } = $props();

  const rows = $derived(splitBattlefield(zones?.battlefield?.[name] ?? []));
  const stacks = $derived(landStacks(rows.lands));
  const dataRows = $derived(JSON.stringify([[rows.creatures.length, 1], [rows.others.length, 1], [stacks.length, 0.7]]));

  const isLit = (card) => offeredIds.has(Number(card.object_id));
</script>

{#snippet face(card, group)}
  <CardFace
    {card}
    count={group ? group.length : 0}
    lit={group ? group.some(isLit) : isLit(card)}
    onclick={(c) => onCardClick?.(c, group)}
  />
{/snippet}

<div
  class="board"
  class:active={isActive}
  class:is-active={isActive}
  class:has-prio={hasPrio}
  data-seat={name}
  data-rows={dataRows}
  data-overhead="3.2"
>
  <div class="zone creatures">
    <div class="row-label">creatures {rows.creatures.length}</div>
    <div class="cards grid8">
      {#each rows.creatures as card (card.object_id)}
        {@render face(card, null)}
      {/each}
    </div>
  </div>
  <div class="zone others">
    <div class="row-label">other {rows.others.length}</div>
    <div class="cards grid8">
      {#each rows.others as card (card.object_id)}
        {@render face(card, null)}
      {/each}
    </div>
  </div>
  <div class="zone lands">
    <div class="row-label">lands {rows.lands.length}</div>
    <div class="cards">
      {#each stacks as stack (stack.key)}
        {@render face(stack.members[0], stack.members)}
      {/each}
    </div>
  </div>
  <ZoneBox {name} {player} {zones} {isActive} {hasPrio} {onSeatMenu} />
</div>
