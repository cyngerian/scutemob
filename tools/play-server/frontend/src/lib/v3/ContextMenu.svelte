<script>
  /**
   * ContextMenu — the two right-click menus, drawn at the table's root from
   * `ui.ctx` (see `ui.svelte.js` for why floating things are not drawn where
   * they are opened).
   *
   * The SEAT menu (mockup `ctxMenu`, `.ctx.seat-ctx`): that player's graveyard,
   * exile and commander damage, and "bring across the table" for an opponent
   * who is not already there. Exile is the seat view's single shared list;
   * per-player exile is a server change (brief §5), and the popover says so
   * rather than guessing owners.
   *
   * The CARD menu (`docs/play-surface-v3-brief.md` §2 item 2, `.ctx.card-ctx`):
   * the card's counters, then what the current decision lets you do WITH it —
   *   "Use"          every answerable action the card is the source of;
   *   "Target with"  every answerable action that lists the card as a legal
   *                  target, so "put a counter on this creature" is chosen on
   *                  the creature instead of by reading a list of every target.
   * Both lists are the decision payload's own (`actionsFrom` /
   * `actionsTargeting`); the menu decides nothing. "Use" starts the picker
   * chain exactly as a click on the card would; "Target with" hands the card
   * to `targeting.svelte.js`, which starts the same chain and then picks the
   * card in the mounted picker.
   *
   * This component also owns the one outside-click / Escape listener that
   * closes either menu and the pile popover.
   */
  import { ui, openPopover, closeFloating } from './ui.svelte.js';
  import { actionsFrom, actionsTargeting, targeting, targetWith } from './targeting.svelte.js';

  // `state` is the prop's name; a local called `state` would shadow the `$state` rune.
  const {
    state: view = null,
    humanName = null,
    focus = null,
    decision = null,
    loading = false,
    onFocus = null,
    onUse = null,
  } = $props();

  let menu = $state(null);

  // ── Seat menu ──────────────────────────────────────────────────────────────

  const name = $derived(ui.ctx?.kind === 'seat' ? ui.ctx.name : null);
  const graveyard = $derived(name ? (view?.zones?.graveyard?.[name] ?? []) : []);
  const exile = $derived(view?.zones?.exile ?? []);

  function commanderDamageNote() {
    const received = view?.players?.[name]?.commander_damage_received ?? {};
    const lines = [];
    for (const [opponent, byCommander] of Object.entries(received)) {
      for (const [commander, damage] of Object.entries(byCommander ?? {})) {
        if (damage > 0) lines.push(`${damage} from ${commander} (${opponent})`);
      }
    }
    return lines.length ? lines.join(' · ') : 'none taken';
  }

  function showGraveyard() {
    const { x, y } = ui.ctx;
    openPopover({ title: `${name}'s graveyard`, cards: graveyard, x, y });
  }

  function showExile() {
    const { x, y } = ui.ctx;
    openPopover({ title: 'Exile', note: 'shared by all players (per-player exile is a later server change)', cards: exile, x, y });
  }

  function showCommanderDamage() {
    const { x, y } = ui.ctx;
    openPopover({ title: `Commander damage dealt to ${name}`, note: commanderDamageNote(), cards: [], x, y });
  }

  function bringAcross() {
    onFocus?.(name);
    closeFloating();
  }

  // ── Card menu ──────────────────────────────────────────────────────────────

  const card = $derived(ui.ctx?.kind === 'card' ? ui.ctx.card : null);
  const sourceOf = $derived(card ? actionsFrom(decision, card.object_id) : []);
  const targetOf = $derived(card ? actionsTargeting(decision, card.object_id) : []);

  /** Counter keys are the view model's labels ("+1/+1", "charge", "loyalty", …), shown as sent. */
  const counters = $derived(
    Object.entries(card?.counters ?? {})
      .filter(([, n]) => n > 0)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([label, n]) => `${label} ×${n}`)
      .join(' · '),
  );

  /**
   * One picker chain at a time: while one is open nothing else can start. The
   * exception is aiming the OPEN target stage's own action at this card.
   */
  const busy = $derived(loading || targeting.chainOpen);
  const canTarget = (action) => !loading && (!targeting.chainOpen || targeting.live?.index === action.index);

  function use(action) {
    closeFloating();
    onUse?.(action);
  }

  function target(action) {
    const chosen = card;
    closeFloating();
    targetWith(action, chosen);
  }

  // Place at the pointer, then pull back inside the viewport once measured.
  $effect(() => {
    const ctx = ui.ctx;
    if (!ctx || !menu) return;
    menu.style.left = `${ctx.x}px`;
    menu.style.top = `${ctx.y}px`;
    const r = menu.getBoundingClientRect();
    if (r.right > window.innerWidth) menu.style.left = `${Math.max(0, window.innerWidth - r.width - 8)}px`;
    if (r.bottom > window.innerHeight) menu.style.top = `${Math.max(0, window.innerHeight - r.height - 8)}px`;
  });

  $effect(() => {
    function onMouseDown(event) {
      if (event.target?.closest?.('.ctx, .popover')) return;
      if (ui.ctx || ui.popover) closeFloating();
    }
    function onKeyDown(event) {
      if (event.key === 'Escape' && (ui.ctx || ui.popover)) closeFloating();
    }
    window.addEventListener('mousedown', onMouseDown);
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('keydown', onKeyDown);
    };
  });
</script>

{#if name !== null}
  <div class="ctx seat-ctx" bind:this={menu} role="menu">
    <div class="ctx-title">{name}</div>
    <button role="menuitem" onclick={showGraveyard}>View graveyard ({graveyard.length})</button>
    <button role="menuitem" onclick={showExile}>View exile ({exile.length})</button>
    <button role="menuitem" onclick={showCommanderDamage}>Commander damage</button>
    {#if name !== humanName && name !== focus}
      <button role="menuitem" onclick={bringAcross}>Bring across the table</button>
    {/if}
  </div>
{:else if card}
  <div class="ctx card-ctx" bind:this={menu} role="menu" data-oid={card.object_id}>
    <div class="ctx-title">{card.name}</div>
    {#if counters}<div class="ctx-counters">{counters}</div>{/if}
    {#if sourceOf.length > 0}
      <div class="ctx-section">Use</div>
      {#each sourceOf as action (action.index)}
        <button role="menuitem" data-role="source" data-action-index={action.index} disabled={busy} onclick={() => use(action)}>
          {action.label}
        </button>
      {/each}
    {/if}
    {#if targetOf.length > 0}
      <div class="ctx-section">Target with</div>
      {#each targetOf as action (action.index)}
        <button role="menuitem" data-role="target" data-action-index={action.index} disabled={!canTarget(action)} onclick={() => target(action)}>
          {action.label}
        </button>
      {/each}
    {/if}
    {#if sourceOf.length === 0 && targetOf.length === 0}
      <div class="ctx-empty">No action uses this card right now.</div>
    {/if}
  </div>
{/if}
