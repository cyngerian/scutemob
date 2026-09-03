<script>
  /**
   * BinaryChoicePicker — the `BinaryChoice` answer shape: a two-way choice that
   * is **not** a cost.
   *
   * PB-DX50 (`scutemob-221`; `OOS-DX29-2`). Its only question today is
   * CR 702.140c's mutate over/under: *"As a mutating creature spell resolves, if
   * its target is legal … The spell's controller chooses whether the spell is put
   * on top of the creature or on the bottom."* Before this batch that decision was
   * taken at ANNOUNCEMENT (`LegalAction::CastWithMutate` carried an `on_top`
   * boolean and the offer layer emitted one action per `(target, on_top)` pair),
   * so the opponent saw the choice before deciding whether to respond and the
   * controller could not change it afterwards.
   *
   * Props:
   *   prompt (string)      — `BlockingDecisionView.prompt`, built server-side
   *   trueLabel (string)   — the button that submits `true` (e.g. "On top of Adrix")
   *   falseLabel (string)  — the button that submits `false` (e.g. "Under Adrix")
   *   template (object)    — the engine's own default answer, serialized verbatim
   *   choiceKey (string)   — key inside the template's variant object that the
   *                          boolean goes in: `"on_top"`
   *   defaultChoice (bool) — the engine's default (`true` for CR 702.140c, the
   *                          exact recovery of the pre-PB-DX50 hard-coded value).
   *                          Shown as a hint so the player can see which button
   *                          the engine would have pressed for them, and so a test
   *                          can assert the human drove the OTHER one.
   *   answerField (string) — `"effect_choice_answer"`
   *   disabled (bool)
   *   onConfirm (fn(params)) — `{[answerField]: <mutated clone of template>}`
   *   onCancel (fn)
   *   onError (fn(message))
   *
   * # Why this is not `ConfirmPicker`
   *
   * The two shapes carry the same information — a template, a boolean key and a
   * default — and reusing `ConfirmPicker` would have compiled and worked. It
   * renders "Pay {cost}" and "Decline", and CR 702.140c's question is *over or
   * under*: nothing is paid, nothing is declined, and neither answer is the
   * passive one. That would be a truthful payload behind a false label, which is
   * the defect class this project keeps filing. Here the server names both
   * buttons and this component renders exactly what it is told.
   *
   * # The variant name is never typed here
   *
   * Same contract as `ConfirmPicker` / `SearchPicker` / `PartitionPicker`, for the
   * same reason: `EffectChoiceAnswer` is an externally-tagged Rust enum, so
   * `template` arrives as `{"MutateOnTop":{"on_top":true}}`. This component clones
   * it, reads its single key, and writes only `choiceKey`. It never spells
   * `"MutateOnTop"`, so the wire encoding stays known in exactly one place.
   *
   * Neither button is pre-focused or styled as primary, and `defaultChoice` is a
   * caption rather than emphasis — `ConfirmPicker`'s argument, for the same
   * reason: the engine's default exists to keep bots and replays behaviourally
   * identical to the pre-batch engine, and nudging a human toward it would
   * re-create the hard-coded choice this batch removed, one layer up.
   *
   * # Untested
   *
   * No frontend test harness exists in this repo (plan §8 R7); nothing in this
   * file is covered by an automated test. Source gates in `main.rs`
   * (`test_dx50_frontend_answers_the_binary_choice_shape_without_spelling_the_variant`)
   * pin the shape dispatch and the never-respell-the-variant rule. The
   * malformed-template guards cannot fire against the real server and are
   * unexercised at runtime.
   */
  import { plainClone } from './plainClone.svelte.js';

  const {
    prompt = '',
    trueLabel = 'Yes',
    falseLabel = 'No',
    template = null,
    choiceKey = 'on_top',
    defaultChoice = true,
    answerField = 'effect_choice_answer',
    disabled = false,
    onConfirm = null,
    onCancel = null,
    onError = null,
  } = $props();

  /**
   * Build and emit the answer. ONE code path for both buttons, so neither can
   * drift away from the other's encoding — `ConfirmPicker.emit`'s argument.
   */
  function emit(choice) {
    if (disabled) return;
    // The malformed-template guards REPORT rather than return in silence (UI-4
    // `/review`): a silent bail is indistinguishable from a dead button, which is
    // the symptom UI-4 was dispatched to remove.
    if (!template || typeof template !== 'object') {
      onError?.('this decision offered no answer template — nothing was submitted');
      return;
    }
    try {
      // `plainClone`, never the platform's deep-copy primitive: `template` is a
      // Svelte 5 reactive proxy by the time it gets here, and that primitive
      // rejects proxies with a `DataCloneError` (UI-4, observed in a browser).
      const answer = plainClone(template);
      const variant = Object.keys(answer)[0];
      if (variant === undefined || typeof answer[variant] !== 'object') {
        onError?.('the binary-choice answer template is not the shape this client can fill in');
        return;
      }
      answer[variant][choiceKey] = choice;
      onConfirm?.({ [answerField]: answer });
    } catch (err) {
      onError?.(`could not submit the binary-choice answer: ${err?.message ?? err}`);
    }
  }
</script>

<div class="binary-choice-picker">
  <div class="picker-header">
    <span class="picker-title">{prompt}</span>
    <span class="picker-count">
      the engine would choose "{defaultChoice ? trueLabel : falseLabel}" — your call
    </span>
  </div>
  <div class="choices">
    <!-- The labels are written without surrounding whitespace inside the element
         on purpose: `main.rs`'s UI-5 gate matches the literal `>Back</button>`
         across every picker, and it caught `ConfirmPicker`'s first draft, which
         pretty-printed the label onto its own line. -->
    <button class="action-btn opt-true" {disabled} onclick={() => emit(true)}
      >{trueLabel}</button
    >
    <button class="action-btn opt-false" {disabled} onclick={() => emit(false)}
      >{falseLabel}</button
    >
    <button class="action-btn control" {disabled} onclick={() => onCancel?.()}>Back</button>
  </div>
</div>

<style>
  .binary-choice-picker {
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

  .choices {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .action-btn {
    font-family: monospace;
    font-size: 0.72rem;
    padding: 0.25rem 0.6rem;
    border-radius: 3px;
    border: 1px solid #2a2a44;
    background: #1c1c3a;
    color: #cce;
    cursor: pointer;
  }

  .action-btn:hover:not(:disabled) {
    background: #26264a;
  }

  .action-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  /* Deliberately the same weight: see "Why this is not ConfirmPicker" in the doc —
     neither answer is the passive one, so neither button is the primary one. */
  .opt-true {
    border-color: #3a4a5a;
  }

  .opt-false {
    border-color: #3a4a5a;
  }

  .control {
    color: #99a;
  }
</style>
