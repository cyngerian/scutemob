<script>
  /**
   * PhaseIndicator — horizontal phase/step progress bar.
   *
   * Renders all MTG turn phases and steps as a horizontal bar.
   * The current phase section is highlighted; the current step has a
   * more intense highlight.
   *
   * Props:
   *   turn (TurnView) — turn state from the view model
   */
  const { turn } = $props();

  /**
   * Phase/step structure for the indicator bar.
   * Each phase has a label and an array of steps.
   * Step names must match the serialized Step enum from the engine.
   */
  const PHASES = [
    {
      phase: 'Beginning',
      label: 'Beginning',
      short: 'BEG',
      steps: [
        { step: 'Untap', label: 'Untap' },
        { step: 'Upkeep', label: 'Upkeep' },
        { step: 'Draw', label: 'Draw' },
      ],
    },
    {
      phase: 'PreCombatMain',
      label: 'Pre-Combat Main',
      short: 'M1',
      steps: [{ step: 'PreCombatMain', label: 'Main 1' }],
    },
    {
      phase: 'Combat',
      label: 'Combat',
      short: 'CMB',
      steps: [
        { step: 'BeginningOfCombat', label: 'Begin' },
        { step: 'DeclareAttackers', label: 'Attackers' },
        { step: 'DeclareBlockers', label: 'Blockers' },
        { step: 'CombatDamage', label: 'Damage' },
        { step: 'EndOfCombat', label: 'End' },
      ],
    },
    {
      phase: 'PostCombatMain',
      label: 'Post-Combat Main',
      short: 'M2',
      steps: [{ step: 'PostCombatMain', label: 'Main 2' }],
    },
    {
      phase: 'Ending',
      label: 'Ending',
      short: 'END',
      steps: [
        { step: 'End', label: 'End' },
        { step: 'Cleanup', label: 'Cleanup' },
      ],
    },
  ];

  /** Map step names to their parent phase for highlight logic. */
  const STEP_TO_PHASE = {};
  for (const ph of PHASES) {
    for (const s of ph.steps) {
      STEP_TO_PHASE[s.step] = ph.phase;
    }
  }

  const currentStep = $derived(turn?.step ?? '');
  const currentPhase = $derived(turn?.phase ?? STEP_TO_PHASE[currentStep] ?? '');
  const turnNumber = $derived(turn?.number ?? 1);
  const activePlayer = $derived(turn?.active_player ?? '');
  const priorityPlayer = $derived(turn?.priority ?? null);
</script>

<div class="phase-indicator">
  <div class="turn-label">
    Turn <span class="turn-num">{turnNumber}</span>
    — <span class="active-player">{activePlayer}</span>
    {#if priorityPlayer && priorityPlayer !== activePlayer}
      <span class="priority-label">| <span class="priority-player">{priorityPlayer}</span> priority</span>
    {/if}
  </div>

  <div class="phases-bar">
    {#each PHASES as ph (ph.phase)}
      {@const isCurrentPhase = ph.phase === currentPhase || ph.steps.some(s => s.step === currentStep)}
      <div
        class="phase-block"
        class:current-phase={isCurrentPhase}
      >
        <div class="phase-label">{ph.short}</div>
        <div class="steps-row">
          {#each ph.steps as s (s.step)}
            {@const isCurrentStep = s.step === currentStep}
            <div
              class="step-pip"
              class:current-step={isCurrentStep}
              title="{ph.label}: {s.label}"
            >
              {s.label}
            </div>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .phase-indicator {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.3rem 1rem;
    background: #0e0e20;
    border-bottom: 1px solid #2a2a44;
    font-family: monospace;
  }

  .turn-label {
    font-size: 0.75rem;
    color: #888;
    padding-bottom: 0.15rem;
  }

  .turn-num {
    color: #adf;
    font-weight: bold;
  }

  .active-player {
    color: #6af;
    font-weight: bold;
  }

  .priority-label {
    color: #666;
  }

  .priority-player {
    color: #fa0;
    font-weight: bold;
  }

  .phases-bar {
    display: flex;
    gap: 2px;
    align-items: stretch;
  }

  .phase-block {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0.15rem 0.3rem;
    border-radius: 3px;
    background: #151530;
    border: 1px solid #252545;
    min-width: 0;
    flex: 1;
    transition: background 0.1s;
  }

  .phase-block.current-phase {
    background: #1a1a50;
    border-color: #4040a0;
  }

  .phase-label {
    font-size: 0.6rem;
    color: #556;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    text-align: center;
    white-space: nowrap;
  }

  .phase-block.current-phase .phase-label {
    color: #88a;
  }

  .steps-row {
    display: flex;
    gap: 2px;
    flex-wrap: wrap;
    justify-content: center;
  }

  .step-pip {
    font-size: 0.62rem;
    padding: 0.1rem 0.25rem;
    border-radius: 2px;
    background: #1c1c38;
    color: #666;
    border: 1px solid transparent;
    white-space: nowrap;
    cursor: default;
  }

  .step-pip.current-step {
    background: #3030a0;
    color: #cce;
    border-color: #6060d0;
    font-weight: bold;
  }

  .phase-block.current-phase .step-pip:not(.current-step) {
    color: #99a;
  }
</style>
