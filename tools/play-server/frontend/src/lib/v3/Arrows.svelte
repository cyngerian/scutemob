<script>
  /**
   * Arrows — the one SVG overlay targeting arrows are drawn on
   * (`docs/play-surface-v3-brief.md` §2 item 1).
   *
   * Fixed over the whole viewport at the table's root, above the centred
   * decision box, and transparent to the pointer — it never takes a click.
   * Two kinds of path:
   *
   *   `.arrow.live`   — follows the pointer while an arrow is being dragged;
   *   `.arrow.chosen` — one per chosen target of the open target stage, from
   *                     the source card (or the decision box when the source
   *                     is not on the table) to the target, tagged
   *                     `data-to-oid` / `data-to-seat`;
   *   `.arrow.aimed`  — C3: from the hovered (or focused) stack item to each thing
   *                     it ALREADY targets (`stackTargets.svelte.js`), red where the
   *                     two above are amber, tagged `data-from-stack` and
   *                     `data-to-oid` / `data-to-seat` / `data-to-stack`. A target
   *                     with no element on screen gets none, and a target the
   *                     server withheld is not in the list at all.
   *
   * Nothing here knows the layout: both ends are `getBoundingClientRect()` of
   * the live elements (`targeting.svelte.js` picks which copy of a card), read
   * again every animation frame while anything is drawn, so an arrow follows a
   * re-fit of the boards, a focus change or a scrolled box. A target that is
   * not drawn anywhere simply gets no arrow; the picker still shows it selected.
   * A chosen spell or ability on the stack (a `stack_object` candidate, C3) is
   * drawn to its item in the dock.
   */
  import { untrack } from 'svelte';
  import { actionElement, cardElement, panelElement, seatElement, stackElement, targeting } from './targeting.svelte.js';
  import { stackTargets, stackHover } from './stackTargets.svelte.js';

  const NONE = { live: null, chosen: [], aimed: [] };

  /** The dock's element for a stack item id; for an object id, the item whose source card that is. */
  const stackItemElement = (id) => document.querySelector(`.v3 .stack-box .stack-item[data-stack-id="${Number(id)}"]`);
  const stackItemOfSource = (oid) => document.querySelector(`.v3 .stack-box .stack-item[data-source-oid="${Number(oid)}"]`);

  /** Leaves the item from its left edge (the table is to the left of the dock). */
  const leftEdge = (el) => {
    const r = el.getBoundingClientRect();
    return [r.left + 2, r.top + r.height / 2];
  };

  function measureAimed() {
    const id = stackHover.id;
    if (id === null) return [];
    const from = stackItemElement(id);
    if (!from) return [];
    const start = leftEdge(from);
    const out = [];
    for (const ref of stackTargets.byItem.get(id) ?? []) {
      let to = null;
      let end = null;
      if (ref.kind === 'player') {
        to = seatElement(ref.name);
      } else if (ref.kind === 'object') {
        // a card on the table first; failing that, the spell on the stack it is
        to = cardElement(ref.id);
        if (!to) {
          to = stackItemOfSource(ref.id);
          if (to) end = leftEdge(to);
        }
      } else if (ref.kind === 'stack_object') {
        to = stackItemElement(ref.id);
        if (to) end = leftEdge(to);
      }
      if (!to || to === from) continue;
      out.push({
        key: ref.kind === 'player' ? `seat:${ref.name}` : `${ref.kind}:${ref.id}`,
        from: id,
        oid: ref.kind === 'object' ? ref.id : undefined,
        seat: ref.kind === 'player' ? ref.name : undefined,
        stack: ref.kind === 'stack_object' ? ref.id : undefined,
        d: curve(start, end ?? centre(to)),
      });
    }
    return out;
  }
  let paths = $state.raw(NONE);

  const centre = (el) => {
    const r = el.getBoundingClientRect();
    return [r.left + r.width / 2, r.top + r.height / 2];
  };

  /** A cubic that bows to one side, so two arrows between neighbours do not lie on top of each other. */
  function curve([x1, y1], [x2, y2]) {
    const dx = x2 - x1;
    const dy = y2 - y1;
    const length = Math.hypot(dx, dy) || 1;
    const bend = Math.min(70, length * 0.18);
    const nx = (-dy / length) * bend;
    const ny = (dx / length) * bend;
    const f = (n) => n.toFixed(1);
    return `M${f(x1)},${f(y1)} C${f(x1 + dx * 0.3 + nx)},${f(y1 + dy * 0.3 + ny)} ${f(x1 + dx * 0.7 + nx)},${f(y1 + dy * 0.7 + ny)} ${f(x2)},${f(y2)}`;
  }

  function measure() {
    const drag = targeting.drag;
    let live = null;
    if (drag) {
      const from = (drag.fromAct !== null ? actionElement(drag.fromAct) : null) ?? (drag.fromOid !== null ? cardElement(drag.fromOid) : null);
      live = curve(from ? centre(from) : [drag.sx, drag.sy], [drag.x, drag.y]);
    }
    const chosen = [];
    if (targeting.live) {
      const source = cardElement(targeting.live.objectId) ?? panelElement();
      if (source) {
        const start = centre(source);
        for (const pick of targeting.selection) {
          const to =
            pick.kind === 'player' ? seatElement(pick.name) : pick.kind === 'object' ? cardElement(pick.id) : pick.kind === 'stack_object' ? stackElement(pick.id) : null;
          if (!to || to === source) continue;
          chosen.push({
            key: pick.kind === 'player' ? `seat:${pick.name}` : pick.kind === 'stack_object' ? `stack:${pick.id}` : `oid:${pick.id}`,
            oid: pick.kind === 'object' ? pick.id : undefined,
            seat: pick.kind === 'player' ? pick.name : undefined,
            stack: pick.kind === 'stack_object' ? pick.id : undefined,
            d: curve(start, pick.kind === 'stack_object' ? leftEdge(to) : centre(to)),
          });
        }
      }
    }
    return { live, chosen, aimed: measureAimed() };
  }

  $effect(() => {
    const active = targeting.drag !== null || (targeting.live !== null && targeting.selection.length > 0) || stackHover.id !== null;
    if (!active) {
      paths = NONE;
      return;
    }
    let frame = 0;
    let last = '';
    const step = () => {
      const next = measure();
      const key = JSON.stringify(next);
      if (key !== last) {
        last = key;
        paths = next;
      }
      frame = requestAnimationFrame(step);
    };
    // The loop reads the pointer position itself; the effect only starts and stops it.
    untrack(step);
    return () => cancelAnimationFrame(frame);
  });
</script>

<svg class="v3-arrows" aria-hidden="true">
  <defs>
    <marker id="v3-arrow-head" markerWidth="9" markerHeight="9" refX="7" refY="4.5" orient="auto" markerUnits="userSpaceOnUse">
      <path d="M0,0 L9,4.5 L0,9 z" />
    </marker>
    <marker id="v3-arrow-head-aimed" markerWidth="9" markerHeight="9" refX="7" refY="4.5" orient="auto" markerUnits="userSpaceOnUse">
      <path d="M0,0 L9,4.5 L0,9 z" />
    </marker>
  </defs>
  {#each paths.aimed as arrow, i (i + ':' + arrow.key)}
    <path
      class="arrow aimed"
      d={arrow.d}
      data-from-stack={arrow.from}
      data-to-oid={arrow.oid}
      data-to-seat={arrow.seat}
      data-to-stack={arrow.stack}
      marker-end="url(#v3-arrow-head-aimed)"
    />
  {/each}
  {#each paths.chosen as arrow (arrow.key)}
    <path class="arrow chosen" d={arrow.d} data-to-oid={arrow.oid} data-to-seat={arrow.seat} data-to-stack={arrow.stack} marker-end="url(#v3-arrow-head)" />
  {/each}
  {#if paths.live}
    <path class="arrow live" d={paths.live} marker-end="url(#v3-arrow-head)" />
  {/if}
</svg>
