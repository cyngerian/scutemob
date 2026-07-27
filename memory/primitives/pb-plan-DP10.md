# Primitive Batch Plan: PB-DP10 — Widen the decision gate (stop the 277-def figure growing silently)

**Generated**: 2026-07-27
**Task**: `scutemob-158` · branch `feat/pb-dp10-widen-the-decision-gate-stop-the-277-def-engine-gues`
**Primitive**: none. **TEST-ONLY.** A corpus gate + taxonomy over the decision sites the engine
still chooses for the player, with a frozen baseline so a *new* instance reddens.
**Findings**: the audit §8 `PB-DP10` row; the §1 "the gate is narrower than the invariant"
table; §3.1's 21 rows / 277 defs; §10's re-audit triggers. **DP-INV** (§1) is the invariant.
**CR Rules**: **608.2d** (the unifying rule for every resolution-time choice), **701.9/701.9b**
(discard — *the audit's `701.8` cite is wrong; 701.8 is Destroy*), 701.21a, 701.22a/b, 701.23a/b/d,
701.25a, 701.34a, **701.39a** (Bolster — *the audit's §4.9 `701.29a` is wrong*), 701.47a, 701.50a,
701.54a, 701.57, 401.4, 115.7a/d, 118.12/118.12a, 603.3b/c/d, 605.1a, 106.1a/b
**Class**: GATE / INVARIANT. Rank 10 of the PB-DP suite; **closes it**.
**Cards affected**: **0 card-def source edits predicted. 0 completeness flips predicted.**
The gate *measures* ~110 `Complete` defs into a frozen baseline; it does not edit them.
**Wire**: **PROTOCOL 31 / HASH 68 UNMOVED.** Argued in §7, with the falsifier.
**Baseline**: PROTOCOL **31** (`rules/protocol.rs:299`), HASH **68** (`state/hash.rs:660`),
tests **3,910** on main at merge `d65e7f1e`.
**Dependencies**: PB-DP7 (`BlockingDecision`), PB-DP8 (`pending_trigger_targets`,
`Command::ChooseTriggerTargets`), PB-DP9 (`pending_effect_choice`,
`Command::AnswerEffectChoice`) — all three only as *facts to classify*, none as code to extend.
SR-2 (`Completeness`), SR-9a (test layout), SR-33/34/37/38 + PB-EF12 (`effect_choose_gate.rs`),
SR-25 (`bare_lookup_ratchet.rs`, the ratchet idiom), SR-12 (`completeness_deviation_scan.rs`,
the allowlist idiom), PB-RS4 (`face_dereg_parity.rs`, the source-scan parity idiom).
**Deferred items carried in**: **OOS-DP7-7** ("the §3.1 re-derivation is still owed") is
**CLOSED by this batch** — T9 is the re-derivation, computed rather than grepped. **OOS-DP8-14**
was already answered by PB-DP9 (one action string, not three) and needs nothing here.

---

## 0. Executive summary — the three decisions this plan makes

1. **Gate-side allowlist, not a new `Completeness` variant.** The WIP file's caution that a
   marker variant is "inside the wire closure" is **falsified by reading** (§1.1): `Completeness`
   is reachable only through `CardDefinition`, and `CardDefinition` is in
   `CLOSURE_MUST_NOT_CONTAIN` for **both** `protocol_schema.rs` and `hash_schema.rs`. A marker
   variant would be wire-free. It is still the wrong answer, for two better reasons (§1.2), and
   the hard constraint forbids touching `card-types` regardless.

2. **The serde walk this batch inherits is BROKEN for two of the 21 rows, and one of them is
   the third-largest still-auto row.** `Effect::Proliferate` and `Effect::TheRingTemptsYou` are
   **unit variants**; serde's external tagging serializes a unit variant as a bare JSON
   **string**, not an object key. `effect_choose_gate.rs::contains_key`,
   `pb_rs1_roster_sweep.rs::contains_key` and `pb_dp9_effect_choice.rs::json_contains_variant`
   all match object keys only, so a naive reuse would report **0** for Proliferate's 25
   `Complete` defs. That is exactly the "gate reports success while checking nothing" failure
   mode OOS-DP7-11 was filed for, and this plan's centre of gravity is not letting it ship
   again. §2 specifies the strengthened walk; **T2 pins the bug in both directions**.

3. **The honest claim is bounded, and the plan says so in the failure message.** PB-DP10 cannot
   stop the 277 growing — stopping it means demoting ~110 `Complete` defs and gutting the
   playable corpus. It converts **silent** growth into **recorded, reviewed** growth: a new
   `Complete` def carrying a still-auto-chosen decision fails the suite until its author either
   demotes it or adds a baseline entry with a written reason. Anyone reading the gate must not be
   able to mistake it for a closure of DP-INV. §5.4.

---

## 1. Gate-widening vs a new `Completeness`-adjacent marker

### 1.1 The wire premise in the brief is false (verified, not assumed)

- `crates/engine/tests/core/protocol_schema.rs:117` —
  `CLOSURE_MUST_NOT_CONTAIN = ["GameState", "PlayerState", "StackObject", "CardDefinition"]`.
- `crates/engine/tests/core/hash_schema.rs:137-141` — `CLOSURE_MUST_NOT_CONTAIN` includes
  `CardRegistry` **and** `CardDefinition`, reachable only through `#[serde(skip)] card_registry`,
  with `serde_skip_is_load_bearing` proving the skip does the work.
- `Completeness` is declared at `crates/card-types/src/cards/card_definition.rs:197` and is
  referenced from exactly one field: `CardDefinition.completeness` (`:170`). Grep across
  `crates/card-types/src` and `crates/engine/src` returns no other structural use — only
  re-exports (`cards/mod.rs:19`, `helpers.rs:8`, `engine/src/lib.rs:10`) and doc comments.

**So adding a `Completeness` variant moves neither fingerprint.** Record this in the plan's
audit update: the WIP file's "check the hash gate before assuming a new marker variant is free"
was the right instruction and the answer is "it is free".

### 1.2 It is still the wrong answer — two reasons that survive the wire finding

- **A new variant must be either deck-legal or deck-illegal, and both are wrong.**
  `Completeness::is_complete()` is the single predicate `validate_deck` uses
  (`rules/commander.rs:232`, reason text at `:46-50`). A new *deck-legal* variant
  ("`ChoiceDeferred`") weakens SR-2 / Architecture Invariant 9 from "every clause is
  implemented" to "every clause is implemented, except the ones we listed", and the invariant's
  own justification — a history that cannot be correctly rewound — applies verbatim to a
  choice no player made. A new *deck-illegal* variant demotes ~110 `Complete` defs and empties
  the playable corpus overnight. The enum's own doc comment already forecloses the middle:
  *"There is no 'mostly fine' tier on purpose."*
- **The hard constraint.** `crates/card-types/src/` must be untouched (WIP file, task brief,
  explicit). A marker is a `card-types` edit plus 1,798 defs' worth of authoring guidance plus a
  `tools/authoring-report.py` bucket. That is a milestone, not a test-only batch.

### 1.3 Therefore: a gate-side, frozen, name-keyed baseline

**Where it lives**: `crates/engine/tests/core/decision_gate.rs`, as

```rust
/// (def name exactly as `all_cards()` reports it, row ids it hits, post-freeze reason)
const BASELINE: &[(&str, &[&str], Option<&str>)] = &[ /* ~110 entries, sorted by name */ ];
```

**What an entry records, and what it deliberately does not.**

| field | content | why |
|---|---|---|
| name | `def.name` verbatim | keys on the runtime identity, not a file stem — `all_cards()` is authoritative (SR-36) |
| row ids | the **exact** sorted set of `ROWS` ids this def hits | a superset means the def *gained* a decision since the freeze ⇒ fail; a subset means it lost one ⇒ fail asking you to tighten (the `bare_lookup_ratchet` two-arm convention) |
| reason | `None` for the 2026-07-27 freeze; `Some(text)` for every later addition | 110 copies of the same sentence is busywork; the *class-level* justification lives once, on `ROWS[i].class`. A post-freeze addition is a deliberate act and carries its own sentence, so **growth is legible in the diff** |

**Why not a count ratchet alone**: the brief names the hole exactly — a bare count lets a new
def in whenever an old one leaves. **Why not the allowlist alone**: a reviewer adding twenty
entries in one commit is invisible in the aggregate, and the aggregate is the number the audit
publishes. **Take both** (T4 + T6). `bare_lookup_ratchet.rs` is the shape precedent for the
ratchet half (per-file ceilings + `MIN_FILES` + `MIN_TOTAL` denominator guards, with the
"you converted some — lower the ceiling" arm); `completeness_deviation_scan.rs::ALLOWLIST` +
`every_allowlist_entry_is_live_and_necessary` is the shape precedent for the allowlist half.
Both are read and both fit; neither fits *alone*.

---

## 2. The walk — what is wrong with the inherited one and what replaces it

### 2.1 The unit-variant hole (the batch's headline technical finding)

serde's default external tagging:

| variant shape | JSON | seen by `contains_key`? |
|---|---|---|
| `Effect::Scry { player, count }` | `{"Scry":{"player":…,"count":…}}` | **yes** (object key) |
| `Effect::CounterUnlessPays(..)` / tuple | `{"CounterUnlessPays":…}` | **yes** |
| `Effect::Proliferate` (unit, `card_definition.rs:1933`) | `"Proliferate"` | **NO** |
| `Effect::TheRingTemptsYou` (unit, `:2122`) | `"TheRingTemptsYou"` | **NO** |

Nested: `Effect::Sequence(vec![Effect::Proliferate])` ⇒ `{"Sequence":["Proliferate"]}`. All three
existing walks recurse into the array, hit `Value::String`, and return `false`.

Consequence if unfixed: row `proliferate` reports **0** instead of ~25, and row
`the_ring_tempts_you` reports 0 for the wrong reason (its true count is also 0 — so it would
look correct and prove nothing). **This is the single most important thing for the runner to get
right, and T2 is written to make it impossible to get wrong silently.**

### 2.2 The canonical walk

New file `crates/engine/tests/core/decision_site_walk.rs`, `mod`-declared in `core/main.rs`
(SR-9a; §6). Public within the `core` target (sibling modules reach it as
`crate::decision_site_walk::…`).

```rust
pub fn def_contains_variant(def: &CardDefinition, variant: &str) -> bool
pub fn find_variant_nodes<'a>(v: &'a Value, variant: &str) -> Vec<&'a Value>   // struct/tuple only
```

`def_contains_variant` matches:
1. an **object key** equal to `variant` (struct and tuple variants), and
2. a **`Value::String`** equal to `variant` (unit variants), **suppressed** when that string is
   the direct value of a field named in `PROSE_FIELDS`.

`PROSE_FIELDS` (free-text `String` fields reachable from a `CardDefinition`, so a card whose
oracle text or description literally *says* a variant name is not a false positive — the runtime
mirror of the audit's own "regex hits in comments" trap):
`name`, `oracle_text`, `subtype` (`Effect::Amass`, `:1641`), `prompt` (`Effect::Choose`, `:1764`),
`first_name`, `second_name`, `has_name`, `card_id`, `description`
(`TriggeredAbilityDef.description`, `state/game_object.rs:891` — **reachable** from
`Effect::CreateToken { triggered_abilities }`, `card_definition.rs:2257`, and it is prose
describing the effect, so it is the most likely false-positive source in the corpus).
Plus the `Completeness` payload keys `Inert` / `Partial` / `KnownWrong`.

`find_variant_nodes` returns every subtree keyed by `variant`, for the two **compound** rows
(§3, rows 1 and 12) which are not expressible as a name match: they are
`AbilityDefinition::Triggered` nodes qualified by a *field* of that node.

### 2.3 De-duplication, and what is NOT shared

Three copies of the object-key walk exist today, and **two of them are already in `core/`**:
`effect_choose_gate.rs::contains_key` (`:64`), `pb_rs1_roster_sweep.rs::contains_key` (`:22`).
Both should be **rewired onto the canonical walk** (same target, so this is real
de-duplication, and it retro-fixes their latent unit-variant blindness).

- **Abort condition, stated**: if rewiring changes either file's offender set or any printed
  count, **revert the rewire and keep the copies** — this batch must not move an SR-33
  assertion. Predicted no change, because every variant those two gates name
  (`Choose`, `MayPayOrElse`, `AddManaChoice`, `AddManaAnyColor`,
  `AddManaAnyColorRestricted`, `AddManaOfAnyColorAmount`, `Scry`, `Surveil`, `RevealAndRoute`,
  `LookAtTopThenPlace`) is a **struct** variant — verified by reading the declarations.
- **`primitives/pb_dp9_effect_choice.rs`'s `roster` module cannot be shared.** Integration-test
  targets are separate crates, and SR-9a's
  `group_main_rs_declares_modules_and_nothing_else` forbids `#[path]` in a group `main.rs`.
  Leave it in place, add a ~6-line doc note recording the unit-variant limitation and that it
  is harmless for its three struct variants, and file **OOS-DP10-1**. Cross-check by *value*
  instead of by text: T10/T11 assert the canonical walk reproduces PB-DP8's and PB-DP9's
  published rosters (`>=` floors). A textual parity gate over a 12-line function is
  over-engineering for a divergence that is now documented and provably harmless.

---

## 3. The taxonomy — all 21 §3.1 rows, classified against the engine

**Every class below was established by reading the named site on this branch.** Line numbers are
snapshots (audit §8's recurring lesson: *a site cite in this document is a snapshot*); the
function names are the durable anchor. Classes:

- **SERVED** — a real decision hook exists and the handler consumes it. A def in a served class
  must **not** be flagged.
- **AUTO** — the engine still picks. Flagged.
- **GATED** — barred from `Complete` by the SR-33 family. Not flagged (its own gate holds a
  hard zero).
- **NO-DECISION** — the row is in §3.1 but there is no player choice to hook.

| # | row id | §3.1 row | runtime predicate | verified engine site | class | gate flags? |
|---:|---|---|---|---|---|:--:|
| 1 | `triggered_targets` | targeted triggered ability (603.3d) | `Triggered` node, `targets` non-empty | `abilities.rs::flush_pending_triggers` emit `:8266`/`:8273`; `handle_choose_trigger_targets` `:9072`; `pending_trigger_targets` throughout | **SERVED** (PB-DP8) | no |
| 2 | `search_library` | `SearchLibrary` (701.23) | key `SearchLibrary` | `effects/mod.rs:3476-3499` → `EffectChoiceQuestion::SearchLibrary` (`stubs.rs:910`) | **SERVED** (PB-DP9); residuals **OOS-DP9-9** (`reveal` inert, CR 701.23h unmodelled), **OOS-DP9-3** (finds exactly one) | no |
| 3 | `proliferate` | `Proliferate` (701.34a) | **string** `Proliferate` (unit variant) | `effects/mod.rs:4460` — auto-selects **all** eligible; TODO at `:4458` | **AUTO** | **yes** |
| 4a | `discard_cards` | `DiscardCards` (701.9/701.9b) | key `DiscardCards` | `effects/mod.rs:1202` → `discard_cards` `:9319` — `min_by_key(id)` in a loop | **AUTO** | **yes** |
| 4b | `wheel_hand` | `WheelHand` / `WheelDraw` | key `WheelHand` (see §4 note 3 — `WheelDraw` has **no** runtime key) | `effects/mod.rs:1214` — discards the **whole** hand, so the pick order is unobservable; `WheelDraw` only sizes the redraw | **NO-DECISION** | no |
| 5 | `scry` | `Effect::Scry` (701.22a) | key `Scry` | served; `EffectChoiceQuestion::Scry` (`stubs.rs:919`) | **SERVED** (PB-DP9) | no |
| 6 | `sacrifice_permanents` | `SacrificePermanents` (701.21a) | key | `effects/mod.rs:4210` → `sacrifice_permanents_for_player` `:8193` — `n` lowest ids | **AUTO** | **yes** |
| 7 | `may_pay_then_effect` | `MayPayThenEffect` (118.12) | key | `effects/mod.rs:4133` → `try_pay_optional_cost` — the only branch is affordability | **AUTO** | **yes** |
| 8 | `choose_color_or_type` | `ChooseColor` / `ChooseCreatureType` | keys `ChooseColor` (**`ReplacementModification::ChooseColor`**, `replacement_effect.rs:219` — *not* an `Effect`) and `ChooseCreatureType` (**both** `Effect` `:1859` and `ReplacementModification` `:170`) | `effects/mod.rs:4372` — most common subtype among the **controller's own** permanents, `BTreeMap`+`max_by_key` (PB-DP9 fix, OOS-DP9-10); `replacement.rs` `ChooseColor` mirror | **AUTO** | **yes** |
| 9 | `look_at_top_or_route` | `LookAtTopThenPlace` / `RevealAndRoute` | keys | `effects/mod.rs:5790` destructures `optional: _` (inert by construction, comment says so); `:5698` | **AUTO** | **yes** |
| 10 | `surveil` | `Effect::Surveil` (701.25a) | key `Surveil` | served; `EffectChoiceQuestion::Surveil` (`stubs.rs:921`) | **SERVED** (PB-DP9) | no |
| 11 | `counter_unless_pays` | `CounterUnlessPays` (118.12a) | key | `effects/mod.rs:4187` — `cost: _`, delegates straight to `CounterSpell` | **AUTO** | **yes** |
| 12 | `modal_trigger` | modal triggered ability (603.3c) | `Triggered` node, `modes` non-null | `abilities.rs:9025-9037` — `modes_chosen = vec![0]` in **both** the `min_modes == 0` and `!= 0` arms | **AUTO** | **yes** |
| 13 | `change_targets` | `ChangeTargets` (115.7d) | key | `effects/mod.rs:7288-7292` — always declines when optional; `must_change` picks the smallest id | **AUTO** | **yes** |
| 14 | `put_on_library` | `PutOnLibrary` (**608.2d + 401.4**, *not* 701.20 — see §4 note 1) | key | `effects/mod.rs:3444-3464` — `sort_by_key(id)`, `truncate(n)` | **AUTO** | **yes** |
| 15 | `bolster_amass` | `Bolster` / `Amass` (**701.39a** / 701.47a) | keys | `effects/mod.rs:3122` (least toughness, tie → id order), `:3193` | **AUTO** | **yes** |
| 16 | `connive` | `Connive` (701.50a) | key | `effects/mod.rs:5496-5502` — inlined discard, `min_by_key(id)` | **AUTO** | **yes** |
| 17 | `discover` | `Discover` (701.57) | key `Discover` — **collides** with `KeywordAbility::Discover` (`state/types.rs:1476`, a unit variant); see §4 note 4 | `effects/mod.rs:4567` → `copy::resolve_discover` — always casts | **AUTO** | **yes** |
| 18 | `may_pay_or_else` | `MayPayOrElse` (118.12a) | key | stub — discards `cost`/`payer`, always `or_else` | **GATED** (`effect_choose_gate.rs:109`) | no (its own hard zero) |
| 19 | `add_mana_filter_choice` | `AddManaFilterChoice` (605.1a) | key | `effects/mod.rs:2823` — always **one of each** colour; AA/BB unreachable | **AUTO**, currently **0 `Complete`** — and see §4 note 5: **this row is NOT gated** | **yes**, as a **hard zero** |
| 20 | `choose_stub` | `Effect::Choose` (700.2) | key `Choose` | stub — always `choices.first()` | **GATED** (`effect_choose_gate.rs:84`) | no |
| 21 | `the_ring_tempts_you` | `TheRingTemptsYou` (701.54a) | **string** (unit variant) | `effects/mod.rs:4798` → `engine.rs::handle_ring_tempts_you` — ring-bearer = lowest id; `Command::TheRingTemptsYou` has no creature field | **AUTO**, currently **0 `Complete`** | **yes**, as a **hard zero** |

**Tally**: 4 SERVED (rows 1, 2, 5, 10) · 14 AUTO (3, 4a, 6–9, 11–17, 19, 21) · 2 GATED
(18, 20) · 1 NO-DECISION (4b). §3.1 lists 21 lines; this table has 22 because row 4 splits into
two different classes.

**Rows whose class I could NOT establish: none.** Every one was read. Two audit *CR cites* and one
audit *framing* were falsified in the process (§4).

### 3.1 How the taxonomy expresses "served for one choice, unserved for another"

The brief's hardest sub-question. The answer is that a row is a **(variant, decision)** pair, not
a variant, and the type says so:

```rust
enum DecisionClass {
    /// A real hook exists for THE DECISION THIS ROW COUNTS. `residual` names the
    /// seeds for other, still-unserved decisions on the SAME variant.
    Served { by: &'static str, residual: &'static [&'static str] },
    AutoChosen { cr: &'static str, site: &'static str, why_not_flagged_is_wrong: &'static str },
    Gated { by: &'static str },
    NoDecision { why: &'static str },
}
```

- Row 2 `search_library` is `Served { by: "PB-DP9 / CR 701.23a", residual: &["OOS-DP9-9", "OOS-DP9-3"] }`
  — the *which card* choice is served; the `reveal` field is inert and CR 701.23h is unmodelled,
  and a `Complete` def must not be flagged for a decision that **is** served.
- Row 9 `look_at_top_or_route` is `AutoChosen` even though row 5 `scry` (a sibling top-N reader)
  is served. Same family, different hook status; the table refuses to smear them.
- **T16** asserts every seed id named in `residual` still appears in the audit, so the
  taxonomy's honesty is machine-checked and cannot rot into a lie in either direction.

---

## 4. §3.1 reconciliation — the mechanisms, written down before the numbers exist

The gate's serde walk over `all_cards()` is authoritative; §3.1's numbers are a source regex.
They **will** disagree. T9 prints the gate's numbers; the audit update explains each material
discrepancy by mechanism. The mechanisms, enumerated in advance:

1. **Two CR cites in §3.1 are wrong.** `DiscardCards … (CR 701.8)` — CR **701.8 is Destroy**;
   discard is CR **701.9**, and **CR 701.9b** is the load-bearing sentence ("By default, effects
   that cause a player to discard a card allow the affected player to choose which card"), which
   PB-DP7 already cited correctly for the cleanup discard. `PutOnLibrary … (CR 701.20)` — CR
   **701.20 is Reveal**; there is no "put on library" keyword action, and the governing rules are
   CR **608.2d** (which cards) + CR **401.4** (their order). §4.9's Bolster cite `701.29a` is also
   wrong (**701.39a**), though the *source* has it right.
2. **Regex hits in comments.** The audit names its own live example: the bare string
   `Effect::Choose` appears in **119** files, all of them prose in doc comments; real code use is
   zero. The serde walk cannot see a comment.
3. **`WheelDraw` has no runtime key at all.** It is a *type*, not a variant of the walked tree —
   `Effect::WheelHand { draw: WheelDraw }` serializes as `"draw": {"ThatMany":…}`. The audit's
   `WheelDraw` needle is a source-regex artifact; at runtime the predicate is `WheelHand`.
4. **One genuine key collision: `Discover`.** `Effect::Discover { player, n }` (struct, so an
   object key) and `KeywordAbility::Discover` (unit, so a bare string). The strengthened walk
   sees both. **Decision: accept the collision, with a written reason** — `KeywordAbility::Discover`'s
   own doc (`state/types.rs:1468`) says the action "is invoked by their triggered abilities via
   `Effect::Discover`", so a def carrying the keyword marker really does reach the same
   auto-cast at `effects/mod.rs:4567`. Record it on `ROWS[discover]`. **T12** pins the collision
   inventory so a *new* enum reusing a row's variant name reddens.
5. **`AddManaFilterChoice`'s zero is unheld, and the audit's "control group" framing is wrong.**
   §3.1 says the four zero rows "get there two different ways", crediting the SR-33 gate for
   `MayPayOrElse` and `Effect::Choose` and hand-marking for `AddManaFilterChoice`. The gate bars
   `AddManaChoice`; `AddManaFilterChoice` is a **different serde key** and `contains_key` matches
   exactly (`k == variant`), so **nothing holds that zero.** Same for `TheRingTemptsYou`. PB-DP10
   converts both hand-facts into machine-facts at zero cost (rows 19 and 21, hard zeros, T7).
6. **Nesting the regex cannot see.** PB-DP9's fix cycle found ten defs missing because a
   hand-written walk skipped `AbilityDefinition::{Spell,Triggered,Activated}::modes`,
   `{SagaChapter,LoyaltyAbility}`, split-card halves (`AbilityDefinition::Fuse` —
   `connive.rs:52` is the worked example) and `Effect::CoinFlip`. A file-level regex misses
   nothing *within* a file, but it cannot distinguish "this file contains X" from "this def's
   effect tree contains X" — which matters for mechanism 7.
7. **The two compound predicates are file-level conjunctions in §3.1 and node-level in the
   gate.** `targeted triggered ability` = `AbilityDefinition::Triggered` **and**
   `targets:\s*vec!\[\s*TargetRequirement::` *anywhere in the same file*. A def with a
   `Triggered` ability **and** a separately-targeted `Activated` ability matches the regex and
   should not. That alone plausibly explains 84 (audit) vs **77** (PB-DP8's enumeration). The
   gate qualifies the field on the *same node*. Conversely the gate reaches **more**: a
   `Triggered` node nested inside a granted ability is visible to it and invisible to a
   file-level conjunction — so this row may go **up** as well as down, and either direction is
   correct.
8. **A class the gate also cannot see, and it is a real gap.** A token's
   `TriggeredAbilityDef` (`state/game_object.rs:884`) is a **different type** from
   `AbilityDefinition::Triggered` and carries its own `targets`-bearing runtime shape. Neither
   the audit's regex, PB-DP8's typed walk, nor this gate's `"Triggered"` predicate covers it.
   File **OOS-DP10-2**; do **not** widen the row in this batch (it would change PB-DP8's
   published roster on no evidence).
9. **`defs/mod.rs` is a non-issue at runtime.** The audit had to exclude it from a file glob;
   `all_cards()` yields *defs*, not files.
10. **"Effectively `Complete`" is the same set both ways.** The audit's regex OR (literal
    `Completeness::Complete` **or** no `completeness:` line) is exactly
    `def.completeness == Completeness::Complete` at runtime, because `Complete` is
    `#[default]`. The runtime form is authoritative and needs no OR.
11. **Union, not sum.** §3.1's per-row column sums to far more than 277 because a def matching
    three rows is counted three times in the table and once in the headline. **T9 must compute a
    `BTreeSet<String>` union and say so in its printed output** — and must print *both* unions:
    all-rows (the 277 analogue) and still-auto-only (the baseline's size).
12. **Corpus drift.** §3.1 was measured 2026-07-26 against 1,139 effectively-`Complete` of
    1,804. The authoring campaign moves both numbers continuously; T9 prints the live
    denominator so the percentage is always self-dating (this mechanizes §10's last trigger).
13. **Word-boundary traps vanish.** The audit had to write `Discover\b` to avoid the card
    *Kindred Discovery*. A serde variant key cannot collide with a card name, because card names
    live in `PROSE_FIELDS` and are suppressed (§2.2).

---

## 5. Scope discipline — what PB-DP10 does NOT do

1. **It does not fix any class-B site.** Not one line of `crates/engine/src` changes. The still-open
   rows (DP-13/14/16/17/18/19/20/25/26/31) remain open and are what PB-DP11+ is for.
2. **It does not add a `Completeness` variant** (§1) and does not touch `crates/card-types/src`.
3. **It does not demote any existing `Complete` def.** Predicted card-def edits: **0**. If the
   sweep turns up a def that is *live-wrong* (not merely un-consulted), **file a seed, do not
   demote** — unless the demotion is trivially correct and argued from the card's oracle text via
   the mtg-rules MCP, in which case one def edit is permitted and must be justified in the commit
   message. The distinction that matters: "the engine chose for you" is class B and is what the
   baseline records; "the engine did something the card does not say" is class D and is a seed.
4. **It does not re-run the audit's regex sweep to make the numbers agree.** The gate's numbers
   *become* the fact (the PB-DP8 / PB-DP9 precedent), and §3.1 gets a note pointing at the test
   that prints them.
5. **The SR-33 gate's tests are KEPT, not subsumed, and their walk is shared.** They assert a
   different claim: rows 18/20 are class **C** — the effect is a *stub* that does one fixed thing
   regardless of what the card prints, which is a correctness defect with a different exit route
   ("author it properly, e.g. one activated ability per colour"). PB-DP10's rows are class **B** —
   a legal-but-unchosen default whose only exits are "demote" or "record". Merging them would
   weaken SR-33's hard zero into a baseline. `effect_choose_gate.rs`'s module doc says "delete the
   stub gates when interactive choice lands"; **that trigger has not fired** for those three, so
   nothing is deleted. Two changes only: (a) rewire its walk onto the canonical one, with the
   abort condition in §2.3; (b) **T14** asserts every variant it bars appears in `ROWS` as
   `Gated`, so the two tables cannot drift apart. Optionally add a 3-line pointer comment in each
   file's module doc.
6. **It does not claim to close DP-INV.** §5.4 of the failure message, verbatim intent: *this
   gate makes the growth recorded, not impossible.*

---

## 6. Files, and the `mod` lines SR-9a requires

| file | action |
|---|---|
| `crates/engine/tests/core/decision_site_walk.rs` | **NEW** — canonical walk (§2.2), `PROSE_FIELDS`, `find_variant_nodes`, `DecisionClass`, `ROWS`, `row_hits(def) -> BTreeSet<&'static str>` |
| `crates/engine/tests/core/decision_gate.rs` | **NEW** — `BASELINE`, the ratchet constants, T1–T16 |
| `crates/engine/tests/core/main.rs` | **EDIT** — add `mod decision_gate;` and `mod decision_site_walk;`, **alphabetically** (between `deck_validation` and `effect_choose_gate`) |
| `crates/engine/tests/core/effect_choose_gate.rs` | **EDIT (conditional)** — rewire `contains_key`/`def_uses` onto the canonical walk; keep every assertion and message verbatim. Abort per §2.3 |
| `crates/engine/tests/core/pb_rs1_roster_sweep.rs` | **EDIT (conditional)** — same rewire, same abort |
| `crates/engine/tests/primitives/pb_dp9_effect_choice.rs` | **EDIT** — doc note on the unit-variant limitation + pointer to the canonical walk (comment only, no logic) |
| `docs/audits/decision-point-audit.md` | **EDIT** — §3.1 note + CR corrections, §4.9 Bolster cite, §5 markers, §6 bullet, §8 `PB-DP10` row → SHIPPED + **suite COMPLETE**, §8.1 close OOS-DP7-7 + file OOS-DP10-*, §10 mechanization status |
| `memory/primitive-wip.md`, `memory/workstream-state.md` | **EDIT** — close-out per house convention |

**SR-9a hazards, all checked**: a group `main.rs` may hold **only** `//!` docs and bare `mod x;`
lines (`group_main_rs_declares_modules_and_nothing_else`), so no `#[path]` and no `pub mod`;
group dirs must stay flat (`group_dirs_are_flat`); a file without its `mod` line compiles clean
and silently deletes its tests (`every_module_file_is_declared_in_its_group`); no
`#![cfg(...)]` in a module file (`no_module_level_cfg_in_group_files`). `decision_site_walk.rs`
contains **no `#[test]`** and that is allowed — but its items must be `pub` and reached as
`crate::decision_site_walk::…` from `decision_gate.rs` (sibling modules of the target's crate
root can see a private `mod` at that root).

---

## 7. Wire-neutrality argument

**Certain not to move — PROTOCOL 31 / HASH 68.**

- `protocol_schema.rs:58` — `SCAN_ROOTS = ["crates/engine/src", "crates/card-types/src"]`.
- `hash_schema.rs:82` — the same two roots.
- `hash_schema.rs`'s second axis, `stream_fingerprint`, is blake3 over `public_state_hash` ++
  each player's `private_state_hash` on a fixture — a function of engine code and state shapes,
  not of test files.
- Every file this batch writes is under `crates/engine/tests/`, `docs/` or `memory/`. **No scanner
  reads any of them.** A new integration-test module cannot change a declaration digest or a hash
  stream.
- SR-6 is also untouched: no engine source changes, so the 1,798 card defs stay `Fresh` and
  `tools/check-defs-fmt.sh` has nothing new to check (0 def edits predicted).

**If I am wrong, check in this order.**
1. `git diff --name-only main -- crates/engine/src crates/card-types/src` — must be **empty**. If
   it is not, an "in-scope" edit escaped the constraint: revert it and re-scope (task brief:
   *stop and re-scope*).
2. If that is empty and a fingerprint still moved, it is `protocol_schema.rs`'s documented
   rustfmt false positive. Check `rustc --version` / `cargo fmt --version` against
   `rust-toolchain.toml`'s pinned `1.95.0` (SR-11 exists precisely to make this impossible).
3. Run the two gates directly: `cargo test -p mtg-engine --test core protocol_schema::` and
   `… hash_schema::`. Both must be green with the histories **unedited** (append-only rule) —
   this batch appends no row to either history table.
4. Confirm no sentinel needs re-pinning: `rg -n 'HASH_SCHEMA_VERSION,\s*68|PROTOCOL_VERSION,\s*31'`
   should return the same file set before and after.

---

## 8. Test list, with fail-before / pass-after expectations

Nine of these are non-vacuity or gate-integrity probes. That ratio is deliberate: **a gate over a
clean corpus proves nothing** (`stub_gates_are_not_vacuous` is the model), and this batch
introduces two novel mechanisms — bare-string matching and a prose denylist — that have never
been exercised in this codebase.

| id | name | asserts | fail-before / pass-after |
|---|---|---|---|
| **T1** | `every_decision_row_predicate_is_non_vacuous` | for **each** of the 22 rows: a synthetic `CardDefinition` built to carry that row's site is detected; a bare def is not. At least one row is probed **nested** (`Sequence(Sequence(..))`) | fails today (no gate). Deliberately breaking one `ROWS` predicate must redden exactly one row |
| **T2** | `unit_variant_rows_need_string_matching` | the **legacy** object-key-only walk returns `false` for `Effect::Proliferate` and `Effect::TheRingTemptsYou`, while `def_contains_variant` returns `true`. Also pins the raw serde shape: `to_value(&Effect::Proliferate) == Value::String("Proliferate")` | **This is the batch's central fail-before.** Written so that a runner who reuses the inherited walk verbatim gets a red test naming the bug, not a green gate reporting 0 |
| **T3** | `prose_fields_do_not_trigger_a_unit_variant_row` | a def whose `oracle_text`, `name`, a granted `TriggeredAbilityDef.description` and a `prompt` are each literally `"Proliferate"` is **not** flagged; a def with `Effect::Sequence(vec![Effect::Proliferate])` **is** | pins both directions of the denylist. Removing a `PROSE_FIELDS` entry must redden |
| **T4** | `no_complete_def_introduces_an_unrecorded_auto_chosen_decision` | **the gate.** Offenders = effectively-`Complete` defs hitting ≥1 `AutoChosen` row that are absent from `BASELINE`, **or** present with a mismatched row set. Message names the CR, the engine site, and the **two** exits (§8.1) | red until `BASELINE` is populated from T9's printed output; green after. Adding a synthetic `Complete` def with a `Proliferate` must redden |
| **T5** | `every_baseline_entry_is_live_and_necessary` | each entry names a def in `all_cards()`; the def is still `Complete`; its row set **equals** the recorded set — superset ⇒ "this def gained a decision", subset ⇒ "tighten the entry"; a `Some(reason)` post-freeze entry is ≥30 chars | mirrors `completeness_deviation_scan::every_allowlist_entry_is_live_and_necessary`. Deleting a def or flipping its marker must redden |
| **T6** | `auto_chosen_complete_union_is_ratcheted` | the still-auto union count **==** `MAX_AUTO_CHOSEN_COMPLETE_UNION`, with the two-arm `bare_lookup_ratchet` messages; plus `MIN_ROWS`, `MIN_BASELINE`, `MIN_CORPUS` denominator guards | catches the "a new def slots into a freed seat" hole the brief names. Its message points at T4 so one cause does not read as two bugs |
| **T7** | `hard_zero_rows_have_no_complete_defs` | rows `add_mana_filter_choice` and `the_ring_tempts_you` have **0** `Complete` defs, with a message recording that these zeros were hand-maintained until now (§4 note 5) | new machine fact. Marking one filter land `Complete` must redden |
| **T8** | `served_rows_still_have_their_hooks` | for each `Served` row: the roster floor is non-zero **and** the hook is compile-forced — construct `Command::AnswerEffectChoice` and `Command::ChooseTriggerTargets`, call `GameState::pending_effect_choice()` / `pending_trigger_targets()` | if PB-DP9's channel were reverted, the row's class would become a lie; this makes it a compile error, not a comment |
| **T9** | `decision_site_reconciliation_report` | **prints** per-row `Complete` / non-`Complete` counts, the all-rows union (277 analogue), the still-auto union, and the live effectively-`Complete` denominator + percentage; asserts `>=` floors only (the PB-DP9 `>=`-not-`==` convention: an `==` pin reddens on unrelated authoring) | criterion 5554's printed artifact. **Closes OOS-DP7-7.** Its output is what populates `BASELINE` and `MAX_*`, and what the audit update quotes |
| **T10** | `canonical_walk_reproduces_pb_dp9_rosters` | `search >= 73`, `scry >= 16`, `surveil >= 8` (PB-DP9's *post-fix-cycle* published numbers, not its first answer) | cross-target value check replacing a textual parity gate (§2.3) |
| **T11** | `canonical_walk_reproduces_pb_dp8_roster` | targeted-trigger `Complete` `>= 77` (PB-DP8's enumerated number, not the audit's 84 nor the planner's 74) | ditto, and it exercises the compound predicate against a known-good answer |
| **T12** | `row_variant_name_collision_inventory_is_pinned` | source scan of `crates/card-types/src`: for each row key, the set of enums declaring a variant of that name equals a pinned map. `Discover` records its accepted collision; `SearchLibrary`/`Scry`/`Surveil` record that their `stubs.rs` twins are `GameState`/wire types **unreachable from `CardDefinition`** | a new enum reusing a row's variant name silently changes what the gate counts. `face_dereg_parity.rs` is the idiom |
| **T13** | `prose_field_denylist_covers_every_string_field_in_the_dsl` | source scan for `String` / `Option<String>` / `Vec<String>` field declarations reachable from `CardDefinition`; every field name is in `PROSE_FIELDS` | a *new* prose field is a new false-positive channel for the unit-variant rows. Forces a decision instead of a silent hole |
| **T14** | `sr33_gated_variants_are_represented_in_the_row_table` | every variant `effect_choose_gate.rs` bars appears in `ROWS` as `Gated`, and no `ROWS` entry claims `Gated` for a variant that gate does not bar | the two tables cannot drift; §5.5 |
| **T15** | `dsl_enum_rosters_are_classified` | count **+** blake3 digest of the sorted variant-name list for `Effect`, `ReplacementModification` and (unless `ability_definition_registry.rs` already pins it — check first) `AbilityDefinition`. Failure message: *"a new variant landed; classify it in `ROWS` and update audit §3.1 / §10"*, with the live count+digest printed | §10 trigger 1. Honest framing in §9 |
| **T16** | `named_residual_seed_ids_still_exist_in_the_audit` | every seed id in any `Served { residual }` appears in `docs/audits/decision-point-audit.md` | keeps the taxonomy's "served, with a residual" honest in both directions |

### 8.1 The failure message T4 must produce

`effect_choose_gate.rs`'s messages are the model — they name the CR, the defect, the site, and
the legal exits. T4's must add the bound on its own claim:

> These effectively-`Complete` card defs contain a decision the **CR gives to a player** and the
> engine still makes for them (audit `DP-INV`, `docs/audits/decision-point-audit.md` §1). The
> decision is legal — this is not a rules violation — but the game history records a choice no
> player made, which is the same defect Architecture Invariant 9 / SR-2 exist to keep out of a
> deck.
>
> Per def, per row: `<name>` hits `<row_id>` (CR `<cr>`, `<site>`).
>
> **This gate cannot stop the growth; it makes it recorded.** Two legal exits, and only two:
> 1. Mark the def non-`Complete` with a note naming the auto-chosen decision —
>    `completeness: Completeness::known_wrong("engine chooses which card is discarded (CR 701.9b)")`.
> 2. Add a `BASELINE` entry in this file with the def's exact row set **and a written reason**,
>    which is a reviewed acknowledgement that this card ships with the engine choosing for the
>    player until the owning PB lands.
>
> Implementing the choice properly is **not** an exit for this batch: it needs the owning
> engine PB (`docs/audits/decision-point-audit.md` §5, rows DP-13..DP-31) — the successor to the
> PB-DP suite, not a card-def edit.

---

## 9. §10 re-audit-trigger mechanization — an honest ledger (criterion 5557)

| §10 trigger | status after PB-DP10 | note |
|---|---|---|
| New `Effect` variant with a `choices`/`optional`/`may`/filter-selection field | **partially mechanized (T15), and the framing in the trigger is the wrong one** | A gate over card defs **cannot** see a variant no def uses — correct, and that is why T15 scans the *declaration*, not the corpus. But the honest marginal value is smaller than it looks: **`Effect` is in both the SR-8 and SR-17 closures**, so a new variant *already* forces a PROTOCOL and a HASH bump. The **notice** is mechanized today; what is missing is the **obligation**. T15 supplies the obligation message. A needle scan for choice-shaped fields would be worse than a whole-roster digest: the DSL has only **four** such fields (`prompt`/`choices` on `Choose`, `optional` on `LookAtTopThenPlace`, `chosen_subtype_filter` on a `TriggerCondition`), while `Proliferate`, `SearchLibrary`, `Scry`, `DiscardCards`, `SacrificePermanents`, `Bolster`, `Connive`, `PutOnLibrary`, `Discover`, `ChangeTargets` and `TheRingTemptsYou` carry **no** decision-shaped field at all — the choice is inherent in the CR keyword action, not in the DSL shape. **Recommendation: ship T15 as a whole-roster digest; do not ship a needle scan, and do not let the trigger's wording imply the needle set is the population.** |
| New `Command` variant → is it another accepted-and-discarded field (DP-24)? | **NOT mechanized** | Feasible test-only as an SR-15-style scan for `_`-bound `Command` fields in handlers. Out of scope; **OOS-DP10-4** |
| After the first blocking pending decision — re-derive §3.1's 277 | **MECHANIZED (T9)** | **Closes OOS-DP7-7.** Now recomputed on every `cargo test`, printed, and ratcheted |
| New `GameEvent` owes `reveals_hidden_info()` **and** `private_to()` | **NOT mechanized** | Both matches end in `_ =>` (`rules/events.rs:1511`, and the `reveals_hidden_info` tail), so a new variant silently answers `false`/`None`. A `GameEvent` roster digest is the *same twenty lines as T15* pointed at another enum, so it is **OPTIONAL-recommended** here (T15b). Same honesty caveat: a new `GameEvent` already forces a PROTOCOL bump, so the gain is the obligation, not the notice. **OOS-DP10-7** if not taken |
| New `BlockingDecision` variant → discharge the **seven** obligations | **stays human** | Only 2 of 7 are compile-forced (PB-DP9's own count). Obligation (7) — "argue whether it belongs in `loop_detection.rs`'s fingerprint" — is a *judgement*, not a shape, and no gate can make it |
| New same-zone caller of `move_object_to_zone` / `move_object_to_bottom_of_zone` (CR 400.7) | **stays human** | A `bare_lookup_ratchet`-style per-file ceiling is feasible and unbuilt; **OOS-DP9-11** already owns the sweep |
| After a `Zone` API change (top/bottom inversion) | **stays human** | §10 already prescribes deriving the roster from `Zone::push_front` call sites |
| `docs/authoring-status.md` shows a material `Complete` jump | **MECHANIZED (T9 + T6)** | T9 prints the live denominator and percentage on every run; T6's exact ratchet reddens on the *numerator* |

**Score: 3 of 8 mechanized, 1 optional, 4 stay human.** Write that number into §10 rather than a
claim of coverage — this suite's recurring lesson (OOS-DP7-11; PB-DP8's meta-lesson iii) is that
*a gate cited as covering something is a claim like any other.*

---

## 10. Step 0 — probes to run BEFORE writing the gate

House convention (rider-seed handoff: *probe-first pays*). Each probe either confirms or
falsifies a premise this plan rests on; a red one changes the design, not the code.

| id | probe | premise tested |
|---|---|---|
| **P0-a** | print `serde_json::to_value(&Effect::Proliferate)` and `to_value(&Effect::Scry{..})` | §2.1's unit-vs-struct representation (**P4**). If a unit variant is *not* a bare string, §2.2's string arm is unnecessary and T2/T3 change shape |
| **P0-b** | print the JSON of one def with a targeted triggered ability and one with `modes: Some(..)` | that `targets` / `modes` are visible on the `Triggered` node and not elided by `#[serde(default)]` / `skip_serializing_if` (**P3**) |
| **P0-c** | print the JSON of `connive.rs` (split half via `AbilityDefinition::Fuse`) and one saga / loyalty def | the walk reaches the four nesting classes PB-DP9's fix cycle found |
| **P0-d** | print, per row, the `Complete` / non-`Complete` counts and both unions | the reconciliation input; **decides `BASELINE`'s size** and whether §11 P2's ~110 estimate holds |
| **P0-e** | `rg -n '^    (<Key>)[ ,{(]' crates/card-types/src` for all 22 keys | T12's pinned collision map |
| **P0-f** | `rg -n '^\s+(pub )?[a-z_]+: (Option<)?(Vec<)?String' crates/card-types/src` | T13's `PROSE_FIELDS` completeness |
| **P0-g** | `git diff --name-only main -- crates/engine/src crates/card-types/src` (empty) | the wire-neutrality precondition; re-run at close (§7) |
| **P0-h** | rewire `effect_choose_gate.rs` + `pb_rs1_roster_sweep.rs` and diff their printed output | §2.3's abort condition |

---

## 11. Premises that may be falsified

- **P1 — "a `Completeness` variant is inside the wire closure" (the WIP file's caution).**
  **ALREADY FALSIFIED** by §1.1. The reason to reject a marker is Invariant 9 + the hard
  constraint, not the wire. If the runner finds a *third* reachability path into `Completeness`
  that I missed, §1.1 is wrong and the marker option must be re-argued — but the hard constraint
  ends it either way.
- **P2 — "the still-auto union is ~110."** Derived by subtracting the four served rows from
  §3.1's sums (277 minus ≈77+73+16+8 of overlapping served defs). P0-d measures it. **If it
  exceeds ~150, reconsider**: a 150-entry hand-maintained `BASELINE` starts to cost more than it
  catches, and the fallback is ratchet-only (T6) plus a per-row ratchet, dropping T4/T5.
  Document the switch if taken.
- **P3 — "the two compound rows are expressible on the serde tree."** P0-b decides. If `targets`
  is elided when empty, the predicate simplifies (presence of the key ⇒ non-empty); if `modes`
  is elided when `None`, likewise. Either way the predicate must be written against the
  *observed* JSON, not the declared Rust.
- **P4 — "a unit variant serializes as a bare string."** High confidence (serde's documented
  external tagging) but **must be pinned by T2, never assumed** — this is the premise the whole
  §2 design hangs on.
- **P5 — "every SR-33 variant is a struct variant, so the rewire is behaviour-neutral."**
  Verified by reading all six declarations. P0-h confirms empirically; §2.3 is the abort.
- **P6 — every number in §3.1.** Three consecutive batches (PB-DP6's 3-vs-14 site roster,
  PB-DP8's 84-vs-77, PB-DP9's 74/16/8-vs-69/16/7-vs-73/16/8) published a roster as fact and were
  wrong. **Treat every §3.1 count as unverified until T9 prints it.** In particular do not
  "reconcile" by adjusting the gate until it agrees with 277 — the gate is the instrument and
  the table is the estimate.
- **P7 — "no `#[serde(skip)]` hides part of the DSL."** `effect_choose_gate.rs`'s module doc
  records this as checked, and `pb_dp9_effect_choice.rs`'s `roster` doc repeats it. Re-check
  once (`rg 'serde\(skip' crates/card-types/src/cards/`) and carry the note forward — it is the
  one way the whole technique goes blind.

---

## 12. Risks & edge cases

- **R1 — allowlist churn during the authoring campaign.** Every new `Complete` def carrying an
  auto-chosen decision reddens T4 (and T6). That **is** criterion 5554, and the failure message
  is the mitigation: it tells the author exactly what to add and where. Keep `BASELINE` sorted
  by name so diffs are one-line.
- **R2 — one cause, two red tests.** T4 and T6 compute the same thing at different granularity.
  T6's message must point at T4 so a reader does not chase two bugs.
- **R3 — the rewire (§2.3) touching an SR-33 assertion.** Abort condition stated; P0-h checks it.
  Under no circumstances weaken an SR-33 message or assertion to make a shared walk fit.
- **R4 — prose false positives on the two unit-variant rows.** T3 + T13. The most dangerous
  field is `TriggeredAbilityDef.description`, which is prose *about the effect* and is reachable
  from `Effect::CreateToken`.
- **R5 — key collisions.** One live (`Discover`), accepted with a reason; three benign
  (`SearchLibrary`/`Scry`/`Surveil` twins in `stubs.rs`, unreachable from a def); T12 pins the
  inventory so a new one cannot arrive silently.
- **R6 — the gate reading as a closure of DP-INV.** §8.1's message and the audit §8 row must both
  say "recorded, not impossible". This is the most likely way PB-DP10 does harm.
- **R7 — the compound row may go UP.** The gate reaches nested `Triggered` nodes a file-level
  regex conjunction cannot. If T11's floor of 77 is exceeded, that is expected and correct;
  record the delta in the audit rather than clamping the predicate.
- **R8 — `MAX_AUTO_CHOSEN_COMPLETE_UNION` set from a wrong first measurement.** Set it from T9's
  printed output on a clean tree, and record the measuring commit next to the constant (the
  `bare_lookup_ratchet` comment convention).
- **R9 — over-engineering.** T12/T13/T15 are three source-scan gates. They are in because each
  defends a *novel* mechanism this batch introduces (string matching, the denylist, the
  obligation on a new variant). If the batch runs long, T16 is the first to drop, then T15b.
  **T1, T2, T3, T4, T5, T6, T7, T9 are non-negotiable.**

---

## 13. Seeds to file in `docs/audits/decision-point-audit.md` §8.1

| seed | finding | class |
|---|---|---|
| **OOS-DP10-1** | `primitives/pb_dp9_effect_choice.rs`'s `roster` walk (and, if the rewire is aborted, the two `core/` copies) matches object keys only and is blind to **unit** variants. Harmless for its three struct variants; a live hazard the moment it is reused. Promote the canonical walk to a shared location if a third target ever needs it — note SR-9a forbids `#[path]` in a group `main.rs`, so the shared home would have to be the engine crate's `testing` module (an engine change) | gate integrity |
| **OOS-DP10-2** | A token's `TriggeredAbilityDef` (`state/game_object.rs:884`) is a **different type** from `AbilityDefinition::Triggered` and carries its own targets. Neither the audit's regex, PB-DP8's typed walk, nor this gate's `"Triggered"` predicate covers it — so a granted CR 603.3d target choice is an uncounted class-B site | correctness, uncounted |
| **OOS-DP10-3** | `Effect::AddManaFilterChoice`'s 0-`Complete` was **hand-maintained with nothing holding it** until T7 (the SR-33 gate bars `AddManaChoice`, a different key). The underlying 3-way filter-land choice (CR 605.1a) is still unimplemented — `effects/mod.rs:2823` always adds one of each colour; the 7 filter lands stay `known_wrong` (PB-RS2) | correctness, deferred |
| **OOS-DP10-4** | §10's "new `Command` variant → accepted-and-discarded field (DP-24)" trigger is **unmechanized**. An SR-15-style source scan for `_`-bound `Command` fields in handlers is feasible test-only and unbuilt | gate coverage |
| **OOS-DP10-5** | `Effect::LookAtTopThenPlace.optional` is inert **by construction** (`effects/mod.rs:5797-5802` destructures `optional: _` and says so) yet is serialized and hashed — the DP-24 accepted-and-discarded class, on an `Effect` rather than a `Command`. Sweep for siblings (`SearchLibrary.reveal` is the other known one, OOS-DP9-9) | correctness, narrow |
| **OOS-DP10-6** | **The successor queue's input.** The still-auto rows ranked by T9's measured `Complete` counts: `proliferate` ≈25, `discard_cards` ≈23, `sacrifice_permanents` ≈11, `may_pay_then_effect` ≈11, `choose_color_or_type` ≈10, `look_at_top_or_route` ≈10, `counter_unless_pays` 7, `modal_trigger` 5, `change_targets`/`put_on_library`/`bolster_amass` 3, `connive` 2, `discover` 1. PB-DP9's `AnswerEffectChoice` channel generalises to most of them "for the cost of one `EffectChoiceQuestion`/`Answer` variant pair and zero new plumbing" (§4.9's own note) — so the ranking is by def count, not by machinery | queue input |
| **OOS-DP10-7** | `GameEvent::reveals_hidden_info()` and `private_to()` both end in `_ =>`, so a new variant silently answers `false` / `None`. §10's trigger exists but nothing enforces it. A roster digest (T15b) is the cheap fix if not taken in-batch | gate coverage |
| *(conditional)* | any def found **live-wrong** (class D, not merely un-consulted) during the sweep — **file, do not demote** (§5.3) | correctness |

---

## 14. Audit updates required (criterion 5557)

- **§3.1** — add a note above the table: the counts are a 2026-07-26 source-regex estimate,
  **superseded** by `core::decision_gate::decision_site_reconciliation_report`, which prints the
  enumerated numbers on every `cargo test`. Fix the two wrong CR cites (`701.8` → **701.9 /
  701.9b**; `701.20` → **608.2d + 401.4**). Note that the `WheelDraw` needle has no runtime key
  and that the `WheelHand` row is **NO-DECISION**.
- **§4.9** — fix the Bolster cite (`701.29a` → **701.39a**; the source is already right).
- **§3.1's control-group paragraph** — correct the claim that the four zero rows "get there two
  different ways": `AddManaFilterChoice` and `TheRingTemptsYou` were held by **nothing** until
  T7 (§4 note 5).
- **§5** — mark the still-open class-B rows as *"baselined by PB-DP10's gate; still class B"*.
  Do not change any class.
- **§6** — add a bullet: the corpus counts are no longer a regex sweep; name the test and the
  file, and record the compound-row and unit-variant mechanisms as the two reasons the numbers
  moved.
- **§8, `PB-DP10` row** — SHIPPED; file paths; measured taxonomy split (4 served / 14 auto /
  2 gated / 1 no-decision) and both unions; the two new hard zeros; the unit-variant finding as
  the batch's headline; PROTOCOL 31 / HASH 68 **unmoved** as predicted. **And mark the PB-DP
  suite COMPLETE.**
- **§8.1** — **close OOS-DP7-7** (the §3.1 re-derivation is now computed, printed and ratcheted);
  file OOS-DP10-1..7.
- **§10** — record the 3-of-8 mechanization ledger from §9 verbatim, including the honest note
  that T15's marginal value is the *obligation*, not the notice.

---

## 15. Verification checklist

- [ ] Step 0 probes P0-a..P0-h run and recorded; any falsified premise reflected in the code
- [ ] `cargo build --workspace` clean
- [ ] `cargo test --all` green; **tests 3,910 → 3,910 + ~16**
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` clean (SR-35)
- [ ] `git diff --name-only main -- crates/engine/src crates/card-types/src` is **EMPTY** (the hard constraint, machine-checked)
- [ ] `PROTOCOL_VERSION == 31` and `HASH_SCHEMA_VERSION == 68`; `protocol_schema::` and `hash_schema::` green; **no history row appended or edited**
- [ ] `core/main.rs` has both new `mod` lines, alphabetically; `no_stray_test_binaries` green
- [ ] T9's printed report captured into the audit §8 row and the close-out
- [ ] T2 verified to fail against the inherited object-key-only walk (the fail-before is real, not asserted)
- [ ] SR-33's three stub assertions and their messages **byte-identical** to `main`
- [ ] 0 card-def source edits (or: each one argued from oracle text via the mtg-rules MCP, in the commit message)
- [ ] Audit §3.1 / §4.9 / §5 / §6 / §8 / §8.1 / §10 updated; `<!-- last_updated -->` bumped
- [ ] `memory/primitive-wip.md` + `memory/workstream-state.md` close-outs written
