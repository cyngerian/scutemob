<script>
  /**
   * ActionBar — the pending decision, rendered as buttons, plus the picker chain
   * that fills in `params` before submitting.
   *
   * M11-local Session 6 (`memory/m11-session-plan.md` §4, item 5); Session 7
   * (§4, item 6 wiring) adds the picker chain described below.
   *
   * Props:
   *   decision (DecisionView|null) — `{ seq, kind, player, actions }`
   *   loading (bool)              — a request is in flight
   *   error ({message,kind,status}|null)
   *   emptyReason (string)        — why there is no decision, supplied by the caller
   *   humanName (string|null)     — `summary.human_name`, threaded to `TargetPicker`
   *                                 so a candidate list can put your own segment
   *                                 first (UI-3, AC 6010)
   *   passUntil ({mode,passes,stopReason}|null) — `stores.js`' auto-pass status
   *   onPassUntil (fn(modeKind))  — start an auto-pass run (UI-3, AC 6009)
   *   onCancelPassUntil (fn)      — stop one mid-run
   *   onDismissPassUntil (fn)     — clear the finished-run status line
   *   onClientError (fn(message)) — a picker failed to build or emit its answer
   *                                 (UI-4, `scutemob-185`). Routed to the same
   *                                 error strip as a rejected request.
   *   onChainOpenChange (fn(bool)) — pushed whenever a picker chain opens or
   *                                 closes (UI-5, `scutemob-190`). The caller
   *                                 needs it because `beginExternal` refuses
   *                                 while a chain is open; see `chainOpen`.
   *   onAct (fn(index, params))   — submit an action
   *   onRefresh (fn)              — re-read the seat view
   *   onDismissError (fn)
   *   onCancel (fn)               — Escape: clear the caller's OWN pending picker
   *                                  (`PlayApp`'s inline chooser/click message) —
   *                                  this component's picker chain has its own
   *                                  cancel path (`cancelChain`), called alongside
   *
   * # `index`, and nothing else
   *
   * A submission is `{seq, action_index, params}`. The server maps the index back
   * through the `PendingDecision` it is still holding, so no engine type is a wire
   * type (`view.rs` module doc, "`LegalAction` is NEVER serialized"). `params` is
   * assembled by the picker chain below, or `{}` for an action that announces
   * nothing.
   *
   * # The picker chain (Session 7)
   *
   * Clicking an option does not submit immediately — it opens whichever pickers
   * that option's `ActionOptionView` fields call for, in the CR-mandated order,
   * accumulating one params object across the chain and submitting once at the
   * end:
   *
   *   0. blocking decision — iff `option.decision` is present (UI-1). Renders one
   *                          of five pickers chosen by `decision.answer.shape`:
   *                          `Subset` → `DiscardPicker`, `PickOne` →
   *                          `SearchPicker`, `Partition` → `PartitionPicker`,
   *                          `Slots` → the same `TargetPicker` stage 2 uses,
   *                          `PickN` (ENG-1, CR 701.9b) → `DiscardPicker` again,
   *                          templated this time.
   *   1. `ValuePrompt`     — iff `needs_x || modes.length > 0` (CR 601.2b
   *                          announces `{X}`/modes as part of casting, before
   *                          CR 601.2c's target announcement)
   *   2. `CostPicker`      — iff `option.costs` is present (UI-2, SIM-6). For a
   *                          CAST, CR 601.2b's additional costs: the required
   *                          sacrifice (CR 118.8) and the optional Squad count
   *                          (CR 702.157a). For an ACTIVATION, CR 602.2's
   *                          object-naming cost components: the sacrifice and the
   *                          discard. This stage's gate was always `option.costs`
   *                          and never the action kind, so SIM-6 needed no change
   *                          here beyond passing the two new blocks through — the
   *                          missing half was on the server, where
   *                          `additional_costs_view` early-returned for anything
   *                          that was not a `CastSpell`.
   *   3. `TargetPicker`    — iff the resolved slot list is non-empty. For a
   *                          per-mode-targeting card (`ModeOptionView.target_slots`
   *                          non-empty on at least one mode) the slots are the
   *                          CHOSEN modes' own `target_slots`, concatenated in
   *                          ascending mode-index order; otherwise the option's
   *                          own flat `target_slots`.
   *   4. `AttackerPicker`  — iff `option.attack` is present (CR 508.1)
   *   5. `BlockerPicker`   — iff `option.block` is present (CR 509.1)
   *
   * A stage that is not needed is skipped entirely (`pickerNeeded` below), and an
   * option needing none of the six submits `{}` immediately, exactly as before
   * Session 7. Escape (or a picker's own **Back** button) aborts the whole chain
   * and submits nothing — `cancelChain` clears every field the chain touched.
   *
   * That button said "Cancel" until UI-5 (`scutemob-190`, G8). The playtest note
   * is *"had to cancel and concede, which ended the game? — i thought the concede
   * button would concede the choice"*: with `Concede` sitting one button away in
   * the same row, "Cancel" read as its peer, as though the two were "abandon this
   * choice" and "abandon this game". It never was — it steps back out of the
   * picker and leaves the decision standing, which a blocking decision requires
   * (it is not cancellable; CR-mandated choices do not go away). "Back" says
   * that. `Concede` moved to the header in the same change, so the two are no
   * longer adjacent either.
   *
   * # Why the cost stage sits between `ValuePrompt` and `TargetPicker`
   *
   * The outer ordering is forced: CR 601.2b announces additional costs and CR
   * 601.2c announces targets, so costs must be collected before targets.
   *
   * The inner ordering is a deliberate simplification. Within CR 601.2b the
   * printed order is modes → splice → **additional costs** → `{X}`, so a strict
   * reading would split `ValuePrompt` — which bundles modes AND `{X}` in one
   * panel — around this stage, putting modes before it and `{X}` after it. That
   * is not done here, and the premise is checked rather than assumed:
   * `crates/engine/tests/core/ui2_additional_cost_roster.rs`'s **R5** pins that no
   * def in the corpus declares an additional cost (sacrifice or Squad) together
   * with an `{X}` or modes, so the sub-ordering inside 601.2b is not observable
   * in any game this engine can deal. If R5 ever fails, this is the paragraph to
   * re-read: the fix is to split `ValuePrompt`, not to reorder the chain.
   *
   * # Why the decision stage is numbered 0 and checked first
   *
   * Not because CR orders it before `{X}` announcement — the two never co-occur.
   * A blocking decision is its own `LegalAction` (`DiscardToHandSize`,
   * `AnswerEffectChoice`, `ChooseTriggerTargets`) and carries NONE of the other
   * stages' fields: `view.rs::action_target_requirements` returns an empty vector
   * for every variant that is not `CastSpell`/`ActivateAbility`, and
   * `combat_options` yields `None`/`None`, so `needs_x`, `modes`, `target_slots`,
   * `attack` and `block` are all empty or absent on such an option.
   *
   * So the ordering is not contentious — whichever position it took, exactly one
   * stage would ever fire for these actions. It is first because the chain reads
   * as "answer the question this action IS, then fill in the announcements a cast
   * needs", and because a reader looking for where a blocking decision is handled
   * finds it at the top of `pickerNeeded` rather than after four conditions that
   * can never be true at the same time.
   *
   * # `Slots` reuses `TargetPicker`, and takes its GROUPED output
   *
   * `AnswerShapeView::Slots` carries a `Vec<TargetSlotView>` — the very type the
   * CR 601.2c picker already renders (`view.rs` calls this the extension proof for
   * OOS-DP8-2). But the answer field differs: `targets` is flat, while
   * `trigger_targets` is one array per slot.
   *
   * Regrouping the flat list here was rejected. A slot contributes between its own
   * `min` and `max` entries, so per-slot counts are not recoverable from a
   * concatenation; any regrouping done here would be a guess that is right only
   * while every slot is exactly-one and silently wrong the first time an optional
   * (`UpToN`) trigger slot shows up — and `view.rs` emits `min: 0` for exactly
   * that case. Instead `TargetPicker.confirm` now builds the grouped form first
   * and passes it as a SECOND argument, deriving the flat one as `.flat()` of it.
   * The stage-2 caller ignores the second argument and is unchanged; this stage
   * ignores the first. The two shapes cannot disagree because one is computed from
   * the other.
   *
   * # Untested
   *
   * There is no frontend test harness in this repo (plan §8 R7), so none of the
   * decision-stage wiring has an automated test. Unexercised in particular: the
   * shape-dispatch fallback below (unreachable against the real server, which sends
   * one of exactly five shapes), and every interaction between the decision stage
   * and the four older stages — which, per the note above, cannot co-occur, but
   * that is an argument from the server's shape rather than an observation of a
   * running game.
   *
   * The same holds for the cost stage: `CostPicker` has no automated test, for the
   * same missing-harness reason. Its *channel* is covered end to end by the
   * play-server HTTP probes, which drive a real CR 118.8 sacrifice payment and a
   * real CR 702.157a Squad count through `POST /api/game/action`; what is untested
   * is this file's wiring of it and that component's rendering of it.
   *
   * The `Slots` *channel* is not in that list: `test_ui1_trigger_targets_are_
   * answered_over_http` drives a real CR 603.3d announcement end to end. What is
   * untested is this file's rendering of it, like everything else here.
   *
   * # Per-mode target ranges come from the server, not from a guess here
   *
   * For a card whose targets are genuinely per-mode, the **option-level**
   * `target_min`/`target_max` are `(0, 0)`: `spell_target_requirements` is
   * queried at render time with an empty `modes_chosen` (`view.rs`'s own
   * divergence-1 note), because the human has not chosen modes yet. So this
   * component sums each **chosen** mode's own `target_min`/`target_max` instead.
   *
   * That pair was not in the payload when this file was first written — the
   * first version had to approximate every per-mode slot as mandatory, which is
   * wrong for a mode carrying CR 601.2c's `UpToN`. It was reported rather than
   * shipped as a guess, and `ModeOptionView` gained the fields.
   *
   * Still unverified against a real per-mode-targeting card: no seeded game in
   * the fixture sweep dealt one, and there is no frontend test harness in this
   * repo (plan §8 R7). The arithmetic is right by construction; that it renders
   * correctly against a live modal spell is untested, and saying so is cheaper
   * than implying otherwise.
   */
  import TargetPicker from './TargetPicker.svelte';
  import AttackerPicker from './AttackerPicker.svelte';
  import BlockerPicker from './BlockerPicker.svelte';
  import ValuePrompt from './ValuePrompt.svelte';
  import DiscardPicker from './DiscardPicker.svelte';
  import SearchPicker from './SearchPicker.svelte';
  import PartitionPicker from './PartitionPicker.svelte';
  import CostPicker from './CostPicker.svelte';

  const {
    decision = null,
    loading = false,
    error = null,
    emptyReason = '',
    humanName = null,
    passUntil = null,
    onAct = null,
    onRefresh = null,
    onDismissError = null,
    onCancel = null,
    onPassUntil = null,
    onCancelPassUntil = null,
    onDismissPassUntil = null,
    onClientError = null,
    onChainOpenChange = null,
  } = $props();

  /**
   * A picker could not build or emit its answer (UI-4, `scutemob-185`).
   *
   * Routed to the same error strip a rejected request uses, and the chain is
   * closed afterwards: leaving the picker open under an error message invites a
   * second click on a button that has already proven it cannot work, which is
   * how G1 read to the player who hit it — the only live control left on screen
   * was Concede.
   */
  function onPickerError(message) {
    onClientError?.(message);
    resetChain();
  }

  /** True while `stores.js`' auto-pass loop is mid-run. */
  const autoPassing = $derived(!!passUntil && passUntil.stopReason === null);

  const actions = $derived(decision?.actions ?? []);

  /**
   * `PassPriority` is pulled into its own group on the right so "pass" is always
   * in the same place, however long the action list gets — plan item 5 asks for
   * it to be easy to find. The original `index` travels with each option, so the
   * split never affects what is submitted.
   *
   * # `Concede` is NOT here any more (G8, UI-5 `scutemob-190`)
   *
   * It used to share this group, one button away from Pass. The playtest note is
   * *"i thought the concede button would concede the choice — this option should
   * be next to new game, not in the priority changing area"*, and it was written
   * by someone who conceded a game by accident. `PlayApp` renders it in the
   * header beside "New game", behind a confirmation step, reading the very same
   * `Concede` option out of `decision.actions` and submitting the very same
   * `option.index` — nothing about the wire changed, only where the button is.
   *
   * It is filtered out of BOTH groups here, not merely out of `controls`: a kind
   * that is excluded from the control group and not from `plays` reappears in
   * the middle of the play list, which is worse than where it started.
   *
   * # `TapForMana` is not here either (G10)
   *
   * See `manaSources` below. Same principle: filtered out of `plays`, but into a
   * group of its own rather than off the surface — hiding it would remove
   * capabilities the client has no other way to reach.
   */
  const controlKinds = ['PassPriority'];
  /** Rendered somewhere other than the two groups below. Excluded from both. */
  const relocatedKinds = ['Concede', 'TapForMana'];
  const plays = $derived(
    actions.filter((a) => !controlKinds.includes(a.kind) && !relocatedKinds.includes(a.kind)),
  );
  const controls = $derived(actions.filter((a) => controlKinds.includes(a.kind)));

  /**
   * G10 — "tap land for mana" clutters the legal-action list.
   *
   * **The kind is grouped and collapsed, never hidden**, and the evidence for
   * that is not a preference:
   *
   *   - `local_game.rs::auto_tap_commands_for` opens
   *     `let Command::CastSpell(cast) = command else { return None; };` — auto-tap
   *     covers **casts and nothing else**.
   *   - An activated ability's mana cost is paid straight from the pool
   *     (`rules/abilities.rs`), so every Equip, every `{2}: …`, needs floating
   *     mana that only this action can produce from the human's side.
   *   - `PayEcho` / `PayCumulativeUpkeep` / `PayRecover` are offered **only when
   *     the existing pool already covers them** (`legal_actions.rs`), and that
   *     file's own comment says outright that *"CR 608.2g lets the player
   *     activate mana abilities first — so TapForMana must stay available
   *     alongside these"*.
   *
   * Hide it and a human can never pay an activation cost, never pay echo or
   * cumulative upkeep, and never float mana ahead of a cost-increase effect.
   * So it collapses instead: one disclosure row saying how many sources there
   * are, and — when opened — **one row per source card name with a count**,
   * because eight Forests are eight identical buttons and the playtest note is
   * about exactly that.
   *
   * The related gap where a human's *mana-cost* activation still 422s because
   * nothing auto-taps for it is `OOS-SIM6-3`, on the SIM track. Deliberately
   * untouched here.
   */
  const manaSources = $derived(actions.filter((a) => a.kind === 'TapForMana'));

  /**
   * Mana sources folded by label, insertion order preserved.
   *
   * The label is `view.rs`'s `format!("Tap {} for mana", card(source))`, so two
   * options share one only when they name the same card. Each row keeps the
   * FULL option list: clicking submits `options[0].index` — arbitrary and
   * immaterial, since the rows are only merged when the server printed the same
   * sentence for both — and the count is what the row is for.
   */
  const manaSourceRows = $derived.by(() => {
    const byLabel = new Map();
    for (const a of manaSources) {
      if (byLabel.has(a.label)) byLabel.get(a.label).push(a);
      else byLabel.set(a.label, [a]);
    }
    return [...byLabel.entries()].map(([label, options]) => ({ label, options }));
  });

  /** Collapsed by default — that is the whole request. */
  let manaOpen = $state(false);

  /** Found by `kind`, never by a hardcoded index — the list order is the server's. */
  const passAction = $derived(actions.find((a) => a.kind === 'PassPriority') ?? null);

  // ── Picker chain state ──────────────────────────────────────────────────────

  /** The `ActionOptionView` currently being answered, or `null` between chains. */
  let activeOption = $state(null);

  /**
   * Which picker is showing right now:
   * 'decision' | 'value' | 'costs' | 'targets' | 'attack' | 'block' | null.
   */
  let stage = $state(null);

  /** Stage names already answered in the current chain. */
  let doneStages = $state(new Set());

  /** Params accumulated across the chain so far; submitted whole at the end. */
  let paramsAcc = $state({});

  /** True while the human is mid-picker — dims/disables the action list. */
  const chainOpen = $derived(stage !== null);

  /**
   * Mirror `chainOpen` out to the caller (UI-5 `/review`, G8 issue 2).
   *
   * `beginChain` early-returns on `if (loading || chainOpen)`, so a caller that
   * cannot see this state can render a control that looks live, submits
   * nothing, and says nothing — the exact silent-dead-button shape UI-4 was
   * dispatched to fix, and the shape that made the playtester reach for
   * Concede. `PlayApp`'s header Concede is such a caller: it routes through
   * `beginExternal`, so it must be able to disable itself for the same reason
   * this component disables its own buttons.
   *
   * Pushed rather than exported as a getter because `PlayApp` needs it inside a
   * `$derived`, and a method call on a `bind:this` handle is not reactive.
   */
  $effect(() => {
    onChainOpenChange?.(chainOpen);
  });

  /** Does `option` declare per-mode target slots (as opposed to flat ones)? */
  function isPerModeTargeting(option) {
    return (option.modes ?? []).some((m) => (m.target_slots?.length ?? 0) > 0);
  }

  /**
   * The slot list the `TargetPicker` stage should render for `option`, given
   * whatever has been accumulated in `paramsSoFar` (specifically `modes_chosen`).
   * See the module doc for the per-mode-vs-flat distinction and its known gap.
   */
  function resolvedTargetSlots(option, paramsSoFar) {
    if (isPerModeTargeting(option)) {
      const chosen = [...(paramsSoFar.modes_chosen ?? [])].sort((a, b) => a - b);
      const slots = [];
      for (const idx of chosen) {
        const mode = option.modes.find((m) => m.index === idx);
        if (mode) slots.push(...mode.target_slots);
      }
      return slots;
    }
    return option.target_slots ?? [];
  }

  /**
   * `[min, max]` for the `TargetPicker` stage (CR 601.2c).
   *
   * For a per-mode-targeting card this is the **sum over the chosen modes** of
   * each mode's own server-computed range, not a slot count — see the module
   * doc. Summing is right because the announced `targets` array is the chosen
   * modes' slots concatenated, so the collective range is the sum of the parts.
   */
  function resolvedTargetRange(option, paramsSoFar) {
    if (isPerModeTargeting(option)) {
      const chosen = [...(paramsSoFar.modes_chosen ?? [])];
      let min = 0;
      let max = 0;
      for (const idx of chosen) {
        const mode = option.modes.find((m) => m.index === idx);
        if (!mode) continue;
        min += mode.target_min ?? 0;
        max += mode.target_max ?? 0;
      }
      return [min, max];
    }
    return [option.target_min, option.target_max];
  }

  const currentTargetSlots = $derived.by(() =>
    activeOption ? resolvedTargetSlots(activeOption, paramsAcc) : [],
  );
  const currentTargetRange = $derived.by(() =>
    activeOption ? resolvedTargetRange(activeOption, paramsAcc) : [0, 0],
  );

  // ── Blocking-decision stage (UI-1) ──────────────────────────────────────────

  /** The active option's `BlockingDecisionView`, or `null` outside that stage. */
  const currentDecision = $derived(activeOption?.decision ?? null);

  /** Its `AnswerShapeView` — the thing dispatched on, per `view.rs`'s own advice. */
  const currentShape = $derived(currentDecision?.answer ?? null);

  /** Slots for the `Slots` shape; `[]` for every other shape. */
  const decisionSlots = $derived(currentShape?.shape === 'Slots' ? currentShape.slots : []);

  /**
   * `[min, max]` for the `Slots` shape, summed from the slots' OWN `min`/`max`.
   *
   * `TargetPicker` wants a collective range as well as the per-slot ones, and for
   * a trigger there is no server-computed collective range to read — `view.rs`
   * emits `min: slot.optional ? 0 : 1` and `max: slot.max` per slot and nothing
   * else. Summing is the right collective bound because the announcement is the
   * slots' contributions concatenated, which is the same argument the per-mode
   * range above makes.
   */
  const decisionSlotRange = $derived(
    decisionSlots.reduce(
      ([mn, mx], slot) => [mn + (slot.min ?? 0), mx + (slot.max ?? 1)],
      [0, 0],
    ),
  );

  /**
   * Which stage (if any) is still needed for `option`, given what `doneSet`
   * already covers and what `paramsSoFar` holds. Returns `null` when nothing is
   * left — the caller submits.
   */
  function pickerNeeded(option, paramsSoFar, doneSet) {
    // Checked first — see the module doc's "Why the decision stage is numbered 0".
    if (!doneSet.has('decision') && option.decision) return 'decision';
    if (!doneSet.has('value')) {
      const needsValue = option.needs_x || (option.modes?.length ?? 0) > 0;
      if (needsValue) return 'value';
    }
    // CR 601.2b before CR 601.2c — see the module doc's "Why the cost stage sits
    // between `ValuePrompt` and `TargetPicker`".
    if (!doneSet.has('costs') && option.costs) return 'costs';
    if (!doneSet.has('targets')) {
      if (resolvedTargetSlots(option, paramsSoFar).length > 0) return 'targets';
    }
    if (!doneSet.has('attack') && option.attack) return 'attack';
    if (!doneSet.has('block') && option.block) return 'block';
    return null;
  }

  function advanceChain() {
    if (!activeOption) return;
    const needed = pickerNeeded(activeOption, paramsAcc, doneStages);
    if (needed === null) {
      const option = activeOption;
      const params = paramsAcc;
      resetChain();
      onAct?.(option.index, params);
      return;
    }
    stage = needed;
  }

  /**
   * Begin the picker chain for `option`, or submit immediately if it needs none.
   * This is the sole entry point for acting on an option — the plain button
   * click below and `PlayApp`'s click-through (via `beginExternal`) both call it,
   * so a targeted spell can never be submitted with an empty `targets` array from
   * either path.
   */
  function beginChain(option) {
    if (loading || chainOpen) return;
    activeOption = option;
    doneStages = new Set();
    paramsAcc = {};
    advanceChain();
  }

  /** Called by `PlayApp`'s click-through via `bind:this` — same entry point as a click. */
  export function beginExternal(option) {
    beginChain(option);
  }

  function cancelChain() {
    resetChain();
  }

  function resetChain() {
    activeOption = null;
    stage = null;
    doneStages = new Set();
    paramsAcc = {};
  }

  /**
   * The single exit from the decision stage. `params` is a whole params fragment
   * (`{discard_cards: [...]}` / `{effect_choice_answer: {...}}` /
   * `{trigger_targets: [[...]]}`) built by the picker from the server's own
   * `answer_field`, so this function never names a params key: the four pickers
   * that route through here (`Subset`, `PickOne`, `Partition`, and — ENG-1 —
   * `PickN`) key their own fragment off the payload, which keeps the
   * `ActionParamsDto` schema known in one place (`view.rs`'s stated reason for
   * sending the field).
   */
  function onDecisionConfirm(params) {
    paramsAcc = { ...paramsAcc, ...params };
    doneStages = new Set([...doneStages, 'decision']);
    advanceChain();
  }

  /**
   * `Slots` bridge. `TargetPicker` hands back `(flat, perSlot)`; a trigger's
   * answer is the GROUPED one — see the module doc. The flat first argument is
   * deliberately ignored here.
   */
  function onDecisionSlotsConfirm(_flat, perSlot) {
    const field = currentDecision?.answer_field;
    if (!field) return;
    onDecisionConfirm({ [field]: perSlot });
  }

  function onValueConfirm(result) {
    paramsAcc = { ...paramsAcc, ...result };
    doneStages = new Set([...doneStages, 'value']);
    advanceChain();
  }

  /**
   * The single exit from the cost stage (UI-2). Like `onDecisionConfirm`, `params`
   * is a whole fragment (`{additional_costs: [...]}`) built by the picker from the
   * server's own `answer_field` — so this function never names a params key
   * either, and `ActionParamsDto`'s schema stays known in one place.
   */
  function onCostsConfirm(params) {
    paramsAcc = { ...paramsAcc, ...params };
    doneStages = new Set([...doneStages, 'costs']);
    advanceChain();
  }

  function onTargetsConfirm(targets) {
    paramsAcc = { ...paramsAcc, targets };
    doneStages = new Set([...doneStages, 'targets']);
    advanceChain();
  }

  function onAttackConfirm(attackers) {
    paramsAcc = { ...paramsAcc, attackers };
    doneStages = new Set([...doneStages, 'attack']);
    advanceChain();
  }

  function onBlockConfirm(blockers) {
    paramsAcc = { ...paramsAcc, blockers };
    doneStages = new Set([...doneStages, 'block']);
    advanceChain();
  }

  /** A decision that disappears (game over, refresh) must not leave a dangling picker. */
  $effect(() => {
    if (!decision) resetChain();
  });

  /**
   * Keyboard shortcuts (plan item 5): space = pass priority, Escape = cancel the
   * pending picker and dismiss the error strip.
   *
   * Bound on `window` in an `$effect` with cleanup. The handler reads `decision`,
   * `loading` and `stage` when it fires rather than when the effect runs, so the
   * effect registers once instead of re-binding on every state change while still
   * seeing current values.
   */
  $effect(() => {
    function isTyping(target) {
      if (!target) return false;
      if (target.isContentEditable) return true;
      return ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
    }

    function onKeyDown(event) {
      // Typing a seed into a field must not pass priority.
      if (isTyping(event.target)) return;
      if (event.ctrlKey || event.metaKey || event.altKey) return;

      if (event.key === ' ' || event.code === 'Space') {
        // Always prevent the default: space scrolls the page, and a scroll jump
        // on a key that sometimes acts and sometimes does not is worse than
        // either behaviour consistently.
        event.preventDefault();
        if (loading) return;
        // A picker chain is open: space must not pass priority underneath it.
        if (stage !== null) return;
        const pass = decision?.actions?.find((a) => a.kind === 'PassPriority');
        if (pass) onAct?.(pass.index, {});
      } else if (event.key === 'Escape') {
        cancelChain();
        onCancel?.();
        onDismissError?.();
      }
    }

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  /**
   * A 409 `stale_decision` means the client answered a superseded action list —
   * `api.rs` says so verbatim, and the remedy it names is "re-read GET /api/game
   * and retry". Do it rather than making the user click.
   *
   * No loop risk: a successful refresh clears `error`, and a refresh that fails
   * differently replaces the kind. A refresh swallowed by the `loading` guard
   * leaves the strip up with its retry button, which is the honest outcome.
   */
  $effect(() => {
    if (error?.kind === 'stale_decision') {
      onRefresh?.();
    }
  });

  /** Prose for the error strip, by envelope `kind` (`ApiError.kind` in `api.rs`). */
  const errorHeadline = $derived.by(() => {
    if (!error) return '';
    switch (error.kind) {
      // 422: syntactically fine, addressed to a real action, and the *engine*
      // refused it — an illegal target, an unpayable cost. The message is the
      // `GameStateError` rendered as text.
      case 'rejected':
        return 'The engine refused this play';
      case 'stale_decision':
        return 'Your view was out of date — refreshing';
      case 'no_pending_decision':
        return 'There is nothing to answer right now';
      case 'not_pregame':
        return 'Too late to mulligan — the game has begun';
      case 'no_session':
        return 'No game is running';
      case 'unknown_action':
      case 'bad_params':
      case 'invalid_body':
      case 'malformed_json':
        return 'The client sent something the server could not use';
      // UI-4: not an `ApiError.kind` — `stores.js` sets this one, and only for a
      // failure that happened in the browser. Worded so a bug report says which
      // side broke, because those are two different bugs to file.
      case 'client_error':
        return 'Something went wrong in this browser — the game is unchanged';
      default:
        return 'Request failed';
    }
  });
</script>

<div class="action-bar">
  {#if error}
    <div class="error-strip" class:engine={error.kind === 'rejected'} role="alert">
      <div class="error-body">
        <span class="error-headline">{errorHeadline}</span>
        <span class="error-detail">{error.message}</span>
        {#if error.status}<span class="error-status">HTTP {error.status}</span>{/if}
      </div>
      <button class="error-dismiss" onclick={() => onDismissError?.()} title="Dismiss (Esc)">
        ✕
      </button>
    </div>
  {/if}

  <!--
    UI-3 (AC 6009): the auto-pass run always says why it stopped. A pass-until
    control that silently returns priority is indistinguishable from one that
    broke, and the reason ("something was put on the stack", "you have a
    CleanupDiscard decision to make") is the whole value of having pressed it.
  -->
  {#if passUntil}
    <div class="auto-pass-strip" class:running={autoPassing}>
      {#if autoPassing}
        <span class="ap-label">passing…</span>
        <span class="ap-detail">{passUntil.passes} pass{passUntil.passes === 1 ? '' : 'es'}</span>
        <button class="ap-btn" onclick={() => onCancelPassUntil?.()}>Cancel</button>
      {:else}
        <span class="ap-label">stopped</span>
        <span class="ap-detail">
          after {passUntil.passes} pass{passUntil.passes === 1 ? '' : 'es'} — {passUntil.stopReason}
        </span>
        <button class="ap-btn" onclick={() => onDismissPassUntil?.()} title="Dismiss">✕</button>
      {/if}
    </div>
  {/if}

  {#if decision}
    <div class="bar-row">
      <div class="decision-heading">
        <span class="decision-kind">{decision.kind}</span>
        <span class="decision-seq">seq {decision.seq}</span>
      </div>

      <div class="action-groups" class:dimmed={chainOpen}>
        <div class="action-group plays">
          {#if plays.length === 0}
            <!--
              G10 moved `TapForMana` out of this list, so "no plays" would be a
              lie on a turn whose only offered actions are lands to tap. Say
              which it is.
            -->
            <span class="no-plays">
              {manaSources.length > 0
                ? 'No plays available beyond tapping for mana.'
                : 'No plays available.'}
            </span>
          {:else}
            {#each plays as option (option.index)}
              <button
                class="action-btn kind-{option.kind}"
                disabled={loading || chainOpen}
                title={option.kind}
                onclick={() => beginChain(option)}
              >
                {option.label}
                {#if option.needs_x}<span class="needs-x">X</span>{/if}
              </button>
            {/each}
          {/if}
        </div>

        <!--
          G10: mana sources, collapsed. See `manaSources` for why this is a
          group and not a deletion. Rendered between the plays and the controls
          so "pass" keeps the rightmost position it has always had.
        -->
        {#if manaSources.length > 0}
          <div class="action-group mana-sources">
            <button
              class="action-btn mana-toggle"
              aria-expanded={manaOpen}
              disabled={loading || chainOpen}
              onclick={() => (manaOpen = !manaOpen)}
            >
              {manaOpen ? '▾' : '▸'} mana sources ({manaSources.length})
            </button>
            {#if manaOpen}
              <div class="mana-rows">
                {#each manaSourceRows as row (row.label)}
                  <button
                    class="action-btn kind-TapForMana"
                    disabled={loading || chainOpen}
                    onclick={() => beginChain(row.options[0])}
                  >
                    {row.label}
                    {#if row.options.length > 1}
                      <span class="mana-count">×{row.options.length}</span>
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/if}

        {#if controls.length > 0}
          <div class="action-group controls">
            {#each controls as option (option.index)}
              <button
                class="action-btn control kind-{option.kind}"
                disabled={loading || chainOpen}
                title={option.kind === 'PassPriority' ? 'Pass priority (space)' : option.kind}
                onclick={() => beginChain(option)}
              >
                {option.label}
                {#if option.kind === 'PassPriority'}<span class="key-hint">space</span>{/if}
              </button>
            {/each}

            <!--
              UI-3 (AC 6009). Shown only when passing is actually offered, so a
              button that cannot work is never on screen — the loop's first
              unconditional check is the same condition.

              Client-side: each is a run of ordinary `PassPriority` submissions
              over the existing route (`stores.js::startPassUntil`). No server
              change, and the loop hands control straight back the moment a
              non-`Priority` decision arrives, so it can never answer a real
              choice on the human's behalf.
            -->
            {#if passAction}
              {#if autoPassing}
                <button
                  class="action-btn auto-pass cancel-auto"
                  onclick={() => onCancelPassUntil?.()}
                  title="Stop passing now"
                >
                  ■ stop ({passUntil.passes})
                </button>
              {:else}
                <button
                  class="action-btn auto-pass"
                  disabled={loading || chainOpen}
                  onclick={() => onPassUntil?.('my-turn')}
                  title="Keep passing priority until your own turn begins"
                >
                  ⏭ until my turn
                </button>
                <button
                  class="action-btn auto-pass"
                  disabled={loading || chainOpen}
                  onclick={() => onPassUntil?.('phase-end')}
                  title="Keep passing until the step changes or something goes on the stack"
                >
                  ⏵ until something happens
                </button>
              {/if}
            {/if}
          </div>
        {/if}
      </div>
    </div>

    {#if stage === 'decision'}
      <!--
        Dispatch on the SHAPE, never on `decision.question` — `view.rs` says a
        client switching on the question tag "is doing more work than it needs
        to", and a fifth question reusing an existing shape must need no change
        here. The `{:else}` arm is unreachable against the real server and exists
        so an unknown shape degrades to a visible, cancellable message rather than
        to an empty bar with no way forward.

        The outer guard is `stage === 'decision'` ALONE, deliberately. It used to
        also require `currentShape`, which meant a missing or malformed `answer`
        rendered NOTHING — no picker, no fallback (the fallback is inside this
        block), and every action button disabled by `chainOpen`, leaving Escape as
        the only way out. That is the exact failure the `{:else}` arm exists to
        prevent, slipping past it. A falsy `currentShape` now falls through to that
        arm instead. (UI-1 review, LOW 5.)
      -->
      {#if currentShape?.shape === 'Subset'}
        <DiscardPicker
          prompt={currentDecision.prompt}
          candidates={currentShape.candidates}
          count={currentShape.count}
          defaults={currentShape.default}
          answerField={currentDecision.answer_field}
          disabled={loading}
          onConfirm={onDecisionConfirm}
          onCancel={cancelChain}
        />
      {:else if currentShape?.shape === 'PickN'}
        <DiscardPicker
          prompt={currentDecision.prompt}
          candidates={currentShape.candidates}
          count={currentShape.count}
          minCount={currentShape.min_count}
          defaults={currentShape.default}
          template={currentShape.template}
          chosenKey={currentShape.chosen_key}
          answerField={currentDecision.answer_field}
          disabled={loading}
          onConfirm={onDecisionConfirm}
          onCancel={cancelChain}
          onError={onPickerError}
        />
      {:else if currentShape?.shape === 'PickOne'}
        <SearchPicker
          prompt={currentDecision.prompt}
          candidates={currentShape.candidates}
          allCards={currentShape.all_cards ?? []}
          mayDecline={currentShape.may_decline}
          template={currentShape.template}
          foundKey={currentShape.found_key}
          answerField={currentDecision.answer_field}
          disabled={loading}
          onConfirm={onDecisionConfirm}
          onCancel={cancelChain}
          onError={onPickerError}
        />
      {:else if currentShape?.shape === 'Partition'}
        <PartitionPicker
          prompt={currentDecision.prompt}
          lookedAt={currentShape.looked_at}
          keptKey={currentShape.kept_key}
          movedKey={currentShape.moved_key}
          movedLabel={currentShape.moved_label}
          template={currentShape.template}
          answerField={currentDecision.answer_field}
          disabled={loading}
          onConfirm={onDecisionConfirm}
          onCancel={cancelChain}
          onError={onPickerError}
        />
      {:else if currentShape?.shape === 'Slots'}
        <TargetPicker
          slots={decisionSlots}
          min={decisionSlotRange[0]}
          max={decisionSlotRange[1]}
          {humanName}
          disabled={loading}
          onConfirm={onDecisionSlotsConfirm}
          onCancel={cancelChain}
        />
      {:else}
        <div class="unknown-shape">
          <span class="unknown-shape-text">
            This client does not know how to answer a "{currentShape?.shape ?? 'malformed'}" decision.
          </span>
          <button class="action-btn control" onclick={cancelChain}>Back</button>
        </div>
      {/if}
    {:else if stage === 'value'}
      <ValuePrompt
        needsX={activeOption.needs_x}
        modes={activeOption.modes}
        modeMin={activeOption.mode_min}
        modeMax={activeOption.mode_max}
        disabled={loading}
        onConfirm={onValueConfirm}
        onCancel={cancelChain}
      />
    {:else if stage === 'costs'}
      <CostPicker
        prompt={activeOption.costs.prompt}
        sacrifice={activeOption.costs.sacrifice}
        squad={activeOption.costs.squad}
        activationSacrifice={activeOption.costs.activation_sacrifice}
        activationDiscard={activeOption.costs.activation_discard}
        answerField={activeOption.costs.answer_field}
        disabled={loading}
        onConfirm={onCostsConfirm}
        onCancel={cancelChain}
        onError={onPickerError}
      />
    {:else if stage === 'targets'}
      <TargetPicker
        slots={currentTargetSlots}
        min={currentTargetRange[0]}
        max={currentTargetRange[1]}
        {humanName}
        disabled={loading}
        onConfirm={onTargetsConfirm}
        onCancel={cancelChain}
      />
    {:else if stage === 'attack'}
      <AttackerPicker
        eligible={activeOption.attack.eligible}
        targets={activeOption.attack.targets}
        disabled={loading}
        onConfirm={onAttackConfirm}
        onCancel={cancelChain}
      />
    {:else if stage === 'block'}
      <BlockerPicker
        eligible={activeOption.block.eligible}
        attackers={activeOption.block.attackers}
        disabled={loading}
        onConfirm={onBlockConfirm}
        onCancel={cancelChain}
      />
    {/if}

    {#if passAction}
      <div class="hint">space = pass priority · Esc = cancel selection</div>
    {/if}
  {:else}
    <div class="bar-row empty">
      <span class="empty-reason">{emptyReason}</span>
      <button class="action-btn control" disabled={loading} onclick={() => onRefresh?.()}>
        Refresh
      </button>
    </div>
  {/if}
</div>

<style>
  .action-bar {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    background: #111120;
    /*
      UI-3: the bar moved from the bottom of the page to the top dock, under the
      seat cards (playtest note: "action bar could actually be at the top, under
      player cards"), so the rule that separated it from the board is now below
      it rather than above it.
    */
    border-bottom: 1px solid #2a2a44;
    padding: 0.35rem 0.6rem;
    font-family: monospace;
  }

  /* UI-3 (AC 6009) — auto-pass controls and status. */
  .action-btn.auto-pass {
    background: #14242a;
    border-color: #256;
    color: #8cc;
  }

  .action-btn.auto-pass:hover:not(:disabled) {
    background: #1c3440;
  }

  .action-btn.cancel-auto {
    background: #2a1a10;
    border-color: #6a3a10;
    color: #fc8;
  }

  .auto-pass-strip {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
    padding: 0.15rem 0.4rem;
    background: #101c20;
    border: 1px solid #1e3a44;
    border-radius: 3px;
    font-size: 0.72rem;
    color: #9bb;
  }

  .auto-pass-strip.running {
    border-color: #2a5a6a;
    background: #10242c;
  }

  .ap-label {
    color: #7cc;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-size: 0.66rem;
  }

  .ap-detail {
    flex: 1;
    color: #89a;
  }

  .ap-btn {
    padding: 0.05rem 0.35rem;
    font-size: 0.68rem;
    background: #16262c;
    color: #9bb;
    border: 1px solid #2a4a54;
    border-radius: 3px;
    cursor: pointer;
  }

  .ap-btn:hover {
    background: #1e3640;
  }

  .bar-row {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
  }

  .bar-row.empty {
    align-items: center;
    justify-content: space-between;
  }

  .decision-heading {
    display: flex;
    flex-direction: column;
    min-width: 8rem;
  }

  .decision-kind {
    font-size: 0.8rem;
    color: #fa0;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .decision-seq {
    font-size: 0.62rem;
    color: #445;
  }

  .action-groups {
    display: flex;
    flex: 1;
    gap: 0.6rem;
    justify-content: space-between;
    flex-wrap: wrap;
  }

  .action-groups.dimmed {
    opacity: 0.45;
  }

  .action-group {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .action-group.controls {
    margin-left: auto;
  }

  /* G10 — the collapsed mana-source group. */
  .action-group.mana-sources {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
  }

  .mana-toggle {
    background: #14201c;
    border-color: #2a4838;
    color: #8b8;
  }

  .mana-rows {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    max-width: 34rem;
  }

  .mana-count {
    color: #8c8;
    font-weight: bold;
  }

  .no-plays {
    font-size: 0.75rem;
    color: #556;
    align-self: center;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0.5rem;
    font-size: 0.76rem;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
  }

  .action-btn:hover:not(:disabled) {
    background: #2a2a58;
    border-color: #4a4a90;
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .action-btn.control {
    background: #24243c;
    color: #aab;
  }

  .action-btn.kind-Concede {
    border-color: #613;
    color: #c88;
  }

  .key-hint,
  .needs-x {
    font-size: 0.6rem;
    color: #667;
    border: 1px solid #33335a;
    border-radius: 2px;
    padding: 0 0.2rem;
  }

  .unknown-shape {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.6rem;
    background: #151530;
    border-top: 1px solid #2a2a44;
  }

  .unknown-shape-text {
    font-size: 0.74rem;
    color: #f9a;
  }

  .empty-reason {
    font-size: 0.78rem;
    color: #778;
  }

  .hint {
    font-size: 0.62rem;
    color: #445;
  }

  .error-strip {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.3rem 0.45rem;
    background: #2a1420;
    border: 1px solid #6a2a40;
    border-radius: 3px;
  }

  .error-strip.engine {
    background: #2a2010;
    border-color: #7a5a10;
  }

  .error-body {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.4rem;
  }

  .error-headline {
    font-size: 0.76rem;
    font-weight: bold;
    color: #f9a;
  }

  .error-strip.engine .error-headline {
    color: #fc8;
  }

  .error-detail {
    font-size: 0.74rem;
    color: #ddd;
    word-break: break-word;
  }

  .error-status {
    font-size: 0.62rem;
    color: #776;
  }

  .error-dismiss {
    background: none;
    border: none;
    color: #a88;
    cursor: pointer;
    font-size: 0.8rem;
    line-height: 1;
  }

  .error-dismiss:hover {
    color: #fdd;
  }
</style>
