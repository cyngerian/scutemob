<script>
  /**
   * TargetPicker — CR 601.2c target announcement, one selector per slot.
   *
   * M11-local Session 7 (`memory/m11-session-plan.md` §4, item 3).
   *
   * Props:
   *   slots (TargetSlotView[]) — one entry per target slot, in the exact order
   *     `Command::CastSpell`/`Command::ActivateAbility`'s `targets` vector must be
   *     in. Each is `{min, max, candidates}`; each candidate's `.value` is the
   *     engine's own serialized `Target` (`{"Object":12}` / `{"Player":2}`) —
   *     echoed back verbatim, never rebuilt.
   *   min (number), max (number) — the COLLECTIVE range from
   *     `mtg_engine::target_count_range`, equal to the sum of the slots' own.
   *   disabled (bool) — a request is in flight
   *   onConfirm (fn(targets, perSlot)) — `targets` is a `Target[]` in slot order,
   *     with a slot contributing between its own `min` and `max` entries.
   *     `perSlot` is the SAME data grouped, `Target[][]`, one inner array per slot
   *     — see "Two shapes of the same announcement" below. A caller that wants the
   *     flat CR 601.2c announcement ignores the second argument, exactly as the
   *     original single-argument caller does.
   *   onCancel (fn)
   *
   * # A slot is a requirement, and a requirement is not always worth one target
   *
   * `TargetRequirement::UpToN { count }` is a **single** requirement admitting up
   * to `count` targets — `casting::target_count_range` adds `count` to the
   * maximum for it, and `validate_targets_inner`'s second pass assigns several
   * announced targets to that one slot. So a slot is multi-select up to its own
   * `max`, not a radio button.
   *
   * The first version of this component held one candidate index per slot and
   * therefore capped `force_of_vigor` — `Complete`, deck-legal, one
   * `UpToN { count: 2 }` requirement — at destroying **one** of its "up to two"
   * targets. Caught in review, not by a test: no seeded game in the S7 fixture
   * sweep dealt such a card, so this multi-select path ships **unexercised**.
   *
   * # Why "candidates within a slot", not "any card, then match a slot"
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
   * # Two shapes of the same announcement (UI-1, `scutemob-174`)
   *
   * A cast/activation announces a FLAT `targets` vector (`Command::CastSpell`), but
   * a trigger's target announcement is `trigger_targets: Vec<Vec<Target>>` — one
   * array per slot (`AnswerShapeView::Slots`, `ActionParamsDto::trigger_targets`).
   * Both are answered by this exact picker over the same `Vec<TargetSlotView>`.
   *
   * The flat list CANNOT be regrouped after the fact: a slot may contribute between
   * its own `min` and `max` entries (see the header), so the per-slot counts are
   * not recoverable from a concatenation. Rather than have the caller guess them —
   * which is correct only while every slot happens to be exactly-one, and silently
   * wrong the first time an `UpToN` trigger slot appears — `confirm` builds the
   * grouped form FIRST and derives the flat one from it, then hands back both.
   * There is exactly one grouping in existence and the two arguments cannot
   * disagree, because one is `.flat()` of the other.
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
   * Selected candidate indices per slot, as an array of arrays — a slot may hold
   * up to its own `max` (see the header). Length is fixed to `slots.length` for
   * the life of this component instance; `TargetPicker` is mounted fresh per
   * action (`ActionBar`'s picker chain unmounts it when the chain moves on), so
   * there is no stale-length hazard across different actions.
   *
   * `untrack` marks this a deliberate one-time read of `slots` at mount, not a
   * dependency `picked` should stay reactive to — the fixed length above is the
   * whole point.
   */
  let picked = $state(untrack(() => slots.map(() => [])));

  /** Total targets currently announced, across every slot. */
  const filledCount = $derived(picked.reduce((n, p) => n + p.length, 0));

  /**
   * CR 601.2c. Every slot must be within its OWN range, and the total within the
   * collective one. The two are not redundant: the collective range is the sum
   * of the slots', so a total inside it can still put a mandatory slot at zero.
   */
  const slotsInRange = $derived(
    picked.every((p, i) => p.length >= (slots[i]?.min ?? 0) && p.length <= (slots[i]?.max ?? 1)),
  );
  const canConfirm = $derived(slotsInRange && filledCount >= min && filledCount <= max);

  function selectCandidate(slotIndex, candidateIndex) {
    if (disabled) return;
    const slotMax = slots[slotIndex]?.max ?? 1;
    const current = picked[slotIndex];
    let next;
    if (current.includes(candidateIndex)) {
      next = current.filter((c) => c !== candidateIndex);
    } else if (slotMax === 1) {
      // A single-target slot behaves as a radio button: picking replaces.
      next = [candidateIndex];
    } else if (current.length < slotMax) {
      next = [...current, candidateIndex];
    } else {
      // At capacity. Deselect something first rather than silently dropping the
      // oldest pick, which would look like the click did nothing in particular.
      return;
    }
    picked = picked.map((p, i) => (i === slotIndex ? next : p));
  }

  function confirm() {
    if (disabled || !canConfirm) return;
    // Grouped first, flat derived from it — see "Two shapes of the same
    // announcement" above. Ascending candidate order within a slot, so the
    // announced list is a deterministic function of the selection rather than of
    // click order.
    const perSlot = slots.map((slot, i) =>
      [...picked[i]].sort((a, b) => a - b).map((c) => slot.candidates[c].value),
    );
    onConfirm?.(perSlot.flat(), perSlot);
  }

  function slotRangeText(slot) {
    if (slot.min === slot.max) return slot.min === 1 ? '' : `exactly ${slot.min}`;
    return `up to ${slot.max}`;
  }
</script>

<div class="target-picker">
  <div class="picker-header">
    <span class="picker-title">Choose target{slots.length === 1 ? '' : 's'}</span>
    <span class="picker-range">{min === max ? `exactly ${min}` : `${min}–${max}`}</span>
  </div>

  <div class="slots">
    {#each slots as slot, slotIndex (slotIndex)}
      <div class="slot">
        <span class="slot-label">
          Target {slotIndex + 1}
          {#if slotRangeText(slot)}<span class="slot-range">{slotRangeText(slot)}</span>{/if}
        </span>
        {#if slot.candidates.length === 0}
          <span class="no-candidates">no legal target for this slot</span>
        {:else}
          <div class="candidates">
            {#each slot.candidates as candidate, candidateIndex (candidateIndex)}
              <button
                class="candidate"
                class:selected={picked[slotIndex].includes(candidateIndex)}
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

  .slot-range {
    margin-left: 0.3rem;
    font-size: 0.6rem;
    color: #667;
    border: 1px solid #33335a;
    border-radius: 2px;
    padding: 0 0.2rem;
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
