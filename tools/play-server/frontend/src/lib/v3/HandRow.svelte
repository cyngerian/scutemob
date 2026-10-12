<script>
  /**
   * HandRow — your hand, under your board
   * (`docs/play-surface-v3-brief.md` §1 rule 1; the mockup's `handRow`).
   *
   * Fixed 70px cards (the stylesheet pins `--card` on `.hand`; autosize never
   * touches them). The whole hand shows while it fits the row's width; only
   * when the width runs out does it page sideways with arrows, one card at a
   * time. The row measures itself, and re-measures when it is resized.
   *
   * The cards can be dragged into another order (brief §2 item 3; `reorder.js` is the
   * drag, `order.js` the order and where it is kept). The order is yours alone — the
   * server never hears of it — and covers the whole hand, though a drag only reaches
   * the cards on the visible page. A drag that leaves the row is not a rearrangement:
   * it is handed to `onDragOut` (the targeting arrow).
   *
   * Props:
   *   cards       — `state.zones.hand[you]`
   *   offeredIds  — `Set<number>` of object ids the decision offers
   *   gameKey     — the key the order is remembered under (`order.js` `trackGame`); null = not remembered
   *   onCardClick — `(card, null)`
   *   onDragOut   — `(card, pointerEvent)`, once, when a drag leaves the row
   */
  import CardFace from './CardFace.svelte';
  import { applyOrder, loadOrder, moveItem, saveOrder } from './order.js';
  import { reorderable } from './reorder.js';

  let { cards = [], offeredIds = new Set(), gameKey = null, onCardClick = null, onDragOut = null } = $props();

  // ── Your order ─────────────────────────────────────────────────────────────

  const idOf = (card) => Number(card?.object_id);
  /** What storage holds for this game, read once per key; a drag since then is `moved`. */
  const stored = $derived(loadOrder(gameKey).hand);
  let moved = $state.raw(null);
  const savedIds = $derived(moved && moved.key === gameKey ? moved.ids : stored);
  const ordered = $derived(applyOrder(cards, savedIds));

  function moveCard(movedId, beforeId) {
    // The end of the visible page is not the end of the hand: the card goes in front
    // of the first one the page hides on that side.
    const before = beforeId ?? (start + paging.visible < ordered.length ? idOf(ordered[start + paging.visible]) : null);
    // saved as the ids on the table now, so a card that has left is gone from the list
    const ids = moveItem(ordered.map(idOf).filter(Number.isFinite), movedId, before);
    moved = { key: gameKey, ids };
    saveOrder(gameKey, { hand: ids });
  }

  function dragOut(oid, event) {
    const card = ordered.find((c) => idOf(c) === oid);
    if (card) onDragOut?.(card, event);
  }

  let row = $state(null);
  let width = $state(0);
  let offset = $state(0);

  $effect(() => {
    if (!row) return;
    const measure = () => (width = row.clientWidth);
    measure();
    if (typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(measure);
    observer.observe(row);
    return () => observer.disconnect();
  });

  // the mockup's arithmetic: a card and its gap, the label, two arrows and the "a–b of n" text
  const paging = $derived.by(() => {
    const n = cards.length;
    if (width === 0) return { paged: false, visible: n, max: 0 };
    const rem = parseFloat(getComputedStyle(document.documentElement).fontSize);
    const cardW = 70 + 6;
    const labelW = 3.4 * rem + 16;
    const fits = Math.floor((width - labelW) / cardW);
    const paged = n > fits;
    const visible = paged ? Math.max(1, Math.floor((width - labelW - 2 * 1.6 * rem - 7 * rem) / cardW)) : n;
    return { paged, visible, max: Math.max(0, n - visible) };
  });
  // clamped on read, so a hand that shrinks never leaves the row scrolled past its end
  const start = $derived(Math.min(offset, paging.max));
  const shown = $derived(ordered.slice(start, start + paging.visible).map((card, k) => ({ card, index: start + k })));
  // Keyed by the object, not its place, so a move re-seats the cards instead of remounting
  // them. A card with no id (the seat view hides it) has only its place to go by.
  const keyOf = (card, index) => (card?.object_id === undefined || card?.object_id === null ? `at:${index}` : `id:${card.object_id}`);
</script>

{#snippet list()}
  <div class="cards" use:reorderable={{ onMove: moveCard, onDragOut: dragOut }}>
    {#each shown as { card, index } (keyOf(card, index))}
      <CardFace {card} size="hand" lit={offeredIds.has(Number(card.object_id))} onclick={(c) => onCardClick?.(c, null)} />
    {/each}
  </div>
{/snippet}

<div class="hand" bind:this={row}>
  <div class="label">HAND {cards.length}</div>
  {#if paging.paged}
    <button class="arrow" disabled={start === 0} onclick={() => (offset = start - 1)}>◀</button>
    {@render list()}
    <button class="arrow" disabled={start >= paging.max} onclick={() => (offset = start + 1)}>▶</button>
    <span class="dim">{start + 1}–{Math.min(cards.length, start + paging.visible)} of {cards.length}</span>
  {:else}
    {@render list()}
  {/if}
</div>
