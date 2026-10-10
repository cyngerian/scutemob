<script>
  /**
   * PlayAppV2 — the play surface, second design (`docs/play-surface-v2-brief.md`).
   *
   * A fixed CSS grid on `100vh`: header, opponent row, your board, prompt panel,
   * hand, and a right column holding the stack, combat and log boxes. Every
   * region has a slot that exists whether or not it has content, so nothing on
   * the page moves when the game state changes (brief principle 1). Only the
   * insides of regions scroll; the document never does.
   *
   * Data flow is the same as the first surface: `stores.js` owns every fetch
   * and the seat-redacted `SeatView` (Architecture Invariant 7 is enforced
   * server-side; this component never filters anything), `PromptPanel` owns
   * the picker chain, and this component owns the click-through — matching a
   * clicked card to the decision's actions by `object_id`, exactly as before.
   *
   * Mounted by `main.js` when the URL carries `?v2`; the first surface stays the
   * default until the owner says otherwise (brief §7).
   */
  import { onMount } from 'svelte';
  import Header from './Header.svelte';
  import OpponentCell from './OpponentCell.svelte';
  import YourBoard from './YourBoard.svelte';
  import PromptPanel from './PromptPanel.svelte';
  import HandBar from './HandBar.svelte';
  import SideColumn from './SideColumn.svelte';
  import { getReport } from '../api.js';
  import {
    seatView,
    decision,
    events,
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
    refresh();
  });

  // ── Derived view bits ──────────────────────────────────────────────────────

  const summary = $derived($seatView?.summary ?? null);
  const state = $derived($seatView?.state ?? null);
  const gameOver = $derived($seatView?.game_over ?? null);
  const humanName = $derived(summary?.human_name ?? null);
  const playerNames = $derived(state?.players ? Object.keys(state.players).sort() : []);
  const opponents = $derived(playerNames.filter((p) => p !== humanName));
  const ownHand = $derived(humanName !== null ? (state?.zones?.hand?.[humanName] ?? []) : []);
  const combat = $derived(state?.combat ?? null);

  let keptHand = $state(false);
  const pregame = $derived(!!summary?.pregame && !keptHand);

  async function handleNewGame(opts = {}) {
    keptHand = false;
    clearSelection();
    await startGame(opts);
  }

  async function handleKeep() {
    const ok = await keepHand();
    if (ok) keptHand = true;
  }

  async function handleMulligan() {
    clearSelection();
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

  // ── Offered actions → lit cards ────────────────────────────────────────────

  const offeredIds = $derived.by(() => {
    const ids = new Set();
    for (const a of $decision?.actions ?? []) {
      if (a.kind === 'TapForMana') continue;
      if (a.object_id !== null && a.object_id !== undefined) ids.add(Number(a.object_id));
      for (const e of a.attack?.eligible ?? []) ids.add(Number(e.id));
      for (const e of a.block?.eligible ?? []) ids.add(Number(e.id));
    }
    return ids;
  });

  /**
   * Every object id the surface draws right now — what `looseActions` checks
   * an action's `object_id` against, so an action on anything NOT drawn (a
   * graveyard card's flashback, a dungeon room, a redacted id) still gets a
   * button in the prompt panel. Graveyard and exile are not drawn in B1.
   */
  const renderedIds = $derived.by(() => {
    const ids = new Set();
    const z = state?.zones ?? {};
    for (const c of ownHand) if (!c?.hidden) ids.add(Number(c.object_id));
    for (const list of Object.values(z.battlefield ?? {})) for (const p of list) ids.add(Number(p.object_id));
    for (const list of Object.values(z.command_zone ?? {})) for (const c of list) if (!c?.hidden) ids.add(Number(c.object_id));
    return ids;
  });

  /** Mana sources folded by label, for the header menu (G10: reachable, never hidden). */
  const manaRows = $derived.by(() => {
    const byLabel = new Map();
    for (const a of $decision?.actions ?? []) {
      if (a.kind !== 'TapForMana') continue;
      if (byLabel.has(a.label)) byLabel.get(a.label).options.push(a);
      else byLabel.set(a.label, { label: a.label, options: [a] });
    }
    return [...byLabel.values()];
  });

  // ── Click-through (ported from the first surface) ──────────────────────────

  let prompt = $state(null);
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
      prompt?.beginExternal(matches[0]);
      return;
    }
    if (matches.length > 1) {
      chooser = { name, options: matches };
      return;
    }
    clickMessage = `No legal action for "${name}" right now.`;
  }

  function chooseOption(option) {
    clearSelection();
    prompt?.beginExternal(option);
  }

  // ── Concede (header menu) ──────────────────────────────────────────────────

  const concedeAction = $derived(($decision?.actions ?? []).find((a) => a.kind === 'Concede') ?? null);
  const concedeDisabledReason = $derived.by(() => {
    if (!$seatView) return 'no game is running';
    if ($seatView.game_over) return 'the game is already over';
    if ($loading) return 'a request is in flight';
    if (!concedeAction) return 'not your decision — the bots are acting';
    if (pickerChainOpen) return 'finish or back out of the open picker first';
    return null;
  });

  function handleConcede() {
    if (concedeAction) prompt?.beginExternal(concedeAction);
  }
</script>

<div class="v2">
  <div class="g-header">
    <Header
      {summary}
      turn={state?.turn ?? null}
      {humanName}
      loading={$loading}
      {concedeAction}
      {concedeDisabledReason}
      {manaRows}
      onNewGame={handleNewGame}
      onExport={handleExportReport}
      onConcede={handleConcede}
      onTapMana={(option) => prompt?.beginExternal(option)}
    />
  </div>

  <div class="g-opps" style="--n: {Math.max(opponents.length, 1)}">
    {#each opponents as pname (pname)}
      <OpponentCell
        playerName={pname}
        player={state.players[pname]}
        commanders={state.zones?.command_zone?.[pname] ?? []}
        permanents={state.zones?.battlefield?.[pname] ?? []}
        isActive={state.turn?.active_player === pname}
        hasPriority={state.turn?.priority === pname}
        {offeredIds}
        onCardClick={handleCardClick}
      />
    {/each}
    {#if opponents.length === 0}
      <div class="placeholder">opponents</div>
    {/if}
  </div>

  <div class="g-board">
    {#if state && humanName}
      <YourBoard
        playerName={humanName}
        player={state.players[humanName]}
        commanders={state.zones?.command_zone?.[humanName] ?? []}
        permanents={state.zones?.battlefield?.[humanName] ?? []}
        isActive={state.turn?.active_player === humanName}
        hasPriority={state.turn?.priority === humanName}
        {offeredIds}
        onCardClick={handleCardClick}
      />
    {:else}
      <div class="placeholder tall">
        <div class="ph-title">No game is running.</div>
        <div class="ph-text">Open the menu and deal a table, or press New game below. You sit in seat 1; the other seats are bots.</div>
      </div>
    {/if}
  </div>

  <div class="g-prompt">
    <PromptPanel
      bind:this={prompt}
      decision={$decision}
      loading={$loading}
      error={$error}
      passUntil={$passUntil}
      {humanName}
      turn={state?.turn ?? null}
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

  <div class="g-side">
    <SideColumn stack={state?.zones?.stack ?? []} {combat} events={$events} />
  </div>

  <div class="g-hand">
    <HandBar cards={ownHand} {offeredIds} onCardClick={handleCardClick} />
  </div>
</div>

<style>
  /*
    The grid. Row heights are fixed numbers except the board, which takes the
    rest of 100vh. Column 2 is the 320px side column (owner decision 2026-10-10,
    brief §8.3). Nothing here depends on content.
  */
  .v2 {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
    grid-template-rows: 32px 236px minmax(0, 1fr) 150px 140px;
    grid-template-areas:
      'header header'
      'opps side'
      'board side'
      'prompt side'
      'hand hand';
    gap: 6px;
    height: 100vh;
    padding: 0 6px 6px;
    box-sizing: border-box;
    overflow: hidden;
    font-family: monospace;
  }

  .g-header { grid-area: header; min-width: 0; }
  .g-opps { grid-area: opps; min-height: 0; min-width: 0; display: grid; grid-template-columns: repeat(var(--n, 3), minmax(0, 1fr)); gap: 6px; }
  .g-board { grid-area: board; min-height: 0; min-width: 0; }
  .g-prompt { grid-area: prompt; min-height: 0; min-width: 0; }
  .g-side { grid-area: side; min-height: 0; min-width: 0; }
  .g-hand { grid-area: hand; min-height: 0; min-width: 0; }

  .placeholder {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #445;
    font-size: 0.72rem;
    border: 1px dashed #22223a;
    border-radius: 4px;
  }

  .ph-title { color: #aab; font-size: 0.95rem; margin-bottom: 0.4rem; }
  .ph-text { color: #667; font-size: 0.76rem; max-width: 34rem; text-align: center; line-height: 1.5; }
</style>
