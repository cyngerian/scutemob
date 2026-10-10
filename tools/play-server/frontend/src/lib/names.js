/**
 * names — human labels for the engine's enum spellings (`scutemob-260`).
 *
 * The wire carries `Step`, `Phase` and `DecisionKind` as their Rust variant
 * names (`PreCombatMain`, `BeginningOfCombat`, `DeclareAttackers`), and
 * `event_view.rs` formats a `StepChanged` line as `"{phase:?} — {step:?}"`. A
 * human reads "PreCombatMain — PreCombatMain" as a bug, and the first-playtest
 * feed was full of it. The replay viewer has the same maps in
 * `eventFormat.js`, but as module-private functions; they are repeated here
 * rather than exported from a step debugger whose formatting is its own
 * concern. The play surface wants shorter labels than the viewer's anyway
 * ("Main 1", matching `PhaseIndicator`'s pips, not "Main Phase 1").
 *
 * Every function falls back to a readable split of the input — `SomeNewStep`
 * → "Some New Step" — so a variant added later is never rendered as a blank
 * or thrown on; it just loses the hand-written label until one is added.
 */

const STEPS = {
  Untap: 'Untap',
  Upkeep: 'Upkeep',
  Draw: 'Draw',
  PreCombatMain: 'Main 1',
  BeginningOfCombat: 'Beginning of combat',
  DeclareAttackers: 'Declare attackers',
  DeclareBlockers: 'Declare blockers',
  FirstStrikeDamage: 'First-strike damage',
  CombatDamage: 'Combat damage',
  EndOfCombat: 'End of combat',
  PostCombatMain: 'Main 2',
  End: 'End step',
  Cleanup: 'Cleanup',
};

const PHASES = {
  Beginning: 'Beginning',
  PreCombatMain: 'Main 1',
  Combat: 'Combat',
  PostCombatMain: 'Main 2',
  Ending: 'Ending',
};

const DECISIONS = {
  Priority: 'Priority',
  Mulligan: 'Mulligan',
  CommanderZoneChoice: 'Commander zone',
  DeclareAttackers: 'Declare attackers',
  DeclareBlockers: 'Declare blockers',
  CleanupDiscard: 'Cleanup discard',
  TriggerTargets: 'Trigger targets',
  EffectChoice: 'Effect choice',
};

/** `PreCombatMain` → "Pre Combat Main"; `ALLCAPS` and non-strings pass through. */
export function splitCamel(s) {
  if (typeof s !== 'string' || s.length === 0) return s ?? '';
  return s.replace(/([a-z0-9])([A-Z])/g, '$1 $2');
}

export function stepName(step) {
  return STEPS[step] ?? splitCamel(step);
}

export function phaseName(phase) {
  return PHASES[phase] ?? splitCamel(phase);
}

export function decisionKindName(kind) {
  return DECISIONS[kind] ?? splitCamel(kind);
}

/**
 * The feed's per-line kind tag, lower-cased: `PriorityGiven` → "priority given".
 * Lower rather than title case because the tag is metadata under the sentence,
 * not a heading.
 */
export function eventKindName(kind) {
  return splitCamel(kind).toLowerCase();
}

/**
 * A `StepChanged` line's text, rebuilt from the enum names the server put in
 * it. `event_view.rs` writes exactly `"{phase:?} — {step:?}"`; anything that
 * does not match that shape is returned untouched, so a future server-side
 * rewording cannot be mangled here.
 *
 * "Main 1 — Main 1" collapses to "Main 1": the phase and the step are the same
 * word for both main phases and repeating it says nothing.
 */
export function stepChangedText(text) {
  if (typeof text !== 'string') return text;
  const m = /^([A-Za-z]+) — ([A-Za-z]+)$/.exec(text);
  if (!m) return text;
  const phase = phaseName(m[1]);
  const step = stepName(m[2]);
  return phase === step ? step : `${phase} — ${step}`;
}
