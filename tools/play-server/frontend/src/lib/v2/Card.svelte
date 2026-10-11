<script>
  /**
   * Card — one card on the v2 surface: a Scryfall image inside a fixed box,
   * with the text chip as the fallback in the same box (brief §4).
   *
   * Fixed size is the point: a card's box never depends on its content or on
   * whether its image has arrived, so a row never changes height when a
   * counter appears, a name is long, or Scryfall is slow. The chip renders
   * first and stays until the image has loaded; if the image fails the chip is
   * what the player sees, in the same box, so the layout is identical offline.
   *
   * Overlays on top of the image: the name band (bottom), P/T and tags (above
   * the band), a `×N` count for a land stack, and the lit ring for an offered
   * action. Tokens and hidden cards have no printing to fetch: a token is a
   * chip, a hidden card is a card back.
   *
   * Props:
   *   card      — `CardInZoneView` or `PermanentView`
   *   size      — 'hand' | 'perm' | 'small' | 'chip'
   *   offered   — the current decision offers an action for it (lit)
   *   count     — >1 for a stack of fungible lands ("×N")
   *   onClick   — click-through; null renders an inspect-only card
   */
  import { cardTooltip, zoneCaption } from '$viewer/cardTooltip.js';
  import { scryfallSmallUrl } from './images.js';

  const { card, size = 'perm', offered = false, count = 1, onClick = null } = $props();

  const hidden = $derived(card?.hidden === true);
  const isPerm = $derived(card && 'tapped' in card);
  const name = $derived(card?.name ?? '');

  function typeLine(c) {
    const parts = [];
    if (c?.supertypes?.length) parts.push(...c.supertypes);
    if (c?.card_types?.length) parts.push(...c.card_types);
    if (c?.subtypes?.length) parts.push('—', ...c.subtypes);
    return parts.join(' ');
  }

  /** The hover caption: the same facts `ZoneBattlefield`'s `captionFor` carried. */
  function caption(c) {
    if (!c || c.hidden) return null;
    if (!isPerm) return zoneCaption(c);
    const bits = [];
    if (c.power != null && c.toughness != null) bits.push(`${c.power}/${c.toughness}`);
    if (c.damage_marked > 0) bits.push(`${c.damage_marked} damage marked`);
    for (const [ct, n] of Object.entries(c.counters ?? {})) if (n > 0) bits.push(`${ct} counter ×${n}`);
    if (c.is_commander) bits.push('commander');
    if (c.is_token) bits.push('token');
    if (c.tapped) bits.push('tapped');
    if (c.summoning_sick) bits.push('summoning sickness');
    if (c.attached_to) bits.push(`attached to object ${c.attached_to}`);
    if (c.keywords?.length) bits.push(c.keywords.join(', '));
    const type = typeLine(c);
    return bits.length ? `${type}\n${bits.join(' · ')}` : type;
  }

  /** No image for a hidden card or a token (nothing to fetch); the caption still shows. */
  const tooltipName = $derived(hidden || card?.is_token ? null : (name || null));

  /** The picture to fetch: none for a chip, a token or a hidden card. */
  const imageUrl = $derived(size === 'chip' || hidden || card?.is_token ? null : scryfallSmallUrl(name));

  // Per-URL load state. `{#key imageUrl}` below remounts the <img> when the
  // name changes, which is what resets these without an effect.
  let loaded = $state(false);
  let failed = $state(false);

  const counters = $derived(
    Object.entries(card?.counters ?? {})
      .filter(([, n]) => n > 0)
      .map(([ct, n]) => `${ct}${n > 1 ? `×${n}` : ''}`),
  );

  const primaryType = $derived.by(() => {
    const t = card?.card_types ?? [];
    for (const k of ['Creature', 'Planeswalker', 'Instant', 'Sorcery', 'Artifact', 'Enchantment', 'Land']) {
      if (t.includes(k)) return k.toLowerCase();
    }
    return 'other';
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="card size-{size} type-{primaryType}"
  class:hidden
  class:offered
  class:tapped={card?.tapped}
  class:sick={card?.summoning_sick}
  class:clickable={onClick !== null && !hidden}
  class:is-commander={card?.is_commander}
  class:is-token={card?.is_token}
  class:has-image={loaded && !failed}
  onclick={() => (!hidden ? onClick?.(card) : null)}
  use:cardTooltip={{ name: tooltipName, caption: caption(card) }}
>
  {#if hidden}
    <span class="back">?</span>
  {:else}
    <!-- The chip: always rendered, under the image once it arrives. -->
    <span class="chip-name">{name}</span>
    {#if size !== 'chip'}
      <span class="chip-type">{(card.card_types ?? []).join(' ')}</span>
    {/if}

    {#if imageUrl}
      {#key imageUrl}
        <img
          class="art"
          src={imageUrl}
          alt=""
          loading="lazy"
          decoding="async"
          draggable="false"
          onload={() => (loaded = true)}
          onerror={() => (failed = true)}
        />
      {/key}
    {/if}

    {#if size !== 'chip'}
      <span class="band">{name}</span>
      <span class="tags">
        {#if card.tapped}<span class="tag tap">T</span>{/if}
        {#if card.summoning_sick}<span class="tag sick">S</span>{/if}
        {#if card.is_commander}<span class="tag cmd">C</span>{/if}
        {#if card.is_token}<span class="tag tok">tok</span>{/if}
        {#if card.attached_to}<span class="tag att">⇗</span>{/if}
        {#each counters as c (c)}<span class="tag ctr">{c}</span>{/each}
      </span>
      {#if isPerm && card.power != null && card.toughness != null}
        <span class="pt" class:damaged={card.damage_marked > 0}>{card.power}/{card.toughness}</span>
      {/if}
      {#if count > 1}<span class="count">×{count}</span>{/if}
    {:else if count > 1}
      <span class="chip-count">×{count}</span>
    {/if}
  {/if}
</div>

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    flex: 0 0 auto;
    padding: 0.2rem 0.25rem;
    border-radius: 5px;
    border: 1px solid #33334f;
    background: #1a1a30;
    font-family: monospace;
    overflow: hidden;
    cursor: default;
    user-select: none;
  }

  /* Scryfall `small` is 146×204 (0.716). Each box keeps that ratio. */
  .size-hand { width: 84px; height: 117px; }
  .size-perm { width: 92px; height: 128px; }
  .size-small { width: 48px; height: 67px; }

  .size-chip {
    width: auto;
    max-width: 14rem;
    height: 1.35rem;
    flex-direction: row;
    align-items: center;
    padding: 0 0.35rem;
    gap: 0.2rem;
  }

  .card.clickable { cursor: pointer; }
  .card.clickable:hover { border-color: #99a; }

  /* ── The chip (fallback) ── */
  .chip-name {
    color: #dde;
    font-size: 0.62rem;
    font-weight: bold;
    line-height: 1.15;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    word-break: break-word;
  }

  .size-chip .chip-name {
    font-size: 0.72rem;
    display: block;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .size-small .chip-name { font-size: 0.52rem; -webkit-line-clamp: 2; }

  .chip-type {
    color: #889;
    font-size: 0.54rem;
    margin-top: auto;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .size-small .chip-type { display: none; }

  .chip-count { color: #8c8; font-size: 0.7rem; }

  /* ── The image, over the chip once loaded ── */
  /* `opacity`, never `display: none`: a lazy image with no box is never in
     the viewport, so the browser never fetches it — the whole surface stayed
     on chips the first time this was tried. */
  .art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    pointer-events: none;
  }

  .card.has-image .art { opacity: 1; }
  .card.has-image .chip-name,
  .card.has-image .chip-type { visibility: hidden; }

  /* ── Overlays ── */
  .band {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    padding: 1px 3px;
    background: rgba(8, 8, 16, 0.82);
    color: #eef;
    font-size: 0.56rem;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: none;
  }

  .card.has-image .band { display: block; }
  .size-small .band { font-size: 0.48rem; padding: 0 2px; }

  .tags {
    position: absolute;
    top: 2px;
    left: 2px;
    display: flex;
    flex-wrap: wrap;
    gap: 1px;
    max-width: calc(100% - 4px);
  }

  .tag {
    padding: 0 0.2rem;
    border-radius: 2px;
    font-size: 0.52rem;
    font-weight: bold;
    line-height: 1.3;
  }

  .tap { background: #3a2800; color: #fc3; }
  .sick { background: #1a3a1a; color: #7c7; }
  .cmd { background: #4a3000; color: #fa0; }
  .tok { background: #1a1a40; color: #88a; }
  .att { background: #1a2040; color: #88c; }
  .ctr { background: #2a2a40; color: #aac; }

  .pt {
    position: absolute;
    right: 2px;
    bottom: 13px;
    padding: 0 0.2rem;
    border-radius: 2px;
    background: rgba(8, 8, 16, 0.85);
    color: #aef;
    font-size: 0.6rem;
    font-weight: bold;
  }

  .size-small .pt { bottom: 10px; font-size: 0.5rem; }
  .pt.damaged { color: #f84; }

  .count {
    position: absolute;
    top: 2px;
    right: 2px;
    padding: 0 0.25rem;
    border-radius: 2px;
    background: #1a3a1a;
    color: #9f9;
    font-size: 0.6rem;
    font-weight: bold;
  }

  /* Type tints for the chip (invisible once the image covers them). */
  .type-creature { border-color: #2a4a30; background: #131d18; }
  .type-instant { border-color: #1a3a6a; background: #10182a; }
  .type-sorcery { border-color: #3a1a6a; background: #181028; }
  .type-enchantment { border-color: #3a2a5a; background: #1a1428; }
  .type-artifact { border-color: #3a3a4a; background: #18181e; }
  .type-land { border-color: #2a3a1a; background: #141a10; }
  .type-planeswalker { border-color: #5a2a2a; background: #241414; }

  /* ── States ── */
  .card.tapped { border-color: #7a6020; }
  .card.tapped .art { filter: grayscale(0.6) brightness(0.55); }
  .card.tapped .chip-name { color: #998; }
  .card.sick { border-style: dashed; }
  .card.is-commander { border-color: #c08a20; }
  .card.is-token { border-style: dotted; }

  /* The current decision offers an action for this card. */
  .card.offered {
    border-color: #5f5;
    box-shadow: 0 0 0 2px #2f7a2f, 0 0 10px rgba(110, 230, 130, 0.6);
  }

  .card.hidden {
    align-items: center;
    justify-content: center;
    background: repeating-linear-gradient(135deg, #1c1c2c 0 4px, #20203a 4px 8px);
    border-color: #2a2a3a;
  }

  .back { color: #556; font-size: 1rem; }
</style>
