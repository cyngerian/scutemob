/**
 * hover — the one card hover of the v3 surface (`docs/play-surface-v3-brief.md` §1 rule 7).
 *
 * A single fixed overlay (240px wide, the `normal` printing from the server's
 * local image store) placed beside the hovered card, flipped near the right edge and clamped vertically,
 * so it is never clipped by a board's overflow. It is appended inside the
 * `.v3` root because the stylesheet is scoped there. Every hover is also
 * written to `localStorage['scutemob-card']`, whose `storage` event feeds the
 * popped-out card window (`CardWindow.svelte`) on the second monitor. The
 * mockup used a `BroadcastChannel`; its send is a structured clone, the one
 * primitive this client bans outright (UI-4), and a storage event carries a
 * card name just as well.
 *
 * The overlay is a text chip with the picture over it, like a card on the
 * table: the chip (name, and the type line and P/T the hovered card face is
 * already showing) is what the player sees until the picture has loaded, and
 * all they see when the store does not hold it (a 404). A miss never hides the
 * overlay. The chip is styled inline because it lives outside any component.
 */
import { cardImageUrl } from '../v2/images.js';

const HOVER_W = 240;

let hoverEl = null;
let hoverOwner = null;
export const CARD_KEY = 'scutemob-card';

function broadcastHover(name, info) {
  try {
    // The timestamp makes a second hover of the same card a change, so it still fires.
    localStorage.setItem(CARD_KEY, JSON.stringify({ name, type: info?.type || null, pt: info?.pt || null, at: Date.now() }));
  } catch {
    // no storage, no second window; the in-page overlay is unaffected
  }
}

const CHIP_STYLE =
  'box-sizing:border-box;width:100%;aspect-ratio:5/7;display:flex;flex-direction:column;align-items:center;justify-content:center;' +
  'gap:8px;padding:16px;text-align:center;background:#1b1b2a;border:1px solid #556;border-radius:10px;color:#ccd;' +
  "font-family:'Inter','Segoe UI',system-ui,sans-serif;overflow:hidden;";

function line(cls, style) {
  const el = document.createElement('div');
  el.className = cls;
  el.style.cssText = style;
  return el;
}

/** Which of the two the player sees: the picture once it has loaded, the chip otherwise. */
function showFace(el, picture) {
  el.querySelector('img').style.display = picture ? 'block' : 'none';
  el.querySelector('.hover-chip').style.display = picture ? 'none' : 'flex';
  el.dataset.face = picture ? 'image' : 'chip';
}

function overlay() {
  if (hoverEl?.isConnected) return hoverEl;
  hoverEl = document.createElement('div');
  hoverEl.className = 'hover-card';
  const chip = line('hover-chip', CHIP_STYLE);
  chip.append(
    line('hover-chip-name', 'font-size:17px;font-weight:700;color:#eef;overflow-wrap:anywhere;'),
    line('hover-chip-type', 'font-size:13px;color:#99a;overflow-wrap:anywhere;'),
    line('hover-chip-pt', 'font-size:16px;font-weight:800;color:#fff;'),
  );
  const img = document.createElement('img');
  img.alt = '';
  img.draggable = false;
  img.onload = () => showFace(hoverEl, true);
  // a miss leaves the chip; dropping the src lets the next hover of that name ask the store again
  img.onerror = () => {
    img.removeAttribute('src');
    showFace(hoverEl, false);
  };
  hoverEl.append(chip, img);
  showFace(hoverEl, false);
  (document.querySelector('.v3') ?? document.body).append(hoverEl);
  return hoverEl;
}

/**
 * Show `name` beside `rect` (a card's `getBoundingClientRect()`) and tell the card window.
 * `info` is `{ type, pt }`: text the hovered card face already displays, for the chip.
 */
export function showHover(name, rect, info = null) {
  if (!name || !rect) return;
  const el = overlay();
  const img = el.querySelector('img');
  el.querySelector('.hover-chip-name').textContent = name;
  el.querySelector('.hover-chip-type').textContent = info?.type || '';
  el.querySelector('.hover-chip-pt').textContent = info?.pt || '';
  const url = cardImageUrl(name, 'normal');
  if (img.getAttribute('src') !== url) {
    showFace(el, false);
    img.src = url;
  }
  const H = HOVER_W * 1.4;
  let x = rect.right + 10;
  if (x + HOVER_W > innerWidth - 8) x = rect.left - HOVER_W - 10;
  if (x < 8) x = 8;
  let y = rect.top + rect.height / 2 - H / 2;
  y = Math.max(8, Math.min(innerHeight - H - 8, y));
  el.style.left = x + 'px';
  el.style.top = y + 'px';
  el.style.display = 'block';
  broadcastHover(name, info);
}

export function hideHover() {
  hoverOwner = null;
  if (hoverEl) hoverEl.style.display = 'none';
}

/**
 * Svelte action: `use:hoverCard={name}`. A null or empty name (a hidden card,
 * a token) does nothing. A card removed while hovered takes the overlay with it.
 * The chip's type line and P/T are read off the card face being hovered (its own
 * `.chip-type` and `.pt` text), so the overlay can say no more than the table does.
 */
export function hoverCard(node, name) {
  let current = name;
  const enter = () => {
    if (!current) return;
    const text = (sel) => node.querySelector(sel)?.textContent?.trim() || null;
    showHover(current, node.getBoundingClientRect(), { type: text('.chip-type'), pt: text('.pt:not(.loy)') });
    hoverOwner = node;
  };
  const leave = () => {
    if (hoverOwner === node) hideHover();
  };
  node.addEventListener('mouseenter', enter);
  node.addEventListener('mouseleave', leave);
  return {
    update(next) {
      current = next;
      if (hoverOwner === node) {
        if (current) enter();
        else hideHover();
      }
    },
    destroy() {
      node.removeEventListener('mouseenter', enter);
      node.removeEventListener('mouseleave', leave);
      if (hoverOwner === node) hideHover();
    },
  };
}
