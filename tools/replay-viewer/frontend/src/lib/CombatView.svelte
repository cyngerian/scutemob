<script>
  /**
   * CombatView — attacker/blocker visualization for the combat phase.
   *
   * Only rendered when combat state is present in the view model.
   *
   * Props:
   *   combat (CombatView) — combat state from the view model:
   *     { attacking_player: string, attackers: AttackerView[] }
   *     AttackerView: { object_id, name, target, blockers: BlockerView[] }
   *     BlockerView: { object_id, name }
   */

  const { combat } = $props();

  /** Parse the target string into a human-readable label.
   *  Format: "player:<name>" or "planeswalker:<id>" */
  function formatTarget(target) {
    if (!target) return '(unknown target)';
    if (target.startsWith('player:')) return target.slice(7);
    if (target.startsWith('planeswalker:')) return `PW #${target.slice(13)}`;
    return target;
  }

  /** Number of attackers with no blockers. */
  const unblocked = $derived(
    (combat?.attackers ?? []).filter((a) => a.blockers.length === 0).length
  );
</script>

{#if combat}
  <div class="combat-view">
    <div class="combat-header">
      <span class="combat-title">Combat</span>
      <span class="combat-attacker">
        Attacking player: <strong>{combat.attacking_player}</strong>
      </span>
      <span class="combat-summary muted">
        {combat.attackers.length} attacker{combat.attackers.length !== 1 ? 's' : ''},
        {unblocked} unblocked
      </span>
    </div>

    {#if combat.attackers.length === 0}
      <div class="no-attackers muted">No attackers declared.</div>
    {:else}
      <div class="attackers-list">
        {#each combat.attackers as attacker (attacker.object_id)}
          <div class="attacker-row">
            <!-- Attacker card -->
            <div class="attacker-card" title="Attacker: {attacker.name}">
              <span class="card-name">{attacker.name}</span>
              <span class="card-id muted">#{attacker.object_id}</span>
            </div>

            <!-- Arrow pointing at target -->
            <div class="arrow-area">
              <div class="arrow-line"></div>
              <div class="arrow-head">▶</div>
            </div>

            <!-- Target -->
            <div class="target-box" title="Attacking: {formatTarget(attacker.target)}">
              <span class="target-label muted">Target:</span>
              <span class="target-name">{formatTarget(attacker.target)}</span>
            </div>

            <!-- Blockers (if any) -->
            {#if attacker.blockers.length > 0}
              <div class="blockers-area">
                <span class="blockers-label muted">Blocked by:</span>
                <div class="blockers-list">
                  {#each attacker.blockers as blocker (blocker.object_id)}
                    <div class="blocker-card" title="Blocker: {blocker.name}">
                      <span class="block-icon">🛡</span>
                      <span class="card-name">{blocker.name}</span>
                      <span class="card-id muted">#{blocker.object_id}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {:else}
              <div class="unblocked-badge">UNBLOCKED</div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .combat-view {
    background: #1a0e0e;
    border: 1px solid #5a2020;
    border-radius: 4px;
    padding: 0.4rem 0.6rem;
    font-family: monospace;
    font-size: 0.78rem;
  }

  .combat-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding-bottom: 0.3rem;
    margin-bottom: 0.4rem;
    border-bottom: 1px solid #3a1a1a;
    flex-wrap: wrap;
  }

  .combat-title {
    font-weight: bold;
    color: #e86;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .combat-attacker {
    color: #caa;
    font-size: 0.78rem;
  }

  .combat-attacker strong {
    color: #fa8;
  }

  .combat-summary {
    font-size: 0.72rem;
  }

  .muted {
    color: #665;
    font-size: 0.7rem;
  }

  .no-attackers {
    padding: 0.4rem;
    text-align: center;
  }

  /* Attacker rows */
  .attackers-list {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .attacker-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.25rem 0.3rem;
    background: #1e1010;
    border: 1px solid #3a1818;
    border-radius: 3px;
    flex-wrap: wrap;
  }

  /* Attacker card chip */
  .attacker-card {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: #2a1a10;
    border: 1px solid #6a3010;
    border-radius: 3px;
    padding: 0.15rem 0.4rem;
    min-width: 90px;
  }

  /* Arrow connecting attacker to target */
  .arrow-area {
    display: flex;
    align-items: center;
    color: #e64;
    flex-shrink: 0;
  }

  .arrow-line {
    width: 20px;
    height: 2px;
    background: #e64;
    flex-shrink: 0;
  }

  .arrow-head {
    font-size: 0.7rem;
    color: #e64;
    margin-left: -2px;
  }

  /* Target chip */
  .target-box {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: #1a1a2a;
    border: 1px solid #3a3a6a;
    border-radius: 3px;
    padding: 0.15rem 0.4rem;
    min-width: 70px;
  }

  .target-name {
    color: #aac;
    font-size: 0.78rem;
  }

  /* Blockers */
  .blockers-area {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    margin-left: 0.3rem;
    flex-wrap: wrap;
  }

  .blockers-list {
    display: flex;
    gap: 0.25rem;
    flex-wrap: wrap;
  }

  .blocker-card {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    background: #101a22;
    border: 1px solid #204050;
    border-radius: 3px;
    padding: 0.1rem 0.3rem;
    font-size: 0.72rem;
  }

  .block-icon {
    font-size: 0.65rem;
  }

  .card-name {
    color: #ccd;
    font-size: 0.75rem;
    font-weight: bold;
    max-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-id {
    font-size: 0.62rem;
  }

  .unblocked-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.3rem;
    background: #3a1000;
    color: #f84;
    border: 1px solid #6a2000;
    border-radius: 3px;
    font-weight: bold;
    letter-spacing: 0.04em;
  }
</style>
