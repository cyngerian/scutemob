<script>
  /**
   * ZoneBattlefield — permanent grid for one player's battlefield.
   *
   * Props:
   *   permanents (PermanentView[]) — list of permanents controlled by this player
   *   playerName (string) — player label for the zone header
   */
  const { permanents = [], playerName } = $props();

  // Group permanents by rough type category
  const groups = $derived(() => {
    const creatures = [];
    const lands = [];
    const artifacts = [];
    const enchantments = [];
    const planeswalkers = [];
    const other = [];

    for (const p of permanents) {
      const types = p.card_types ?? [];
      if (types.includes('Creature')) {
        creatures.push(p);
      } else if (types.includes('Land')) {
        lands.push(p);
      } else if (types.includes('Planeswalker')) {
        planeswalkers.push(p);
      } else if (types.includes('Artifact')) {
        artifacts.push(p);
      } else if (types.includes('Enchantment')) {
        enchantments.push(p);
      } else {
        other.push(p);
      }
    }
    return { creatures, lands, artifacts, enchantments, planeswalkers, other };
  });

  function typeLineStr(p) {
    const parts = [];
    if (p.supertypes?.length) parts.push(...p.supertypes);
    if (p.card_types?.length) parts.push(...p.card_types);
    if (p.subtypes?.length) parts.push('—', ...p.subtypes);
    return parts.join(' ');
  }
</script>

<div class="zone-battlefield">
  <div class="zone-header">
    <span class="zone-label">Battlefield</span>
    <span class="zone-count muted">{permanents.length} permanents</span>
  </div>

  {#if permanents.length === 0}
    <div class="empty-zone muted">— empty —</div>
  {:else}
    <!-- Creatures -->
    {#if groups().creatures.length > 0}
      <div class="perm-group">
        <div class="group-label">Creatures ({groups().creatures.length})</div>
        <div class="perm-grid">
          {#each groups().creatures as p (p.object_id)}
            <div
              class="permanent-card"
              class:tapped={p.tapped}
              class:summoning-sick={p.summoning_sick}
              class:is-commander={p.is_commander}
              class:is-token={p.is_token}
              title={typeLineStr(p)}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.is_commander}
                <span class="badge badge-cmd" title="Commander">CMD</span>
              {/if}
              {#if p.is_token}
                <span class="badge badge-token" title="Token">TKN</span>
              {/if}
              {#if p.tapped}
                <span class="badge badge-tapped" title="Tapped">TAP</span>
              {/if}
              {#if p.summoning_sick}
                <span class="badge badge-sick" title="Summoning sickness">SICK</span>
              {/if}

              <!-- Power/Toughness -->
              {#if p.power !== null && p.toughness !== null}
                <div class="pt-box">
                  <span class="pt-value" class:pt-damaged={p.damage_marked > 0}>
                    {p.power}/{p.toughness}
                    {#if p.damage_marked > 0}
                      <span class="dmg-marker" title="{p.damage_marked} damage marked">
                        -{p.damage_marked}
                      </span>
                    {/if}
                  </span>
                </div>
              {/if}

              <!-- Counters -->
              {#each Object.entries(p.counters ?? {}) as [ct, n]}
                {#if n > 0}
                  <span class="counter-badge counter-{ct.replace('/', '').replace('+', 'p').replace('-', 'm')}" title="{ct} counter x{n}">
                    {ct}×{n}
                  </span>
                {/if}
              {/each}

              <!-- Keywords (abbreviated) -->
              {#if p.keywords?.length > 0}
                <div class="keyword-list">
                  {#each p.keywords as kw}
                    <span class="kw-badge" title={kw}>{kw.slice(0, 3).toUpperCase()}</span>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Lands -->
    {#if groups().lands.length > 0}
      <div class="perm-group">
        <div class="group-label">Lands ({groups().lands.length})</div>
        <div class="perm-grid perm-grid-lands">
          {#each groups().lands as p (p.object_id)}
            <div
              class="permanent-card permanent-land"
              class:tapped={p.tapped}
              class:is-commander={p.is_commander}
              title={typeLineStr(p)}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
              {#each Object.entries(p.counters ?? {}) as [ct, n]}
                {#if n > 0}
                  <span class="counter-badge">{ct}×{n}</span>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Planeswalkers -->
    {#if groups().planeswalkers.length > 0}
      <div class="perm-group">
        <div class="group-label">Planeswalkers ({groups().planeswalkers.length})</div>
        <div class="perm-grid">
          {#each groups().planeswalkers as p (p.object_id)}
            <div
              class="permanent-card permanent-pw"
              class:tapped={p.tapped}
              title={typeLineStr(p)}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
              {#each Object.entries(p.counters ?? {}) as [ct, n]}
                {#if n > 0}
                  <span class="counter-badge counter-loyalty" title="Loyalty">{ct}×{n}</span>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Artifacts & Enchantments -->
    {#if groups().artifacts.length + groups().enchantments.length > 0}
      <div class="perm-group">
        <div class="group-label">
          Artifacts/Enchantments ({groups().artifacts.length + groups().enchantments.length})
        </div>
        <div class="perm-grid">
          {#each [...groups().artifacts, ...groups().enchantments] as p (p.object_id)}
            <div
              class="permanent-card"
              class:tapped={p.tapped}
              class:permanent-artifact={p.card_types?.includes('Artifact')}
              class:permanent-enchantment={p.card_types?.includes('Enchantment')}
              title={typeLineStr(p)}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
              {#if p.attached_to}
                <span class="badge badge-attached" title="Attached to object {p.attached_to}">ATT</span>
              {/if}
              {#each Object.entries(p.counters ?? {}) as [ct, n]}
                {#if n > 0}
                  <span class="counter-badge">{ct}×{n}</span>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Other -->
    {#if groups().other.length > 0}
      <div class="perm-group">
        <div class="group-label">Other ({groups().other.length})</div>
        <div class="perm-grid">
          {#each groups().other as p (p.object_id)}
            <div
              class="permanent-card"
              class:tapped={p.tapped}
              title={typeLineStr(p)}
            >
              <div class="perm-name">{p.name}</div>
              {#if p.tapped}
                <span class="badge badge-tapped">TAP</span>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .zone-battlefield {
    background: #0e1a12;
    border: 1px solid #1e3a22;
    border-radius: 4px;
    padding: 0.4rem 0.5rem;
    font-family: monospace;
    font-size: 0.78rem;
  }

  .zone-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.4rem;
    border-bottom: 1px solid #1e3a22;
    padding-bottom: 0.25rem;
  }

  .zone-label {
    color: #4a8;
    font-weight: bold;
    font-size: 0.8rem;
  }

  .muted {
    color: #556;
    font-size: 0.75rem;
  }

  .empty-zone {
    text-align: center;
    padding: 0.5rem;
    font-size: 0.75rem;
  }

  .perm-group {
    margin-bottom: 0.4rem;
  }

  .group-label {
    color: #668;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.2rem;
  }

  .perm-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .perm-grid-lands {
    gap: 0.2rem;
  }

  .permanent-card {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.3rem 0.4rem;
    background: #1a2820;
    border: 1px solid #2a4830;
    border-radius: 3px;
    min-width: 70px;
    max-width: 100px;
    transition: border-color 0.1s;
  }

  .permanent-card:hover {
    border-color: #4a8060;
  }

  .permanent-card.tapped {
    background: #1a2010;
    border-color: #4a4020;
    opacity: 0.8;
  }

  .permanent-card.summoning-sick {
    border-style: dashed;
    border-color: #4a4a20;
  }

  .permanent-card.is-commander {
    border-color: #8a6020;
    background: #221a10;
  }

  .permanent-card.is-token {
    border-style: dotted;
    border-color: #3a3a60;
    background: #14142a;
  }

  .permanent-land {
    min-width: 50px;
    max-width: 70px;
    background: #121a14;
  }

  .permanent-artifact {
    background: #1a1a22;
    border-color: #3a3a5a;
  }

  .permanent-enchantment {
    background: #1a1224;
    border-color: #3a2a5a;
  }

  .permanent-pw {
    background: #2a1a1a;
    border-color: #5a2a2a;
    min-width: 80px;
  }

  .perm-name {
    color: #ccd;
    font-size: 0.72rem;
    font-weight: bold;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 90px;
  }

  .badge {
    font-size: 0.6rem;
    padding: 0.05rem 0.25rem;
    border-radius: 2px;
    font-weight: bold;
    align-self: flex-start;
  }

  .badge-cmd {
    background: #4a3000;
    color: #fa0;
    border: 1px solid #8a6000;
  }

  .badge-token {
    background: #1a1a40;
    color: #88a;
  }

  .badge-tapped {
    background: #3a2800;
    color: #a80;
  }

  .badge-sick {
    background: #1a3a1a;
    color: #6a6;
  }

  .badge-attached {
    background: #1a2040;
    color: #66a;
  }

  .pt-box {
    margin-top: 0.1rem;
  }

  .pt-value {
    color: #aef;
    font-weight: bold;
    font-size: 0.75rem;
  }

  .pt-value.pt-damaged {
    color: #f84;
  }

  .dmg-marker {
    color: #f44;
    font-size: 0.65rem;
    margin-left: 0.15rem;
  }

  .counter-badge {
    font-size: 0.6rem;
    padding: 0.05rem 0.2rem;
    border-radius: 2px;
    background: #2a2a40;
    color: #aac;
    align-self: flex-start;
  }

  .counter-loyalty {
    background: #1a2a5a;
    color: #8af;
  }

  .keyword-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.1rem;
  }

  .kw-badge {
    font-size: 0.55rem;
    padding: 0.05rem 0.15rem;
    border-radius: 2px;
    background: #1a3a4a;
    color: #68a;
  }
</style>
