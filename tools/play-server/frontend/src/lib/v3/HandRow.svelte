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
   * Props:
   *   cards       — `state.zones.hand[you]`
   *   offeredIds  — `Set<number>` of object ids the decision offers
   *   onCardClick — `(card, null)`
   */
  import CardFace from './CardFace.svelte';

  let { cards = [], offeredIds = new Set(), onCardClick = null } = $props();

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
  const shown = $derived(cards.slice(start, start + paging.visible).map((card, k) => ({ card, index: start + k })));
</script>

{#snippet list()}
  <div class="cards">
    {#each shown as { card, index } (`${card.object_id}:${index}`)}
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
