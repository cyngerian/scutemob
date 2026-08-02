<script>
  /**
   * CostPicker — CR 601.2b's additional costs: the REQUIRED sacrifice (CR 118.8)
   * and the OPTIONAL Squad cost (CR 702.157a), answered in one panel.
   *
   * UI-2 (`scutemob-178`; `memory/playtest-triage-2026-08-02.md` F9). Before this,
   * `StubProvider` did not describe either cost, so the browser offered Life's
   * Legacy on mana affordability alone and `casting.rs` refused it with a 422, and
   * a Squad creature was always cast at `count = 0` with the optional cost
   * silently lost.
   *
   * Props:
   *   prompt (string)      — `AdditionalCostsView.prompt`
   *   sacrifice (SacrificeCostView|null) — `{prompt, candidates, default,
   *                          template, ids_key}`. Candidates are battlefield
   *                          permanents (public under CR 400.1), labelled
   *                          server-side through `NameIndex`.
   *   squad (SquadCostView|null) — `{prompt, cost_label, max_count, template,
   *                          count_key}`
   *   activationSacrifice, activationDiscard (ActivationChoiceView|null) —
   *                          `{prompt, candidates, default, answer_field}`
   *                          (SIM-6, CR 602.2). An ACTIVATED ABILITY's two
   *                          object-naming cost components. Present only for an
   *                          `ActivateAbility` option; the two `CastSpell` props
   *                          above are then null, and vice versa — they come from
   *                          different `LegalAction` variants and never co-occur.
   *   answerField (string) — `"additional_costs"`
   *   disabled (bool)
   *   onConfirm (fn(params)) — `{[answerField]: [<entry>, …]}`
   *   onCancel (fn)
   *   onError (fn(message)) — a failure while building or emitting the answer.
   *                          UI-4 (`scutemob-185`): before this, a throw here
   *                          escaped the click handler and the DOM was simply
   *                          untouched — the button read as dead. Never let this
   *                          path fail in silence again.
   *
   * # Required vs optional, and what "no answer" means
   *
   * The two costs are not symmetric, and the asymmetry is the whole UI:
   *
   * * A sacrifice is REQUIRED (CR 118.8) — `casting.rs` refuses the cast without
   *   one, which is the 422 this batch exists to remove. So there is no "skip"
   *   button and no way to clear the selection: exactly one permanent is always
   *   chosen, starting on the server's own `default` (the engine's deterministic
   *   bot answer), so a human who just wants to cast the spell can press Confirm
   *   and get the same play a bot would make. Confirm is disabled only in the
   *   degenerate case of a sacrifice offer with no candidates, which the provider
   *   suppresses rather than sends.
   * * Squad is OPTIONAL (CR 702.157a: "any number of times", including zero). It
   *   starts at 0 — DECLINING — and a declined Squad contributes **no entry at
   *   all** to the emitted array, so the submitted command is byte-identical to a
   *   plain cast. Sending `Squad { count: 0 }` would also be accepted, but an
   *   absent entry is what every recorded fuzz seed already submits, and keeping
   *   the two identical is why no seed moves.
   *
   * `max_count === 0` is a real and reachable state: the spell is castable but no
   * extra copy is affordable right now. The control is shown and pinned at 0
   * rather than hidden, because hiding it would make an unaffordable Squad
   * indistinguishable from a spell that has none.
   *
   * # Why the variant names are never typed here
   *
   * The same contract `SearchPicker` and `PartitionPicker` state for
   * `EffectChoiceAnswer`, applied to `AdditionalCost`. It is an externally-tagged
   * Rust enum, so `sacrifice.template` arrives as
   * `{"Sacrifice":{"ids":[<default>],"lki":[]}}` and `squad.template` as
   * `{"Squad":{"count":0}}`. This component clones each template, reads its single
   * key, and writes ONLY the field the server named (`ids_key` / `count_key`). It
   * never spells `"Sacrifice"`, `"Squad"` or `"additional_costs"`, so the wire
   * encoding of `AdditionalCost` stays known in exactly one place — the engine —
   * instead of two that can drift apart silently.
   *
   * `lki` is left exactly as it arrives (`[]`) and is never written. `casting.rs`
   * PATCHES it from the layer-resolved characteristics it captures before the zone
   * move (CR 608.2b/608.2h/608.2i); a client-supplied `lki` would be a second
   * opinion about LKI the engine already owns.
   *
   * # Untested
   *
   * No frontend test harness exists in this repo (plan §8 R7); nothing in this
   * file is covered by an automated test. Specifically unexercised: the
   * `max_count === 0` rendering, the both-costs-at-once layout (no card in the
   * corpus carries both), and the malformed-template guards, which cannot fire
   * against the real server. The *channel* is covered end to end by the
   * play-server HTTP probes, which drive a real sacrifice payment and a real
   * Squad count through `POST /api/game/action`.
   */
  import { plainClone } from './plainClone.svelte.js';

  const {
    prompt = '',
    sacrifice = null,
    squad = null,
    activationSacrifice = null,
    activationDiscard = null,
    answerField = 'additional_costs',
    disabled = false,
    onConfirm = null,
    onCancel = null,
    onError = null,
  } = $props();

  /**
   * CR 602.2 (SIM-6): the two ACTIVATION blocks, as one list so the markup and the
   * answer builder walk the same thing.
   *
   * Each entry is an `ActivationChoiceView`: `{prompt, candidates, default,
   * answer_field}`. Both are REQUIRED costs — `handle_activate_ability` refuses the
   * activation without them, which is the 422 this batch exists to remove — so they
   * behave like the CR 118.8 sacrifice above: pre-selected on the server's own
   * default, no skip, no way to clear.
   *
   * The variant-name argument in the module doc does not apply to these two: an
   * activation cost reaches the engine as a bare `ObjectId` on a scalar field, not
   * as an externally-tagged enum, so there is no template to clone and no encoding
   * for this component to know. It still never spells the field name — the server
   * sends it in `answer_field`.
   */
  const activationBlocks = $derived(
    [
      activationSacrifice && { kind: 'sacrifice', view: activationSacrifice },
      activationDiscard && { kind: 'discard', view: activationDiscard },
    ].filter(Boolean),
  );

  /**
   * `answer_field` → the id this human picked, for the activation blocks only.
   * Empty until a click; the effective value falls back to the server's default,
   * for the same reason `chosenId` does above.
   */
  let activationPicked = $state({});

  const activationChoices = $derived(
    activationBlocks.map((block) => ({
      ...block,
      chosen: activationPicked[block.view.answer_field] ?? block.view.default ?? null,
    })),
  );

  /**
   * The permanent the human explicitly clicked, or `null` for "hasn't clicked
   * one". Kept separate from the effective choice below so the pre-selection
   * stays a `$derived` read of the prop rather than a snapshot of it taken at
   * construction — the same reason `DiscardPicker` never seeds `$state` from
   * `defaults`.
   */
  let picked = $state(null);

  /**
   * The permanent to sacrifice: what was clicked, else the server's own
   * `default`. There is no "clear the selection" — CR 118.8 makes this
   * REQUIRED, so a cleared state would be an unsubmittable one with nothing to
   * offer the human in exchange. `null` only when this offer has no sacrifice.
   */
  const chosenId = $derived(picked ?? sacrifice?.default ?? null);

  /** CR 702.157a: how many extra copies to pay for. 0 = decline. */
  let squadCount = $state(0);

  const squadMax = $derived(squad?.max_count ?? 0);

  const chosenLabel = $derived(
    chosenId === null
      ? null
      : ((sacrifice?.candidates ?? []).find((c) => c.id === chosenId)?.label ?? `#${chosenId}`),
  );

  /**
   * CR 118.8: a required sacrifice with nothing to choose FROM cannot be submitted.
   *
   * Gated on `candidates.length`, not on `chosenId !== null`, and the difference is a
   * real bug fix rather than belt-and-braces. `sacrifice.default` is
   * `ObjectId::SENTINEL` when the provider's eligible set is empty, which serialises
   * as the NUMBER `0` — so `chosenId` was non-null and Confirm stayed enabled,
   * submitting `ids: [0]` for a 400. Observed on the wire, not reasoned about: the
   * review reverted the provider's suppression gate and the payload came back
   * `"candidates": [], "default": 0, "template": {"Sacrifice":{"ids":[0],"lki":[]}}`.
   * Unreachable while the suppression gate holds, but a component whose own doc
   * claims this guard should have it.
   */
  const canConfirm = $derived(
    (!sacrifice || ((sacrifice.candidates ?? []).length > 0 && chosenId !== null)) &&
      // CR 602.2 (SIM-6): the same guard, for the same reason, on each activation
      // block. `default` is `ObjectId::SENTINEL` (the number 0) when the provider's
      // eligible set is empty, so gating on `chosen !== null` alone would leave
      // Confirm live and submit `0` for a 400 — the exact bug UI-4's review found on
      // the cast side. Unreachable while the provider's suppression gate holds.
      activationChoices.every(
        (block) => (block.view.candidates ?? []).length > 0 && block.chosen !== null,
      ),
  );

  function select(id) {
    if (disabled) return;
    picked = id;
  }

  /** CR 602.2: pick the object that pays one activation-cost component. */
  function selectActivation(field, id) {
    if (disabled) return;
    activationPicked = { ...activationPicked, [field]: id };
  }

  function setSquad(n) {
    if (disabled) return;
    const clamped = Number.isFinite(n) ? Math.min(Math.max(Math.trunc(n), 0), squadMax) : 0;
    squadCount = clamped;
  }

  /**
   * Clone `template`, take its single variant key, and write `key`. Returns `null`
   * rather than a half-built object if the template is not the shape an
   * externally-tagged enum produces — bailing beats posting a body the server
   * will 400.
   */
  function fillTemplate(template, key, value) {
    if (!template || typeof template !== 'object') return null;
    // `plainClone`, never the platform's deep-copy primitive — `template` is a
    // Svelte 5 reactive proxy here and that primitive rejects proxies with a
    // `DataCloneError`. See `plainClone.svelte.js`; this site is why sacrifice
    // additional costs (CR 118.8) and Squad (CR 702.157a) had never worked in a
    // browser.
    const entry = plainClone(template);
    const variant = Object.keys(entry)[0];
    if (variant === undefined || typeof entry[variant] !== 'object' || entry[variant] === null) {
      return null;
    }
    entry[variant][key] = value;
    return entry;
  }

  function confirm() {
    if (disabled || !canConfirm) return;

    try {
      const entries = [];

      // `fillTemplate` returning null used to bail in silence, which is the same
      // symptom UI-4 repaired arriving from a different cause. It reports now.
      if (sacrifice) {
        const entry = fillTemplate(sacrifice.template, sacrifice.ids_key, [chosenId]);
        if (!entry) {
          onError?.('the sacrifice cost template is not the shape this client can fill in');
          return;
        }
        entries.push(entry);
      }

      // Omitted entirely at 0 — see the module doc. This is the declined case, not
      // a zero-cost payment.
      if (squad && squadCount > 0) {
        const entry = fillTemplate(squad.template, squad.count_key, squadCount);
        if (!entry) {
          onError?.('the Squad cost template is not the shape this client can fill in');
          return;
        }
        entries.push(entry);
      }

      // CR 602.2 (SIM-6): each activation block contributes ONE scalar field, named
      // by the server. `entries` stays empty for an activation (its costs are not
      // `AdditionalCost`s at all), and the array field is still sent — an empty
      // `additional_costs` is what every action that announces none already sends.
      const activationParams = {};
      for (const block of activationChoices) {
        const field = block.view.answer_field;
        if (!field) {
          onError?.('the activation cost offer did not name a field to answer in');
          return;
        }
        activationParams[field] = block.chosen;
      }

      onConfirm?.({ [answerField]: entries, ...activationParams });
    } catch (err) {
      onError?.(`could not submit the additional-cost payment: ${err?.message ?? err}`);
    }
  }
</script>

<div class="cost-picker">
  <div class="picker-header">
    <span class="picker-title">{prompt}</span>
  </div>

  {#if sacrifice}
    <div class="cost-block">
      <div class="cost-label">
        <span class="required">required</span>
        <span class="cost-prompt">{sacrifice.prompt}</span>
        {#if chosenLabel !== null}
          <span class="picker-chosen">sacrificing: {chosenLabel}</span>
        {/if}
      </div>

      {#if (sacrifice.candidates ?? []).length === 0}
        <span class="no-candidates">nothing eligible to sacrifice</span>
      {:else}
        <div class="candidates">
          {#each sacrifice.candidates as card (card.id)}
            <button
              class="candidate"
              class:selected={chosenId === card.id}
              disabled={disabled}
              onclick={() => select(card.id)}
            >
              {card.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#each activationChoices as block (block.view.answer_field)}
    <div class="cost-block">
      <div class="cost-label">
        <span class="required">required</span>
        <span class="cost-prompt">{block.view.prompt}</span>
        {#if block.chosen !== null}
          <span class="picker-chosen">
            {block.kind === 'discard' ? 'discarding' : 'sacrificing'}:
            {(block.view.candidates ?? []).find((c) => c.id === block.chosen)?.label ??
              `#${block.chosen}`}
          </span>
        {/if}
      </div>

      {#if (block.view.candidates ?? []).length === 0}
        <span class="no-candidates">nothing eligible to pay this cost</span>
      {:else}
        <div class="candidates">
          {#each block.view.candidates as card (card.id)}
            <button
              class="candidate"
              class:selected={block.chosen === card.id}
              disabled={disabled}
              onclick={() => selectActivation(block.view.answer_field, card.id)}
            >
              {card.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}

  {#if squad}
    <div class="cost-block">
      <div class="cost-label">
        <span class="optional">optional</span>
        <span class="cost-prompt">{squad.prompt}</span>
        <span class="cost-mana">{squad.cost_label} per extra copy</span>
      </div>

      <div class="squad-row">
        <label for="squad-count-input">extra copies</label>
        <input
          id="squad-count-input"
          type="number"
          min="0"
          max={squadMax}
          step="1"
          disabled={disabled || squadMax === 0}
          value={squadCount}
          oninput={(e) => setSquad(Number.parseInt(e.currentTarget.value, 10))}
        />
        <span class="squad-range">
          {#if squadMax === 0}
            no extra copies are affordable right now — casting without Squad
          {:else}
            0–{squadMax} affordable · 0 declines the cost
          {/if}
        </span>
      </div>
    </div>
  {/if}

  <div class="picker-actions">
    <button class="confirm" disabled={disabled || !canConfirm} onclick={confirm}>Confirm</button>
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Cancel</button>
  </div>
</div>

<style>
  .cost-picker {
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

  .cost-block {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .cost-label {
    display: flex;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .cost-prompt {
    font-size: 0.74rem;
    color: #ccd;
  }

  .cost-mana {
    font-size: 0.7rem;
    color: #cb8;
  }

  .required,
  .optional {
    font-size: 0.6rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-radius: 2px;
    padding: 0 0.2rem;
  }

  .required {
    color: #f9a;
    border: 1px solid #6a2a40;
  }

  .optional {
    color: #8c8;
    border: 1px solid #2a5a3a;
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

  .candidates {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
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

  .squad-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .squad-row label {
    font-size: 0.72rem;
    color: #aab;
  }

  .squad-row input {
    width: 4.5rem;
    background: #141428;
    border: 1px solid #33335a;
    border-radius: 3px;
    color: #ccd;
    font-family: monospace;
    font-size: 0.74rem;
    padding: 0.15rem 0.3rem;
  }

  .squad-row input:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .squad-range {
    font-size: 0.68rem;
    color: #667;
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
