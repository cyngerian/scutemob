# PB-DX51 — plan and PRE-CODE wire prediction

Task `scutemob-226`; v4 queue rank 11 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 11).
Seeds: **OOS-DX21-4** (headline), **OOS-DX21-2** (offer soundness), **OOS-DX21-5** (rider).

> **This file is written and committed BEFORE any production line changes.** Its §3 wire
> prediction is the pre-code claim the criterion 7325 asks for; §0's census figures are the
> stage-0 re-derivation the brief mandates (all three registry cites had drifted).

---

## §0 — Stage-0 re-derivation (the brief says the cites drifted 250-300 lines; they had)

| thing | registry / memo cite | **at HEAD `71113bda`** |
|---|---|---|
| CR 508.8 skip | `turn_structure.rs:43-51` (registry) | **`rules/turn_structure.rs:43-47`** — `no_attackers = state.combat.as_ref().map(\|c\| c.attackers.is_empty()).unwrap_or(true)`; consumed at `:50` |
| `CombatState` init | `combat.rs:78-80` (registry) | **`rules/combat.rs:88-90`** (`if state.combat.is_none() { state.combat = Some(CombatState::new(player)); }`), after the CR 508.1 guard at `:74` |
| blocker refusal | `combat.rs:1103` (registry) / `combat.rs:1149` (memo) | **`rules/combat.rs:1158-1160`** (`defenders_declared.contains` → `AlreadyDeclaredBlockers`) |
| `remove_from_combat` | `combat.rs:1063` (registry) | **`rules/combat.rs:1087`** (decl), `:1092` the `attackers.remove` |
| blocker OFFER | `legal_actions.rs:923-954` (registry) / `:1227-1259` (memo) | **`crates/simulator/src/legal_actions.rs:1313-1344`** — `if state.turn().step == Step::DeclareBlockers && stack_empty { if let Some(ref combat) = state.combat() { if !combat.attackers.is_empty() {` … **no `defenders_declared` condition anywhere** |
| attacker suppression (PB-DX21's fix) | `:878` (registry) / `:1182` (memo) | **`legal_actions.rs:1261-1268`** |
| `defenders_declared.insert` | — | **`rules/combat.rs:1707`** |

**Every cite in both documents is stale.** Recorded, and the rows will be corrected at close.

### §0.1 CR text, read verbatim via the MCP rules server (criterion 7322)

* **CR 508.8** — *"If no creatures are declared as attackers or put onto the battlefield
  attacking, skip the declare blockers and combat damage steps."*
* **CR 508.4** — *"If a creature is put onto the battlefield attacking, its controller chooses
  which defending player, planeswalker a defending player controls, or battle a defending player
  protects it's attacking as it enters the battlefield (unless the effect that put it onto the
  battlefield specifies what it's attacking). Similarly, if an effect states that a creature is
  attacking, its controller chooses … Such creatures are "attacking" but, for the purposes of
  trigger events and effects, they never "attacked.""*
  * **508.4a** — if the specified defender/planeswalker/battle is gone when the effect resolves,
    *"the creature is put onto the battlefield but is never considered an attacking creature."*
  * **508.4c** — such a creature *"isn't affected by requirements or restrictions that apply to
    the declaration of attackers."*
* **CR 506.4** — *"A permanent is removed from combat if it leaves the battlefield, if its
  controller changes, if it phases out, if an effect specifically removes it from combat, … or if
  it's an attacking or blocking creature that regenerates …, stops being a creature, or becomes a
  battle. A creature that's removed from combat stops being an attacking, blocking, blocked,
  and/or unblocked creature."*
* **CR 509.1a** — *"The defending player chooses which creatures they control, if any, will
  block. …"* (each defending player, once — the twin of CR 508.1's once-per-combat action).

**The predicate CR 508.8 states is a two-part historical fact**: *were any declared* (a
declaration-TIME fact, CR 508.1) **or** *has any been put onto the battlefield attacking* (a
since-declaration fact, CR 508.4). Neither part asks what is in combat **now**, which is the
whole of `OOS-DX21-4`: CR 506.4 empties `combat.attackers` at instant speed and the engine reads
the emptied map at step end.

**Why `!attackers_declared` is NOT the fix** (the row says so and it is right): CR 508.1a's
*"if any"* makes an **empty** declaration a completed declaration, so `attackers_declared` is
`true` and CR 508.8 still demands the skip. The two questions are different and the field
`CombatState::attackers_declared`'s own doc already says so at `card-types/src/state/combat.rs:60-73`.

### §0.2 CR 508.4 entrant census — RE-DERIVED, the brief's list is a floor with two false members

Method: `grep -rn "attackers.insert" crates/ tools/ --include=*.rs`, then classify every
**production** hit (test-tree hits are fixtures and are counted separately).

| # | site | mechanism | CR |
|---|---|---|---|
| 0 | `rules/combat.rs:773` | the CR 508.1 **declaration** loop | 508.1 |
| 1 | `effects/mod.rs:1971` | `CreateToken` with `spec.enters_attacking` | 508.4 |
| 2 | `effects/mod.rs:7000` | `CreateTokenCopy`-family `enters_tapped_and_attacking` | 508.4 |
| 3 | `rules/resolution.rs:6173` | **Myriad** (CR 702.116a) token, "tapped and attacking" | 508.4 |
| 4 | `rules/resolution.rs:6650` | **Ninjutsu** (CR 702.49a), "tapped and attacking" | 508.4 |

**FOUR CR 508.4 entry sites — PB-DX21's recorded four reproduce exactly.** Two members the
task brief named are **not** sites and are recorded as refuted rather than silently dropped:

* `state/builder.rs` — its only combat line is `combat: None` (`:369`). It cannot put a creature
  onto the battlefield attacking, because it builds a state with no `CombatState` at all.
* `rules/replacement.rs:2347` — the nearest real line is `:2435`, `enters_attacking: false`, a
  **`TokenSpec` field initialiser**, not an insert. The spec flows into site 1.

Test-tree `cs.attackers.insert(..)` fixtures: **95** across 25 files (`view-model/src/tests.rs`
included). These are hand-built and never route through a marking site — see §2.3, which is why
the shipped predicate keeps an observational conjunct.

### §0.3 Pre-edit baseline

Full workspace, `--workspace --no-fail-fast`, captured to a file, taken on this branch **before
any edit**. Figures and the byte-exact test-NAME set difference are published in
`pb-DX51-execution-notes.md` §1.

---

## §1 — Design

### §1.1 ONE new field, and why one suffices

```rust
/// CR 508.8: whether any creature has been **declared as an attacker** (CR 508.1) or
/// **put onto the battlefield attacking** (CR 508.4) during this combat phase.
#[serde(default)]
pub had_attackers: bool,
```

CR 508.8's predicate is a pure **existential** over the disjunction of two events. A `bool` is
exactly that predicate and nothing more:

* a COUNT would be a second, unasked-for question — and the declaration count already exists as
  `PlayerState::attackers_declared_this_turn` (PB-OS6(b)), which is a *per-turn* quantity with
  its own open seed (`OOS-DX21-1`); duplicating it per-combat would create a third disagreeing
  counter;
* two fields (declaration half + entrant half) would let the two halves drift, and no CR rule
  distinguishes them for this purpose — CR 508.8 ORs them in one sentence.

**Monotone**: set `true`, never cleared. That is the fix — CR 506.4 removing a creature from
combat must NOT unset it, because CR 508.8's predicate is about what *was declared*, not what
survives. It is scoped to one combat phase for free: `end_combat` nulls `state.combat`
(`turn_actions.rs`) and `begin_combat` installs a fresh `CombatState::new` (`:1900`), so an extra
combat phase (CR 500.8 / 506.5, e.g. Aurelia) gets its own `false` — matching
`attackers_declared`'s established scoping exactly.

`#[serde(default)]` mirrors `attackers_declared`: an old snapshot deserialises as `false`. That
default is lossy in the **skip-happy** direction for a resumed mid-combat snapshot, which is the
same class as `OOS-DX21-3` and will be filed as a rider rather than left unstated.

### §1.2 ONE mutation path, so a fifth site cannot forget

The four CR 508.4 sites plus the declaration loop all become callers of a single method:

```rust
impl CombatState {
    /// CR 508.1 / CR 508.4: the ONLY way to add an attacking creature.
    /// Maintains CR 508.8's `had_attackers` marker for both routes.
    pub fn add_attacker(&mut self, id: ObjectId, target: AttackTarget) { … }
}
```

The declaration loop needs **no** empty-declaration special case: an empty `attackers` vector
never enters the loop, so `had_attackers` stays `false` and CR 508.8 still skips (probe (b)).
That is the property that makes one method serve both CR rules.

A source gate (`r1`) asserts that **no production file outside `combat.rs`'s own `impl` block
spells `.attackers.insert(`** — a sixth site is a red test, not a silent regression. The gate
is keyed on the mechanism (the raw map mutation), not on a file list, because
`OOS-DX48`'s `SITE_SRCS` defeat is a hardcoded-file-list defeat.

### §1.3 The skip predicate

```rust
// CR 508.8
let no_attackers = state.combat.as_ref()
    .map(|c| !c.had_attackers && c.attackers.is_empty())
    .unwrap_or(true);
```

**Both conjuncts are CR-grounded and the second strictly narrows the skip:**

* `!had_attackers` is CR 508.8's actual predicate — the declaration-time + entrant fact.
* `attackers.is_empty()` is an **observational fallback for the CR 508.4 half**: a creature
  sitting in `attackers` right now *is* attacking, and CR 508.8 says do not skip. It can only
  ever *prevent* a skip, never cause one, so it cannot make the engine less CR-correct. It is
  what keeps the change **behaviour-identical for the 95 hand-built test fixtures** that insert
  into the map directly (§0.2), and it is what a sixth unmarked production site would fall back
  on for the un-removed case. It does **not** rescue the removed case, which is why `r1` and the
  mark are still load-bearing — proven by revert row R2.

`unwrap_or(true)` is unchanged: no `CombatState` means nothing was ever declared.

### §1.4 OOS-DX21-2 — one condition on the blocker offer

`legal_actions.rs:1313` gains exactly one conjunct, mirroring PB-DX21 §2.7's attacker-side
shape verbatim:

```rust
if !combat.defenders_declared.contains(&player) { … }
```

CR 509.1a gives the action to each defending player **once**; the engine already refuses a second
with `AlreadyDeclaredBlockers` (`combat.rs:1158`), so this is SR-38 offer soundness — an action
the engine will refuse must not be offered.

**`OOS-DX21-6` budget**: `random_bot.rs` picks uniformly **by index**, so suppressing an offered
action reindexes every later draw. The PB-DX32 fuzz gate config is run **before and after** and
both numbers are published; any moved seeded pin is re-observed and reported as a measurement.

### §1.5 OOS-DX21-5 — the rider

`combat.rs:88-90`'s `CombatState::new` init moves below the per-attacker validation loop (and
below every other `return Err` in the function), so a refused declaration leaves no
`CombatState`. Behaviour-preserving through `process_command`, whose `Err` arm carries no
`GameState` (`OOS-DX21-7`) — so the probe must call `rules::combat::handle_declare_attackers`
**directly** with `&mut state`. That is the only idiom that can see it.

The move must not break the sites between the old init and the success path that assume
`state.combat` is `Some` — the CR 508.1h tax path, the goad/must-attack loops and the
`debug_assert!` at `:760`. Those are all reads of `state.objects`/`state.restrictions`, not of
`state.combat`; verified before moving, and the `debug_assert` stays.

---

## §2 — Probes (criterion 7322 (a)/(b)/(c) + the rider)

| id | property | CR |
|---|---|---|
| `t1` | declare ONE attacker, remove it at instant speed during the step → declare-blockers and combat-damage steps still occur | 508.8 / 506.4 |
| `t2` | …and a SECOND creature's block and combat damage actually happen (the consequence, not just the step name) | 509.1a / 510 |
| `t3` | **empty** declaration still skips (the PB-DX21 pin, wrong-way-round protection) | 508.1a / 508.8 |
| `t4` | a CR 508.4 entrant with **no** declaration → steps NOT skipped | 508.4 / 508.8 |
| `t5` | `had_attackers` survives `remove_from_combat` (monotone) | 506.4 |
| `t6` | a fresh combat phase starts with `had_attackers == false` | 500.8 / 506.5 |
| `r1` | source gate: no production `.attackers.insert(` outside the `CombatState` impl | — |
| `b1` | `DeclareBlockers` is not offered once `defenders_declared` contains the player | 509.1a |
| `x1` | a refused `DeclareAttackers` leaves `state.combat` `None` — **direct handler** | 508.1 |

Each is watched RED by an executed revert; the matrix goes in
`memory/primitives/pb-DX51-execution-notes.md`, and any UNDISCRIMINATED row is disclosed in the
test file's own module doc, not only in `memory/`.

---

## §3 — WIRE PREDICTION (written before any production line changes)

**`HASH_SCHEMA_VERSION` 81 → 82. ONE bump for the whole PB.**
**`PROTOCOL_VERSION` 41 → UNMOVED.**

*Reason, stated rather than asserted:* the only new field is `CombatState::had_attackers`.
`CombatState` lives in `GameState::combat` and **nowhere else** — it is not a field of any
`Command`, `GameEvent`, `Effect` or `Characteristics`, and `protocol_schema.rs`'s
`CLOSURE_MUST_NOT_CONTAIN` = `["GameState", "PlayerState", "StackObject", "CardDefinition"]`
excludes `GameState` from the wire closure by construction, so nothing can reach `CombatState`
through it. HASH, by contrast, hashes `GameState`, and `impl HashInto for CombatState`
(`state/hash.rs:4882`) already hashes `attackers_declared` and `defenders_declared`, so one more
hashed field moves the state-hash digest.

**Direct precedent**: PB-DX21 added `CombatState::attackers_declared: bool` and measured
**HASH 72 → 73, PROTOCOL 35 unmoved** (`hash.rs:744`, `:969`, `:1385` history rows).

The memo's confidence on this cell is **MEDIUM**, so per criterion 7325 both gates are
**computed at stage 0, immediately after the field lands**, and a PROTOCOL move is treated as
**stop-and-re-read**, not as a number to write down.

**No new type, variant or field is predicted for `Command`/`GameEvent`/`Effect`.** The offer
change (§1.4) is a suppression and adds nothing; the rider (§1.5) is a statement move.

**Sentinel census, taken at HEAD before the bump** (multi-line-aware, both `81` and `81u8`
spellings): **47 `HASH_SCHEMA_VERSION` sentinels across 46 files** (2 of them spelled across a
line break) and **13 `PROTOCOL_VERSION` sentinels across 13 files**. This reproduces PB-DX20b's
corrected 47 / 13 exactly. Re-pin by symbol, then survivor-scan with a **differently-shaped**
regex — and, per `OOS-DX18-3`, read every changed line of the diff, because a survivor scan is
structurally blind to an OVER-replacement.

**Coverage**: 0 flips predicted, 0 card-def edits expected. Nothing in this batch reads or
writes a `CardDefinition`.

**Benches**: `full_turn_4p` / `full_turn_6p` run the combat steps, so this surface IS on the
bench path. A merge-base A/B in an isolated worktree with its own `CARGO_TARGET_DIR` is owed,
with the same-code repeatability band measured FIRST (PB-DX20b's lesson) — or the claim is
"not measured" and says so.
