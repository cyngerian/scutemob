<script>
  /**
   * TargetPicker — CR 601.2c target announcement, one selector per slot.
   *
   * M11-local Session 7 (`memory/m11-session-plan.md` §4, item 3).
   *
   * Props:
   *   slots (TargetOptionView[][]) — outer index = slot, in the exact order
   *     `Command::CastSpell`/`Command::ActivateAbility`'s `targets` vector must
   *     be in. Each entry's `.value` is the engine's own serialized `Target`
   *     (`{"Object":12}` / `{"Player":2}`) — echoed back verbatim, never rebuilt.
   *   min (number), max (number) — from `mtg_engine::target_count_range`. `max`
   *     exceeds `min` only for a `TargetRequirement::UpToN` slot (CR 601.2c), in
   *     which case a later slot may be left unfilled and the pick still confirms.
   *   disabled (bool) — a request is in flight
   *   onConfirm (fn(targets)) — `targets` is a `Target[]`, one entry per slot the
   *     human actually filled, in slot order, skipping any left empty under `UpToN`
   *   onCancel (fn)
   *
   * # Why "one candidate per slot", not "any card, then match a slot"
   *
   * `legal_targets_per_slot` (`crates/engine/src/rules/queries.rs`) already ran
   * every relevant check — hexproof, shroud, protection, the requirement's own
   * filter — per slot, so the candidate list handed to this component is exactly
   * the legal set. This component picks *from* that set; it never re-derives
   * legality, and it never lets the human select a candidate outside the slot it
   * was enumerated for (a creature legal for slot 0 is not necessarily legal for
   * slot 1 — e.g. `TargetPermanentDistinctFrom` — and the server is the one place
   * that actually enforces distinctness, per `queries.rs`'s own "advisory" doc
   * comment).
   *
   * # Labels are never re-derived
   *
   * Every `label` here came from the server's seat-redacted `NameIndex`
   * (`view.rs` module doc). This component only ever displays `label` — it does
   * not fall back to formatting `id` when a label exists, and it does not try to
   * identify a hidden candidate from its id (a hidden object's id may be a
   * sentinel, e.g. redacted hand cards all reporting `object_id: 0` — see
   * `PlayApp.svelte`'s `isUnidentifiable` doc for where that is documented in
   * this codebase).
   */
  import { untrack } from 'svelte';

  const { slots = [], min = 0, max = 0, disabled = false, onConfirm = null, onCancel = null } =
    $props();

  /**
   * One selected candidate index per slot, or `null` if that slot is still
   * unfilled. Length is fixed to `slots.length` for the life of this component
   * instance — `TargetPicker` is mounted fresh per action (`ActionBar`'s picker
   * chain keys it away when the chain moves on), so there is no stale-length
   * hazard across different actions.
   *
   * `untrack` tells the compiler this is a deliberate one-time read of `slots`
   * at mount, not a dependency `picked` should stay reactive to — the whole
   * point of the fixed length above is that it does NOT track subsequent
   * changes to the `slots` prop.
   */
  let picked = $state(untrack(() => slots.map(() => null)));

  /** How many slots currently have a selection. */
  const filledCount = $derived(picked.filter((p) => p !== null).length);

  /**
   * CR 601.2c range check. Below `min` the spell cannot legally be cast/activated
   * yet; at or above it, and never exceeding `max` (each slot holds at most one
   * candidate index, so `filledCount` cannot exceed `slots.length`, and `max` is
   * never less than `slots.length` for a well-formed payload), confirming is
   * allowed.
   */
  const canConfirm = $derived(filledCount >= min && filledCount <= max);

  function selectCandidate(slotIndex, candidateIndex) {
    if (disabled) return;
    const next = picked.slice();
    next[slotIndex] = next[slotIndex] === candidateIndex ? null : candidateIndex;
    picked = next;
  }

  function confirm() {
    if (disabled || !canConfirm) return;
    const targets = [];
    for (let i = 0; i < slots.length; i++) {
      const choice = picked[i];
      if (choice !== null) targets.push(slots[i][choice].value);
    }
    onConfirm?.(targets);
  }
</script>

<div class="target-picker">
  <div class="picker-header">
    <span class="picker-title">Choose target{slots.length === 1 ? '' : 's'}</span>
    <span class="picker-range">{min === max ? `exactly ${min}` : `${min}–${max}`}</span>
  </div>

  <div class="slots">
    {#each slots as candidates, slotIndex (slotIndex)}
      <div class="slot">
        <span class="slot-label">Target {slotIndex + 1}</span>
        {#if candidates.length === 0}
          <span class="no-candidates">no legal target for this slot</span>
        {:else}
          <div class="candidates">
            {#each candidates as candidate, candidateIndex (candidateIndex)}
              <button
                class="candidate"
                class:selected={picked[slotIndex] === candidateIndex}
                disabled={disabled}
                onclick={() => selectCandidate(slotIndex, candidateIndex)}
              >
                {candidate.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <div class="picker-actions">
    <button class="confirm" disabled={disabled || !canConfirm} onclick={confirm}>
      Confirm ({filledCount}/{max})
    </button>
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Cancel</button>
  </div>
</div>

<style>
  .target-picker {
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

  .picker-range {
    font-size: 0.68rem;
    color: #667;
  }

  .slots {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .slot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
  }

  .slot-label {
    font-size: 0.68rem;
    color: #778;
    min-width: 4.5rem;
  }

  .no-candidates {
    font-size: 0.72rem;
    color: #766;
    font-style: italic;
  }

  .candidates {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .candidate {
    padding: 0.2rem 0.45rem;
    font-size: 0.74rem;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
  }

  .candidate:hover:not(:disabled) {
    background: #2a2a58;
    border-color: #4a4a90;
  }

  .candidate:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .candidate.selected {
    background: #23386a;
    border-color: #3a5aa0;
    color: #dde;
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
