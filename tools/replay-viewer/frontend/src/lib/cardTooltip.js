/**
 * cardTooltip — Svelte action that shows a floating Scryfall card image on hover.
 *
 * Usage:
 *   <div use:cardTooltip={cardName}>...</div>
 *   <div use:cardTooltip={isToken ? null : cardName}>...</div>  // skip tokens
 *
 * A single tooltip element is shared across all instances to avoid creating
 * hundreds of img elements for large battlefields.
 */

const IMG_WIDTH = 223; // normal image width at the size we display it

let tooltipEl = null;
let imgEl = null;
let refCount = 0;

function ensureTooltip() {
  if (tooltipEl) return;
  tooltipEl = document.createElement('div');
  tooltipEl.style.cssText = [
    'position:fixed',
    'z-index:9999',
    'pointer-events:none',
    'display:none',
    `width:${IMG_WIDTH}px`,
    'border-radius:10px',
    'overflow:hidden',
    'box-shadow:0 4px 24px rgba(0,0,0,0.85)',
  ].join(';');

  imgEl = document.createElement('img');
  imgEl.style.cssText = 'width:100%;display:block;';
  imgEl.onerror = () => { tooltipEl.style.display = 'none'; };
  tooltipEl.appendChild(imgEl);
  document.body.appendChild(tooltipEl);
}

function position(e) {
  if (!tooltipEl) return;
  const pad = 14;
  // Approximate height from Scryfall normal aspect ratio (680/488 ≈ 1.393)
  const imgHeight = Math.round(IMG_WIDTH * 1.393);
  const vw = window.innerWidth;
  const vh = window.innerHeight;

  let x = e.clientX + pad;
  let y = e.clientY - imgHeight / 2;

  // Flip left if too close to right edge
  if (x + IMG_WIDTH > vw - pad) x = e.clientX - IMG_WIDTH - pad;
  // Clamp vertically
  if (y < pad) y = pad;
  if (y + imgHeight > vh - pad) y = vh - imgHeight - pad;

  tooltipEl.style.left = `${x}px`;
  tooltipEl.style.top = `${y}px`;
}

export function cardTooltip(node, name) {
  if (!name) return {};

  ensureTooltip();
  refCount++;

  let currentName = name;

  function scryfallUrl(n) {
    return `https://api.scryfall.com/cards/named?exact=${encodeURIComponent(n)}&format=image&version=normal`;
  }

  function onEnter(e) {
    imgEl.src = scryfallUrl(currentName);
    tooltipEl.style.display = 'block';
    position(e);
  }

  function onMove(e) {
    position(e);
  }

  function onLeave() {
    tooltipEl.style.display = 'none';
  }

  node.addEventListener('mouseenter', onEnter);
  node.addEventListener('mousemove', onMove);
  node.addEventListener('mouseleave', onLeave);

  return {
    update(newName) {
      currentName = newName;
    },
    destroy() {
      node.removeEventListener('mouseenter', onEnter);
      node.removeEventListener('mousemove', onMove);
      node.removeEventListener('mouseleave', onLeave);
      refCount--;
      if (refCount === 0 && tooltipEl) {
        tooltipEl.remove();
        tooltipEl = null;
        imgEl = null;
      }
    },
  };
}
