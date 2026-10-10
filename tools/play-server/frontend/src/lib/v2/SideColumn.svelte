<script>
  /**
   * SideColumn — the 320px right column: a fixed stack box, a fixed combat box,
   * and the log in the remaining height (brief §3). The two boxes keep their
   * height when empty and say so, which is what keeps the board still when a
   * spell goes on the stack or combat begins.
   */
  import EventFeed from '../EventFeed.svelte';

  const { stack = [], combat = null, events = [] } = $props();

  function kindLabel(kind) {
    switch (kind) {
      case 'spell': return 'spell';
      case 'activated_ability': return 'ability';
      case 'triggered_ability': return 'trigger';
      case 'cascade_trigger': return 'cascade';
      case 'storm_trigger': return 'storm';
      case 'room_ability': return 'room';
      default: return kind ?? '';
    }
  }

  function formatTarget(t) {
    if (!t) return '?';
    if (t.startsWith('player:')) return t.slice(7);
    if (t.startsWith('planeswalker:')) return `${t.slice(13)} (pw)`;
    return t;
  }

  /** Top of stack first: the wire is bottom-first (last element = top). */
  const topFirst = $derived([...stack].reverse());
  const attackers = $derived(combat?.attackers ?? []);
</script>

<div class="side">
  <section class="box stack">
    <div class="box-head">
      <span class="box-title">STACK</span>
      <span class="box-count">{stack.length}</span>
    </div>
    <div class="box-body">
      {#if topFirst.length === 0}
        <div class="empty">empty</div>
      {:else}
        {#each topFirst as item, i (item.id)}
          <div class="stack-item" class:top={i === 0}>
            <span class="pos">{i + 1}</span>
            <span class="kind kind-{item.kind}">{kindLabel(item.kind)}</span>
            <span class="src">{item.source_name}</span>
            <span class="ctl">{item.controller}</span>
            {#if item.is_copy}<span class="copy">copy</span>{/if}
            {#if item.targets?.length}
              <span class="targets">→ {item.targets.join(', ')}</span>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </section>

  <section class="box combat">
    <div class="box-head">
      <span class="box-title">COMBAT</span>
      {#if combat}<span class="box-count">{combat.attacking_player} attacks</span>{/if}
    </div>
    <div class="box-body">
      {#if attackers.length === 0}
        <div class="empty">{combat ? 'no attackers declared' : 'not in combat'}</div>
      {:else}
        {#each attackers as a (a.object_id)}
          <div class="atk">
            <span class="atk-name">{a.name}</span>
            <span class="arrow">→</span>
            <span class="tgt">{formatTarget(a.target)}</span>
            {#if a.blockers?.length}
              <span class="blk">blocked by {a.blockers.map((b) => b.name).join(', ')}</span>
            {:else}
              <span class="unblocked">unblocked</span>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </section>

  <section class="box log">
    <EventFeed {events} />
  </section>
</div>

<style>
  .side {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    height: 100%;
    min-height: 0;
    font-family: monospace;
  }

  .box {
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: #111120;
    border: 1px solid #222238;
    border-radius: 4px;
    overflow: hidden;
  }

  .box.stack {
    flex: 0 0 150px;
    background: #16102a;
    border-color: #2a2050;
  }

  .box.combat {
    flex: 0 0 120px;
    background: #1a0e0e;
    border-color: #4a2020;
  }

  .box.log {
    flex: 1;
  }

  .box-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 0.2rem 0.45rem;
    border-bottom: 1px solid #22223a;
    flex-shrink: 0;
  }

  .box-title {
    font-size: 0.66rem;
    color: #99a;
    letter-spacing: 0.06em;
  }

  .stack .box-title { color: #a8f; }
  .combat .box-title { color: #e86; }

  .box-count {
    font-size: 0.62rem;
    color: #667;
  }

  .box-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0.2rem 0.4rem;
    font-size: 0.68rem;
  }

  .empty {
    color: #445;
    font-size: 0.66rem;
    padding: 0.2rem 0;
  }

  .stack-item {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.3rem;
    padding: 0.15rem 0.25rem;
    border-radius: 3px;
    margin-bottom: 0.15rem;
    background: #1e1836;
  }

  .stack-item.top {
    outline: 1px solid #5a4a9a;
  }

  .pos { color: #668; font-size: 0.6rem; }
  .kind { font-size: 0.58rem; padding: 0 0.25rem; border-radius: 2px; background: #2a2a5a; color: #aac; }
  .kind-spell { background: #1a2a6a; color: #8af; }
  .kind-triggered_ability { background: #1a3a2a; color: #8ca; }
  .kind-activated_ability { background: #2a1a4a; color: #c8a; }
  .src { color: #dde; font-weight: bold; }
  .ctl { color: #667; font-size: 0.62rem; }
  .copy { color: #88c; font-size: 0.58rem; }
  .targets { width: 100%; color: #8ac; font-size: 0.62rem; padding-left: 1rem; }

  .atk {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    align-items: baseline;
    padding: 0.1rem 0;
  }

  .atk-name { color: #fca; font-weight: bold; }
  .arrow { color: #e64; }
  .tgt { color: #aac; }
  .blk { width: 100%; color: #8bc; font-size: 0.62rem; padding-left: 0.5rem; }
  .unblocked { color: #f84; font-size: 0.58rem; font-weight: bold; }
</style>
