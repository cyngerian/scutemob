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

/** The in-page hover card: one fixed overlay (never clipped by a board's overflow),
 *  the `normal` 488px printing scaled to HOVER_W, placed beside the hovered card. */
const HOVER_W = 240;
let hoverEl = null;
function showHover(name, r) {
  if (!hoverEl) { hoverEl = el('div', 'hover-card'); hoverEl.append(el('img')); document.body.append(hoverEl); }
  const img = hoverEl.firstChild; const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  img.onerror = () => { if (!img.dataset.fb) { img.dataset.fb = '1'; img.src = SCRY(name, 'normal'); } };
  img.dataset.fb = ''; img.src = `../img/${slug}-normal.jpg`;
  const H = HOVER_W * 1.4;
  let x = r.right + 10; if (x + HOVER_W > innerWidth - 8) x = r.left - HOVER_W - 10; if (x < 8) x = 8;
  let y = r.top + r.height / 2 - H / 2; y = Math.max(8, Math.min(innerHeight - H - 8, y));
  hoverEl.style.left = x + 'px'; hoverEl.style.top = y + 'px'; hoverEl.style.display = 'block';
}
function hideHover() { if (hoverEl) hoverEl.style.display = 'none'; }

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

// ---- six-player late game (mockup 7's design state) ----------------------
const POOL = {
  creatures: ['Goblin Guide', 'Neriv, Heart of the Storm', 'Llanowar Elves', 'Beast Whisperer', 'Toski, Bearer of Secrets', 'Ayara, First of Locthwain', 'Gravecrawler', 'Bloodghast', 'Thalia, Guardian of Thraben', 'Adeline, Resplendent Cathar', 'Luminarch Aspirant', 'Seasoned Hallowblade', 'Archon of Emeria', 'Benalish Marshal', 'Dusk Legion Duelist', 'Sun Titan'],
  others: ['Sol Ring', 'Lightning Greaves', 'Sylvan Library', 'Phyrexian Arena', 'Arcane Signet', 'Swiftfoot Boots', 'Thought Vessel', 'Marble Diamond', 'Hedron Archive', 'Smothering Tithe', "Elspeth, Sun's Champion"],
  spells: ['Swords to Plowshares', 'Path to Exile', 'Wrath of God', 'Beast Within'],
  lands: ['Plains', 'Forest', 'Command Tower', 'Reliquary Tower'],
};
const PT = { 'Goblin Guide': '2/2', 'Neriv, Heart of the Storm': '4/4', 'Llanowar Elves': '1/1', 'Beast Whisperer': '2/3', 'Toski, Bearer of Secrets': '1/1', 'Ayara, First of Locthwain': '2/3', 'Gravecrawler': '2/1', 'Bloodghast': '2/1', 'Thalia, Guardian of Thraben': '2/1', 'Adeline, Resplendent Cathar': '4/4', 'Luminarch Aspirant': '2/2', 'Seasoned Hallowblade': '3/1', 'Archon of Emeria': '2/3', 'Benalish Marshal': '3/3', 'Dusk Legion Duelist': '2/2', 'Sun Titan': '6/6' };
function pick(pool, n, seed) {
  // deterministic spread so each seat looks different
  const out = []; for (let i = 0; i < n; i++) { const name = pool[(i * 7 + seed * 3) % pool.length]; out.push({ n: name, pt: PT[name], tapped: (i * 5 + seed) % 4 === 0 }); } return out;
}
function seat(name, commander, life, o, seed) {
  const creatures = pick(POOL.creatures, o.creatures, seed); if (!o.inCommandZone) creatures[0] = { n: commander, pt: PT[commander] || '4/4', cmd: true };
  return {
    name, commander, life, hand: o.hand, lib: o.lib ?? 60 - seed * 4, gy: o.gy, active: !!o.active,
    creatures, others: pick(POOL.others, o.others, seed + 1),
    lands: { total: o.lands[0], tapped: o.lands[1] },
    handCards: Array.from({ length: o.hand }, (_, i) => (o.revealed && i < o.revealed.length) ? { n: o.revealed[i], revealed: true } : { back: true }),
    libTop: o.libTop || null, inCommandZone: !!o.inCommandZone, commanderTax: o.commanderTax || 0,
    gyCards: pick([...POOL.creatures, ...POOL.spells, ...POOL.others, ...POOL.lands], o.gy, seed + 2).map((c) => ({ n: c.n })),
    exileCards: pick([...POOL.spells, ...POOL.creatures], o.ex, seed + 3).map((c) => ({ n: c.n })),
  };
}
function sixPlayerLateGame() {
  const g = cloneGame();
  g.turn = 23; g.step = 'Blockers'; g.active = 'Bot-3'; g.priority = 'Human-1';
  g.opps = [
    seat('Bot-2', 'Neriv, Heart of the Storm', 38, { creatures: 16, others: 5, lands: [9, 3], hand: 7, revealed: ['Goblin Guide', 'Sol Ring'], gy: 14, ex: 3 }, 1),
    seat('Bot-3', 'Toski, Bearer of Secrets', 40, { creatures: 14, others: 7, lands: [11, 8], hand: 6, gy: 12, ex: 1, libTop: 'Forest', active: true }, 2),
    seat('Bot-4', 'Ayara, First of Locthwain', 34, { creatures: 11, others: 4, lands: [10, 10], hand: 4, gy: 19, ex: 2, inCommandZone: true, commanderTax: 2 }, 3),
    seat('Bot-5', "Atraxa, Praetors' Voice", 29, { creatures: 18, others: 6, lands: [12, 2], hand: 3, gy: 9, ex: 6 }, 4),
    seat('Bot-6', 'Korvold, Fae-Cursed King', 17, { creatures: 9, others: 8, lands: [8, 0], hand: 8, revealed: ['Beast Within'], gy: 22, ex: 0, inCommandZone: true, commanderTax: 4 }, 5),
  ];
  const y = g.you;
  y.creatures = pick(POOL.creatures, 17, 0); y.creatures[0] = { n: 'Nadaar, Selfless Paladin', pt: '3/3', cmd: true }; y.creatures[3].targeted = true;
  y.others = pick(POOL.others, 8, 6);
  y.lands = [{ n: 'Plains', count: 9, tapped: 4 }, { n: 'Command Tower' }, { n: 'Reliquary Tower', tapped: true }, { n: 'Forest', count: 2 }];
  y.hand = [{ n: 'Swords to Plowshares', lit: true }, { n: 'Path to Exile', lit: true }, { n: 'Sun Titan' }, { n: 'Plains' }, { n: 'Wrath of God' }, { n: 'Beast Within', lit: true }, { n: 'Plains' }, { n: 'Smothering Tithe' }, { n: 'Archon of Emeria' }, { n: 'Forest' }, { n: 'Sol Ring' }, { n: 'Elspeth, Sun\'s Champion' }];
  y.gyCards = pick([...POOL.creatures, ...POOL.spells], 11, 7).map((c) => ({ n: c.n })); y.exileCards = pick(POOL.creatures, 4, 8).map((c) => ({ n: c.n }));
  y.lib = 61; y.libTop = null;
  g.stack = [{ who: 'Bot-3', n: 'Beast Within', text: 'targeting your Luminarch Aspirant' }, { who: 'Bot-5', n: 'Swords to Plowshares', text: 'targeting Bot-3\'s Beast Whisperer' }];
  g.prompt = { sentence: 'Bot-5 cast Swords to Plowshares targeting Beast Whisperer, in response to Beast Within. Respond, or let it resolve.', buttons: [{ t: 'Let it resolve', key: 'space', primary: true }, { t: 'Swords to Plowshares → …', lit: true }, { t: 'Path to Exile → …', lit: true }, { t: 'Beast Within → …', lit: true }] };
  return g;
}
STATES[5] = () => ({ label: 'Six players · turn 23 · late game everywhere · two spells on the stack', game: sixPlayerLateGame(), focus: 'Bot-3' });

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
  if (c.back) { d.classList.add('back'); d.title = c.n || 'hidden card'; if (c.count) d.append(el('div', 'count', `×${c.count}`)); return d; }
  if (c.tapped) d.classList.add('tapped');
  if (c.lit) d.classList.add('lit');
  if (c.targeted) d.classList.add('targeted');
  if (c.cmd) d.classList.add('cmd');
  d.title = c.n;
  d.dataset.name = c.n;
  d.addEventListener('mouseenter', () => { showHover(c.n, d.getBoundingClientRect()); broadcastHover(c.n); });
  d.addEventListener('mouseleave', hideHover);
  const chip = el('div', 'chip', `<div class="chip-name">${c.n}</div>`);
  const img = el('img', 'art');
  const v = size === 'hand' || size === 'big' ? 'normal' : 'small';
  // Local copy first (fetch-images.py), Scryfall once as the fallback.
  img.src = `../img/${c.n.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')}-${v}.jpg`;
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
