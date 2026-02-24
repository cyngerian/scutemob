<script>
  import { onMount } from 'svelte';
  import StepControls from './lib/StepControls.svelte';
  import StateView from './lib/StateView.svelte';
  import {
    session,
    currentStepIndex,
    stepData,
    loading,
    initSession,
    goToStep,
  } from './lib/stores.js';
  import { loadScript, fetchScripts } from './lib/api.js';

  let scripts = $state(null);
  let showScriptPicker = $state(false);

  onMount(async () => {
    await initSession();
    try {
      scripts = await fetchScripts();
    } catch (err) {
      console.warn('Could not load scripts list:', err);
    }
  });

  async function handleLoadScript(path) {
    try {
      await loadScript(path);
      showScriptPicker = false;
      await initSession();
    } catch (err) {
      console.error('Failed to load script:', err);
    }
  }

  // Flatten all script entries from groups
  function allScripts(scriptsData) {
    if (!scriptsData) return [];
    return Object.values(scriptsData.groups).flat();
  }

  function getActionKind(action) {
    if (!action) return 'unknown';
    if (typeof action === 'object') {
      const keys = Object.keys(action);
      if (keys.length > 0) return keys[0];
    }
    return String(action);
  }
</script>

<div class="app">
  <!-- Top bar: controls + session info -->
  <header class="top-bar">
    <div class="session-info">
      {#if $session?.loaded}
        <span class="script-name">{$session.script_name}</span>
        <span class="script-id muted">({$session.script_id})</span>
      {:else}
        <span class="muted">No script loaded</span>
      {/if}
      <button
        class="script-picker-btn"
        onclick={() => (showScriptPicker = !showScriptPicker)}
      >
        Browse Scripts
      </button>
    </div>

    <StepControls
      currentIndex={$currentStepIndex}
      totalSteps={$session?.total_steps ?? 0}
      onStep={goToStep}
      loading={$loading}
    />
  </header>

  <!-- Script picker dropdown -->
  {#if showScriptPicker && scripts}
    <div class="script-picker">
      <div class="script-picker-header">
        <strong>Select a script ({scripts.total} total)</strong>
        <button onclick={() => (showScriptPicker = false)}>Close</button>
      </div>
      {#each Object.entries(scripts.groups).sort() as [subdir, entries]}
        <div class="script-group">
          <div class="script-group-title">{subdir}/</div>
          {#each entries as entry}
            <button
              class="script-entry"
              onclick={() => handleLoadScript(entry.path)}
            >
              <span class="entry-name">{entry.name}</span>
              {#if entry.review_status}
                <span class="badge badge-{entry.review_status}">{entry.review_status}</span>
              {/if}
            </button>
          {/each}
        </div>
      {/each}
    </div>
  {/if}

  <!-- Main content area -->
  <main class="content">
    {#if $loading && !$stepData}
      <div class="loading-message">Loading…</div>
    {:else if !$session?.loaded}
      <div class="empty-state">
        <p>No script loaded. Click "Browse Scripts" to select one.</p>
      </div>
    {:else if $stepData}
      <!-- Step metadata -->
      <div class="step-meta">
        <span class="meta-item">
          <span class="meta-label">Action:</span>
          <span class="meta-value">{getActionKind($stepData.script_action)}</span>
        </span>
        <span class="meta-item">
          <span class="meta-label">Events:</span>
          <span class="meta-value">{$stepData.events?.length ?? 0}</span>
        </span>
        {#if $stepData.state?.turn}
          <span class="meta-item">
            <span class="meta-label">Turn:</span>
            <span class="meta-value">{$stepData.state.turn.number}</span>
          </span>
          <span class="meta-item">
            <span class="meta-label">Active:</span>
            <span class="meta-value">{$stepData.state.turn.active_player}</span>
          </span>
          <span class="meta-item">
            <span class="meta-label">Step:</span>
            <span class="meta-value">{$stepData.state.turn.step}</span>
          </span>
          {#if $stepData.state.turn.priority}
            <span class="meta-item">
              <span class="meta-label">Priority:</span>
              <span class="meta-value">{$stepData.state.turn.priority}</span>
            </span>
          {/if}
        {/if}
        {#if $stepData.assertions}
          {#each $stepData.assertions as a}
            <span class="meta-item assertion-{a.passed ? 'pass' : 'fail'}">
              {a.passed ? '✓' : '✗'} {a.path}: {JSON.stringify(a.actual)}
            </span>
          {/each}
        {/if}
      </div>

      <!-- State display: rich zone components (Session 3) -->
      <div class="state-container">
        <StateView state={$stepData.state} />
      </div>
    {/if}
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    background: #0d0d1a;
    color: #ddd;
    font-family: monospace;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .top-bar {
    display: flex;
    flex-direction: column;
    background: #111120;
    border-bottom: 2px solid #333;
    flex-shrink: 0;
  }

  .session-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 1rem;
    border-bottom: 1px solid #222;
    font-size: 0.85rem;
  }

  .script-name {
    font-weight: bold;
    color: #adf;
  }

  .muted {
    color: #666;
    font-size: 0.8rem;
  }

  .script-picker-btn {
    margin-left: auto;
    background: #2a2a5a;
    color: #ccc;
    border: 1px solid #444;
    padding: 0.25rem 0.6rem;
    cursor: pointer;
    border-radius: 3px;
    font-size: 0.8rem;
    font-family: monospace;
  }

  .script-picker-btn:hover {
    background: #3a3a7a;
    color: #fff;
  }

  .script-picker {
    position: absolute;
    top: 80px;
    right: 1rem;
    width: 400px;
    max-height: 500px;
    overflow-y: auto;
    background: #1a1a30;
    border: 1px solid #444;
    border-radius: 4px;
    z-index: 100;
    padding: 0.5rem;
  }

  .script-picker-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.25rem 0;
    margin-bottom: 0.5rem;
    border-bottom: 1px solid #333;
    font-size: 0.85rem;
  }

  .script-group {
    margin-bottom: 0.5rem;
  }

  .script-group-title {
    font-size: 0.75rem;
    color: #888;
    padding: 0.1rem 0;
    text-transform: uppercase;
  }

  .script-entry {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    text-align: left;
    background: transparent;
    color: #ccc;
    border: none;
    padding: 0.2rem 0.4rem;
    cursor: pointer;
    font-size: 0.8rem;
    font-family: monospace;
  }

  .script-entry:hover {
    background: #2a2a4a;
    color: #fff;
  }

  .entry-name {
    flex: 1;
  }

  .badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.3rem;
    border-radius: 2px;
  }

  .badge-approved {
    background: #1a4a1a;
    color: #6f6;
  }

  .badge-pending {
    background: #4a4a1a;
    color: #ff6;
  }

  .badge-disputed {
    background: #4a1a1a;
    color: #f66;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 1rem;
  }

  .loading-message,
  .empty-state {
    text-align: center;
    color: #666;
    padding: 2rem;
    font-size: 0.9rem;
  }

  .step-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    padding: 0.5rem 0;
    border-bottom: 1px solid #222;
    margin-bottom: 0.75rem;
    font-size: 0.8rem;
  }

  .meta-item {
    display: flex;
    gap: 0.3rem;
  }

  .meta-label {
    color: #888;
  }

  .meta-value {
    color: #adf;
  }

  .assertion-pass {
    color: #6f6;
  }

  .assertion-fail {
    color: #f66;
  }

  .state-container {
    overflow: auto;
  }
</style>
