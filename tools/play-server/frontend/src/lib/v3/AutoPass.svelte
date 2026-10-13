<script>
  /**
   * AutoPass — the auto-pass panel under the right column's cells
   * (`docs/play-surface-v3-brief.md` §1 rule 12; the mockup's `autoPassPanel`).
   *
   * Off / until end of turn / until my next turn / until a chosen step, three
   * stop-conditions, and a sentence stating the rule. No props: it reads
   * `seatView` / `decision` / `passUntil` / `loading` from `stores.js` and starts
   * runs through `startPassUntilOnServer({ kind, rule, predicate })` there. The
   * rule itself — the wire object, the predicate and the sentence — is
   * `autopass.js`, which is pure.
   *
   * # One request per run (C3, `scutemob-268`)
   *
   * A run is ONE `POST /api/game/action`: the pass, plus the rule as the
   * server's `pass_until` parameter. The server keeps passing inside that
   * request and answers with where it stopped and why; the panel turns that
   * into the same three endings as before (below). The store finds the
   * `PassPriority` action and sends empty params itself — this panel hands it
   * a rule about *when to stop*, never an action (the surface's "one
   * submission path" rule, and the gate that pins it, are untouched).
   *
   * What that costs, stated on the panel while it applies: a server run cannot
   * be cancelled part-way, so **Off is disabled until the answer arrives**, and
   * the boxes and the step are locked for that moment (the request carries a
   * snapshot of them). Measured on a debug server a whole round of the table
   * answers in well under a second; the brakes that remain are the "unless"
   * stops and the server's 400-pass cap.
   *
   * The client loop (`startPassUntil`, one request per pass, cancellable, boxes
   * read live) is still there and is what runs when the server does not know
   * the parameter (400 `invalid_body`) — the store falls back by itself.
   *
   * # What choosing a mode does
   *
   * Choosing a mode other than Off **starts a run now**. It ends one of three ways:
   *
   *   - the "until" point was reached → the mode returns to Off and the status
   *     says where it stopped;
   *   - a stop-condition fired, or the loop met a decision that is not a
   *     `Priority` one (blockers, a discard, targets…) → the mode stays selected
   *     and the panel is **paused**: the status says why and shows Resume.
   *     Nothing is passed again until the human presses it. The client never
   *     passes on its own after the human has been handed a real choice — they
   *     may want to respond twice;
   *   - Off, or the decision panel's Stop button → cancelled, back to Off.
   *
   * Resume continues toward the **same** point: the turn and step the mode was
   * chosen in stay the anchor, so "until end of turn" paused by a cleanup discard
   * does not become "until the end of the next turn". If that point comes up
   * while the panel is paused (the human passed there by hand, or the decision
   * that paused it was the last thing in the turn), the run is over and the mode
   * returns to Off without another pass.
   *
   * # What is remembered
   *
   * The three checkboxes and the chosen step go to `localStorage`. The armed
   * mode never does: a page reload must not start passing on its own.
   *
   * # The select is always shown
   *
   * The mockup shows "stop at [step]" only once "Until step…" is selected. Here
   * selecting the mode starts the run, so the step has to be choosable *before*
   * the button is pressed; the row is always there, dimmed while another mode
   * is on.
   *
   * # The other pass-until buttons
   *
   * `PromptPanel` still offers "until my turn" / "until the phase ends" through
   * `TableV3`. A run this panel did not start is shown as running, Off cancels
   * it, and no second run is started (the store refuses one anyway).
   */
  import { get } from 'svelte/store';
  import { untrack } from 'svelte';
  import { seatView, decision, loading, passUntil, startPassUntilOnServer, cancelPassUntil } from '../stores.js';
  import { targeting } from './targeting.svelte.js';
  import { decisionKindName } from '../names.js';
  import { SHORT_STEP } from './layout.js';
  import { MODES, STOPS, STOPPABLE_STEPS, normaliseSettings, ruleSentence, buildPredicate, buildServerRule, turnFacts, untilReached } from './autopass.js';

  const STORE_KEY = 'scutemob-v3-autopass';

  function loadSettings() {
    try {
      return normaliseSettings(JSON.parse(localStorage.getItem(STORE_KEY) ?? 'null'));
    } catch {
      // No storage, or something else's junk under the key: the defaults.
      return normaliseSettings(null);
    }
  }

  // Mode-independent, remembered. Handed to the predicate as this live object,
  // so a box ticked mid-run takes effect on the next pass.
  const settings = $state(loadSettings());
  $effect(() => {
    const snapshot = JSON.stringify({ step: settings.step, spell: settings.spell, ability: settings.ability, combat: settings.combat });
    try {
      localStorage.setItem(STORE_KEY, snapshot);
    } catch {
      // Private mode / quota: the settings simply last for this page.
    }
  });

  // Never persisted (see the header).
  let mode = $state('off');
  // 'idle' | 'running' (a run this panel started) | 'paused' (waiting for Resume)
  let phase = $state('idle');
  // Why the last run of this panel stopped: `{ passes, reason, seq }` (`seq` =
  // the decision it stopped on), or null.
  let stopped = $state(null);
  // Where the mode was chosen (`turnFacts`): what "this turn" / "this step"
  // mean for the run and for every Resume of it.
  let anchor = null;
  // Bumped whenever the panel abandons a run, so a run that finishes after the
  // human pressed Off does not write its ending over the Off.
  let runToken = 0;

  const view = $derived($seatView);
  const pending = $derived($decision);
  const running = $derived(!!$passUntil && $passUntil.stopReason === null);
  // The run in flight is one server request: nothing can interrupt it.
  const serverRun = $derived(running && $passUntil.server === true);
  // A run in flight that is not ours: `PromptPanel`'s two buttons.
  const foreign = $derived(running && phase !== 'running');

  /** Why a run cannot start right now, or null when it can. */
  const inert = $derived.by(() => {
    if (!view) return 'No game is running.';
    if (view.game_over) return 'The game is over.';
    if (view.summary?.pregame) return 'Keep or mulligan first.';
    if (!pending) return 'Nothing to answer right now.';
    if (pending.kind !== 'Priority') return `Answer the ${decisionKindName(pending.kind)} decision first.`;
    // A picker chain keeps the option it was opened on until it is confirmed or
    // backed out of; passing under it would leave it pointing at a decision
    // that no longer exists (review finding 1: its Confirm then posted a stale
    // action index). `PromptPanel` hides its own pass buttons for the same reason.
    if (targeting.chainOpen) return 'Finish or back out of the open picker first.';
    return null;
  });
  const canStart = $derived(inert === null && !running && !$loading);

  function reset() {
    runToken += 1;
    mode = 'off';
    phase = 'idle';
    stopped = null;
    anchor = null;
  }

  async function begin(kind, { resume = false } = {}) {
    if (!canStart) return;
    const start = get(seatView);
    if (!resume || !anchor) anchor = turnFacts(start);
    let outcome = null;
    const predicate = buildPredicate(kind, settings, start, (o) => (outcome = o), anchor);
    // The same rule in the server's words. The anchor rides only on a Resume.
    const rule = buildServerRule(kind, settings, resume ? anchor : null);
    const token = ++runToken;
    mode = kind;
    phase = 'running';
    stopped = null;

    const started = await startPassUntilOnServer({ kind, rule, predicate });
    if (token !== runToken) return; // Off, or a redeal, got there first.

    const done = get(passUntil);
    const passes = done?.passes ?? 0;
    const reason = done?.stopReason ?? null;
    // A server run reports its ending as a tag; the fallback loop reported it
    // through the predicate's `onStop` above. Either way `outcome.stop` is
    // 'until' (finished) or 'unless' (interrupted), and anything else — a real
    // decision, nothing to answer, the pass cap, a refusal — hands the table back.
    if (done?.server && (done.stop === 'until' || done.stop === 'unless')) outcome = { stop: done.stop, reason };
    const now = get(seatView);
    if (!started || !reason) {
      // The store refused (a rival run, a request in flight): nothing ran.
      reset();
      return;
    }
    stopped = { passes, reason, seq: get(decision)?.seq ?? null };
    if (outcome?.stop === 'until' || !now || now.game_over) {
      // Arrived, or there is no game left to pass in.
      mode = 'off';
      phase = 'idle';
    } else if (!outcome && reason === 'cancelled') {
      // The decision panel's Stop button (`cancelPassUntil`): the human said stop.
      reset();
    } else {
      // A stop-condition, a real decision, or the loop's own stall / failure
      // stops: all of them hand the table back with the mode still chosen.
      phase = 'paused';
    }
  }

  function choose(kind) {
    if (kind === 'off') {
      // Cancels whatever is passing, ours or not.
      if (running) cancelPassUntil();
      reset();
      return;
    }
    begin(kind);
  }

  // A redeal clears everything: the mode, the pause and the reason all describe
  // a table that no longer exists. `passUntil` going null cannot be the signal
  // on its own — `act()` also clears it when the human answers after a run, and
  // a pause must outlive that — so the redeal is read off the view: no game,
  // back in the mulligan, or the turn counter running backwards.
  let lastTurn = null;
  $effect(() => {
    const turnNo = view?.state?.turn?.number ?? null;
    const redealt = !view || !!view.summary?.pregame || (lastTurn !== null && turnNo !== null && turnNo < lastTurn);
    lastTurn = turnNo;
    if (redealt) untrack(() => { if (mode !== 'off' || stopped) reset(); });
  });

  // The "until" point can come up while the panel is paused: the decision that
  // paused it was the last of the turn, or the human passed there by hand. The
  // run is then over — there is nothing left for Resume to do.
  $effect(() => {
    if (phase !== 'paused' || !view || !anchor) return;
    const reason = untilReached(mode, settings, anchor, view, view.summary?.human_name ?? null);
    if (!reason) return;
    untrack(() => {
      stopped = { passes: stopped?.passes ?? 0, reason, seq: pending?.seq ?? null };
      mode = 'off';
      phase = 'idle';
    });
  });

  // A finished run's "stopped — your turn began" has done its job once the
  // human dismisses the decision panel's strip (`passUntil` → null) or answers
  // the decision it stopped on, as that strip's own rule has it (`scutemob-260`).
  // A pause keeps its reason: it is what Resume is about.
  let hadStatus = false;
  $effect(() => {
    const has = $passUntil !== null;
    const moved = pending?.seq !== undefined && stopped !== null && stopped.seq !== null && pending.seq !== stopped.seq;
    if ((hadStatus && !has) || moved) untrack(() => { if (phase === 'idle') stopped = null; });
    hadStatus = has;
  });

  const plural = (n) => `${n} pass${n === 1 ? '' : 'es'}`;
  // The rule this panel is applying — or, while somebody else's run is going,
  // that fact instead of an "Off" that would not be true.
  const rule = $derived(
    foreign ? `Passing… ${$passUntil.passes} — started from the decision panel. Off cancels it.` : ruleSentence(mode, settings),
  );
  /** The second sentence: what is happening now, or why the last run stopped. */
  const why = $derived.by(() => {
    if (foreign) return null;
    if (serverRun) return 'Passing on the server — one request for the whole run; it cannot be stopped until it answers.';
    if (phase === 'running') return `Passing… ${$passUntil?.passes ?? 0}`;
    if (stopped) return `Stopped after ${plural(stopped.passes)} — ${stopped.reason}.`;
    return inert;
  });
</script>

<div class="autopass" class:running={phase === 'running' || foreign} class:paused={phase === 'paused'} class:server-run={serverRun}>
  <div class="panel-title">Auto-pass priority</div>
  <div class="modes">
    {#each MODES as [kind, label] (kind)}
      <button
        class="mode"
        class:on={mode === kind}
        data-mode={kind}
        disabled={kind === 'off' ? serverRun : !canStart}
        onclick={() => choose(kind)}
      >{label}</button>
    {/each}
  </div>
  <label class="stop-at" class:idle={mode !== 'step'}>
    stop at
    <select class="step" disabled={serverRun} bind:value={settings.step}>
      {#each STOPPABLE_STEPS as step (step)}
        <option value={step}>{SHORT_STEP[step] ?? step}</option>
      {/each}
    </select>
  </label>
  {#each STOPS as [key, text] (key)}
    <label><input type="checkbox" data-stop={key} disabled={serverRun} bind:checked={settings[key]} />{text}</label>
  {/each}
  <div class="status" class:off={mode === 'off' && !foreign}>
    <span class="rule">{rule}</span>
    {#if why}<span class="why">{why}</span>{/if}
    {#if phase === 'paused'}
      <button class="resume" disabled={!canStart} onclick={() => begin(mode, { resume: true })}>Resume</button>
    {/if}
  </div>
</div>
