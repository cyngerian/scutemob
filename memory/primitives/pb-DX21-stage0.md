# PB-DX21 — Stage 0 observations (verified at branch HEAD, before any edit)

**Batch**: PB-DX21 — CR 508.1: attackers may be declared without limit
**Seed**: `OOS-M11-9` · **Task**: `scutemob-200`
**Brief (authoritative)**: `memory/primitives/seed-rerank-2026-08-02.md` §4 row 3, lines 873-907.
**Plan**: `memory/primitives/pb-plan-DX21.md`

Baseline: **4,388 / 0 / 5** full-workspace, `--workspace --no-fail-fast` captured to a file
(43 `test result:` lines summed). Matches the criterion's stated pre-edit number exactly.

## The brief's three consequences all reproduce in source, unmodified

1. `combat.rs:743-747` — `combat.attackers.insert(*attacker_id, target.clone())` inside a `for`
   over the submitted vec. Declarations **accumulate**; a repeated same-id entry **overwrites
   that creature's `AttackTarget` mid-combat**.
2. `combat.rs:795-806` — `events.push(GameEvent::AttackersDeclared { .. })` then
   `abilities::check_triggers(state, &events)` + `flush_pending_triggers`. Every attack trigger
   **re-fires per declaration**.
3. `combat.rs:753-761` — `ps.attackers_declared_this_turn = attackers.len() as u32` (assignment,
   not `+=`; the in-source comment says "overwritten (not accumulated) on multi-combat turns").
   Read by `Condition::YouAttackedWithNOrMore` at `effects/mod.rs:10215`; live on **two**
   deck-legal defs — `legions_landing.rs:76` and `windbrisk_heights.rs:71`, both
   `YouAttackedWithNOrMore(3)`.

Blockers side is **covered and must not be widened**: `combat.rs:1103` →
`GameStateError::AlreadyDeclaredBlockers(player)` (`error.rs:64`), keyed on
`combat.defenders_declared` (`combat.rs:1652` inserts; `hash.rs:4430` hashes it).

## CORRECTION to the brief's preferred implementation

The brief says "**PREFER reading `combat.attackers`** over adding a field". That guard is **not
exact**, and the hole is reachable from the shipped browser path:

- An **empty** declaration is legal and is a real client action. `params.rs:474` maps a
  `LegalAction::DeclareAttackers` with default params to
  `Command::DeclareAttackers { attackers: vec![] }`, and `api.rs:299-301` documents it verbatim
  as "a legal, **irreversible** 'I attack with nothing' that the engine accepts silently".
- `combat.attackers` stays **empty** after such a declaration, so an
  `!combat.attackers.is_empty()` guard lets a player who declined to attack then attack anyway —
  contradicting `api.rs`'s own word *irreversible*, and CR 508.1 (declaring attackers is a
  once-per-combat turn-based action; declaring **none** is still declaring).
- Rejecting the empty declaration instead is explicitly ruled out by that same doc: "declaring no
  attackers is legal under CR 508.1 and rejecting it would deadlock a combat".

So the marker has to be a real one. `CombatState` gets a `bool`, hashed alongside its
`defenders_declared` sibling, and the HASH bump is **computed from the failing gate's own output**
(AC 6174 provides for exactly this branch). A field that decides legality but is not hashed would
break SR-9b's per-step fingerprint cross-validation.

Multi-combat is safe: `turn_actions.rs:2507` (`end_combat`) sets `state.combat = None` and
`turn_actions.rs:1897` / `combat.rs:71` rebuild a fresh `CombatState`, so an extra combat phase
starts with the marker clear.

## A FOURTH consequence the brief does not list

`combat.rs:818-820` — each accepted declaration resets `state.turn.players_passed = OrdSet::new()`
and re-grants priority to the declarer. A client that re-declares therefore also **resets the CR
117.4 pass-round**, so a combat can be held open indefinitely without any attacker ever changing.
This is the empty-declaration case's only consequence, and it is another reason the guard cannot
key on `combat.attackers`.
