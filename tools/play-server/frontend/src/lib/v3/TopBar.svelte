<script>
  /**
   * TopBar — the mockup's `.top` row: the table's facts, the card-sizing
   * controls (the mockup's dashed group, kept as the real controls), the card
   * window button and the menu (new game, export, concede — ported from the v2
   * header).
   *
   * The seed is not shown because the seat view does not carry it:
   * `GameSummary` leaves it out on purpose (Architecture Invariant 7 — with the
   * player count and bot it rebuilds every other seat's hand and library). What
   * the summary does expose is shown: seats and the bot kind.
   */
  import { settings, setCap, setAuto, setOppAuto } from './ui.svelte.js';

  const {
    summary = null,
    seats = null,
    loading = false,
    concedeAction = null,
    concedeDisabledReason = null,
    onNewGame = null,
    onExport = null,
    onConcede = null,
  } = $props();

  let open = $state(false);
  let seedInput = $state('');
  let playersInput = $state('');
  let botInput = $state('');
  let concedeArmed = $state(false);
  let vw = $state(0);
  let vh = $state(0);

  const seatCount = $derived(Array.isArray(seats) ? seats.length : (seats ?? summary?.players ?? null));

  function submitNewGame() {
    const opts = {};
    const seed = Number.parseInt(seedInput, 10);
    if (Number.isFinite(seed)) opts.seed = seed;
    const players = Number.parseInt(playersInput, 10);
    if (Number.isFinite(players)) opts.players = players;
    if (botInput) opts.bot = botInput;
    open = false;
    onNewGame?.(opts);
  }

  function confirmConcede() {
    concedeArmed = false;
    open = false;
    onConcede?.();
  }

  function openCardWindow() {
    window.open('?card-window', 'scutemob-card', 'width=760,height=1060');
  }

  $effect(() => {
    if (!open) {
      concedeArmed = false;
      return;
    }
    function onMouseDown(event) {
      if (!event.target?.closest?.('.v3-menu-wrap')) open = false;
    }
    window.addEventListener('mousedown', onMouseDown);
    return () => window.removeEventListener('mousedown', onMouseDown);
  });
</script>

<svelte:window bind:innerWidth={vw} bind:innerHeight={vh} />

<div class="top">
  <b style="color:#adf">scutemob</b>
  <span class="dim">
    {#if summary}{seatCount} seats · {summary.bot} bots{:else}no game{/if}
  </span>
  <span class="mock">
    card size
    <input
      type="range"
      min="44"
      max="140"
      value={settings.cap}
      oninput={(e) => setCap(Number(e.currentTarget.value))}
    />
    <span>{settings.cap}px</span>
    <label><input type="checkbox" checked={settings.auto} onchange={(e) => setAuto(e.currentTarget.checked)} /> auto-fit</label>
    <label><input type="checkbox" checked={settings.oppAuto} onchange={(e) => setOppAuto(e.currentTarget.checked)} /> opp auto-size</label>
    · <span>viewport {vw}×{vh}</span>
  </span>
  <span class="spacer"></span>
  <button class="btn" onclick={openCardWindow}>card window ↗</button>
  <div class="v3-menu-wrap">
    <button class="btn" aria-expanded={open} onclick={() => (open = !open)}>≡ menu</button>
    {#if open}
      <div class="v3-menu">
        <div class="sec">
          <div class="sec-title">New game</div>
          <label>players <input type="text" inputmode="numeric" placeholder="4" bind:value={playersInput} /></label>
          <label>
            bots
            <select bind:value={botInput}>
              <option value="">server default</option>
              <option value="heuristic">Heuristic</option>
              <option value="random">Random</option>
            </select>
          </label>
          <label>seed <input type="text" inputmode="numeric" placeholder="default" bind:value={seedInput} /></label>
          <button class="btn primary" disabled={loading} onclick={submitNewGame}>Deal a new table</button>
        </div>
        <div class="sec">
          <button class="btn" disabled={!summary} onclick={() => { open = false; onExport?.(); }}>Export report</button>
        </div>
        <div class="sec">
          {#if concedeArmed}
            <div class="why warn">Concede — end your game?</div>
            <button class="btn danger" onclick={confirmConcede}>Yes, concede</button>
            <button class="btn" onclick={() => (concedeArmed = false)}>Keep playing</button>
          {:else}
            <button class="btn concede" disabled={concedeDisabledReason !== null || !concedeAction} onclick={() => (concedeArmed = true)}>
              Concede
            </button>
            {#if concedeDisabledReason}<div class="why">{concedeDisabledReason}</div>{/if}
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
