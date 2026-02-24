<script>
  /**
   * AssertionBadges — inline pass/fail indicator for assert_state steps.
   *
   * Shows a compact row of colored badges next to the step counter.
   * Clicking expands to show path/expected/actual for each assertion.
   *
   * Props:
   *   assertions (AssertionResultView[]|null) — from the StepViewModel; null or [] if no assertions
   */

  const { assertions = null } = $props();

  /** Whether the detail panel is expanded. */
  let expanded = $state(false);

  function toggleExpanded() {
    expanded = !expanded;
  }

  const hasAssertions = $derived(assertions && assertions.length > 0);
  const passCount = $derived((assertions ?? []).filter((a) => a.passed).length);
  const failCount = $derived((assertions ?? []).filter((a) => !a.passed).length);

  /** All assertions passed? */
  const allPassed = $derived(failCount === 0 && passCount > 0);
  /** Any assertion failed? */
  const anyFailed = $derived(failCount > 0);
</script>

{#if hasAssertions}
  <div class="assertion-badges">
    <!-- Compact summary button -->
    <button
      class="summary-btn"
      class:all-passed={allPassed}
      class:any-failed={anyFailed}
      onclick={toggleExpanded}
      title={expanded ? 'Click to collapse assertions' : 'Click to expand assertions'}
    >
      {#if passCount > 0}
        <span class="badge-pass">✓ {passCount}</span>
      {/if}
      {#if failCount > 0}
        <span class="badge-fail">✗ {failCount}</span>
      {/if}
      <span class="expand-icon">{expanded ? '▲' : '▼'}</span>
    </button>

    <!-- Expanded detail -->
    {#if expanded}
      <div class="assertion-detail">
        {#each assertions as a, i}
          <div class="assertion-row" class:pass={a.passed} class:fail={!a.passed}>
            <span class="assertion-icon">{a.passed ? '✓' : '✗'}</span>
            <div class="assertion-content">
              <div class="assertion-path">{a.path}</div>
              <div class="assertion-values">
                <span class="av-label">expected:</span>
                <span class="av-value expected">{JSON.stringify(a.expected)}</span>
                {#if !a.passed}
                  <span class="av-label">actual:</span>
                  <span class="av-value actual">{JSON.stringify(a.actual)}</span>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .assertion-badges {
    display: inline-flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.15rem;
    font-family: monospace;
    font-size: 0.75rem;
    position: relative;
  }

  /* Compact summary button */
  .summary-btn {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    border-radius: 3px;
    padding: 0.15rem 0.4rem;
    cursor: pointer;
    font-family: monospace;
    font-size: 0.72rem;
    transition: background 0.1s;
  }

  .summary-btn:hover {
    background: #222240;
  }

  .summary-btn.all-passed {
    border-color: #1c4a1c;
    background: #0e2a0e;
  }

  .summary-btn.any-failed {
    border-color: #4a1c1c;
    background: #2a0e0e;
  }

  .badge-pass {
    color: #5c5;
    font-weight: bold;
  }

  .badge-fail {
    color: #c55;
    font-weight: bold;
  }

  .expand-icon {
    color: #446;
    font-size: 0.6rem;
  }

  /* Expanded detail panel */
  .assertion-detail {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 100;
    background: #111122;
    border: 1px solid #2a2a4a;
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    min-width: 280px;
    max-width: 380px;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    box-shadow: 0 4px 16px rgba(0,0,0,0.7);
    margin-top: 2px;
  }

  .assertion-row {
    display: flex;
    align-items: flex-start;
    gap: 0.35rem;
    padding: 0.2rem;
    border-radius: 3px;
    border: 1px solid transparent;
  }

  .assertion-row.pass {
    background: #0a1f0a;
    border-color: #1a3a1a;
  }

  .assertion-row.fail {
    background: #1f0a0a;
    border-color: #3a1a1a;
  }

  .assertion-icon {
    font-size: 0.75rem;
    flex-shrink: 0;
    padding-top: 0.05rem;
  }

  .assertion-row.pass .assertion-icon { color: #5c5; }
  .assertion-row.fail .assertion-icon { color: #c55; }

  .assertion-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .assertion-path {
    color: #aad;
    font-size: 0.72rem;
    font-weight: bold;
    word-break: break-all;
  }

  .assertion-values {
    display: flex;
    flex-wrap: wrap;
    gap: 0.2rem 0.4rem;
    align-items: center;
    font-size: 0.68rem;
  }

  .av-label {
    color: #446;
    flex-shrink: 0;
  }

  .av-value {
    font-size: 0.68rem;
    word-break: break-all;
  }

  .av-value.expected { color: #6a8; }
  .av-value.actual   { color: #c66; }
</style>
