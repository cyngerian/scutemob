# End State — Pod Play

**Status**: DISCUSSION PIECE. Written 2026-08-15 at the owner's direction, from a
portfolio-level review. Items marked **OWNER RULING** are settled. Everything
else is a proposal — workstreams and oversight sessions are invited to contest,
refine, or refute it, and should record their position when they do.

---

## The ruling

**OWNER RULING (2026-08-15): scutemob's end state is playable matches with the
owner's 6-person pod.**

Not a rules-engine research vehicle, not an agent-protocol testbed — those are
means. The engine is done *enough* when the owner and five friends can sit down
and play real matches with their real decks.

Two corollaries, also owner-stated:

1. **Card authoring is expected to become the critical path** once the major
   engine pieces are in place — and to get substantially quicker at that point.
2. **The pod's decks will be checked into the repo** as authoring targets
   before matches happen.

## Why this doc exists

The seed queue is a closed loop: re-ranks generate seeds, seeds justify
re-ranks. Every individual workstream is rigorous, but nothing outside the
queue currently answers the question *"is the project closer to its end state
than it was last month?"* This doc names the end state so that question has a
referent.

## The metric this makes possible

Once the pod's six decklists are checked in, distance-to-end-state stops being
a feeling and becomes a number:

> **Pod coverage: X of the N distinct cards across the six pod decks are
> author-complete and engine-supported.**

This is proposed as the headline metric because it is finite, measurable,
demand-driven (the decks define exactly which cards matter — no card is
authored speculatively while pod coverage is incomplete), and it naturally
prioritizes engine gaps: a card that *cannot* be authored because the engine
lacks a mechanic is a ranked engine seed with a named customer.

Proposed mechanics:

- A `decks/` (or similar) directory holding the six decklists in a parseable
  format. Format to be decided (see open questions).
- A coverage report — in the spirit of `authoring-status.md` — scoped to the
  union of pod-deck cards, regenerated per handoff.
- Engine seeds gain an optional "pod blocker" annotation: which deck(s) and
  card(s) they unblock. Seeds with pod blockers argue for rank from that;
  seeds without one argue from something else, explicitly.

## Proposed process amendments

These plug into machinery that already exists (eot/oversight handoffs,
re-ranks, /review). None are settled; each can be adopted, adapted, or refused
with a recorded reason.

1. **Operator-visible delta line.** Every oversight/eot handoff includes one
   required line: *what can a player observe now that they could not at the
   last handoff?* Two consecutive empty entries force a pod-facing workstream
   (play session, coverage push, or a pod-blocker engine seed) to the front of
   the queue.
2. **Play sessions as queue input.** On some cadence, the owner (or owner +
   subset of pod) actually plays — even partial games, even hot-seat. Defects
   found in play jump the queue. This is the ss1/emustack "proven means
   operated" rule imported: a test count is evidence about the code; a played
   turn is evidence about the game.
3. **Net-negative re-ranks.** A re-rank (v5 onward) must retire at least as
   many seeds as it admits, with explicit won't-do rows. Open-seed count is
   reported per handoff; if it trends up while pod coverage is flat, the
   process is feeding itself and the handoff should say so.
4. **Releases in player terms.** Every K workstreams, cut a tag whose notes are
   written for the pod ("equip costs display correctly", "Saga chapters
   resolve through layers"), not in OOS-registry vocabulary. An empty release
   note is a smell-test firing — cheap and unambiguous.

## Open questions for discussion

Agents encountering these in the course of a workstream should record findings
or positions here (or in a successor doc) rather than resolving them silently:

- **Format**: what does the pod actually play — Commander, or another
  multiplayer format? What does 6-player multiplayer require of the engine
  (turn order, multiplayer-specific rules, range-of-influence: none assumed)
  that current 2-player-oriented work does not cover? This may be the largest
  unranked engine surface and deserves a census seed of its own.
- **Play surface**: what do matches physically run on — the tauri-app, hot-seat
  on one machine, networked clients? What is the *minimum* surface for match
  one (hot-seat is a legitimate answer)?
- **Deck check-in format**: plain decklist text, an existing interchange
  format, or a repo-native schema? Who validates a checked-in deck against
  card-name canon?
- **Adjudication tolerance**: for early pod matches, is manual override /
  judge-mode acceptable where the engine is incomplete? A "playable with
  known holes + a judge button" milestone may be reachable far sooner than
  full-rules completeness, and the pod may prefer it.
- **Authoring throughput**: what is the current cards-per-workstream rate, and
  what rate would pod coverage need? The claim "authoring gets quicker once
  major pieces land" is owner-stated and plausible — it should eventually be
  *measured* against the pod-coverage burn-down.

## What this doc is not

It does not re-rank anything, close any seed, or override any standing
protocol rule. It adds an external referent. The queue remains the queue — it
now has something to answer to.
