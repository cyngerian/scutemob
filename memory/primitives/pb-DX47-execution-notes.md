# PB-DX47 — execution notes

**Task**: `scutemob-218`. v4 queue rank 5 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 5).
**Seed**: `OOS-DX24-4` — "is the `WhenDealsCombatDamageToPlayer` double-push real?"
**Branch**: `feat/pb-dx47-probe-first-is-the-whendealscombatdamagetoplayer-dou`

---

## §0 — Wire prediction, written BEFORE any code changed

**PROTOCOL 39 / HASH 78, both UNMOVED.** Confidence HIGH, and the reason is stated rather
than asserted: whichever way the probe decides, the repair is a *suppression* inside
`rules/abilities.rs`'s `GameEvent::CombatDamageDealt` arm — it adds no type, no variant and
no field to the `Command`/`GameEvent`/`Effect`/`Characteristics` closure, and it changes no
hashed field's *declaration*. `PendingTrigger` is hashed, but the fix changes how many are
*produced*, not what one *is*.

Gate-computed result: recorded in §6.

---

## §1 — The experiment ran FIRST, and it is decisive

`crates/simulator/tests/pb_dx47_double_push_probe.rs`, committed **before any fix**.

### Fixture (why it is what it is)

* **Production pregame path.** The state is built by `mtg_simulator::setup::build_initial_state`
  — the same function `tools/play-server`'s `session::new_game` and `tools/tui` build through
  — not by `GameStateBuilder`. This is not fussiness: the in-source comment the seed is filed
  against claims the runtime lowering "only happens in `enrich_spec_from_def` for tests", so a
  hand-built fixture is exactly the shape the false claim says is special. Proving anything on
  one would prove nothing.
* **Subject `drana_liberator_of_malakir`** — `Complete` by derive, deck-legal, and
  **legendary**, so CR 903.6 puts it in the command zone by construction rather than leaving
  the probe dependent on a shuffle. Its trigger puts a `+1/+1` counter on each attacking
  creature you control, so a double dispatch is visible as **two counters on a lone attacker**,
  not merely as two stack entries.
* **Both seats human** (`human_seats = {p1, p2}`), so no bot RNG enters: every decision in the
  game is made by the probe's own `choose()`.
* Deck: the subject as commander + 99 `Swamp`. Basic lands are exempt from CR 903.5b's
  singleton rule and mono-black satisfies CR 903.4, so the real `validate_deck` gate
  (Architecture Invariant 9, run inside `build_initial_state`) admits it.

### Result — **the double-push is REAL**

```
PB-DX47 P1: subject=Drana, Liberator of Malakir lowered(A)=1 registry(B)=1
PB-DX47 P2: PendingTrigger census by kind = {"CardDefETB": 1, "Normal": 1} (total 2);
            +1/+1 counters on the lone attacker = 2; commands = 126
```

* **P1** — on the object `setup.rs` actually built, **both** dispatch preconditions hold:
  the runtime lowering produced exactly one `TriggeredAbilityDef` with
  `trigger_on == TriggerEvent::SelfDealsCombatDamageToPlayer` (path A), and the card-registry
  def carries exactly one `AbilityDefinition::Triggered { WhenDealsCombatDamageToPlayer }`
  (path B). If the justifying comment were true, `lowered(A)` would be `0`. It is `1`.
* **P2** — the engine's own `check_triggers`, called on the REAL driven state at the moment
  the subject was a declared attacker, pushes **two** `PendingTrigger`s for one event: one
  `PendingTriggerKind::Normal` (the runtime lowering, via `collect_triggers_for_event`) and one
  `PendingTriggerKind::CardDefETB` (the registry scan in the same arm). **No dedup exists.**
  End to end, a card printing ONE `+1/+1` counter put **TWO** on its lone attacker.

### A measurement that returned 0 for the wrong reason — recorded, not dropped

The probe's first draft censused `state.pending_triggers()` after every `advance()`/`submit()`
and measured **0 at every command boundary**. Not because nothing was pushed: because
`check_and_flush_triggers` drains the queue onto the stack inside the *same* `process_command`
call, so `pending_triggers` is never non-empty at any point a test can observe.

**A census that returns 0 because it never got to look is indistinguishable from one that
returns 0 because nothing happened.** What caught it was the end-to-end assertion running
beside it (`counters == 2`) — the census said "nothing" while the board said "twice". The
shipped census therefore calls the engine's own dispatcher directly. Filed as `OOS-DX47-1`.

---

## §2 — Why nobody noticed (the comment is false in TWO ways)

`crates/engine/src/rules/abilities.rs`, the `GameEvent::CombatDamageDealt` arm:

> "CardDef-level `WhenDealsCombatDamageToPlayer` triggers from `AbilityDefinition::Triggered`.
> These are not converted to runtime `TriggeredAbilityDef` (that only happens in
> `enrich_spec_from_def` for tests), so we collect them here from the card registry."

Both halves are false at HEAD:

1. **"not converted to runtime `TriggeredAbilityDef`"** — `build_face_ability_vectors`
   (`testing/replay_harness.rs`) has a dedicated loop that converts exactly this
   `TriggerCondition` into a `TriggeredAbilityDef { trigger_on:
   TriggerEvent::SelfDealsCombatDamageToPlayer, .. }`. PB-DX1 even *extended* that loop
   (`intervening_if` propagation) without anyone reconciling it with this comment.
2. **"only happens in `enrich_spec_from_def` for tests"** — `enrich_spec_from_def` is the
   **production pregame path**: `setup.rs:419/433/440` (commander, opening hand, library) and
   `fuzz_setup.rs:119/130`. Every object in every real game goes through it.

The same false sentence is copy-pasted onto the `WhenExertedAsAttacks` arm
(`abilities.rs:4290`), which cites this arm as its precedent — so the claim propagated.

