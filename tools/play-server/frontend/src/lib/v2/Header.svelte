<script>
  /**
   * Header — one line: the game's clock (turn, whose turn, step) and a menu
   * (brief §3). Everything that is not part of playing the current decision
   * lives in the menu: new game, export, mana sources, concede. The menu is a
   * dropdown drawn over the page, so opening it moves nothing.
   */
  import { stepName } from '../names.js';

  const {
    summary = null,
    turn = null,
    humanName = null,
    loading = false,
    concedeAction = null,
    concedeDisabledReason = null,
    manaRows = [],
    onNewGame = null,
    onExport = null,
    onConcede = null,
    onTapMana = null,
  } = $props();

  let open = $state(false);
  let seedInput = $state('');
  let playersInput = $state('');
  let concedeArmed = $state(false);

  const mine = $derived(turn?.active_player && turn.active_player === humanName);

  function submitNewGame() {
    const opts = {};
    const seed = Number.parseInt(seedInput, 10);
    if (Number.isFinite(seed)) opts.seed = seed;
    const players = Number.parseInt(playersInput, 10);
    if (Number.isFinite(players)) opts.players = players;
    open = false;
    onNewGame?.(opts);
  }

  function confirmConcede() {
    concedeArmed = false;
    open = false;
    onConcede?.();
  }

  $effect(() => {
    if (!open) concedeArmed = false;
  });
</script>

<div class="header">
  <span class="brand">scutemob</span>
  {#if turn}
    <span class="clock">
      Turn <b>{turn.number}</b>
      · <span class:mine>{mine ? 'your turn' : `${turn.active_player}'s turn`}</span>
      · {stepName(turn.step)}
      {#if turn.priority && turn.priority !== turn.active_player}
        · <span class="prio">{turn.priority === humanName ? 'you have' : `${turn.priority} has`} priority</span>
      {/if}
    </span>
  {:else}
    <span class="clock dim">no game</span>
  {/if}
  {#if summary}
    <span class="meta">{summary.players} seats · {summary.bot} · {summary.command_count} commands</span>
  {/if}

  <div class="menu-wrap">
    <button class="menu-btn" class:open aria-expanded={open} onclick={() => (open = !open)}>≡ menu</button>
    {#if open}
      <div class="menu">
        <div class="section">
          <div class="section-title">New game</div>
          <label>seed <input type="text" inputmode="numeric" placeholder="default" bind:value={seedInput} /></label>
          <label>players <input type="text" inputmode="numeric" placeholder="4" bind:value={playersInput} /></label>
          <button class="mbtn primary" disabled={loading} onclick={submitNewGame}>Deal a new table</button>
        </div>
        <div class="section">
          <div class="section-title">Mana sources</div>
          {#if manaRows.length === 0}
            <div class="dim small">none offered right now</div>
          {:else}
            {#each manaRows as row (row.label)}
              <button class="mbtn" disabled={loading} onclick={() => { open = false; onTapMana?.(row.options[0]); }}>
                {row.label}{#if row.options.length > 1} ×{row.options.length}{/if}
              </button>
            {/each}
          {/if}
        </div>
        <div class="section">
          <button class="mbtn" disabled={!summary} onclick={() => { open = false; onExport?.(); }}>Export report</button>
        </div>
        <div class="section">
          {#if concedeArmed}
            <div class="concede-q">Concede — end your game?</div>
            <button class="mbtn danger" onclick={confirmConcede}>Yes, concede</button>
            <button class="mbtn" onclick={() => (concedeArmed = false)}>Keep playing</button>
          {:else}
            <button class="mbtn concede" disabled={concedeDisabledReason !== null || !concedeAction} onclick={() => (concedeArmed = true)}>
              Concede
            </button>
            {#if concedeDisabledReason}<div class="dim small">{concedeDisabledReason}</div>{/if}
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    height: 100%;
    padding: 0 0.6rem;
    background: #0b0b18;
    border-bottom: 1px solid #2a2a44;
    font-family: monospace;
    font-size: 0.74rem;
    white-space: nowrap;
    position: relative;
  }

  .brand { color: #adf; font-weight: bold; }
  .clock { color: #aab; }
  .clock b { color: #adf; }
  .mine { color: #6af; font-weight: bold; }
  .prio { color: #fa0; }
  .dim { color: #556; }
  .meta { color: #556; font-size: 0.64rem; }

  .menu-wrap {
    margin-left: auto;
    position: relative;
  }

  .menu-btn {
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.7rem;
    padding: 0.1rem 0.5rem;
    cursor: pointer;
  }

  .menu-btn.open { background: #2a2a58; }

  .menu {
    position: absolute;
    right: 0;
    top: 1.6rem;
    z-index: 50;
    width: 18rem;
    background: #141428;
    border: 1px solid #33335a;
    border-radius: 4px;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.7);
    padding: 0.4rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    white-space: normal;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding-bottom: 0.4rem;
    border-bottom: 1px solid #22223a;
  }

  .section:last-child { border-bottom: none; padding-bottom: 0; }

  .section-title {
    font-size: 0.62rem;
    color: #667;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  label { display: flex; justify-content: space-between; align-items: center; gap: 0.4rem; color: #889; font-size: 0.68rem; }
  input { width: 6rem; background: #0f0f22; border: 1px solid #33335a; border-radius: 3px; color: #ccd; font-family: monospace; font-size: 0.7rem; padding: 0.1rem 0.3rem; }

  .mbtn {
    text-align: left;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    cursor: pointer;
  }

  .mbtn:hover:not(:disabled) { background: #2a2a58; }
  .mbtn:disabled { opacity: 0.4; cursor: not-allowed; }
  .mbtn.primary { background: #23386a; border-color: #3a5aa0; }
  .mbtn.concede { background: #2a1418; border-color: #5a2a30; color: #d99; }
  .mbtn.danger { background: #7a1c26; border-color: #a83a46; color: #fdd; font-weight: bold; }
  .concede-q { color: #eaa; font-size: 0.7rem; }
  .small { font-size: 0.62rem; }
</style>
