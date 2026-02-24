<script>
  /**
   * EventTimeline — scrollable list of events for the current step.
   *
   * Default view: shows a single collapsed summary entry per step
   * ("PassPriority(p2) — 4 events"). Click to expand and see all
   * individual events formatted as human-readable text.
   *
   * Props:
   *   events (array) — serialized GameEvent values for the current step
   *   scriptAction (object|string) — the script action metadata
   *   stepIndex (number) — current step index (for display)
   *   totalSteps (number) — total steps (for display)
   */
  import { formatEvent, eventCategory, eventVariantName } from './eventFormat.js';

  const { events = [], scriptAction = null, stepIndex = 0, totalSteps = 0 } = $props();

  /** Whether the event list is expanded. */
  let expanded = $state(false);

  function toggleExpanded() {
    expanded = !expanded;
  }

  /** Get the action kind from a serialized ScriptAction. */
  function getActionKind(action) {
    if (!action) return 'unknown';
    if (typeof action === 'string') return action;
    if (typeof action === 'object') {
      const keys = Object.keys(action);
      if (keys.length > 0) return keys[0];
    }
    return String(action);
  }

  /** Build a compact summary of the action. */
  function actionSummary(action) {
    const kind = getActionKind(action);
    const data = typeof action === 'object' ? action[kind] : null;
    if (!data) return kind;

    // Compact summaries for common actions
    switch (kind) {
      case 'PlayerAction': {
        const pa = data?.action ?? data;
        if (typeof pa === 'object') {
          const [paKey, paVal] = Object.entries(pa)[0] ?? [];
          if (paKey && paVal && typeof paVal === 'object') {
            const player = paVal.player ?? paVal.casting_player ?? '';
            return player ? `${paKey}(${player})` : paKey;
          }
          return paKey ?? kind;
        }
        return String(pa);
      }
      case 'AssertState':
        return `AssertState (${data.assertions?.length ?? 0} checks)`;
      default:
        return kind;
    }
  }

  const actionLabel = $derived(actionSummary(scriptAction));
  const eventCount = $derived(events?.length ?? 0);

  /** Category → CSS class name map for color-coded labels. */
  const CAT_CLASS = {
    turn: 'cat-turn',
    priority: 'cat-priority',
    cast: 'cat-cast',
    damage: 'cat-damage',
    life: 'cat-life',
    zone: 'cat-zone',
    combat: 'cat-combat',
    commander: 'cat-commander',
    mana: 'cat-mana',
    system: 'cat-system',
  };

  function catClass(event) {
    return CAT_CLASS[eventCategory(event)] ?? 'cat-system';
  }
</script>

<div class="event-timeline">
  <div class="timeline-header">
    <span class="header-title">Events</span>
    <span class="step-badge">Step {stepIndex}/{totalSteps - 1}</span>
  </div>

  <!-- Collapsed summary -->
  <button
    class="summary-row"
    class:expanded
    onclick={toggleExpanded}
    title={expanded ? 'Click to collapse' : 'Click to expand events'}
  >
    <span class="action-label">{actionLabel}</span>
    <span class="event-count">{eventCount} event{eventCount !== 1 ? 's' : ''}</span>
    <span class="expand-chevron">{expanded ? '▲' : '▼'}</span>
  </button>

  <!-- Expanded event list -->
  {#if expanded}
    <div class="events-list">
      {#if eventCount === 0}
        <div class="no-events muted">No events this step.</div>
      {:else}
        {#each events as event, i (i)}
          <div class="event-row {catClass(event)}">
            <span class="event-index">{i + 1}</span>
            <span class="event-cat-dot" title={eventCategory(event)}></span>
            <span class="event-text">{formatEvent(event)}</span>
          </div>
        {/each}
      {/if}
    </div>
  {/if}

  <!-- Category legend (visible when expanded) -->
  {#if expanded && eventCount > 0}
    <div class="legend">
      <span class="legend-item cat-turn">turn</span>
      <span class="legend-item cat-priority">priority</span>
      <span class="legend-item cat-cast">cast</span>
      <span class="legend-item cat-damage">damage</span>
      <span class="legend-item cat-life">life</span>
      <span class="legend-item cat-zone">zone</span>
      <span class="legend-item cat-combat">combat</span>
      <span class="legend-item cat-commander">cmdr</span>
      <span class="legend-item cat-mana">mana</span>
    </div>
  {/if}
</div>

<style>
  .event-timeline {
    display: flex;
    flex-direction: column;
    background: #0e0e20;
    border: 1px solid #252545;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.78rem;
    overflow: hidden;
  }

  .timeline-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.25rem 0.5rem;
    background: #141430;
    border-bottom: 1px solid #252545;
    flex-shrink: 0;
  }

  .header-title {
    font-weight: bold;
    color: #88a;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .step-badge {
    font-size: 0.68rem;
    color: #556;
  }

  .summary-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.5rem;
    background: transparent;
    border: none;
    border-bottom: 1px solid #1a1a30;
    cursor: pointer;
    text-align: left;
    width: 100%;
    font-family: monospace;
    color: #aaa;
    font-size: 0.78rem;
    transition: background 0.1s;
  }

  .summary-row:hover {
    background: #181830;
  }

  .summary-row.expanded {
    background: #181836;
    border-bottom-color: #303060;
  }

  .action-label {
    flex: 1;
    color: #cce;
    font-weight: bold;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .event-count {
    color: #668;
    font-size: 0.72rem;
    flex-shrink: 0;
  }

  .expand-chevron {
    color: #556;
    font-size: 0.65rem;
    flex-shrink: 0;
  }

  .events-list {
    overflow-y: auto;
    max-height: 400px;
  }

  .no-events {
    padding: 0.5rem;
    text-align: center;
  }

  .muted {
    color: #445;
  }

  .event-row {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
    padding: 0.18rem 0.5rem;
    border-bottom: 1px solid #111128;
    transition: background 0.08s;
  }

  .event-row:hover {
    background: #171730;
  }

  .event-index {
    color: #3a3a55;
    font-size: 0.65rem;
    min-width: 1.4rem;
    text-align: right;
    flex-shrink: 0;
  }

  .event-cat-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 0.2rem;
    background: currentColor;
  }

  .event-text {
    flex: 1;
    word-break: break-word;
    line-height: 1.4;
    color: #bbb;
  }

  /* Category color classes — applied to the whole row */
  .cat-turn        { color: #778899; }
  .cat-turn .event-text    { color: #99aacc; }

  .cat-priority    { color: #aaaa44; }
  .cat-priority .event-text { color: #cccc88; }

  .cat-cast        { color: #4488cc; }
  .cat-cast .event-text    { color: #88bbee; }

  .cat-damage      { color: #cc4444; }
  .cat-damage .event-text  { color: #ee8888; }

  .cat-life        { color: #44cc44; }
  .cat-life .event-text    { color: #88ee88; }

  .cat-zone        { color: #8844cc; }
  .cat-zone .event-text    { color: #aa88ee; }

  .cat-combat      { color: #cc6644; }
  .cat-combat .event-text  { color: #ee9977; }

  .cat-commander   { color: #cc9900; }
  .cat-commander .event-text { color: #eebb44; }

  .cat-mana        { color: #44aacc; }
  .cat-mana .event-text    { color: #77ccee; }

  .cat-system      { color: #445566; }
  .cat-system .event-text  { color: #667788; }

  /* Legend at bottom */
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    padding: 0.25rem 0.5rem;
    border-top: 1px solid #1a1a30;
    background: #0c0c1c;
  }

  .legend-item {
    font-size: 0.6rem;
    padding: 0.05rem 0.25rem;
    border-radius: 2px;
    opacity: 0.8;
    background: #151525;
  }
</style>
