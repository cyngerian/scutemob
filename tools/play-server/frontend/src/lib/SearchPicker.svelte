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
   * UI-6 (`scutemob-194`; `memory/playtest-triage-2026-08-02b.md` G9) turned the
   * wrapped button grid into a scrollable checkable LIST and added the CR 701.23a
   * whole-library look — see "Look and pick are two different lists" below.
   *
   * Props:
   *   prompt (string)      — `BlockingDecisionView.prompt`, seat-redacted server-side
   *   candidates (CardOptionView[]) — `{id, label}`. The engine's answer space, and
   *                          the ONLY ids this component will ever submit.
   *   allCards (CardOptionView[]) — `AnswerShapeView::PickOne::all_cards` (UI-6).
   *                          The searcher's whole library, LOOK-ONLY, already
   *                          sorted by name server-side. Empty for a question that
   *                          carries no look entitlement, in which case this
   *                          renders exactly the pre-UI-6 candidate list.
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
   *   onError (fn(message)) — a failure while building or emitting the answer.
   *                          UI-4 (`scutemob-185`): before this, a throw here
   *                          escaped the click handler and the DOM was simply
   *                          untouched — the button read as dead. Never let this
   *                          path fail in silence again.
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
   * # Look and pick are two different lists, and that is the whole feature
   *
   * The playtest complaint was *"only showed legal basic lands — should be able to
   * view whole library when searching"*. The filter was never the defect:
   * `candidates` IS the engine's answer space, and `handle_answer_effect_choice`
   * refuses anything outside it — offering more as *answers* would be offering
   * illegal ones (SR-38). What was missing is CR 701.23a's **look**: *"To search
   * for a card in a zone, look at all cards in that zone (even if it's a hidden
   * zone)."*
   *
   * So `rows` is the union of `allCards` and `candidates`, and each row carries a
   * `pickable` flag that is `true` **iff its id is in `candidates`**. A look-only
   * row renders as a plain element with a visible `look only` tag and no click
   * target at all — not a disabled button, because a disabled button reads as
   * "temporarily unavailable" and this is a permanent rules fact about the card.
   *
   * `emit` re-checks membership before submitting anything. That is deliberately
   * redundant with the render: a UI-only guard is one refactor away from being the
   * only guard (the same argument `failToFind` already makes about `mayDecline`),
   * and the server's own refusal of a look-only id is a **400**, which reaches the
   * player as a request failure rather than as an explanation.
   *
   * The union runs both ways because containment does not: a "search your library
   * **and** graveyard" effect puts graveyard cards in `candidates` that are in no
   * library, so `allCards` is not a superset. Rows the server did not send in
   * `allCards` are appended and are pickable.
   *
   * # Radio, not multi-select
   *
   * `PickOne` is one card or none. Clicking a candidate REPLACES the selection,
   * which is `TargetPicker`'s `slotMax === 1` behaviour reused deliberately so the
   * two card-choosing surfaces in this client feel the same. Clicking the selected
   * candidate again clears it.
   *
   * # The filters narrow the view, never the answer
   *
   * A 99-card library does not fit on screen, so the list scrolls and a text box
   * narrows it by `label`, case-insensitively. It matches on `label` ONLY, never
   * on `id`: an object id is an engine-internal handle, not something a player
   * knows or should be searching by, and matching it would let a stray digit in a
   * card name pull in unrelated cards.
   *
   * A second, one-click filter hides the look-only rows. It is off by default —
   * showing the whole library is the point of UI-6, and defaulting it on would
   * restore the exact behaviour that was complained about — but a player who has
   * already looked should not have to scroll past 80 Swamps to find their pick.
   *
   * Filtering is purely a display operation. A card that is selected and then
   * filtered out STAYS selected — the header keeps showing what is chosen so the
   * selection can never be invisible, and Confirm keeps working. Clearing the
   * selection when the filter changed would silently discard a deliberate choice.
   *
   * # Untested
   *
   * No frontend test harness exists in this repo (plan §8 R7); nothing in this file
   * is covered by an automated test, though two source gates in `main.rs` pin the
   * `pickable` split and the `onError` wiring. Specifically unexercised: the
   * fail-to-find path (both that it is rendered when `mayDecline` is true and that
   * it is absent when false), the selected-then-filtered-out case described above,
   * the graveyard-search union branch (no `Complete` card in the corpus reaches it
   * through a `PickOne`), and the malformed-template guard, which cannot fire
   * against the real server. `emit`'s membership guard is likewise unexercised at
   * runtime **by construction** — the render gives a look-only card no click
   * target, so nothing reachable from the page can set `chosenId` to one. It is
   * there for the refactor that changes that, and only the source gate holds it.
   *
   * Browser-verified live (UI-6, seed 116, Three Visits, turn 9): 89 rows / 33
   * findable / 56 look-only, the list scrolling 2082px inside 224px, a look-only
   * row rendering as a `DIV` whose click produced 0 POSTs and 0 selection, and a
   * non-default pick posting `{"found":97}` against a server default of `10`.
   */
  import { plainClone } from './plainClone.svelte.js';

  const {
    prompt = '',
    candidates = [],
    allCards = [],
    mayDecline = false,
    template = null,
    foundKey = 'found',
    answerField = 'effect_choice_answer',
    disabled = false,
    onConfirm = null,
    onCancel = null,
    onError = null,
  } = $props();

  /** The chosen object id, or `null` for "nothing chosen yet". */
  let chosenId = $state(null);

  /** Display-only text filter over `label`. */
  let filterText = $state('');

  /** Display-only: hide the CR 701.23a look-only rows. Off by default — see doc. */
  let findableOnly = $state(false);

  /**
   * The answer space, as a lookup. **The single source of truth for `pickable`**:
   * every other decision in this component asks this set, so widening the LOOK can
   * never widen the ANSWER by accident.
   */
  const candidateIds = $derived(new Set(candidates.map((c) => c.id)));

  /**
   * Every card to render, look-only ones included. `allCards` first (the server
   * already sorted it by name), then any candidate the look list did not carry —
   * see the union note in the doc above.
   */
  const rows = $derived.by(() => {
    const seen = new Set();
    const out = [];
    for (const card of allCards) {
      if (seen.has(card.id)) continue;
      seen.add(card.id);
      out.push({ id: card.id, label: card.label, pickable: candidateIds.has(card.id) });
    }
    for (const card of candidates) {
      if (seen.has(card.id)) continue;
      seen.add(card.id);
      out.push({ id: card.id, label: card.label, pickable: true });
    }
    return out;
  });

  const lookOnlyCount = $derived(rows.filter((r) => !r.pickable).length);

  const normalizedFilter = $derived(filterText.trim().toLowerCase());

  const visible = $derived(
    rows.filter(
      (r) =>
        (!findableOnly || r.pickable) &&
        (normalizedFilter === '' || r.label.toLowerCase().includes(normalizedFilter)),
    ),
  );

  const chosenLabel = $derived(
    chosenId === null ? null : (rows.find((r) => r.id === chosenId)?.label ?? `#${chosenId}`),
  );

  const canConfirm = $derived(chosenId !== null);

  function select(id) {
    if (disabled) return;
    // CR 701.23a again, from the other side: looking at a card is not finding it.
    // A look-only row renders no click target, so this is defence in depth — and
    // it is the guard that keeps the answer space pinned to `candidates` if the
    // markup is ever refactored.
    if (!candidateIds.has(id)) return;
    chosenId = chosenId === id ? null : id;
  }

  /**
   * Build and emit the answer. `found` is `null` for a CR 701.23b fail-to-find and
   * the chosen id otherwise — one code path for both, so the decline button cannot
   * drift away from the confirm button's encoding.
   */
  function emit(found) {
    if (disabled) return;
    // CR 701.23a / SR-38: the look widened, the answer space did not. The server
    // refuses a look-only id with a 400, which the player would read as "request
    // failed" — so the refusal is explained here, in the terms of the rule, and
    // nothing is posted. Redundant with the render on purpose; see the doc.
    if (found !== null && !candidateIds.has(found)) {
      onError?.(
        'CR 701.23a: you may look at every card in your library, but this search can ' +
          'only find one of the cards it lists as findable — nothing was submitted',
      );
      return;
    }
    // The malformed-template guards REPORT rather than return in silence (UI-4
    // `/review`). Bailing is still right — a half-built body the server will 400
    // helps nobody — but a silent bail is indistinguishable from the dead button
    // this component was just repaired for, and the whole point of the repair is
    // that the symptom never recurs from any cause.
    if (!template || typeof template !== 'object') {
      onError?.('this search offered no answer template — nothing was submitted');
      return;
    }
    // `plainClone`, never the platform's deep-copy primitive: `template` is a
    // Svelte 5 reactive proxy by the time it gets here, and that primitive
    // rejects proxies with a `DataCloneError`. See `plainClone.svelte.js` — this
    // is the site whose failure was observed in a browser.
    try {
      const answer = plainClone(template);
      const variant = Object.keys(answer)[0];
      // An externally-tagged enum has exactly one key. If it somehow does not, bail
      // rather than write into `undefined` and post a body the server will 400.
      if (variant === undefined || typeof answer[variant] !== 'object') {
        onError?.('the search answer template is not the shape this client can fill in');
        return;
      }
      answer[variant][foundKey] = found;
      onConfirm?.({ [answerField]: answer });
    } catch (err) {
      onError?.(`could not submit the search answer: ${err?.message ?? err}`);
    }
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
    {/if}
    <!-- CR 701.23a stated as two numbers rather than one, so the look/pick split
         is legible before a single row is read. -->
    <span class="picker-count">
      {rows.length} in library · {candidates.length} findable
    </span>
  </div>

  {#if rows.length === 0}
    <span class="no-candidates">this decision offered no cards</span>
  {:else}
    <div class="filters">
      <input
        class="filter"
        type="text"
        placeholder="filter by name…"
        disabled={disabled}
        value={filterText}
        oninput={(e) => (filterText = e.currentTarget.value)}
      />
      {#if lookOnlyCount > 0}
        <label class="findable-toggle">
          <input
            type="checkbox"
            disabled={disabled}
            checked={findableOnly}
            onchange={(e) => (findableOnly = e.currentTarget.checked)}
          />
          hide the {lookOnlyCount} I can't find
        </label>
      {/if}
    </div>

    <div class="candidates">
      {#if visible.length === 0}
        <span class="no-candidates">
          {#if normalizedFilter === ''}
            every card here is look-only
          {:else}
            no card matches "{filterText.trim()}"
          {/if}
        </span>
      {:else}
        {#each visible as card (card.id)}
          {#if card.pickable}
            <button
              class="candidate"
              class:selected={chosenId === card.id}
              disabled={disabled}
              onclick={() => select(card.id)}
            >
              <span class="box">{chosenId === card.id ? '☑' : '☐'}</span>
              <span class="row-label">{card.label}</span>
            </button>
          {:else}
            <!-- Deliberately NOT a disabled button: a disabled control reads as
                 "not right now", and CR 701.23a's distinction is permanent — this
                 card can be looked at and can never be found by this search. -->
            <div class="candidate look-only">
              <span class="box">·</span>
              <span class="row-label">{card.label}</span>
              <span class="look-tag">look only</span>
            </div>
          {/if}
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
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Back</button>
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

  .filters {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .filter {
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

  .findable-toggle {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.68rem;
    color: #889;
    cursor: pointer;
  }

  /* UI-6: a vertical LIST, not a wrapped button grid. The whole library is now
     on screen (~99 rows), and a grid of variable-width chips makes a name
     genuinely hard to find in it — a fixed left edge is what makes scanning
     work. */
  .candidates {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    max-height: 14rem;
    overflow-y: auto;
  }

  .candidate {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
    width: 100%;
    text-align: left;
    padding: 0.12rem 0.4rem;
    font-family: monospace;
    font-size: 0.74rem;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid transparent;
    border-radius: 3px;
    cursor: pointer;
  }

  .candidate .box {
    color: #7a8;
    width: 1ch;
  }

  .candidate .row-label {
    flex: 1 1 auto;
  }

  .candidate:hover:not(:disabled):not(.look-only) {
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

  /* CR 701.23a: visible, readable, and plainly not a control. */
  .candidate.look-only {
    background: #16162a;
    color: #778;
    cursor: default;
  }

  .candidate.look-only .box {
    color: #445;
  }

  .look-tag {
    flex: 0 0 auto;
    font-size: 0.62rem;
    color: #667;
    font-style: italic;
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
