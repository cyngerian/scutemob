/**
 * layout — how the seat view's lists become the table (`docs/play-surface-v3-brief.md` §1).
 *
 * Pure functions over `StateViewModel` pieces. Nothing here filters hidden
 * information (Invariant 7): a hidden hand card arrives as `{hidden: true}`
 * and is rendered as a back because that is what the server sent.
 */

const TYPE = (p) => p?.card_types ?? [];

/** Split one player's battlefield into the three rows. */
export function splitBattlefield(permanents = []) {
  const creatures = [];
  const lands = [];
  const others = [];
  for (const p of permanents) {
    const t = TYPE(p);
    if (t.includes('Creature')) creatures.push(p);
    else if (t.includes('Land')) lands.push(p);
    else others.push(p);
  }
  return { creatures, lands, others };
}

/**
 * Same-name lands stacked by the fungibility key the earlier surfaces used
 * (G13): name, tapped, counters, attachment, commander, token.
 */
export function landStacks(lands = []) {
  const byKey = new Map();
  for (const p of lands) {
    const counters = Object.entries(p.counters ?? {})
      .filter(([, n]) => n > 0)
      .sort()
      .map(([ct, n]) => `${ct}=${n}`)
      .join(',');
    const key = JSON.stringify([p.name, !!p.tapped, counters, p.attached_to ?? null, !!p.is_commander, !!p.is_token]);
    if (byKey.has(key)) byKey.get(key).push(p);
    else byKey.set(key, [p]);
  }
  return [...byKey.entries()].map(([key, members]) => ({ key, members }));
}

/**
 * Seat order: the human first, then the others in the server's player order.
 * Player maps are keyed by name; the summary carries no explicit seat list, so
 * sorted names are the stable order every surface has used.
 */
export function seatOrder(playerNames = [], humanName = null) {
  const others = playerNames.filter((p) => p !== humanName).sort();
  return humanName ? [humanName, ...others] : others;
}

/**
 * Side columns: opponents in seat order go left, right, left, right … so six
 * seats put three on the left and two on the right (mockup rule 3). The
 * focused opponent keeps its cell.
 */
export function sideColumns(opponents = []) {
  const left = [];
  const right = [];
  opponents.forEach((name, i) => (i % 2 === 0 ? left : right).push(name));
  return { left, right };
}

/**
 * Who sits across the table: the last focus if it is still an opponent, else
 * the active player if that is an opponent, else the first opponent.
 */
export function pickFocus(current, opponents, activePlayer) {
  if (current && opponents.includes(current)) return current;
  if (activePlayer && opponents.includes(activePlayer)) return activePlayer;
  return opponents[0] ?? null;
}

/** `player:<name>` / `planeswalker:<name>` → a defender key for the combat card. */
export function defenderOf(target) {
  if (typeof target !== 'string') return { kind: 'unknown', name: String(target ?? '?') };
  const i = target.indexOf(':');
  if (i < 0) return { kind: 'player', name: target };
  return { kind: target.slice(0, i), name: target.slice(i + 1) };
}

/** Attackers grouped by the player (or planeswalker) they attack, in first-seen order. */
export function combatGroups(combat) {
  if (!combat) return [];
  const groups = new Map();
  for (const a of combat.attackers ?? []) {
    const d = defenderOf(a.target);
    const key = `${d.kind}:${d.name}`;
    if (!groups.has(key)) groups.set(key, { key, defender: d, attackers: [] });
    groups.get(key).attackers.push(a);
  }
  return [...groups.values()];
}

/** Every permanent on the table by object id, for P/T lookups off a name+id pair. */
export function permanentIndex(battlefield = {}) {
  const idx = new Map();
  for (const list of Object.values(battlefield)) for (const p of list) idx.set(Number(p.object_id), p);
  return idx;
}

/** The ordered steps of a turn, as the wire spells them, for the phase track. */
export const STEP_ORDER = [
  'Untap',
  'Upkeep',
  'Draw',
  'PreCombatMain',
  'BeginningOfCombat',
  'DeclareAttackers',
  'DeclareBlockers',
  'FirstStrikeDamage',
  'CombatDamage',
  'EndOfCombat',
  'PostCombatMain',
  'End',
  'Cleanup',
];

export const SHORT_STEP = {
  Untap: 'Untap',
  Upkeep: 'Upkeep',
  Draw: 'Draw',
  PreCombatMain: 'Main 1',
  BeginningOfCombat: 'Begin combat',
  DeclareAttackers: 'Attackers',
  DeclareBlockers: 'Blockers',
  FirstStrikeDamage: 'First strike',
  CombatDamage: 'Damage',
  EndOfCombat: 'End combat',
  PostCombatMain: 'Main 2',
  End: 'End',
  Cleanup: 'Cleanup',
};

export const COMBAT_STEPS = new Set(['BeginningOfCombat', 'DeclareAttackers', 'DeclareBlockers', 'FirstStrikeDamage', 'CombatDamage', 'EndOfCombat']);
