<script>
  /**
   * ScriptPicker — tree-view browser for game script JSON files.
   *
   * Props:
   *   scripts (object) — { groups: { [subdir]: ScriptEntry[] }, total: number }
   *   onLoad (function) — called with (path: string) when user selects a script
   *   onClose (function) — called when user clicks Close
   */

  const { scripts = null, onLoad, onClose } = $props();

  /** Filter text for the search input. */
  let filterText = $state('');

  /** Which subdirectory is collapsed (key = subdir, value = true when collapsed). */
  let collapsed = $state({});

  function toggleCollapse(subdir) {
    collapsed = { ...collapsed, [subdir]: !collapsed[subdir] };
  }

  /** Sorted subdirectory keys. */
  const subdirs = $derived(
    scripts ? Object.keys(scripts.groups).sort() : []
  );

  /**
   * Filter entries in a subdir by filterText.
   * Matches on name, id, tags, or description (case-insensitive).
   */
  function filteredEntries(subdir) {
    const entries = scripts?.groups?.[subdir] ?? [];
    if (!filterText.trim()) return entries;
    const q = filterText.trim().toLowerCase();
    return entries.filter(
      (e) =>
        e.name.toLowerCase().includes(q) ||
        e.id.toLowerCase().includes(q) ||
        (e.description ?? '').toLowerCase().includes(q) ||
        (e.tags ?? []).some((t) => t.toLowerCase().includes(q))
    );
  }

  /** Total count of visible entries across all subdirs. */
  const visibleCount = $derived(
    subdirs.reduce((n, sd) => n + filteredEntries(sd).length, 0)
  );

  function handleSelect(entry) {
    onLoad?.(entry.path);
  }

  /** Review status → badge CSS class. */
  function badgeClass(status) {
    switch (status) {
      case 'approved': return 'badge badge-approved';
      case 'pending': return 'badge badge-pending';
      case 'disputed': return 'badge badge-disputed';
      default: return 'badge badge-unknown';
    }
  }
</script>

<div class="script-picker">
  <!-- Header -->
  <div class="picker-header">
    <span class="picker-title">
      Browse Scripts
      <span class="total-count">{scripts?.total ?? 0} total</span>
    </span>
    <button class="close-btn" onclick={onClose} title="Close script browser">✕</button>
  </div>

  <!-- Search -->
  <div class="search-row">
    <!-- svelte-ignore a11y_autofocus -->
    <input
      class="search-input"
      type="text"
      placeholder="Filter by name, tag, or id…"
      bind:value={filterText}
      autofocus
    />
    {#if filterText}
      <button class="clear-btn" onclick={() => (filterText = '')} title="Clear filter">✕</button>
    {/if}
  </div>

  {#if filterText && visibleCount === 0}
    <div class="no-results muted">No scripts match "{filterText}"</div>
  {/if}

  <!-- Script tree -->
  <div class="script-tree">
    {#each subdirs as subdir}
      {@const entries = filteredEntries(subdir)}
      {#if entries.length > 0}
        <div class="subdir-group">
          <!-- Subdirectory header (collapsible) -->
          <button
            class="subdir-header"
            onclick={() => toggleCollapse(subdir)}
            title="Toggle {subdir}/"
          >
            <span class="collapse-icon">{collapsed[subdir] ? '▶' : '▼'}</span>
            <span class="subdir-name">{subdir}/</span>
            <span class="subdir-count">{entries.length}</span>
          </button>

          <!-- Script entries -->
          {#if !collapsed[subdir]}
            <div class="entries-list">
              {#each entries as entry (entry.id)}
                <button
                  class="script-entry"
                  onclick={() => handleSelect(entry)}
                  title={entry.description || entry.name}
                >
                  <span class="entry-name">{entry.name}</span>
                  <div class="entry-meta">
                    {#if entry.tags?.length > 0}
                      {#each entry.tags.slice(0, 3) as tag}
                        <span class="tag">{tag}</span>
                      {/each}
                    {/if}
                    {#if entry.review_status}
                      <span class={badgeClass(entry.review_status)}>{entry.review_status}</span>
                    {/if}
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    {/each}
  </div>
</div>

<style>
  .script-picker {
    display: flex;
    flex-direction: column;
    background: #111125;
    border: 1px solid #2a2a50;
    border-radius: 5px;
    overflow: hidden;
    font-family: monospace;
    font-size: 0.8rem;
    width: 360px;
    max-height: 580px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.6);
  }

  /* Header */
  .picker-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.4rem 0.6rem;
    background: #1a1a35;
    border-bottom: 1px solid #2a2a50;
    flex-shrink: 0;
  }

  .picker-title {
    font-weight: bold;
    color: #99b;
    font-size: 0.82rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .total-count {
    color: #556;
    font-size: 0.72rem;
    font-weight: normal;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: #556;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    line-height: 1;
  }

  .close-btn:hover {
    background: #2a2a4a;
    color: #aaa;
  }

  /* Search */
  .search-row {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.35rem 0.5rem;
    border-bottom: 1px solid #1a1a30;
    flex-shrink: 0;
    background: #0e0e20;
  }

  .search-input {
    flex: 1;
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    border-radius: 3px;
    color: #ccc;
    font-family: monospace;
    font-size: 0.78rem;
    padding: 0.25rem 0.4rem;
    outline: none;
  }

  .search-input:focus {
    border-color: #4a4a8a;
    background: #1e1e38;
  }

  .search-input::placeholder {
    color: #445;
  }

  .clear-btn {
    background: transparent;
    border: none;
    color: #556;
    cursor: pointer;
    font-size: 0.72rem;
    padding: 0 0.2rem;
  }

  .clear-btn:hover {
    color: #aaa;
  }

  .no-results {
    text-align: center;
    padding: 1rem;
    font-size: 0.75rem;
  }

  .muted {
    color: #445;
  }

  /* Script tree */
  .script-tree {
    overflow-y: auto;
    flex: 1;
  }

  /* Subdir group */
  .subdir-group {
    border-bottom: 1px solid #111128;
  }

  .subdir-header {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    width: 100%;
    padding: 0.2rem 0.5rem;
    background: #0e0e20;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: monospace;
    color: #668;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    transition: background 0.1s;
  }

  .subdir-header:hover {
    background: #141430;
    color: #889;
  }

  .collapse-icon {
    font-size: 0.6rem;
    color: #446;
    flex-shrink: 0;
  }

  .subdir-name {
    flex: 1;
  }

  .subdir-count {
    font-size: 0.65rem;
    color: #446;
    background: #1a1a2a;
    padding: 0.05rem 0.25rem;
    border-radius: 3px;
  }

  /* Script entries */
  .entries-list {
    display: flex;
    flex-direction: column;
  }

  .script-entry {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.28rem 0.6rem 0.28rem 1rem;
    background: transparent;
    border: none;
    border-bottom: 1px solid #0e0e1c;
    cursor: pointer;
    text-align: left;
    font-family: monospace;
    color: #bbb;
    transition: background 0.08s;
  }

  .script-entry:hover {
    background: #1a1a35;
    color: #ddf;
  }

  .entry-name {
    font-size: 0.78rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .entry-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.2rem;
    align-items: center;
  }

  .tag {
    font-size: 0.62rem;
    padding: 0.05rem 0.2rem;
    background: #1a2a3a;
    color: #68a;
    border-radius: 2px;
  }

  .badge {
    font-size: 0.62rem;
    padding: 0.05rem 0.25rem;
    border-radius: 2px;
    font-weight: bold;
  }

  .badge-approved  { background: #0e2e0e; color: #5c5; border: 1px solid #1c4a1c; }
  .badge-pending   { background: #2e2e0e; color: #cc5; border: 1px solid #4a4a1c; }
  .badge-disputed  { background: #2e0e0e; color: #c55; border: 1px solid #4a1c1c; }
  .badge-unknown   { background: #1a1a2a; color: #778; border: 1px solid #2a2a3a; }
</style>
