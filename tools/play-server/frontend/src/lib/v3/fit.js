/**
 * fit — the measured battlefield fit (`docs/play-surface-v3-brief.md` §1 rule 5),
 * ported from `mockups/07-table6.html` with its constants unchanged.
 *
 * A board declares its rows in `data-rows` (`[[cards, scale], …]` per zone),
 * its fixed chrome in `data-overhead` (rem), its row-label width in
 * `data-label` (rem) and optionally `data-per-row`. `fitBoard` finds the one
 * `--card` size that makes those rows fit the box: a model pass gives the
 * ceiling, then the browser is asked whether the box scrolls — shrink until it
 * does not, grow until it would. Piles and the hand are fixed size and never
 * touched here.
 */

const PER_ROW = 8;
const MIN_CARD = 22;

function apply(el, v) {
  el.style.setProperty('--card', v.toFixed(1) + 'px');
  el.style.setProperty('--cardn', v.toFixed(1));
}

function sizeOf(el) {
  return parseFloat(el.style.getPropertyValue('--card'));
}

/** Give every element in `els` the smallest `--card` among them. */
function shareSmallest(els) {
  const sizes = els.map(sizeOf).filter((v) => Number.isFinite(v));
  if (sizes.length === 0) return;
  const c = Math.min(...sizes);
  for (const e of els) apply(e, c);
}

/** The one card size for `el` that makes its rows fit the box. `opts` is `settings` (`cap`, `auto`, `oppAuto`). */
export function fitBoard(el, { cap = 84, auto = true, oppAuto = true } = {}) {
  if (!el?.dataset?.rows) return;
  if (el.clientHeight === 0 || el.clientWidth === 0) return;
  let rows;
  try {
    rows = JSON.parse(el.dataset.rows);
  } catch {
    return;
  }
  if (!Array.isArray(rows)) return;
  const isCell = el.classList.contains('cell');
  // opponent cells: with opp auto-size on, rows may hold fewer than 8 so the cards grow into the cell's height
  const minPerRow = isCell && oppAuto ? 1 : Number(el.dataset.perRow || PER_ROW);
  const rem = parseFloat(getComputedStyle(document.documentElement).fontSize);
  const H = el.clientHeight - Number(el.dataset.overhead || 0) * rem - rows.length * 6;
  const W = el.clientWidth - Number(el.dataset.label ?? 4.6) * rem - 20;
  // never wider than minPerRow-per-row allows; then shrink until the stacked rows fit the height
  let c = Math.min(cap, (W - (minPerRow - 1) * 3) / minPerRow);
  for (; (auto || isCell) && c > MIN_CARD; c -= 1) {
    let units = 0;
    const Wb = isCell ? W : W - (110 + 2.5 * rem); // centre boards reserve the zone box's width
    for (const [n, sc] of rows) {
      const perRow = Math.max(minPerRow, Math.floor((Wb + 3) / (c * sc + 3))); // a row of this zone's cards fills the width
      units += Math.max(1, Math.ceil(n / perRow)) * sc * 1.4 + 0.06;
    }
    if (units * c <= H) break;
  }
  c = Math.max(MIN_CARD, c);
  const cUp = c; // the model's answer is the ceiling; the DOM has the final say
  const overflows = () => el.scrollHeight > el.clientHeight + 1; // vertical only: wrapped rows never overflow sideways
  apply(el, c);
  if (auto || isCell) {
    while (c > MIN_CARD && overflows()) {
      c -= 1;
      apply(el, c);
    } // never scroll
    while (c < cUp) {
      apply(el, c + 1);
      if (overflows()) {
        apply(el, c);
        break;
      }
      c += 1;
    } // fill the box
  }
}

/** Strip piles: sized once from the cell's width so the five share the name line (34–56px). */
export function sizeStrips(root) {
  if (!root) return;
  root.querySelectorAll('.cell').forEach((cell) => {
    if (cell.clientWidth === 0) return;
    const who = cell.querySelector('.who');
    const whoW = who ? who.getBoundingClientRect().width : 70;
    const room = cell.clientWidth - 14 - whoW - 8 - 10 - 4 * 5 - 10; // padding, name block, gap, strip padding, pile gaps, the CMD outline
    const pw = Math.max(34, Math.min(56, Math.floor(room / 5)));
    cell.style.setProperty('--pile', pw + 'px');
  });
}

/**
 * Fit every board under `root`. Your board and the board across the table
 * share one size (the smaller fit); all side cells share the smallest. The
 * hand row is not touched: `HandRow` measures itself.
 */
export function fitAll(root, opts) {
  if (!root) return;
  sizeStrips(root);
  root.querySelectorAll('[data-rows]').forEach((el) => fitBoard(el, opts));
  const centre = [root.querySelector('.board'), root.querySelector('.focused')].filter((e) => e && e.style.getPropertyValue('--card'));
  if (centre.length === 2) shareSmallest(centre);
  const cells = [...root.querySelectorAll('.cell[data-rows]:not(.cell-pop)')];
  if (cells.length) shareSmallest(cells);
}
