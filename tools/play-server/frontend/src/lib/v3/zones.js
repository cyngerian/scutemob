/**
 * zones — one seat's piles out of the seat view (`docs/play-surface-v3-brief.md` §1 rule 4).
 *
 * Pure functions: `ZoneBox` (the 2×2 box on a centre board) and `ZoneStrip`
 * (the 1×5 strip on a side cell) draw the same five piles from here, and each
 * pile carries what its popover lists. Nothing is filtered (Invariant 7): a
 * hidden hand entry stays in the list and is drawn as a back.
 *
 * Exile is per player (`zones.exile_by_owner`, C3 `scutemob-268`): each seat's
 * pile is its OWN exiled cards, newest on top as the view orders them. A
 * face-down exiled card this seat may not identify arrives `hidden: true` and
 * is a back. Against a server that does not send the map, the pile falls back
 * to the one shared list and is labelled `EX*` with a note, as C1 drew it.
 *
 * Commander tax (`players[name].commander_tax`, same task) is a CAST COUNT per
 * commander name; the next cast from the command zone costs `{2 × n}` more
 * (CR 903.8). The command pile carries one `+2n` badge per commander with
 * n > 0 — whether or not that commander is in the command zone right now.
 */

/** "W2 G1" for a non-empty mana pool, else ''. The pool is public (CR 106.4). */
export function poolText(player) {
  const p = player?.mana_pool ?? {};
  return [['W', p.white], ['U', p.blue], ['B', p.black], ['R', p.red], ['G', p.green], ['C', p.colorless]]
    .filter(([, n]) => n > 0)
    .map(([k, n]) => `${k}${n}`)
    .join(' ');
}

/** 'conceded' / 'out' for a seat no longer in the game, else null. */
export function seatOut(player) {
  return player?.has_conceded ? 'conceded' : player?.has_lost ? 'out' : null;
}

export const EXILE_NOTE = 'shared by all players (this server sends no per-player exile)';

/**
 * The commanders of a seat and what the next cast of each costs extra:
 * `[{ name, casts, extra, inZone }]`, every key of `commander_tax` (0 included),
 * in name order. `extra` is `2 × casts` generic mana (CR 903.8).
 */
export function commanderTaxes(player, command) {
  const tax = player?.commander_tax;
  if (!tax || typeof tax !== 'object') return [];
  const inZone = new Set((command ?? []).filter((c) => !c?.hidden).map((c) => c.name));
  return Object.keys(tax)
    .sort()
    .map((name) => {
      const casts = Number(tax[name]) || 0;
      return { name, casts, extra: casts * 2, inZone: inZone.has(name) };
    });
}

/** "Name: cast 2× — next cast costs 4 more" per taxed commander, for the pile's popover. */
function taxNote(taxes) {
  return taxes
    .filter((t) => t.casts > 0)
    .map((t) => `${t.name}: cast ${t.casts}× from here — next cast costs ${t.extra} more${t.inZone ? '' : ' (not in the command zone now)'}`)
    .join('; ');
}

const BACK = { hidden: true };

/**
 * The five piles of `name`: `{ cmd, hand, lib, gy, ex }`, each `{ label, top, count, cz, title, note, cards }`.
 * `cmd` also carries `taxes` (`commanderTaxes`).
 */
export function seatPiles(name, player, zones) {
  const taxes = commanderTaxes(player, zones?.command_zone?.[name] ?? []);
  // Every key of `commander_tax` is a commander, so a card of that name in the
  // command zone gets the ★ the battlefield copy carries (`CardInZoneView` has no flag of its own).
  const commanders = new Set(taxes.map((t) => t.name));
  const command = (zones?.command_zone?.[name] ?? []).map((c) => (c && !c.hidden && commanders.has(c.name) ? { ...c, is_commander: true } : c));
  const hand = zones?.hand?.[name] ?? [];
  const graveyard = zones?.graveyard?.[name] ?? [];
  const perPlayer = zones?.exile_by_owner && typeof zones.exile_by_owner === 'object' ? zones.exile_by_owner : null;
  const exile = perPlayer ? (perPlayer[name] ?? []) : (zones?.exile ?? []);
  const tax = taxNote(taxes);
  const handSize = player?.hand_size ?? hand.length;
  const librarySize = player?.library_size ?? 0;
  const revealed = hand.filter((c) => !c.hidden).length;
  return {
    cmd: {
      label: 'CMD',
      top: command[0] ?? null,
      count: null,
      cz: true,
      title: `${name}'s command zone`,
      note: [command.length ? '' : 'commander is not in the command zone', tax].filter(Boolean).join(' · '),
      cards: command,
      taxes,
    },
    hand: {
      label: 'HAND',
      top: handSize > 0 ? BACK : null,
      count: handSize,
      cz: false,
      title: `${name}'s hand`,
      note: revealed ? `${revealed} revealed` : 'all hidden',
      cards: hand,
    },
    lib: {
      label: 'LIB',
      top: librarySize > 0 ? BACK : null,
      count: librarySize,
      cz: false,
      title: `${name}'s library`,
      note: `hidden — ${librarySize} cards`,
      cards: [],
    },
    gy: {
      label: 'GY',
      top: graveyard[0] ?? null,
      count: graveyard.length,
      cz: false,
      title: `${name}'s graveyard`,
      note: '',
      cards: graveyard,
    },
    ex: {
      label: perPlayer ? 'EX' : 'EX*',
      top: exile[0] ?? null,
      count: exile.length,
      cz: false,
      title: perPlayer ? `${name}'s exile` : 'Exile',
      note: perPlayer ? '' : EXILE_NOTE,
      cards: exile,
    },
  };
}

/**
 * Where a peeked cell goes (rule 14): double the cell's size, capped at half
 * the viewport's width and its height less a margin, anchored on the cell's
 * own corner (its right edge for a right-hand cell) and kept on screen.
 * `rect` is the cell's `getBoundingClientRect()`.
 */
export function peekStyle(rect) {
  if (!rect) return { style: '', fromRight: false };
  const fromRight = rect.left > innerWidth / 2;
  const w = Math.min(rect.width * 2, innerWidth * 0.5);
  const h = Math.min(rect.height * 2, innerHeight - 16);
  const top = Math.max(8, Math.min(rect.top, innerHeight - h - 8));
  const right = rect.right ?? rect.left + rect.width;
  const left = Math.max(8, Math.min(fromRight ? right - w : rect.left, innerWidth - w - 8));
  return { style: `width: ${w}px; height: ${h}px; top: ${top}px; left: ${left}px;`, fromRight };
}
