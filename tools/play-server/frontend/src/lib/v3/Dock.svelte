<script>
  /**
   * Dock — the right column (`docs/play-surface-v3-brief.md` §1 rule 10): the
   * stack, the combat card and the decision, in the mockup's
   * `minmax(0, 1fr) auto auto` grid.
   *
   * Everything other than `state`, `humanName` and `cardIndex` is the
   * `DecisionPanel` contract (which is `PromptPanel`'s) and is passed straight
   * through.
   */
  import StackBox from './StackBox.svelte';
  import CombatCard from './CombatCard.svelte';
  import DecisionPanel from './DecisionPanel.svelte';

  // `state` is the prop's name; a local called `state` would shadow the `$state` rune.
  const { state: view = null, humanName = null, cardIndex = new Map(), ...decisionProps } = $props();

  let panel = $state(null);

  export function beginExternal(option) {
    panel?.beginExternal(option);
  }
</script>

<div class="dock">
  <StackBox stack={view?.zones?.stack ?? []} />
  <CombatCard combat={view?.combat ?? null} battlefield={view?.zones?.battlefield ?? {}} {humanName} />
  <DecisionPanel bind:this={panel} {cardIndex} {humanName} turn={view?.turn ?? null} {...decisionProps} />
</div>
