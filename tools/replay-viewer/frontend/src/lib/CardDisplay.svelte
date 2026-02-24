<script>
  /**
   * CardDisplay — detail panel for a selected card.
   *
   * Shown as a modal/overlay when clicking any card in any zone.
   * Supports both PermanentView (battlefield) and CardInZoneView (other zones).
   *
   * Props:
   *   card (object|null) — PermanentView or CardInZoneView; null to hide
   *   onClose (function) — called when the panel should be dismissed
   */

  const { card = null, onClose } = $props();

  /** Build the type line: "Supertypes CardTypes — Subtypes" */
  function typeLine(c) {
    if (!c) return '';
    const parts = [];
    if (c.supertypes?.length) parts.push(...c.supertypes);
    if (c.card_types?.length) parts.push(...c.card_types);
    const types = parts.join(' ');
    const subs = (c.subtypes ?? []).join(' ');
    if (subs) return `${types} — ${subs}`;
    return types || (c.card_types ?? []).join(' ');
  }

  /** Format a counter type for display (strip leading/trailing chars) */
  function fmtCounter(ct) {
    return ct;
  }

  const hasCounters = $derived(
    card && card.counters && Object.keys(card.counters).length > 0
  );

  const hasPT = $derived(
    card && card.power !== undefined && card.power !== null &&
    card.toughness !== undefined && card.toughness !== null
  );

  const hasKeywords = $derived(
    card && card.keywords && card.keywords.length > 0
  );

  const hasAttachments = $derived(
    card && card.attachments && card.attachments.length > 0
  );

  /** Close on Escape key */
  function handleKeydown(e) {
    if (e.key === 'Escape') {
      onClose?.();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if card}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="card-display-backdrop" onclick={onClose}></div>

  <!-- Panel -->
  <div class="card-display-panel" role="dialog" aria-modal="true" aria-label={card.name}>
    <div class="panel-header">
      <span class="card-name">{card.name}</span>
      <button class="close-btn" onclick={onClose} title="Close (Esc)">✕</button>
    </div>

    <div class="panel-body">
      <!-- Type line -->
      <div class="field-row">
        <span class="field-label">Type:</span>
        <span class="field-value type-line">{typeLine(card)}</span>
      </div>

      <!-- Controller / owner (battlefield permanents only) -->
      {#if card.controller !== undefined}
        <div class="field-row">
          <span class="field-label">Controller:</span>
          <span class="field-value">{card.controller}</span>
        </div>
      {/if}

      <!-- Power / Toughness -->
      {#if hasPT}
        <div class="field-row">
          <span class="field-label">P/T:</span>
          <span class="field-value pt-value">
            {card.power}/{card.toughness}
            {#if card.damage_marked > 0}
              <span class="damage-marked">(−{card.damage_marked} damage)</span>
            {/if}
          </span>
        </div>
      {/if}

      <!-- Status badges (tapped, summoning sick) -->
      {#if card.tapped !== undefined || card.summoning_sick !== undefined}
        <div class="field-row">
          <span class="field-label">Status:</span>
          <div class="status-badges">
            {#if card.tapped}
              <span class="status-badge badge-tapped">Tapped</span>
            {:else if card.tapped === false}
              <span class="status-badge badge-untapped">Untapped</span>
            {/if}
            {#if card.summoning_sick}
              <span class="status-badge badge-sick">Summoning Sick</span>
            {/if}
            {#if card.is_commander}
              <span class="status-badge badge-commander">Commander</span>
            {/if}
            {#if card.is_token}
              <span class="status-badge badge-token">Token</span>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Keywords -->
      {#if hasKeywords}
        <div class="field-row">
          <span class="field-label">Keywords:</span>
          <div class="keywords-list">
            {#each card.keywords as kw}
              <span class="kw-badge">{kw}</span>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Counters -->
      {#if hasCounters}
        <div class="field-row">
          <span class="field-label">Counters:</span>
          <div class="counters-list">
            {#each Object.entries(card.counters) as [ct, n]}
              {#if n > 0}
                <span class="counter-badge">{fmtCounter(ct)} × {n}</span>
              {/if}
            {/each}
          </div>
        </div>
      {/if}

      <!-- Attached to -->
      {#if card.attached_to}
        <div class="field-row">
          <span class="field-label">Attached to:</span>
          <span class="field-value muted">object #{card.attached_to}</span>
        </div>
      {/if}

      <!-- Attachments -->
      {#if hasAttachments}
        <div class="field-row">
          <span class="field-label">Attachments:</span>
          <div class="attach-list">
            {#each card.attachments as id}
              <span class="attach-badge">#{id}</span>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Object ID (for debugging) -->
      <div class="field-row field-debug">
        <span class="field-label">Object ID:</span>
        <span class="field-value muted">#{card.object_id}</span>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Semi-transparent backdrop — click to close */
  .card-display-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    z-index: 500;
    cursor: pointer;
  }

  /* Floating panel */
  .card-display-panel {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 510;
    width: 340px;
    max-height: 80vh;
    overflow-y: auto;
    background: #111125;
    border: 1px solid #3a3a6a;
    border-radius: 6px;
    box-shadow: 0 6px 30px rgba(0, 0, 0, 0.8);
    font-family: monospace;
    font-size: 0.8rem;
  }

  /* Header */
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.7rem;
    background: #1a1a35;
    border-bottom: 1px solid #2a2a50;
    flex-shrink: 0;
  }

  .card-name {
    font-weight: bold;
    color: #ccddff;
    font-size: 0.9rem;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: #557;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .close-btn:hover {
    background: #2a2a4a;
    color: #aaa;
  }

  /* Body */
  .panel-body {
    padding: 0.5rem 0.7rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  /* Field row */
  .field-row {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .field-label {
    color: #556;
    font-size: 0.72rem;
    min-width: 72px;
    flex-shrink: 0;
    padding-top: 0.1rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .field-value {
    color: #bbc;
    font-size: 0.78rem;
    flex: 1;
    word-break: break-word;
  }

  .type-line {
    color: #99b;
    font-style: italic;
  }

  .pt-value {
    color: #aef;
    font-weight: bold;
    font-size: 0.82rem;
  }

  .damage-marked {
    color: #f84;
    font-size: 0.72rem;
    margin-left: 0.3rem;
    font-weight: normal;
  }

  .muted {
    color: #445;
  }

  /* Status badges */
  .status-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .status-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    font-weight: bold;
  }

  .badge-tapped     { background: #3a2800; color: #d90; border: 1px solid #6a4a00; }
  .badge-untapped   { background: #0e2a0e; color: #5c5; border: 1px solid #1c4a1c; }
  .badge-sick       { background: #1a3a1a; color: #7c7; border: 1px solid #2a5a2a; }
  .badge-commander  { background: #4a3000; color: #fa0; border: 1px solid #8a6000; }
  .badge-token      { background: #1a1a40; color: #88a; border: 1px solid #2a2a60; }

  /* Keywords */
  .keywords-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .kw-badge {
    font-size: 0.68rem;
    padding: 0.1rem 0.3rem;
    background: #1a3040;
    color: #7ac;
    border-radius: 3px;
    border: 1px solid #2a4a60;
  }

  /* Counters */
  .counters-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .counter-badge {
    font-size: 0.68rem;
    padding: 0.1rem 0.3rem;
    background: #1e1e3a;
    color: #99c;
    border-radius: 3px;
    border: 1px solid #2a2a5a;
  }

  /* Attachments */
  .attach-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .attach-badge {
    font-size: 0.68rem;
    padding: 0.1rem 0.25rem;
    background: #1a2040;
    color: #68a;
    border-radius: 3px;
  }

  /* Debug row */
  .field-debug {
    border-top: 1px solid #1a1a2e;
    padding-top: 0.35rem;
    margin-top: 0.1rem;
  }
</style>
