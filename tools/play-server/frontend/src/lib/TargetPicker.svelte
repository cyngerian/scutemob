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
   *   humanName (string|null) — `summary.human_name`; the seat whose segment
   *     sorts first. Purely cosmetic — see "Segmented by player" below.
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
   *
   * # Segmented by player (UI-3, `scutemob-180`)
   *
   * Playtest note: "target selector should have segments broken up by player."
   * With four seats on the board a flat row of candidates is a wall of card
   * names with no way to tell whose creature you are about to Murder.
   *
   * The segment key is the server's `TargetOptionView.owner`, derived from the
   * same already-redacted view model every `label` comes from. It is **not**
   * re-derived here from the board — that would be a second opinion about which
   * seat an object belongs to, and it would be wrong for exactly the cases that
   * matter (a permanent stolen with Control Magic sits in its *controller's*
   * battlefield map, which is the association the engine targets by).
   *
   * Grouping is presentational and cannot change what is submitted: the value
   * carried through selection is still the candidate's index into the slot's own
   * `candidates` array, so `confirm` emits `slot.candidates[c].value` in
   * ascending index order exactly as before. A candidate with a missing `owner`
   * is not dropped — it lands in a trailing unlabelled segment, so a server that
   * stopped sending the field would degrade to one flat group rather than to an
   * empty picker.
   */
  import { untrack } from 'svelte';

  const {
    slots = [],
    min = 0,
    max = 0,
    humanName = null,
    disabled = false,
    onConfirm = null,
    onCancel = null,
  } = $props();

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

  /** Label for a segment with no `owner` — see the header's degradation note. */
  const NO_OWNER = ' no-owner';

  /**
   * Group one slot's candidates by `owner`, carrying each candidate's ORIGINAL
   * index so selection state is untouched by the grouping.
   *
   * Segment order: the human's own segment first (it is the one you reach for
   * most, and "which of these is mine" is the question the flat list could not
   * answer), then every other owner in the order the engine first mentioned
   * them, then the unlabelled segment last. First-appearance order rather than
   * alphabetical, so the segments track the engine's own candidate order and two
   * renders of the same payload cannot differ.
   */
  function segmentsFor(slot) {
    const byOwner = new Map();
    (slot.candidates ?? []).forEach((candidate, index) => {
      const key = candidate?.owner ?? NO_OWNER;
      if (!byOwner.has(key)) byOwner.set(key, []);
      byOwner.get(key).push({ candidate, index });
    });

    const keys = [...byOwner.keys()];
    keys.sort((a, b) => {
      if (a === b) return 0;
      // Unlabelled always last.
      if (a === NO_OWNER) return 1;
      if (b === NO_OWNER) return -1;
      // Your own seat always first.
      if (humanName !== null) {
        if (a === humanName) return -1;
        if (b === humanName) return 1;
      }
      // Otherwise: stable, and first-mentioned wins.
      return 0;
    });

    return keys.map((key) => ({
      key,
      owner: key === NO_OWNER ? null : key,
      isHuman: humanName !== null && key === humanName,
      entries: byOwner.get(key),
    }));
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
          <div class="segments">
            {#each segmentsFor(slot) as segment (segment.key)}
              <div class="segment" class:mine={segment.isHuman}>
                <span class="segment-owner">
                  {segment.owner ?? 'elsewhere'}
                  {#if segment.isHuman}<span class="you">you</span>{/if}
                </span>
                <div class="candidates">
                  {#each segment.entries as entry (entry.index)}
                    <button
                      class="candidate"
                      class:selected={picked[slotIndex].includes(entry.index)}
                      disabled={disabled}
                      onclick={() => selectCandidate(slotIndex, entry.index)}
                    >
                      {entry.candidate.label}
                    </button>
                  {/each}
                </div>
              </div>
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
    align-items: flex-start;
    gap: 0.4rem;
  }

  /* UI-3: one block per owning seat, so "whose creature is this" is legible. */
  .segments {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem 0.6rem;
    flex: 1;
    min-width: 0;
  }

  .segment {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.15rem 0.3rem;
    border-left: 2px solid #2a2a4a;
    min-width: 0;
  }

  .segment.mine {
    border-left-color: #3a5aa0;
  }

  .segment-owner {
    display: flex;
    align-items: baseline;
    gap: 0.25rem;
    font-size: 0.62rem;
    color: #667;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .segment.mine .segment-owner {
    color: #7ac;
  }

  .you {
    color: #6af;
    border: 1px solid #2a4a7a;
    border-radius: 2px;
    padding: 0 0.15rem;
    letter-spacing: 0;
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
