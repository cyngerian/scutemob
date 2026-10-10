/**
 * prompt — the one sentence the prompt panel shows (`docs/play-surface-v2-brief.md` §3).
 *
 * Composed client-side from `decision.kind`, `state.turn`, and whether the decision
 * offers anything beyond pass / concede / tap-for-mana. The server's own `prompt`
 * (a blocking decision's question) wins when there is one: it is the engine's words
 * for a real choice, and nothing here should paraphrase a scry.
 *
 * Returns `{ text, tone }` where `tone` is one of `'yours' | 'theirs' | 'choice' |
 * 'over' | 'idle'` and only picks a colour.
 */
import { decisionKindName, stepName } from '../names.js';

const QUIET_KINDS = new Set(['PassPriority', 'Concede', 'TapForMana']);

function isMainStep(step) {
  return step === 'PreCombatMain' || step === 'PostCombatMain';
}

export function composePrompt({
  decision = null,
  turn = null,
  humanName = null,
  gameOver = null,
  pregame = false,
  loading = false,
} = {}) {
  if (gameOver) {
    if (gameOver.halted) return { text: `Game halted — ${gameOver.reason ?? 'see the log'}.`, tone: 'over' };
    return {
      text: gameOver.winner ? `Game over — ${gameOver.winner} wins.` : 'Game over — no winner.',
      tone: 'over',
    };
  }
  if (pregame) return { text: 'Keep this hand?', tone: 'choice' };

  const active = turn?.active_player ?? null;
  const step = stepName(turn?.step ?? '');
  const mine = active !== null && active === humanName;

  if (!decision) {
    if (loading) return { text: 'Working…', tone: 'idle' };
    if (active) return { text: `${mine ? 'Your' : `${active}'s`} ${step.toLowerCase()}. Waiting for the bots…`, tone: 'idle' };
    return { text: 'No game is running. Open the menu and start one.', tone: 'idle' };
  }

  const actions = decision.actions ?? [];
  const serverPrompt = actions.find((a) => a.decision?.prompt)?.decision?.prompt ?? null;

  switch (decision.kind) {
    case 'Priority': {
      const castable = actions.some((a) => !QUIET_KINDS.has(a.kind));
      if (mine && isMainStep(turn?.step)) {
        return {
          text: castable
            ? 'Your main phase. Play a land or cast a spell, or pass.'
            : 'Your main phase. Nothing left to play — pass.',
          tone: 'yours',
        };
      }
      if (mine) {
        return {
          text: castable ? `Your ${step.toLowerCase()}. You could act, or pass.` : `Your ${step.toLowerCase()}. Nothing to do here — pass.`,
          tone: 'yours',
        };
      }
      return {
        text: castable
          ? `${active}'s ${step.toLowerCase()}. You could respond, or pass.`
          : `${active}'s ${step.toLowerCase()}. You have nothing to cast — pass.`,
        tone: 'theirs',
      };
    }
    case 'DeclareAttackers':
      return { text: 'Declare attackers.', tone: 'choice' };
    case 'DeclareBlockers':
      return { text: `${active ?? 'The attacker'} is attacking. Declare blockers.`, tone: 'choice' };
    case 'CleanupDiscard':
      return { text: serverPrompt ?? 'Discard down to your hand size.', tone: 'choice' };
    case 'Mulligan':
      return { text: 'Keep this hand?', tone: 'choice' };
    case 'CommanderZoneChoice':
      return { text: serverPrompt ?? 'Your commander is leaving the battlefield. Send it to the command zone?', tone: 'choice' };
    default:
      return { text: serverPrompt ?? `${decisionKindName(decision.kind)}.`, tone: 'choice' };
  }
}

/**
 * Actions the lit cards cannot stand for: not quiet, and either carrying no
 * `object_id` or carrying one that no rendered card has.
 *
 * The second half is what makes the surface never dead-end (`scutemob-263`
 * probe finding): a dungeon room's scry arrived as an `AnswerEffectChoice`
 * with `object_id: 0` — the redacted placeholder id, since a room is not a
 * card — so no chip was lit and nothing was offered. `renderedIds` is every
 * id the surface currently draws (hand, battlefields, command zones); an
 * action outside that set gets a button, whatever its id says.
 */
export function looseActions(decision, renderedIds = null) {
  return (decision?.actions ?? []).filter((a) => {
    if (QUIET_KINDS.has(a.kind)) return false;
    if (!answerable(a)) return false;
    if (a.object_id === null || a.object_id === undefined) return true;
    return !renderedIds || !renderedIds.has(Number(a.object_id));
  });
}

/**
 * Can this action be carried through from the client at all?
 *
 * The server offers an activation whose target requirement has no legal
 * candidate (Equip with no creature on your side, B1 probe at turn 21): the
 * picker then shows "no legal target" with Confirm disabled, and Back is the
 * only way out. Lighting the chip for that invites a dead click, so an action
 * whose every target slot is empty while it still demands a target is not
 * offered by this surface. Conservative on purpose: anything with at least one
 * candidate somewhere, or with no target requirement, is left alone.
 */
export function answerable(a) {
  const slots = a?.target_slots ?? [];
  const min = a?.target_min ?? 0;
  if (min <= 0 || slots.length === 0) return true;
  return slots.some((s) => (s.candidates?.length ?? 0) > 0);
}

export { QUIET_KINDS };
