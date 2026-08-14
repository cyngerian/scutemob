<script>
  /**
   * DiscardPicker — the `Subset` AND `PickN` answer shapes: choose EXACTLY
   * `count` of `candidates`.
   *
   * UI-1 (`scutemob-174`; `memory/playtest-triage-2026-08-02.md` F8). Originally
   * the only question that arrived in this shape was the CR 514.1 cleanup
   * discard (`LegalAction::DiscardToHandSize`, `Subset`), and this component was
   * written against the SHAPE, not the question — `view.rs`'s `AnswerShapeView`
   * doc makes that the explicit contract. ENG-1 is the predicted reuse: an
   * effect-driven discard (CR 701.9b) arrives as `PickN`, which answers through
   * `effect_choice_answer` and a `template` rather than through a bare
   * `discard_cards` array. Both shapes render identically; only `confirm()`'s
   * output differs, branched on whether `template` was sent.
   *
   * Props:
   *   prompt (string)      — `BlockingDecisionView.prompt`, already seat-redacted
   *                          and rendered server-side. Displayed verbatim; this
   *                          component never composes prompt text of its own about
   *                          *which* cards are involved.
   *   candidates (CardOptionView[]) — `{id, label}`. For the cleanup discard this
   *                          is the player's WHOLE hand, not a pre-trimmed subset;
   *                          same for the ENG-1 effect-driven discard.
   *   count (number)       — choose AT MOST this many (the maximum). `api.rs`
   *                          rejects a length outside `[minCount, count]` with a
   *                          400 ("CR 514.1 requires exactly N" / "CR 701.9b: this
   *                          effect discards exactly N card(s)" / PB-DX28's
   *                          CR 115.10 "up to N" wording).
   *   minCount (number)    — the fewest legal (PB-DX28, `AnswerShapeView::PickN::
   *                          min_count`). Defaults to `count` — every PRE-PB-DX28
   *                          use (CR 514.1, CR 701.9b) is exact-count, so the two
   *                          are equal and this component's existing exact-count
   *                          behaviour is unchanged unless a caller passes a
   *                          smaller `minCount` (a PB-DX28 "up to N" untargeted
   *                          choice, `EffectChoiceQuestion::ChooseObject` with
   *                          `up_to: true`).
   *   defaults (number[])  — the engine's OWN default subset. For `Subset`,
   *                          `default_cleanup_discard`: the `count` HIGHEST
   *                          `ObjectId`s. For `PickN`, `default_discard_answer`:
   *                          the `count` LOWEST `ObjectId`s — the opposite end of
   *                          the hand, see that function's doc for why the two
   *                          auto-picks genuinely differ. Named `defaults` because
   *                          `default` is a reserved word in JS and cannot be
   *                          destructured.
   *   answerField (string) — `BlockingDecisionView.answer_field`: `"discard_cards"`
   *                          for `Subset`, `"effect_choice_answer"` for `PickN`.
   *                          Taken from the payload rather than hardcoded here:
   *                          `view.rs` sends this field precisely so the client is
   *                          not a second place that has to know the
   *                          `ActionParamsDto` schema.
   *   template (object|null) — `null` for `Subset` (unchanged, bare-array path).
   *                          For `PickN` (ENG-1, CR 701.9b), the engine's own
   *                          default answer, serialized verbatim — see
   *                          `SearchPicker`'s doc for the "why the variant name is
   *                          never typed here" contract, reused verbatim below.
   *   chosenKey (string)   — `AnswerShapeView::PickN::chosen_key`, i.e. `"chosen"`
   *                          — the key inside `template`'s variant object the
   *                          chosen ids go in. Unused when `template` is `null`.
   *   disabled (bool)      — a request is in flight
   *   onConfirm (fn(params)) — for `Subset`, `{[answerField]: [id, ...]}`; for
   *                          `PickN`, `{[answerField]: <mutated clone of template>}`
   *                          — which `ActionBar` merges into the chain's accumulator
   *   onCancel (fn)
   *   onError (fn(message)) — a failure while building or emitting a `PickN`
   *                          answer. UI-4 (`scutemob-185`): before that batch, a
   *                          throw here escaped the click handler and the DOM was
   *                          simply untouched — the button read as dead. Never let
   *                          this path fail in silence again. Unused on the
   *                          `Subset` path, which cannot fail this way.
   *
   * # Ascending ids, not click order — correct for `Subset`, a stated deviation
   * # for `PickN`
   *
   * The submitted list is sorted ascending, the same argument `TargetPicker`
   * makes: the submission should be a function of WHICH cards were selected, not
   * of the order the human happened to click them. For `Subset` this is simply
   * correct — `check_ids` in `api.rs` treats the list as a set (membership + no
   * duplicates), so order carries no meaning on the wire.
   *
   * For `PickN` this is, IN PRINCIPLE, wrong: CR 608.2f / CR 404.3 make discard
   * order a real player payload (it is the relative order the cards enter the
   * graveyard), and `EffectChoiceAnswer::Discard::chosen`'s own doc says so. This
   * component ships ascending-ids anyway, because the engine's own `api.rs` check
   * treats `chosen` as a set too (membership + no duplicates, not order) and no
   * card in the corpus reads graveyard order — but the deviation is real and is
   * seeded as `OOS-ENG1-7` rather than left to be rediscovered.
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
   * pick), the "use the default" button, the degenerate `count === 0` case
   * (which the server never sends — `DiscardToHandSize` is only offered when the
   * hand is over the maximum, and `PickN`'s short-circuit means the engine never
   * sends `count === 0` either — and which would render as an immediately
   * confirmable empty choice), (ENG-1) the `PickN`/`template` path and its
   * malformed-template guard, and (PB-DX28) the `minCount < count` "up to N"
   * range display and the ability to Confirm with FEWER than `count` selected.
   */
  import { plainClone } from './plainClone.svelte.js';

  const {
    prompt = '',
    candidates = [],
    count = 0,
    minCount = count,
    defaults = [],
    answerField = 'discard_cards',
    template = null,
    chosenKey = 'chosen',
    disabled = false,
    onConfirm = null,
    onCancel = null,
    onError = null,
  } = $props();

  /** Chosen object ids, in click order; re-sorted ascending on confirm. */
  let selected = $state([]);

  const canConfirm = $derived(selected.length >= minCount && selected.length <= count);

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

  /**
   * `Subset` (`template === null`) submits a bare id array unchanged. `PickN`
   * (ENG-1) submits a mutated clone of `template` — the exact `SearchPicker.emit`
   * body: `plainClone` (never `structuredClone` — the UI-4 `DataCloneError`
   * defect), take the enum's single key, guard that the value under it is an
   * object, write `chosenKey`, report any failure via `onError` rather than
   * bailing in silence (the UI-4 `/review` lesson).
   */
  function confirm() {
    if (disabled || !canConfirm) return;
    if (template === null) {
      onConfirm?.({ [answerField]: [...selected].sort((a, b) => a - b) });
      return;
    }
    if (typeof template !== 'object') {
      onError?.('this discard offered no answer template — nothing was submitted');
      return;
    }
    try {
      const answer = plainClone(template);
      const variant = Object.keys(answer)[0];
      // An externally-tagged enum has exactly one key. If it somehow does not,
      // bail rather than write into `undefined` and post a body the server will
      // 400.
      if (variant === undefined || typeof answer[variant] !== 'object') {
        onError?.('the discard answer template is not the shape this client can fill in');
        return;
      }
      answer[variant][chosenKey] = [...selected].sort((a, b) => a - b);
      onConfirm?.({ [answerField]: answer });
    } catch (err) {
      onError?.(`could not submit the discard answer: ${err?.message ?? err}`);
    }
  }
</script>

<div class="discard-picker">
  <div class="picker-header">
    <span class="picker-title">{prompt}</span>
    <span class="picker-count" class:satisfied={canConfirm}
      >{selected.length}/{minCount < count ? `${minCount}-${count}` : count}</span
    >
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
      {minCount < count ? `Confirm ${selected.length} chosen` : `Discard ${selected.length} of ${count}`}
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
