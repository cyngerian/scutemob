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
   *                     `data-to-oid` / `data-to-seat`.
   *
   * Nothing here knows the layout: both ends are `getBoundingClientRect()` of
   * the live elements (`targeting.svelte.js` picks which copy of a card), read
   * again every animation frame while anything is drawn, so an arrow follows a
   * re-fit of the boards, a focus change or a scrolled box. A target that is
   * not drawn anywhere (a spell on the stack) simply gets no arrow; the picker
   * still shows it selected.
   */
  import { untrack } from 'svelte';
  import { actionElement, cardElement, panelElement, seatElement, targeting } from './targeting.svelte.js';

  const NONE = { live: null, chosen: [] };
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
          const to = pick.kind === 'player' ? seatElement(pick.name) : pick.kind === 'object' ? cardElement(pick.id) : null;
          if (!to || to === source) continue;
          chosen.push({
            key: pick.kind === 'player' ? `seat:${pick.name}` : `oid:${pick.id}`,
            oid: pick.kind === 'object' ? pick.id : undefined,
            seat: pick.kind === 'player' ? pick.name : undefined,
            d: curve(start, centre(to)),
          });
        }
      }
    }
    return { live, chosen };
  }

  $effect(() => {
    const active = targeting.drag !== null || (targeting.live !== null && targeting.selection.length > 0);
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
  </defs>
  {#each paths.chosen as arrow (arrow.key)}
    <path class="arrow chosen" d={arrow.d} data-to-oid={arrow.oid} data-to-seat={arrow.seat} marker-end="url(#v3-arrow-head)" />
  {/each}
  {#if paths.live}
    <path class="arrow live" d={paths.live} marker-end="url(#v3-arrow-head)" />
  {/if}
</svg>
