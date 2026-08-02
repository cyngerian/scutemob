<script>
  /**
   * EventFeed — the rendered, already-redacted history lines, grouped by turn and
   * filterable by tier.
   *
   * M11-local Session 6 (`memory/m11-session-plan.md` §4, item 4); redesigned by
   * UI-3 (`scutemob-180`, AC 6007).
   *
   * Props:
   *   events (EventView[]) — `{ kind, tier, text, player, seq }`, oldest first
   *
   * # There is nothing to format here
   *
   * `EventView` arrives rendered *and* redacted: `api.rs::seat_view` builds every
   * line through `event_view_for(.., Viewer::Seat(human))`, Architecture
   * Invariant 7's chokepoint #2. `text` is the line; `kind` is the serde
   * discriminant of the source `GameEvent`, carrying no payload; `tier` is that
   * variant's classification, decided in the same match that wrote the sentence.
   * This component therefore deliberately does **not** use the replay viewer's
   * `$viewer/eventFormat.js` — that formats raw `GameEvent`s, an entirely
   * different shape, and reaching for it would mean formatting outside the
   * redaction chokepoint.
   *
   * # Why the tiers come from the server and are not matched on `kind` here
   *
   * The playtest complaint ("events is too sparse and not verbose enough") came
   * with a three-tier sketch: player actions, card actions, stack actions. The
   * obvious client-side implementation is a substring match over `kind` — which
   * is precisely what the *tone* function below still does, and precisely why
   * that function is only allowed to pick a colour. `GameEvent` has ~141
   * variants; a classification list maintained over here would go stale the day
   * a variant is added, silently, and a filter that silently drops a whole class
   * of event is worse than no filter. `tier` is assigned in
   * `crates/view-model/src/event_view.rs`, in the same `match` arm that renders
   * the sentence, so a new variant cannot get a sentence without also getting a
   * tier.
   *
   * A line whose `tier` is missing (an older server, or a variant the classifier
   * has not been taught) is treated as `'game'` and is **never hidden** by a
   * filter — see `tierOf`.
   */
  const { events = [] } = $props();

  /** The scroll container. */
  let box = $state(null);

  /**
   * Whether new lines should pull the view down.
   *
   * Only auto-scroll when the user is already at the bottom: a feed that yanks
   * itself away mid-read is unusable when the bots emit twenty lines per request,
   * which is the normal case here.
   */
  let stick = $state(true);

  /** Within this many pixels of the bottom still counts as "at the bottom". */
  const STICK_SLACK_PX = 24;

  function onScroll() {
    if (!box) return;
    stick = box.scrollHeight - box.scrollTop - box.clientHeight <= STICK_SLACK_PX;
  }

  /**
   * The tier filter. All four on by default: the complaint was that the feed
   * showed too *little*, so the out-of-the-box state must be everything, with
   * the filter there for when a long combat turn makes the priority chatter
   * drown the interesting lines.
   */
  const TIERS = [
    { key: 'game', label: 'turn', title: 'Turn structure, steps, and the end of the game' },
    { key: 'player', label: 'players', title: 'What each player did: passes, draws, lands, attacks, taps' },
    { key: 'card', label: 'cards', title: 'What happened to cards: ETB, deaths, exiles, counters, damage' },
    { key: 'stack', label: 'stack', title: 'Casts, activations, triggers, resolutions and counters' },
  ];

  let enabled = $state(new Set(TIERS.map((t) => t.key)));

  function toggleTier(key) {
    const next = new Set(enabled);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    enabled = next;
  }

  function showAll() {
    enabled = new Set(TIERS.map((t) => t.key));
  }

  function onlyTier(key) {
    enabled = new Set([key]);
  }

  const KNOWN_TIERS = new Set(TIERS.map((t) => t.key));

  /**
   * A line's tier, defaulting to `'game'`.
   *
   * An unrecognised value maps to `'game'` rather than to itself, so a line can
   * never fall outside every filter chip and become permanently invisible with
   * no control that brings it back. That is the failure mode a filter must not
   * have: "the feed is missing something and there is no way to ask for it" is
   * strictly worse than the sparse feed this replaced.
   */
  function tierOf(ev) {
    const t = ev?.tier;
    return KNOWN_TIERS.has(t) ? t : 'game';
  }

  /** Per-tier totals for the chip badges — computed over ALL lines, not the filtered ones. */
  const counts = $derived.by(() => {
    const c = { game: 0, player: 0, card: 0, stack: 0 };
    for (const ev of events) c[tierOf(ev)] += 1;
    return c;
  });

  const visible = $derived(events.filter((ev) => enabled.has(tierOf(ev))));

  /**
   * Group the lines into turn sections.
   *
   * The boundary is a `TurnStarted` line, which is the only event that
   * unambiguously opens a turn (`event_view_for` renders it "Turn N — name").
   * Lines that arrive before the first one — the pregame deal — go into a
   * leading section with a null heading rather than being attached to turn 1,
   * because they did not happen in turn 1.
   *
   * **Boundaries come from the unfiltered list, contents from the filtered
   * one**, and the split matters: `TurnStarted` is itself a `game`-tier event,
   * so deriving the boundaries from `visible` would make every turn heading
   * disappear the moment someone unticked "turn" — collapsing the whole history
   * into one undivided scroll exactly when they were trying to make it easier to
   * read. A heading is structure, not a line.
   *
   * A section whose visible lines are all filtered out is dropped, so the filter
   * never leaves a column of empty headings.
   */
  const sections = $derived.by(() => {
    const out = [];
    let current = null;
    for (const ev of events) {
      if (ev.kind === 'TurnStarted') {
        current = { key: `turn-${ev.seq}`, heading: ev.text, lines: [] };
        out.push(current);
        continue;
      }
      if (current === null) {
        current = { key: 'pre', heading: null, lines: [] };
        out.push(current);
      }
      if (enabled.has(tierOf(ev))) current.lines.push(ev);
    }
    return out.filter((s) => s.lines.length > 0);
  });

  /** Turn headings the user has collapsed, by section key. */
  let collapsed = $state(new Set());

  function toggleSection(key) {
    const next = new Set(collapsed);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    collapsed = next;
  }

  $effect(() => {
    // This read is the whole point of the statement: `$effect` re-runs when
    // something it read reactively changes, and nothing else in the body touches
    // `events`. Do not "clean up" the unused binding — deleting it silently kills
    // the auto-scroll. `$effect` runs after the DOM update, so `scrollHeight`
    // below already includes the new lines.
    void events.length;
    void visible.length;
    if (stick && box) {
      box.scrollTop = box.scrollHeight;
    }
  });

  /**
   * Coarse severity class from the event discriminant, so a loss or an
   * invariant-shaped line reads differently from a routine draw. Substring
   * matching rather than an enumerated list: `GameEvent` has hundreds of
   * variants and a list here would go stale silently.
   *
   * This is allowed to be approximate **because it only picks a colour**. The
   * same technique is explicitly not used for `tier`, which decides whether a
   * line is shown at all — see the header.
   */
  function tone(kind) {
    const k = kind ?? '';
    if (k.includes('Lost') || k.includes('Conceded') || k.includes('Won')) return 'critical';
    if (k.includes('Damage') || k.includes('Destroyed') || k.includes('Died')) return 'combat';
    if (k.includes('Cast') || k.includes('Resolved') || k.includes('Activated')) return 'spell';
    return 'plain';
  }
</script>

<div class="event-feed">
  <div class="feed-header">
    <span class="feed-label">Events</span>
    <span class="feed-count">
      {visible.length === events.length ? events.length : `${visible.length}/${events.length}`}
    </span>
  </div>

  <!--
    Tier chips. Click toggles; shift-click (or the "only" affordance in the
    title) narrows to one. All four are on by default — see `enabled`.
  -->
  <div class="tier-bar">
    {#each TIERS as t (t.key)}
      <button
        class="tier-chip tier-{t.key}"
        class:off={!enabled.has(t.key)}
        title="{t.title} — shift-click to show only this"
        onclick={(e) => (e.shiftKey ? onlyTier(t.key) : toggleTier(t.key))}
      >
        {t.label}
        <span class="tier-count">{counts[t.key]}</span>
      </button>
    {/each}
    {#if enabled.size < TIERS.length}
      <button class="tier-all" onclick={showAll} title="Show every tier again">all</button>
    {/if}
  </div>

  <!-- Newest at the bottom: the journal is already oldest-first. -->
  <div class="feed-lines" bind:this={box} onscroll={onScroll}>
    {#if events.length === 0}
      <div class="feed-empty">— nothing yet —</div>
    {:else if visible.length === 0}
      <div class="feed-empty">— every tier is filtered out —</div>
    {:else}
      {#each sections as section (section.key)}
        <div class="feed-section">
          {#if section.heading}
            <button
              class="section-heading"
              onclick={() => toggleSection(section.key)}
              title={collapsed.has(section.key) ? 'Expand' : 'Collapse'}
            >
              <span class="section-caret">{collapsed.has(section.key) ? '▸' : '▾'}</span>
              {section.heading}
              <span class="section-count">{section.lines.length}</span>
            </button>
          {/if}

          {#if !collapsed.has(section.key)}
            <!--
              Keyed on the monotonic `seq` `stores.js::applySeatView` stamps on
              append, not on the array index: the feed is a front-truncating
              window, so once the cap engages every index shifts and an index key
              re-keys the whole list on each response. `?? i` is the floor for a
              caller that passes unstamped lines (nothing does today).
            -->
            {#each section.lines as ev, i (ev.seq ?? i)}
              <div class="feed-line tone-{tone(ev.kind)} tier-{tierOf(ev)}">
                <div class="line-text">{ev.text}</div>
                <div class="line-meta">
                  {#if ev.player}<span class="line-player">{ev.player}</span>{/if}
                  <span class="line-kind">{ev.kind}</span>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  {#if !stick}
    <button class="jump" onclick={() => { stick = true; if (box) box.scrollTop = box.scrollHeight; }}>
      ↓ jump to latest
    </button>
  {/if}
</div>

<style>
  .event-feed {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    width: 100%;
    background: #111120;
    border: 1px solid #222238;
    border-radius: 4px;
    font-family: monospace;
  }

  .feed-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 0.3rem 0.5rem;
    border-bottom: 1px solid #222238;
  }

  .feed-label {
    font-size: 0.75rem;
    color: #88a;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .feed-count {
    font-size: 0.7rem;
    color: #556;
  }

  /* Tier filter chips (UI-3, AC 6007). */
  .tier-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.2rem;
    padding: 0.25rem 0.4rem;
    border-bottom: 1px solid #1c1c30;
  }

  .tier-chip {
    display: flex;
    align-items: baseline;
    gap: 0.2rem;
    padding: 0.05rem 0.3rem;
    font-family: monospace;
    font-size: 0.66rem;
    border-radius: 3px;
    cursor: pointer;
    border: 1px solid #2a2a48;
    background: #16162c;
    color: #aab;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .tier-chip.off {
    opacity: 0.35;
    background: #101020;
  }

  .tier-chip.tier-game   { border-color: #3a3a5a; color: #99b; }
  .tier-chip.tier-player { border-color: #2a4a6a; color: #8bd; }
  .tier-chip.tier-card   { border-color: #4a3a2a; color: #db9; }
  .tier-chip.tier-stack  { border-color: #4a2a6a; color: #b9e; }

  .tier-count {
    color: #556;
    font-size: 0.62rem;
  }

  .tier-all {
    padding: 0.05rem 0.3rem;
    font-family: monospace;
    font-size: 0.66rem;
    border-radius: 3px;
    cursor: pointer;
    border: 1px solid #2a4a3a;
    background: #14241c;
    color: #8c9;
  }

  .feed-lines {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0.25rem 0.4rem;
  }

  .feed-empty {
    color: #445;
    font-size: 0.75rem;
    padding: 0.4rem 0.1rem;
  }

  .feed-section {
    margin-bottom: 0.2rem;
  }

  .section-heading {
    display: flex;
    align-items: baseline;
    gap: 0.3rem;
    width: 100%;
    text-align: left;
    padding: 0.15rem 0.2rem;
    margin: 0.25rem 0 0.1rem;
    background: #16162c;
    border: none;
    border-left: 2px solid #3a3a6a;
    border-radius: 2px;
    color: #99b;
    font-family: monospace;
    font-size: 0.7rem;
    font-weight: bold;
    cursor: pointer;
    letter-spacing: 0.03em;
  }

  .section-heading:hover {
    background: #1e1e3a;
  }

  .section-caret {
    color: #556;
  }

  .section-count {
    margin-left: auto;
    color: #445;
    font-weight: normal;
    font-size: 0.64rem;
  }

  .feed-line {
    padding: 0.15rem 0.25rem;
    border-left: 2px solid transparent;
    margin-bottom: 0.1rem;
  }

  .line-text {
    font-size: 0.78rem;
    color: #ccd;
    word-break: break-word;
  }

  .line-meta {
    display: flex;
    gap: 0.35rem;
    align-items: baseline;
  }

  .line-player {
    font-size: 0.65rem;
    color: #668;
  }

  .line-kind {
    font-size: 0.62rem;
    color: #445;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  /*
    Tier is a left rail; tone is the text colour. They are different questions
    ("what kind of thing is this" vs "how bad is it") and a line answers both.
  */
  .feed-line.tier-player { border-left-color: #2a4a6a; }
  .feed-line.tier-card   { border-left-color: #4a3a2a; }
  .feed-line.tier-stack  { border-left-color: #4a2a6a; }
  .feed-line.tier-game   { border-left-color: #2a2a44; }

  .tone-critical .line-text {
    color: #f99;
  }

  .tone-combat .line-text {
    color: #eb9;
  }

  .tone-spell .line-text {
    color: #bcf;
  }

  .jump {
    margin: 0.2rem;
    padding: 0.2rem 0.4rem;
    font-size: 0.7rem;
    background: #1a1a38;
    color: #aab;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
  }

  .jump:hover {
    background: #23234a;
  }
</style>
