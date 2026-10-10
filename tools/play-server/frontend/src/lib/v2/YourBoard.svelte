<script>
  /**
   * YourBoard — the viewer seat's board, the largest region (brief §3).
   *
   * One header line (identity, life, counts, mana pool, commander chips), then
   * creatures, then other non-land permanents, then lands in their own bottom
   * row with same-name lands stacked by `(name, tapped)` (G13 kept). A chip the
   * current decision offers an action for is lit; clicking it is the
   * click-through the app owns.
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

  function offered(c) {
    return !!offeredIds && offeredIds.has(Number(c?.object_id));
  }

  const creatures = $derived(permanents.filter((p) => (p.card_types ?? []).includes('Creature')));
  const lands = $derived(
    permanents.filter((p) => !(p.card_types ?? []).includes('Creature') && (p.card_types ?? []).includes('Land')),
  );
  const others = $derived(
    permanents.filter(
      (p) => !(p.card_types ?? []).includes('Creature') && !(p.card_types ?? []).includes('Land'),
    ),
  );

  /** Land stacks keyed on the same fungibility string the old surface used. */
  const landStacks = $derived.by(() => {
    const byKey = new Map();
    for (const p of lands) {
      const counters = Object.entries(p.counters ?? {})
        .filter(([, n]) => n > 0)
        .sort()
        .map(([ct, n]) => `${ct}=${n}`)
        .join(',');
      const key = JSON.stringify([p.name, !!p.tapped, counters, p.attached_to ?? null, !!p.is_commander, !!p.is_token]);
      if (byKey.has(key)) byKey.get(key).push(p);
      else byKey.set(key, [p]);
    }
    return [...byKey.entries()].map(([key, members]) => ({ key, members }));
  });

  /** A stack nominates the member with an offered action, else its first. */
  function clickStack(members) {
    const pick = members.find((m) => offered(m)) ?? members[0];
    onCardClick?.(pick, members);
  }

  const MANA = [
    ['white', 'W', 'w'], ['blue', 'U', 'u'], ['black', 'B', 'b'], ['red', 'R', 'r'], ['green', 'G', 'g'], ['colorless', 'C', 'c'],
  ];
  const pool = $derived(MANA.filter(([k]) => (player?.mana_pool?.[k] ?? 0) > 0).map(([k, l, c]) => ({ k, l, c, n: player.mana_pool[k] })));

  const cmdDamage = $derived.by(() => {
    const out = [];
    for (const [, byCard] of Object.entries(player?.commander_damage_received ?? {})) {
      for (const [card, dmg] of Object.entries(byCard)) if (dmg > 0) out.push({ card, dmg });
    }
    return out;
  });
</script>

<div class="board" class:is-active={isActive} class:has-priority={hasPriority}>
  <div class="head">
    <span class="you">YOU</span>
    <span class="name">{playerName}</span>
    {#if isActive}<span class="badge active">YOUR TURN</span>{/if}
    {#if hasPriority}<span class="badge prio">PRIORITY</span>{/if}
    <span class="counts">
      hand {player?.hand_size ?? 0} · lib {player?.library_size ?? 0}
      {#if (player?.graveyard_size ?? 0) > 0}· gy {player.graveyard_size}{/if}
      {#if (player?.land_plays_remaining ?? 0) > 0}· <span class="drop">land drop available</span>{/if}
      {#if (player?.poison ?? 0) > 0}· <span class="poison">☠ {player.poison}</span>{/if}
      {#each cmdDamage as d (d.card)}· <span class="cmddmg" class:lethal={d.dmg >= 21}>⚔ {d.card} {d.dmg}</span>{/each}
    </span>
    {#each pool as m (m.k)}<span class="mana mana-{m.c}">{m.l}{m.n > 1 ? m.n : ''}</span>{/each}
    {#each commanders as c (c.object_id)}
      <Card card={c} size="chip" offered={offered(c)} onClick={onCardClick} />
    {/each}
    <span class="life" class:low={(player?.life ?? 20) <= 5}>{player?.life ?? '?'}</span>
  </div>

  <div class="body">
    <div class="row">
      <span class="row-label">creatures {creatures.length}</span>
      <div class="chips">
        {#each creatures as p (p.object_id)}
          <Card card={p} size="perm" offered={offered(p)} onClick={onCardClick} />
        {/each}
      </div>
    </div>
    <div class="row">
      <span class="row-label">other {others.length}</span>
      <div class="chips">
        {#each others as p (p.object_id)}
          <Card card={p} size="perm" offered={offered(p)} onClick={onCardClick} />
        {/each}
      </div>
    </div>
  </div>

  <div class="lands">
    <span class="row-label">lands {lands.length}</span>
    <div class="chips">
      {#each landStacks as s (s.key)}
        <Card
          card={s.members[0]}
          size="small"
          count={s.members.length}
          offered={s.members.some(offered)}
          onClick={onCardClick === null ? null : () => clickStack(s.members)}
        />
      {/each}
    </div>
  </div>
</div>

<style>
  .board {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding: 0.25rem 0.5rem;
    background: #0e1a12;
    border: 1px solid #1e3a22;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.72rem;
    overflow: hidden;
  }

  .board.is-active {
    border-color: #3a7a4a;
  }

  .board.has-priority {
    border-color: #fa0;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-height: 1.5rem;
    flex-shrink: 0;
    overflow: hidden;
    white-space: nowrap;
  }

  .you {
    color: #6af;
    font-size: 0.6rem;
    border: 1px solid #2a4a7a;
    border-radius: 2px;
    padding: 0 0.2rem;
  }

  .name {
    font-weight: bold;
    color: #ddd;
    font-size: 0.82rem;
  }

  .badge {
    font-size: 0.58rem;
    padding: 0 0.3rem;
    border-radius: 2px;
    font-weight: bold;
  }

  .active { background: #1a4a8a; color: #9cf; }
  .prio { background: #4a3a0a; color: #fa0; }

  .counts {
    color: #778;
    font-size: 0.66rem;
  }

  .drop { color: #8c4; }
  .poison { color: #9f6; }
  .cmddmg { color: #c88; }
  .cmddmg.lethal { color: #f44; }

  .mana {
    font-size: 0.62rem;
    font-weight: bold;
    padding: 0 0.3rem;
    border-radius: 2px;
  }

  .mana-w { background: #6a6030; color: #ffe; }
  .mana-u { background: #1a3a6a; color: #adf; }
  .mana-b { background: #2a1a4a; color: #d9f; }
  .mana-r { background: #5a2a1a; color: #fba; }
  .mana-g { background: #1a4a2a; color: #afa; }
  .mana-c { background: #3a3a3a; color: #ccc; }

  .life {
    margin-left: auto;
    font-size: 1.1rem;
    font-weight: bold;
    color: #aef;
  }

  .life.low {
    color: #f44;
  }

  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding-top: 0.25rem;
  }

  .row {
    display: flex;
    gap: 0.4rem;
    align-items: flex-start;
  }

  .row-label {
    flex: 0 0 5.5rem;
    color: #586;
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding-top: 0.2rem;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    min-height: 44px;
  }

  .lands {
    display: flex;
    gap: 0.4rem;
    align-items: flex-start;
    flex-shrink: 0;
    max-height: 90px;
    overflow-y: auto;
    border-top: 1px solid #1e3a22;
    padding-top: 0.25rem;
    margin-top: 0.25rem;
  }

  .lands .chips {
    min-height: 36px;
  }
</style>
