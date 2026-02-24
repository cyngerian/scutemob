<script>
  /**
   * StepControls — navigation bar for stepping through a replay.
   *
   * Props:
   *   currentIndex (number) — current step index
   *   totalSteps (number) — total number of steps
   *   currentStep (string|null) — current turn step name (for phase-jump detection)
   *   onStep (function) — called with new index when navigating
   *   loading (boolean) — disables buttons while fetching
   */
  import { fetchStepState } from './api.js';

  const { currentIndex, totalSteps, currentStep = null, onStep, loading = false } = $props();

  let autoPlayInterval = $state(null);
  let jumpingPhase = $state(false);
  const AUTO_PLAY_MS = 1000;

  // Derived state
  const isFirst = $derived(currentIndex <= 0);
  const isLast = $derived(currentIndex >= totalSteps - 1);

  function goFirst() {
    if (!isFirst && !loading) onStep(0);
  }

  function goPrev() {
    if (!isFirst && !loading) onStep(currentIndex - 1);
  }

  function goNext() {
    if (!isLast && !loading) onStep(currentIndex + 1);
  }

  function goLast() {
    if (!isLast && !loading) onStep(totalSteps - 1);
  }

  function toggleAutoPlay() {
    if (autoPlayInterval) {
      clearInterval(autoPlayInterval);
      autoPlayInterval = null;
    } else {
      autoPlayInterval = setInterval(() => {
        if (currentIndex >= totalSteps - 1) {
          clearInterval(autoPlayInterval);
          autoPlayInterval = null;
        } else {
          onStep(currentIndex + 1);
        }
      }, AUTO_PLAY_MS);
    }
  }

  /**
   * Phase-jump: scan forward or backward through steps until state.turn.step changes.
   * Uses GET /api/step/:n/state (lighter payload) to probe phase boundaries.
   * Calls onStep() with the first index where the step has changed.
   */
  async function jumpPhase(direction) {
    if (jumpingPhase || loading) return;
    if (direction > 0 && isLast) return;
    if (direction < 0 && isFirst) return;

    jumpingPhase = true;
    try {
      const startStep = currentStep;
      let probe = currentIndex + direction;
      const limit = direction > 0 ? totalSteps - 1 : 0;

      while (true) {
        // Fetch the lightweight state for the probe index
        let probeState;
        try {
          probeState = await fetchStepState(probe);
        } catch {
          break;
        }

        const probeStep = probeState?.turn?.step ?? null;
        if (probeStep !== startStep) {
          // Found a step with a different turn.step value — jump here
          onStep(probe);
          break;
        }

        // Haven't changed step yet — continue scanning
        if (probe === limit) {
          // Reached the boundary without finding a step change — jump to limit
          onStep(probe);
          break;
        }
        probe += direction;
      }
    } finally {
      jumpingPhase = false;
    }
  }

  // Keyboard navigation (bound to window via svelte:window)
  function handleKeydown(e) {
    if (loading) return;
    switch (e.key) {
      case 'ArrowLeft':
        e.preventDefault();
        if (e.shiftKey) {
          jumpPhase(-1);
        } else {
          goPrev();
        }
        break;
      case 'ArrowRight':
        e.preventDefault();
        if (e.shiftKey) {
          jumpPhase(1);
        } else {
          goNext();
        }
        break;
      case 'Home':
        goFirst();
        break;
      case 'End':
        goLast();
        break;
      case ' ':
        e.preventDefault();
        toggleAutoPlay();
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="step-controls">
  <button class="ctrl-btn" onclick={goFirst} disabled={isFirst || loading} title="First step (Home)">
    |&lt;
  </button>
  <button class="ctrl-btn" onclick={goPrev} disabled={isFirst || loading} title="Previous step (Arrow Left)">
    &lt;
  </button>

  <span class="step-counter">
    {#if totalSteps > 0}
      Step {currentIndex} / {totalSteps - 1}
    {:else}
      No script loaded
    {/if}
  </span>

  <button class="ctrl-btn" onclick={goNext} disabled={isLast || loading} title="Next step (Arrow Right)">
    &gt;
  </button>
  <button class="ctrl-btn" onclick={goLast} disabled={isLast || loading} title="Last step (End)">
    &gt;|
  </button>

  <button
    class="ctrl-btn autoplay-btn"
    class:active={autoPlayInterval !== null}
    onclick={toggleAutoPlay}
    disabled={totalSteps === 0}
    title="Auto-play (Space)"
  >
    {autoPlayInterval !== null ? '⏸' : '▶'}
  </button>

  {#if loading || jumpingPhase}
    <span class="loading-indicator">{jumpingPhase ? 'jumping…' : 'loading…'}</span>
  {/if}
</div>

<style>
  .step-controls {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: #1a1a2e;
    border-bottom: 1px solid #333;
    font-family: monospace;
  }

  .ctrl-btn {
    background: #2a2a4a;
    color: #ccc;
    border: 1px solid #444;
    padding: 0.3rem 0.6rem;
    cursor: pointer;
    font-family: monospace;
    font-size: 0.9rem;
    border-radius: 3px;
  }

  .ctrl-btn:hover:not(:disabled) {
    background: #3a3a6a;
    color: #fff;
  }

  .ctrl-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .ctrl-btn.active {
    background: #4a4aaa;
    color: #fff;
  }

  .step-counter {
    color: #aaa;
    font-size: 0.9rem;
    min-width: 12rem;
    text-align: center;
  }

  .loading-indicator {
    color: #888;
    font-size: 0.85rem;
    font-style: italic;
  }

  .autoplay-btn {
    margin-left: 0.5rem;
  }
</style>
