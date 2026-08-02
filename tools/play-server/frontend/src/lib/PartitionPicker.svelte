<script>
  /**
   * PartitionPicker — the `Partition` answer shape: split `lookedAt` into two
   * piles. Covers BOTH scry (CR 701.22a) and surveil (CR 701.25a).
   *
   * UI-1 (`scutemob-174`; `memory/playtest-triage-2026-08-02.md` F8). Before this,
   * the browser resolved every scry as a no-op because the client submitted the
   * engine's identity-partition default (`view.rs`, `BlockingDecisionView` doc).
   *
   * One component for two questions is the point, not a shortcut: the difference
   * between a scry and a surveil is entirely in `movedKey`/`movedLabel`, which the
   * server supplies. Nothing in this file names "scry" or "surveil".
   *
   * Props:
   *   prompt (string)      — `BlockingDecisionView.prompt`, seat-redacted server-side
   *   lookedAt (CardOptionView[]) — `{id, label}`, TOP-FIRST (`Zone::top_n`'s order)
   *   keptKey (string)     — key inside the template for the pile that stays on
   *                          the library: always `"top"`
   *   movedKey (string)    — `"bottom"` (scry) or `"graveyard"` (surveil)
   *   movedLabel (string)  — prose for the other pile's heading
   *   template (object)    — the engine's own default answer, serialized verbatim
   *   answerField (string) — `"effect_choice_answer"`; see the note below on why
   *                          this is a prop and not a literal
   *   disabled (bool)
   *   onConfirm (fn(params)) — `{[answerField]: <mutated clone of template>}`
   *   onCancel (fn)
   *   onError (fn(message)) — a failure while building or emitting the answer.
   *                          UI-4 (`scutemob-185`): before this, a throw here
   *                          escaped the click handler and the DOM was simply
   *                          untouched — the button read as dead. Never let this
   *                          path fail in silence again.
   *
   * # Why the variant name is never typed here
   *
   * `EffectChoiceAnswer` is an externally-tagged Rust enum, so `template` arrives
   * looking like `{"Scry":{"bottom":[],"top":[20,21]}}`. This component clones it,
   * reads its single key, and replaces only the two arrays named by `keptKey` and
   * `movedKey`. It therefore never spells `"Scry"` or `"Surveil"` itself, and the
   * wire encoding of `EffectChoiceAnswer` stays known in exactly ONE place — the
   * engine. A client that built `{"Scry": ...}` from scratch would be a second
   * place for that encoding to drift, and the drift would be silent right up until
   * `api.rs`'s "you answered the wrong question" 400. `answerField` is a prop for
   * the same reason applied one level up: `view.rs` sends `answer_field` precisely
   * so the client is not a second place that has to know the params schema.
   *
   * # Order is meaningful in BOTH piles, and only one of them matters much
   *
   * `keptKey`'s pile is ordered: its first entry becomes the top card of the
   * library, so the human must be able to reorder it. Hence the ▲/▼ buttons and
   * the header line saying the first entry ends up on top.
   *
   * The moved pile's order is also transmitted, and for a surveil it is the order
   * the cards are put into the graveyard (CR 608.2f: the player performing the
   * action chooses the order for simultaneous moves, which matters for
   * graveyard-order-sensitive cards and for "leaves the graveyard" ordering). This
   * component keeps that simple deliberately: the moved pile is in the order the
   * human moved the cards, with no reorder controls. That is a real choice being
   * made implicitly rather than explicitly, and saying so here is the honest
   * version of shipping it.
   *
   * # Every partition is legal, so Confirm is never disabled
   *
   * `check_partition` in `api.rs` accepts any split of `lookedAt` — including the
   * identity partition "keep everything", which is both the engine's default and
   * this component's initial state (CR 701.22a's own default). There is no
   * invalid arrangement to guard against, so the Confirm button has no disabled
   * condition beyond `disabled`.
   *
   * # Untested
   *
   * No frontend test harness exists in this repo (plan §8 R7), so nothing here is
   * covered by an automated test. Specifically unexercised: the reorder buttons at
   * the ends of the kept pile (first ▲ / last ▼ are no-ops), moving a card to the
   * other pile and back (it returns to the BOTTOM of the kept pile, not its
   * original index — a reorder the human can undo but was not asked about), and
   * the malformed-template guard, which cannot fire against the real server.
   */
  import { untrack } from 'svelte';
  import { plainClone } from './plainClone.svelte.js';

  const {
    prompt = '',
    lookedAt = [],
    keptKey = 'top',
    movedKey = 'bottom',
    movedLabel = 'bottom of library',
    template = null,
    answerField = 'effect_choice_answer',
    disabled = false,
    onConfirm = null,
    onCancel = null,
    onError = null,
  } = $props();

  /**
   * Ids in the kept pile, TOP-FIRST. Seeded with every card in the order the
   * server sent, which is the identity partition — the engine's own default and
   * CR 701.22a's ("you may put any number on the bottom": zero is a valid any).
   *
   * `untrack` marks the seeding a deliberate one-time read of `lookedAt` at mount.
   * This component is mounted fresh per decision by `ActionBar`'s picker chain, so
   * there is no stale-content hazard across different questions.
   */
  let kept = $state(untrack(() => lookedAt.map((c) => c.id)));

  /** Ids in the other pile, in the order the human moved them (see the header). */
  let moved = $state([]);

  /** `id -> label`, so each pile can render without re-scanning `lookedAt`. */
  const labelById = $derived(new Map(lookedAt.map((c) => [c.id, c.label])));

  function labelOf(id) {
    return labelById.get(id) ?? `#${id}`;
  }

  function toMoved(id) {
    if (disabled) return;
    kept = kept.filter((k) => k !== id);
    moved = [...moved, id];
  }

  function toKept(id) {
    if (disabled) return;
    moved = moved.filter((m) => m !== id);
    // Appended, not restored to its original index — see the module doc.
    kept = [...kept, id];
  }

  function moveUp(index) {
    if (disabled || index <= 0) return;
    const next = [...kept];
    [next[index - 1], next[index]] = [next[index], next[index - 1]];
    kept = next;
  }

  function moveDown(index) {
    if (disabled || index >= kept.length - 1) return;
    const next = [...kept];
    [next[index], next[index + 1]] = [next[index + 1], next[index]];
    kept = next;
  }

  function confirm() {
    if (disabled) return;
    // Reports rather than returning in silence — see `SearchPicker.emit`'s note:
    // a silent bail is indistinguishable from the dead button UI-4 repaired.
    if (!template || typeof template !== 'object') {
      onError?.('this decision offered no answer template — nothing was submitted');
      return;
    }
    // `plainClone`, never the platform's deep-copy primitive — `template` is a
    // Svelte 5 reactive proxy here and that primitive rejects proxies with a
    // `DataCloneError`. See `plainClone.svelte.js`; this site is why scry
    // (CR 701.22a) and surveil (CR 701.25a) had never worked in a browser.
    try {
      const answer = plainClone(template);
      const variant = Object.keys(answer)[0];
      // An externally-tagged enum has exactly one key. If it somehow does not, bail
      // rather than write into `undefined` and post a body the server will 400.
      if (variant === undefined || typeof answer[variant] !== 'object') {
        onError?.('the answer template is not the shape this client can fill in');
        return;
      }
      answer[variant][keptKey] = [...kept];
      answer[variant][movedKey] = [...moved];
      onConfirm?.({ [answerField]: answer });
    } catch (err) {
      onError?.(`could not submit the card-ordering answer: ${err?.message ?? err}`);
    }
  }
</script>

<div class="partition-picker">
  <div class="picker-header">
    <span class="picker-title">{prompt}</span>
    <span class="picker-note">first card in "Keep on top" ends up on top of your library</span>
  </div>

  {#if lookedAt.length === 0}
    <span class="no-candidates">this decision offered no cards</span>
  {:else}
    <div class="piles">
      <div class="pile">
        <span class="pile-label">Keep on top <span class="pile-count">{kept.length}</span></span>
        {#if kept.length === 0}
          <span class="pile-empty">nothing stays on the library</span>
        {:else}
          <div class="pile-rows">
            {#each kept as id, index (id)}
              <div class="pile-row">
                <span class="pile-index">{index + 1}</span>
                <span class="card-label">{labelOf(id)}</span>
                <button
                  class="nudge"
                  disabled={disabled || index === 0}
                  title="Move toward the top"
                  onclick={() => moveUp(index)}>▲</button
                >
                <button
                  class="nudge"
                  disabled={disabled || index === kept.length - 1}
                  title="Move away from the top"
                  onclick={() => moveDown(index)}>▼</button
                >
                <button class="shift" disabled={disabled} onclick={() => toMoved(id)}>
                  → {movedLabel}
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="pile">
        <span class="pile-label">{movedLabel} <span class="pile-count">{moved.length}</span></span>
        {#if moved.length === 0}
          <span class="pile-empty">nothing moved</span>
        {:else}
          <div class="pile-rows">
            {#each moved as id, index (id)}
              <div class="pile-row">
                <span class="pile-index">{index + 1}</span>
                <span class="card-label">{labelOf(id)}</span>
                <button class="shift" disabled={disabled} onclick={() => toKept(id)}>
                  ← keep on top
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <div class="picker-actions">
    <button class="confirm" disabled={disabled} onclick={confirm}>
      {moved.length === 0
        ? 'Keep everything on top'
        : `Keep ${kept.length}, move ${moved.length}`}
    </button>
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Back</button>
  </div>
</div>

<style>
  .partition-picker {
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

  .picker-note {
    font-size: 0.62rem;
    color: #667;
  }

  .no-candidates {
    font-size: 0.72rem;
    color: #766;
    font-style: italic;
  }

  .piles {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
  }

  .pile {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 14rem;
    flex: 1;
  }

  .pile-label {
    font-size: 0.68rem;
    color: #778;
  }

  .pile-count {
    font-size: 0.6rem;
    color: #667;
    border: 1px solid #33335a;
    border-radius: 2px;
    padding: 0 0.2rem;
  }

  .pile-empty {
    font-size: 0.68rem;
    color: #667;
    font-style: italic;
  }

  .pile-rows {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .pile-row {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .pile-index {
    font-size: 0.62rem;
    color: #556;
    min-width: 1rem;
  }

  .card-label {
    flex: 1;
    font-size: 0.74rem;
    color: #ccd;
    background: #1c1c38;
    border: 1px solid #33335a;
    border-radius: 3px;
    padding: 0.2rem 0.45rem;
  }

  .nudge {
    padding: 0.2rem 0.3rem;
    font-size: 0.68rem;
    line-height: 1;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
  }

  .nudge:hover:not(:disabled) {
    background: #2a2a58;
    border-color: #4a4a90;
  }

  .nudge:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .shift {
    padding: 0.2rem 0.45rem;
    font-size: 0.7rem;
    background: #1c1c38;
    color: #aab;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
    white-space: nowrap;
  }

  .shift:hover:not(:disabled) {
    background: #2a2a58;
    border-color: #4a4a90;
  }

  .shift:disabled {
    opacity: 0.4;
    cursor: not-allowed;
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
