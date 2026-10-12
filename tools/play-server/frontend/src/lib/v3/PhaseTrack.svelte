<script>
  /**
   * PhaseTrack — the row under the header (`docs/play-surface-v3-brief.md` §1
   * rule 9): turn and whose turn on the left, one segment per step, priority on
   * the right. Past / current / combat are the mockup's `.seg` classes.
   *
   * The wire has thirteen steps; the first-strike damage step only exists in a
   * combat that has a first striker, so its segment is shown only while it is
   * the current step and the track is twelve wide the rest of the time.
   */
  import { STEP_ORDER, SHORT_STEP, COMBAT_STEPS } from './layout.js';

  const { turn = null, humanName = null } = $props();

  const steps = $derived(STEP_ORDER.filter((s) => s !== 'FirstStrikeDamage' || turn?.step === s));
  const now = $derived(steps.indexOf(turn?.step));
  const mine = $derived(!!turn?.active_player && turn.active_player === humanName);
  const prioMine = $derived(!!turn?.priority && turn.priority === humanName);
</script>

<div class="phase">
  {#if turn}
    <span class="turn">TURN {turn.number}</span>
    <span class="who" class:me={mine}>{mine ? 'your turn' : `${turn.active_player}'s turn`}</span>
    <div class="track">
      {#each steps as step, k (step)}
        <div class="seg" class:past={k < now} class:now={k === now} class:combat={COMBAT_STEPS.has(step)}>
          {COMBAT_STEPS.has(step) ? '⚔ ' : ''}{SHORT_STEP[step] ?? step}
        </div>
      {/each}
    </div>
    <span class="prio" class:me={prioMine}>priority: {prioMine ? 'YOU' : (turn.priority ?? '—')}</span>
  {:else}
    <span class="who">no game</span>
  {/if}
</div>
