/**
 * eventFormat.js — convert serialized GameEvent objects to human-readable strings.
 *
 * Each GameEvent variant is serialized by serde as either:
 *   - A plain string for unit variants (e.g., "AllPlayersPassed")
 *   - An object with a single key matching the variant name (e.g., { "TurnStarted": { player, turn_number } })
 *
 * `formatEvent(event)` returns a concise human-readable description.
 */

/**
 * Format a serialized GameEvent into a human-readable string.
 * @param {string|object} event — raw event from the API
 * @returns {string}
 */
export function formatEvent(event) {
  if (!event) return '(null event)';

  // Unit variants: plain string
  if (typeof event === 'string') {
    switch (event) {
      case 'AllPlayersPassed': return 'All players passed priority';
      case 'ManaPoolsEmptied': return 'Mana pools emptied';
      case 'CleanupPerformed': return 'Cleanup performed';
      case 'DamageCleared': return 'Damage cleared from permanents';
      case 'CombatEnded': return 'Combat ended';
      default: return event;
    }
  }

  // Object variants: { VariantName: { ...fields } }
  if (typeof event === 'object') {
    const [key] = Object.keys(event);
    const data = event[key];

    switch (key) {
      case 'TurnStarted':
        return `Turn ${data.turn_number} started for ${data.player}`;

      case 'StepChanged':
        return `Step changed: ${formatPhase(data.phase)} / ${formatStep(data.step)}`;

      case 'PriorityGiven':
        return `Priority given to ${data.player}`;

      case 'PriorityPassed':
        return `${data.player} passed priority`;

      case 'PermanentsUntapped': {
        const count = data.objects?.length ?? 0;
        return `${data.player} untapped ${count} permanent${count !== 1 ? 's' : ''}`;
      }

      case 'CardDrawn':
        return `${data.player} drew a card`;

      case 'DiscardedToHandSize':
        return `${data.player} discarded to hand size`;

      case 'PlayerLost':
        return `${data.player} lost the game (${formatLossReason(data.reason)})`;

      case 'PlayerConceded':
        return `${data.player} conceded`;

      case 'GameOver':
        return data.winner
          ? `Game over — winner: ${data.winner}`
          : 'Game over — draw';

      case 'ExtraTurnAdded':
        return `${data.player} gets an extra turn`;

      case 'LandPlayed':
        return `${data.player} played a land`;

      case 'ManaAdded':
        return `${data.player} added ${data.amount} ${formatManaColor(data.color)} mana`;

      case 'PermanentTapped':
        return `Permanent ${data.object_id} tapped (${data.player})`;

      case 'PermanentUntapped':
        return `Permanent ${data.object_id} untapped (${data.player})`;

      case 'SpellCast':
        return `${data.player} cast a spell (stack id: ${data.stack_object_id})`;

      case 'SpellResolved':
        return `Spell resolved for ${data.player}`;

      case 'PermanentEnteredBattlefield':
        return `Permanent ${data.object_id} entered the battlefield (${data.player})`;

      case 'SpellCountered':
        return `Spell countered for ${data.player}`;

      case 'SpellFizzled':
        return `Spell fizzled (all targets illegal) for ${data.player}`;

      case 'ManaCostPaid':
        return `${data.player} paid mana cost`;

      case 'AbilityActivated':
        return `${data.player} activated ability (source: ${data.source_object_id})`;

      case 'AbilityTriggered':
        return `Triggered ability from ${data.source_object_id} (controller: ${data.controller})`;

      case 'AbilityResolved':
        return `Ability resolved (controller: ${data.controller})`;

      case 'CreatureDied':
        return `Creature ${data.object_id} died`;

      case 'PlaneswalkerDied':
        return `Planeswalker ${data.object_id} died`;

      case 'AuraFellOff':
        return `Aura ${data.object_id} fell off`;

      case 'EquipmentUnattached':
        return `Equipment ${data.object_id} unattached`;

      case 'TokenCeasedToExist':
        return `Token ${data.object_id} ceased to exist`;

      case 'CountersAnnihilated':
        return `${data.amount} counter pair(s) annihilated on ${data.object_id}`;

      case 'LegendaryRuleApplied':
        return `Legendary rule applied — kept ${data.kept_id}`;

      case 'AttackersDeclared': {
        const count = data.attackers?.length ?? 0;
        return `${data.attacking_player} declared ${count} attacker${count !== 1 ? 's' : ''}`;
      }

      case 'BlockersDeclared': {
        const count = data.blockers?.length ?? 0;
        return `${data.defending_player} declared ${count} blocker${count !== 1 ? 's' : ''}`;
      }

      case 'CombatDamageDealt': {
        const total = (data.assignments ?? []).reduce((s, a) => s + (a.amount ?? 0), 0);
        return `Combat damage dealt (${total} total damage in ${data.assignments?.length ?? 0} assignment${data.assignments?.length !== 1 ? 's' : ''})`;
      }

      case 'LifeGained':
        return `${data.player} gained ${data.amount} life`;

      case 'LifeLost':
        return `${data.player} lost ${data.amount} life`;

      case 'DamageDealt': {
        const targetStr = formatDamageTarget(data.target);
        return `${data.amount} damage dealt to ${targetStr} (source: ${data.source})`;
      }

      case 'ObjectExiled':
        return `Object ${data.object_id} exiled (${data.player})`;

      case 'PermanentDestroyed':
        return `Permanent ${data.object_id} destroyed`;

      case 'CardDiscarded':
        return `${data.player} discarded a card`;

      case 'CardMilled':
        return `${data.player} milled a card`;

      case 'TokenCreated':
        return `Token created for ${data.player} (id: ${data.object_id})`;

      case 'LibraryShuffled':
        return `${data.player}'s library shuffled`;

      case 'CounterAdded':
        return `${data.count} ${formatCounter(data.counter)} counter(s) added to ${data.object_id}`;

      case 'CounterRemoved':
        return `${data.count} ${formatCounter(data.counter)} counter(s) removed from ${data.object_id}`;

      case 'ObjectReturnedToHand':
        return `Object ${data.object_id} returned to ${data.player}'s hand`;

      case 'ObjectPutInGraveyard':
        return `Object ${data.object_id} put in graveyard (${data.player})`;

      case 'ObjectPutOnLibrary':
        return `Object ${data.object_id} put on ${data.player}'s library`;

      case 'CommanderZoneRedirect':
        return `Commander ${data.object_id} redirected to command zone (${data.owner})`;

      case 'ReplacementEffectApplied':
        return `Replacement effect applied: ${data.description}`;

      case 'ReplacementChoiceRequired':
        return `${data.player} must choose replacement effect order`;

      case 'DamagePrevented':
        return `${data.prevented} damage prevented (${data.remaining} got through)`;

      case 'CommanderCastFromCommandZone':
        return `${data.player} cast their commander from the command zone (tax: ${data.tax_paid})`;

      case 'CommanderReturnedToCommandZone':
        return `Commander returned to command zone (${data.owner}) from ${formatZoneType(data.from_zone)}`;

      case 'CommanderZoneReturnChoiceRequired':
        return `${data.owner} must choose: return commander to command zone or leave it`;

      case 'MulliganTaken':
        return `${data.player} took mulligan #${data.mulligan_number}${data.is_free ? ' (free)' : ''}`;

      case 'MulliganKept':
        return `${data.player} kept their hand${data.cards_to_bottom?.length > 0 ? ` (${data.cards_to_bottom.length} to bottom)` : ''}`;

      case 'CompanionBroughtToHand':
        return `${data.player} brought companion to hand`;

      case 'CascadeExiled':
        return `${data.player} exiled ${data.cards_exiled?.length ?? 0} card(s) for cascade`;

      case 'CascadeCast':
        return `${data.player} cast card ${data.card_id} for free via cascade`;

      case 'SpellCopied':
        return `Spell copied (original: ${data.original_stack_id}, copy: ${data.copy_stack_id}, controller: ${data.controller})`;

      case 'LoopDetected':
        return `Infinite loop detected: ${data.description}`;

      case 'Scried':
        return `${data.player} scried ${data.count}`;

      case 'Goaded':
        return `Permanent ${data.object_id} goaded by ${data.goading_player}`;

      default:
        return `${key}: ${JSON.stringify(data)}`;
    }
  }

  return JSON.stringify(event);
}

/** Format a Phase enum value from the engine. */
function formatPhase(phase) {
  const map = {
    Beginning: 'Beginning',
    PreCombatMain: 'Pre-Combat Main',
    Combat: 'Combat',
    PostCombatMain: 'Post-Combat Main',
    Ending: 'Ending',
  };
  return map[phase] ?? phase;
}

/** Format a Step enum value from the engine. */
function formatStep(step) {
  const map = {
    Untap: 'Untap',
    Upkeep: 'Upkeep',
    Draw: 'Draw',
    PreCombatMain: 'Main Phase 1',
    BeginningOfCombat: 'Beginning of Combat',
    DeclareAttackers: 'Declare Attackers',
    DeclareBlockers: 'Declare Blockers',
    CombatDamage: 'Combat Damage',
    EndOfCombat: 'End of Combat',
    PostCombatMain: 'Main Phase 2',
    End: 'End Step',
    Cleanup: 'Cleanup',
  };
  return map[step] ?? step;
}

/** Format a LossReason enum value. */
function formatLossReason(reason) {
  const map = {
    LifeTotal: 'life total 0 or less',
    LibraryEmpty: 'drew from empty library',
    PoisonCounters: '10+ poison counters',
    CommanderDamage: '21+ commander damage',
    Conceded: 'conceded',
  };
  if (typeof reason === 'string') return map[reason] ?? reason;
  if (typeof reason === 'object') {
    const [k] = Object.keys(reason);
    return map[k] ?? k;
  }
  return String(reason);
}

/** Format a ManaColor enum value. */
function formatManaColor(color) {
  if (typeof color === 'string') {
    const map = {
      White: 'W',
      Blue: 'U',
      Black: 'B',
      Red: 'R',
      Green: 'G',
      Colorless: 'C',
    };
    return map[color] ?? color;
  }
  return String(color);
}

/** Format a CombatDamageTarget (Player/Creature/Planeswalker). */
function formatDamageTarget(target) {
  if (!target) return '(unknown)';
  if (typeof target === 'string') return target;
  if (typeof target === 'object') {
    if ('Player' in target) return `player ${target.Player}`;
    if ('Creature' in target) return `creature ${target.Creature}`;
    if ('Planeswalker' in target) return `planeswalker ${target.Planeswalker}`;
  }
  return JSON.stringify(target);
}

/** Format a CounterType enum value. */
function formatCounter(counter) {
  if (typeof counter === 'string') return counter;
  if (typeof counter === 'object') {
    const [k, v] = Object.entries(counter)[0];
    if (v && typeof v === 'object') return `${k}(${JSON.stringify(v)})`;
    return k;
  }
  return String(counter);
}

/** Format a ZoneType enum value. */
function formatZoneType(zone) {
  const map = {
    Hand: 'hand',
    Library: 'library',
    Graveyard: 'graveyard',
    Battlefield: 'battlefield',
    Stack: 'stack',
    Exile: 'exile',
    CommandZone: 'command zone',
  };
  if (typeof zone === 'string') return map[zone] ?? zone;
  return String(zone);
}

/**
 * Get the variant name from a serialized GameEvent.
 * Returns the string itself for unit variants.
 * @param {string|object} event
 * @returns {string}
 */
export function eventVariantName(event) {
  if (typeof event === 'string') return event;
  if (typeof event === 'object') {
    const [k] = Object.keys(event);
    return k;
  }
  return '?';
}

/**
 * Get the "category" of an event for color-coding.
 * Returns one of: 'turn', 'priority', 'cast', 'damage', 'life', 'zone', 'combat', 'commander', 'system'
 * @param {string|object} event
 * @returns {string}
 */
export function eventCategory(event) {
  const name = eventVariantName(event);
  switch (name) {
    case 'TurnStarted':
    case 'StepChanged':
    case 'PermanentsUntapped':
    case 'ManaPoolsEmptied':
    case 'CleanupPerformed':
    case 'DamageCleared':
      return 'turn';

    case 'PriorityGiven':
    case 'PriorityPassed':
    case 'AllPlayersPassed':
      return 'priority';

    case 'SpellCast':
    case 'SpellResolved':
    case 'SpellCountered':
    case 'SpellFizzled':
    case 'SpellCopied':
    case 'ManaCostPaid':
    case 'AbilityActivated':
    case 'AbilityTriggered':
    case 'AbilityResolved':
    case 'CascadeExiled':
    case 'CascadeCast':
      return 'cast';

    case 'DamageDealt':
    case 'DamagePrevented':
    case 'CombatDamageDealt':
      return 'damage';

    case 'LifeGained':
    case 'LifeLost':
    case 'PlayerLost':
    case 'PlayerConceded':
    case 'GameOver':
      return 'life';

    case 'CardDrawn':
    case 'CardDiscarded':
    case 'CardMilled':
    case 'LandPlayed':
    case 'ObjectExiled':
    case 'ObjectReturnedToHand':
    case 'ObjectPutInGraveyard':
    case 'ObjectPutOnLibrary':
    case 'PermanentEnteredBattlefield':
    case 'PermanentDestroyed':
    case 'CreatureDied':
    case 'PlaneswalkerDied':
    case 'AuraFellOff':
    case 'EquipmentUnattached':
    case 'TokenCreated':
    case 'TokenCeasedToExist':
    case 'LibraryShuffled':
    case 'CounterAdded':
    case 'CounterRemoved':
    case 'CountersAnnihilated':
    case 'LegendaryRuleApplied':
    case 'DiscardedToHandSize':
      return 'zone';

    case 'AttackersDeclared':
    case 'BlockersDeclared':
    case 'CombatEnded':
    case 'PermanentTapped':
    case 'PermanentUntapped':
      return 'combat';

    case 'CommanderCastFromCommandZone':
    case 'CommanderReturnedToCommandZone':
    case 'CommanderZoneRedirect':
    case 'CommanderZoneReturnChoiceRequired':
    case 'MulliganTaken':
    case 'MulliganKept':
    case 'CompanionBroughtToHand':
      return 'commander';

    case 'ManaAdded':
      return 'mana';

    case 'ReplacementEffectApplied':
    case 'ReplacementChoiceRequired':
    case 'LoopDetected':
    case 'Scried':
    case 'Goaded':
    case 'ExtraTurnAdded':
      return 'system';

    default:
      return 'system';
  }
}
