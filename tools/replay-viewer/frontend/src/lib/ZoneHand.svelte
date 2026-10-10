<script>
  /**
   * ZoneHand — horizontal list of cards in a player's hand.
   *
   * In the replay viewer this is a dev tool and all hands are visible. Since
   * M11-local Session 6 it is ALSO rendered by `tools/play-server/frontend`
   * against a **seat-redacted** view, where another player's hand is a row of
   * anonymous placeholders. See `eachKey` below — that difference is load-bearing.
   *
   * Props:
   *   cards (CardInZoneView[]) — cards in this player's hand
   *   playerName (string) — player label for the zone header
   */
  import { cardTooltip, zoneCaption } from './cardTooltip.js';
  /**
   * `offeredIds` (Set<number>|null) — play-surface opt-in (`scutemob-260`):
   * the `object_id`s the current decision offers an action for. A card in the
   * set is marked so the human can see what is playable before clicking. The
   * replay viewer does not pass it (there is no decision there), and `null`
   * renders exactly what it did before.
   */
  const { cards = [], playerName, onCardClick = null, offeredIds = null } = $props();

  function isOffered(card) {
    return !!offeredIds && !card?.hidden && offeredIds.has(Number(card?.object_id));
  }

  /**
   * Keyed-`#each` key. **Not** `card.object_id` alone, and the reason is a crash,
   * not a preference.
   *
   * `mtg_view_model`'s `redact::redact_hands` replaces every card of a hand the
   * viewer may not read with `redact::hidden_placeholder()`, which sets
   * `object_id: 0` — the id itself is a handle onto a hidden object and is not
   * the viewer's to hold. So a redacted 7-card hand is seven entries with the
   * *same* key, and Svelte 5's keyed reconciler is not lenient about that: it
   * evaluates `length > keys.size` and calls `each_key_duplicate`, which
   * **throws in production as well as in DEV**
   * (`svelte/src/internal/client/dom/blocks/each.js`, `errors.js`). With no
   * `<svelte:boundary>` above it the throw escapes the effect flush and takes the
   * whole mount down, so the play surface rendered nothing at all against a real
   * 4-player payload. Measured on one: `Bot-2`/`Bot-3`/`Bot-4` each had
   * `length 7, keys.size 1`; the seat's own hand had `length 7, keys.size 7`.
   *
   * Keying on `hidden` rather than on `object_id === 0` is deliberate: `hidden`
   * is the flag the redactor actually sets, whereas 0 is a sentinel value that
   * happens not to collide with a real id today.
   *
   * Inert for the replay viewer, which is omniscient and never sets `hidden` on a
   * hand card — every key there is still the object id, unchanged.
   */
  function eachKey(card, i) {
    return card?.hidden ? `hidden-${i}` : card?.object_id;
  }

  function primaryType(cardTypes) {
    if (!cardTypes?.length) return 'unknown';
    if (cardTypes.includes('Creature')) return 'creature';
    if (cardTypes.includes('Instant')) return 'instant';
    if (cardTypes.includes('Sorcery')) return 'sorcery';
    if (cardTypes.includes('Enchantment')) return 'enchantment';
    if (cardTypes.includes('Artifact')) return 'artifact';
    if (cardTypes.includes('Land')) return 'land';
    if (cardTypes.includes('Planeswalker')) return 'planeswalker';
    return 'other';
  }
</script>

<div class="zone-hand">
  <div class="zone-header">
    <span class="zone-label">Hand</span>
    <span class="zone-count muted">{cards.length} card{cards.length !== 1 ? 's' : ''}</span>
  </div>

  {#if cards.length === 0}
    <div class="empty-zone muted">— empty —</div>
  {:else}
    <div class="hand-cards">
      {#each cards as card, i (eachKey(card, i))}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="hand-card card-type-{primaryType(card.card_types)}"
          class:clickable={onCardClick !== null}
          class:offered={isOffered(card)}
          onclick={() => onCardClick?.(card)}
          use:cardTooltip={{ name: card.hidden ? null : card.name, caption: zoneCaption(card) }}
        >
          <span class="card-name">{card.name}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .zone-hand {
    background: #141420;
    border: 1px solid #222238;
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    font-family: monospace;
    font-size: 0.78rem;
  }

  .zone-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.3rem;
  }

  .zone-label {
    color: #88a;
    font-weight: bold;
    font-size: 0.78rem;
  }

  .muted {
    color: #445;
    font-size: 0.72rem;
  }

  .empty-zone {
    padding: 0.25rem 0;
    font-size: 0.72rem;
  }

  .hand-cards {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .hand-card {
    padding: 0.15rem 0.35rem;
    border-radius: 3px;
    border: 1px solid #333;
    cursor: default;
    font-size: 0.72rem;
    background: #1a1a30;
    transition: border-color 0.1s;
  }

  .hand-card:hover {
    border-color: #556;
  }

  .hand-card.clickable {
    cursor: pointer;
  }

  .hand-card.clickable:hover {
    border-color: #888;
    background: #222240;
  }

  .card-name {
    color: #ccce;
    white-space: nowrap;
  }

  /*
    A card the current decision offers an action for. Two classes so it wins
    over the single-class type tints below regardless of source order; the
    green is the same family as the battlefield's own "offered" ring.
  */
  .hand-card.offered {
    border-color: #5c5;
    box-shadow: 0 0 0 1px #2a5a2a, 0 0 7px rgba(110, 230, 130, 0.35);
  }

  .hand-card.offered .card-name {
    color: #efe;
  }

  .hand-card.offered.clickable:hover {
    border-color: #9f9;
  }

  /* Type-based background tints */
  .card-type-creature    { border-color: #2a4a30; background: #131d18; }
  .card-type-instant     { border-color: #1a3a6a; background: #10182a; }
  .card-type-sorcery     { border-color: #3a1a6a; background: #181028; }
  .card-type-enchantment { border-color: #3a2a5a; background: #1a1428; }
  .card-type-artifact    { border-color: #3a3a4a; background: #18181e; }
  .card-type-land        { border-color: #2a3a1a; background: #141a10; }
  .card-type-planeswalker{ border-color: #5a2a2a; background: #241414; }
</style>
