<script>
  /**
   * SearchPicker — the `PickOne` answer shape: choose one of `candidates`, or
   * none iff `mayDecline` (CR 701.23a/b).
   *
   * UI-1 (`scutemob-174`; `memory/playtest-triage-2026-08-02.md` F8). Before this,
   * the browser submitted the engine's default — `candidates.first()` — so every
   * tutor in a human game fetched whatever happened to be at the front of the
   * candidate list.
   *
   * Props:
   *   prompt (string)      — `BlockingDecisionView.prompt`, seat-redacted server-side
   *   candidates (CardOptionView[]) — `{id, label}`. This is a LIBRARY search, so
   *                          the list can be ~99 entries long; see the filter note.
   *   mayDecline (bool)    — `AnswerShapeView::PickOne::may_decline`. CR 701.23b:
   *                          only a search "for a card with a stated quality" may
   *                          fail to find. When false, CR 701.23d makes finding
   *                          MANDATORY and `api.rs` 400s a `found: null` answer, so
   *                          the button is not merely disabled — it is not rendered.
   *   template (object)    — the engine's own default answer, serialized verbatim
   *   foundKey (string)    — key inside the template's variant object that the
   *                          chosen id goes in: `"found"`
   *   answerField (string) — `"effect_choice_answer"`
   *   disabled (bool)
   *   onConfirm (fn(params)) — `{[answerField]: <mutated clone of template>}`
   *   onCancel (fn)
   *
   * # Why the variant name is never typed here
   *
   * Same contract as `PartitionPicker`, and worth restating rather than
   * cross-referencing, because it is the one rule in these components that is easy
   * to "simplify" into a bug. `EffectChoiceAnswer` is an externally-tagged Rust
   * enum, so `template` arrives as `{"SearchLibrary":{"found":30}}`. This component
   * clones it, reads its single key, and writes only `foundKey`. It never spells
   * `"SearchLibrary"`, so the wire encoding of `EffectChoiceAnswer` stays known in
   * exactly one place — the engine — instead of two that can drift apart silently.
   *
   * # Radio, not multi-select
   *
   * `PickOne` is one card or none. Clicking a candidate REPLACES the selection,
   * which is `TargetPicker`'s `slotMax === 1` behaviour reused deliberately so the
   * two card-choosing surfaces in this client feel the same. Clicking the selected
   * candidate again clears it.
   *
   * # The filter narrows the view, never the answer
   *
   * A 99-card library does not fit on screen, so the list scrolls and a text box
   * narrows it by `label`, case-insensitively. It matches on `label` ONLY, never
   * on `id`: an object id is an engine-internal handle, not something a player
   * knows or should be searching by, and matching it would let a stray digit in a
   * card name pull in unrelated cards.
   *
   * Filtering is purely a display operation. A card that is selected and then
   * filtered out STAYS selected — the header keeps showing what is chosen so the
   * selection can never be invisible, and Confirm keeps working. Clearing the
   * selection when the filter changed would silently discard a deliberate choice.
   *
   * # Untested
   *
   * No frontend test harness exists in this repo (plan §8 R7); nothing in this file
   * is covered by an automated test. Specifically unexercised: the fail-to-find
   * path (both that it is rendered when `mayDecline` is true and that it is absent
   * when false), the filter box against a genuinely long candidate list, the
   * selected-then-filtered-out case described above, and the malformed-template
   * guard, which cannot fire against the real server.
   */
  const {
    prompt = '',
    candidates = [],
    mayDecline = false,
    template = null,
    foundKey = 'found',
    answerField = 'effect_choice_answer',
    disabled = false,
    onConfirm = null,
    onCancel = null,
  } = $props();

  /** The chosen object id, or `null` for "nothing chosen yet". */
  let chosenId = $state(null);

  /** Display-only text filter over `label`. */
  let filterText = $state('');

  const normalizedFilter = $derived(filterText.trim().toLowerCase());

  const visible = $derived(
    normalizedFilter === ''
      ? candidates
      : candidates.filter((c) => c.label.toLowerCase().includes(normalizedFilter)),
  );

  const chosenLabel = $derived(
    chosenId === null ? null : (candidates.find((c) => c.id === chosenId)?.label ?? `#${chosenId}`),
  );

  const canConfirm = $derived(chosenId !== null);

  function select(id) {
    if (disabled) return;
    chosenId = chosenId === id ? null : id;
  }

  /**
   * Build and emit the answer. `found` is `null` for a CR 701.23b fail-to-find and
   * the chosen id otherwise — one code path for both, so the decline button cannot
   * drift away from the confirm button's encoding.
   */
  function emit(found) {
    if (disabled) return;
    if (!template || typeof template !== 'object') return;
    const answer = structuredClone(template);
    const variant = Object.keys(answer)[0];
    // An externally-tagged enum has exactly one key. If it somehow does not, bail
    // rather than write into `undefined` and post a body the server will 400.
    if (variant === undefined || typeof answer[variant] !== 'object') return;
    answer[variant][foundKey] = found;
    onConfirm?.({ [answerField]: answer });
  }

  function confirm() {
    if (!canConfirm) return;
    emit(chosenId);
  }

  function failToFind() {
    // Guarded here as well as at the render site: CR 701.23d says the server will
    // refuse this answer whenever `mayDecline` is false, and a UI-only guard is one
    // refactor away from being the only guard.
    if (!mayDecline) return;
    emit(null);
  }
</script>

<div class="search-picker">
  <div class="picker-header">
    <span class="picker-title">{prompt}</span>
    {#if chosenLabel !== null}
      <span class="picker-chosen">chosen: {chosenLabel}</span>
    {:else}
      <span class="picker-count">{candidates.length} card{candidates.length === 1 ? '' : 's'}</span>
    {/if}
  </div>

  {#if candidates.length === 0}
    <span class="no-candidates">this decision offered no cards</span>
  {:else}
    <input
      class="filter"
      type="text"
      placeholder="filter by name…"
      disabled={disabled}
      value={filterText}
      oninput={(e) => (filterText = e.currentTarget.value)}
    />

    <div class="candidates">
      {#if visible.length === 0}
        <span class="no-candidates">no card matches "{filterText.trim()}"</span>
      {:else}
        {#each visible as card (card.id)}
          <button
            class="candidate"
            class:selected={chosenId === card.id}
            disabled={disabled}
            onclick={() => select(card.id)}
          >
            {card.label}
          </button>
        {/each}
      {/if}
    </div>
  {/if}

  <div class="picker-actions">
    <button class="confirm" disabled={disabled || !canConfirm} onclick={confirm}>Confirm</button>
    {#if mayDecline}
      <button
        class="secondary"
        disabled={disabled}
        title="CR 701.23b: this search may legally find nothing"
        onclick={failToFind}
      >
        Fail to find
      </button>
    {/if}
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Cancel</button>
  </div>
</div>

<style>
  .search-picker {
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
    flex-wrap: wrap;
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

  .picker-chosen {
    font-size: 0.68rem;
    color: #8c8;
  }

  .no-candidates {
    font-size: 0.72rem;
    color: #766;
    font-style: italic;
  }

  .filter {
    align-self: flex-start;
    width: 16rem;
    max-width: 100%;
    background: #141428;
    border: 1px solid #33335a;
    border-radius: 3px;
    color: #ccd;
    font-family: monospace;
    font-size: 0.74rem;
    padding: 0.15rem 0.3rem;
  }

  .candidates {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    /* A library search can offer ~99 cards; scroll rather than push the action
       row off the bottom of the bar. */
    max-height: 11rem;
    overflow-y: auto;
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

  .secondary {
    padding: 0.2rem 0.5rem;
    font-size: 0.76rem;
    background: #1c1c38;
    color: #aab;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
  }

  .secondary:hover:not(:disabled) {
    background: #2a2a58;
    border-color: #4a4a90;
  }

  .secondary:disabled {
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
