<script>
  /**
   * CombatCard — the dock's combat box and its enlarged view
   * (`docs/play-surface-v3-brief.md` §1 rule 10): one bordered box per defender,
   * each attacker with its blockers beside it; a click opens a centred
   * lane-per-defender view.
   *
   * `CombatView` carries names and object ids only, so the card drawn for an
   * attacker or blocker is the battlefield entry with that id (which has the
   * P/T); when the seat view has no such permanent the bare name is drawn.
   *
   * The enlarged view is a `<section>`, not a `<div>`: it is `position: fixed`
   * but sits next to the box in the DOM, and the mockup's `.dock > div` rule
   * would otherwise repaint it as a dock panel.
   */
  import CardFace from './CardFace.svelte';
  import { combatGroups, permanentIndex } from './layout.js';
  import { ui } from './ui.svelte.js';

  const { combat = null, battlefield = {}, humanName = null } = $props();

  const groups = $derived(combatGroups(combat));
  const perms = $derived(permanentIndex(battlefield ?? {}));

  function cardFor(entry) {
    return perms.get(Number(entry.object_id)) ?? { name: entry.name };
  }

  function defenderLabel(d) {
    const who = d.name === humanName ? 'you' : d.name;
    return d.kind === 'planeswalker' ? `${d.name} (planeswalker)` : who;
  }

  function pairTitle(a) {
    return a.blockers?.length ? `${a.name} blocked by ${a.blockers.map((b) => b.name).join(', ')}` : `${a.name} unblocked`;
  }

  function closeFloat(event) {
    event?.stopPropagation();
    ui.combatOpen = false;
  }

  // The enlarged view cannot outlive the combat it shows.
  $effect(() => {
    if (!combat && ui.combatOpen) ui.combatOpen = false;
  });

  $effect(() => {
    function onKeyDown(event) {
      if (event.key === 'Escape') ui.combatOpen = false;
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="combat-box" onclick={() => { if (combat) ui.combatOpen = true; }}>
  <div class="panel-title">Combat</div>
  {#if !combat}
    <div class="dim">not in combat</div>
  {:else}
    <div><b>{combat.attacking_player}</b> attacks</div>
    {#if groups.length === 0}
      <div class="dim">no attackers declared</div>
    {/if}
    {#each groups as group (group.key)}
      <div class="def">
        <div class="to">→ attacking {defenderLabel(group.defender)}</div>
        <div class="grp">
          {#each group.attackers as a (a.object_id)}
            <div class="pair" class:blocked={a.blockers?.length > 0} title={pairTitle(a)}>
              <CardFace card={cardFor(a)} size="perm" />
              {#each a.blockers ?? [] as b (b.object_id)}
                <span class="vs">⛨</span>
                <span class="blocker"><CardFace card={cardFor(b)} size="perm" /></span>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {/each}
    <div class="dim">click to enlarge</div>
  {/if}
</div>

{#if ui.combatOpen && combat}
  <section class="combat-float">
    <div class="head">
      <div class="panel-title">Combat — {combat.attacking_player} attacks</div>
      <button class="mini-btn" onclick={closeFloat}>✕ close</button>
    </div>
    <div class="lanes">
      {#each groups as group (group.key)}
        <div class="lane">
          <div class="to">→ {defenderLabel(group.defender)}</div>
          {#each group.attackers as a (a.object_id)}
            <div class="pair">
              <CardFace card={cardFor(a)} size="perm" />
              <div class="arrow">⟵</div>
              {#each a.blockers ?? [] as b (b.object_id)}
                <CardFace card={cardFor(b)} size="perm" />
              {:else}
                <div class="none">unblocked</div>
              {/each}
            </div>
          {/each}
        </div>
      {:else}
        <div class="dim">no attackers declared</div>
      {/each}
    </div>
  </section>
{/if}
