<script>
  /**
   * EventFeed — the rendered, already-redacted history lines.
   *
   * M11-local Session 6 (`memory/m11-session-plan.md` §4, item 4).
   *
   * Props:
   *   events (EventView[]) — `{ kind, text, player }`, oldest first
   *
   * # There is nothing to format here
   *
   * `EventView` arrives rendered *and* redacted: `api.rs::seat_view` builds every
   * line through `event_view_for(.., Viewer::Seat(human))`, Architecture
   * Invariant 7's chokepoint #2. `text` is the line; `kind` is the serde
   * discriminant of the source `GameEvent`, carrying no payload. This component
   * therefore deliberately does **not** use the replay viewer's
   * `$viewer/eventFormat.js` — that formats raw `GameEvent`s, an entirely
   * different shape, and reaching for it would mean formatting outside the
   * redaction chokepoint.
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

  $effect(() => {
    // Track the length so the effect re-runs when lines arrive.
    const _count = events.length;
    if (stick && box) {
      box.scrollTop = box.scrollHeight;
    }
  });

  /**
   * Coarse severity class from the event discriminant, so a loss or an
   * invariant-shaped line reads differently from a routine draw. Substring
   * matching rather than an enumerated list: `GameEvent` has hundreds of
   * variants and a list here would go stale silently.
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
    <span class="feed-count">{events.length}</span>
  </div>

  <!-- Newest at the bottom: the journal is already oldest-first. -->
  <div class="feed-lines" bind:this={box} onscroll={onScroll}>
    {#if events.length === 0}
      <div class="feed-empty">— nothing yet —</div>
    {:else}
      {#each events as ev, i (i)}
        <div class="feed-line tone-{tone(ev.kind)}">
          <div class="line-text">{ev.text}</div>
          <div class="line-meta">
            {#if ev.player}<span class="line-player">{ev.player}</span>{/if}
            <span class="line-kind">{ev.kind}</span>
          </div>
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

  .tone-critical {
    border-left-color: #a33;
  }
  .tone-critical .line-text {
    color: #f99;
  }

  .tone-combat {
    border-left-color: #a63;
  }
  .tone-combat .line-text {
    color: #eb9;
  }

  .tone-spell {
    border-left-color: #46a;
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
