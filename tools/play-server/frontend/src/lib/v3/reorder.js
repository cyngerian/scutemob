/**
 * reorder — the pointer drag that rearranges a row of your own cards
 * (`docs/play-surface-v3-brief.md` §2 item 3). The order itself is `order.js`.
 *
 * A Svelte action for a row's `.cards` container: `use:reorderable={{ onMove, onDragOut }}`.
 * Its direct `.card[data-oid]` children are the things that move, each within its own row.
 *
 * Pointer events, not HTML5 drag-and-drop: the card image is `draggable="false"`, and a
 * native drag would take the pointer away from the hover overlay and the targeting arrow.
 *
 *   - pointerdown on a card arms; nothing visible happens until the pointer has moved
 *     more than `SLOP` px. Short of that it is a click and this module stays out of it —
 *     the card's own `onclick` runs exactly as before.
 *   - past `SLOP` the card carries `.reorder-dragging` and a `.reorder-marker` (a thin bar)
 *     shows where it would land. Rows wrap, so the slot is found in two dimensions: the
 *     nearest line of cards first, then the nearest gap on that line.
 *   - pointerup → `onMove(movedOid, beforeOid | null)` (null = the end of the row), unless
 *     the card would land where it already is.
 *   - the pointer leaving the row → `onDragOut(oid, pointerEvent)`, ONCE, and the drag ends
 *     here with nothing moved: the targeting arrow (`targeting.svelte.js`) takes the
 *     pointer from there, or declines and nothing happens.
 *   - Escape cancels.
 *   - after any real drag the trailing `click` is swallowed, so a drop never also opens
 *     the card's action.
 *
 * The marker is `position: fixed` at the `.v3` root (where `hover.js` puts its overlay),
 * so it can never shift a row or give a board a scrollbar — the measured fit (`fit.js`)
 * reads the boards' scroll sizes.
 *
 * Params: `{ onMove, onDragOut, disabled }`.
 */
import { hideHover } from './hover.js';

/** Under this many px a press is a click, not a drag. */
const SLOP = 6;
/** How far outside the row the pointer may stray before the drag is handed off. */
const LEAVE = 10;
const MARKER_W = 3;

const cardsOf = (node) => [...node.children].filter((el) => el.matches('.card[data-oid]'));

/**
 * The cards of a wrapped row as lines, top to bottom. A card starts a new line when it
 * sits to the left of the one before it (a tapped card's tilt changes its top, never
 * the left-to-right run).
 */
function linesOf(cards) {
  const lines = [];
  let prev = null;
  for (const el of cards) {
    const rect = el.getBoundingClientRect();
    if (prev === null || rect.left < prev.left - 1) lines.push({ top: rect.top, bottom: rect.bottom, cells: [] });
    const line = lines[lines.length - 1];
    line.top = Math.min(line.top, rect.top);
    line.bottom = Math.max(line.bottom, rect.bottom);
    line.cells.push({ el, rect });
    prev = rect;
  }
  return lines;
}

/**
 * The insertion slot nearest (x, y): `{ before, x, top, height }` — the card the dragged
 * one would go in front of (null = the end of the row) and where to draw the marker.
 */
export function nearestSlot(cards, x, y) {
  const lines = linesOf(cards);
  if (lines.length === 0) return null;
  const off = (line) => (y < line.top ? line.top - y : y > line.bottom ? y - line.bottom : 0);
  let li = 0;
  for (let i = 1; i < lines.length; i++) if (off(lines[i]) < off(lines[li])) li = i;
  const line = lines[li];
  const cells = line.cells;
  // half the real gap between two cards of this row, so the bar sits in the middle of it
  const half = cells.length > 1 ? Math.max(0, cells[1].rect.left - cells[0].rect.right) / 2 : 2;
  // one slot at each card's left edge, and one after the line's last card
  let best = { before: cells[0].el, x: cells[0].rect.left - half, d: Infinity };
  for (const { el, rect } of cells) {
    const d = Math.abs(x - rect.left);
    if (d < best.d) best = { before: el, x: rect.left - half, d };
  }
  const last = cells[cells.length - 1];
  if (Math.abs(x - last.rect.right) < best.d) {
    const next = lines[li + 1]?.cells[0].el ?? null; // the end of a line is the front of the next one
    best = { before: next, x: last.rect.right + half, d: 0 };
  }
  return { before: best.before, x: best.x, top: line.top, height: line.bottom - line.top };
}

export function reorderable(node, params) {
  let opts = params ?? {};
  /** `{ el, pointerId, x0, y0, live, marker, slot }` between pointerdown and the end of the gesture. */
  let drag = null;

  function clearVisuals() {
    if (!drag) return;
    drag.el.classList.remove('reorder-dragging');
    node.classList.remove('reorder-active');
    drag.marker?.remove();
    drag.marker = null;
    try {
      if (drag.captured) drag.el.releasePointerCapture(drag.pointerId);
    } catch {
      // the capture is already gone (the element left the DOM, the pointer was cancelled)
    }
    drag.captured = false;
  }

  function stopListening() {
    window.removeEventListener('pointermove', onMove, true);
    window.removeEventListener('pointerup', onUp, true);
    window.removeEventListener('pointercancel', onCancel, true);
    window.removeEventListener('keydown', onKey, true);
  }

  /** The `click` that follows a drag's pointerup would open the card's action: eat it. */
  function swallowClick() {
    const eat = (event) => {
      event.stopPropagation();
      event.preventDefault();
    };
    window.addEventListener('click', eat, { capture: true, once: true });
    // the click, if there is one, is dispatched with the pointerup; a timer is always after it
    setTimeout(() => window.removeEventListener('click', eat, true), 0);
  }

  function begin(event) {
    drag.live = true;
    hideHover(); // the 240px overlay would sit on the row being rearranged
    getSelection()?.removeAllRanges();
    try {
      drag.el.setPointerCapture(drag.pointerId); // keeps hover off the cards the pointer crosses
      drag.captured = true;
    } catch {
      drag.captured = false;
    }
    drag.el.classList.add('reorder-dragging');
    node.classList.add('reorder-active');
    const marker = document.createElement('div');
    marker.className = 'reorder-marker';
    (node.closest('.v3') ?? document.body).append(marker);
    drag.marker = marker;
    place(event);
  }

  function place(event) {
    const slot = nearestSlot(cardsOf(node), event.clientX, event.clientY);
    drag.slot = slot;
    if (!slot || !drag.marker) return;
    drag.marker.style.left = `${slot.x - MARKER_W / 2}px`;
    drag.marker.style.top = `${slot.top}px`;
    drag.marker.style.height = `${slot.height}px`;
  }

  function outside(event) {
    const r = node.getBoundingClientRect();
    return (
      event.clientX < r.left - LEAVE || event.clientX > r.right + LEAVE || event.clientY < r.top - LEAVE || event.clientY > r.bottom + LEAVE
    );
  }

  function onMove(event) {
    if (!drag || event.pointerId !== drag.pointerId) return;
    if (drag.dead) return; // cancelled; only waiting for the button to come up
    if (!drag.live) {
      if (Math.hypot(event.clientX - drag.x0, event.clientY - drag.y0) <= SLOP) return;
      begin(event);
    }
    if (!drag.el.isConnected) {
      // the card left the table under the pointer (a bot's action resolved): nothing to move
      clearVisuals();
      drag.dead = true;
      return;
    }
    if (outside(event)) {
      // Out of the row: this is not a rearrangement. Hand the pointer to the targeting
      // arrow — once — and end here without moving anything.
      const oid = Number(drag.el.dataset.oid);
      clearVisuals();
      drag.dead = true;
      opts.onDragOut?.(oid, event);
      return;
    }
    place(event);
  }

  function onUp(event) {
    if (!drag || event.pointerId !== drag.pointerId) return;
    const done = drag;
    const slot = done.live && !done.dead ? done.slot : null;
    clearVisuals();
    stopListening();
    drag = null;
    if (!done.live) return; // a click: the card's own handler has it
    swallowClick();
    if (!slot || !done.el.isConnected) return;
    const cards = cardsOf(node);
    const next = cards[cards.indexOf(done.el) + 1] ?? null;
    if (slot.before === done.el || slot.before === next) return; // dropped where it already is
    const before = slot.before?.isConnected ? Number(slot.before.dataset.oid) : null;
    opts.onMove?.(Number(done.el.dataset.oid), before);
  }

  function onCancel(event) {
    if (!drag || event.pointerId !== drag.pointerId) return;
    clearVisuals();
    stopListening();
    drag = null;
  }

  function onKey(event) {
    if (event.key !== 'Escape' || !drag?.live || drag.dead) return;
    // Escape is this drag's alone while it is live (it also closes popovers and menus)
    event.stopPropagation();
    clearVisuals();
    drag.dead = true;
  }

  function onDown(event) {
    if (opts.disabled || drag || event.button !== 0 || event.isPrimary === false) return;
    const el = event.target instanceof Element ? event.target.closest('.card[data-oid]') : null;
    if (!el || el.parentElement !== node) return;
    drag = { el, pointerId: event.pointerId, x0: event.clientX, y0: event.clientY, live: false, dead: false, captured: false, marker: null, slot: null };
    window.addEventListener('pointermove', onMove, true);
    window.addEventListener('pointerup', onUp, true);
    window.addEventListener('pointercancel', onCancel, true);
    window.addEventListener('keydown', onKey, true);
  }

  node.addEventListener('pointerdown', onDown);
  return {
    update(next) {
      opts = next ?? {};
    },
    destroy() {
      node.removeEventListener('pointerdown', onDown);
      clearVisuals();
      stopListening();
      drag = null;
    },
  };
}
