<script>
  /**
   * ContextMenu — the seat right-click menu (mockup `ctxMenu`): that player's
   * graveyard, exile and commander damage, and "bring across the table" for an
   * opponent who is not already there.
   *
   * Drawn at the table's root from `ui.ctx` (see `ui.svelte.js` for why floating
   * things are not drawn where they are opened). It also owns the one
   * outside-click / Escape listener that closes both this menu and the pile
   * popover.
   *
   * Exile is the seat view's single shared list; per-player exile is a server
   * change (brief §5), and the popover says so rather than guessing owners.
   */
  import { ui, openPopover, closeFloating } from './ui.svelte.js';

  // `state` is the prop's name; a local called `state` would shadow the `$state` rune.
  const { state: view = null, humanName = null, focus = null, onFocus = null } = $props();

  let menu = $state(null);

  const name = $derived(ui.ctx?.name ?? null);
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

{#if ui.ctx}
  <div class="ctx" bind:this={menu} role="menu">
    <div class="ctx-title">{name}</div>
    <button role="menuitem" onclick={showGraveyard}>View graveyard ({graveyard.length})</button>
    <button role="menuitem" onclick={showExile}>View exile ({exile.length})</button>
    <button role="menuitem" onclick={showCommanderDamage}>Commander damage</button>
    {#if name !== humanName && name !== focus}
      <button role="menuitem" onclick={bringAcross}>Bring across the table</button>
    {/if}
  </div>
{/if}
