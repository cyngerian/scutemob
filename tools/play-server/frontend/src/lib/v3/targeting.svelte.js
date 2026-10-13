/**
 * targeting — arrow targeting and "target with" for the v3 table
 * (`docs/play-surface-v3-brief.md` §2 items 1 and 2).
 *
 * Three gestures choose a target ON the table instead of in the decision box's
 * list: dragging an arrow from an action (a decision-panel card item) or from
 * one of your own cards onto a card or a seat, clicking a highlighted card or
 * seat while a target stage is open, and a card context menu's "Target with".
 *
 * None of them submits anything. Each ends in the same two steps:
 *
 *   1. `table.begin(action)` — `PromptPanel.beginExternal`, the picker chain
 *      every other click on this surface starts — unless the chain is already
 *      at that action's target stage;
 *   2. once `TargetPicker` is mounted, `pickerDriver` clicks ITS button for the
 *      chosen candidate and, when the announcement is then complete, ITS
 *      Confirm.
 *
 * So the picker always shows what an arrow chose, stays the fallback list, and
 * is the one place a `targets` parameter is ever built. Between the two steps
 * the choice waits here as the *pending pick* (an X, modes or cost stage may
 * come first).
 *
 * This module is the controller: plain functions and listeners, with the few
 * bits the view needs (`targeting`) as `$state`. `TableV3` binds it to the
 * table with `bindTable`; `Arrows.svelte` draws from `targeting`; `CardFace`
 * reads the highlight sets. Legality is never decided here — a card is a legal
 * target because the decision payload lists it as a candidate, and for no
 * other reason.
 */
import { tick } from 'svelte';
import { answerable } from '../v2/prompt.js';
import { closeFloating } from './ui.svelte.js';
import * as driver from './pickerDriver.js';

/** A press on an action becomes a drag after this many pixels; under it, it is a click. */
const DRAG_THRESHOLD = 6;

const EMPTY = new Set();

/**
 * What the view draws from.
 *   drag        — `{ x, y, sx, sy, fromAct, fromOid }` while an arrow follows the pointer
 *   live        — `{ index, objectId, label }` while a target stage this module can drive is open
 *   unavailable — a target picker is open but could not be matched to the payload (list only)
 *   chainOpen   — any picker stage is open
 *   selection   — the open stage's chosen candidates: `{ kind: 'object', id }` | `{ kind: 'player', name }`
 *   okObjects / okSeats — legal targets of the drag, else of the open stage (object ids / seat names)
 *   chosenObjects       — the object ids in `selection`
 */
export const targeting = $state({
  drag: null,
  live: null,
  unavailable: false,
  chainOpen: false,
  selection: [],
  okObjects: EMPTY,
  okSeats: EMPTY,
  chosenObjects: EMPTY,
});

// ── Private controller state (nothing draws from it) ─────────────────────────

/** `bindTable`'s argument, or null while no table is mounted. */
let table = null;
/** `pickerDriver.resolvePicker`'s reading of the mounted target picker, or null. */
let resolved = null;
/** A press that may become a drag: `{ spec, x, y, pointerId }`. */
let armed = null;
/** The live drag's `{ actions, fromAct, fromOid, live }`. */
let dragSpec = null;
/** `{ indexes, labels, pick, seq, at }` — a target chosen before its picker is mounted. */
let pending = null;
/** The pick whose Confirm should be clicked once the picker shows it selected. */
let autoConfirm = null;
/** The click that ends a drag is not a click on whatever is under it. */
let swallowClickUntil = 0;

const host = () => table?.root?.querySelector('.decision-host') ?? null;
const actionsNow = () => table?.decision()?.actions ?? [];
const sameSet = (a, b) => a.size === b.size && [...a].every((v) => b.has(v));

function say(message) {
  table?.say?.(message);
}

/** Can this action be started from an arrow at all? */
function usable(action) {
  return action.kind !== 'PassPriority' && action.kind !== 'Concede' && answerable(action);
}

function takesTargets(action) {
  const c = driver.candidatesOf(action);
  return c.objects.size + c.players.size > 0;
}

function hasCandidate(action, pick) {
  const c = driver.candidatesOf(action);
  return pick.kind === 'object' ? c.objects.has(Number(pick.id)) : c.players.has(pick.name);
}

// ── What the context menu lists ──────────────────────────────────────────────

/**
 * The answerable actions a card is the SOURCE of: those carrying its
 * `object_id` (casts, activations, mana), plus a declare action that lists it
 * as an eligible attacker or blocker (which carries no `object_id` of its own).
 */
export function actionsFrom(decision, objectId) {
  const id = Number(objectId);
  return (decision?.actions ?? []).filter((a) => {
    if (!usable(a)) return false;
    if (a.object_id !== null && a.object_id !== undefined && Number(a.object_id) === id) return true;
    return (
      (a.attack?.eligible ?? []).some((e) => Number(e.id) === id) ||
      (a.block?.eligible ?? []).some((e) => Number(e.id) === id)
    );
  });
}

/** The answerable actions that list a card as a legal target in any of their slot lists. */
export function actionsTargeting(decision, objectId) {
  const id = Number(objectId);
  return (decision?.actions ?? []).filter((a) => usable(a) && driver.candidatesOf(a).objects.has(id));
}

// ── Where things are on screen (shared with `Arrows.svelte`) ─────────────────

/** A copy inside the decision box, a popover or a menu is not "on the table". */
const OFF_TABLE = '.decision-host, .popover, .ctx';

/**
 * The same object or seat can be drawn more than once. The peeked copy is on
 * top of everything, then the board across, then your own rows, then the side
 * cell (brief: `.focused`, then `.board`, then `.cell`).
 */
function rank(el) {
  if (el.closest('.cell-pop')) return 0;
  if (el.closest('.focused')) return 1;
  if (el.closest('.board, .hand')) return 2;
  if (el.closest('.cell')) return 3;
  return 4;
}

function best(elements) {
  let pick = null;
  let pickRank = Infinity;
  for (const el of elements) {
    if (el.closest(OFF_TABLE)) continue;
    const r = el.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) continue;
    const k = rank(el);
    if (k < pickRank) {
      pick = el;
      pickRank = k;
    }
  }
  return pick;
}

/** The on-table element for an object id, or null when it is not drawn (a spell on the stack). */
export function cardElement(objectId) {
  const root = table?.root;
  if (!root || objectId === null || objectId === undefined) return null;
  return best(root.querySelectorAll(`.card[data-oid="${Number(objectId)}"]`));
}

/** The element an arrow to a seat points at: that seat's name, in its best container. */
export function seatElement(name) {
  const root = table?.root;
  if (!root || !name) return null;
  const container = best([...root.querySelectorAll('[data-seat]')].filter((el) => el.dataset.seat === name));
  return container?.querySelector('.nameplate, .who') ?? container;
}

/** The decision-panel card item of an action, while it is drawn. */
export function actionElement(index) {
  return host()?.querySelector(`.act[data-action-index="${Number(index)}"]`) ?? null;
}

/** Where a chosen arrow starts when the source is not on the table. */
export function panelElement() {
  const h = host();
  return h?.querySelector('.stage-line') ?? h;
}

// ── Highlights ───────────────────────────────────────────────────────────────

/**
 * Seat containers are other components' elements, so their `.target-ok` is
 * toggled here rather than declared there. Idempotent; `TableV3` re-runs it
 * after anything that could have re-created a container.
 */
function markSeats() {
  const root = table?.root;
  if (!root) return;
  for (const el of root.querySelectorAll('[data-seat]')) {
    el.classList.toggle('target-ok', targeting.okSeats.has(el.dataset.seat));
  }
}

function refreshHighlights() {
  let objects = EMPTY;
  let seats = EMPTY;
  if (dragSpec && !dragSpec.live) {
    objects = new Set();
    seats = new Set();
    for (const action of dragSpec.actions) {
      const c = driver.candidatesOf(action);
      c.objects.forEach((id) => objects.add(id));
      c.players.forEach((name) => seats.add(name));
    }
  } else if (resolved) {
    objects = new Set();
    seats = new Set();
    for (const slot of resolved.slots) {
      for (const c of slot.candidates ?? []) {
        if (c.kind === 'object') objects.add(Number(c.id));
        else if (c.kind === 'player' && c.owner) seats.add(c.owner);
      }
    }
  }
  if (!sameSet(objects, targeting.okObjects)) targeting.okObjects = objects;
  if (!sameSet(seats, targeting.okSeats)) targeting.okSeats = seats;
  const chosen = new Set(targeting.selection.filter((p) => p.kind === 'object').map((p) => p.id));
  if (!sameSet(chosen, targeting.chosenObjects)) targeting.chosenObjects = chosen;
  markSeats();
}

// ── The open target stage ────────────────────────────────────────────────────

const pickOf = (candidate) =>
  candidate.kind === 'player' ? { kind: 'player', name: candidate.owner } : { kind: candidate.kind, id: Number(candidate.id) };

const samePick = (a, b) => a.kind === b.kind && (a.kind === 'player' ? a.name === b.name : a.id === b.id);

/**
 * Re-read the decision host. Called by the `MutationObserver` on it, after
 * every gesture, and by `TableV3` when the decision or the table changes.
 */
export function sync() {
  if (!table) return;
  const h = host();
  const decision = table.decision();
  const open = driver.chainOpen(h);
  const picker = driver.findPicker(h);
  resolved = picker ? driver.resolvePicker(h, decision?.actions ?? [], table.humanName()) : null;

  if (targeting.chainOpen !== open) targeting.chainOpen = open;
  const unavailable = !!picker && !resolved;
  if (targeting.unavailable !== unavailable) targeting.unavailable = unavailable;
  if (!resolved) {
    if (targeting.live !== null) targeting.live = null;
  } else if (targeting.live?.index !== resolved.option.index || targeting.live?.picker !== resolved.picker) {
    // Two same-label options fit: which card the chain started from is not knowable.
    const objectId = resolved.options.length === 1 ? (resolved.option.object_id ?? null) : null;
    targeting.live = { index: resolved.option.index, objectId, label: resolved.option.label, picker: resolved.picker };
  }

  if (pending) {
    if (!decision || decision.seq !== pending.seq) {
      pending = null;
    } else if (open) {
      const stageName = (h.querySelector('.stage-line .stage-name')?.textContent ?? '').replace(/\s+/g, ' ').trim();
      if (!pending.labels.has(stageName)) {
        // Some other action's chain is open; the pick was not for it.
        pending = null;
      } else if (picker && !resolved) {
        pending = null;
        say('The arrow could not be matched to this picker — choose from the list.');
      } else if (resolved && !resolved.options.some((o) => pending.indexes.has(o.index))) {
        pending = null;
      } else if (resolved && !picker.querySelector('button.cancel')?.disabled) {
        const { pick } = pending;
        pending = null;
        applyToLive(pick, { confirm: true });
        return;
      }
      // else: an X / modes / costs stage first, or a request in flight — the pick waits.
    } else if (!table.chooserOpen() && performance.now() - pending.at > 150) {
      // The chain never opened (a request was in flight) or was backed out of.
      pending = null;
    }
  }

  const selection = resolved
    ? driver.readSelection(resolved).map(({ slot, candidate }) => pickOf(resolved.slots[slot].candidates[candidate]))
    : [];
  const before = targeting.selection;
  if (selection.length !== before.length || selection.some((p, i) => !samePick(p, before[i]))) targeting.selection = selection;
  refreshHighlights();

  if (autoConfirm) {
    if (!resolved) autoConfirm = null;
    else if (selection.some((p) => samePick(p, autoConfirm))) {
      autoConfirm = null;
      driver.confirmIfComplete(h);
    }
  }
}

/**
 * Choose `pick` in the open target stage by clicking the picker's own button.
 *   toggle  — a pick that is already chosen is un-chosen (a click on the table)
 *   confirm — click the picker's Confirm afterwards if the announcement is complete
 */
function applyToLive(pick, { toggle = false, confirm = false } = {}) {
  if (!resolved) return;
  const places = [];
  resolved.slots.forEach((slot, s) => {
    (slot.candidates ?? []).forEach((candidate, c) => {
      if (driver.matchesPick(candidate, pick)) places.push({ slot: s, candidate: c });
    });
  });
  const name = pick.label ?? (pick.kind === 'player' ? pick.name : 'That card');
  if (places.length === 0) {
    say(`${name} is not a legal target for ${resolved.option.label}.`);
    return;
  }
  const chosen = driver.readSelection(resolved);
  const isChosen = (p) => chosen.some((x) => x.slot === p.slot && x.candidate === p.candidate);
  const already = places.find(isChosen);
  let clicked = true;
  if (already) {
    if (toggle) clicked = driver.selectCandidate(resolved, already.slot, already.candidate);
  } else {
    // The first slot that lists it and has room; failing that a single-target
    // slot, where the picker's own rule is that a new pick replaces the old.
    const count = (s) => chosen.filter((x) => x.slot === s).length;
    const max = (s) => resolved.slots[s].max ?? 1;
    const place = places.find((p) => count(p.slot) < max(p.slot)) ?? places.find((p) => max(p.slot) === 1);
    if (!place) {
      say(`No room for ${name} — un-choose a target first.`);
      return;
    }
    clicked = driver.selectCandidate(resolved, place.slot, place.candidate);
  }
  if (!clicked) {
    say('The picker is busy — choose from the list.');
    return;
  }
  if (confirm && !(already && toggle)) {
    const wanted = { kind: pick.kind, id: pick.id, name: pick.name };
    autoConfirm = wanted;
    // The intent to confirm lives for this gesture only: if the picker has not
    // shown the pick as chosen two flushes later, it is dropped, so a later
    // manual choice of the same candidate in the list never confirms by itself.
    tick()
      .then(sync)
      .then(tick)
      .then(() => {
        sync();
        if (autoConfirm === wanted) autoConfirm = null;
      });
    return;
  }
  tick().then(sync);
}

/**
 * A target was chosen for one of `actions` (several only when a dragged card
 * is the source of more than one action the drop is legal for).
 */
function choose(actions, pick, sourceName) {
  if (!table || actions.length === 0) return;
  if (driver.chainOpen(host())) {
    if (resolved && actions.some((a) => resolved.options.includes(a))) applyToLive(pick, { confirm: true });
    else say('Finish or back out of the open picker first.');
    return;
  }
  if (table.loading()) return;
  const decision = table.decision();
  pending = {
    indexes: new Set(actions.map((a) => a.index)),
    labels: new Set(actions.map((a) => String(a.label ?? '').replace(/\s+/g, ' ').trim())),
    pick,
    seq: decision?.seq,
    at: performance.now(),
  };
  if (actions.length === 1) table.begin(actions[0]);
  else table.openChooser(sourceName ?? 'this card', actions);
  tick().then(sync);
  // Expire a pick whose chain never opened, even if nothing in the host changes.
  setTimeout(sync, 250);
}

/** The context menu's "Target with": `card` is the target, `action` the thing aimed at it. */
export function targetWith(action, card) {
  if (!action || !card) return;
  choose([action], { kind: 'object', id: Number(card.object_id), label: card.name }, null);
}

// ── Dragging an arrow ────────────────────────────────────────────────────────

function startDrag(spec, event) {
  closeFloating();
  dragSpec = spec;
  window.getSelection?.()?.removeAllRanges();
  targeting.drag = {
    x: event.clientX,
    y: event.clientY,
    sx: armed?.x ?? event.clientX,
    sy: armed?.y ?? event.clientY,
    fromAct: spec.fromAct,
    fromOid: spec.fromOid,
  };
  refreshHighlights();
}

function endDrag() {
  armed = null;
  dragSpec = null;
  if (targeting.drag !== null) targeting.drag = null;
  window.removeEventListener('pointermove', onPointerMove, true);
  window.removeEventListener('pointerup', onPointerUp, true);
  window.removeEventListener('pointercancel', onPointerCancel, true);
  window.removeEventListener('keydown', onDragKey, true);
  refreshHighlights();
}

function listen() {
  window.addEventListener('pointermove', onPointerMove, true);
  window.addEventListener('pointerup', onPointerUp, true);
  window.addEventListener('pointercancel', onPointerCancel, true);
  window.addEventListener('keydown', onDragKey, true);
}

function onPointerMove(event) {
  if (armed && !dragSpec) {
    if (Math.hypot(event.clientX - armed.x, event.clientY - armed.y) <= DRAG_THRESHOLD) return;
    startDrag(armed.spec, event);
  }
  if (targeting.drag) {
    targeting.drag.x = event.clientX;
    targeting.drag.y = event.clientY;
  }
}

/**
 * What a drop landed on: the topmost thing under the pointer that is not the
 * faded decision box or an overlay. A card wins over the seat it sits in —
 * a drop on a card that is not a legal target must never become "that player".
 */
function hitTest(x, y) {
  const root = table?.root;
  if (!root) return null;
  for (const el of document.elementsFromPoint(x, y)) {
    if (!root.contains(el) || el === root) continue;
    if (el.closest('.v3-arrows, .hover-card, .decision-float')) continue;
    const card = el.closest('.card');
    if (card) {
      const hidden = card.dataset.oid === undefined;
      return { kind: 'card', hidden, id: hidden ? null : Number(card.dataset.oid), name: card.dataset.name ?? null };
    }
    const seat = el.closest('[data-seat]');
    if (seat) return { kind: 'seat', name: seat.dataset.seat };
    return null;
  }
  return null;
}

function onPointerUp(event) {
  if (!dragSpec) {
    // Never left the threshold: this is a click, and the click handler has it.
    endDrag();
    return;
  }
  const spec = dragSpec;
  const hit = hitTest(event.clientX, event.clientY);
  endDrag();
  swallowClickUntil = performance.now() + 250;
  if (!hit) return;
  const what = spec.actions.length === 1 ? spec.actions[0].label : (spec.sourceName ?? 'that');
  if (hit.kind === 'card' && hit.hidden) {
    say(`A hidden card is not a legal target for ${what}.`);
    return;
  }
  const pick =
    hit.kind === 'card' ? { kind: 'object', id: hit.id, label: hit.name ?? 'That card' } : { kind: 'player', name: hit.name, label: hit.name };
  const fits = spec.actions.filter((a) => hasCandidate(a, pick));
  if (fits.length === 0) {
    say(`${pick.label} is not a legal target for ${what}.`);
    return;
  }
  // Not in this event: the browser dispatches the drag's trailing `click` right
  // after this `pointerup`, and both this module and `reorder.js` (whose drag
  // this may have begun as) swallow that click with a capture listener that
  // lives until the next timer turn. The driver's own click on the picker must
  // come after both are gone — hence two timer turns, not a microtask.
  setTimeout(() => {
    setTimeout(() => {
      if (!table) return;
      if (spec.live) applyToLive(pick, { confirm: true });
      else choose(fits, pick, spec.sourceName);
    }, 0);
  }, 0);
}

function onPointerCancel() {
  endDrag();
}

function onDragKey(event) {
  if (event.key !== 'Escape' || !dragSpec) return;
  // Escape ends the drag only; it must not also back out of an open picker.
  event.stopImmediatePropagation();
  event.preventDefault();
  endDrag();
  swallowClickUntil = performance.now() + 250;
}

/**
 * A press on a decision-panel card item. It becomes an arrow once the pointer
 * has moved `DRAG_THRESHOLD` px and the action takes targets; otherwise the
 * item's own click runs untouched.
 */
export function beginArrowFromAction(action, event) {
  if (!table || event.button !== 0 || table.loading() || targeting.chainOpen) return;
  if (!usable(action) || !takesTargets(action)) return;
  endDrag();
  armed = { spec: { actions: [action], fromAct: action.index, fromOid: null, live: false }, x: event.clientX, y: event.clientY };
  listen();
}

/**
 * A drag that began on one of your own cards has left its row (`HandRow` /
 * `YourBoard` call this once, then stop tracking the pointer). Taken over as an
 * arrow when the card is the source of an action that takes targets — every
 * such action lights its targets, and the drop decides which is meant — or
 * when the open target stage belongs to that card. Returns whether it was.
 */
export function beginArrowFromCard(card, event) {
  if (!table || !card || card.hidden || !event || table.loading()) return false;
  if (card.object_id === null || card.object_id === undefined) return false;
  const id = Number(card.object_id);
  let spec;
  if (driver.chainOpen(host())) {
    if (!resolved || !resolved.options.some((o) => o.object_id !== null && o.object_id !== undefined && Number(o.object_id) === id)) return false;
    spec = { actions: [resolved.option], fromAct: null, fromOid: id, live: true, sourceName: card.name };
  } else {
    const actions = actionsNow().filter(
      (a) => a.object_id !== null && a.object_id !== undefined && Number(a.object_id) === id && usable(a) && takesTargets(a),
    );
    if (actions.length === 0) return false;
    spec = { actions, fromAct: null, fromOid: id, live: false, sourceName: card.name };
  }
  endDrag();
  armed = null;
  listen();
  startDrag(spec, event);
  return true;
}

// ── Listeners on the table root ──────────────────────────────────────────────

const INTERACTIVE = 'button, input, select, a, label, .pile';

/**
 * While a target stage is open, a click on a highlighted card or seat chooses
 * it (or un-chooses it). Capture phase on the table root: Svelte delegates
 * `click` to the app's mount target, so this runs before any component's
 * handler — a side cell would otherwise read the click as "bring this seat
 * across" — and stops there when it is consumed.
 */
function onRootClick(event) {
  // `isTrusted`: the driver's own `button.click()` on the picker passes through
  // this listener too, and must never be the click that gets swallowed.
  if (event.isTrusted && performance.now() < swallowClickUntil) {
    swallowClickUntil = 0;
    event.stopPropagation();
    event.preventDefault();
    return;
  }
  if (!resolved || !(event.target instanceof Element)) return;
  const target = event.target;
  if (target.closest('.decision-host, .ctx, .v3-arrows')) return;
  const card = target.closest('.card');
  if (card) {
    // A card that is not a candidate keeps its ordinary click; it never falls through to its seat.
    const id = card.dataset.oid === undefined ? null : Number(card.dataset.oid);
    if (id === null || !targeting.okObjects.has(id)) return;
    event.stopPropagation();
    event.preventDefault();
    applyToLive({ kind: 'object', id, label: card.dataset.name }, { toggle: true });
    return;
  }
  if (target.closest(INTERACTIVE)) return;
  const seat = target.closest('[data-seat]');
  if (!seat || !targeting.okSeats.has(seat.dataset.seat)) return;
  event.stopPropagation();
  event.preventDefault();
  applyToLive({ kind: 'player', name: seat.dataset.seat, label: seat.dataset.seat }, { toggle: true });
}

/**
 * A right-click on a card the seat can identify opens the card menu, and the
 * seat menu under it does not also open. A hidden card (no `data-oid`) is left
 * to whatever is under it, as in C1.
 */
function onRootContextMenu(event) {
  if (!(event.target instanceof Element)) return;
  if (event.target.closest('.ctx')) return;
  const card = event.target.closest('.card[data-oid]');
  if (!card) return;
  event.preventDefault();
  event.stopPropagation();
  table.openCardMenu(Number(card.dataset.oid), card.dataset.name ?? '', event.clientX, event.clientY);
}

/**
 * Bind the controller to a mounted table. `t`:
 *   root                — the `.v3` element
 *   decision()          — the current decision or null
 *   loading()           — a request is in flight
 *   humanName()         — `summary.human_name`
 *   begin(action)       — start the picker chain for an action (`PromptPanel.beginExternal`)
 *   openChooser(name, actions), chooserOpen() — the status slot's "which of these" chooser
 *   say(message)        — a one-line note in the status slot
 *   openCardMenu(objectId, name, x, y)
 * Returns the unbind function.
 */
export function bindTable(t) {
  table = t;
  const root = t.root;
  root.addEventListener('click', onRootClick, true);
  root.addEventListener('contextmenu', onRootContextMenu, true);
  let observer = null;
  const h = host();
  if (h && typeof MutationObserver !== 'undefined') {
    // The picker mounting / unmounting, a candidate gaining `.selected`, the
    // buttons being disabled while a request is in flight.
    observer = new MutationObserver(sync);
    observer.observe(h, { subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: ['class', 'disabled'] });
  }
  tick().then(sync);
  return () => {
    observer?.disconnect();
    root.removeEventListener('click', onRootClick, true);
    root.removeEventListener('contextmenu', onRootContextMenu, true);
    endDrag();
    pending = null;
    autoConfirm = null;
    resolved = null;
    if (table === t) table = null;
  };
}
