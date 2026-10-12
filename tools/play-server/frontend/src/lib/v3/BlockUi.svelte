<script>
  /**
   * BlockUi — declare blockers by drawing arrows (`docs/play-surface-v3-brief.md`
   * §1 rule 11; the mockup's `blockUi`, `mockups/07-table6.html`): the attackers
   * on top, your creatures that can block below, an arrow from each blocker to
   * the attacker it blocks, and a sentence saying the same thing in words.
   *
   * # One submission path: this box drives `BlockerPicker`, it never answers
   *
   * The declaration is submitted by `lib/BlockerPicker.svelte`, which
   * `PromptPanel` mounts as the `block` stage of its picker chain. This
   * component is a second front end to that picker and holds NO assignment of
   * its own:
   *   - assigning writes the `<select class="block-target">` of the blocker's
   *     row and dispatches a bubbling `change`, exactly what a user picking
   *     from the dropdown does; the picker's own handler does the rest;
   *   - what is drawn is READ BACK from the selects — after our own dispatch
   *     and after any `change` in the host, so the dropdowns stay usable and
   *     the arrows and the list cannot disagree;
   *   - the picker's "Block with N" / "Block with nothing" button stays the
   *     only confirm, and its list stays on screen as the fallback.
   * Nothing here builds an action parameter or calls `act`.
   *
   * The picker is found through the DOM (`closest('.decision-host')` from our
   * own anchor, watched with a `MutationObserver`), because `DecisionPanel`
   * hosts `PromptPanel` as a closed box. Before anything is drawn or driven the
   * picker is checked against the decision's own `block.legal_blocks`: one
   * `.blocker-row` per entry and in the same order, the row's label equal to
   * the blocker's label, the options equal to that blocker's legal attackers.
   * If any of that is off the box draws nothing and the list does the job —
   * a wrong arrow would be worse than no arrow.
   *
   * # Known limitation, inherited: CR 509.1b
   *
   * `BlockerPicker` holds one attacker per blocker, so a creature that "can
   * block an additional creature" can be given only one attacker here too (the
   * server would accept more; see the picker's header and the crate README's
   * Known limitations). Assigning an already-assigned blocker moves its arrow.
   *
   * # A blank dropdown is the picker's, and is repaired here
   *
   * `BlockerPicker` renders `<option value={attacker.id}>` (a number) under
   * `<select value={String(id)}>`, and Svelte matches the two strictly: as soon
   * as an attacker is chosen — by us or straight from the dropdown, on every
   * surface — the picker's own re-render finds no matching option and leaves
   * the select with nothing selected (`value === ''`), although its assignment
   * is right ("Block with 1"). The picker is not ours to edit, so `held`
   * remembers the value each select carried when its `change` was dispatched
   * (the very value the picker's handler read); a select found blank is put
   * back on that option (a DOM write, no event), and the total is checked
   * against the count in the picker's confirm label. If the two disagree the
   * box draws nothing. With the picker fixed none of this ever fires.
   *
   * Legality is the server's: an attacker a selected blocker is not offered is
   * dimmed and inert. The view does not say WHY a pair is missing (flying,
   * protection, …), so the hint says only that the pair is not legal.
   *
   * Props: decision (DecisionView|null), cardIndex (Map<number, card>), loading (bool).
   * Mounted by `DecisionPanel` right after `PromptPanel`, inside `.decision-host`.
   */
  import { tick } from 'svelte';
  import CardFace from './CardFace.svelte';

  const { decision = null, cardIndex = new Map(), loading = false } = $props();

  /** `BlockerPicker`'s sentinel for "doesn't block" (its `NO_BLOCK`). */
  const NO_BLOCK = '__none__';
  /** A press that moves less than this is a click, not a drag. */
  const DRAG_SLOP = 6;
  /** The mockup's curve: how far each control point stands off its card. */
  const BEND = 30;

  const action = $derived(decision?.actions?.find((a) => a.block) ?? null);
  const legalBlocks = $derived(action?.block?.legal_blocks ?? []);
  const attackers = $derived(action?.block?.attackers ?? []);

  /** Always rendered and never shown: the element `closest()` starts from. */
  let anchor = $state(null);
  let box = $state(null);

  /** `blockerId -> its <select>` once the mounted picker has been checked against `legalBlocks`; else null. */
  let rows = $state.raw(null);
  /** `blockerId -> attackerId`, read back from the selects. Never written from a click. */
  let assign = $state.raw(new Map());
  /** The blocker waiting for its attacker (a click, or the one being dragged). */
  let sel = $state(null);
  /** Why the last click did nothing, for the hint. */
  let note = $state('');
  /** `{ id, x0, y0, x, y, live }` while a pointer is down on one of your creatures. */
  let drag = $state.raw(null);
  /** Drawn arrows, in `.blockui` coordinates: `[{ blocker, attacker, d }]`. */
  let paths = $state.raw([]);
  let livePath = $state('');

  const shown = $derived(rows !== null && legalBlocks.length > 0 && attackers.length > 0);

  const mine = $derived(
    legalBlocks.map((entry) => {
      const id = Number(entry.blocker.id);
      const card = cardIndex?.get(id) ?? null;
      return {
        id,
        card,
        name: card?.name ?? entry.blocker.label,
        label: entry.blocker.label,
        legal: new Set(entry.attackers.map((a) => Number(a.id))),
      };
    }),
  );
  const theirs = $derived(
    attackers.map((a) => {
      const id = Number(a.id);
      const card = cardIndex?.get(id) ?? null;
      return { id, card, name: card?.name ?? a.label, label: a.label };
    }),
  );
  const mineById = $derived(new Map(mine.map((m) => [m.id, m])));
  const theirsById = $derived(new Map(theirs.map((t) => [t.id, t])));
  const selected = $derived(sel === null ? null : (mineById.get(sel) ?? null));

  const ptOf = (card) => (card && card.power != null && card.toughness != null ? `${card.power}/${card.toughness}` : '');

  const hint = $derived.by(() => {
    const words = [...assign].map(([b, a]) => `${mineById.get(b)?.name ?? b} blocks ${theirsById.get(a)?.name ?? a}`);
    const parts = [];
    if (note) parts.push(note);
    if (words.length) parts.push(`${words.join(' · ')}. Click a blocker again to undo.`);
    if (selected) parts.push(`${selected.name} selected — now click the attacker it blocks.`);
    if (!words.length && !selected) parts.push('No blocks yet.');
    return parts.join(' ');
  });

  // ── The picker: find it, check it, read it ──────────────────────────────────

  /** The mounted picker's selects by blocker id, or null if it is not exactly what `legalBlocks` describes. */
  function checkPicker(host) {
    const picker = host?.querySelector('.blocker-picker');
    if (!picker || legalBlocks.length === 0) return null;
    const rowEls = [...picker.querySelectorAll('.blocker-row')];
    if (rowEls.length !== legalBlocks.length) return null;
    const found = new Map();
    for (let i = 0; i < rowEls.length; i += 1) {
      const entry = legalBlocks[i];
      const select = rowEls[i].querySelector('select.block-target');
      if (!select) return null;
      if (rowEls[i].querySelector('.blocker-label')?.textContent !== entry.blocker.label) return null;
      const want = [NO_BLOCK, ...entry.attackers.map((a) => String(a.id))];
      const have = [...select.options].map((o) => o.value);
      if (have.length !== want.length || have.some((v, k) => v !== want[k])) return null;
      found.set(Number(entry.blocker.id), select);
    }
    return found.size === legalBlocks.length ? found : null;
  }

  const sameSelects = (a, b) => a.size === b.size && [...a].every(([k, v]) => b.get(k) === v);

  /** select element → the value it carried at its last `change`: what the picker's handler read. */
  const held = new WeakMap();

  function onHostChange(e) {
    const select = e.target;
    if (select instanceof HTMLSelectElement && select.matches('.blocker-picker select.block-target') && !select.disabled) {
      held.set(select, select.value);
    }
    scheduleSync();
  }

  /** The number of blocks the picker says it holds (its confirm label), or null if the label does not say. */
  function pickerCount(host) {
    const text = host?.querySelector('.blocker-picker .picker-actions .confirm')?.textContent?.trim() ?? '';
    if (text === 'Block with nothing') return 0;
    const m = /^Block with (\d+)$/.exec(text);
    return m ? Number(m[1]) : null;
  }

  /** Re-check the picker and re-read every select. State is replaced only when it differs, so our own redraw settles. */
  function sync() {
    const host = anchor?.closest('.decision-host');
    const found = checkPicker(host);
    const read = new Map();
    let agreed = found !== null;
    if (found !== null) {
      for (const [id, select] of found) {
        let value = select.value;
        if (select.selectedIndex < 0 && held.has(select)) {
          // The picker blanked it (header: "A blank dropdown"); show what it holds.
          value = held.get(select);
          select.value = value;
        }
        if (select.selectedIndex < 0) agreed = false;
        else if (value !== NO_BLOCK) read.set(id, Number(value));
      }
      const count = pickerCount(host);
      if (count === null || count !== read.size) agreed = false;
    }
    if (!agreed) {
      if (rows !== null) rows = null;
      if (assign.size) assign = new Map();
      sel = null;
      note = '';
      cancelDrag();
      return;
    }
    if (rows === null || !sameSelects(rows, found)) rows = found;
    if (!sameSelects(assign, read)) assign = read;
    if (sel !== null && !found.has(sel)) sel = null;
  }

  let syncQueued = false;
  /** After the picker's own DOM update, and once more on the next frame (a select's value is set late on mount). */
  function scheduleSync() {
    if (syncQueued) return;
    syncQueued = true;
    tick().then(() => {
      sync();
      requestAnimationFrame(() => {
        syncQueued = false;
        sync();
        draw();
      });
    });
  }

  /** Write one row of the picker the way a user would, then read everything back. */
  function drive(blockerId, value) {
    if (loading) return;
    sync();
    const select = rows?.get(blockerId);
    if (!select || select.disabled) return;
    const text = String(value);
    if (![...select.options].some((o) => o.value === text)) return;
    select.value = text;
    // `onHostChange` hears this on its way up, like any change made in the dropdown itself.
    select.dispatchEvent(new Event('change', { bubbles: true }));
  }

  $effect(() => {
    const host = anchor?.closest('.decision-host');
    if (!host) return;
    // Our own box lives in the host too; its redraws are not news.
    const mo = new MutationObserver((list) => {
      if (list.some((m) => !box || !box.contains(m.target))) scheduleSync();
    });
    mo.observe(host, { childList: true, subtree: true });
    host.addEventListener('change', onHostChange);
    host.addEventListener('scroll', scheduleDraw, { passive: true });
    window.addEventListener('resize', scheduleDraw);
    scheduleSync();
    return () => {
      mo.disconnect();
      host.removeEventListener('change', onHostChange);
      host.removeEventListener('scroll', scheduleDraw);
      window.removeEventListener('resize', scheduleDraw);
      cancelDrag();
    };
  });

  // A new decision under a picker that stays mounted is checked again.
  $effect(() => {
    void legalBlocks;
    void attackers;
    scheduleSync();
  });

  // ── Arrows ─────────────────────────────────────────────────────────────────

  /** The middle of a card's top or bottom edge, in `.blockui` coordinates. */
  function edge(el, side, r0) {
    const r = (el?.querySelector('.card, .name-chip') ?? el)?.getBoundingClientRect();
    if (!r) return null;
    return [r.left + r.width / 2 - r0.left, (side === 'top' ? r.top : r.bottom) - r0.top];
  }

  const curve = ([x1, y1], [x2, y2]) => `M${x1},${y1} C${x1},${y1 - BEND} ${x2},${y2 + BEND} ${x2},${y2}`;

  function draw() {
    if (!box) {
      if (paths.length) paths = [];
      livePath = '';
      return;
    }
    const r0 = box.getBoundingClientRect();
    const next = [];
    for (const [b, a] of assign) {
      const from = edge(box.querySelector(`.act.mine[data-blocker="${b}"]`), 'top', r0);
      const to = edge(box.querySelector(`.act.attacker[data-attacker="${a}"]`), 'bottom', r0);
      if (from && to) next.push({ blocker: b, attacker: a, d: curve(from, to) });
    }
    if (next.length !== paths.length || next.some((p, i) => p.blocker !== paths[i].blocker || p.attacker !== paths[i].attacker || p.d !== paths[i].d)) {
      paths = next;
    }
    let live = '';
    if (drag?.live) {
      const from = edge(box.querySelector(`.act.mine[data-blocker="${drag.id}"]`), 'top', r0);
      if (from) live = curve(from, [drag.x - r0.left, drag.y - r0.top]);
    }
    livePath = live;
  }

  let drawQueued = false;
  function scheduleDraw() {
    if (drawQueued) return;
    drawQueued = true;
    requestAnimationFrame(() => {
      drawQueued = false;
      draw();
    });
  }

  // Re-measure once the DOM reflects a change of assignment, selection or drag …
  $effect(() => {
    void assign;
    void sel;
    void drag;
    void shown;
    scheduleDraw();
  });
  // … and whenever the box itself changes size (the float resizing, a row wrapping).
  $effect(() => {
    if (!box) return;
    const ro = new ResizeObserver(scheduleDraw);
    ro.observe(box);
    return () => ro.disconnect();
  });

  // ── Click and drag ─────────────────────────────────────────────────────────

  /** A drag that ended on the card it began on is followed by a `click` we must not also act on. */
  let swallowClick = false;

  function attempt(blocker, attackerId) {
    const target = theirsById.get(attackerId);
    if (!blocker || !target) return;
    if (!blocker.legal.has(attackerId)) {
      note = `${blocker.name} cannot legally block ${target.name}.`;
      return;
    }
    note = '';
    sel = null;
    drive(blocker.id, attackerId);
  }

  function clickMine(m) {
    if (loading) return;
    if (swallowClick) {
      swallowClick = false;
      return;
    }
    note = '';
    if (assign.has(m.id)) {
      sel = null;
      drive(m.id, NO_BLOCK);
    } else {
      sel = sel === m.id ? null : m.id;
    }
  }

  function clickAttacker(t) {
    if (loading || !selected) return;
    attempt(selected, t.id);
  }

  function onMove(e) {
    if (!drag) return;
    const live = drag.live || Math.hypot(e.clientX - drag.x0, e.clientY - drag.y0) >= DRAG_SLOP;
    if (live && !drag.live) {
      // The dragged blocker is the selected one, so the attackers it cannot block dim.
      sel = drag.id;
      note = '';
    }
    drag = { ...drag, x: e.clientX, y: e.clientY, live };
  }

  function onUp(e) {
    const d = drag;
    cancelDrag();
    if (!d?.live) return; // a click: `clickMine` handles it
    swallowClick = true;
    setTimeout(() => (swallowClick = false), 0);
    const over = document.elementFromPoint(e.clientX, e.clientY)?.closest('.act.attacker');
    const blocker = mineById.get(d.id);
    sel = null;
    if (over && box?.contains(over) && !loading) attempt(blocker, Number(over.dataset.attacker));
  }

  function cancelDrag() {
    window.removeEventListener('pointermove', onMove);
    window.removeEventListener('pointerup', onUp);
    window.removeEventListener('pointercancel', cancelDrag);
    if (drag) drag = null;
  }

  function downMine(e, m) {
    if (loading || e.button !== 0) return;
    cancelDrag();
    drag = { id: m.id, x0: e.clientX, y0: e.clientY, x: e.clientX, y: e.clientY, live: false };
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
    window.addEventListener('pointercancel', cancelDrag);
  }
</script>

<span class="blockui-anchor" hidden bind:this={anchor}></span>
{#if shown}
  <div class="blockui" class:inert={loading} class:dragging={drag?.live} bind:this={box}>
    <div class="brow">
      <div class="lbl">attacking you</div>
      {#each theirs as t (t.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="act attacker"
          class:illegal={selected !== null && !selected.legal.has(t.id)}
          class:open={selected !== null && selected.legal.has(t.id)}
          data-attacker={t.id}
          onclick={() => clickAttacker(t)}
        >
          {#if t.card}
            <CardFace card={t.card} size="perm" />
            <div class="cap">{ptOf(t.card)}</div>
          {:else}
            <div class="name-chip">{t.label}</div>
          {/if}
        </div>
      {/each}
    </div>
    <div class="brow">
      <div class="lbl">your creatures that can block — click one, then the attacker it blocks</div>
      {#each mine as m (m.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="act mine"
          class:sel={sel === m.id}
          class:assigned={assign.has(m.id)}
          data-blocker={m.id}
          onpointerdown={(e) => downMine(e, m)}
          onclick={() => clickMine(m)}
        >
          {#if m.card}
            <CardFace card={m.card} size="perm" />
            <div class="cap">{ptOf(m.card)}</div>
          {:else}
            <div class="name-chip">{m.label}</div>
          {/if}
        </div>
      {/each}
    </div>
    <svg class="arrows" aria-hidden="true">
      <defs>
        <marker id="v3-block-ah" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto">
          <path d="M0,0 L8,4 L0,8 z" fill="#9bd" />
        </marker>
      </defs>
      {#each paths as p (p.blocker)}
        <path d={p.d} data-blocker={p.blocker} data-attacker={p.attacker} stroke="#9bd" stroke-width="2.5" fill="none" marker-end="url(#v3-block-ah)" />
      {/each}
      {#if livePath}
        <path class="live" d={livePath} stroke="#9bd" stroke-width="2.5" fill="none" marker-end="url(#v3-block-ah)" />
      {/if}
    </svg>
    <div class="hint">{hint}</div>
  </div>
{/if}
