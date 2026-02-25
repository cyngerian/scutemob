<script>
  import { onMount } from 'svelte';
  import StepControls from './lib/StepControls.svelte';
  import StateView from './lib/StateView.svelte';
  import PhaseIndicator from './lib/PhaseIndicator.svelte';
  import EventTimeline from './lib/EventTimeline.svelte';
  import ScriptPicker from './lib/ScriptPicker.svelte';
  import CombatView from './lib/CombatView.svelte';
  import CardDisplay from './lib/CardDisplay.svelte';
  import AssertionBadges from './lib/AssertionBadges.svelte';
  import {
    session,
    currentStepIndex,
    stepData,
    loading,
    stateDiff,
    runResult,
    initSession,
    goToStep,
  } from './lib/stores.js';
  import { loadScript, fetchScripts, approveScript } from './lib/api.js';

  let approving = $state(false);

  async function handleApprove() {
    if (!$session?.script_id) return;
    approving = true;
    try {
      await approveScript($session.script_id);
      // Update local state immediately — don't wait for a full reload.
      session.update(s => s ? { ...s, review_status: 'approved' } : s);
      // Refresh script list so the picker shows updated status.
      try {
        scripts = await fetchScripts();
      } catch (_) { /* ignore */ }
    } catch (err) {
      console.error('Failed to approve script:', err);
    } finally {
      approving = false;
    }
  }

  let scripts = $state(null);
  let showScriptPicker = $state(false);

  /** The card currently shown in the CardDisplay modal (null = hidden). */
  let selectedCard = $state(null);

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
      // Refresh script list (review_status might have changed)
      try {
        scripts = await fetchScripts();
      } catch (_) { /* ignore */ }
    } catch (err) {
      console.error('Failed to load script:', err);
    }
  }

  function getActionKind(action) {
    if (!action) return 'unknown';
    if (typeof action === 'object') {
      const keys = Object.keys(action);
      if (keys.length > 0) return keys[0];
    }
    return String(action);
  }

  // Current turn step for phase-jump in StepControls
  const currentTurnStep = $derived($stepData?.state?.turn?.step ?? null);

  /** Open the card detail panel for a given card object. */
  function openCard(card) {
    selectedCard = card;
  }

  /** Close the card detail panel. */
  function closeCard() {
    selectedCard = null;
  }

</script>

<div class="app">
  <!-- Top bar: session info + step controls + phase indicator -->
  <header class="top-bar">
    <div class="session-info">
      {#if $session?.loaded}
        <span class="script-name">{$session.script_name}</span>
        <span class="script-id muted">({$session.script_id})</span>

        <!-- RunResult badge -->
        {#if $runResult}
          {#if $runResult.harness_error}
            <span class="run-badge run-error" title={$runResult.harness_error}>
              ⚠ Error: {$runResult.harness_error.slice(0, 60)}{$runResult.harness_error.length > 60 ? '…' : ''}
            </span>
          {:else if $runResult.passed}
            <span class="run-badge run-pass">
              ✓ PASS ({$runResult.passed_count}/{$runResult.total_assertions})
            </span>
          {:else}
            <span
              class="run-badge run-fail"
              title={$runResult.first_failure ? `${$runResult.first_failure.path}: expected ${JSON.stringify($runResult.first_failure.expected)}, got ${JSON.stringify($runResult.first_failure.actual)}` : ''}
            >
              ✗ FAIL ({$runResult.passed_count}/{$runResult.total_assertions})
              {#if $runResult.first_failure}
                <span class="run-fail-path">— {$runResult.first_failure.path}</span>
              {/if}
            </span>
          {/if}
        {/if}

        <!-- Approve button (only for pending_review scripts) -->
        {#if $session.review_status === 'pending_review'}
          <button
            class="approve-btn"
            onclick={handleApprove}
            disabled={approving}
          >
            {approving ? 'Approving…' : 'Approve ✓'}
          </button>
        {:else if $session.review_status === 'approved'}
          <span class="approved-badge">✓ approved</span>
        {/if}
      {:else}
        <span class="muted">No script loaded</span>
      {/if}
      <button
        class="script-picker-btn"
        onclick={() => (showScriptPicker = !showScriptPicker)}
      >
        {showScriptPicker ? 'Close' : 'Browse Scripts'}
      </button>
    </div>

    <StepControls
      currentIndex={$currentStepIndex}
      totalSteps={$session?.total_steps ?? 0}
      currentStep={currentTurnStep}
      onStep={goToStep}
      loading={$loading}
    />

    <!-- Assertion badges (only on assert_state steps) -->
    {#if $stepData?.assertions?.length > 0}
      <div class="assertion-bar">
        <AssertionBadges assertions={$stepData.assertions} />
      </div>
    {/if}

    <!-- Phase indicator below step controls -->
    {#if $stepData?.state?.turn}
      <PhaseIndicator turn={$stepData.state.turn} />
    {:else}
      <div class="phase-placeholder"></div>
    {/if}
  </header>

  <!-- Script picker overlay (absolute, positioned near the button) -->
  {#if showScriptPicker}
    <div class="script-picker-overlay">
      <ScriptPicker
        {scripts}
        onLoad={handleLoadScript}
        onClose={() => (showScriptPicker = false)}
      />
    </div>
  {/if}

  <!-- Card display modal (portal-style via fixed positioning in CardDisplay.svelte) -->
  <CardDisplay card={selectedCard} onClose={closeCard} />

  <!-- Body: main state area + right sidebar (event timeline) -->
  <div class="body-layout">
    <!-- Main content area (left/center) -->
    <main class="content">
      {#if $loading && !$stepData}
        <div class="loading-message">Loading…</div>
      {:else if !$session?.loaded}
        <div class="empty-state">
          <p>No script loaded. Click "Browse Scripts" to select one.</p>
        </div>
      {:else if $stepData}
        <!-- Step metadata: action kind, turn/step -->
        <div class="step-meta">
          <span class="meta-item">
            <span class="meta-label">Action:</span>
            <span class="meta-value">{getActionKind($stepData.script_action)}</span>
          </span>
          {#if $stepData.state?.turn}
            <span class="meta-item">
              <span class="meta-label">Turn:</span>
              <span class="meta-value" class:changed={$stateDiff.has('turn.number')}>{$stepData.state.turn.number}</span>
            </span>
            <span class="meta-item">
              <span class="meta-label">Active:</span>
              <span class="meta-value" class:changed={$stateDiff.has('turn.active_player')}>{$stepData.state.turn.active_player}</span>
            </span>
            <span class="meta-item">
              <span class="meta-label">Step:</span>
              <span class="meta-value" class:changed={$stateDiff.has('turn.step')}>{$stepData.state.turn.step}</span>
            </span>
            {#if $stepData.state.turn.priority}
              <span class="meta-item">
                <span class="meta-label">Priority:</span>
                <span class="meta-value" class:changed={$stateDiff.has('turn.priority')}>{$stepData.state.turn.priority}</span>
              </span>
            {/if}
          {/if}
        </div>

        <!-- Combat view (only when combat state is present) -->
        {#if $stepData.state?.combat}
          <div class="combat-container" class:changed={$stateDiff.has('combat')}>
            <CombatView combat={$stepData.state.combat} />
          </div>
        {/if}

        <!-- State display: rich zone components with diff flags and card click handler -->
        <div class="state-container">
          <StateView
            state={$stepData.state}
            diff={$stateDiff}
            onCardClick={openCard}
          />
        </div>
      {/if}
    </main>

    <!-- Right sidebar: event timeline -->
    {#if $stepData}
      <aside class="event-sidebar">
        <EventTimeline
          events={$stepData.events ?? []}
          scriptAction={$stepData.script_action}
          stepIndex={$currentStepIndex}
          totalSteps={$session?.total_steps ?? 0}
        />
      </aside>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    background: #0d0d1a;
    color: #ddd;
    font-family: monospace;
  }

  /* Flash animation for changed fields */
  :global(.changed) {
    animation: changed-flash 1s ease-out forwards;
  }

  @keyframes -global-changed-flash {
    0%   { background-color: rgba(255, 220, 60, 0.35); }
    100% { background-color: transparent; }
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* ── Top bar ──────────────────────────────────────────────────────────── */

  .top-bar {
    display: flex;
    flex-direction: column;
    background: #111120;
    border-bottom: 2px solid #2a2a44;
    flex-shrink: 0;
  }

  .session-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 1rem;
    border-bottom: 1px solid #1a1a30;
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

  /* ── RunResult badge ─────────────────────────────────────────────────── */

  .run-badge {
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem;
    border-radius: 3px;
    font-family: monospace;
    white-space: nowrap;
    max-width: 420px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .run-pass {
    background: #1a3a1a;
    color: #5f5;
    border: 1px solid #2a5a2a;
  }

  .run-fail {
    background: #3a1a1a;
    color: #f77;
    border: 1px solid #5a2a2a;
  }

  .run-error {
    background: #3a2a00;
    color: #fa0;
    border: 1px solid #5a4a00;
  }

  .run-fail-path {
    color: #f99;
    font-size: 0.7rem;
  }

  /* ── Approve button / approved badge ──────────────────────────────────── */

  .approve-btn {
    background: #1a3a1a;
    color: #5f5;
    border: 1px solid #2a5a2a;
    padding: 0.2rem 0.6rem;
    cursor: pointer;
    border-radius: 3px;
    font-size: 0.78rem;
    font-family: monospace;
  }

  .approve-btn:hover:not(:disabled) {
    background: #2a5a2a;
    color: #afa;
  }

  .approve-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .approved-badge {
    font-size: 0.75rem;
    color: #5f5;
    padding: 0.15rem 0.4rem;
    border: 1px solid #2a5a2a;
    border-radius: 3px;
    background: #1a3a1a;
  }

  .assertion-bar {
    padding: 0.2rem 1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-bottom: 1px solid #1a1a30;
  }

  .phase-placeholder {
    height: 4px;
  }

  /* ── Script picker overlay ────────────────────────────────────────────── */

  .script-picker-overlay {
    position: absolute;
    top: 90px;
    right: 1rem;
    z-index: 200;
  }

  /* ── Body layout: main + sidebar ─────────────────────────────────────── */

  .body-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
    gap: 0;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 0.75rem 1rem;
    min-width: 0;
  }

  .event-sidebar {
    width: 320px;
    flex-shrink: 0;
    overflow-y: auto;
    padding: 0.5rem 0.5rem 0.5rem 0;
    border-left: 1px solid #1a1a30;
    background: #0b0b18;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  /* ── Loading / empty states ───────────────────────────────────────────── */

  .loading-message,
  .empty-state {
    text-align: center;
    color: #666;
    padding: 2rem;
    font-size: 0.9rem;
  }

  /* ── Step metadata bar ────────────────────────────────────────────────── */

  .step-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    padding: 0.4rem 0;
    border-bottom: 1px solid #1a1a30;
    margin-bottom: 0.6rem;
    font-size: 0.78rem;
  }

  .meta-item {
    display: flex;
    gap: 0.3rem;
  }

  .meta-label { color: #666; }
  .meta-value { color: #adf; }

  /* ── Combat container ─────────────────────────────────────────────────── */

  .combat-container {
    margin-bottom: 0.6rem;
  }

  /* ── State container ──────────────────────────────────────────────────── */

  .state-container {
    overflow: auto;
  }
</style>
