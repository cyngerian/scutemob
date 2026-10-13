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
   * Each row can be dragged into another order, within itself (brief §2 item 3;
   * `reorder.js` is the drag, `order.js` the order and where it is kept). Lands are
   * ordered as a flat list and stacked afterwards, so a stack sits where its first
   * land does and moving a stack moves every land in it; the list is by object id, so
   * it survives a land tapping into another stack. A drag that leaves its row is
   * handed to `onDragOut` (the targeting arrow) instead.
   *
   * Props:
   *   name, player, zones — your seat, `state.players[name]`, `state.zones`
   *   isActive, hasPrio   — border state (rule 8)
   *   offeredIds          — `Set<number>` of object ids the decision offers
   *   gameKey             — the key the order is remembered under (`order.js` `trackGame`); null = not remembered
   *   onCardClick         — `(card, group)`; `group` is a land stack's members or null
   *   onDragOut           — `(card, pointerEvent)`, once, when a drag leaves its row
   *   onSeatMenu          — `(event, name)` on a right-click of the name plate
   */
  import CardFace from './CardFace.svelte';
  import ZoneBox from './ZoneBox.svelte';
  import { splitBattlefield, landStacks } from './layout.js';
  import { applyOrder, loadOrder, moveItems, saveOrder } from './order.js';
  import { reorderable } from './reorder.js';

  let {
    name,
    player,
    zones,
    isActive = false,
    hasPrio = false,
    offeredIds = new Set(),
    gameKey = null,
    onCardClick = null,
    onSeatMenu = null,
    onDragOut = null,
  } = $props();

  // ── Your order ─────────────────────────────────────────────────────────────

  const idOf = (card) => Number(card.object_id);
  /** What storage holds for this game, read once per key; drags since then are `moved`. */
  const stored = $derived(loadOrder(gameKey));
  let moved = $state.raw(null);
  const saved = $derived(moved && moved.key === gameKey ? { ...stored, ...moved.lists } : stored);

  const split = $derived(splitBattlefield(zones?.battlefield?.[name] ?? []));
  const rows = $derived({
    creatures: applyOrder(split.creatures, saved.creatures),
    others: applyOrder(split.others, saved.others),
    lands: applyOrder(split.lands, saved.lands), // ordered first: stacks form in first-seen order
  });
  const stacks = $derived(landStacks(rows.lands));

  /** The land stack drawn with `oid` on its face, as its members' ids (a lone card is a stack of one). */
  const stackIds = (oid) => (stacks.find((s) => idOf(s.members[0]) === oid)?.members ?? []).map(idOf);

  function moveIn(list, movedId, beforeId) {
    // A stack moves whole, to just in front of the first land of the stack it was dropped on.
    const group = list === 'lands' ? stackIds(movedId) : [movedId];
    // saved as the ids on the table now, so a permanent that has left is gone from the list
    const ids = moveItems(rows[list].map(idOf), group, beforeId);
    moved = { key: gameKey, lists: { ...(moved && moved.key === gameKey ? moved.lists : {}), [list]: ids } };
    saveOrder(gameKey, { [list]: ids });
  }

  function dragOut(list, oid, event) {
    let card = rows[list].find((c) => idOf(c) === oid);
    if (list === 'lands') {
      // a stack stands for all its lands: hand over one the decision can use, if any
      const members = stacks.find((s) => idOf(s.members[0]) === oid)?.members ?? [];
      card = members.find(isLit) ?? card;
    }
    if (card) onDragOut?.(card, event);
  }
  const rowParams = (list) => ({ onMove: (id, before) => moveIn(list, id, before), onDragOut: (id, event) => dragOut(list, id, event) });
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
    <div class="cards grid8" use:reorderable={rowParams('creatures')}>
      {#each rows.creatures as card (card.object_id)}
        {@render face(card, null)}
      {/each}
    </div>
  </div>
  <div class="zone others">
    <div class="row-label">other {rows.others.length}</div>
    <div class="cards grid8" use:reorderable={rowParams('others')}>
      {#each rows.others as card (card.object_id)}
        {@render face(card, null)}
      {/each}
    </div>
  </div>
  <div class="zone lands">
    <div class="row-label">lands {rows.lands.length}</div>
    <div class="cards" use:reorderable={rowParams('lands')}>
      {#each stacks as stack (stack.key)}
        {@render face(stack.members[0], stack.members)}
      {/each}
    </div>
  </div>
  <ZoneBox {name} {player} {zones} {isActive} {hasPrio} {onSeatMenu} />
</div>
