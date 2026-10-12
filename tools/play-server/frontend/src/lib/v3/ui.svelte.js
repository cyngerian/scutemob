/**
 * ui — the v3 surface's shared client-side state (`docs/play-surface-v3-brief.md` §3).
 *
 * Floating things (a pile popover, the seat context menu, a peeked cell) are
 * opened from deep inside a board but must be drawn at the table's root: the
 * focused opponent's cell is greyed out with a CSS `filter`, and a filtered
 * element becomes the containing block of its `position: fixed` descendants,
 * so a popover rendered inside it would be clipped and dimmed with it. The
 * opener writes here; `TableV3` renders.
 *
 * Nothing here is game state. The seat view stays in `stores.js`.
 */

function readFlag(key, fallback) {
  try {
    const v = localStorage.getItem(key);
    return v === null ? fallback : v !== '0';
  } catch {
    return fallback;
  }
}

function readNumber(key, fallback) {
  try {
    return Number(localStorage.getItem(key)) || fallback;
  } catch {
    return fallback;
  }
}

function write(key, value) {
  try {
    localStorage.setItem(key, String(value));
  } catch {
    // storage is a convenience; the table works without it
  }
}

/**
 * Card sizing (mockup rule 5). `cap` is the size slider; `auto` is the measured
 * battlefield fit for the centre boards; `oppAuto` lets side cells drop below
 * eight per row to use their height. Moving the slider turns `auto` off.
 */
export const settings = $state({
  cap: readNumber('v3-cap', 84),
  auto: readFlag('v3-auto', true),
  oppAuto: readFlag('v3-oppauto', true),
});

export function setCap(px) {
  settings.cap = px;
  settings.auto = false;
  write('v3-cap', px);
  write('v3-auto', '0');
}

export function setAuto(on) {
  settings.auto = on;
  write('v3-auto', on ? '1' : '0');
}

export function setOppAuto(on) {
  settings.oppAuto = on;
  write('v3-oppauto', on ? '1' : '0');
}

/**
 * `popover`: `{ title, note, cards, x, y }` — `cards` are seat-view entries
 *   (`CardInZoneView`, hidden ones included: they draw as backs).
 * `ctx`: `{ name, x, y }` — the seat whose context menu is open.
 * `peek`: `{ name, rect }` — the side cell grown to double size; `rect` is the
 *   cell's `getBoundingClientRect()` when the hover timer fired.
 * `combatOpen`: the enlarged combat view.
 */
export const ui = $state({ popover: null, ctx: null, peek: null, combatOpen: false });

export function openPopover({ title, note = '', cards = [], x = 0, y = 0 }) {
  ui.ctx = null;
  ui.popover = { title, note, cards, x, y };
}

export function openCtx(name, x, y) {
  ui.popover = null;
  ui.ctx = { name, x, y };
}

export function closeFloating() {
  ui.popover = null;
  ui.ctx = null;
}

export function openPeek(name, rect) {
  ui.peek = { name, rect: { left: rect.left, right: rect.right, top: rect.top, width: rect.width, height: rect.height } };
}

export function closePeek() {
  ui.peek = null;
}
