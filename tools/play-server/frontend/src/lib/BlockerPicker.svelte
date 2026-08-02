<script>
  /**
   * BlockerPicker — CR 509.1a: pair each eligible blocker with the attacker it
   * blocks, or leave it unassigned.
   *
   * M11-local Session 7 (`memory/m11-session-plan.md` §4, item 5).
   *
   * Props:
   *   eligible (CombatantOptionView[]) — `{id, label}`, creatures that may block,
   *     from `LegalAction::DeclareBlockers { eligible, .. }`
   *   attackers (CombatantOptionView[]) — `{id, label}`, the attacking creatures
   *     a blocker may be assigned to, from the same `LegalAction`'s `attackers`
   *   disabled (bool)
   *   onConfirm (fn(blockers)) — `blockers` is `[[blockerId, attackerId], ...]`
   *     for every blocker the human actually assigned; an unassigned blocker
   *     contributes no entry
   *   onCancel (fn)
   *
   * # CR 509.1 — declaring no blocks is legal, and must say so
   *
   * Same shape as `AttackerPicker`'s fix for the S6 review MEDIUM: every blocker
   * defaults to "doesn't block", and the confirm button's own label says
   * "Block with nothing" whenever every blocker is still at that default, so an
   * empty submission can only happen through an explicit, visibly-labelled click.
   *
   * # What this cannot express: CR 509.1b
   *
   * `assignment` is `Map<blockerId, attackerId>` — structurally one attacker per
   * blocker. A creature with "can block an additional creature" can legally be
   * assigned to more than one, and the **server deliberately allows it**:
   * `api.rs::validate_combat_params` rejects only the *identical*
   * `(blocker, attacker)` pair twice, precisely so the validator does not
   * foreclose the exception. So this is a client limitation, not a rules one, and
   * it is recorded in the crate README's Known limitations rather than left as a
   * silent shortfall. Such a blocker can still be assigned once through this UI.
   */
  const {
    eligible = [],
    attackers = [],
    disabled = false,
    onConfirm = null,
    onCancel = null,
  } = $props();

  /** Sentinel for "doesn't block" in the `<select>` below — no attacker has this id. */
  const NO_BLOCK = '__none__';

  /** `blockerId -> attackerId` for every blocker currently assigned to block. */
  let assignment = $state(new Map());

  const assignedCount = $derived(assignment.size);
  const blockingWithNone = $derived(assignedCount === 0);

  function setBlock(blockerId, value) {
    if (disabled) return;
    const next = new Map(assignment);
    if (value === NO_BLOCK) {
      next.delete(blockerId);
    } else {
      next.set(blockerId, Number(value));
    }
    assignment = next;
  }

  function confirm() {
    if (disabled) return;
    onConfirm?.([...assignment.entries()]);
  }
</script>

<div class="blocker-picker">
  <div class="picker-header">
    <span class="picker-title">Declare blockers</span>
    <span class="picker-count">{eligible.length} eligible</span>
  </div>

  {#if eligible.length === 0}
    <span class="no-candidates">no creature is eligible to block</span>
  {:else if attackers.length === 0}
    <span class="no-candidates">no attacker to block</span>
  {:else}
    <div class="blockers">
      {#each eligible as creature (creature.id)}
        <div class="blocker-row">
          <span class="blocker-label">{creature.label}</span>
          <select
            class="block-target"
            disabled={disabled}
            value={assignment.has(creature.id) ? String(assignment.get(creature.id)) : NO_BLOCK}
            onchange={(e) => setBlock(creature.id, e.currentTarget.value)}
          >
            <option value={NO_BLOCK}>doesn't block</option>
            {#each attackers as attacker (attacker.id)}
              <option value={attacker.id}>block {attacker.label}</option>
            {/each}
          </select>
        </div>
      {/each}
    </div>
  {/if}

  <div class="picker-actions">
    <button class="confirm" disabled={disabled} onclick={confirm}>
      {blockingWithNone ? 'Block with nothing' : `Block with ${assignedCount}`}
    </button>
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Back</button>
  </div>
</div>

<style>
  .blocker-picker {
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

  .blockers {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .blocker-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .blocker-label {
    font-size: 0.74rem;
    color: #ccd;
    min-width: 6rem;
  }

  .block-target {
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
