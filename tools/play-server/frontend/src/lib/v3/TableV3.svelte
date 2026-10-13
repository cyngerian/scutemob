<script>
  /**
   * TableV3 — the alpha play surface (`docs/play-surface-v3-brief.md`, mockup 7).
   *
   * You in the middle, one opponent across the table, every opponent in a
   * fixed seat down the sides, the dock (stack, combat, decision) on the
   * right. This component is the grid and the wiring; every pixel rule lives
   * in `v3.css`, which is the mockup's stylesheet scoped under `.v3`.
   *
   * Data flow is the earlier surfaces' unchanged: `stores.js` owns every fetch
   * and the seat-redacted `SeatView` (Architecture Invariant 7 is enforced
   * server-side; nothing here filters), `PromptPanel` — hosted inside the
   * dock's decision panel — owns the picker chain, and this component owns the
   * click-through, matching a clicked card to the decision's actions by
   * `object_id`. The click-through, offered ids and concede are ported from
   * `PlayAppV2`, which stays behind `?v2` untouched.
   *
   * Mounted by `main.js` when the URL carries `?v3`.
   */
  import { onMount, tick, untrack } from 'svelte';
  import { get } from 'svelte/store';
  import './v3.css';
  import './v3-dock.css';
  import './v3-c2.css';
  import './v3-arrows.css';
  import './v3-order.css';
  import './v3-block.css';
  import './v3-autopass.css';
  import './v3-c3.css';
  import TopBar from './TopBar.svelte';
  import PhaseTrack from './PhaseTrack.svelte';
  import SideCell from './SideCell.svelte';
  import FocusedBoard from './FocusedBoard.svelte';
  import YourBoard from './YourBoard.svelte';
  import HandRow from './HandRow.svelte';
  import Dock from './Dock.svelte';
  import PilePopover from './PilePopover.svelte';
  import ContextMenu from './ContextMenu.svelte';
  import AutoPass from './AutoPass.svelte';
  import Arrows from './Arrows.svelte';
  import { beginArrowFromCard, bindTable, sync as syncTargeting, targeting } from './targeting.svelte.js';
  import { newGameKey, trackGame } from './order.js';
  import { syncStackTargets } from './stackTargets.svelte.js';
  import { fitAll } from './fit.js';
  import { peekStyle } from './zones.js';
  import { landStacks, pickFocus, seatOrder, sideColumns, splitBattlefield } from './layout.js';
  import { closePeek, openCardCtx, openCtx, settings, ui } from './ui.svelte.js';
  import { getReport } from '../api.js';
  import { answerable } from '../v2/prompt.js';
  import {
    seatView,
    decision,
    loading,
    error,
    passUntil,
    startGame,
    refresh,
    act,
    takeMulligan,
    keepHand,
    dismissError,
    reportClientError,
    startPassUntil,
    cancelPassUntil,
    dismissPassUntil,
  } from '../stores.js';

  onMount(() => {
    // The page chrome (13px root like the mockup, no document scroll) is keyed
    // on a class so the other two surfaces in this bundle never see it.
    document.documentElement.classList.add('v3-page');
    refresh();
    return () => document.documentElement.classList.remove('v3-page');
  });

  // ── Derived view bits ──────────────────────────────────────────────────────

  const summary = $derived($seatView?.summary ?? null);
  const view = $derived($seatView?.state ?? null);
  const gameOver = $derived($seatView?.game_over ?? null);
  const humanName = $derived(summary?.human_name ?? null);
  const seats = $derived(seatOrder(view?.players ? Object.keys(view.players) : [], humanName));
  const opponents = $derived(seats.filter((p) => p !== humanName));
  const columns = $derived(sideColumns(opponents));
  const zones = $derived(view?.zones ?? null);
  const ownHand = $derived(humanName !== null ? (zones?.hand?.[humanName] ?? []) : []);
  const activePlayer = $derived(view?.turn?.active_player ?? null);
  const priority = $derived(view?.turn?.priority ?? null);

  let keptHand = $state(false);

  /**
   * The key your own card order is remembered under (`order.js`): the
   * server's `summary.game_id` (C3), which changes on every new deal whoever
   * dealt it. `trackGame` falls back to the C2 counters heuristic only for a
   * summary that carries none.
   */
  let gameKey = $state(null);
  $effect(() => {
    gameKey = summary ? trackGame(summary) : null;
  });
  const pregame = $derived(!!summary?.pregame && !keptHand);

  // What the stack is already aimed at (C3): the red `targeted` outline on
  // cards, the ring on a targeted seat, a stack item's hover arrows. All of it
  // is `stack[i].target_refs`, looked up — see `stackTargets.svelte.js`.
  $effect(() => {
    const stack = zones?.stack ?? [];
    untrack(() => syncStackTargets(stack));
  });
  // A server-side auto-pass run is one request: nothing can stop it (C3).
  const serverPassing = $derived(!!$passUntil && $passUntil.stopReason === null && $passUntil.server === true);

  // ── Who sits across the table (mockup rule 2) ──────────────────────────────

  let chosenFocus = $state(null);
  let lastActive = null;
  $effect(() => {
    // A turn change brings the new active player across; a click overrides it
    // until the next turn change.
    if (activePlayer !== lastActive) {
      lastActive = activePlayer;
      if (activePlayer && opponents.includes(activePlayer)) chosenFocus = activePlayer;
    }
  });
  const focus = $derived(pickFocus(chosenFocus, opponents, activePlayer));

  function bringAcross(name) {
    if (!opponents.includes(name)) return;
    chosenFocus = name;
    closePeek();
  }

  /** The focused board is sized for the largest opponent, so a focus change does not resize it. */
  const maxRows = $derived.by(() => {
    const m = { creatures: 0, others: 0, lands: 0 };
    for (const name of opponents) {
      const s = splitBattlefield(zones?.battlefield?.[name] ?? []);
      m.creatures = Math.max(m.creatures, s.creatures.length);
      m.others = Math.max(m.others, s.others.length);
      m.lands = Math.max(m.lands, landStacks(s.lands).length);
    }
    return m;
  });

  // ── Offered actions → lit cards (ported from PlayAppV2) ────────────────────

  const offeredIds = $derived.by(() => {
    const ids = new Set();
    for (const a of $decision?.actions ?? []) {
      if (a.kind === 'TapForMana') continue;
      if (!answerable(a)) continue;
      if (a.object_id !== null && a.object_id !== undefined) ids.add(Number(a.object_id));
      for (const e of a.attack?.eligible ?? []) ids.add(Number(e.id));
      for (const e of a.block?.eligible ?? []) ids.add(Number(e.id));
    }
    return ids;
  });

  /**
   * Every card this seat can identify, by object id — what the decision panel
   * draws its card items from on a `Priority` decision.
   */
  const cardIndex = $derived.by(() => {
    const idx = new Map();
    const add = (c) => {
      if (c && !c.hidden && c.object_id !== null && c.object_id !== undefined) idx.set(Number(c.object_id), c);
    };
    for (const c of ownHand) add(c);
    for (const list of Object.values(zones?.battlefield ?? {})) for (const p of list) add(p);
    for (const list of Object.values(zones?.command_zone ?? {})) for (const c of list) add(c);
    for (const list of Object.values(zones?.graveyard ?? {})) for (const c of list) add(c);
    for (const c of zones?.exile ?? []) add(c);
    return idx;
  });

  /**
   * The ids some clickable card stands for right now — `looseActions`' cue to
   * NOT draw a button. On `Priority` that is every card in the index (each
   * becomes a card item). On any other decision the panel draws no card items,
   * so only cards on the table itself count: your hand and the battlefields.
   * A pile's cards are not clickable, which is why a commander-zone choice —
   * both of whose answers carry the id of a commander already in a graveyard
   * or in exile — must fall through to buttons (review finding 1: with the
   * whole index here it had no answer on screen at all).
   */
  const renderedIds = $derived.by(() => {
    if ($decision?.kind === 'Priority') return new Set(cardIndex.keys());
    const ids = new Set();
    for (const c of ownHand) if (!c?.hidden) ids.add(Number(c.object_id));
    for (const list of Object.values(zones?.battlefield ?? {})) for (const p of list) ids.add(Number(p.object_id));
    return ids;
  });

  // ── Click-through (ported from PlayAppV2) ──────────────────────────────────

  let dock = $state(null);
  let chooser = $state(null);
  let clickMessage = $state(null);
  let pickerChainOpen = $state(false);

  function clearSelection() {
    chooser = null;
    clickMessage = null;
  }

  function actionsForCard(card) {
    const id = card?.object_id;
    if (id === undefined || id === null) return [];
    return ($decision?.actions ?? []).filter(
      (a) => a.object_id !== null && a.object_id !== undefined && Number(a.object_id) === Number(id),
    );
  }

  function handleCardClick(card, group = null) {
    clearSelection();
    if (Array.isArray(group) && group.length > 1 && actionsForCard(card).length === 0) {
      card = group.find((p) => actionsForCard(p).length > 0) ?? card;
    }
    const name = card?.name ?? 'this card';
    if (card?.hidden) {
      clickMessage = 'That card is hidden from this seat.';
      return;
    }
    if (!$decision) {
      clickMessage = gameOver
        ? `The game is over — "${name}" can no longer be used.`
        : `Not your decision right now — "${name}" cannot be used.`;
      return;
    }
    const matches = actionsForCard(card);
    if (matches.length === 1) {
      dock?.beginExternal(matches[0]);
      return;
    }
    if (matches.length > 1) {
      chooser = { name, options: matches };
      return;
    }
    // An eligible attacker or blocker carries no `object_id` of its own — the
    // declare action does — so a click on one opens the declare picker.
    const declare = ($decision?.actions ?? []).find(
      (a) =>
        (a.attack?.eligible ?? []).some((e) => Number(e.id) === Number(card.object_id)) ||
        (a.block?.eligible ?? []).some((e) => Number(e.id) === Number(card.object_id)),
    );
    if (declare) {
      dock?.beginExternal(declare);
      return;
    }
    clickMessage = `No legal action for "${name}" right now.`;
  }

  function chooseOption(option) {
    clearSelection();
    dock?.beginExternal(option);
  }

  // ── Game lifecycle, export, concede (ported from PlayAppV2) ────────────────

  async function handleNewGame(opts = {}) {
    keptHand = false;
    chosenFocus = null;
    clearSelection();
    newGameKey();
    await startGame(opts);
  }

  async function handleKeep() {
    const ok = await keepHand();
    if (ok) keptHand = true;
  }

  async function handleMulligan() {
    clearSelection();
    newGameKey();
    await takeMulligan();
  }

  async function handleExportReport() {
    let url = null;
    try {
      const report = await getReport();
      const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
      url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `scutemob-report-seed${report.seed}-mull${summary?.mulligan_count ?? 0}.json`;
      document.body.appendChild(a);
      a.click();
      a.remove();
    } catch (err) {
      reportClientError(`export failed: ${err?.message ?? err}`);
    } finally {
      if (url) URL.revokeObjectURL(url);
    }
  }

  const concedeAction = $derived(($decision?.actions ?? []).find((a) => a.kind === 'Concede') ?? null);
  const concedeDisabledReason = $derived.by(() => {
    if (!$seatView) return 'no game is running';
    if ($seatView.game_over) return 'the game is already over';
    if ($loading) return 'a request is in flight';
    if (!concedeAction) return 'not your decision — the bots are acting';
    if (pickerChainOpen) return 'finish or back out of the open picker first';
    return null;
  });

  // ── The focused seat's own cell (mockup rule 3) ────────────────────────────

  /**
   * The grey-out lifts while that cell's cards can be chosen: an arrow is
   * being dragged, a picker chain is open (a target stage's candidates are
   * clickable on the table) or blockers are being declared. Hover is the
   * stylesheet's.
   */
  const liftFocused = $derived(pickerChainOpen || targeting.drag !== null || $decision?.kind === 'DeclareBlockers');

  // ── Sizing (mockup rule 5) ─────────────────────────────────────────────────

  let root = $state(null);
  const peek = $derived(ui.peek && opponents.includes(ui.peek.name) ? ui.peek : null);
  const peekGeom = $derived(peek ? peekStyle(peek.rect) : null);

  $effect(() => {
    // Everything a fit depends on: the table's contents, who is across, the
    // size controls. Read first, measure after the DOM has caught up.
    void $seatView;
    void focus;
    void settings.cap;
    void settings.auto;
    void settings.oppAuto;
    if (!root) return;
    const opts = { cap: settings.cap, auto: settings.auto, oppAuto: settings.oppAuto };
    tick().then(() => {
      if (root) fitAll(root, opts);
    });
  });

  $effect(() => {
    if (!root) return;
    const ro = new ResizeObserver(() => {
      fitAll(root, { cap: settings.cap, auto: settings.auto, oppAuto: settings.oppAuto });
    });
    ro.observe(root);
    return () => ro.disconnect();
  });

  function seatMenu(event, name) {
    event.preventDefault();
    event.stopPropagation();
    openCtx(name, event.clientX, event.clientY);
  }

  // ── Arrow targeting and the card menu (C2 items 1 and 2) ───────────────────

  /**
   * `targeting.svelte.js` is the controller; this hands it the table. Its
   * listeners sit on the root in the capture phase because the boards are
   * other components (a side cell reads any click as "bring this seat
   * across"), and everything it starts goes through `dock.beginExternal` —
   * the same picker chain a click starts. The callbacks read lazily, so this
   * effect depends on the root alone.
   */
  $effect(() => {
    if (!root) return;
    return bindTable({
      root,
      decision: () => get(decision),
      loading: () => get(loading),
      humanName: () => humanName,
      begin: (action) => {
        clearSelection();
        dock?.beginExternal(action);
      },
      openChooser: (name, options) => {
        clickMessage = null;
        chooser = { name, options };
      },
      chooserOpen: () => chooser !== null,
      say: (message) => {
        chooser = null;
        clickMessage = message;
      },
      // The seat view's entry when there is one (it carries the counters); a
      // card drawn from somewhere the index does not cover still gets a menu.
      openCardMenu: (objectId, name, x, y) => openCardCtx(cardIndex.get(objectId) ?? { object_id: objectId, name }, x, y),
    });
  });

  $effect(() => {
    // Anything that can change what a target stage means or re-create a seat
    // container: a new decision, the chooser closing, the table itself, who is
    // across, the peeked copy. Read first, re-sync once the DOM has caught up.
    void $decision;
    void $seatView;
    void chooser;
    void focus;
    void ui.peek;
    tick().then(syncTargeting);
  });
</script>

{#snippet cell(name)}
  <SideCell
    {name}
    player={view.players[name]}
    {zones}
    isActive={activePlayer === name}
    hasPrio={priority === name}
    focused={focus === name}
    lift={liftFocused}
    {offeredIds}
    onCardClick={handleCardClick}
    onFocus={bringAcross}
    onSeatMenu={seatMenu}
  />
{/snippet}

<div class="v3 t6 t7" class:arrow-dragging={targeting.drag !== null} class:server-passing={serverPassing} bind:this={root}>
  <div class="app">
    <TopBar
      {summary}
      seats={seats.length}
      loading={$loading}
      {concedeAction}
      {concedeDisabledReason}
      onNewGame={handleNewGame}
      onExport={handleExportReport}
      onConcede={() => concedeAction && dock?.beginExternal(concedeAction)}
    />
    <PhaseTrack turn={view?.turn ?? null} {humanName} />

    <div class="side left">
      {#if view}
        {#each columns.left as name (name)}{@render cell(name)}{/each}
      {/if}
    </div>

    <div class="centre">
      {#if view && humanName && zones}
        {#if focus}
          <FocusedBoard
            name={focus}
            player={view.players[focus]}
            {zones}
            isActive={activePlayer === focus}
            hasPrio={priority === focus}
            {maxRows}
            {offeredIds}
            onCardClick={handleCardClick}
            onSeatMenu={seatMenu}
          />
        {:else}
          <div class="focused"></div>
        {/if}
        <YourBoard
          name={humanName}
          player={view.players[humanName]}
          {zones}
          isActive={activePlayer === humanName}
          hasPrio={priority === humanName}
          {offeredIds}
          {gameKey}
          onCardClick={handleCardClick}
          onSeatMenu={seatMenu}
          onDragOut={beginArrowFromCard}
        />
        <HandRow cards={ownHand} {offeredIds} {gameKey} onCardClick={handleCardClick} onDragOut={beginArrowFromCard} />
      {:else}
        <div class="focused"></div>
        <div class="board v3-empty">
          <div class="v3-empty-title">No game is running.</div>
          <div class="dim">Open the menu and deal a table, or press New game in the decision panel. You sit in seat 1; the other seats are bots.</div>
        </div>
        <div class="hand"></div>
      {/if}
    </div>

    <div class="side right">
      {#if view}
        {#each columns.right as name (name)}{@render cell(name)}{/each}
        <AutoPass />
      {/if}
    </div>

    <Dock
      bind:this={dock}
      state={view}
      {humanName}
      {cardIndex}
      decision={$decision}
      loading={$loading}
      error={$error}
      passUntil={$passUntil}
      turn={view?.turn ?? null}
      {gameOver}
      {pregame}
      {chooser}
      {clickMessage}
      hasGame={!!$seatView}
      {renderedIds}
      onAct={(index, params) => act(index, params)}
      onRefresh={refresh}
      onDismissError={dismissError}
      onPassUntil={(mode) => startPassUntil(mode)}
      onCancelPassUntil={cancelPassUntil}
      onDismissPassUntil={dismissPassUntil}
      onClientError={reportClientError}
      onKeep={handleKeep}
      onMulligan={handleMulligan}
      onNewGame={() => handleNewGame()}
      onExport={handleExportReport}
      onChooseOption={chooseOption}
      onClearSelection={clearSelection}
      onChainOpenChange={(open) => (pickerChainOpen = open)}
    />
  </div>

  <!-- Floating layers live at the root: see `ui.svelte.js` for why. -->
  {#if peek && view && zones}
    <SideCell
      pop
      popStyle={peekGeom.style}
      fromRight={peekGeom.fromRight}
      name={peek.name}
      player={view.players[peek.name]}
      {zones}
      isActive={activePlayer === peek.name}
      hasPrio={priority === peek.name}
      {offeredIds}
      onCardClick={handleCardClick}
      onFocus={bringAcross}
      onSeatMenu={seatMenu}
    />
  {/if}
  <PilePopover />
  <ContextMenu state={view} {humanName} {focus} decision={$decision} loading={$loading} onFocus={bringAcross} onUse={chooseOption} />
  <Arrows />
</div>
