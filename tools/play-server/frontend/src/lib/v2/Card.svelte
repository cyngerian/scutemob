<script>
  /**
   * Card — one card on the v2 surface, as a fixed-size chip (B1) that B2 turns
   * into an image with the chip as its fallback (`docs/play-surface-v2-brief.md` §4).
   *
   * Fixed size is the point: a chip's box never depends on its content, so a
   * row of chips never changes height when a counter appears or a name is long.
   *
   * Props:
   *   card      — `CardInZoneView` or `PermanentView`
   *   size      — 'hand' | 'perm' | 'small' | 'chip'
   *   offered   — the current decision offers an action for it (lit)
   *   count     — >1 for a stack of fungible lands ("×N")
   *   onClick   — click-through; null renders an inspect-only chip
   */
  import { cardTooltip, zoneCaption } from '$viewer/cardTooltip.js';

  const { card, size = 'perm', offered = false, count = 1, onClick = null } = $props();

  const hidden = $derived(card?.hidden === true);
  const isPerm = $derived(card && 'tapped' in card);

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
  const tooltipName = $derived(hidden || card?.is_token ? null : (card?.name ?? null));

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
  onclick={() => (!hidden ? onClick?.(card) : null)}
  use:cardTooltip={{ name: tooltipName, caption: caption(card) }}
>
  {#if hidden}
    <span class="back">?</span>
  {:else}
    <span class="name">{card.name}{#if count > 1}<span class="count">×{count}</span>{/if}</span>
    {#if isPerm}
      <span class="line2">
        {#if card.power != null && card.toughness != null}
          <span class="pt" class:damaged={card.damage_marked > 0}>{card.power}/{card.toughness}</span>
        {/if}
        {#if card.tapped}<span class="tag tap">T</span>{/if}
        {#if card.summoning_sick}<span class="tag sick">S</span>{/if}
        {#if card.is_commander}<span class="tag cmd">C</span>{/if}
        {#if card.is_token}<span class="tag tok">tok</span>{/if}
        {#if card.attached_to}<span class="tag att">⇗</span>{/if}
        {#each counters as c (c)}<span class="tag ctr">{c}</span>{/each}
      </span>
    {:else}
      <span class="line2 type">{(card.card_types ?? []).join(' ')}</span>
    {/if}
  {/if}
</div>

<style>
  .card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    box-sizing: border-box;
    flex: 0 0 auto;
    padding: 0.2rem 0.3rem;
    border-radius: 4px;
    border: 1px solid #33334f;
    background: #1a1a30;
    font-family: monospace;
    overflow: hidden;
    cursor: default;
    user-select: none;
  }

  .size-perm {
    width: 108px;
    height: 44px;
  }

  .size-hand {
    width: 124px;
    height: 48px;
  }

  .size-small {
    width: 84px;
    height: 36px;
    font-size: 0.9em;
  }

  .size-chip {
    width: auto;
    max-width: 14rem;
    height: 1.35rem;
    flex-direction: row;
    align-items: center;
    padding: 0 0.35rem;
  }

  .size-chip .line2 {
    display: none;
  }

  .card.clickable {
    cursor: pointer;
  }

  .card.clickable:hover {
    border-color: #778;
    background: #222240;
  }

  .name {
    color: #dde;
    font-size: 0.72rem;
    font-weight: bold;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .count {
    color: #8c8;
    margin-left: 0.2rem;
  }

  .line2 {
    display: flex;
    gap: 0.2rem;
    align-items: baseline;
    font-size: 0.62rem;
    color: #889;
    white-space: nowrap;
    overflow: hidden;
  }

  .pt {
    color: #aef;
    font-weight: bold;
    font-size: 0.68rem;
  }

  .pt.damaged {
    color: #f84;
  }

  .tag {
    padding: 0 0.2rem;
    border-radius: 2px;
    font-size: 0.56rem;
    font-weight: bold;
  }

  .tap { background: #3a2800; color: #d90; }
  .sick { background: #1a3a1a; color: #7c7; }
  .cmd { background: #4a3000; color: #fa0; }
  .tok { background: #1a1a40; color: #88a; }
  .att { background: #1a2040; color: #88c; }
  .ctr { background: #2a2a40; color: #aac; }

  /* Type tints (hand and board alike). */
  .type-creature { border-color: #2a4a30; background: #131d18; }
  .type-instant { border-color: #1a3a6a; background: #10182a; }
  .type-sorcery { border-color: #3a1a6a; background: #181028; }
  .type-enchantment { border-color: #3a2a5a; background: #1a1428; }
  .type-artifact { border-color: #3a3a4a; background: #18181e; }
  .type-land { border-color: #2a3a1a; background: #141a10; }
  .type-planeswalker { border-color: #5a2a2a; background: #241414; }

  .card.tapped {
    opacity: 0.7;
    border-style: solid;
    border-color: #5a4a20;
  }

  .card.sick {
    border-style: dashed;
  }

  .card.is-commander {
    border-color: #8a6020;
  }

  .card.is-token {
    border-style: dotted;
  }

  /* The current decision offers an action for this card. */
  .card.offered {
    border-color: #5c5;
    box-shadow: 0 0 0 1px #2a5a2a, 0 0 8px rgba(110, 230, 130, 0.45);
    opacity: 1;
  }

  .card.offered .name {
    color: #efe;
  }

  .card.hidden {
    align-items: center;
    justify-content: center;
    background: #1c1c2c;
    border-color: #2a2a3a;
  }

  .back {
    color: #445;
    font-size: 1rem;
  }
</style>
