<script>
  /**
   * ZoneBattlefield — permanent grid for one player's battlefield.
   *
   * Props:
   *   permanents (PermanentView[]) — list of permanents controlled by this player
   *   playerName (string) — player label for the zone header
   *   onCardClick (fn(permanent, group)|null) — see `clickStack` below for the
   *     second argument, which is new in UI-5 and which the replay viewer ignores
   *   stackLands (bool) — collapse fungible same-name lands into one chip (G13).
   *     **Off by default**; the play surface opts in, the replay viewer does not.
   *     See "Why `stackLands` is a prop" below.
   *
   * # Why `stackLands` is a prop and not the default (UI-5, `scutemob-190`)
   *
   * This file is imported IN PLACE by both surfaces through `vite.config.js`'s
   * `$viewer` alias. UI-5's standing rule for this batch: **edit the shared file
   * in place; where the two surfaces genuinely want opposite behaviour, express
   * the difference as a prop rather than as a copy.** G11 (the tooltip caption)
   * and G12 (artifacts above lands) are wanted by both surfaces and are therefore
   * unconditional. G13 is not: the replay viewer is a step *debugger* whose whole
   * job is per-object identity across steps — `App.svelte`'s `openCard` opens the
   * object you clicked — and folding five Forests into one chip deletes four of
   * the objects you might be stepping to inspect. A fork of this component would
   * have duplicated 476 lines including the other two fixes and would fork again
   * on the next `PermanentView` field, which is exactly what
   * `PlayBoard.svelte`'s module doc says the *leaf* components must not do.
   *
   * # The G13 grouping key, and why each part of it is there
   *
   * Playtest note: "same-name lands should stack when tapped". The key is
   * `(name, tapped)` **plus every other field this component renders or styles
   * on**, because merging is a claim that the two chips are interchangeable and
   * a chip that shows a counter is not interchangeable with one that does not:
   *
   *   - `tapped` — this is the information the request is *about*. Tapped and
   *     untapped must never merge, or the stack cannot answer "how much mana do
   *     I have left".
   *   - `counters` — rendered as badges in the land block.
   *   - `is_commander` — rendered as a border class (a land can be a commander
   *     under CR 903.3 variants, and Dryad Arbor is a legal one in some pools).
   *   - `is_token`, `summoning_sick`, `damage_marked`, `attached_to` — not all
   *     of them are drawn in the land block today, but each is a real difference
   *     between two permanents, and a chip that merges on them would start
   *     lying the moment the land block learns to draw one.
   *
   * The list is deliberately a superset of what is rendered: the failure mode of
   * a too-narrow key is a silent lie, and of a too-wide key is a chip that does
   * not stack. Only one of those is worth risking.
   */
  import { cardTooltip } from './cardTooltip.js';
  const {
    permanents = [],
    playerName,
    onCardClick = null,
    stackLands = false,
  } = $props();

  // Group permanents by rough type category
  const groups = $derived(() => {
    const creatures = [];
    const lands = [];
    const artifacts = [];
    const enchantments = [];
    const planeswalkers = [];
    const other = [];

    for (const p of permanents) {
      const types = p.card_types ?? [];
      if (types.includes('Creature')) {
        creatures.push(p);
      } else if (types.includes('Land')) {
        // G12 note (`scutemob-190`): this chain is FIRST-MATCH and tests Land
        // before Artifact, so an **artifact land** (Ancient Tomb is not one;
        // Darksteel Citadel, Seat of the Synod, Tree of Tales are) renders under
        // Lands and not with the artifacts. **That is deliberate and is left
        // unchanged.** The playtest request was about reading the board, and a
        // player reads an artifact land as a land: it is the thing you tap for
        // mana, it is subject to the one-land-per-turn rule (CR 305.2), and
        // putting it in the artifact row would take it out of the row where you
        // count your mana. It is still an artifact for every rules purpose —
        // Metalcraft, artifact removal — because nothing here touches
        // `card_types`; only where the chip is drawn. The other defensible
        // answer (move the Artifact test above Land) is a one-line change at
        // this site if a later playtest says otherwise.
        lands.push(p);
      } else if (types.includes('Planeswalker')) {
        planeswalkers.push(p);
      } else if (types.includes('Artifact')) {
        artifacts.push(p);
      } else if (types.includes('Enchantment')) {
        enchantments.push(p);
      } else {
        other.push(p);
      }
    }
    return { creatures, lands, artifacts, enchantments, planeswalkers, other };
  });

  function typeLineStr(p) {
    const parts = [];
    if (p.supertypes?.length) parts.push(...p.supertypes);
    if (p.card_types?.length) parts.push(...p.card_types);
    if (p.subtypes?.length) parts.push('—', ...p.subtypes);
    return parts.join(' ');
  }

  /**
   * The hover caption (G11). Replaces the native `title=` that used to sit on
   * the same element and overdraw the card image — see `cardTooltip.js`.
   *
   * **It absorbs the badges' titles too, and that is not scope creep.** The
   * triage located nine `title=` attributes on the card elements themselves; a
   * `title` on a *descendant* of a tooltip anchor produces the identical
   * collision over a smaller hit area, and the badges (`CMD`, `TAP`, `SICK`,
   * `ATT`, counters, keyword abbreviations) are all descendants of a chip that
   * is only ~70px wide, so hovering one is ordinary rather than exotic. Every
   * one of those titles existed to expand an abbreviation, so folding them into
   * a second caption line loses no information and gains a line you can
   * actually read — a native tooltip is chrome and cannot be styled, selected
   * or kept on screen.
   */
  function captionFor(p) {
    const bits = [];
    if (p.power !== null && p.power !== undefined && p.toughness !== null && p.toughness !== undefined) {
      bits.push(`${p.power}/${p.toughness}`);
    }
    if (p.damage_marked > 0) bits.push(`${p.damage_marked} damage marked`);
    for (const [ct, n] of Object.entries(p.counters ?? {})) {
      if (n > 0) bits.push(`${ct} counter ×${n}`);
    }
    if (p.is_commander) bits.push('commander');
    if (p.is_token) bits.push('token');
    if (p.tapped) bits.push('tapped');
    if (p.summoning_sick) bits.push('summoning sickness');
    if (p.attached_to) bits.push(`attached to object ${p.attached_to}`);
    if (p.keywords?.length) bits.push(p.keywords.join(', '));
    const type = typeLineStr(p);
    return bits.length > 0 ? `${type}\n${bits.join(' · ')}` : type;
  }

  function tooltipArg(p, { image = true } = {}) {
    return { name: image ? p.name : null, caption: captionFor(p) };
  }

  /** Stable identity for a land stack's fungibility key. See the module doc. */
  function landStackKey(p) {
    const counters = Object.entries(p.counters ?? {})
      .filter(([, n]) => n > 0)
      .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
      .map(([ct, n]) => `${ct}=${n}`)
      .join(',');
    return JSON.stringify([
      p.name,
      !!p.tapped,
      counters,
      p.attached_to ?? null,
      !!p.is_commander,
      !!p.is_token,
      !!p.summoning_sick,
      p.damage_marked ?? 0,
    ]);
  }

  /**
   * Lands as they are drawn: either one entry per permanent (`stackLands` off,
   * or every land distinct) or one entry per fungible group.
   *
   * `members` is always the full list, so a stack of one and a stack of five go
   * down the same code path and the click handler has no special case. Insertion
   * order is preserved — a `Map` keyed on the fungibility string — so a stack
   * appears where its first member would have.
   */
  const landStacks = $derived.by(() => {
    const ls = groups().lands;
    if (!stackLands) return ls.map((p) => ({ key: String(p.object_id), members: [p] }));
    const byKey = new Map();
    for (const p of ls) {
      const k = landStackKey(p);
      if (byKey.has(k)) byKey.get(k).push(p);
      else byKey.set(k, [p]);
    }
    // The `#each` key is the fungibility string and NOT the representative's
    // `object_id`: tapping one Forest of five moves that permanent into a
    // different stack, and a key derived from a member that just left would
    // make Svelte destroy and rebuild a chip that only changed its count.
    return [...byKey.entries()].map(([key, members]) => ({ key, members }));
  });

  /**
   * The click path for a stack, decided explicitly (G13 constraint 3).
   *
   * `PlayApp.svelte::actionsForCard` matches a decision's actions by a **single**
   * `object_id`, so a chip standing for five permanents has to nominate one or
   * the click is undefined. It nominates `members[0]` — arbitrary *and
   * immaterial*, because the fungibility key already required every member to be
   * indistinguishable, and because tap state is part of that key, so a stack is
   * either wholly tapped or wholly untapped and "first untapped" collapses to
   * "first".
   *
   * The whole group is passed as a second argument anyway. The caller is the
   * only party that knows which actions the server offered, so it — not this
   * component — is where a representative that happens to carry no offered
   * action can fall through to a sibling that does. The replay viewer's
   * `openCard(card)` takes one parameter and ignores this, unchanged.
   */
  function clickStack(stack) {
    onCardClick?.(stack.members[0], stack.members);
  }
</script>

<div class="zone-battlefield">
  <div class="zone-header">
    <span class="zone-label">Battlefield</span>
    <span class="zone-count muted">{permanents.length} permanents</span>
  </div>

  {#if permanents.length === 0}
    <div class="empty-zone muted">— empty —</div>
  {:else}
    <!-- Creatures -->
    {#if groups().creatures.length > 0}
      <div class="perm-group">
        <div class="group-label">Creatures ({groups().creatures.length})</div>
        <div class="perm-grid">
          {#each groups().creatures as p (p.object_id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="permanent-card"
              class:tapped={p.tapped}
              class:summoning-sick={p.summoning_sick}
              class:is-commander={p.is_commander}
              class:is-token={p.is_token}
              class:clickable={onCardClick !== null}
              onclick={() => onCardClick?.(p, [p])}
              use:cardTooltip={tooltipArg(p, { image: !p.is_token })}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.is_commander}
                <span class="badge badge-cmd">CMD</span>
              {/if}
              {#if p.is_token}
                <span class="badge badge-token">TKN</span>
              {/if}
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
              {#if p.summoning_sick}
                <span class="badge badge-sick">SICK</span>
              {/if}

              <!-- Power/Toughness -->
              {#if p.power !== null && p.toughness !== null}
                <div class="pt-box">
                  <span class="pt-value" class:pt-damaged={p.damage_marked > 0}>
                    {p.power}/{p.toughness}
                    {#if p.damage_marked > 0}
                      <span class="dmg-marker">
                        -{p.damage_marked}
                      </span>
                    {/if}
                  </span>
                </div>
              {/if}

              <!-- Counters -->
              {#each Object.entries(p.counters ?? {}) as [ct, n]}
                {#if n > 0}
                  <span class="counter-badge counter-{ct.replace('/', '').replace('+', 'p').replace('-', 'm')}">
                    {ct}×{n}
                  </span>
                {/if}
              {/each}

              <!-- Keywords (abbreviated) -->
              {#if p.keywords?.length > 0}
                <div class="keyword-list">
                  {#each p.keywords as kw}
                    <span class="kw-badge">{kw.slice(0, 3).toUpperCase()}</span>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Planeswalkers -->
    {#if groups().planeswalkers.length > 0}
      <div class="perm-group">
        <div class="group-label">Planeswalkers ({groups().planeswalkers.length})</div>
        <div class="perm-grid">
          {#each groups().planeswalkers as p (p.object_id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="permanent-card permanent-pw"
              class:tapped={p.tapped}
              class:clickable={onCardClick !== null}
              onclick={() => onCardClick?.(p, [p])}
              use:cardTooltip={tooltipArg(p)}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
              {#each Object.entries(p.counters ?? {}) as [ct, n]}
                {#if n > 0}
                  <span class="counter-badge counter-loyalty">{ct}×{n}</span>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!--
      Artifacts & Enchantments — G12 (`scutemob-190`), from the playtest note
      "artifacts and enchantments should sit above lands".

      **One block moved, and it is Lands, not this one.** Sliding this block up
      above Lands would also have pushed it above Planeswalkers, changing an
      ordering nobody complained about; moving Lands *down* to the end of the
      permanent rows satisfies the request, leaves every other pair in its
      existing relative order, and lands where a paper board already puts them —
      the back row you count mana in, under everything that is doing something.

      The server sends `HashMap<String, Vec<PermanentView>>` with no ordering
      semantics at all (`view-model/src/lib.rs`), so this order is entirely a
      client decision and there is nothing to keep in sync.
    -->
    {#if groups().artifacts.length + groups().enchantments.length > 0}
      <div class="perm-group">
        <div class="group-label">
          Artifacts/Enchantments ({groups().artifacts.length + groups().enchantments.length})
        </div>
        <div class="perm-grid">
          {#each [...groups().artifacts, ...groups().enchantments] as p (p.object_id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="permanent-card"
              class:tapped={p.tapped}
              class:permanent-artifact={p.card_types?.includes('Artifact')}
              class:permanent-enchantment={p.card_types?.includes('Enchantment')}
              class:clickable={onCardClick !== null}
              onclick={() => onCardClick?.(p, [p])}
              use:cardTooltip={tooltipArg(p, { image: !p.is_token })}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
              {#if p.attached_to}
                <span class="badge badge-attached">ATT</span>
              {/if}
              {#each Object.entries(p.counters ?? {}) as [ct, n]}
                {#if n > 0}
                  <span class="counter-badge">{ct}×{n}</span>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!--
      Lands — moved below Artifacts/Enchantments by G12 (see that block's
      comment), and stacked by G13 when `stackLands` is set (see the module doc
      for the fungibility key and the click path).
    -->
    {#if groups().lands.length > 0}
      <div class="perm-group">
        <div class="group-label">Lands ({groups().lands.length})</div>
        <div class="perm-grid perm-grid-lands">
          {#each landStacks as stack (stack.key)}
            {@const p = stack.members[0]}
            {@const n = stack.members.length}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="permanent-card permanent-land"
              class:tapped={p.tapped}
              class:is-commander={p.is_commander}
              class:clickable={onCardClick !== null}
              class:land-stack={n > 1}
              data-land-stack-count={n}
              onclick={() => clickStack(stack)}
              use:cardTooltip={tooltipArg(p)}
            >
              <div class="perm-name">
                {p.name}{#if n > 1}<span class="stack-count">×{n}</span>{/if}
              </div>
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
              {#each Object.entries(p.counters ?? {}) as [ct, c]}
                {#if c > 0}
                  <span class="counter-badge">{ct}×{c}</span>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Other -->
    {#if groups().other.length > 0}
      <div class="perm-group">
        <div class="group-label">Other ({groups().other.length})</div>
        <div class="perm-grid">
          {#each groups().other as p (p.object_id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="permanent-card"
              class:tapped={p.tapped}
              class:clickable={onCardClick !== null}
              onclick={() => onCardClick?.(p, [p])}
              use:cardTooltip={tooltipArg(p, { image: !p.is_token })}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .zone-battlefield {
    background: #0e1a12;
    border: 1px solid #1e3a22;
    border-radius: 4px;
    padding: 0.4rem 0.5rem;
    font-family: monospace;
    font-size: 0.78rem;
  }

  .zone-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.4rem;
    border-bottom: 1px solid #1e3a22;
    padding-bottom: 0.25rem;
  }

  .zone-label {
    color: #4a8;
    font-weight: bold;
    font-size: 0.8rem;
  }

  .muted {
    color: #556;
    font-size: 0.75rem;
  }

  .empty-zone {
    text-align: center;
    padding: 0.5rem;
    font-size: 0.75rem;
  }

  .perm-group {
    margin-bottom: 0.4rem;
  }

  .group-label {
    color: #668;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.2rem;
  }

  .perm-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .perm-grid-lands {
    gap: 0.2rem;
  }

  .permanent-card {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.3rem 0.4rem;
    background: #1a2820;
    border: 1px solid #2a4830;
    border-radius: 3px;
    min-width: 70px;
    max-width: 100px;
    transition: border-color 0.1s;
  }

  .permanent-card:hover {
    border-color: #4a8060;
  }

  .permanent-card.clickable {
    cursor: pointer;
  }

  .permanent-card.clickable:hover {
    border-color: #6aaa80;
    box-shadow: 0 0 4px rgba(100, 200, 130, 0.3);
  }

  .permanent-card.tapped {
    background: #1a2010;
    border-color: #4a4020;
    opacity: 0.8;
  }

  .permanent-card.summoning-sick {
    border-style: dashed;
    border-color: #4a4a20;
  }

  .permanent-card.is-commander {
    border-color: #8a6020;
    background: #221a10;
  }

  .permanent-card.is-token {
    border-style: dotted;
    border-color: #3a3a60;
    background: #14142a;
  }

  .permanent-land {
    min-width: 50px;
    max-width: 70px;
    background: #121a14;
  }

  /* G13: a chip standing for more than one fungible land. */
  .permanent-card.land-stack {
    box-shadow: 2px 2px 0 -1px #2a4830, 4px 4px 0 -2px #2a4830;
  }

  .stack-count {
    color: #8c8;
    font-weight: bold;
    margin-left: 0.15rem;
  }

  .permanent-artifact {
    background: #1a1a22;
    border-color: #3a3a5a;
  }

  .permanent-enchantment {
    background: #1a1224;
    border-color: #3a2a5a;
  }

  .permanent-pw {
    background: #2a1a1a;
    border-color: #5a2a2a;
    min-width: 80px;
  }

  .perm-name {
    color: #ccd;
    font-size: 0.72rem;
    font-weight: bold;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 90px;
  }

  .badge {
    font-size: 0.6rem;
    padding: 0.05rem 0.25rem;
    border-radius: 2px;
    font-weight: bold;
    align-self: flex-start;
  }

  .badge-cmd {
    background: #4a3000;
    color: #fa0;
    border: 1px solid #8a6000;
  }

  .badge-token {
    background: #1a1a40;
    color: #88a;
  }

  .badge-tapped {
    background: #3a2800;
    color: #a80;
  }

  .badge-sick {
    background: #1a3a1a;
    color: #6a6;
  }

  .badge-attached {
    background: #1a2040;
    color: #66a;
  }

  .pt-box {
    margin-top: 0.1rem;
  }

  .pt-value {
    color: #aef;
    font-weight: bold;
    font-size: 0.75rem;
  }

  .pt-value.pt-damaged {
    color: #f84;
  }

  .dmg-marker {
    color: #f44;
    font-size: 0.65rem;
    margin-left: 0.15rem;
  }

  .counter-badge {
    font-size: 0.6rem;
    padding: 0.05rem 0.2rem;
    border-radius: 2px;
    background: #2a2a40;
    color: #aac;
    align-self: flex-start;
  }

  .counter-loyalty {
    background: #1a2a5a;
    color: #8af;
  }

  .keyword-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.1rem;
  }

  .kw-badge {
    font-size: 0.55rem;
    padding: 0.05rem 0.15rem;
    border-radius: 2px;
    background: #1a3a4a;
    color: #68a;
  }
</style>
