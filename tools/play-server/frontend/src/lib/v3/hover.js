/**
 * hover — the one card hover of the v3 surface (`docs/play-surface-v3-brief.md` §1 rule 7).
 *
 * A single fixed overlay (240px wide, Scryfall's `normal` printing) placed
 * beside the hovered card, flipped near the right edge and clamped vertically,
 * so it is never clipped by a board's overflow. It is appended inside the
 * `.v3` root because the stylesheet is scoped there. Every hover is also
 * written to `localStorage['scutemob-card']`, whose `storage` event feeds the
 * popped-out card window (`CardWindow.svelte`) on the second monitor. The
 * mockup used a `BroadcastChannel`; its send is a structured clone, the one
 * primitive this client bans outright (UI-4), and a storage event carries a
 * card name just as well.
 */
import { scryfallImageUrl } from '../v2/images.js';

const HOVER_W = 240;

let hoverEl = null;
let hoverOwner = null;
export const CARD_KEY = 'scutemob-card';

function broadcastHover(name) {
  try {
    // The timestamp makes a second hover of the same card a change, so it still fires.
    localStorage.setItem(CARD_KEY, JSON.stringify({ name, at: Date.now() }));
  } catch {
    // no storage, no second window; the in-page overlay is unaffected
  }
}

function overlay() {
  if (hoverEl?.isConnected) return hoverEl;
  hoverEl = document.createElement('div');
  hoverEl.className = 'hover-card';
  const img = document.createElement('img');
  img.alt = '';
  img.draggable = false;
  // a failed printing hides the overlay; dropping the src lets the next hover of that name try again
  img.onerror = () => {
    img.removeAttribute('src');
    hideHover();
  };
  hoverEl.append(img);
  (document.querySelector('.v3') ?? document.body).append(hoverEl);
  return hoverEl;
}

/** Show `name` beside `rect` (a card's `getBoundingClientRect()`) and tell the card window. */
export function showHover(name, rect) {
  if (!name || !rect) return;
  const el = overlay();
  const img = el.firstChild;
  const url = scryfallImageUrl(name, 'normal');
  if (img.getAttribute('src') !== url) img.src = url;
  const H = HOVER_W * 1.4;
  let x = rect.right + 10;
  if (x + HOVER_W > innerWidth - 8) x = rect.left - HOVER_W - 10;
  if (x < 8) x = 8;
  let y = rect.top + rect.height / 2 - H / 2;
  y = Math.max(8, Math.min(innerHeight - H - 8, y));
  el.style.left = x + 'px';
  el.style.top = y + 'px';
  el.style.display = 'block';
  broadcastHover(name);
}

export function hideHover() {
  hoverOwner = null;
  if (hoverEl) hoverEl.style.display = 'none';
}

/**
 * Svelte action: `use:hoverCard={name}`. A null or empty name (a hidden card,
 * a token) does nothing. A card removed while hovered takes the overlay with it.
 */
export function hoverCard(node, name) {
  let current = name;
  const enter = () => {
    if (!current) return;
    showHover(current, node.getBoundingClientRect());
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
