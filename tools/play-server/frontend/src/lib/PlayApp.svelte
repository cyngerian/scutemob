<script>
  /**
   * PlayApp — the play surface: header, phase bar, seat-scoped state, event feed,
   * action bar, and the hand/battlefield click-through.
   *
   * M11-local Session 6 (`memory/m11-session-plan.md` §4, items 4 and 6).
   */
  import { onMount } from 'svelte';

  // Imported from the replay viewer **in place** via the `$viewer` alias
  // (`vite.config.js`). These components are props-based precisely so a second
  // app can reuse them without a copy — see
  // `docs/mtg-engine-replay-viewer.md` §"Import Mechanism". A copy would fork on
  // the next `StackObjectKind` variant.
  import PhaseIndicator from '$viewer/PhaseIndicator.svelte';
  import StateView from '$viewer/StateView.svelte';

  import ActionBar from './ActionBar.svelte';
  import EventFeed from './EventFeed.svelte';
  import { getReport } from './api.js';
  import {
    act,
    decision,
    dismissError,
    error,
    events,
    keepHand,
    loading,
    refresh,
    seatView,
    startGame,
    takeMulligan,
  } from './stores.js';

  // ── New-game form ───────────────────────────────────────────────────────────

  // Both boxes start EMPTY, and that is the point rather than an oversight: an
  // omitted field takes the server's own CLI default via `merge_defaults`, so a
  // blank box means "whatever this server was started with". Pre-seeding the
  // player count to '4' would have sent `players: 4` on every New game and
  // silently overridden a server run with `--players 6`.
  let seedInput = $state('');
  let playersInput = $state('');

  async function handleNewGame() {
    // Omit rather than send a blank — see the note on the inputs above.
    const opts = {};
    const seed = Number.parseInt(seedInput, 10);
    if (Number.isFinite(seed)) opts.seed = seed;
    const players = Number.parseInt(playersInput, 10);
    if (Number.isFinite(players)) opts.players = players;
    keptHand = false;
    clearSelection();
    await startGame(opts);
  }

  // ── Pregame ────────────────────────────────────────────────────────────────

  /**
   * The human has kept this hand. Purely client-side.
   *
   * `summary.pregame` is `command_count == 0` and stays true until the first
   * command is applied, so "keep" cannot be read off the payload — nothing on the
   * server records it (`post_mulligan` with `take: false` only re-renders).
   */
  let keptHand = $state(false);

  /**
   * # Which mulligan path this uses, and why
   *
   * The dedicated `POST /api/game/mulligan` route — **not** a `TakeMulligan` /
   * `KeepHand` entry in `decision.actions`, which the brief allowed for.
   *
   * Those actions are unreachable on this surface. `legal_actions.rs` gates them
   * on `state.turn().is_first_turn_of_game && turn_number == 0`, and
   * `setup::build_initial_state` + `GameStateBuilder` leave a freshly built game
   * already *in* turn 1 — `session.rs::is_pregame` says so in as many words, and
   * `local_game.rs::decision_kind_for` carries the same gate, which is why
   * `DecisionKind::Mulligan` never appears either. So the pregame decision's kind
   * is `Priority`, and this block is gated on `summary.pregame` alone rather than
   * on `decision.kind === 'Mulligan'`.
   *
   * The route itself is a whole-table redeal through `setup::redeal` (CR 103.5),
   * with the two limitations `PlaySession::mulligan` documents: it re-rolls every
   * seat, and CR 103.5's bottoming half is not expressible (the server refuses a
   * non-empty `cards_to_bottom` with 400 rather than discarding it silently).
   */
  const showPregame = $derived(!!$seatView?.summary?.pregame && !keptHand);

  async function handleMulligan() {
    clearSelection();
    await takeMulligan();
  }

  async function handleKeep() {
    clearSelection();
    keptHand = true;
    await keepHand();
  }

  // ── Bug-report export (Session 8, item 5) ──────────────────────────────────

  /**
   * Status line for the export button. `null` when idle.
   *
   * Deliberately NOT routed through the shared `error` store: that store drives
   * the dismissible banner every *game action* uses, and a failed download is not
   * a game event — surfacing it there would suggest the game state is in doubt
   * when nothing was submitted.
   */
  let reportStatus = $state(null);

  /**
   * Fetch `GET /api/game/report` and hand it to the browser as a download.
   *
   * `URL.createObjectURL` + a synthetic `<a download>` rather than opening the
   * endpoint in a tab: a tab would render 20,000 lines of JSON, and the filename
   * carries the reproduction key (seed and mulligan count) so a saved file is
   * self-identifying without being opened.
   *
   * The object URL is revoked in a `finally`, so a failed click leaks nothing.
   */
  async function handleExportReport() {
    reportStatus = 'Building report…';
    let url = null;
    try {
      const report = await getReport();
      const json = JSON.stringify(report, null, 2);
      const blob = new Blob([json], { type: 'application/json' });
      url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `scutemob-report-seed${report.seed}-mull${report.config.mulligan_count}.json`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      reportStatus = `Saved ${a.download} (${report.journal.length} commands).`;
    } catch (e) {
      // `.kind` is the play server's stable machine tag; `no_session` is the
      // ordinary "you have not started a game" case and deserves plain words.
      reportStatus =
        e?.kind === 'no_session'
          ? 'No game is running — start one first.'
          : `Export failed: ${e?.message ?? e}`;
    } finally {
      if (url) URL.revokeObjectURL(url);
    }
  }

  // ── Click-through (plan item 6) ────────────────────────────────────────────

  /**
   * When one card carries several actions (cast vs. play, or a permanent with
   * more than one ability), the choice is offered inline rather than guessed at.
   * `{ name, options }`; null when nothing is pending.
   */
  let chooser = $state(null);

  /** Why the last click produced nothing. Cleared by Escape or the next click. */
  let clickMessage = $state(null);

  /**
   * The mounted `ActionBar` instance, so click-through can hand an option to its
   * picker chain instead of submitting `{}` directly (Session 7).
   *
   * `ActionBar` exports `beginExternal(option)` precisely so a second entry point
   * exists that is not a plain button click — see its module doc. Lifting the
   * whole picker-chain state up into `PlayApp` instead was the other option
   * considered; this is simpler because `ActionBar` already owns every picker
   * component and the params-accumulation logic, and `PlayApp` has no other
   * reason to know about `target_slots` / `modes` / etc.
   */
  let actionBar = $state(null);

  function clearSelection() {
    chooser = null;
    clickMessage = null;
  }

  /**
   * Match `decision.actions` to a clicked card by `object_id`.
   *
   * `ActionOptionView.object_id` is `action_object(action)` — the card being cast,
   * the permanent being tapped — and is null for actions that are about no single
   * object (pass, declare attackers). Both sides are JSON numbers; ids are small
   * counters, well inside the exactly-representable range.
   */
  function actionsForCard(card) {
    const id = card?.object_id;
    if (id === undefined || id === null) return [];
    return ($decision?.actions ?? []).filter(
      (a) => a.object_id !== null && a.object_id !== undefined && Number(a.object_id) === Number(id),
    );
  }

  /**
   * A card carrying the redactor's `hidden` flag is refused *before*
   * `actionsForCard`, not allowed to fall through it.
   *
   * The reason is specific rather than defensive: `redact::hidden_placeholder`
   * does not omit a hidden card's id, it **rewrites it to 0** — every entry in
   * another seat's hand arrives as
   * `{hidden: true, name: "Hidden card", object_id: 0}`, and `redact_hands`'
   * doc says why ("the id itself is a handle onto a hidden object"). Read off a
   * real payload: a 4-player seed-0 playthrough carried 569 such entries, all
   * with id 0, while the lowest id any `ActionOptionView` ever carried was 2. So
   * there is no collision today — but every one of a bot's seven hand cards is
   * the *same* id, and one action about object 0 would make all seven submit it.
   * Matching on a sentinel is the wrong shape whether or not it collides yet.
   *
   * Scope, stated because it is narrower than "hidden things": `hidden` is a
   * field of `CardInZoneView` (hand, graveyard, exile, command zone), not of
   * `PermanentView`. An opponent's **face-down permanent** keeps its real
   * `object_id` and is redacted only by name (`redact_face_down_permanents`), so
   * it is matched normally — which is right, because an action that legitimately
   * names it is about an object the seat can point at even without knowing what
   * it is.
   */
  function isUnidentifiable(card) {
    return card?.hidden === true;
  }

  /**
   * `StateView` threads `onCardClick` into `ZoneHand`, `ZoneBattlefield`,
   * `ZoneGraveyard` and `ZoneExile`, each of which calls it with the card /
   * permanent object itself, carrying `object_id`.
   *
   * **`ZoneStack` is threaded the prop and never invokes it** — it destructures
   * `onCardClick` and there is no `onclick` anywhere in the file. So a stack item
   * is inert. Nothing here needs it (every `LegalAction` carrying an `object_id`
   * names a card or a permanent, never a stack object — `view.rs::action_object`),
   * but Session 7 renders targets on stack items and should know two things
   * before it relies on this path: the prop is dead, **and** a stack entry's id
   * field is `id`, not `object_id`, so `actionsForCard` would read `undefined` and
   * return `[]` in silence rather than failing loudly.
   */
  function handleCardClick(card) {
    clearSelection();
    const name = card?.name ?? 'this card';

    if (isUnidentifiable(card)) {
      clickMessage =
        'That card is hidden from this seat — another player’s hand, or a card ' +
        'exiled face down. There is nothing to play, and the server does not send ' +
        'its identity at all (Architecture Invariant 7).';
      return;
    }

    if (!$decision) {
      // Distinguish "no decision at all" from "no action for this card": the
      // first is a timing fact we know for certain, the second is not.
      clickMessage = $seatView?.game_over
        ? `The game is over — "${name}" can no longer be used.`
        : `No decision is outstanding right now, so "${name}" cannot be used — ` +
          'it is not your priority (the bots are acting, or the last request is still in flight).';
      return;
    }

    const matches = actionsForCard(card);
    if (matches.length === 1) {
      // Route through the picker chain rather than submitting `{}` directly —
      // a click-through cast of a targeted spell must open `TargetPicker`, not
      // submit a targetless cast the engine 422s under CR 601.2c.
      actionBar?.beginExternal(matches[0]);
      return;
    }
    if (matches.length > 1) {
      chooser = { name, options: matches };
      return;
    }

    // Zero matches. **The server does not tell us why**, so nothing here invents a
    // rules reason (a land drop already used, an unpayable cost, a sorcery-speed
    // restriction all look identical from this side). Report exactly what is
    // known: the action list this decision offered did not include this card.
    const offered = ($decision.actions ?? []).map((a) => a.label);
    const offeredText = offered.length > 0 ? offered.join(' · ') : '(nothing)';
    clickMessage =
      `No legal action for "${name}" right now — the server offered none for it in ` +
      `the current decision (${$decision.kind}). What is offered: ${offeredText}`;
  }

  function chooseOption(option) {
    clearSelection();
    // Same reasoning as `handleCardClick`'s single-match branch: a chosen option
    // may itself need targets/X/modes, so it goes through the picker chain too.
    actionBar?.beginExternal(option);
  }

  // ── Derived view bits ──────────────────────────────────────────────────────

  const summary = $derived($seatView?.summary ?? null);
  const state = $derived($seatView?.state ?? null);
  const gameOver = $derived($seatView?.game_over ?? null);

  /** True only when we know there is no game — not merely that none has loaded yet. */
  const noGame = $derived(!$seatView && !$error && !$loading);

  /**
   * Why the action bar is empty. Passed in rather than guessed at inside
   * `ActionBar`, which cannot see `game_over`.
   */
  const emptyReason = $derived.by(() => {
    if (!$seatView) return 'No game loaded.';
    if (gameOver) return 'The game is over.';
    if ($loading) return 'Working…';
    return 'Waiting for the bots to act.';
  });

  onMount(() => {
    // A 404 `no_session` here is the ordinary cold-start case and `refresh`
    // handles it silently — see its doc comment.
    refresh();
  });
</script>

<div class="play-app">
  <header class="app-header">
    <div class="brand">
      <span class="title">scutemob</span>
      <span class="subtitle">play</span>
    </div>

    {#if summary}
      <div class="summary">
        <span class="stat"><b>turn</b> {summary.turn}</span>
        <span class="stat"><b>seat</b> Human-{summary.human}</span>
        <span class="stat"><b>players</b> {summary.players}</span>
        <span class="stat"><b>bot</b> {summary.bot}</span>
        <span class="stat"><b>seed</b> {summary.seed}</span>
        <span class="stat"><b>mulligans</b> {summary.mulligan_count}</span>
        <span class="stat"><b>commands</b> {summary.command_count}</span>
        {#if summary.pregame}<span class="badge">pregame</span>{/if}
      </div>
    {/if}

    <div class="new-game">
      <label>
        seed
        <input type="text" inputmode="numeric" placeholder="default" bind:value={seedInput} />
      </label>
      <label>
        players
        <input type="text" inputmode="numeric" placeholder="default" bind:value={playersInput} />
      </label>
      <button class="primary" disabled={$loading} onclick={handleNewGame}>New game</button>
      <!--
        Not disabled on `$loading`: the export is a pure read that cannot disturb a
        request in flight (`api.rs::get_report` neither advances the game nor moves
        `journal_cursor`), and the moment you most want a repro file is while
        something is stuck.
      -->
      <!--
        No braces in this title: Svelte reads `{...}` in an attribute as an
        expression, so the obvious "a {seed, config, journal, state hash} artefact"
        would be compiled as code rather than shown as text.
      -->
      <button
        onclick={handleExportReport}
        title="Download a repro artefact: seed, config, journal and final state hash"
      >
        Export report
      </button>
    </div>
  </header>

  {#if reportStatus}
    <div class="report-status">{reportStatus}</div>
  {/if}

  {#if state}
    <PhaseIndicator turn={state.turn} />
  {/if}

  {#if gameOver}
    <div class="game-over" class:halted={gameOver.halted}>
      <span class="go-title">{gameOver.halted ? 'Game halted' : 'Game over'}</span>
      {#if gameOver.winner}<span class="go-item">winner: <b>{gameOver.winner}</b></span>{/if}
      <span class="go-item">turns: {gameOver.turn_count}</span>
      <span class="go-item">commands: {gameOver.total_commands}</span>
      {#if gameOver.reason}<span class="go-item">reason: {gameOver.reason}</span>{/if}
      {#if gameOver.violations?.length > 0}
        <span class="go-violations">
          invariant violations ({gameOver.violations.length}): {gameOver.violations.join(' | ')}
        </span>
      {/if}
    </div>
  {/if}

  {#if showPregame}
    <div class="pregame">
      <span class="pregame-title">CR 103.5 — pregame</span>
      <span class="pregame-text">
        Mulliganing redeals the whole table from a perturbed seed
        (mulligans taken: {summary?.mulligan_count ?? 0}).
      </span>
      <button disabled={$loading} onclick={handleMulligan}>Take a mulligan</button>
      <button class="primary" disabled={$loading} onclick={handleKeep}>Keep this hand</button>
    </div>
  {/if}

  <main class="body">
    <section class="state-pane">
      {#if noGame}
        <div class="empty-state">
          <div class="empty-title">No game is running.</div>
          <div class="empty-text">
            Set a seed and a player count above, then press <b>New game</b> to deal a
            table. You sit in seat 1; the other seats are bots.
          </div>
        </div>
      {:else if state}
        <!--
          `SeatView.state` is ALREADY seat-redacted: `api.rs::seat_view` builds it
          with `StateViewModel::from_game_state_for(.., Viewer::Seat(human))`,
          Architecture Invariant 7's chokepoint. This component must not attempt
          any redaction of its own — a hidden card arrives with `hidden: true` and
          no name, and re-filtering here would only be able to remove information
          the server already removed, while creating a second place for the two
          policies to disagree.
        -->
        <StateView {state} onCardClick={handleCardClick} />
      {:else}
        <div class="empty-state"><div class="empty-title">Loading…</div></div>
      {/if}
    </section>

    <aside class="feed-pane">
      <EventFeed events={$events} />
    </aside>
  </main>

  {#if chooser}
    <div class="chooser">
      <span class="chooser-label">"{chooser.name}" —</span>
      {#each chooser.options as option (option.index)}
        <button disabled={$loading} onclick={() => chooseOption(option)}>{option.label}</button>
      {/each}
      <button class="cancel" onclick={clearSelection} title="Cancel (Esc)">Cancel</button>
    </div>
  {:else if clickMessage}
    <div class="click-message">
      <span>{clickMessage}</span>
      <button class="cancel" onclick={clearSelection} title="Dismiss (Esc)">✕</button>
    </div>
  {/if}

  <ActionBar
    bind:this={actionBar}
    decision={$decision}
    loading={$loading}
    error={$error}
    {emptyReason}
    onAct={(index, params) => act(index, params)}
    onRefresh={refresh}
    onDismissError={dismissError}
    onCancel={clearSelection}
  />
</div>

<style>
  .play-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    min-height: 0;
    font-family: monospace;
  }

  .app-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
    padding: 0.35rem 0.8rem;
    background: #0b0b18;
    border-bottom: 1px solid #2a2a44;
  }

  .brand {
    display: flex;
    align-items: baseline;
    gap: 0.3rem;
  }

  .title {
    color: #adf;
    font-weight: bold;
  }

  .subtitle {
    color: #556;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .summary {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
    font-size: 0.72rem;
    color: #99a;
  }

  .stat b {
    color: #667;
    font-weight: normal;
  }

  .badge {
    background: #2a2010;
    border: 1px solid #7a5a10;
    color: #fc8;
    border-radius: 3px;
    padding: 0 0.3rem;
    font-size: 0.68rem;
  }

  .new-game {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-left: auto;
    font-size: 0.7rem;
    color: #778;
  }

  .new-game label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .new-game input {
    width: 5.5rem;
    background: #141428;
    border: 1px solid #33335a;
    border-radius: 3px;
    color: #ccd;
    font-family: monospace;
    font-size: 0.72rem;
    padding: 0.15rem 0.3rem;
  }

  button {
    padding: 0.2rem 0.45rem;
    font-size: 0.74rem;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: #2a2a58;
  }

  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  button.primary {
    background: #23386a;
    border-color: #3a5aa0;
    color: #dde;
  }

  button.cancel {
    background: #241824;
    border-color: #4a2a4a;
    color: #caa;
  }

  .body {
    display: flex;
    flex: 1;
    min-height: 0;
    gap: 0.5rem;
    padding: 0.5rem;
  }

  .state-pane {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }

  .feed-pane {
    width: 22rem;
    min-width: 14rem;
    display: flex;
    min-height: 0;
  }

  .empty-state {
    padding: 2rem 1rem;
    text-align: center;
  }

  .empty-title {
    color: #aab;
    font-size: 0.95rem;
    margin-bottom: 0.4rem;
  }

  .empty-text {
    color: #667;
    font-size: 0.78rem;
    max-width: 34rem;
    margin: 0 auto;
    line-height: 1.5;
  }

  .game-over {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.6rem;
    padding: 0.3rem 0.8rem;
    background: #101c28;
    border-bottom: 1px solid #2a4a60;
    font-size: 0.76rem;
    color: #9bd;
  }

  .game-over.halted {
    background: #2a1420;
    border-bottom-color: #6a2a40;
    color: #f9a;
  }

  .go-title {
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .go-item {
    color: #89a;
  }

  .go-violations {
    color: #f99;
    word-break: break-word;
  }

  .pregame {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    padding: 0.3rem 0.8rem;
    background: #1a140a;
    border-bottom: 1px solid #4a3010;
    font-size: 0.75rem;
  }

  .pregame-title {
    color: #a80;
    font-weight: bold;
  }

  .pregame-text {
    color: #997;
  }

  /* Session 8, item 5 — the export button's own status line. */
  .report-status {
    padding: 0.3rem 0.8rem;
    background: #0f1a14;
    border-bottom: 1px solid #1e4030;
    color: #8ba;
    font-size: 0.75rem;
  }

  .chooser,
  .click-message {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.6rem;
    background: #151530;
    border-top: 1px solid #2a2a44;
    font-size: 0.76rem;
    color: #bbc;
  }

  .chooser-label {
    color: #88a;
  }
</style>
