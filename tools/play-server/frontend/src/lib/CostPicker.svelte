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
   *   counts (CountCostView[]) — PB-DX29. `{kind, prompt, cost_label,
   *                          max_count, template, count_key}` per entry, one per
   *                          pay-N-times rider (Replicate CR 702.56a, Escalate
   *                          CR 702.120a). Absent from the payload when empty —
   *                          `view.rs` skips the field — so the default matters.
   *   markers (MarkerCostView[]) — PB-DX29. `{kind, prompt, cost_label,
   *                          template}` per entry, one per pay-or-not rider
   *                          (Entwine CR 702.42a, Fuse CR 702.102a, Offspring
   *                          CR 702.175a). See "the marker templates are not
   *                          objects" below — this family is answered
   *                          differently from every other one.
   *   gift (GiftCostView|null) — PB-DX29, CR 702.174a. `{prompt, gift_label,
   *                          candidates, template, player_key}`. Candidates are
   *                          SEATS, not cards; player identity is public.
   *   splice (SpliceCostView|null) — PB-DX29, CR 702.47a. `{prompt, candidates,
   *                          template, ids_key}`. The only genuinely
   *                          MULTI-select cost: the answer is a list of card ids
   *                          from this seat's own hand.
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
   * # The marker templates are NOT objects, and the difference is a wire fact
   *
   * PB-DX29. `AdditionalCost::Entwine`, `::Fuse` and `::Offspring` are Rust UNIT
   * variants, and serde's externally-tagged encoding serialises a unit variant as a
   * bare JSON **string** — `"Entwine"` — not as an object with one key. So the
   * clone-and-write-one-field idiom described above has nothing to write into: on a
   * marker it would read a key off a string (`Object.keys` of a string yields its
   * character indices, so the "variant" would come back as `"0"`) and then assign
   * into a primitive.
   *
   * For this family **the template IS the whole answer**: push it verbatim to pay,
   * push nothing to decline. The template filler is never handed a marker, and
   * `test_frontend_cost_picker_never_fills_a_unit_variant_marker_template` in
   * `tools/play-server/src/main.rs` fails the build if that ever changes. A marker
   * whose template arrives as anything but a bare string is REPORTED through
   * `onError`, not coerced: a shape change on the server is a thing a human should
   * be told about, not something this client should paper over into a 400.
   *
   * Fuse is also the one cost with no figure to show. CR 702.102b makes a fused
   * spell's cost the two halves SUMMED, so `cost_label` arrives `null` and the UI
   * says so in words — rendering `{0}` would be a lie about a real mana cost.
   *
   * # Declining every optional rider must cost nothing
   *
   * Six of the eight families here are optional (Squad, both counts, all three
   * markers, gift, splice), and every one of them follows Squad's rule: a declined
   * rider contributes **no entry at all**, so an answer with everything declined is
   * byte-identical to a plain cast. Contributing `Replicate { count: 0 }` or
   * `Splice { cards: [] }` instead would be accepted by
   * `validate_additional_cost_params` and then read by `casting.rs` as a payment of
   * nothing — the same value by a different route, and a different set of bytes for
   * every recorded seed to disagree with.
   *
   * That is also why gift starts with NOTHING selected, unlike the CR 118.8
   * sacrifice, which starts on the server's default. A pre-selected gift would
   * promise an opponent a card, a Treasure or an extra turn (CR 702.174d-i) that
   * the human never chose, and Confirm alone would give it away.
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
    counts = [],
    markers = [],
    gift = null,
    splice = null,
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

  /**
   * PB-DX29 — the four new families, as lists the markup and the answer builder
   * both walk, so a rendered widget and a contributed entry cannot get out of step.
   *
   * `counts` and `markers` are `#[serde(skip_serializing_if = "Vec::is_empty")]`
   * server-side, so they are ABSENT rather than `[]` on the overwhelming majority
   * of casts. The prop default covers `undefined`; `?? []` covers a future `null`.
   */
  const countList = $derived(counts ?? []);
  const markerList = $derived(markers ?? []);
  const giftCandidates = $derived(gift?.candidates ?? []);
  const spliceCandidates = $derived(splice?.candidates ?? []);

  /**
   * Per-entry answers for the two list families, keyed by POSITION in the list.
   *
   * Position, not `kind`: `kind` is unique in every offer the provider builds
   * today, but a keyed map would collide silently the day it is not, and the
   * failure would be one rider paying another rider's count. The list is a prop
   * fixed for the life of this component instance — `ActionBar` rebuilds the whole
   * chain when the option changes — so a position is stable while it matters.
   *
   * Absent key = 0 / not paid. Neither map is ever seeded from the server, because
   * neither rider has a server-side default: they are optional and the decline is
   * the empty answer (see the module doc).
   */
  let countValues = $state({});
  let markerPaid = $state({});

  /**
   * CR 702.174a: the seat promised the gift, or `null` for "no gift promised".
   *
   * `null` and not `undefined`, and every read of it compares with `!== null`
   * rather than testing truthiness — `PlayerId(0)` is a real seat and is falsy.
   * That is the same class of bug as `ObjectId::SENTINEL` serialising as the number
   * `0`, which UI-4's review caught leaving Confirm live over an empty candidate
   * set.
   */
  let giftPicked = $state(null);

  /** CR 702.47a: the cards to splice, in click order. Empty = decline. */
  let splicePicked = $state([]);

  const chosenGiftLabel = $derived(
    giftPicked === null
      ? null
      : (giftCandidates.find((p) => p.id === giftPicked)?.label ?? `#${giftPicked}`),
  );

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

  /** CR 702.56a / CR 702.120a: how many times to pay rider `index`. Same clamp as Squad. */
  function setCount(index, n) {
    if (disabled) return;
    const max = countList[index]?.max_count ?? 0;
    const clamped = Number.isFinite(n) ? Math.min(Math.max(Math.trunc(n), 0), max) : 0;
    countValues = { ...countValues, [index]: clamped };
  }

  /** The count actually in force for rider `index` — unanswered means declined. */
  function countOf(index) {
    return countValues[index] ?? 0;
  }

  /** CR 702.42a / CR 702.102a / CR 702.175a: pay this rider, or stop paying it. */
  function toggleMarker(index) {
    if (disabled) return;
    markerPaid = { ...markerPaid, [index]: markerPaid[index] !== true };
  }

  /**
   * CR 702.174a: name the seat that gets the gift, or clear it.
   *
   * Clicking the chosen seat again clears it back to "no gift promised", which the
   * required sacrifice above deliberately does NOT allow. The asymmetry is the
   * asymmetry between a required cost and an optional one: a misclick here would
   * otherwise be an irreversible promise of a card or an extra turn.
   */
  function selectGift(id) {
    if (disabled) return;
    giftPicked = giftPicked === id ? null : id;
  }

  /** CR 702.47a: add or remove one card from the splice list. */
  function toggleSplice(id) {
    if (disabled) return;
    splicePicked = splicePicked.includes(id)
      ? splicePicked.filter((other) => other !== id)
      : [...splicePicked, id];
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

      // PB-DX29, CR 702.56a / CR 702.120a: the pay-N-times riders, on exactly the
      // rule Squad uses one block up — omitted at 0, because that is the decline
      // and not a payment of nothing.
      for (let i = 0; i < countList.length; i += 1) {
        const count = countList[i];
        const n = countOf(i);
        if (n <= 0) continue;
        const entry = fillTemplate(count.template, count.count_key, n);
        if (!entry) {
          onError?.(`the ${count.kind} cost template is not the shape this client can fill in`);
          return;
        }
        entries.push(entry);
      }

      // PB-DX29, CR 702.42a / CR 702.102a / CR 702.175a: the pay-or-not riders.
      // These are serde UNIT variants — a bare JSON string, not an object — so the
      // template is pushed VERBATIM and the template filler is never called on one.
      // See the module doc; a shape change is reported rather than coerced.
      for (let i = 0; i < markerList.length; i += 1) {
        const marker = markerList[i];
        if (markerPaid[i] !== true) continue;
        if (typeof marker.template !== 'string') {
          onError?.(
            `the ${marker.kind} cost did not arrive as the bare tag this client can pay; ` +
              'the server changed the encoding of a unit variant',
          );
          return;
        }
        entries.push(plainClone(marker.template));
      }

      // PB-DX29, CR 702.174a. `!== null` and not a truth test: seat 0 is a real
      // player and is falsy, so `if (giftPicked)` would silently refuse to promise
      // a gift to the first seat at the table.
      if (gift && giftPicked !== null) {
        const entry = fillTemplate(gift.template, gift.player_key, giftPicked);
        if (!entry) {
          onError?.('the gift cost template is not the shape this client can fill in');
          return;
        }
        entries.push(entry);
      }

      // PB-DX29, CR 702.47a: the only multi-select cost. Omitted when nothing is
      // checked — an empty list would be accepted and then splice nothing.
      if (splice && splicePicked.length > 0) {
        const entry = fillTemplate(splice.template, splice.ids_key, [...splicePicked]);
        if (!entry) {
          onError?.('the splice cost template is not the shape this client can fill in');
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

  <!-- PB-DX29, CR 702.56a / CR 702.120a: the pay-N-times riders. Same control as
       Squad, and pinned at 0 rather than hidden when nothing is affordable, for the
       same reason: a hidden widget makes "cannot afford it right now" look
       identical to "this spell has no such cost". -->
  {#each countList as entry, i (i)}
    <div class="cost-block">
      <div class="cost-label">
        <span class="optional">optional</span>
        <span class="cost-prompt">{entry.prompt}</span>
        <span class="cost-mana">{entry.cost_label} each time</span>
      </div>

      <div class="squad-row">
        <label for={`count-input-${i}`}>{entry.kind} payments</label>
        <input
          id={`count-input-${i}`}
          type="number"
          min="0"
          max={entry.max_count ?? 0}
          step="1"
          disabled={disabled || (entry.max_count ?? 0) === 0}
          value={countOf(i)}
          oninput={(e) => setCount(i, Number.parseInt(e.currentTarget.value, 10))}
        />
        <span class="squad-range">
          {#if (entry.max_count ?? 0) === 0}
            none are affordable right now — casting without this cost
          {:else}
            0–{entry.max_count} affordable · 0 declines the cost
          {/if}
        </span>
      </div>
    </div>
  {/each}

  <!-- PB-DX29, CR 702.42a / CR 702.102a / CR 702.175a: the pay-or-not riders. -->
  {#each markerList as entry, i (i)}
    <div class="cost-block">
      <div class="cost-label">
        <span class="optional">optional</span>
        <span class="cost-prompt">{entry.prompt}</span>
        {#if entry.cost_label}
          <span class="cost-mana">{entry.cost_label}</span>
        {:else if entry.kind === 'Fuse'}
          <!-- CR 702.102b: the cost IS both halves added together, so there is no
               separate figure. Saying so beats rendering a zero mana cost, which
               would be a lie about a real cost. -->
          <span class="cost-mana">costs both halves' mana costs, added together</span>
        {:else}
          <span class="cost-mana">the server sent no cost for this rider</span>
        {/if}
      </div>

      <label class="marker-row">
        <input
          type="checkbox"
          disabled={disabled}
          checked={markerPaid[i] === true}
          onchange={() => toggleMarker(i)}
        />
        pay the {entry.kind} cost
      </label>
    </div>
  {/each}

  <!-- PB-DX29, CR 702.174a: pick one seat, or none. The sacrifice list's
       interaction, minus the pre-selection — see the module doc. -->
  {#if gift}
    <div class="cost-block">
      <div class="cost-label">
        <span class="optional">optional</span>
        <span class="cost-prompt">{gift.prompt}</span>
        {#if chosenGiftLabel !== null}
          <span class="picker-chosen">promising {gift.gift_label} to: {chosenGiftLabel}</span>
        {:else}
          <span class="squad-range">no gift promised · click a seat to promise one</span>
        {/if}
      </div>

      {#if giftCandidates.length === 0}
        <span class="no-candidates">no opponent is eligible to receive this gift</span>
      {:else}
        <div class="candidates">
          {#each giftCandidates as seat (seat.id)}
            <button
              class="candidate"
              class:selected={giftPicked === seat.id}
              disabled={disabled}
              onclick={() => selectGift(seat.id)}
            >
              {seat.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- PB-DX29, CR 702.47a: the multi-select one. Rendered as `SearchPicker`'s
       checkable list rather than as a third list idiom — same box glyphs, same row
       shape, so a player who has answered a search already knows this control. -->
  {#if splice}
    <div class="cost-block">
      <div class="cost-label">
        <span class="optional">optional</span>
        <span class="cost-prompt">{splice.prompt}</span>
        {#if splicePicked.length === 0}
          <span class="squad-range">nothing spliced · check a card to splice it</span>
        {:else}
          <span class="picker-chosen">
            splicing {splicePicked.length} card{splicePicked.length === 1 ? '' : 's'}
          </span>
        {/if}
      </div>

      {#if spliceCandidates.length === 0}
        <span class="no-candidates">no card in your hand can be spliced onto this spell</span>
      {:else}
        <div class="candidates rows">
          {#each spliceCandidates as card (card.id)}
            <button
              class="candidate"
              class:selected={splicePicked.includes(card.id)}
              disabled={disabled}
              onclick={() => toggleSplice(card.id)}
            >
              <span class="box">{splicePicked.includes(card.id) ? '☑' : '☐'}</span>
              <span class="row-label">{card.label}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <div class="picker-actions">
    <button class="confirm" disabled={disabled || !canConfirm} onclick={confirm}>Confirm</button>
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Back</button>
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

  /* PB-DX29: the splice list is the one vertical list here — `SearchPicker`'s row
     shape, so the two checkable lists in this client read as one control. */
  .candidates.rows {
    flex-direction: column;
    flex-wrap: nowrap;
    align-items: stretch;
  }

  .box {
    color: #8ab;
  }

  .row-label {
    flex: 1;
    text-align: left;
  }

  .candidates.rows .candidate {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
  }

  /* PB-DX29: the pay-or-not riders (Entwine / Fuse / Offspring). */
  .marker-row {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.72rem;
    color: #aab;
    cursor: pointer;
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
