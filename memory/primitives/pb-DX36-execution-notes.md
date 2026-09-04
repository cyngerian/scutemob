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
noncombat trigger; `DamageDealt` reads `ctx.damage_dealt_amount`, *"that much"* for the triggering damage event
whichever kind it was. *(↻ This sentence cited **CR 603.10a** when it was written at stage 0, and
§0.6(i) below refutes that cite. It was never re-taken until the `/review` found this file
contradicting itself — PB-DX28's re-take MEDIUM, inside the batch's own binding record.)* They agree on a combat-damage trigger by
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
3. **The still-blocked self-family list is a FLOOR by 5×, and the batch's own scan was a floor too.**
   The task brief names NEITHER of the two this file first credited it with — queried directly, it
   names one self-family def, `exalted_angel`. `goblin_lackey`, `warren_instigator` and
   `tandem_lookout` all came from this batch's own stage-0 inverse oracle scan; the roster's
   `all_cards()` walk then corrected THAT to **ten** still blocked. *An inherited member list is a
   floor; so is the one you derived yourself an hour earlier.*
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
`crates/engine/benches/engine_perf.rs` contains **five** damage-related occurrences across four
lines, and none deals noncombat damage — `board_wipe_4p` is a `DestroyAll`, and `full_turn_4p`/`6p` walk *through*
the CombatDamage step with **no attackers declared**, so `assignments` is empty and the extracted
loop does nothing. The new `GameEvent::DamageDealt` call site is off every benched path by
construction, and the combat-arm change is an extraction of a loop that was already there.

## §7 — Revert matrix (11 rows executed, 11 discriminating, 0 UNDISCRIMINATED)

Each row was applied to source, the suite run, the failure observed, and the source restored
(`git diff` empty after each). Rows 8-9 are the batch's own gate being defeated and re-keyed.

| # | revert | reddens | note |
|---|---|---|---|
| R1 | the `combat_only` guard on the enchanted-combat arm (`if is_combat` → `if true`) | `t4b` only | the flag is load-bearing on exactly one probe, which is what "animated" has to mean |
| R2 | drop the opponent exclusion `pid != source_controller` | `t5` only | isolates the recipient axis from the damage-kind axis |
| R3 | `EffectAmount::DamageDealt` misrouted to `ctx.combat_damage_amount` | `t3`, `t6` | this is the shipped-defect shape of `OOS-DX36-5`, executed |
| R4 | disable the noncombat `queue_damage_source_triggers(.., false)` call | `t1`, `t3`, `t4a`, `t5`, `t6`, `t7` + the channel probe | **this is the pre-batch engine**, i.e. the whole defect reproduced; `t2`/`t4b` stay green as STATED CONTROLS (combat path, unaffected) |
| R5 | duplicate the combat-arm dispatch call | `t2`, `t3`, `t4b` | PB-DX47's double-push shape. Only the COUNT assertions catch it — a `>= 1` probe is green under this revert |
| R6 | `goblin_lackey` back to `WhenDealsCombatDamageToPlayer` | `r1a` only | |
| R7 | remove `curiosity`'s trigger condition | `r1d` only | |
| R8 | **a second dispatcher planted in `rules/mana.rs`, written `for ability in defs.iter()`** | **NOTHING — `r3` stayed GREEN** | the batch's own class gate defeated, see below |
| R9 | the same bypass, against the RE-KEYED `r3` | `r3` | red, as required |
| R10 | enchanted-family lowering `match` reduced to a bare `_ =>` | `r3` (non-vacuity) + `r4` | |
| R11 | self-family lowering `match` reduced to a bare `_ =>` | `r3` (non-vacuity) + `r4` | |

### §7.1 — R8: the class gate was defeated by its own author, inside the batch that cites the rule

`r3`'s first draft keyed its walk detection on the literal identifier `abilities`
(`"abilities.iter()"`, `"for ability in abilities"`, …). A second dispatcher written
`for ability in defs.iter() { … }` beside a `TriggerEvent::SelfDealsDamage` mention left **every
gate in the file green**; only a behavioural probe would have caught it, and no behavioural probe
exists for a dispatcher nobody has written yet.

That is `OOS-DX47-7` and `OOS-DX51-6` recurring — *a gate written for one spelling measures that
spelling* — committed inside the gate whose own module doc cites both. Re-keyed on the mechanism
(`"for ability in "`, loop-variable-agnostic, plus a bare `".iter()"`, deliberately
over-collecting, since over-collection can only make `r3` redder), and the identical bypass
re-executed against the fix: **red**. The narrative lives in `r3`'s own doc comment, not only here.

**This is the fifth consecutive batch in this queue in which a source gate written for one
syntactic form was defeated by execution** (PB-DX26 → PB-DX43 → PB-DX45 → PB-DX47 → PB-DX51 → here).
The pattern is now reliable enough to state as a rule rather than a lesson: **write the gate, then
write the bypass you would use to sneak past it, and run it — before you write the gate's doc
comment claiming it cannot be evaded.**

---

## §8 — The `/review` cycle: 2 HIGH / 4 MEDIUM / 5 LOW-NIT, all eleven taken

The reviewer had a shell and used it. Both HIGHs were **proven by execution**, not argued, and the
first is a correctness defect this batch shipped.

### §8.1 — HIGH 1: the batch's headline invariant was FALSE, and no probe it wrote could see it

`queue_damage_source_triggers` was called **inside `for assignment in assignments`**. One
`GameEvent::CombatDamageDealt` carries every assignment of the step in a single `events.push`
(`rules/combat.rs:2382`), and **CR 510.2** makes them simultaneous, so **CR 603.2c** — *"An ability
triggers only once each time its trigger event occurs"* — is violated by any source with more than
one assignment. Measured: a 5/5 `exalted_angel`-shaped creature blocked by two 2/2s dispatched the
self family **twice**, gaining 2 + 3 in two separate resolutions; a 6/6 trampler carrying
`Sigil of Sleep` dispatched the self family **twice** while the Aura half correctly fired once.

The official mirror ruling settles the CR question: **Boros Reckoner**, Gatherer 2017-03-14 —
*"If Boros Reckoner is dealt damage by multiple sources at once, such as by two creatures blocking
it, its ability triggers once and one target is dealt that much damage."*

**Three things make this the batch's own subject matter rather than an ordinary bug.**

1. **The doc comment asserted the opposite, unconditionally**, and the census behind it was
   *correct*: emit-site disjointness (1 `CombatDamageDealt` site, 5 noncombat `DamageDealt` sites)
   is true and **bounds the ARMS, not the LOOP INSIDE one arm**. Nobody checked the second, and the
   sentence read as though the first covered it. *A true premise can carry a false conclusion, and
   a comment is where that gets frozen.*
2. **Every COUNT probe drove a single-assignment fixture.** `t2` is the file's dedicated
   exactly-once probe and its own docstring says *"a `>= 1` assertion here would still pass on
   PB-DX47's double-push shape"* — and `t2` **passes under the defect**, re-verified by the
   coordinator. **A COUNT assertion proves exactly-once only on the fixture shape it drives**;
   PB-DX47's own lesson (*a differential probe proves agreement on the branches it drives and
   nothing about the branches it does not*), one axis over, inside the batch that cites it.
3. **It was a regression this batch introduced**, not inherited: the pre-batch attachment loop
   `continue`d unless the target was a Player, and a source has at most one Player assignment per
   step — which is exactly why the Aura half reads 1 and the new self family reads 2.

**Fixed** by grouping the event's assignments by `source` (first-appearance order, never sorted by
`ObjectId`, which would reorder triggers) and dispatching the self family once per source with
`amount` = the SUM. The attachment halves keep the single Player entry's own amount — the two must
NOT share an amount. New probes `t8` (multi-block) and `t9` (trample). **The revert was re-executed
independently by the coordinator rather than accepted from the report**: reinstating per-assignment
dispatch reddens exactly `t8` and `t9` at `left: 2, right: 1` and leaves all eight other probes
green — including `t2`.

### §8.2 — HIGH 2: the class gate was bypassable on the two axes it did not key on

`r3` scanned `read_dir("crates/engine/src/rules")` — **non-recursive, one directory** — and matched
the **qualified** string `TriggerEvent::<Name>`. The reviewer compiled and ran two bypasses and the
whole `--test core` target stayed green (710 passed) for each:

* **outside `src/rules/`** — a second dispatcher in `effects/mod.rs`, *the file that emits
  `GameEvent::DamageDealt` at four of its five sites*, i.e. the likeliest place a future author
  writes one;
* **a `use` alias inside the scanned directory** — `use crate::TriggerEvent::{SelfDealsDamage as
  SDD, …}`, after which the qualified literal never appears in the file.

Both fixes already existed **in this same test crate**, one batch old: PB-DX49's `/review` widened
its `r6` to a workspace walk (`workspace_src_files_checked()`, 14 roots / 148 files, with executing
non-vacuity floors) and re-keyed its `r7` onto the **bare** name at word boundaries precisely
because *"the qualified path is evaded by a `use` import"*. `r3` re-derived a narrower scan instead
of reusing either.

**And fixing it surfaced a third axis nobody had named**: the scan window only looked FORWARD from a
walk marker, but **a `use` alias's bare name sits BEFORE the marker it gives meaning to**, so
bare-name matching alone was still green. The window is now bidirectional. Both bypasses
re-executed against the final gate: **RED**.

**§7's *"11 rows executed, 11 discriminating"* was true and is not the same claim as *"the gate
cannot be evaded"*.** The docstring said the gate keys on "the mechanism"; the mechanism it keyed on
was the WALK, and the axes it did not key on were the FILE SET and the SPELLING. That correction is
now in the test's own doc.

### §8.3 — The MEDIUMs and LOWs, all taken

* **M3** — `warren_instigator`'s `Completeness` marker, a MACHINE-SCANNED surface, said *"Trigger
  currently resolves to `Effect::Nothing`"* while a comment eight lines above (same commit) says
  explicitly that no trigger is declared, and the def has none. Copied verbatim from
  `goblin_lackey`, where it is true. **PB-DX27's class authored fresh**, and neither
  `pb_dx27_stale_blocker_notes` nor `completeness_deviation_scan` caught it.
* **M4** — §0.5(e) claims `damaged_player`'s doc was *"the one thing that said combat"* and the
  batch fixed **two of six** sites. The four missed included `TargetController::DamagedPlayer`,
  which **names `Sigil of Sleep`** and still called itself combat-only — the very arm that card
  uses, and the reason the noncombat path had to populate the field at all.
* **M5** — `grateful_apparition`'s header asserts the variant this batch shipped does not exist.
  Narrowed, and the interesting half is the DIRECTION: `WhenDealsDamage` is too **WIDE** for it
  (that card prints *"deals COMBAT damage"*), because PB-DX36 deliberately gave the new condition
  no `combat_only` flag. It needs its own variant, not a field.
* **M6** — **both** attributions of `tandem_lookout` were wrong, in **opposite** directions: the
  registry said *"a fourth member no document names"*, the roster said *"the task brief's list"*.
  Queried directly, the brief names **one** self-family def (`exalted_angel`); `goblin_lackey`,
  `warren_instigator` and `tandem_lookout` all came from this batch's own stage-0 scan, which the
  `all_cards()` roster then corrected to ten. **An inherited member list is a floor; so is the one
  you derived yourself an hour earlier.**
* **L7** — three published figures re-taken: *"6 card-def files"* is **8** (transcribed from the
  stage-0 prediction and never re-taken after two defs were narrowed — PB-DX28's MEDIUM again);
  *"all 13 cites this batch introduced"* is false as written (~110 CR cites in all; 13 is the count
  that would otherwise have said 603.10a); the bench note's *"two damage-related occurrences"* is
  five across four lines, conclusion unaffected.
* **L8** — §0.5(d) cited CR 603.10a and §0.6(i) refutes that cite: **the binding record
  contradicting itself**, never re-taken until the review read it.
* **L9** — CR 510.3a is a combat-damage-STEP rule and is this family's house cite, so ~26 new sites
  attach it to explicitly noncombat behaviour. **Filed (`OOS-DX36-9`) rather than swept**, with the
  reason: the cites are redundant not wrong-in-consequence, the convention predates the batch, and
  rewriting it corpus-wide inside a fix cycle under a LOW is churn.
* **L10** — the v4 memo cell said `OOS-DX36-1..7` while three other surfaces said `-1..8`.
  **Dispatch hygiene 8's exact case for the third consecutive batch**, and this time on the surface
  the next dispatcher reads. Now `-1..9` everywhere, reconciled **after** the fix cycle.
* **NIT** — two stale strings, and the census-disjointness caveat: **eight of the ten
  still-blocked members print PB-DX47's own phrase**, and the two rosters stay disjoint only
  because PB-DX47's inverse ratchet is `Complete`-only while all eight are `partial`. Promoting any
  one of them moves it between both rosters at once. Now stated in the roster's own doc.

**The reviewer also re-derived and CONFIRMED**: the 190-literal `TriggeredAbilityDef` figure, the
1 + 5 emit-site census, `hash.rs:6848` as the sole pre-batch `combat_only` read, that a fourth
`DamageRecipient` variant is a compile error in both lowering matches, that a `_ =>` wildcard
reddens `r4`, that R4 reddens exactly the six probes plus the channel with `t2`/`t4b` green as
stated controls, 0 stale sentinel survivors under an independently-shaped scan, an empty census
partition on all four intersections, and the suite/gate figures against the final tree.
