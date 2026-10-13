<script>
  /**
   * FocusedBoard — the board across the table
   * (`docs/play-surface-v3-brief.md` §1 rule 2; the mockup's `focusedArea`).
   *
   * One opponent at the top of the centre column, mirrored: lands row at the
   * top (with their hand on it — backs for the cards the seat view hides, face
   * up for the ones it reveals), other permanents, creatures nearest yours.
   * Its 2×2 zone box sits top-right.
   *
   * `maxRows` is the largest creature / other / land-stack count over ALL
   * opponents (the table computes it), so the fitted card size does not jump
   * when the focus moves to another seat.
   *
   * Props:
   *   name, player, zones — the seat, `state.players[name]`, `state.zones`
   *   isActive, hasPrio   — border state (rule 8)
   *   maxRows             — `{ creatures, others, lands }`
   *   offeredIds          — `Set<number>` of object ids the decision offers
   *   onCardClick         — `(card, group)` for an offered card; `group` is a land stack's members or null
   *   onSeatMenu          — `(event, name)` on a right-click of the name plate
   */
  import CardFace from './CardFace.svelte';
  import ZoneBox from './ZoneBox.svelte';
  import { splitBattlefield, landStacks } from './layout.js';
  import { stackTargets } from './stackTargets.svelte.js';

  let {
    name,
    player,
    zones,
    isActive = false,
    hasPrio = false,
    maxRows = { creatures: 0, others: 0, lands: 0 },
    offeredIds = new Set(),
    onCardClick = null,
    onSeatMenu = null,
  } = $props();

  const rows = $derived(splitBattlefield(zones?.battlefield?.[name] ?? []));
  const stacks = $derived(landStacks(rows.lands));
  const hand = $derived(zones?.hand?.[name] ?? []);
  const handSize = $derived(player?.hand_size ?? hand.length);
  const dataRows = $derived(
    JSON.stringify([[maxRows.creatures ?? 0, 1], [maxRows.others ?? 0, 1], [Math.max(maxRows.lands ?? 0, 2), 0.7]]),
  );

  const isLit = (card) => offeredIds.has(Number(card.object_id));
</script>

{#snippet face(card, group)}
  {@const lit = group ? group.some(isLit) : isLit(card)}
  <CardFace {card} count={group ? group.length : 0} {lit} onclick={lit ? (c) => onCardClick?.(c, group) : null} />
{/snippet}

<div
  class="focused"
  class:active={isActive}
  class:is-active={isActive}
  class:has-prio={hasPrio}
  class:seat-targeted={stackTargets.seats.has(name)}
  data-seat={name}
  data-rows={dataRows}
  data-overhead="1.4"
>
  <ZoneBox {name} {player} {zones} {isActive} {hasPrio} {onSeatMenu} />
  <div class="zone lands">
    <div class="row-label">lands {rows.lands.length}</div>
    <div class="cards">
      {#each stacks as stack (stack.key)}
        {@render face(stack.members[0], stack.members)}
      {/each}
    </div>
    <div class="meta handmeta">
      <span>hand {handSize}</span>
      <div class="handrow">
        {#each hand as card, i (i)}
          <CardFace {card} size="perm" />
        {/each}
      </div>
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
  <div class="zone creatures">
    <div class="row-label">creatures {rows.creatures.length}</div>
    <div class="cards grid8">
      {#each rows.creatures as card (card.object_id)}
        {@render face(card, null)}
      {/each}
    </div>
  </div>
</div>
