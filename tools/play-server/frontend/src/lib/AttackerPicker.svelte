<script>
  /**
   * AttackerPicker — CR 508.1a: declare attackers and, for each, what it attacks.
   *
   * M11-local Session 7 (`memory/m11-session-plan.md` §4, item 4).
   *
   * Props:
   *   eligible (CombatantOptionView[]) — `{id, label}`, from the provider's own
   *     `LegalAction::DeclareAttackers { eligible, .. }` (`view.rs::combat_options`)
   *   targets (AttackTargetOptionView[]) — `{kind, id, label, value}`, what any
   *     attacker may be declared as attacking (a player or a planeswalker)
   *   disabled (bool)
   *   onConfirm (fn(attackers)) — `attackers` is `[[objectId, AttackTargetValue], ...]`
   *     for every SELECTED creature only
   *   onCancel (fn)
   *
   * # CR 508.1 — declaring no attackers is legal, and must say so
   *
   * This is the direct fix for the S6 review MEDIUM: a default-params submission
   * used to send an empty `attackers` array with no indication that is what the
   * button did. Here the empty case is reachable only through this component's
   * own confirm button, and the button's own label says "Attack with nothing"
   * whenever the current selection is empty, so the human cannot land on it by
   * accident the way a blanket default-submit did.
   *
   * # One attack-target choice per SELECTED attacker
   *
   * `targets` is the single flat CR 508.1a list every attacker may choose among
   * (multiple defending players' planeswalkers/players in a Commander game do not
   * come pre-filtered per attacker — the provider emits one shared list). Each
   * selected attacker gets its own chooser, defaulted to `targets[0]`'s value,
   * because CR 508.1 requires every attacker to be attacking *something* the
   * instant it is declared — there is no "declared but undecided" state to leave
   * a picker in.
   */
  const { eligible = [], targets = [], disabled = false, onConfirm = null, onCancel = null } =
    $props();

  /** `attackerId -> index into targets` for every currently selected attacker. */
  let selection = $state(new Map());

  const selectedIds = $derived([...selection.keys()]);
  const attackingWithNone = $derived(selectedIds.length === 0);

  function toggleAttacker(id) {
    if (disabled) return;
    const next = new Map(selection);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.set(id, 0);
    }
    selection = next;
  }

  function setAttackTarget(id, targetIndex) {
    if (disabled) return;
    const next = new Map(selection);
    next.set(id, targetIndex);
    selection = next;
  }

  function confirm() {
    if (disabled) return;
    const attackers = [...selection.entries()].map(([id, targetIndex]) => [
      id,
      targets[targetIndex].value,
    ]);
    onConfirm?.(attackers);
  }
</script>

<div class="attacker-picker">
  <div class="picker-header">
    <span class="picker-title">Declare attackers</span>
    <span class="picker-count">{eligible.length} eligible</span>
  </div>

  {#if eligible.length === 0}
    <span class="no-candidates">no creature is eligible to attack</span>
  {:else}
    <div class="attackers">
      {#each eligible as creature (creature.id)}
        <div class="attacker-row">
          <button
            class="attacker-toggle"
            class:selected={selection.has(creature.id)}
            disabled={disabled}
            onclick={() => toggleAttacker(creature.id)}
          >
            {creature.label}
          </button>
          {#if selection.has(creature.id) && targets.length > 0}
            <select
              class="attack-target"
              disabled={disabled}
              value={selection.get(creature.id)}
              onchange={(e) => setAttackTarget(creature.id, Number(e.currentTarget.value))}
            >
              {#each targets as target, i (i)}
                <option value={i}>{target.label}</option>
              {/each}
            </select>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <div class="picker-actions">
    <button class="confirm" disabled={disabled} onclick={confirm}>
      {attackingWithNone ? 'Attack with nothing' : `Attack with ${selectedIds.length}`}
    </button>
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Cancel</button>
  </div>
</div>

<style>
  .attacker-picker {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.4rem 0.6rem;
    background: #151530;
    border-top: 1px solid #2a2a44;
    font-family: monospace;
  }

  .picker-header {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  .picker-title {
    font-size: 0.78rem;
    color: #adf;
    font-weight: bold;
  }

  .picker-count {
    font-size: 0.68rem;
    color: #667;
  }

  .no-candidates {
    font-size: 0.72rem;
    color: #766;
    font-style: italic;
  }

  .attackers {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .attacker-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .attacker-toggle {
    padding: 0.2rem 0.45rem;
    font-size: 0.74rem;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
  }

  .attacker-toggle:hover:not(:disabled) {
    background: #2a2a58;
    border-color: #4a4a90;
  }

  .attacker-toggle:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .attacker-toggle.selected {
    background: #23386a;
    border-color: #3a5aa0;
    color: #dde;
  }

  .attack-target {
    font-family: monospace;
    font-size: 0.72rem;
    background: #141428;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    padding: 0.15rem 0.3rem;
  }

  .picker-actions {
    display: flex;
    gap: 0.4rem;
  }

  .confirm {
    padding: 0.2rem 0.5rem;
    font-size: 0.76rem;
    background: #23386a;
    color: #dde;
    border: 1px solid #3a5aa0;
    border-radius: 3px;
    cursor: pointer;
  }

  .confirm:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .cancel {
    padding: 0.2rem 0.5rem;
    font-size: 0.76rem;
    background: #241824;
    color: #caa;
    border: 1px solid #4a2a4a;
    border-radius: 3px;
    cursor: pointer;
  }

  .cancel:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
