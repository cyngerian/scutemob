<script>
  /**
   * OpponentCell — one opponent seat in the top row (brief §3).
   *
   * Fixed-height cell; its body scrolls. Two header lines (identity + counts),
   * then non-land permanents as chips, then lands as tap-state pips — a bot's
   * twelve basics are not information (owner decision 2026-10-10, brief §8.1).
   * Eliminated seats grey to their header; the row does not reflow.
   */
  import Card from './Card.svelte';

  const {
    playerName,
    player,
    commanders = [],
    permanents = [],
    isActive = false,
    hasPriority = false,
    offeredIds = null,
    onCardClick = null,
  } = $props();

  const eliminated = $derived(!!(player?.has_lost || player?.has_conceded));
  const lands = $derived(permanents.filter((p) => (p.card_types ?? []).includes('Land')));
  const nonLands = $derived(permanents.filter((p) => !(p.card_types ?? []).includes('Land')));
  const tappedLands = $derived(lands.filter((p) => p.tapped).length);

  function offered(c) {
    return !!offeredIds && offeredIds.has(Number(c?.object_id));
  }

  const cmdDamage = $derived.by(() => {
    const out = [];
    for (const [, byCard] of Object.entries(player?.commander_damage_received ?? {})) {
      for (const [card, dmg] of Object.entries(byCard)) if (dmg > 0) out.push({ card, dmg });
    }
    return out;
  });
</script>

<div class="opp" class:eliminated class:is-active={isActive} class:has-priority={hasPriority}>
  <div class="head">
    <span class="name">{playerName}</span>
    {#if isActive}<span class="badge active">ACTIVE</span>{/if}
    {#if hasPriority}<span class="badge prio">PRIO</span>{/if}
    {#if eliminated}<span class="badge out">{player?.has_conceded ? 'CONCEDED' : 'OUT'}</span>{/if}
    <span class="life" class:low={(player?.life ?? 20) <= 5}>{player?.life ?? '?'}</span>
  </div>
  <div class="counts">
    <span>hand {player?.hand_size ?? 0}</span>
    <span>lib {player?.library_size ?? 0}</span>
    {#if (player?.graveyard_size ?? 0) > 0}<span>gy {player.graveyard_size}</span>{/if}
    {#if (player?.poison ?? 0) > 0}<span class="poison">☠ {player.poison}</span>{/if}
    {#each cmdDamage as d (d.card)}<span class="cmddmg" class:lethal={d.dmg >= 21}>⚔ {d.dmg}</span>{/each}
    {#each commanders as c (c.object_id)}
      <Card card={c} size="chip" offered={offered(c)} onClick={onCardClick} />
    {/each}
  </div>
  {#if !eliminated}
    <div class="body">
      <div class="perms">
        {#each nonLands as p (p.object_id)}
          <Card card={p} size="small" offered={offered(p)} onClick={onCardClick} />
        {/each}
        {#if nonLands.length === 0}<span class="none">no permanents</span>{/if}
      </div>
      <div class="lands" title="{lands.length} lands, {tappedLands} tapped">
        <span class="lands-label">lands {lands.length}</span>
        {#each lands as p (p.object_id)}
          <span class="pip" class:tapped={p.tapped}></span>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .opp {
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    padding: 0.25rem 0.4rem;
    background: #10101e;
    border: 1px solid #22223a;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.7rem;
    overflow: hidden;
  }

  .opp.is-active {
    border-color: #4a7ac0;
    background: #121230;
  }

  .opp.has-priority {
    border-color: #fa0;
  }

  .opp.eliminated {
    opacity: 0.4;
    filter: grayscale(0.7);
  }

  .head,
  .counts {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-wrap: nowrap;
    min-height: 1.2rem;
    overflow: hidden;
  }

  .name {
    font-weight: bold;
    color: #ddd;
    font-size: 0.8rem;
  }

  .badge {
    font-size: 0.56rem;
    padding: 0 0.25rem;
    border-radius: 2px;
    font-weight: bold;
  }

  .active { background: #1a4a8a; color: #9cf; }
  .prio { background: #4a3a0a; color: #fa0; }
  .out { background: #4a1a1a; color: #f66; }

  .life {
    margin-left: auto;
    font-size: 1rem;
    font-weight: bold;
    color: #aef;
  }

  .life.low {
    color: #f44;
  }

  .counts {
    color: #778;
    font-size: 0.64rem;
  }

  .poison { color: #9f6; }
  .cmddmg { color: #c88; }
  .cmddmg.lethal { color: #f44; }

  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding-top: 0.25rem;
  }

  .perms {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    align-content: flex-start;
  }

  .none {
    color: #445;
    font-size: 0.64rem;
  }

  .lands {
    display: flex;
    align-items: center;
    gap: 0.15rem;
    flex-wrap: wrap;
  }

  .lands-label {
    color: #686;
    font-size: 0.62rem;
    margin-right: 0.25rem;
  }

  .pip {
    display: inline-block;
    width: 8px;
    height: 12px;
    border-radius: 2px;
    background: #4a8;
    border: 1px solid #2a5a3a;
  }

  .pip.tapped {
    background: transparent;
    border-color: #5a4a20;
  }
</style>
