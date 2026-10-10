<script>
  /**
   * PromptPanel — the one place that says what the game is waiting for and
   * offers the answer (`docs/play-surface-v2-brief.md` §3, principle 2).
   *
   * Fixed height (set by the parent grid). Inside it, from the top:
   *
   *   1. the STATUS slot — one line, always present: an error, the auto-pass
   *      strip, the click-through chooser, or a click message, else blank;
   *   2. the SENTENCE and the BUTTON ROW — the primary action is always the
   *      leftmost button and is always the same size;
   *   or, while a picker chain is open, the PICKER in place of 2, scrolling
   *      inside the panel so nothing outside it moves.
   *
   * The picker chain is `ActionBar`'s state machine, carried over unchanged in
   * substance (`pickerNeeded` → `advanceChain`, stage handlers, `beginExternal`
   * for click-through). `ActionBar` itself is untouched and still serves the
   * old surface; the two will be reconciled when the old surface is deleted.
   *
   * Error routing follows UI-4: every template-copying picker gets
   * `onError={onPickerError}`, which forwards to `onClientError` and closes
   * the chain — the same shape the source gate pins on `ActionBar`.
   */
  import TargetPicker from '../TargetPicker.svelte';
  import AttackerPicker from '../AttackerPicker.svelte';
  import BlockerPicker from '../BlockerPicker.svelte';
  import ValuePrompt from '../ValuePrompt.svelte';
  import DiscardPicker from '../DiscardPicker.svelte';
  import SearchPicker from '../SearchPicker.svelte';
  import PartitionPicker from '../PartitionPicker.svelte';
  import ConfirmPicker from '../ConfirmPicker.svelte';
  import BinaryChoicePicker from '../BinaryChoicePicker.svelte';
  import CostPicker from '../CostPicker.svelte';
  import { composePrompt, looseActions } from './prompt.js';

  const {
    decision = null,
    loading = false,
    error = null,
    passUntil = null,
    humanName = null,
    turn = null,
    gameOver = null,
    pregame = false,
    chooser = null,
    clickMessage = null,
    hasGame = false,
    onAct = null,
    onRefresh = null,
    onDismissError = null,
    onPassUntil = null,
    onCancelPassUntil = null,
    onDismissPassUntil = null,
    onClientError = null,
    onKeep = null,
    onMulligan = null,
    onNewGame = null,
    onExport = null,
    onChooseOption = null,
    onClearSelection = null,
    onChainOpenChange = null,
  } = $props();

  const actions = $derived(decision?.actions ?? []);
  const passAction = $derived(actions.find((a) => a.kind === 'PassPriority') ?? null);
  const loose = $derived(looseActions(decision));
  const autoPassing = $derived(!!passUntil && passUntil.stopReason === null);

  const prompt = $derived(
    composePrompt({ decision, turn, humanName, gameOver, pregame, loading }),
  );

  // ── Picker chain (ported from ActionBar) ───────────────────────────────────

  let activeOption = $state(null);
  let stage = $state(null);
  let doneStages = $state(new Set());
  let paramsAcc = $state({});
  const chainOpen = $derived(stage !== null);

  $effect(() => {
    onChainOpenChange?.(chainOpen);
  });

  function isPerModeTargeting(option) {
    return (option.modes ?? []).some((m) => (m.target_slots?.length ?? 0) > 0);
  }

  function isFusedCast(paramsSoFar) {
    return (paramsSoFar.additional_costs ?? []).some((c) => c === 'Fuse');
  }

  function resolvedTargetSlots(option, paramsSoFar) {
    if (isFusedCast(paramsSoFar) && (option.fused_target_slots?.length ?? 0) > 0) {
      return option.fused_target_slots;
    }
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

  function resolvedTargetRange(option, paramsSoFar) {
    if (isFusedCast(paramsSoFar) && (option.fused_target_slots?.length ?? 0) > 0) {
      return [option.fused_target_min, option.fused_target_max];
    }
    if (isPerModeTargeting(option)) {
      let min = 0;
      let max = 0;
      for (const idx of paramsSoFar.modes_chosen ?? []) {
        const mode = option.modes.find((m) => m.index === idx);
        if (!mode) continue;
        min += mode.target_min ?? 0;
        max += mode.target_max ?? 0;
      }
      return [min, max];
    }
    return [option.target_min, option.target_max];
  }

  const currentTargetSlots = $derived(activeOption ? resolvedTargetSlots(activeOption, paramsAcc) : []);
  const currentTargetRange = $derived(activeOption ? resolvedTargetRange(activeOption, paramsAcc) : [0, 0]);
  const currentDecision = $derived(activeOption?.decision ?? null);
  const currentShape = $derived(currentDecision?.answer ?? null);
  const decisionSlots = $derived(currentShape?.shape === 'Slots' ? currentShape.slots : []);
  const decisionSlotRange = $derived(
    decisionSlots.reduce(([mn, mx], slot) => [mn + (slot.min ?? 0), mx + (slot.max ?? 1)], [0, 0]),
  );

  function pickerNeeded(option, paramsSoFar, doneSet) {
    if (!doneSet.has('decision') && option.decision) return 'decision';
    if (!doneSet.has('value') && (option.needs_x || (option.modes?.length ?? 0) > 0)) return 'value';
    if (!doneSet.has('costs') && option.costs) return 'costs';
    if (!doneSet.has('targets') && resolvedTargetSlots(option, paramsSoFar).length > 0) return 'targets';
    if (!doneSet.has('attack') && option.attack) return 'attack';
    if (!doneSet.has('block') && option.block) return 'block';
    return null;
  }

  const STAGE_LABELS = {
    decision: 'answer',
    value: 'announce X / modes',
    costs: 'additional costs',
    targets: 'targets',
    attack: 'attackers',
    block: 'blockers',
  };

  /** "step 2 of 3" for the stage line, from the stages this option will visit. */
  const stageProgress = $derived.by(() => {
    if (!activeOption || !stage) return '';
    const all = [];
    const done = new Set();
    for (let guard = 0; guard < 8; guard++) {
      const next = pickerNeeded(activeOption, paramsAcc, done);
      if (!next) break;
      all.push(next);
      done.add(next);
    }
    const i = all.indexOf(stage);
    return all.length > 1 && i >= 0 ? `step ${i + 1} of ${all.length}` : '';
  });

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

  function beginChain(option) {
    if (loading || chainOpen) return;
    activeOption = option;
    doneStages = new Set();
    paramsAcc = {};
    advanceChain();
  }

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

  function finishStage(name, params) {
    paramsAcc = { ...paramsAcc, ...params };
    doneStages = new Set([...doneStages, name]);
    advanceChain();
  }

  function onDecisionConfirm(params) {
    finishStage('decision', params);
  }

  function onDecisionSlotsConfirm(_flat, perSlot) {
    const field = currentDecision?.answer_field;
    if (!field) return;
    onDecisionConfirm({ [field]: perSlot });
  }

  function onPickerError(message) {
    onClientError?.(message);
    resetChain();
  }

  $effect(() => {
    if (!decision) resetChain();
  });

  $effect(() => {
    if (error?.kind === 'stale_decision') onRefresh?.();
  });

  /**
   * A declare decision opens its picker on arrival, once per `seq`, so Back
   * sticks (carried over from `scutemob-260`).
   */
  let autoOpenedSeq = null;
  $effect(() => {
    if (!decision || loading || stage !== null) return;
    if (decision.seq === autoOpenedSeq) return;
    if (decision.kind !== 'DeclareAttackers' && decision.kind !== 'DeclareBlockers') return;
    const declare = actions.filter((a) => a.attack || a.block);
    if (declare.length !== 1) return;
    autoOpenedSeq = decision.seq;
    beginChain(declare[0]);
  });

  // ── Keyboard ───────────────────────────────────────────────────────────────

  $effect(() => {
    function isTyping(target) {
      if (!target) return false;
      if (target.isContentEditable) return true;
      return ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
    }
    function onKeyDown(event) {
      if (isTyping(event.target)) return;
      if (event.ctrlKey || event.metaKey || event.altKey) return;
      if (event.key === ' ' || event.code === 'Space') {
        event.preventDefault();
        if (loading || stage !== null || chooser) return;
        if (passAction) onAct?.(passAction.index, {});
      } else if (event.key === 'Escape') {
        cancelChain();
        onClearSelection?.();
        onDismissError?.();
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  const errorHeadline = $derived.by(() => {
    if (!error) return '';
    switch (error.kind) {
      case 'rejected': return 'The engine refused this play';
      case 'stale_decision': return 'Your view was out of date — refreshing';
      case 'no_pending_decision': return 'Nothing to answer right now';
      case 'not_pregame': return 'Too late to mulligan';
      case 'no_session': return 'No game is running';
      case 'client': return 'The page hit a bug';
      default: return 'Request failed';
    }
  });
</script>

<div class="prompt tone-{prompt.tone}" class:chain={chainOpen}>
  <div class="status">
    {#if error}
      <span class="st error" role="alert">
        <b>{errorHeadline}</b> {error.message}
        {#if error.status}<span class="dim">HTTP {error.status}</span>{/if}
      </span>
      <button class="x" onclick={() => onDismissError?.()} title="Dismiss (Esc)">✕</button>
    {:else if chooser}
      <span class="st chooser">
        "{chooser.name}" —
        {#each chooser.options as option (option.index)}
          <button class="mini" disabled={loading} onclick={() => onChooseOption?.(option)}>{option.label}</button>
        {/each}
      </span>
      <button class="x" onclick={() => onClearSelection?.()} title="Cancel (Esc)">✕</button>
    {:else if clickMessage}
      <span class="st note">{clickMessage}</span>
      <button class="x" onclick={() => onClearSelection?.()} title="Dismiss (Esc)">✕</button>
    {:else if passUntil}
      <span class="st auto" class:running={autoPassing}>
        {#if autoPassing}
          passing… {passUntil.passes}
        {:else}
          stopped after {passUntil.passes} pass{passUntil.passes === 1 ? '' : 'es'} — {passUntil.stopReason}
        {/if}
      </span>
      {#if autoPassing}
        <button class="x" onclick={() => onCancelPassUntil?.()} title="Stop">■</button>
      {:else}
        <button class="x" onclick={() => onDismissPassUntil?.()} title="Dismiss">✕</button>
      {/if}
    {:else}
      <span class="st blank">&nbsp;</span>
    {/if}
  </div>

  {#if chainOpen}
    <div class="picker-area">
      <div class="stage-line">
        <span class="stage-name">{activeOption?.label ?? ''}</span>
        <span class="stage-step">{STAGE_LABELS[stage] ?? stage}{stageProgress ? ` · ${stageProgress}` : ''}</span>
      </div>
      {#if stage === 'decision'}
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
        {:else if currentShape?.shape === 'Confirm'}
          <ConfirmPicker
            prompt={currentDecision.prompt}
            costLabel={currentShape.cost_label}
            template={currentShape.template}
            payKey={currentShape.pay_key}
            defaultPay={currentShape.default}
            answerField={currentDecision.answer_field}
            disabled={loading}
            onConfirm={onDecisionConfirm}
            onCancel={cancelChain}
            onError={onPickerError}
          />
        {:else if currentShape?.shape === 'BinaryChoice'}
          <BinaryChoicePicker
            prompt={currentDecision.prompt}
            trueLabel={currentShape.true_label}
            falseLabel={currentShape.false_label}
            template={currentShape.template}
            choiceKey={currentShape.choice_key}
            defaultChoice={currentShape.default}
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
          <div class="unknown">
            This client does not know how to answer a "{currentShape?.shape ?? 'malformed'}" decision.
            <button class="btn" onclick={cancelChain}>Back</button>
          </div>
        {/if}
      {:else if stage === 'value'}
        <ValuePrompt
          needsX={activeOption.needs_x}
          modes={activeOption.modes}
          modeMin={activeOption.mode_min}
          modeMax={activeOption.mode_max}
          disabled={loading}
          onConfirm={(r) => finishStage('value', r)}
          onCancel={cancelChain}
        />
      {:else if stage === 'costs'}
        <CostPicker
          prompt={activeOption.costs.prompt}
          sacrifice={activeOption.costs.sacrifice}
          squad={activeOption.costs.squad}
          counts={activeOption.costs.counts}
          markers={activeOption.costs.markers}
          gift={activeOption.costs.gift}
          splice={activeOption.costs.splice}
          pitch={activeOption.costs.pitch}
          activationSacrifice={activeOption.costs.activation_sacrifice}
          activationDiscard={activeOption.costs.activation_discard}
          answerField={activeOption.costs.answer_field}
          disabled={loading}
          onConfirm={(p) => finishStage('costs', p)}
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
          onConfirm={(targets) => finishStage('targets', { targets })}
          onCancel={cancelChain}
        />
      {:else if stage === 'attack'}
        <AttackerPicker
          eligible={activeOption.attack.eligible}
          targets={activeOption.attack.targets}
          disabled={loading}
          onConfirm={(attackers) => finishStage('attack', { attackers })}
          onCancel={cancelChain}
        />
      {:else if stage === 'block'}
        <BlockerPicker
          eligible={activeOption.block.eligible}
          attackers={activeOption.block.attackers}
          legalBlocks={activeOption.block.legal_blocks}
          disabled={loading}
          onConfirm={(blockers) => finishStage('block', { blockers })}
          onCancel={cancelChain}
        />
      {/if}
    </div>
  {:else}
    <div class="sentence">{prompt.text}</div>
    <div class="buttons">
      {#if gameOver}
        <button class="btn primary" disabled={loading} onclick={() => onNewGame?.()}>New game</button>
        <button class="btn" onclick={() => onExport?.()}>Export report</button>
      {:else if pregame}
        <button class="btn primary" disabled={loading} onclick={() => onKeep?.()}>Keep this hand</button>
        <button class="btn" disabled={loading} onclick={() => onMulligan?.()}>Take a mulligan</button>
      {:else if decision}
        {#if passAction}
          {#if autoPassing}
            <button class="btn primary stop" onclick={() => onCancelPassUntil?.()}>■ Stop passing ({passUntil.passes})</button>
          {:else}
            <button class="btn primary" disabled={loading} onclick={() => onAct?.(passAction.index, {})}>
              Pass priority <kbd>space</kbd>
            </button>
            <button class="btn" disabled={loading} onclick={() => onPassUntil?.('my-turn')} title="Keep passing until your own turn begins">
              ⏭ to my next turn
            </button>
            <button class="btn" disabled={loading} onclick={() => onPassUntil?.('phase-end')} title="Keep passing until the step changes or something goes on the stack">
              ⏵ to next step
            </button>
          {/if}
        {/if}
        {#each loose as option (option.index)}
          <button class="btn act" class:primary={!passAction} disabled={loading} onclick={() => beginChain(option)}>
            {option.label}{#if option.needs_x}<kbd>X</kbd>{/if}
          </button>
        {/each}
        {#if !passAction && loose.length === 0}
          <span class="hint">Click a lit card.</span>
        {/if}
      {:else if hasGame}
        <button class="btn" disabled={loading} onclick={() => onRefresh?.()}>Refresh</button>
      {:else}
        <button class="btn primary" disabled={loading} onclick={() => onNewGame?.()}>New game</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .prompt {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding: 0.25rem 0.6rem;
    background: #121226;
    border: 1px solid #2a2a44;
    border-left: 4px solid #33335a;
    border-radius: 4px;
    font-family: monospace;
    overflow: hidden;
  }

  .tone-yours { border-left-color: #6af; background: #121a30; }
  .tone-theirs { border-left-color: #556; }
  .tone-choice { border-left-color: #fa0; background: #1a1810; }
  .tone-over { border-left-color: #e66; }
  .tone-idle { border-left-color: #334; }

  .status {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    height: 1.3rem;
    flex-shrink: 0;
    font-size: 0.66rem;
    overflow: hidden;
    white-space: nowrap;
  }

  .st {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .st.error { color: #f9a; }
  .st.error b { color: #fcc; }
  .st.note { color: #bbc; }
  .st.chooser { color: #88a; display: flex; gap: 0.3rem; align-items: center; }
  .st.auto { color: #8bb; }
  .st.auto.running { color: #7dd; }
  .dim { color: #667; }

  .x,
  .mini {
    background: #1c1c38;
    color: #aab;
    border: 1px solid #33335a;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.62rem;
    padding: 0 0.3rem;
    cursor: pointer;
    line-height: 1.2;
  }

  .sentence {
    font-size: 0.95rem;
    font-weight: bold;
    color: #eef;
    padding: 0.15rem 0 0.3rem;
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tone-theirs .sentence { color: #aab; font-weight: normal; }
  .tone-idle .sentence { color: #778; font-weight: normal; }

  .buttons {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    flex-wrap: wrap;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    align-content: flex-start;
  }

  .btn {
    padding: 0.3rem 0.7rem;
    font-size: 0.78rem;
    font-family: monospace;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 4px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .btn:hover:not(:disabled) { background: #2a2a58; border-color: #4a4a90; }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }

  .btn.primary {
    min-width: 11rem;
    justify-content: center;
    background: #23386a;
    border-color: #3a5aa0;
    color: #eef;
    font-weight: bold;
  }

  .btn.primary:hover:not(:disabled) { background: #2d4a8a; }
  .btn.primary.stop { background: #5a2a10; border-color: #8a4a10; color: #fc8; }
  .btn.act { border-color: #2a5a3a; background: #14241c; color: #bfb; }

  kbd {
    font-size: 0.58rem;
    color: #99b;
    border: 1px solid #44466a;
    border-radius: 2px;
    padding: 0 0.2rem;
    font-weight: normal;
  }

  .hint { color: #667; font-size: 0.7rem; }

  .picker-area {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .stage-line {
    display: flex;
    gap: 0.5rem;
    align-items: baseline;
    font-size: 0.66rem;
    color: #99a;
    padding-bottom: 0.15rem;
  }

  .stage-name { color: #fc8; font-weight: bold; }
  .stage-step { color: #667; }

  .unknown { color: #f9a; font-size: 0.74rem; display: flex; gap: 0.5rem; align-items: center; }
</style>
