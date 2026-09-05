# PB-DX53 — plan and stage-0 record (scutemob-231)

> v4 queue rank 16. Seed **OOS-DX21-1**. Merge base `5182600e` (PB-DX39 merged).
> **This file is written BEFORE any production line changes.** The wire prediction in §5
> is the pre-commitment AC 7369 asks for; the coverage-flip prediction in §6 is AC 7370's.

---

## 1. The rule, verbatim, from MCP

**Windbrisk Heights, ruling 2007-10-01** (looked up this session, `mcp__mtg-rules__lookup_card`):

> "At the time the ability resolves, you'll get to play the card if you declared three
> **different** creatures as attackers **at any point in the turn**. A creature declared as an
> attacker in two different attack phases **counts only once**. A creature that entered attacking
> (such as a token created by Militia's Pride) **doesn't count** because you never attacked
> with it."

That one ruling answers all three questions AC 7368 asks, and it answers them without
inference:

| question | ruling's answer |
|---|---|
| turn-scoped or declaration-scoped? | **turn** — "at any point in the turn" |
| a vigilant creature attacking in both combats? | **counts once** — "counts only once", verbatim |
| CR 508.4 "put onto the battlefield attacking" entrants? | **do not count** — "you never attacked with it" |

**CR 508.4** (verbatim): "Such creatures are 'attacking' but, for the purposes of trigger events
and effects, they never 'attacked.'"

**CR 508.3d** (verbatim): "An ability that reads 'Whenever [a player] attacks, . . .' triggers if
one or more creatures that player controls are declared as attackers." — this is Legion's
Landing's family: **per declaration**, and its ruling 2017-09-29 agrees ("only counts creatures
that you declare as attacking creatures").

**CR 400.7** makes a creature that left and returned a NEW object, so it is genuinely a
different creature and counts again. That is the ruling's "different creatures" reading, not a
deviation from it.

---

## 2. The defect

`rules/combat.rs:823` **ASSIGNS**:

```rust
ps.attackers_declared_this_turn = attackers.len() as u32;
```

`turn_actions.rs:1679` zeroes it at the turn boundary; `effects/mod.rs:11034` reads it as
`>= n`. PB-DX21 closed the *within-one-combat* re-declaration half. The **extra-combat** half
survives, because a fresh `CombatState` is installed at each `BeginningOfCombat`
(CR 500.8/506.5) and the guard is scoped to one combat by design. Attack with three in combat 1
and one in combat 2 and the count drops to **one**: Windbrisk Heights goes dead for the rest of
the turn, which the printed card does not do.

It is also **not deduplicated**: nothing in the current shape could dedup, because a `u32` does
not know which creatures it counted.

---

## 3. THE ROOT DEFECT IS ONE DSL IDENTIFIER CARRYING TWO CR CONCEPTS

`Condition::YouAttackedWithNOrMore(u32)` has exactly two readers and they want **opposite**
semantics:

| def | printed text | CR / ruling | scope |
|---|---|---|---|
| `windbrisk_heights` | "if you attacked with three or more creatures **this turn**" | ruling 2007-10-01 | **per turn**, dedup'd, entrants excluded |
| `legions_landing` | "**Whenever** you attack with three or more creatures" | CR 508.3d | **per declaration** |

So the fix cannot be "make the field accumulate": that repairs Windbrisk and **regresses**
Legion's Landing (attack with 2 in combat 1 and 2 in combat 2 → per-turn set is 4 ≥ 3 →
transforms, which the printed card does not do; the trigger would not even have fired). PB-DX21
review finding M3 says exactly this, and AC 7368 requires Legion's Landing byte-identical.

**Therefore the DSL must split.** One identifier per CR concept.

### 3.1 Shipped shape

| symbol | was | means |
|---|---|---|
| `PlayerState.latest_attacker_declaration_size: u32` | `attackers_declared_this_turn` | size of this player's most recent declaration this turn (semantics UNCHANGED — assigned, not accumulated) |
| `PlayerState.creatures_declared_as_attackers_this_turn: OrdSet<ObjectId>` | *(new)* | every creature this player has **declared** as an attacker this turn; dedup by `ObjectId` is the CR 400.7 identity the ruling asks for |
| `Condition::YouAttackedWithNOrMoreThisDeclaration(u32)` | `YouAttackedWithNOrMore` | CR 508.3d — reads the `u32` |
| `Condition::YouAttackedWithNOrMoreCreaturesThisTurn(u32)` | *(new)* | ruling 2007-10-01 — reads the set's `len()` |

**Both old names are lies at HEAD and both are renamed rather than re-documented.**
`attackers_declared_this_turn` says "this turn" and means "the latest declaration";
`YouAttackedWithNOrMore` says neither scope while two cards read it for different scopes. A card
author reaching for the shorter identifier gets the per-declaration one silently — which is the
default-choice trap that produced this seed. Renaming is compiler-forced at every site, and the
wire is bumping anyway (§5), so the rename is free on the wire.

### 3.2 Legion's Landing is byte-identical BY CONSTRUCTION, not by measurement

Its path is: `latest_attacker_declaration_size` (same field, same assignment, same reset) read by
`YouAttackedWithNOrMoreThisDeclaration` (same arm body). Zero behavioural lines change on it.
The probe AC 7368 asks for is therefore a *pin* on a property that already holds structurally —
which is the right way round, because the next batch is the one that could break it.

### 3.3 CR 508.4 exclusion holds BY CONSTRUCTION and is pinned anyway

PB-DX51 made `CombatState::add_attacker` the only production path into `combat.attackers`, and
the four CR 508.4 entrant sites (two `effects/mod.rs` token paths, `resolution.rs` Myriad and
Ninjutsu) call it directly. The PlayerState write lives in `handle_declare_attackers` alone and
reads the **command's** attacker list. So an entrant can never reach the set. Pinned by a probe
**and** by a roster gate asserting exactly one production write site — because "by construction"
is a property of today's call graph, and a gate is what keeps it one.

---

## 4. Alternatives costed and REJECTED

**(a) Position-dependent evaluation** — keep one `Condition` variant and evaluate it per-turn
when it is an `activation_condition` and per-declaration when it is inside `Effect::Conditional`.
**PROTOCOL-neutral**, which is what the AC's prediction assumed. **Rejected**: it makes one DSL
identifier mean two things depending on where it is written, so a card author cannot state which
they want and the first card that pairs them the other way round ("Whenever you attack, if you
attacked with three or more creatures this turn, …") is silently wrong with nothing to catch it.
AC 7370 requires each census member "classified per-declaration vs per-turn", which presupposes
the classification is a property of the DEF, not of its position.

**(b) Move Legion's Landing's count onto `TriggerCondition::WheneverYouAttack { filter }`** as a
`min_attackers` field — the CR-purest model (CR 508.3d makes the count part of the trigger
condition, not of an effect gate). **Rejected on cost, not on correctness**: PB-OS11's history row
(`rules/protocol.rs:256`, `state/hash.rs:567`) records that changing that very variant from a unit
to a struct moved **both** fingerprints, so this is not cheaper on the wire; and the count would
have to reach trigger-collection time through the runtime `TriggeredAbilityDef`, which PB-DX35
measured at **190 exhaustive struct literals across 44 files** with no `Default` derive.

**(c) Delete the `u32` and read the per-declaration count off `CombatState`** — a new
`CombatState.declared_attackers: OrdSet<ObjectId>` set by the declaration only. Cleaner naming and
self-clearing per combat (PB-DX51's `had_attackers` precedent, HASH-only). **Rejected**: it makes
Legion's Landing's gate depend on `state.combat` being `Some` at trigger-resolution time, which is
true on every reachable path today and is exactly the load-bearing accident this project files
seeds about; keeping the stored `u32` makes the byte-identical claim in §3.2 structural.

---

## 5. WIRE PREDICTION — committed before any production line

### 5.1 HASH `84 → 85` — ONE bump

Two causes, one bump:

1. `PlayerState` gains a hashed field. `PlayerState` is in the HASH closure — the precedent is
   this very field: `state/hash.rs`'s v58 row records "`PlayerState` gains
   `attackers_declared_this_turn: u32`, hashed right after `attacked_this_turn`".
2. `Condition` gains a variant and renames another. `Condition` is hashed — the same v58 row
   records "`Condition` gains `TopCardIsInstantOrSorcery` (discriminant 49) and
   `YouAttackedWithNOrMore(u32)` (discriminant 50)".

`decl_fingerprint` MOVES (a new struct field plus an enum's declared shape). `stream_fingerprint`
moves per the v40 mechanism (`HASH_SCHEMA_VERSION` is its own first byte) — and is predicted to
be RED only *after* the version bump if `canonical_fixture()` carries neither a non-zero attacker
set nor a `Condition`, which is the two-step observation PB-DX51/DX36/DX52 recorded four times
running. That prediction is recorded here so the observation is a confirmation, not a discovery.

### 5.2 PROTOCOL `43 → 44` — ONE bump — **THE AC's PREDICTION IS REFUTED, AND HERE IS WHY**

AC 7369 predicts **PROTOCOL 43 UNMOVED**, on the stated ground that "`PlayerState` is in
`CLOSURE_MUST_NOT_CONTAIN`". That ground is **true and verified** —
`crates/engine/tests/core/protocol_schema.rs:116` lists
`["GameState", "PlayerState", "StackObject", "CardDefinition"]` — and it is **not sufficient**,
because the AC's prediction assumed the fix is a `PlayerState` field *alone*. It cannot be: §3
shows one identifier cannot carry both CR concepts, so the DSL must split, and `Condition` **is**
in the PROTOCOL closure — reachable from `Effect` (a `CLOSURE_MUST_CONTAIN` root) through
`Effect::Conditional { condition: Condition, .. }`.

This is not an inference. `rules/protocol.rs`'s **v21** history row says it in the tree already:

> "`Condition` (**already in the closure via `Effect::Conditional`**) gains two new unit/tuple
> variants … and `YouAttackedWithNOrMore(u32)` … (`PlayerState.attackers_declared_this_turn`, the
> fourth new field in this batch, is inside `GameState`, **not the wire closure** — HASH_SCHEMA_VERSION
> bump only)."

So the same batch that created this field measured, and wrote down, both halves of this
prediction five weeks ago. The refutation is of the AC's *scope assumption*, not of its rule.

### 5.3 Closure type counts — predicted UNMOVED

PROTOCOL **98**, HASH **132** (PB-DX52's gate-confirmed figures). Neither half adds a **type**:
`Condition` gains a variant of an existing type, and `OrdSet<ObjectId>` on `PlayerState` is a
generic already in the HASH closure (`dungeons_completed_set: OrdSet<DungeonId>`) over an element
type already in it. Both to be read off the gates' own output, never invented.

### 5.4 Sentinels

**48 HASH + 14 PROTOCOL** expected (PB-DX52's figures). Re-pin **by symbol**, then survivor-scan on
**BOTH** axes (`OOS-DX36-8`): a differently-shaped matcher (a ±3-line window, not symbol-adjacent)
**and** a suffix-tolerant value pattern (`84(u8|u32)?`, `43(u8|u32)?` — `\b` between a digit and
`u` is not a word boundary). Then `OOS-DX18-3`'s opposite check: read every changed line of the
re-pin diff for an OVER-replacement, which a survivor scan is structurally blind to.

### 5.5 `size_of::<PlayerState>()`

Measured at both revisions and published. PB-DX18's precedent is a **real** uniform 2.5–4.5%
regression from `PlayerState` 360 → 376 (+16 bytes) on a struct copied at every mutation, so the
bench A/B is **owed**, with the same-code band measured FIRST across three merge-base runs before
any HEAD run is compiled.

---

## 6. COVERAGE-FLIP PREDICTION — committed before any regeneration

**ONE flip, NAMED: `minas_tirith` `partial` → `Complete`.**

`minas_tirith` is a **third member of the turn-scoped class that no document in the chain names**
— not the seed, not the v4 memo row, not the task brief. It prints
*"{1}{W}, {T}: Draw a card. Activate only if you attacked with two or more creatures this turn."*
and its third ability is **unauthored**, behind an in-source blocker note that is **FALSE at HEAD**:

> "ENGINE-BLOCKED: … Needs a count-based attacked condition (`Condition::AttackedWithNCreatures(2)`).
> PB-AC6's `Condition::YouAttackedThisTurn` is a bool and is insufficient."

`Condition::YouAttackedWithNOrMore(u32)` has existed since PB-OS6 (2026-07-19). The note outlived
the commit that falsified it — PB-DX27's "a blocker note is a claim" and `OOS-DX47-6`'s "a false
comment outlives its commit", found here by the census rather than by reading the file the seed
named. Its other two abilities are already authored, so authoring the third promotes it.

Consequences budgeted rather than discovered:
- coverage **1,139 → 1,140 / 1,803 = 63.2%**;
- exactly **one** `Completeness` marker line moves in the whole card-def diff, checked by
  `git diff` over the marker rather than inferred from the total (PB-DX26's lesson that a stable
  COUNT is not a stable SET);
- `CORPUS_COMPLETE` 1139 → 1140, so **every seeded fixture is re-dealt** (`OOS-CARDS2-3`).
  `COMMANDER_POOL` to be **re-measured by executing the gate**, not reasoned about ("a Land is not
  Legendary" is an argument, not a measurement) — note `minas_tirith` **is** Legendary, so this
  one genuinely may move and must be read off the gate.
- Any seeded-pin or fuzz movement is attributed by an **executed ablation** (whole engine change
  in the tree, only `minas_tirith`'s third ability + marker reverted), never argued.

No other flip is predicted. `moraug_fury_of_akoum` (`inert`), `breath_of_fury` (`partial`) and
`scourge_of_the_throne` (`partial`) keep their markers with their surviving blockers named.

---

## 7. Census plan (AC 7370) — by `all_cards()`, PRINTED by a test, never grepped

Three axes, all walked over `all_cards()` and all **printed** by their test so the figure in any
document is a transcription of executed output (PB-DX8's rule):

1. **Declared axis** — every def whose ability tree contains either `Condition` variant, walked
   recursively through every `Effect` nesting site (PB-DX26's `RollDice` lesson: a `Box`/`Vec`
   count is not exhaustive), classified per-declaration vs per-turn.
2. **Inverse oracle axis** — every def whose `oracle_text` (front face **and** every `CardFace`,
   PB-DX8's lesson) matches the attack-scope vocabulary, classified into the four scopes found:
   per-turn-count, per-declaration-count, per-turn-boolean (Raid), per-combat (Melee, Pack
   tactics). Each member repaired or its exact missing identifier named.
3. **Extra-combat axis** — every def declaring `Effect::AdditionalCombatPhase`, with its marker.
   *A source grep already disagrees with this axis*: `grep -rl AdditionalCombatPhase` returns
   **8** files and one of them is `windbrisk_heights.rs`, which only mentions the variant in a
   comment. SR-36's exact failure mode, available as a worked example before the batch starts.

---

## 8. Test plan

| id | what | file |
|---|---|---|
| `t1` | extra combat: 3 in combat 1 + 1 in combat 2 → per-turn set is 4, condition TRUE | primitives |
| `t2` | dedup: the SAME creature declared in both combats counts **once** (ruling verbatim) | primitives |
| `t3` | CR 508.4: a creature put onto the battlefield attacking does NOT enter the set (cited) | primitives |
| `t4` | per-declaration `u32` still ASSIGNED, so `…ThisDeclaration` is 1 after combat 2 | primitives |
| `t5` | turn boundary clears both | primitives |
| `t6` | `legions_landing` across an extra combat: 2 + 2 does **not** transform (M3, wrong-way-round) | primitives |
| `t7` | `legions_landing` 3 in combat 1 **does** transform (non-vacuity floor for `t6`) | primitives |
| `c1` | **channel**: real `LocalGame`/`HumanChoice`, `aggravated_assault` (`Complete`, deck-legal) for the extra combat, 3 attackers in combat 1 + 1 in combat 2, then the Windbrisk activation **ACCEPTED** and the exiled card **RESOLVING** | simulator |
| `c2` | same drive, 2 attackers in combat 1 + 1 in combat 2 → activation **REFUSED** (non-vacuity: `c1` must not pass because everything is accepted) | simulator |
| `r1..rN` | the three censuses, printed, ratcheted; the single-write-site mechanism gate | core roster |

Every gate and probe proven RED by an **executed** revert; the matrix is executed by the
coordinator rather than accepted from a delegated report, and any UNDISCRIMINATED row is disclosed
in the test's own module doc, not only in `memory/`.
