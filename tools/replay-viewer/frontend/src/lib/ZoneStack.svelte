<script>
  /**
   * ZoneStack — ordered stack items in LIFO display (top of stack at top).
   *
   * Props:
   *   items (StackItemView[]) — stack items, last element = top of stack
   */
  const { items = [] } = $props();

  // Display in reverse so top of stack appears first
  const displayItems = $derived([...items].reverse());

  function kindLabel(kind) {
    switch (kind) {
      case 'spell':             return 'Spell';
      case 'activated_ability': return 'Activated';
      case 'triggered_ability': return 'Triggered';
      case 'cascade_trigger':   return 'Cascade';
      case 'storm_trigger':     return 'Storm';
      default:                  return kind;
    }
  }

  function kindClass(kind) {
    switch (kind) {
      case 'spell':             return 'kind-spell';
      case 'activated_ability': return 'kind-activated';
      case 'triggered_ability': return 'kind-triggered';
      case 'cascade_trigger':   return 'kind-cascade';
      case 'storm_trigger':     return 'kind-storm';
      default:                  return '';
    }
  }
</script>

<div class="zone-stack">
  <div class="zone-header">
    <span class="zone-label">Stack</span>
    <span class="zone-count muted">{items.length} item{items.length !== 1 ? 's' : ''}</span>
  </div>

  {#if items.length === 0}
    <div class="empty-zone muted">— empty —</div>
  {:else}
    <div class="stack-items">
      {#each displayItems as item, i (item.id)}
        <div class="stack-item" class:is-copy={item.is_copy}>
          <!-- Stack position badge (1 = top) -->
          <span class="stack-pos" title="Stack position (1 = top)">{i + 1}</span>

          <!-- Kind badge -->
          <span class="kind-badge {kindClass(item.kind)}" title="Kind: {item.kind}">
            {kindLabel(item.kind)}
          </span>

          <!-- Source name -->
          <span class="source-name">{item.source_name}</span>

          <!-- Controller -->
          <span class="controller muted">({item.controller})</span>

          <!-- Copy badge -->
          {#if item.is_copy}
            <span class="badge badge-copy" title="Copy">COPY</span>
          {/if}

          <!-- Targets -->
          {#if item.targets?.length > 0}
            <div class="targets">
              <span class="targets-label">→</span>
              {#each item.targets as target}
                <span class="target-badge">{target}</span>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .zone-stack {
    background: #1a1030;
    border: 1px solid #2a2050;
    border-radius: 4px;
    padding: 0.4rem 0.5rem;
    font-family: monospace;
    font-size: 0.78rem;
    min-width: 200px;
  }

  .zone-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.4rem;
    border-bottom: 1px solid #2a2050;
    padding-bottom: 0.25rem;
  }

  .zone-label {
    color: #a8f;
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

  .stack-items {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .stack-item {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.3rem;
    padding: 0.3rem 0.4rem;
    background: #20183a;
    border: 1px solid #32285a;
    border-radius: 3px;
    position: relative;
  }

  /* Slight visual overlap to suggest LIFO depth */
  .stack-item:not(:last-child) {
    margin-bottom: -1px;
    border-bottom-color: #28205a;
  }

  .stack-item.is-copy {
    border-style: dashed;
    border-color: #4a4a8a;
    background: #181830;
  }

  .stack-pos {
    font-size: 0.65rem;
    color: #668;
    min-width: 1rem;
    text-align: center;
    font-weight: bold;
  }

  .kind-badge {
    font-size: 0.65rem;
    padding: 0.05rem 0.3rem;
    border-radius: 2px;
    font-weight: bold;
  }

  .kind-spell {
    background: #1a2a6a;
    color: #8af;
    border: 1px solid #2a3a9a;
  }

  .kind-activated {
    background: #2a1a4a;
    color: #c8a;
    border: 1px solid #4a2a7a;
  }

  .kind-triggered {
    background: #1a3a2a;
    color: #8ca;
    border: 1px solid #2a5a3a;
  }

  .kind-cascade {
    background: #3a2a1a;
    color: #ca8;
    border: 1px solid #5a4a2a;
  }

  .kind-storm {
    background: #3a1a1a;
    color: #f88;
    border: 1px solid #6a2a2a;
  }

  .source-name {
    color: #dde;
    font-weight: bold;
    font-size: 0.78rem;
  }

  .controller {
    font-size: 0.72rem;
  }

  .badge {
    font-size: 0.6rem;
    padding: 0.05rem 0.25rem;
    border-radius: 2px;
    font-weight: bold;
  }

  .badge-copy {
    background: #2a2a5a;
    color: #88c;
    border: 1px solid #4a4a9a;
  }

  .targets {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    flex-wrap: wrap;
    width: 100%;
    padding-left: 1.5rem;
  }

  .targets-label {
    color: #668;
    font-size: 0.7rem;
  }

  .target-badge {
    background: #1a2030;
    color: #8ac;
    border: 1px solid #2a3050;
    font-size: 0.68rem;
    padding: 0.05rem 0.25rem;
    border-radius: 2px;
  }
</style>
