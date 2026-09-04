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

### §0.6 Two corrections to the task brief, made at stage 0

**(i) The brief's CR cite for *"that much"* is wrong.** Both the task description and acceptance
criterion 7333 say *"a damage-dealt `EffectAmount` ('that much', **CR 603.10a** amount from the
triggering event)"*. **CR 603.10a is about look-back-in-time ZONE-CHANGE triggers** — verbatim:
*"Some zone-change triggers look back in time. These are leaves-the-battlefield abilities,
abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object
that all players can see is put into a hand or library."* It has nothing to do with a damage
amount. The rules actually in play are **CR 603.2c** (*"An ability triggers only once each time its
trigger event occurs"* — the exactly-once property this batch asserts by COUNT) and
**CR 608.2h** with **CR 113.7a** (information at resolution / last known information — why the
amount is captured onto the `PendingTrigger` at queue time rather than re-read at resolution, which
is the idiom `EffectAmount::CombatDamageDealt` already uses). Every doc comment shipped by this
batch cites those; **no line in this batch cites CR 603.10a**, and the brief is recorded as
mis-cited rather than obeyed.

**(ii) The disjointness claim is verified at the EMIT sites, not assumed.** The two arms are
disjoint only if combat damage never produces a `GameEvent::DamageDealt`. Enumerated at HEAD:
`GameEvent::CombatDamageDealt` is emitted at exactly **one** site (`rules/combat.rs:2382`), and it
emits no `DamageDealt`. `GameEvent::DamageDealt` is emitted at exactly **five** sites —
`effects/mod.rs:1443` and `:1462` (CR 120.3a/120.3b inside `execute_effect_inner`), `:1619`
(same function), `:8950` (`deal_creature_power_damage`, the fight/ping path) and
`rules/mana.rs:610` (CR 605 pain-land damage) — **all five noncombat by construction**. So a combat
damage event reaches only the combat arm and a noncombat one only the `DamageDealt` arm.

---

## §1 — What shipped

**Half A** — `TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer` keeps `combat_only` and
gains `recipient: DamageRecipient`. The lowering
(`testing::replay_harness::build_face_triggered_abilities`) selects `trigger_on` through an
**exhaustive, wildcard-free** `match (combat_only, recipient)` over four new `TriggerEvent`s, so one
card ability lowers to exactly **one** `TriggeredAbilityDef`.

**Half B** — new `TriggerCondition::WhenDealsDamage { recipient }` (three `TriggerEvent`s, same
discipline) and new `EffectAmount::DamageDealt`, carried on a new `damage_dealt_amount: u32` through
the `PendingTrigger → StackObject → EffectContext` chain `combat_damage_amount` already uses.

**Both halves are served by ONE arithmetic**: `rules::abilities::queue_damage_source_triggers`,
called from the `GameEvent::CombatDamageDealt` arm with `is_combat: true` and from the
`GameEvent::DamageDealt` arm with `is_combat: false`. `is_combat` is a property of the **event**,
never of an ability — which is the distinction `combat_only` failed to make.

`TODO(PB-37)` and its in-def echoes are deleted.

## §2 — Corrections this batch made to its own inputs

1. **The brief's CR cite** (§0.6 (i)) — CR 603.10a is zone-change look-back, not "that much".
   13 cites moved to CR 608.2h / CR 113.7a. It had been copied into acceptance criterion 7333, so
   obeying it would have put 13 wrong cites in the tree under an AC that read as satisfied.
2. **"Delete the unread flag" is wrong** (§0.5(b)) — 0 declared users of `combat_only: true`, but
   **1 printed one** (`breath_of_fury`). The declared axis and the printed axis do not nest.
3. **The still-blocked self-family list is a FLOOR by 5×.** The brief names two
   (`warren_instigator`, `tandem_lookout`); the roster's `all_cards()` walk finds **ten**.
4. **Two blocker notes this batch falsifies**, found by the inverse axis and repaired in place —
   `niv_mizzet_visionary` (its "neither is expressible" is now half false) and `tandem_lookout`.
5. **A fourth `WhenDealsDamage` member no document names** — `tandem_lookout`, a *granted* ability,
   structurally invisible to a per-def ability-list walk.

## §3 — Wire: prediction vs outcome

| | predicted (§0.3, commit `a9fca688`) | measured |
|---|---|---|
| PROTOCOL | 41 → 42, ONE bump | **41 → 42** ✓ |
| HASH | 82 → 83, ONE bump | **82 → 83** ✓ |
| PROTOCOL closure type count | unchanged, 98 | **98** ✓ |
| HASH closure type count | unchanged, 132 | **132** ✓ |
| `TriggerCondition` / `DamageRecipient` on either closure | no | absent from both ✓ |

The stop condition never fired. Both fingerprints were taken from the failing gates' own output.

**The two-step stream observation recurs** (v82, v40): with everything in the tree and hashed but
BEFORE the version bump, `declaration_fingerprint_is_pinned` was RED and
`stream_fingerprint_is_pinned` was **GREEN** — `canonical_fixture()` carries no pending trigger, no
stack object with a damage amount and no card registry, so none of this batch's new bytes can reach
it. The stream moved only once `HASH_SCHEMA_VERSION` became its own first byte.

### §3.1 — The sentinel sweep failed once, and the survivor scan reproduced the failure

**49 HASH + 14 PROTOCOL** sentinels, final. The first sweep re-pinned 48 + 13 and **missed
`pb_dx2_command_gates.rs`'s `41u32`**: the PROTOCOL regex ended `41\b`, and `\b` between `1` and `u`
is not a word boundary. `OOS-DX20b`'s recorded lesson (`79u8`) — handled for HASH (`82(u8)?`) in the
same script and not carried across one symbol.

**The survivor scan did not catch it, and the reason is the durable half.** PB-DX50's rule is *"a
survivor check written with the same regex as the re-pin is not a check"*, and this scan obeyed the
letter of it — a ±3-line window instead of a symbol-adjacent match, a genuinely different SHAPE.
It used the same **value** pattern, `\b41\b`. **Changing the shape of the matcher is not enough if
the literal stays the same**; the refinement belongs beside PB-DX50's sentence. Re-swept with a
value pattern admitting any integer type suffix, on both symbols; one further site found and fixed,
0 real survivors after. Filed as **`OOS-DX36-8`**.

Then `OOS-DX18-3`'s opposite check: all 61 changed lines of the first sweep read individually — all
61 assertion arguments, no prose rewritten. The corrected scan's only remaining hits are two
historical-prose lines in `hash.rs`'s own v81/v82 history recording that PROTOCOL was 41 **at the
time**, which are correct and were not touched.

## §4 — Tests

**5,115 / 0 / 5** full-workspace against the FINAL tree, **64** result-producing targets (63 → 64:
one new simulator test binary), residual list empty. Baseline **5,097 / 0 / 5** at 63 targets,
measured on this branch before any edit and **reproducing PB-DX35's published close pin exactly**.

Delta by test NAME, by a **byte-exact Python set difference** of the two run logs (never `sort` +
`comm` — `OOS-DX20b-5`): **18 additions, 0 leavers, 0 removals, 0 renames**. Count delta 18 ==
name-set delta 18; duplicate-name scan **empty on both runs** (`OOS-DX35-8`'s check, which a
byte-exact set difference is structurally blind to).

## §5 — Coverage

**1,138 → 1,139 / 1,803 = 63.1% → 63.2%.** ONE flip, `exalted_angel`, **named in writing before any
code** (§0.4). Exactly one `Completeness` marker line moves in the batch's whole card-def diff —
checked by `git diff` over the marker, not inferred from the count.

**A contradiction this batch introduced and the report caught**: deleting the `TODO(PB-37)` echoes
left `curiosity`, `ophidian_eye` and `warren_instigator` marked `partial` with **no** in-source
`TODO` / `ENGINE-BLOCKED` comment, taking `authoring-report.py`'s marker-vs-comment consistency
check 16 → 19. A `partial` def that names no gap reads as finished. Each now carries a TODO naming
the surviving blocker; back to 16 with no pre-existing entry dropped.

## §6 — Benches: measured, six runs, NO REGRESSION, and nothing claimed in the other direction

Matched-set A/B against merge base `e7d7ae31`, each revision in its own `git worktree` with its own
`CARGO_TARGET_DIR`, on an otherwise-quiet machine. **The same-code repeatability band was measured
FIRST, across THREE merge-base runs** (PB-DX20b's lesson), before any HEAD number was looked at.

| bench (µs) | base ×3 | HEAD ×2 | same-code band | verdict |
|---|---|---|---|---|
| `priority_cycle_4p` | 24.162 / 23.918 / 24.057 | 24.463 / 24.125 | 1.02% | overlap |
| `priority_cycle_6p` | 38.432 / 38.173 / 38.169 | 38.890 / 38.047 | 0.69% | overlap |
| `sba_check` | 14.589 / 15.137 / 14.946 | 15.066 / 14.853 | **3.76%** | HEAD entirely INSIDE the base band |
| `full_turn_4p` | 216.38 / 215.99 / 214.84 | 216.77 / 216.50 | 0.72% | overlap |
| `full_turn_6p` | 342.53 / 342.38 / 343.45 | 344.01 / 342.26 | 0.31% | overlap |
| `board_wipe_4p` | 120.85 / 121.52 / 120.17 | 117.53 / 120.27 | 1.12% | overlap |

**Verdict: no regression demonstrated.** `sba_check`'s same-code band (3.76%) is wider than any
base-vs-HEAD difference measured anywhere in the table. **`board_wipe_4p`'s apparent −2.7% is
deliberately NOT claimed** — its second HEAD run (120.27) sits inside the base range, so run 1 is
the outlier, not the effect.

**Bounded independently by a mechanism fact rather than left to the numbers.** The criterion's
premise is *"`DamageDealt` dispatch is on the hot path"*. It is not on any BENCHED path:
`crates/engine/benches/engine_perf.rs` contains **two** damage-related occurrences and neither
deals noncombat damage — `board_wipe_4p` is a `DestroyAll`, and `full_turn_4p`/`6p` walk *through*
the CombatDamage step with **no attackers declared**, so `assignments` is empty and the extracted
loop does nothing. The new `GameEvent::DamageDealt` call site is off every benched path by
construction, and the combat-arm change is an extraction of a loop that was already there.
