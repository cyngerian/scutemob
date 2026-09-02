<script>
  /**
   * ConfirmPicker — the `Confirm` answer shape: pay an optional cost, or decline
   * (CR 118.12).
   *
   * PB-DX45 (`scutemob-217`; `OOS-DX24-9` ≡ `OOS-DX27-5`). Before this batch the
   * engine paid CR 118.12's optional cost whenever it could, at both of its
   * `try_pay_optional_cost` call sites, so **the decline was not a thing a client
   * could express** — not by being hard to reach, but by not existing. This
   * component is the human end of the channel that makes it exist.
   *
   * Props:
   *   prompt (string)     — `BlockingDecisionView.prompt`, built server-side
   *   costLabel (string)  — `AnswerShapeView::Confirm::cost_label`, e.g. `{B}`,
   *                         `2 life`, `a permanent you sacrifice`. Display only.
   *   template (object)   — the engine's own default answer, serialized verbatim
   *   payKey (string)     — key inside the template's variant object that the
   *                         boolean goes in: `"pay"`
   *   defaultPay (bool)   — the engine's default (`true`, the exact recovery of
   *                         the pre-PB-DX45 auto-pay). Shown as a hint so the
   *                         player can see which button the engine would have
   *                         pressed for them, and so a test can assert the human
   *                         drove the OTHER one.
   *   answerField (string) — `"effect_choice_answer"`
   *   disabled (bool)
   *   onConfirm (fn(params)) — `{[answerField]: <mutated clone of template>}`
   *   onCancel (fn)
   *   onError (fn(message))
   *
   * # The variant name is never typed here
   *
   * Same contract as `SearchPicker` / `PartitionPicker`, and it matters for the
   * same reason: `EffectChoiceAnswer` is an externally-tagged Rust enum, so
   * `template` arrives as `{"PayOptionalCost":{"pay":true}}`. This component
   * clones it, reads its single key, and writes only `payKey`. It never spells
   * `"PayOptionalCost"`, so the wire encoding stays known in exactly one place.
   *
   * # Two buttons, not a checkbox and a Confirm
   *
   * Every other picker in this client collects a selection and then submits it,
   * because every other question has an answer SPACE to browse. This one has two
   * answers. A checkbox-plus-Confirm would add a step whose only content is
   * re-stating the choice just made, and — worse — would give the decline the
   * shape of "not doing anything", which is exactly the reading CR 118.12 says is
   * wrong: declining is a decision, not an absence of one.
   *
   * Neither button is pre-focused or styled as primary. `defaultPay` is rendered
   * as a caption, not as emphasis: the engine's default exists to keep bots and
   * replays behaviourally identical to the pre-batch engine, and nudging a human
   * toward it would re-create the auto-pay this batch removed, one layer up.
   *
   * # Untested
   *
   * No frontend test harness exists in this repo (plan §8 R7); nothing in this
   * file is covered by an automated test. A source gate in `main.rs`
   * (`test_dx45_frontend_answers_the_confirm_shape_without_spelling_the_variant`)
   * pins the shape dispatch and the never-respell-the-variant rule. The
   * malformed-template guards cannot fire against the real server and are
   * unexercised at runtime.
   */
  import { plainClone } from './plainClone.svelte.js';

  const {
    prompt = '',
    costLabel = '',
    template = null,
    payKey = 'pay',
    defaultPay = true,
    answerField = 'effect_choice_answer',
    disabled = false,
    onConfirm = null,
    onCancel = null,
    onError = null,
  } = $props();

  /**
   * Build and emit the answer. ONE code path for both buttons, so the decline can
   * never drift away from the pay button's encoding — the same argument
   * `SearchPicker.emit` makes about its fail-to-find.
   */
  function emit(pay) {
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
        onError?.('the optional-cost answer template is not the shape this client can fill in');
        return;
      }
      answer[variant][payKey] = pay;
      onConfirm?.({ [answerField]: answer });
    } catch (err) {
      onError?.(`could not submit the optional-cost answer: ${err?.message ?? err}`);
    }
  }
</script>

<div class="confirm-picker">
  <div class="picker-header">
    <span class="picker-title">{prompt}</span>
    <span class="picker-count">
      the engine would {defaultPay ? 'pay' : 'decline'} — your call (CR 118.12)
    </span>
  </div>
  <div class="choices">
    <button class="action-btn pay" {disabled} onclick={() => emit(true)}>
      Pay {costLabel}
    </button>
    <button class="action-btn decline" {disabled} onclick={() => emit(false)}> Decline </button>
    <button class="action-btn control" {disabled} onclick={() => onCancel?.()}> Back </button>
  </div>
</div>

<style>
  .confirm-picker {
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

  /* Deliberately the same weight as `.decline`: see "Two buttons" in the doc —
     emphasising the pay button would re-create the auto-pay this batch removed. */
  .pay {
    border-color: #3a5a3a;
  }

  .decline {
    border-color: #5a3a3a;
  }

  .control {
    color: #99a;
  }
</style>
