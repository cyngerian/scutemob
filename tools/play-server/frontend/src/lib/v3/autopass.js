/**
 * autopass — the auto-pass panel's rule, as pure functions (`scutemob-267`, play surface v3 C2;
 * `docs/play-surface-v3-brief.md` §1 rule 12).
 *
 * The panel (`AutoPass.svelte`) states a rule — "pass until X, unless Y" — and
 * `stores.js::startPassUntil` runs the loop. This module turns the panel's
 * settings into the `(view, ctx) => string | null` predicate that loop consults
 * after every pass, and composes the sentence that states the rule. Nothing
 * here touches a store or the DOM, so plain `node` can exercise it against
 * fabricated seat views.
 *
 * # Two kinds of stop, and why the predicate reports which
 *
 *   - **until** — the point the mode names was reached (the turn ended, your
 *     next turn began, the chosen step came up). The run is finished.
 *   - **unless** — a stop-condition fired on the way there (a spell was cast,
 *     an ability was activated, attackers were declared at you). The run is
 *     *interrupted*: the panel pauses and offers Resume.
 *
 * The loop's contract only carries a reason string, so the predicate also calls
 * `onStop({ stop: 'until' | 'unless', reason })` and the panel keeps that.
 *
 * # Everything is read from the seat-redacted view (Invariant 7)
 *
 * The "unless" checks read `view.events` — only the lines since the previous
 * response, which is exactly "what happened because of the pass just made" —
 * and `view.state.combat`. Nothing is inferred beyond what those say. The stop
 * step as a server parameter is C3 (brief §5).
 */
import { STEP_ORDER, SHORT_STEP } from './layout.js';
import { stepName } from '../names.js';

/** The panel's modes, in the mockup's order. `off` is the absence of a run. */
export const MODES = [
  ['off', 'Off'],
  ['eot', 'Until end of turn'],
  ['mynext', 'Until my next turn'],
  ['step', 'Until step…'],
];

/** The stop-conditions, in the mockup's order and wording. */
export const STOPS = [
  ['spell', 'stop if a spell is cast'],
  ['ability', 'stop if an ability is activated'],
  ['combat', 'stop when attackers are declared at me'],
];

/**
 * The steps a run can be told to stop at.
 *
 * A run only ever stops where this seat is handed a `Priority` decision, so a
 * step nobody receives priority in is not a place it can stop:
 *
 *   - `Untap` — no player receives priority (CR 502.4);
 *   - `Cleanup` — no player receives priority unless a state-based action or a
 *     trigger happens there (CR 514.3), so "until Cleanup" would nearly always
 *     run into the stall guard instead of stopping.
 *
 * `FirstStrikeDamage` stays: it only exists in a combat with a first or double
 * striker (CR 510.4), but when it exists priority is given in it.
 */
export const STOPPABLE_STEPS = STEP_ORDER.filter((s) => s !== 'Untap' && s !== 'Cleanup');

/** Mockup defaults: spell on, ability off, combat on, step = Main 2. */
export const DEFAULT_SETTINGS = { step: 'PostCombatMain', spell: true, ability: false, combat: true };

/** Coerce whatever `localStorage` held into a complete, valid settings object. */
export function normaliseSettings(raw) {
  const out = { ...DEFAULT_SETTINGS };
  if (raw && typeof raw === 'object') {
    if (STOPPABLE_STEPS.includes(raw.step)) out.step = raw.step;
    for (const [key] of STOPS) if (typeof raw[key] === 'boolean') out[key] = raw[key];
  }
  return out;
}

/** What each mode passes *until*, as the mockup words it. */
export function untilPhrase(mode, settings) {
  if (mode === 'eot') return 'end of this turn';
  if (mode === 'mynext') return 'your next turn';
  if (mode === 'step') return `the ${SHORT_STEP[settings?.step] ?? settings?.step} step`;
  return null;
}

/**
 * The sentence stating the rule, composed as the mockup composes it:
 * "Passing automatically until your next turn, unless a spell is cast or
 * attackers are declared at you." / "Off — every priority stops here."
 *
 * "An ability is activated" means *activated* (CR 602): a triggered ability
 * going on the stack does not stop a run, and neither does a mana ability.
 */
export function ruleSentence(mode, settings) {
  const until = untilPhrase(mode, settings);
  if (!until) return 'Off — every priority stops here.';
  const unless = [
    settings?.spell && 'a spell is cast',
    settings?.ability && 'an ability is activated',
    settings?.combat && 'attackers are declared at you',
  ].filter(Boolean);
  return `Passing automatically until ${until}${unless.length ? ', unless ' + unless.join(' or ') : ''}.`;
}

// ── the "unless" checks ──────────────────────────────────────────────────────

/**
 * A spell was cast by somebody else: a `SpellCast` line whose `player` is not
 * this seat. Returns the line's own sentence ("Bot-2 casts Lightning Bolt").
 */
export function spellCastBy(view, humanName) {
  const line = (view?.events ?? []).find((ev) => ev?.kind === 'SpellCast' && ev.player !== humanName);
  return line ? line.text || 'a spell was cast' : null;
}

/**
 * The event kinds that are "an ability was activated".
 *
 *   - `AbilityActivated` — every non-mana activated ability put on the stack
 *     (CR 602), which in this engine includes loyalty abilities and the keyword
 *     activations (cycling, level up, …): `rules/engine.rs` and
 *     `rules/abilities.rs` emit this one event for all of them.
 *   - `CraftActivated` — the one activation with an event of its own.
 *
 * Deliberately absent: **mana abilities** (they emit `ManaAdded` /
 * `PermanentTapped` and never this — a bot tapping lands must not stop a run)
 * and **triggered abilities** (`AbilityTriggered`: triggered, not activated).
 */
export const ACTIVATION_KINDS = new Set(['AbilityActivated', 'CraftActivated']);

/**
 * An ability was activated by somebody else. A line with no `player` at all
 * (`CraftActivated` renders kind-only) cannot be attributed, and stops the run:
 * stopping is the safe direction for a line nobody can vouch for.
 */
export function abilityActivatedBy(view, humanName) {
  const line = (view?.events ?? []).find((ev) => ACTIVATION_KINDS.has(ev?.kind) && ev.player !== humanName);
  return line ? line.text || 'an ability was activated' : null;
}

/**
 * The attackers currently declared at this seat: those whose `target` is
 * `player:<human>`, or `planeswalker:<name>` naming a planeswalker on this
 * seat's battlefield (the view spells a planeswalker target by name only, so a
 * same-named planeswalker elsewhere also matches — the cautious reading).
 */
export function attackersAtMe(view, humanName) {
  const attackers = view?.state?.combat?.attackers ?? [];
  if (attackers.length === 0 || !humanName) return [];
  const mine = view?.state?.zones?.battlefield?.[humanName] ?? [];
  const walkers = new Set(mine.filter((p) => (p.card_types ?? []).includes('Planeswalker')).map((p) => p.name));
  return attackers.filter((a) => {
    if (typeof a?.target !== 'string') return false;
    if (a.target === `player:${humanName}`) return true;
    return a.target.startsWith('planeswalker:') && walkers.has(a.target.slice('planeswalker:'.length));
  });
}

/**
 * Attackers were declared at this seat **since the previous look**. `seen` is
 * the set of attacker ids already known to be coming, and is updated in place:
 * the same attack must not stop a resumed run a second time.
 */
export function newAttackersAtMe(view, humanName, seen) {
  const fresh = [];
  const now = new Set();
  for (const a of attackersAtMe(view, humanName)) {
    const id = Number(a.object_id);
    now.add(id);
    if (!seen.has(id)) fresh.push(a);
  }
  seen.clear();
  for (const id of now) seen.add(id);
  if (fresh.length === 0) return null;
  const names = fresh.map((a) => a.name || 'a creature');
  const who = view?.state?.combat?.attacking_player;
  return `attackers are declared at you (${who ? who + ': ' : ''}${names.join(', ')})`;
}

// ── the "until" checks ───────────────────────────────────────────────────────

/** `{ number, active, step }` off a seat view's turn. */
export function turnFacts(view) {
  const turn = view?.state?.turn ?? {};
  return { number: turn.number ?? null, active: turn.active_player ?? null, step: turn.step ?? null };
}

const whose = (active, humanName) => (active === humanName ? 'your' : `${active ?? 'the next player'}'s`);

/**
 * Has the mode's "until" point been reached on `view`? All three compare
 * against where the run **started**, never against the previous look:
 *
 *   - `eot` — the turn number moved on;
 *   - `mynext` — this seat is the active player in a *different* turn, so a run
 *     started on your own turn goes round the table to your next one (the older
 *     `'my-turn'` predicate stops after one pass there);
 *   - `step` — the turn is at the chosen step, and it is not the very
 *     (turn, step) the run started in: any player's next occurrence of it.
 */
export function untilReached(mode, settings, start, view, humanName) {
  const now = turnFacts(view);
  if (now.number === null) return null;
  if (mode === 'eot') {
    return now.number !== start.number ? `the turn ended (turn ${now.number} is ${whose(now.active, humanName)})` : null;
  }
  if (mode === 'mynext') {
    return now.active === humanName && now.number !== start.number ? `your turn ${now.number} began` : null;
  }
  if (mode === 'step') {
    const at = now.step === settings?.step;
    const same = now.number === start.number && now.step === start.step;
    return at && !same ? `reached ${whose(now.active, humanName)} ${stepName(now.step)} (turn ${now.number})` : null;
  }
  return null;
}

/**
 * Build the predicate for one run.
 *
 * `settings` is read **live** on every call (the panel's checkboxes and step
 * can change while a run is going, and the rule sentence on screen always
 * shows the current ones — the run must follow what the sentence says). `mode`
 * is fixed for the run. `start` is the seat view the run starts from: it gives
 * the turn the "until" checks compare against and the attackers already
 * declared, so an attack the human has already seen does not stop the run.
 *
 * `anchor` (`turnFacts` of an earlier view) replaces the first of those when a
 * paused run is **resumed**: "until end of turn" keeps meaning the turn the
 * human chose it in, not the turn they pressed Resume in.
 *
 * "Until" is checked first: when the run has arrived, it is over, and a spell
 * cast on the same response is added to the reason rather than turning an
 * arrival into a pause.
 */
export function buildPredicate(mode, settings, start, onStop = null, anchor = null) {
  const humanName = start?.summary?.human_name ?? null;
  const startFacts = anchor ?? turnFacts(start);
  const seen = new Set(attackersAtMe(start, humanName).map((a) => Number(a.object_id)));

  return (view) => {
    // Always folded in, whether or not the condition is on, so switching it on
    // mid-run does not fire for an attack declared several passes ago.
    const attack = newAttackersAtMe(view, humanName, seen);
    const unless =
      (settings?.spell && spellCastBy(view, humanName)) ||
      (settings?.ability && abilityActivatedBy(view, humanName)) ||
      (settings?.combat && attack) ||
      null;

    const until = untilReached(mode, settings, startFacts, view, humanName);
    if (until) {
      const reason = unless ? `${until}; ${unless}` : until;
      onStop?.({ stop: 'until', reason });
      return reason;
    }
    if (unless) {
      onStop?.({ stop: 'unless', reason: unless });
      return unless;
    }
    return null;
  };
}

// ── the same rule, for the server (C3, `scutemob-268`) ───────────────────────

/** The panel's mode → the wire's `until.kind` (`src/pass_until.rs::Until`). */
const SERVER_UNTIL = { eot: 'end_of_turn', mynext: 'my_next_turn', step: 'step' };

/**
 * The `pass_until` object `POST /api/game/action` takes: the rule the panel
 * states, in the server's words (`tools/play-server/README.md` §"Pass-until as
 * a server parameter"). The server then makes the whole run inside one request
 * and reports `{ passes, stop, reason }` — `buildPredicate` above is the very
 * rule it ported, and stays the fallback for a server that does not know it.
 *
 * Three things differ from the predicate, all forced by "one request":
 *
 *   - `settings` is a **snapshot** (a box ticked while the request is in flight
 *     cannot reach it; the panel locks the boxes for that moment);
 *   - the panel's `combat` box is the wire's `attackers` flag (sending `combat`
 *     is a 400);
 *   - `anchor` is sent only for a **resumed** run — `{ turn, step }` of where
 *     the mode was chosen (`turnFacts`), so "until end of turn" keeps meaning
 *     that turn. A fresh run is measured from the state the request finds.
 *
 * Returns null for anything the wire cannot say (an unknown mode, a step
 * priority is never given in); the caller then runs the client loop.
 */
export function buildServerRule(mode, settings, anchor = null) {
  const kind = SERVER_UNTIL[mode];
  if (!kind) return null;
  const until = { kind };
  if (mode === 'step') {
    if (!STOPPABLE_STEPS.includes(settings?.step)) return null;
    until.step = settings.step;
  }
  const rule = {
    until,
    unless: { spell: settings?.spell === true, ability: settings?.ability === true, attackers: settings?.combat === true },
  };
  if (anchor && Number.isInteger(anchor.number) && anchor.number >= 0 && typeof anchor.step === 'string') {
    rule.anchor = { turn: anchor.number, step: anchor.step };
  }
  return rule;
}
