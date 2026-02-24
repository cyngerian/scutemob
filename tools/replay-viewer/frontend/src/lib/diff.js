/**
 * diff.js — compare two StateViewModel objects and return a Set of changed paths.
 *
 * A "path" is a dot-separated string like:
 *   "players.p1.life"
 *   "zones.battlefield.p1"
 *   "zones.stack"
 *   "turn.step"
 *
 * Usage:
 *   import { diffState } from './diff.js';
 *   const changed = diffState(prevState, currentState);
 *   if (changed.has('zones.stack')) { ... }
 */

/**
 * Compare two StateViewModel objects.
 * Returns a Set<string> of changed paths at a coarse (section-level) granularity.
 *
 * @param {object|null} prev — previous StateViewModel
 * @param {object|null} curr — current StateViewModel
 * @returns {Set<string>} set of changed path strings
 */
export function diffState(prev, curr) {
  const changed = new Set();
  if (!prev || !curr) return changed;

  // Turn-level fields
  diffScalar(prev.turn?.number, curr.turn?.number, 'turn.number', changed);
  diffScalar(prev.turn?.active_player, curr.turn?.active_player, 'turn.active_player', changed);
  diffScalar(prev.turn?.phase, curr.turn?.phase, 'turn.phase', changed);
  diffScalar(prev.turn?.step, curr.turn?.step, 'turn.step', changed);
  diffScalar(prev.turn?.priority, curr.turn?.priority, 'turn.priority', changed);

  // Stack
  const prevStack = JSON.stringify(prev.zones?.stack);
  const currStack = JSON.stringify(curr.zones?.stack);
  if (prevStack !== currStack) {
    changed.add('zones.stack');
  }

  // Exile
  const prevExile = JSON.stringify(prev.zones?.exile);
  const currExile = JSON.stringify(curr.zones?.exile);
  if (prevExile !== currExile) {
    changed.add('zones.exile');
  }

  // Combat
  const prevCombat = JSON.stringify(prev.combat);
  const currCombat = JSON.stringify(curr.combat);
  if (prevCombat !== currCombat) {
    changed.add('combat');
  }

  // Per-player: life, poison, mana, hand, library, graveyard, commander damage
  const allPlayers = new Set([
    ...Object.keys(prev.players ?? {}),
    ...Object.keys(curr.players ?? {}),
  ]);

  for (const pname of allPlayers) {
    const pp = prev.players?.[pname];
    const cp = curr.players?.[pname];

    if (!pp || !cp) {
      changed.add(`players.${pname}`);
      continue;
    }

    diffScalar(pp.life, cp.life, `players.${pname}.life`, changed);
    diffScalar(pp.poison, cp.poison, `players.${pname}.poison`, changed);
    diffScalar(pp.hand_size, cp.hand_size, `players.${pname}.hand_size`, changed);
    diffScalar(pp.library_size, cp.library_size, `players.${pname}.library_size`, changed);
    diffScalar(pp.graveyard_size, cp.graveyard_size, `players.${pname}.graveyard_size`, changed);
    diffScalar(pp.land_plays_remaining, cp.land_plays_remaining, `players.${pname}.land_plays_remaining`, changed);
    diffScalar(pp.has_lost, cp.has_lost, `players.${pname}.has_lost`, changed);

    // Mana pool
    const prevMana = JSON.stringify(pp.mana_pool);
    const currMana = JSON.stringify(cp.mana_pool);
    if (prevMana !== currMana) {
      changed.add(`players.${pname}.mana_pool`);
    }

    // Commander damage
    const prevCmdr = JSON.stringify(pp.commander_damage_received);
    const currCmdr = JSON.stringify(cp.commander_damage_received);
    if (prevCmdr !== currCmdr) {
      changed.add(`players.${pname}.commander_damage`);
    }

    // Battlefield (compare as whole serialized blob per player)
    const prevBf = JSON.stringify(prev.zones?.battlefield?.[pname]);
    const currBf = JSON.stringify(curr.zones?.battlefield?.[pname]);
    if (prevBf !== currBf) {
      changed.add(`zones.battlefield.${pname}`);
    }

    // Hand
    const prevHand = prev.zones?.hand?.[pname]?.length;
    const currHand = curr.zones?.hand?.[pname]?.length;
    if (prevHand !== currHand) {
      changed.add(`zones.hand.${pname}`);
    }

    // Graveyard
    const prevGy = JSON.stringify(prev.zones?.graveyard?.[pname]);
    const currGy = JSON.stringify(curr.zones?.graveyard?.[pname]);
    if (prevGy !== currGy) {
      changed.add(`zones.graveyard.${pname}`);
    }
  }

  return changed;
}

/** Helper: add path to changed set if values differ. */
function diffScalar(a, b, path, changed) {
  if (a !== b) {
    changed.add(path);
  }
}
