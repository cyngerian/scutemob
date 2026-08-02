<script>
  /**
   * PlayApp — the play surface: header, phase bar, seat-scoped state, event feed,
   * action bar, and the hand/battlefield click-through.
   *
   * M11-local Session 6 (`memory/m11-session-plan.md` §4, items 4 and 6);
   * re-laid-out by UI-3 (`scutemob-180`, AC 6006 + 6008).
   *
   * # The layout, and why it is flex rather than `position: sticky`
   *
   * The first-human-playtest notes asked for four things at once: "player cards
   * should stay in place on scroll", "action bar could actually be at the top,
   * under player cards", "stack under the action bar", and "the players hand
   * should be a permanent bar on the bottom like the action bar".
   *
   * All four fall out of one arrangement. `.play-app` is a full-height flex
   * column; the seat row, the action bar and the stack/combat dock are flex
   * children ABOVE the scrolling region, and the hand bar is a flex child BELOW
   * it. Only `.body` scrolls. Nothing is `position: sticky`, which matters
   * because sticky elements still occupy the scroll container and still slide
   * under each other when several of them stack up — with four docked strips
   * that is exactly the mess it looks like.
   *
   * The board itself (2×2 battlefields, dead-player reflow) is `PlayBoard`, and
   * the per-seat card is `SeatCard`; both are play-local, and the reasons are in
   * their own module docs. The shared `$viewer/StateView.svelte` is no longer
   * used by this surface and is **unmodified**, so the replay viewer's own
   * composition of it renders exactly what it did.
   *
   * Stated narrowly on purpose: UI-3 *did* change one `$viewer` file —
   * `CombatView.svelte`, deliberately and in place, because the replay viewer
   * carried the identical planeswalker-label defect. "The replay viewer is
   * untouched" would be the convenient sentence and it would be false.
   */
  import { onMount } from 'svelte';

  // Imported from the replay viewer **in place** via the `$viewer` alias
  // (`vite.config.js`). These components are props-based precisely so a second
  // app can reuse them without a copy — see
  // `docs/mtg-engine-replay-viewer.md` §"Import Mechanism". A copy would fork on
  // the next `StackObjectKind` variant.
  import PhaseIndicator from '$viewer/PhaseIndicator.svelte';
  import ZoneStack from '$viewer/ZoneStack.svelte';
  import ZoneHand from '$viewer/ZoneHand.svelte';
  import CombatView from '$viewer/CombatView.svelte';

  import ActionBar from './ActionBar.svelte';
  import EventFeed from './EventFeed.svelte';
  import PlayBoard from './PlayBoard.svelte';
  import SeatCard from './SeatCard.svelte';
  import { getReport } from './api.js';
  import {
    act,
    cancelPassUntil,
    decision,
    dismissError,
    dismissPassUntil,
    error,
    events,
    keepHand,
    loading,
    passUntil,
    refresh,
    reportClientError,
    seatView,
    startGame,
    startPassUntil,
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
   * with the two limitations `PlaySession::mulligan` documents: it reshuffles and
   * redeals every seat rather than just this one, and CR 103.5's bottoming half is
   * not expressible (the server refuses a non-empty `cards_to_bottom` with 400
   * rather than discarding it silently).
   *
   * What it does **not** do — since `scutemob-187` — is change anyone's cards. It
   * did until then: the session held a seeded deck *recipe*, so every mulligan
   * re-rolled all four decklists and all four commanders, and CR 903.6 makes the
   * command zone public, so the other seats' commanders visibly changed. The
   * session now stores the decklists it was actually dealt (`setup::dealt_decks`),
   * and a redeal permutes that fixed multiset.
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
  /**
   * G13's click path, resolved here rather than in `ZoneBattlefield`.
   *
   * A stacked land chip stands for several permanents and nominates
   * `members[0]` — arbitrary and immaterial, since the fungibility key required
   * every member to be indistinguishable. But *this* component is the only one
   * that knows which actions the server offered, so it is the right place for
   * the one case the key cannot rule out: a representative that carries no
   * offered action while a sibling in the same stack does. Falling through to
   * the first member that has one turns "clicking a 5-Forest stack is
   * undefined" into a decided answer.
   *
   * Returns the clicked card unchanged when the group is a single permanent, or
   * when no member has an action — the second is the honest input to the
   * zero-match branch, which reports the name the player clicked.
   */
  function representativeFor(card, group) {
    if (!Array.isArray(group) || group.length <= 1) return card;
    if (actionsForCard(card).length > 0) return card;
    return group.find((p) => actionsForCard(p).length > 0) ?? card;
  }

  /**
   * `group` (G13) is the full stack a clicked chip stands for. Every unstacked
   * call site passes `[card]`, and the replay viewer's own `openCard(card)`
   * takes one parameter and never sees it.
   */
  function handleCardClick(card, group = null) {
    clearSelection();
    card = representativeFor(card, group);
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

  // ── Concede (G8, UI-5 `scutemob-190`) ─────────────────────────────────────

  /**
   * `LegalAction::Concede`, read out of the decision the server is holding.
   *
   * **Nothing about the submission changed** — it is still the option's own
   * `index` over `POST /api/game/act` with `{}` params, which the server maps
   * back through its `PendingDecision` (`params.rs`). Only the button moved,
   * out of the action row and into the header beside "New game", which is
   * verbatim what the playtest note asked for: *"this option should be next to
   * new game, not in the priority changing area"*.
   *
   * Why that note exists at all is worth carrying: the player did not want to
   * concede the game. They wanted to back out of a picker. G1 had made the
   * picker's own Confirm button dead (`structuredClone` on a Svelte 5 proxy),
   * and `legal_actions.rs` early-returns with **only** the answer action when a
   * blocking decision stands — no `PassPriority` — so the entire action row was
   * `[answer] [Concede]` with the answer inert. Concede was the only live
   * control on screen. G1 is fixed (UI-4), so this is now placement and
   * confirmation alone; but a control that ends the game should never have been
   * one slip away from a control that passes priority.
   */
  const concedeAction = $derived(
    ($decision?.actions ?? []).find((a) => a.kind === 'Concede') ?? null,
  );

  /**
   * Why the header button is disabled, or `null` when it is live.
   *
   * **Disabled with a reason, never absent.** `Concede` is only in the payload
   * while this seat holds a decision (`local_game.rs` appends it to every
   * decision it builds for the human, including a blocking one), so a button
   * that rendered only when offerable would blink in and out of the header on
   * every bot turn — and a control that vanishes reads as a bug, or worse, gets
   * hunted for and found in the one moment it is dangerous.
   */
  const concedeDisabledReason = $derived.by(() => {
    if (!$seatView) return 'no game is running';
    if ($seatView.game_over) return 'the game is already over';
    if ($loading) return 'a request is in flight';
    if (!concedeAction) return 'not your decision — the bots are acting';
    return null;
  });

  /** Two-step confirmation: the first click arms, the second submits. */
  let concedeArmed = $state(false);

  function armConcede() {
    concedeArmed = true;
  }

  function cancelConcede() {
    concedeArmed = false;
  }

  /**
   * Submit the concession, through `ActionBar`'s picker chain rather than
   * straight to `act`.
   *
   * `beginExternal` is the single entry point for acting on an option
   * (`ActionBar.beginChain`'s doc), and `Concede` needs none of its six stages,
   * so it submits `{}` immediately — the same request the old in-row button
   * made. Routing through it anyway costs nothing and means a `Concede` that
   * ever grew a stage would not silently bypass it. It also inherits the
   * chain's `if (loading || chainOpen) return` guard, so conceding cannot race
   * a half-answered picker.
   */
  function confirmConcede() {
    concedeArmed = false;
    if (!concedeAction) return;
    actionBar?.beginExternal(concedeAction);
  }

  /**
   * An armed confirmation must not survive the thing it was armed against. The
   * decision advancing, the game ending, or the seat losing priority all make
   * the pending "yes, concede" click refer to an option that is gone.
   */
  $effect(() => {
    if (!concedeAction) concedeArmed = false;
  });

  // ── Derived view bits ──────────────────────────────────────────────────────

  const summary = $derived($seatView?.summary ?? null);
  const state = $derived($seatView?.state ?? null);
  const gameOver = $derived($seatView?.game_over ?? null);

  /**
   * This seat's display name, straight off the payload — **not** rebuilt as
   * `Human-${summary.human}`.
   *
   * `GameSummary::human_name` exists precisely so the client does not keep a
   * second copy of `mtg_simulator::setup::seat_name`'s naming convention; see
   * that field's doc. Everything that needs to know "which of these seats is
   * mine" reads this one derived value.
   */
  const humanName = $derived(summary?.human_name ?? null);

  /** Sorted for a stable seat order across responses. */
  const playerNames = $derived(state?.players ? Object.keys(state.players).sort() : []);

  /**
   * CR 506.1: combat state, present only during the combat phase.
   *
   * Rendered here for the first time on this surface. It was in the view model
   * from M9.5 and the play client never showed it, because `StateView` — the
   * component this surface used to render — does not include `CombatView`; the
   * replay viewer wires the two together in its own `App.svelte` instead. That
   * is the whole of playtest finding "not clear which card are attacking which
   * player after attackers declared": the data shipped on every payload and no
   * component read it.
   */
  const combat = $derived(state?.combat ?? null);

  /** The human's own hand, for the permanent bottom bar. */
  const ownHand = $derived(
    humanName !== null ? (state?.zones?.hand?.[humanName] ?? []) : [],
  );

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
        <!--
          The seed is deliberately NOT shown, and not sent: review MR-M11-01 removed it
          from `GameSummary` because it reconstructs every other seat's opening hand and
          library order (Architecture Invariant 7). It is in the "Export report"
          download, which is the documented exception.
        -->
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

      <!--
        G8: Concede, beside "New game" and behind a confirmation step. See
        `concedeAction` / `concedeDisabledReason` for why it is disabled rather
        than hidden when this seat holds no decision.
      -->
      {#if concedeArmed}
        <span class="concede-confirm" role="alert">
          <span class="concede-question">Concede — end your game?</span>
          <button class="concede-yes" onclick={confirmConcede}>Yes, concede</button>
          <button onclick={cancelConcede}>Keep playing</button>
        </span>
      {:else}
        <button
          class="concede"
          disabled={concedeDisabledReason !== null}
          aria-describedby={concedeDisabledReason !== null ? 'concede-reason' : null}
          onclick={armConcede}
        >
          Concede
        </button>
        {#if concedeDisabledReason !== null}
          <!--
            A visible reason, not a `title=`. A native tooltip does not open on
            a disabled button — a disabled control fires no pointer events — so
            "disabled with a reason" written as a `title` is a reason nobody can
            read. Same lesson as G11, from the other direction.
          -->
          <span class="concede-reason" id="concede-reason">{concedeDisabledReason}</span>
        {/if}
      {/if}
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

  <!--
    The top dock: seat cards, then the action bar, then the stack and combat.
    Everything here is OUTSIDE `.body`, so it never scrolls away — see the module
    doc for why this is a flex arrangement rather than `position: sticky`.
  -->
  <div class="top-dock">
    {#if state}
      <section class="seat-row">
        {#each playerNames as pname (pname)}
          <SeatCard
            player={state.players[pname]}
            playerName={pname}
            isActive={state.turn.active_player === pname}
            hasPriority={state.turn.priority === pname}
            isHuman={pname === humanName}
            commanders={state.zones?.command_zone?.[pname] ?? []}
            hand={state.zones?.hand?.[pname] ?? []}
            graveyard={state.zones?.graveyard?.[pname] ?? []}
            onCardClick={handleCardClick}
          />
        {/each}
      </section>
    {/if}

    <!--
      Mounted ONCE, outside the `{#if state}` above, and that is deliberate:
      `bind:this={actionBar}` is `PlayApp`'s click-through handle into the picker
      chain (`beginExternal`), and a second copy of this element in an `{:else}`
      branch would null the binding out on every transition between the two.
      With no state there is simply no seat row above it.
    -->
    <ActionBar
      bind:this={actionBar}
      decision={$decision}
      loading={$loading}
      error={$error}
      {emptyReason}
      {humanName}
      passUntil={$passUntil}
      onAct={(index, params) => act(index, params)}
      onRefresh={refresh}
      onDismissError={dismissError}
      onCancel={clearSelection}
      onPassUntil={(mode) => startPassUntil(mode)}
      onCancelPassUntil={cancelPassUntil}
      onDismissPassUntil={dismissPassUntil}
      onClientError={reportClientError}
    />

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

    <!--
      Stack + combat, capped so a deep stack scrolls inside its own box rather
      than pushing the board off the screen. Combat sits directly under the
      stack, which is where the playtest note asked for it ("could be a section
      under the stack which shows attackers and subsequent blockers").
    -->
    {#if state && ((state.zones?.stack?.length ?? 0) > 0 || combat)}
      <section class="stack-dock">
        {#if (state.zones?.stack?.length ?? 0) > 0}
          <ZoneStack items={state.zones.stack} onCardClick={handleCardClick} />
        {/if}
        {#if combat}
          <CombatView {combat} />
        {/if}
      </section>
    {/if}
  </div>

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
        <PlayBoard {state} {humanName} onCardClick={handleCardClick} />
      {:else}
        <div class="empty-state"><div class="empty-title">Loading…</div></div>
      {/if}
    </section>

    <aside class="feed-pane">
      <EventFeed events={$events} />
    </aside>
  </main>

  <!--
    The human's own hand, permanently docked at the bottom (playtest note). Uses
    the shared `ZoneHand`, so click-through and the Scryfall hover preview are
    the same code path every other zone uses.
  -->
  {#if state && humanName !== null}
    <footer class="hand-bar">
      <ZoneHand cards={ownHand} playerName={humanName} onCardClick={handleCardClick} />
    </footer>
  {/if}
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

  /* G8 — the header concede control and its confirmation step. */
  button.concede {
    background: #2a1418;
    border-color: #5a2a30;
    color: #d99;
  }

  button.concede:hover:not(:disabled) {
    background: #4a1c22;
  }

  .concede-reason {
    font-size: 0.66rem;
    color: #866;
    max-width: 12rem;
  }

  .concede-confirm {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.15rem 0.35rem;
    border: 1px solid #7a2a34;
    border-radius: 3px;
    background: #2a1418;
  }

  .concede-question {
    font-size: 0.7rem;
    color: #eaa;
  }

  button.concede-yes {
    background: #7a1c26;
    border-color: #a83a46;
    color: #fdd;
    font-weight: bold;
  }

  button.concede-yes:hover:not(:disabled) {
    background: #9a242f;
  }

  button.cancel {
    background: #241824;
    border-color: #4a2a4a;
    color: #caa;
  }

  /*
    UI-3 layout. `.top-dock` and `.hand-bar` are `flex-shrink: 0` siblings of the
    scrolling `.body`, which is what keeps them in place — see the module doc for
    why this is not `position: sticky`.
  */
  .top-dock {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    flex-shrink: 0;
    /*
      Capped for the same reason `.stack-dock` and `.hand-bar` are, and it was
      missed on the first pass while both of its neighbours got one. The dock is
      `flex-shrink: 0` and now hosts the ActionBar, which hosts every picker — so
      an expanded seat drawer plus a four-seat segmented `TargetPicker` can grow
      it without limit, squeeze `.body` toward zero and push the page into a
      document-level scrollbar. That does not merely look wrong: it destroys the
      "player cards stay in place on scroll" property this whole arrangement
      exists to provide, because once the *document* scrolls there is no fixed
      region left. `.body` is therefore guaranteed at least 38vh.

      Scrolling inside the dock is the lesser evil, not a happy outcome: the seat
      cards can leave view when it overflows. It only engages in the case where
      the alternative is the board vanishing entirely.
    */
    max-height: 62vh;
    overflow-y: auto;
    background: #0d0d1a;
    border-bottom: 1px solid #22223a;
  }

  .seat-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: flex-start;
    padding: 0.35rem 0.6rem 0;
  }

  /*
    A deep stack scrolls inside its own box. Without the cap, a stack ten deep
    would push the board out of the viewport entirely — and the board is what the
    stack is about.
  */
  .stack-dock {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    max-height: 30vh;
    overflow-y: auto;
    padding: 0 0.6rem 0.35rem;
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

  /*
    The human's own hand, always visible. Capped and scrollable for the same
    reason the stack is: a 15-card hand after a Windfall must not eat the board.
  */
  .hand-bar {
    flex-shrink: 0;
    max-height: 22vh;
    overflow-y: auto;
    padding: 0.3rem 0.6rem;
    background: #0d0d1a;
    border-top: 1px solid #22223a;
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
    /* Sits inside the top dock now, under the action bar. */
    border-bottom: 1px solid #2a2a44;
    font-size: 0.76rem;
    color: #bbc;
  }

  .chooser-label {
    color: #88a;
  }
</style>
