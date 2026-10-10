<script>
  /**
   * PhaseStrip — the turn structure as ONE row (`scutemob-261`).
   *
   * Play-local replacement for `$viewer/PhaseIndicator`, which draws five
   * two-line phase blocks (label row + step row, ~70px). On a 1080p display
   * that was the second-largest band of chrome above the board, for a fact a
   * single highlighted pip states just as well. The replay viewer keeps its own
   * component untouched; this one shares the step table by copying it — the
   * table is twelve rows and the viewer does not export it.
   *
   * Props:
   *   turn (TurnView) — `state.turn`: `number`, `active_player`, `priority`,
   *                     `step`, `phase`
   */
  const { turn } = $props();

  const PHASES = [
    { short: 'BEG', steps: [['Untap', 'Untap'], ['Upkeep', 'Upkeep'], ['Draw', 'Draw']] },
    { short: 'M1', steps: [['PreCombatMain', 'Main 1']] },
    {
      short: 'CMB',
      steps: [
        ['BeginningOfCombat', 'Begin'],
        ['DeclareAttackers', 'Attack'],
        ['DeclareBlockers', 'Block'],
        ['FirstStrikeDamage', '1st'],
        ['CombatDamage', 'Dmg'],
        ['EndOfCombat', 'End'],
      ],
    },
    { short: 'M2', steps: [['PostCombatMain', 'Main 2']] },
    { short: 'END', steps: [['End', 'End'], ['Cleanup', 'Clean']] },
  ];

  const currentStep = $derived(turn?.step ?? '');
  const turnNumber = $derived(turn?.number ?? 1);
  const activePlayer = $derived(turn?.active_player ?? '');
  const priorityPlayer = $derived(turn?.priority ?? null);

  function phaseIsCurrent(ph) {
    return ph.steps.some(([s]) => s === currentStep);
  }
</script>

<div class="phase-strip">
  <span class="turn">
    Turn <b>{turnNumber}</b> · <span class="active">{activePlayer}</span>
    {#if priorityPlayer && priorityPlayer !== activePlayer}
      <span class="prio">· <b>{priorityPlayer}</b> priority</span>
    {/if}
  </span>
  <div class="pips">
    {#each PHASES as ph (ph.short)}
      <span class="phase" class:current={phaseIsCurrent(ph)}>
        <span class="phase-label">{ph.short}</span>
        {#each ph.steps as [step, label] (step)}
          <span class="pip" class:on={step === currentStep}>{label}</span>
        {/each}
      </span>
    {/each}
  </div>
</div>

<style>
  .phase-strip {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    padding: 0.15rem 0.6rem;
    background: #0e0e20;
    border-bottom: 1px solid #2a2a44;
    font-family: monospace;
    font-size: 0.7rem;
    white-space: nowrap;
    overflow-x: auto;
  }

  .turn {
    color: #889;
    flex-shrink: 0;
  }

  .turn b {
    color: #adf;
  }

  .active {
    color: #6af;
    font-weight: bold;
  }

  .prio {
    color: #776;
  }

  .prio b {
    color: #fa0;
  }

  .pips {
    display: flex;
    gap: 0.35rem;
    align-items: center;
  }

  .phase {
    display: inline-flex;
    align-items: center;
    gap: 0.15rem;
    padding: 0.05rem 0.25rem;
    border-radius: 3px;
    border: 1px solid #1e1e3a;
    background: #12122a;
  }

  .phase.current {
    border-color: #4040a0;
    background: #1a1a50;
  }

  .phase-label {
    color: #556;
    font-size: 0.6rem;
    letter-spacing: 0.06em;
    margin-right: 0.15rem;
  }

  .phase.current .phase-label {
    color: #99b;
  }

  .pip {
    color: #556;
    padding: 0 0.2rem;
    border-radius: 2px;
    border: 1px solid transparent;
  }

  .phase.current .pip {
    color: #99a;
  }

  .pip.on {
    background: #3030a0;
    color: #dde;
    border-color: #6060d0;
    font-weight: bold;
  }
</style>
