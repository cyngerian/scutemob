// Shared sample state + card renderer for the five v2 layout mockups.
// No wiring: everything here is static. Images come straight from Scryfall.

const SCRY = (n, v = 'small') =>
  `https://api.scryfall.com/cards/named?exact=${encodeURIComponent(n)}&format=image&version=${v}`;

const GAME = {
  turn: 10,
  active: 'Bot-3',
  step: 'Main 1',
  priority: 'Human-1',
  you: {
    name: 'Human-1', life: 40, hand: 5, lib: 89, gy: 3, exile: 1,
    commander: 'Nadaar, Selfless Paladin',
    creatures: [
      { n: 'Nadaar, Selfless Paladin', pt: '3/3', cmd: true },
      { n: 'Thalia, Guardian of Thraben', pt: '2/1', targeted: true },
      { n: 'Adeline, Resplendent Cathar', pt: '4/4' },
      { n: 'Luminarch Aspirant', pt: '3/3', counters: '+1/+1 ×1' },
      { n: 'Seasoned Hallowblade', pt: '3/1', tapped: true },
      { n: 'Archon of Emeria', pt: '2/3' },
      { n: 'Benalish Marshal', pt: '3/3' },
      { n: 'Dusk Legion Duelist', pt: '2/2' },
    ],
    others: [
      { n: "Elspeth, Sun's Champion", loy: 4 },
      { n: 'Swiftfoot Boots' },
      { n: 'Thought Vessel', tapped: true },
      { n: 'Marble Diamond' },
      { n: 'Hedron Archive' },
      { n: 'Smothering Tithe' },
    ],
    lands: [
      { n: 'Plains', count: 5, tapped: 2 },
      { n: 'Command Tower' },
      { n: 'Reliquary Tower', tapped: true },
    ],
    hand: [
      { n: 'Swords to Plowshares', lit: true },
      { n: 'Path to Exile', lit: true },
      { n: 'Sun Titan' },
      { n: 'Plains' },
      { n: 'Wrath of God' },
    ],
  },
  opps: [
    { name: 'Bot-2', life: 38, hand: 7, lib: 90, gy: 2, commander: 'Neriv, Heart of the Storm',
      creatures: [{ n: 'Goblin Guide', pt: '2/2' }, { n: 'Neriv, Heart of the Storm', pt: '4/4', cmd: true }],
      others: [{ n: 'Sol Ring', tapped: true }, { n: 'Lightning Greaves' }],
      lands: { total: 5, tapped: 3 } },
    { name: 'Bot-3', life: 40, hand: 6, lib: 88, gy: 5, commander: 'Toski, Bearer of Secrets', active: true,
      creatures: [{ n: 'Llanowar Elves', pt: '1/1', tapped: true }, { n: 'Beast Whisperer', pt: '2/3' }, { n: 'Toski, Bearer of Secrets', pt: '1/1', cmd: true }],
      others: [{ n: 'Sylvan Library' }],
      lands: { total: 4, tapped: 3 } },
    { name: 'Bot-4', life: 34, hand: 4, lib: 91, gy: 7, commander: 'Ayara, First of Locthwain',
      creatures: [{ n: 'Ayara, First of Locthwain', pt: '2/3', cmd: true }, { n: 'Gravecrawler', pt: '2/1' }, { n: 'Bloodghast', pt: '2/1' }],
      others: [{ n: 'Phyrexian Arena' }, { n: 'Arcane Signet' }],
      lands: { total: 6, tapped: 0 } },
  ],
  stack: [
    { who: 'Bot-3', n: 'Beast Within', text: 'targeting your Thalia, Guardian of Thraben' },
  ],
  prompt: {
    sentence: 'Bot-3 cast Beast Within targeting your Thalia. Respond, or let it resolve.',
    buttons: [
      { t: 'Let it resolve', key: 'space', primary: true },
      { t: 'Swords to Plowshares → Beast Whisperer', lit: true },
      { t: 'Path to Exile → Beast Whisperer', lit: true },
    ],
  },
  log: [
    ['Bot-3', 'untaps, draws'],
    ['Bot-3', 'plays Forest'],
    ['Bot-3', 'taps Llanowar Elves for {G}'],
    ['Bot-3', 'casts Beast Within'],
    ['', '→ targeting Thalia, Guardian of Thraben'],
    ['Bot-4', 'passes'],
    ['Bot-2', 'passes'],
    ['Human-1', 'has priority'],
  ],
};

GAME.steps = ['Untap', 'Upkeep', 'Draw', 'Main 1', 'Begin combat', 'Attackers', 'Blockers', 'Damage', 'End combat', 'Main 2', 'End', 'Cleanup'];
GAME.you.gyCards = [{ n: 'Wrath of God' }, { n: 'Luminarch Aspirant' }, { n: 'Plains' }];
GAME.you.exileCards = [{ n: 'Gravecrawler' }];
GAME.opps[0].gyCards = [{ n: 'Goblin Guide' }, { n: 'Sol Ring' }];
GAME.opps[1].gyCards = [{ n: 'Llanowar Elves' }, { n: 'Sylvan Library' }, { n: 'Beast Whisperer' }, { n: 'Llanowar Elves' }, { n: 'Beast Within' }];
GAME.opps[2].gyCards = [{ n: 'Bloodghast' }, { n: 'Gravecrawler' }, { n: 'Arcane Signet' }, { n: 'Phyrexian Arena' }, { n: 'Bloodghast' }, { n: 'Ayara, First of Locthwain' }, { n: 'Gravecrawler' }];

/** Hover → the popped-out card window (card-window.html) on the second monitor. */
const CARD_CHANNEL = typeof BroadcastChannel !== 'undefined' ? new BroadcastChannel('scutemob-card') : null;
function broadcastHover(name) { CARD_CHANNEL?.postMessage({ name }); }

/** Deep-ish copy of the sample state so a state switcher can mutate freely. */
function cloneGame() { return JSON.parse(JSON.stringify(GAME)); }

/** Repeat a list to n entries (names repeat; stands in for tokens and duplicates). */
function padTo(list, n) { const out = []; for (let i = 0; out.length < n; i++) out.push({ ...list[i % list.length] }); return out; }

/** The four board states mockup 6 switches between. Each returns {game, focus}. */
const STATES = {
  1: () => ({ label: "Bot-3's turn · Beast Within on the stack · you hold priority", game: cloneGame(), focus: 'Bot-3' }),
  2: () => {
    const g = cloneGame();
    g.active = 'Human-1'; g.priority = 'Human-1'; g.step = 'Main 1'; g.stack = [];
    g.you.creatures.forEach((c) => { delete c.targeted; });
    g.you.hand = [{ n: 'Swords to Plowshares', lit: true }, { n: 'Path to Exile', lit: true }, { n: 'Sun Titan', lit: true }, { n: 'Plains', lit: true }, { n: 'Wrath of God', lit: true }];
    g.prompt = { sentence: 'Your main phase. Play a land, cast a spell, or pass.', buttons: [{ t: 'Pass priority', key: 'space', primary: true }, { t: 'to combat' }, { t: 'to end of turn' }] };
    g.opps.forEach((o) => { delete o.active; });
    return { label: 'Your turn · Main 1 · empty stack', game: g, focus: 'Bot-3' };
  },
  3: () => {
    const g = cloneGame();
    g.step = 'Blockers';
    g.you.creatures = padTo(g.you.creatures, 12); g.you.others = padTo(g.you.others, 6);
    g.you.lands = [{ n: 'Plains', count: 9, tapped: 4 }, { n: 'Command Tower' }, { n: 'Reliquary Tower', tapped: true }, { n: 'Plains', count: 2 }];
    g.opps[1].creatures = padTo(g.opps[1].creatures, 12); g.opps[1].others = padTo(g.opps[1].others, 6); g.opps[1].lands = { total: 11, tapped: 8 };
    g.opps[0].creatures = padTo(g.opps[0].creatures, 8); g.opps[0].others = padTo(g.opps[0].others, 4); g.opps[0].lands = { total: 9, tapped: 2 };
    g.opps[2].creatures = padTo(g.opps[2].creatures, 9); g.opps[2].others = padTo(g.opps[2].others, 3); g.opps[2].lands = { total: 10, tapped: 10 };
    g.you.hand = padTo(g.you.hand, 9);
    return { label: 'Late game · 22 / 18 / 12 / 12 permanents · declare blockers', game: g, focus: 'Bot-3' };
  },
  4: () => {
    const g = cloneGame();
    g.priority = 'Bot-4';
    g.you.hand.forEach((c) => { delete c.lit; });
    g.prompt = { sentence: 'Bot-4 is deciding…', buttons: [], waiting: true };
    return { label: "Bot-3's turn · Bot-4 holds priority · you are waiting", game: g, focus: 'Bot-3' };
  },
};

// ---- rendering helpers ------------------------------------------------------

function el(tag, cls, html) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (html !== undefined) e.innerHTML = html;
  return e;
}

/** A card box. size ∈ hand|perm|small|mini. Fixed box; image fills it; chip under it. */
function cardEl(c, size = 'perm') {
  const d = el('div', `card ${size}`);
  if (c.tapped) d.classList.add('tapped');
  if (c.lit) d.classList.add('lit');
  if (c.targeted) d.classList.add('targeted');
  if (c.cmd) d.classList.add('cmd');
  d.title = c.n;
  d.dataset.name = c.n;
  d.addEventListener('mouseenter', (ev) => {
    // Keep a magnified card on screen: flip the transform origin near the edges.
    const r = d.getBoundingClientRect();
    const ox = r.left < 180 ? 'left' : window.innerWidth - r.right < 180 ? 'right' : 'center';
    const oy = r.top < 160 ? 'top' : window.innerHeight - r.bottom < 160 ? 'bottom' : 'center';
    d.style.transformOrigin = `${ox} ${oy}`;
    broadcastHover(c.n);
  });
  const chip = el('div', 'chip', `<div class="chip-name">${c.n}</div>`);
  const img = el('img', 'art');
  const v = size === 'hand' || size === 'big' ? 'normal' : 'small';
  // Local copy first (fetch-images.py), Scryfall once as the fallback.
  img.src = `img/${c.n.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')}-${v}.jpg`;
  img.loading = 'lazy';
  img.onload = () => d.classList.add('has-image');
  img.onerror = () => { if (img.dataset.fb) { d.classList.add('no-image'); return; } img.dataset.fb = '1'; img.src = SCRY(c.n, v); };
  d.append(chip, img);
  d.append(el('div', 'band', c.n));
  if (c.pt) d.append(el('div', 'pt', c.pt));
  if (c.loy) d.append(el('div', 'pt loy', `◆ ${c.loy}`));
  if (c.count) d.append(el('div', 'count', `×${c.count}`));
  if (c.tapped) d.append(el('div', 'tag tap', 'T'));
  if (c.cmd) d.append(el('div', 'tag cmdr', '★'));
  if (c.counters) d.append(el('div', 'tag ctr', c.counters));
  return d;
}

function cardsRow(list, size, cls = '') {
  const r = el('div', `cards ${cls}`);
  list.forEach((c) => r.append(cardEl(c, size)));
  return r;
}

/** Opponent lands as pips: ▮ untapped, ▯ tapped. */
function landPips(l) {
  let s = '';
  for (let i = 0; i < l.total; i++) s += i < l.total - l.tapped ? '<i class="pip"></i>' : '<i class="pip tapped"></i>';
  return `<span class="pips">${s}</span> <span class="dim">${l.total - l.tapped}/${l.total}</span>`;
}

function stackList() {
  const d = el('div', 'stack-list');
  GAME.stack.forEach((s, i) => {
    d.append(el('div', 'stack-item' + (i === GAME.stack.length - 1 ? ' top' : ''),
      `<div class="s-who">${s.who} cast</div><div class="s-name">${s.n}</div><div class="s-text">${s.text}</div>`));
  });
  if (GAME.stack.length === 0) d.append(el('div', 'dim', 'empty'));
  return d;
}

function promptEl(compact = false) {
  const p = el('div', 'prompt' + (compact ? ' compact' : ''));
  p.append(el('div', 'p-sentence', GAME.prompt.sentence));
  const b = el('div', 'p-buttons');
  GAME.prompt.buttons.forEach((x) => {
    const btn = el('button', 'btn' + (x.primary ? ' primary' : '') + (x.lit ? ' lit' : ''), x.t + (x.key ? ` <kbd>${x.key}</kbd>` : ''));
    b.append(btn);
  });
  p.append(b);
  return p;
}

function logEl() {
  const d = el('div', 'log');
  GAME.log.forEach(([w, t]) => d.append(el('div', 'log-line', `<b>${w}</b> ${t}`)));
  return d;
}

function seatLine(p, extra = '') {
  return `<span class="seat-name">${p.name}</span>${p.active ? ' <span class="badge active">active</span>' : ''}${p.name === GAME.priority ? ' <span class="badge prio">priority</span>' : ''}
    <span class="counts">hand ${Array.isArray(p.hand) ? p.hand.length : p.hand} · lib ${p.lib} · gy ${p.gy}</span>${extra}<span class="life${p.life < 20 ? ' low' : ''}">${p.life}</span>`;
}
