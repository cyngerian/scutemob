# PB-DX36 — execution notes

**Task**: `scutemob-228` · v4 queue rank 13 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 13)
**Seed**: `OOS-CARDS2-6` (both halves) — **had no registry row before this batch**; grepped
(`docs/audits/decision-point-audit.md`) and confirmed absent 2026-09-04, then filed.

---

## §0 — Stage 0: measured facts, taken BEFORE any production line changed

Everything in this section was executed on the merge base (`e7d7ae31`) with the tree otherwise
untouched. The wire cells are **probed, not inherited from the memo**.

### §0.1 Pre-edit baseline

`cargo test --workspace --no-fail-fast` to a file:

* **5,097 passed / 0 failed / 5 ignored**, **63** result-producing targets.
* This **reproduces PB-DX35's published close pin exactly** (5,097 / 0 / 5, 63 targets).
  `OOS-DX51-5`'s "the close pin does not reproduce" failure mode did **not** recur here.

### §0.2 Wire probe — executed, not argued

Method: temporarily extend each gate's `CLOSURE_MUST_NOT_CONTAIN` with the candidate type names
and run the closure gate; a type that **is** in the closure fails the assertion by name. The
`MIN_CLOSURE_TYPES` floor was raised to 9999 separately to read each closure's live type count off
the failure message (PB-DX51's technique).

| type | PROTOCOL closure (`Command`/`GameEvent`) | HASH closure (`GameState` serde) |
|---|---|---|
| `TriggerCondition` | **absent** (probe passed) | absent by construction — reachable only through `#[serde(skip)] card_registry` → `CardDefinition`, which `CLOSURE_MUST_NOT_CONTAIN` pins out |
| `PendingTrigger` | **absent** (probe passed) | **present** (`CLOSURE_MUST_CONTAIN` entry) |
| `EffectAmount` | **PRESENT** (probe fired) | present (via `Effect`, a `CLOSURE_MUST_CONTAIN` entry) |
| `TriggerEvent` | **PRESENT** (probe fired) | present (via `Characteristics.triggered_abilities`) |
| `TriggeredAbilityDef` | **PRESENT** (probe fired) | present (same edge) |

Live type counts at the merge base: **PROTOCOL closure = 98**, **HASH closure = 132**.
Current versions: **PROTOCOL 41**, **HASH 82**.

The v4 memo's row-13 wire cell said *"`TriggerCondition` is **off-wire**, which the v3 row had
backwards"*. **Confirmed by execution.** `protocol.rs`'s own v25/v26 correction paragraphs say the
same thing and are also confirmed.

### §0.3 Wire PREDICTION (written before any production line changed)

**PROTOCOL 41 → 42 — ONE bump. HASH 82 → 83 — ONE bump.
Both closure type counts UNCHANGED: PROTOCOL 98, HASH 132.**

Reasoning **per half**, so a gate that moves differently falsifies a stated claim rather than a
vague expectation:

| change | PROTOCOL | HASH | why |
|---|---|---|---|
| `TriggerCondition::WhenDealsDamage { recipient }` — new variant | no | no *(declaration)* | `TriggerCondition` is in neither closure (§0.2). The **state stream** for a game holding such a card does change, which is what the version number exists to record — but no *declaration* fingerprint moves for this alone. |
| `TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer` gains `recipient` | no | no *(declaration)* | same edge |
| new `DamageRecipient` enum | **no** | **no** | reachable only from `TriggerCondition`; deliberately NOT placed on `TriggerEvent`, so it enters neither closure and neither type count moves |
| `TriggerEvent` — 7 new unit variants, 1 removed | **YES** | **YES** | `TriggerEvent` is in both closures (§0.2). Unit variants add no new *type*, so both counts stay put. |
| `EffectAmount::DamageDealt` — new variant | **YES** | **YES** | `EffectAmount` is in both closures. Unit variant → no count change. |
| `PendingTrigger.damage_dealt_amount: u32` | no | **YES** | `PendingTrigger` is a HASH `CLOSURE_MUST_CONTAIN` entry and absent from the PROTOCOL closure (§0.2). `u32` is already in both closures → no count change. |
| `EffectContext.damage_dealt_amount` | no | no | `EffectContext` is a transient resolution-time struct: not serialized into `GameState`, not a wire frame. |

**Stop condition**: if either gate moves in a way this table does not explain, or does not move at
all, stop and re-derive rather than re-pinning.

### §0.4 Coverage prediction

**ONE flip, named before regeneration: `exalted_angel` `partial` → `Complete`.**
1,138 → **1,139 / 1,803 = 63.2%**.

Nothing else flips, and the reasons are stated per def rather than asserted collectively:

* `sigil_of_sleep` — already `Complete` (marker-less, by derive). Repaired in place; stays `Complete`.
* `curiosity` — the `recipient` axis closes its *"an opponent"* approximation, but its printed
  **"you may draw a card"** is a **costless optional effect**, and no such expression exists in the
  DSL at HEAD (`Effect::MayPayThenEffect` needs a `Cost`; a `{0}` cost was rejected as dishonest by
  PB-DX35; `Effect::MayPayOrElse` discards its cost — `OOS-DX48-2`; `Effect::Choose` is
  non-interactive). Stays `partial`, marker rewritten to name only the surviving blocker.
* `ophidian_eye` — same, plus its own second deviation. Stays `partial`.
* `goblin_lackey` — trigger condition repaired (blocker (c) of its own marker discharged); blockers
  (a) filtered hand→battlefield put and (b) the costless "may" survive. Stays `partial`.
* `warren_instigator` — printed trigger is now *expressible as a condition* and still has no
  expressible **effect** and no costless "may". Marker rewritten, stays `partial`.
* `breath_of_fury` — the printed combat-only Aura member. Its blocker is Aura re-attachment, not
  the trigger condition. Untouched, stays `partial`.

### §0.5 Design decisions taken at stage 0, with the measurement behind each

**(a) The recipient axis lives on `TriggerEvent` (unit variants), NOT on `TriggeredAbilityDef`.**
Measured: `TriggeredAbilityDef` has **no `Default` derive** and **190 exhaustive struct literals
across 44 files** — reproducing `OOS-DX35-1`'s figure exactly at HEAD. That is the cost that seed
declined to pay and this batch declines it for the same reason. `TriggerEvent` is a unit enum
matched by equality in `collect_triggers_for_event`, and the tree already carries this exact idiom
(`EquippedCreatureDealsCombatDamageToPlayer` beside `EquippedCreatureDealsCombatDamage`).

**(b) `combat_only` is KEPT and animated, not deleted — because a printed corpus member needs it.**
The temptation is to delete a flag only the hasher reads. But an inverse-method oracle scan finds
**`breath_of_fury`** — *"When enchanted creature deals **combat** damage to a player…"* — a real
`combat_only: true` member that simply does not declare the condition today (its blocker is Aura
re-attachment). Deleting the flag would make that card permanently over-fire on noncombat damage.
**Declared users of `combat_only: true` at HEAD: 0. Printed users: 1.** A census over the declared
axis alone would have said "delete it".

**(c) One ability lowers to exactly ONE `trigger_on`, which is what makes the two arms disjoint.**
The lowering is an exhaustive `match (combat_only, recipient)` (and `match recipient` for the self
family) with **no wildcard arm**, so a new axis value is a compile error rather than a silent drop —
the failure mode `combat_only` itself is. The combat-damage arm fires the `…CombatDamage…` events
**and** the `…AnyDamage…` events; the `GameEvent::DamageDealt` arm fires only the `…AnyDamage…`
events. A combat damage event therefore fires any one ability **exactly once**, because that
ability's single `trigger_on` value matches at most one of the events fired. This is the property
PB-DX47's double-push defect violated, and it is asserted by COUNT (`>= 1` would pass on the broken
shape).

**(d) `EffectAmount::DamageDealt` is a NEW variant, not a rename of `CombatDamageDealt`.**
The two are not redundant: `CombatDamageDealt` reads `ctx.combat_damage_amount`, which is **0** on a
noncombat trigger; `DamageDealt` reads `ctx.damage_dealt_amount`, CR 603.10a's *"that much"* for the
triggering damage event whichever kind it was. They agree on a combat-damage trigger by
construction and disagree on a noncombat one — pinned by probe rather than asserted. The rejected
alternative (rename `CombatDamageDealt` → `DamageDealt` and generalise the storage) was costed:
**75 occurrences of `combat_damage_amount` across 37 files**.

**(e) `damaged_player` is REUSED, not duplicated.** Its name is already accurate for both damage
kinds; only its doc said "combat". `TargetController::DamagedPlayer` (which `sigil_of_sleep`'s
target filter uses) reads it, so the noncombat arm must populate it or the repair would ship a
trigger with no legal target space.
