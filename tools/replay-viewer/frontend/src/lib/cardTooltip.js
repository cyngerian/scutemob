/**
 * cardTooltip — Svelte action that shows a floating Scryfall card image on hover.
 *
 * Usage:
 *   <div use:cardTooltip={cardName}>...</div>
 *   <div use:cardTooltip={isToken ? null : cardName}>...</div>  // skip tokens
 *   <div use:cardTooltip={{ name: cardName, caption: typeLine }}>...</div>
 *
 * A single tooltip element is shared across all instances to avoid creating
 * hundreds of img elements for large battlefields.
 *
 * # Why this action grew a caption (G11, UI-5 `scutemob-190`)
 *
 * Every call site used to pair `use:cardTooltip` with a native `title=` on the
 * SAME element — the type line, or "Name (Types)". The browser/OS draws a
 * `title` tooltip **at the cursor, above every z-index this document can
 * reach**, which is exactly where this action anchors the card image. The
 * playtest note ("hover card name interferes with the card image") is that
 * collision, and it **cannot be fixed with CSS**: a native tooltip is chrome,
 * not DOM. The only fixes are to delete the `title` or to move its text
 * somewhere this code controls. The text is worth keeping, so it moved: pass
 * `{name, caption}` and the caption renders inside the floating div, under the
 * image, where nothing overdraws it.
 *
 * `title` remains fine on controls that are NOT tooltip anchors (buttons,
 * badges, the pass-priority hint) — the ban is on the card elements alone, and
 * `test_frontend_card_elements_carry_no_native_title` states it that way.
 */

const IMG_WIDTH = 223; // normal image width at the size we display it

let tooltipEl = null;
let imgEl = null;
let captionEl = null;
let refCount = 0;

/**
 * The caption for a `CardInZoneView` — hand, graveyard, exile, command zone.
 *
 * Shared by the three zone components and `SeatCard` so the four G11 sites that
 * used to write the same `"{name} ({types})"` template into a `title=` cannot
 * drift apart. `ZoneBattlefield` has its own richer `captionFor` because a
 * `PermanentView` carries battlefield-only state (P/T, damage, counters, tap)
 * that no card in another zone has.
 *
 * Returns `null` for a card the redactor hid (`redact::hidden_placeholder`
 * rewrites the name to "Hidden card" and the id to 0): there is nothing to
 * caption and nothing to fetch, and asking Scryfall for "Hidden card" was a
 * guaranteed 404 even before this.
 */
export function zoneCaption(card) {
  if (!card || card.hidden) return null;
  const types = (card.card_types ?? []).join(' ');
  return types ? `${card.name}\n${types}` : (card.name ?? null);
}

/** Normalize both accepted argument shapes to `{name, caption}`. */
function normalize(arg) {
  if (arg === null || arg === undefined || arg === false) return { name: null, caption: null };
  if (typeof arg === 'string') return { name: arg, caption: null };
  return { name: arg.name ?? null, caption: arg.caption ?? null };
}

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
    'background:#0b0b12',
  ].join(';');

  imgEl = document.createElement('img');
  imgEl.style.cssText = 'width:100%;display:block;';
  // The image failing must not take the caption with it: an unreachable
  // Scryfall (the M14 known limitation — images are fetched, not cached) is
  // precisely when the text is the only thing left to read. With no caption
  // either there is nothing to show, and the pre-G11 behaviour — hide the whole
  // box — is still the right one, so both are kept.
  imgEl.onerror = () => {
    imgEl.style.display = 'none';
    if (!captionEl.textContent) tooltipEl.style.display = 'none';
  };
  tooltipEl.appendChild(imgEl);

  captionEl = document.createElement('div');
  captionEl.setAttribute('data-card-tooltip-caption', '');
  captionEl.style.cssText = [
    'font-family:monospace',
    'font-size:0.72rem',
    'line-height:1.25',
    'color:#ccd',
    // `pre-line` so a caption may carry a second line (the status line the
    // badges' titles folded into) without needing markup — `textContent` keeps
    // the caption plain text, which is what makes it injection-proof.
    'white-space:pre-line',
    'padding:0.3rem 0.45rem',
    'display:none',
  ].join(';');
  tooltipEl.appendChild(captionEl);

  document.body.appendChild(tooltipEl);
}

function position(e) {
  if (!tooltipEl) return;
  const pad = 14;
  // Approximate height from Scryfall normal aspect ratio (680/488 ≈ 1.393).
  //
  // `offsetHeight` once the element is displayed, because the caption adds a
  // variable number of lines below the image and the constant alone would clamp
  // the box off the bottom of a short viewport — but **floored** at the
  // constant whenever an image is expected, which is a `/review` correction.
  // `onEnter` assigns `imgEl.src` and calls this synchronously, so on the very
  // first frame of a hover the image has no layout box yet and `offsetHeight`
  // is caption-height alone (~30px); centring against that put the image itself
  // off-screen until the first `mousemove` fixed it.
  const measured = tooltipEl.offsetHeight;
  const nominal = Math.round(IMG_WIDTH * 1.393);
  const imgHeight = imgEl.style.display === 'none'
    ? measured || nominal
    : Math.max(measured, nominal);
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

export function cardTooltip(node, arg) {
  let current = normalize(arg);

  // A node with neither a name nor a caption has nothing to show. Kept as the
  // pre-G11 early return so `use:cardTooltip={isToken ? null : name}` still
  // costs nothing on a token — but a caption alone is now enough to hover,
  // which is what a token needs (no Scryfall page, a real type line).
  if (!current.name && !current.caption) return {};

  ensureTooltip();
  refCount++;

  function scryfallUrl(n) {
    return `https://api.scryfall.com/cards/named?exact=${encodeURIComponent(n)}&format=image&version=normal`;
  }

  function render() {
    if (current.name) {
      imgEl.style.display = 'block';
      // Guarded: `update` re-renders while hovered (tapping the land you are
      // pointing at), and re-assigning the same `src` restarts the image-data
      // algorithm — usually cache-served, but it can flash. `/review` finding.
      const url = scryfallUrl(current.name);
      if (imgEl.src !== url) imgEl.src = url;
    } else {
      imgEl.style.display = 'none';
      imgEl.removeAttribute('src');
    }
    if (current.caption) {
      captionEl.textContent = current.caption;
      captionEl.style.display = 'block';
    } else {
      captionEl.textContent = '';
      captionEl.style.display = 'none';
    }
  }

  function onEnter(e) {
    render();
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
    update(newArg) {
      current = normalize(newArg);
      // Only repaint while this node is the one being hovered; otherwise the
      // next `mouseenter` will do it. `render()` unconditionally would rewrite
      // another node's visible tooltip on any unrelated reactive update.
      if (tooltipEl && tooltipEl.style.display === 'block' && node.matches(':hover')) render();
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
        captionEl = null;
      }
    },
  };
}
