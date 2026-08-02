<script>
  /**
   * DiscardPicker — the `Subset` answer shape: choose EXACTLY `count` of
   * `candidates`.
   *
   * UI-1 (`scutemob-174`; `memory/playtest-triage-2026-08-02.md` F8). Today the
   * only question that arrives in this shape is the CR 514.1 cleanup discard
   * (`LegalAction::DiscardToHandSize`), but this component is written against the
   * SHAPE, not the question — `view.rs`'s `AnswerShapeView` doc makes that the
   * explicit contract, so a second "choose exactly N of these cards" question
   * would reuse this component with no new client code.
   *
   * Props:
   *   prompt (string)      — `BlockingDecisionView.prompt`, already seat-redacted
   *                          and rendered server-side. Displayed verbatim; this
   *                          component never composes prompt text of its own about
   *                          *which* cards are involved.
   *   candidates (CardOptionView[]) — `{id, label}`. For the cleanup discard this
   *                          is the player's WHOLE hand, not a pre-trimmed subset.
   *   count (number)       — choose exactly this many. `api.rs` rejects any other
   *                          length with a 400 ("CR 514.1 requires exactly N").
   *   defaults (number[])  — the engine's OWN default subset
   *                          (`default_cleanup_discard`: the `count` highest
   *                          `ObjectId`s). Named `defaults` because `default` is a
   *                          reserved word in JS and cannot be destructured.
   *   answerField (string) — `BlockingDecisionView.answer_field`, i.e.
   *                          `"discard_cards"`. Taken from the payload rather than
   *                          hardcoded here: `view.rs` sends this field precisely
   *                          so the client is not a second place that has to know
   *                          the `ActionParamsDto` schema.
   *   disabled (bool)      — a request is in flight
   *   onConfirm (fn(params)) — a params fragment, `{[answerField]: [id, ...]}`,
   *                          which `ActionBar` merges into the chain's accumulator
   *   onCancel (fn)
   *
   * # Ascending ids, not click order
   *
   * The submitted list is sorted ascending, the same argument `TargetPicker`
   * makes: the submission should be a function of WHICH cards were selected, not
   * of the order the human happened to click them. `check_ids` in `api.rs` treats
   * the list as a set (membership + no duplicates), so order carries no meaning
   * on the wire and letting it vary would only make two identical choices look
   * like different requests in a replay log.
   *
   * # "Use the default" fills the selection, it does not submit
   *
   * Clicking it selects exactly the ids in `defaults` and stops. The whole reason
   * UI-1 exists is that the pre-UI-1 client rendered one bare button that silently
   * submitted the engine's default — "the game discarded the right-hand cards for
   * you" (`view.rs`, `BlockingDecisionView` doc). A one-click path back to that
   * default is genuinely useful, but it must SHOW what it picked and still require
   * a deliberate Confirm, or it is the same failure with a longer label.
   *
   * `defaults` is intersected with `candidates` before being applied. The engine's
   * default is by construction drawn from the hand it also sent, so the
   * intersection is expected to be the identity; it is there so that a payload
   * where they disagree degrades to "selects fewer than `count`, Confirm stays
   * disabled" rather than to an invisible selection of a card with no button.
   *
   * # Untested
   *
   * There is no frontend test harness in this repo (plan §8 R7), so NOTHING in
   * this file is covered by an automated test. Specifically unexercised: the
   * capacity behaviour when `count` cards are already selected and a further
   * candidate is clicked (the click is ignored rather than evicting an earlier
   * pick), the "use the default" button, and the degenerate `count === 0` case
   * (which the server never sends — `DiscardToHandSize` is only offered when the
   * hand is over the maximum — and which would render as an immediately
   * confirmable empty choice).
   */
  const {
    prompt = '',
    candidates = [],
    count = 0,
    defaults = [],
    answerField = 'discard_cards',
    disabled = false,
    onConfirm = null,
    onCancel = null,
  } = $props();

  /** Chosen object ids, in click order; re-sorted ascending on confirm. */
  let selected = $state([]);

  const canConfirm = $derived(selected.length === count);

  /** Ids that actually have a button, for the `defaults` intersection above. */
  const candidateIds = $derived(new Set(candidates.map((c) => c.id)));

  const defaultsApplicable = $derived(defaults.length > 0);

  function toggle(id) {
    if (disabled) return;
    if (selected.includes(id)) {
      selected = selected.filter((s) => s !== id);
    } else if (selected.length < count) {
      selected = [...selected, id];
    }
    // At capacity: ignore. Silently evicting an earlier pick would make the click
    // look like it did nothing in particular (`TargetPicker` makes the same call).
  }

  function useDefault() {
    if (disabled) return;
    selected = defaults.filter((id) => candidateIds.has(id));
  }

  function confirm() {
    if (disabled || !canConfirm) return;
    onConfirm?.({ [answerField]: [...selected].sort((a, b) => a - b) });
  }
</script>

<div class="discard-picker">
  <div class="picker-header">
    <span class="picker-title">{prompt}</span>
    <span class="picker-count" class:satisfied={canConfirm}>{selected.length}/{count}</span>
  </div>

  {#if candidates.length === 0}
    <span class="no-candidates">this decision offered no cards</span>
  {:else}
    <div class="candidates">
      {#each candidates as card (card.id)}
        <button
          class="candidate"
          class:selected={selected.includes(card.id)}
          disabled={disabled}
          onclick={() => toggle(card.id)}
        >
          {card.label}
        </button>
      {/each}
    </div>
  {/if}

  <div class="picker-actions">
    <button class="confirm" disabled={disabled || !canConfirm} onclick={confirm}>
      Discard {selected.length} of {count}
    </button>
    {#if defaultsApplicable}
      <button
        class="secondary"
        disabled={disabled}
        title="Select the cards the engine would have chosen — you still have to confirm"
        onclick={useDefault}
      >
        Use the default
      </button>
    {/if}
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Back</button>
  </div>
</div>

<style>
  .discard-picker {
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

  .picker-count.satisfied {
    color: #8c8;
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
