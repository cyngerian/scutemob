<script>
  /**
   * CardFace — one card on the v3 surface (`docs/play-surface-v3-brief.md` §1 rule 6).
   *
   * The DOM and class names are the mockup's `cardEl`, so the ported
   * stylesheet (`v3.css`) applies unchanged: a fixed box sized by the board's
   * `--card`, the Scryfall image over a text chip, the name band on top, P/T
   * bottom-right, the stack count, the commander star, counters. Tapped is the
   * whole-card tilt (no "T" tag).
   *
   * The chip is always rendered and stays until the image element has loaded;
   * if the image fails the chip is what the player sees, in the same box. A
   * token has no printing to fetch, and a hidden card is a card back — drawn
   * that way because the seat view sent `hidden: true`, never decided here
   * (Invariant 7).
   *
   * Props:
   *   card           — `PermanentView` | `CardInZoneView` | `{ name }`; `{ hidden: true }` or null is a back
   *   size           — 'perm' | 'hand' | 'small' | 'mini'
   *   count          — >1 draws "×N" (a land stack)
   *   lit / targeted — offered by the current decision / targeted by something on the stack
   *
   * A creature the seat view marks `summoning_sick` (and that has no haste) carries
   * the `sick` outline (brief §2 item 4). Both facts are the view's own fields.
   *   pt             — a "p/t" string for a card that carries no power/toughness of its own
   *   targetable     — false for a copy that is not on the table (a decision-panel item): it never
   *                    carries the legal-target highlight
   *   onclick        — `(card, event)`; null leaves the click to whatever is underneath
   *   oncontextmenu  — `(event, card)`
   */
  import { scryfallSmallUrl } from '../v2/images.js';
  import { hoverCard } from './hover.js';
  import { targeting } from './targeting.svelte.js';

  let { card, size = 'perm', count = 0, lit = false, targeted = false, pt = null, targetable = true, onclick = null, oncontextmenu = null } = $props();

  const hidden = $derived(!card || card.hidden === true);
  const name = $derived(hidden ? '' : (card.name ?? ''));
  const isToken = $derived(!hidden && card.is_token === true);

  /** Nothing to fetch, and nothing to magnify, for a hidden card or a token. */
  const url = $derived(hidden || isToken ? null : scryfallSmallUrl(name));
  const hoverName = $derived(hidden || isToken ? null : name || null);

  // Load state is remembered per URL, so a card whose name changes starts over on its chip.
  let loadedUrl = $state(null);
  let failedUrl = $state(null);
  const hasImage = $derived(url !== null && loadedUrl === url && failedUrl !== url);

  const ptText = $derived.by(() => {
    if (hidden) return null;
    if (card.power != null && card.toughness != null) return `${card.power}/${card.toughness}`;
    return pt;
  });

  /** Counter keys are the view model's labels (`format_counter_type`): "+1/+1", "loyalty", "charge", … */
  const isLoyalty = (ct) => ct.toLowerCase() === 'loyalty';
  const counterEntries = $derived(hidden ? [] : Object.entries(card.counters ?? {}).filter(([, n]) => n > 0));
  const loyalty = $derived(counterEntries.find(([ct]) => isLoyalty(ct))?.[1] ?? 0);
  const counterText = $derived(
    counterEntries
      .filter(([ct]) => !isLoyalty(ct))
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([ct, n]) => `${ct} ×${n}`)
      .join(' · '),
  );

  /** CR 302.6: only a creature is held back by it, and haste lifts it. */
  const sick = $derived(
    !hidden &&
      card.summoning_sick === true &&
      (card.card_types ?? []).includes('Creature') &&
      !(card.keywords ?? []).some((k) => String(k).toLowerCase() === 'haste'),
  );

  /**
   * A legal target of the arrow being dragged or of the open target stage, and
   * one already chosen there (C2 item 1). Both are the decision payload's own
   * candidate ids, looked up — nothing about legality is decided here.
   */
  const targetOk = $derived(targetable && !hidden && targeting.okObjects.has(Number(card.object_id)));
  const targetChosen = $derived(targetable && !hidden && targeting.chosenObjects.has(Number(card.object_id)));

  const typeLine = $derived(hidden ? '' : [isToken ? 'token' : null, ...(card.card_types ?? [])].filter(Boolean).join(' '));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="card {size}"
  class:back={hidden}
  class:tapped={!hidden && card.tapped}
  class:lit
  class:targeted
  class:sick
  class:target-ok={targetOk}
  class:target-chosen={targetChosen}
  class:cmd={!hidden && card.is_commander}
  class:token={isToken}
  class:has-image={hasImage}
  class:clickable={onclick !== null}
  data-name={hidden ? undefined : name}
  data-oid={hidden ? undefined : card.object_id}
  use:hoverCard={hoverName}
  onclick={(event) => onclick?.(card, event)}
  oncontextmenu={(event) => oncontextmenu?.(event, card)}
>
  {#if hidden}
    {#if count > 1}<div class="count">×{count}</div>{/if}
  {:else}
    <div class="chip">
      <div class="chip-name">{name}</div>
      {#if typeLine}<div class="chip-type">{typeLine}</div>{/if}
    </div>
    {#if url}
      {#key url}
        <img
          class="art"
          src={url}
          alt=""
          loading="lazy"
          decoding="async"
          draggable="false"
          onload={() => (loadedUrl = url)}
          onerror={() => (failedUrl = url)}
        />
      {/key}
    {/if}
    <div class="band">{name}</div>
    {#if ptText}<div class="pt">{ptText}</div>{/if}
    {#if loyalty > 0}<div class="pt loy">◆ {loyalty}</div>{/if}
    {#if count > 1}<div class="count">×{count}</div>{/if}
    {#if card.is_commander}<div class="tag cmdr">★</div>{/if}
    {#if counterText}<div class="tag ctr">{counterText}</div>{/if}
  {/if}
</div>
