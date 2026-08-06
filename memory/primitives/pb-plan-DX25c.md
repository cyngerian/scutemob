# Primitive Batch Plan: PB-DX25c — CR 115.7a's "another **legal** target"

**Generated**: 2026-08-06
**Primitive**: the victim spell's own `TargetRequirement` list becomes **stored state**
(`StackObject.target_requirements`, hashed), and the "which object or player may become the new
target of this stack object" decision is encoded **once**, in a new `rules::retarget` module, as a
delegation to `casting::validate_targets_inner` — the same collective legality arithmetic the cast
itself was validated by (CR 115.7e).
**Seed**: `OOS-DX25b-3` (registry row `docs/audits/decision-point-audit.md:1370`)
**Queue**: `memory/primitives/seed-rerank-2026-08-02.md` §4 row **7c** (INSERTED 2026-08-06,
user-approved)
**CR rules**: 115.7 / 115.7a / 115.7b / 115.7c / 115.7d / 115.7e / 115.7f, 115.3, 115.4, 109.5,
601.2c, 608.2b, 707.10, 400.7, 104.3a
**Cards affected**: **2 live-wrong `Complete` deck-legal defs repaired** (`misdirection`,
`bolt_bend` — comment-only edits); 1 `partial` def (`untimely_malfunction`) improves on the same
path; 1 def (`deflecting_swat`) unchanged by construction (`must_change: false`, `OOS-DX25b-4`).
**Predicted `completeness` flips: 0.**
**Dependencies**: PB-DX25 (`state::stack_registry::card_in_stack_zone`) and PB-DX25b
(`stack_index_for_announced_target`, and the whole reason the defect is reachable). PB-DX20's
`aura_spell_target_requirements` is on the requirement-derivation path this batch records from.
**Deferred items from prior PBs**: none this batch is obliged to take. The standing undispatched
set (feedback rows 2/4/5/6/7/8, `OOS-DX22-8`, `OOS-DX32-1`, v3 §4 not re-rowed with DX42a/b,
`OOS-ADJ-1..7` not rowed into §8.1, `scutemob-127`) is unchanged and out of scope.

**Baseline to re-measure BEFORE any edit** (do not trust these — re-measure and record in
`memory/primitives/pb-DX25c-execution-notes.md`): workspace tests **4,469 / 0 / 5**; PROTOCOL
**35**; HASH **73**; coverage **1,133/1,803 = 62.8%**.

---

## §1 Premise re-verification

Every claim in the dispatch brief and in the `OOS-DX25b-3` registry row was checked against source
at HEAD in this worktree (`/home/skydude/projects/scutemob/.worktrees/scutemob-205`). Line numbers
below are measured, not quoted.

| # | Claim (source) | Verdict | Evidence |
|---|---|---|---|
| 1 | The defect lives in `Effect::ChangeTargets`'s `Target::Object` branch at `effects/mod.rs:7628-7663` (brief) / `:7619-7654` (registry row) | **CONFIRMED as to substance; BOTH line ranges are stale** | Measured: the `Effect::ChangeTargets` arm is `effects/mod.rs:7539-7681`. The `Target::Player` branch is `:7589-7624`; the `Target::Object` branch is `:7625-7660`; the `KNOWN LIMITATION` comment is `:7630-7635`; the candidate build is `:7637-7651`. PB-DX25b's own edits (the `pos` resolution at `:7553-7558` and the `real_stack_id` capture at `:7570-7580`) are what moved them. **Use the measured range; both published ranges are ~6 and ~9 lines off respectively.** |
| 2 | The candidate set is "every `state.objects` entry whose `zone == spell_target.zone_at_cast`, excluding the current target, sorted by `ObjectId`, first taken"; no requirement / protection / hexproof / shroud / controller / card-type check | **CONFIRMED, verbatim** | `effects/mod.rs:7637-7651`. The filter is `**oid != *current_oid && original_zone.map(\|z\| obj.zone == z).unwrap_or(true)`. `candidates.sort(); candidates.into_iter().next()`. Nothing else. The `unwrap_or(true)` clause is worth noting on its own: when `zone_at_cast` is `None` the candidate set is **every object in the game, in every zone**, including hands and libraries — a hidden-information-adjacent widening the registry row does not mention. Reachable only for a target recorded with `zone_at_cast: None`, which `casting.rs:6345` produces for a stale id; recorded, not exploited (§8 R6). |
| 3 | The behaviour is pinned wrong-way-round by `t9_object_target_redirect_ignores_the_original_requirement` at `pb_dx25b_announced_stack_target_space.rs:1243`, whose doc instructs this batch to invert it | **CONFIRMED.** The `#[test]` is at `:1242`, the fn at `:1243`; the instruction is at `:1236-1241` | The fixture is a **real two-cast chain** (`Command::CastSpell` for a purpose-built "Destroy target creature", then for the real `cards::defs::misdirection::card()`), resolved by real `PassPriority`. It is exactly the fixture shape this batch needs, and it needs no rebuild — only inversion. |
| 4 | `misdirection.rs:25` and `bolt_bend.rs:38` carry pointer comments to that test and must be updated (comment-only) | **CONFIRMED**, and the blocks are larger than "a pointer": `misdirection.rs:12-33` is a 22-line COMPLETENESS DECISION block naming `OOS-DX25b-3`, `OOS-DX25b-2` and the successor batch; `bolt_bend.rs` carries the sibling block plus an `OOS-DX25b-1` note | Both must be rewritten to record closure of the `OOS-DX25b-3` half **and to leave the `OOS-DX25b-1`/`OOS-DX25b-2` halves standing** — those are not closed here. **Both edits are `//` comments only**; neither def has a `Completeness::partial(...)` string to touch (both are `Complete` by derive), so SR-35 / coverage risk is nil if the rule is obeyed. |
| 5 | `casting::validate_targets_inner(state, targets, requirements, caster, source_chars, self_id) -> Result<Vec<SpellTarget>, _>` at `casting.rs:6042` is the one authoritative legality arithmetic, doing a collective two-pass best-fit slot assignment | **CONFIRMED**, and it is `pub(crate)`, so `effects/mod.rs` and a new `rules::` module can both call it | `casting.rs:6042-6173`. Count range `:6055-6063`; pass 1 `:6101-6115`; pass 2 `:6118-6134`; unassigned rejection `:6137-6144`; **CR 115.3 inter-target distinctness at `:6154`**; tail `validate_mapped_targets` at `:6165`/`:6228`. `queries::legal_targets_per_slot` (`queries.rs:214-259`) already delegates to it one candidate at a time (`:245-253`). |
| 6 | *(implicit in the brief)* Storing the `TargetRequirement` **list** on `StackObject` is enough to "consult it at redirect" | **PARTLY REFUTED — the list alone cannot tell you which requirement governs `targets[i]`.** | `validate_mapped_targets`' own doc (`casting.rs:6224-6227`): *"The returned `Vec<SpellTarget>` preserves declaration order (positions are NOT reordered to match requirement/slot order)."* The target→slot map built at `:6082-6162` is discarded. So a stored list supports **set-level** validation, not per-index validation. That is not a problem — it is the *correct* reading of **CR 115.7e** ("only the final set of targets is evaluated") — but the design must validate the **whole candidate final set** through `validate_targets_inner`, never index-match a stored list against `targets[i]`. §3.3. |
| 7 | The `Target::Player` branch has the same defect class: prefers `ctx.controller`, checks only `has_lost` (not `has_conceded`), never checks the requirement | **CONFIRMED on both counts, and BOTH are independently reachable.** In scope. | (a) `effects/mod.rs:7595-7599` and `:7607-7614` test `ps.has_lost` only; `validate_mapped_targets` (`casting.rs:6245`) rejects on `has_lost \|\| has_conceded`, and `queries::legal_targets_per_slot` (`queries.rs:225`) does too. **`handle_concede` (`engine.rs:2757-2770`) sets `has_conceded = true` and does NOT set `has_lost`** — verified by reading the function; the only `has_lost = true` writes are in `sba.rs:268/283/307`, `engine.rs:2552/2599`, `replacement.rs:1061`, `effects/mod.rs:4503`, `abilities.rs:9634`, none of them on the concede path. So a conceded-but-not-yet-lost player is a redirect candidate today. (b) No `TargetRequirement` is consulted anywhere in the branch; `TargetRequirement::TargetOpponent` exists (`hash.rs:435-438`, PB-EF6) and appears on **8 corpus defs** (`vengeful_bloodwitch`, `shaman_of_the_pack`, `raiders_wake`, `forbidden_orchard`, `fell_specter`, `blood_tribute`, `blessed_alliance`, `ajani_sleeper_agent` — measure which are single-target *spells* at implement time). Reachable configuration: p1 casts a "target opponent …" spell at p2, then Misdirects **their own spell**; the branch prefers `ctx.controller` = p1, and p1 is not an opponent of p1 → an illegal target, chosen deliberately. **Shipping only the Object half would leave this live in the same function.** |
| 8 | `rules::copy::copy_spell_on_stack`'s `_choose_new_targets` parameter is a reserved/unused latent site | **CONFIRMED** (`copy.rs:140-145`, the parameter is `_`-prefixed and the doc at `:138-139` says so). **OUT of scope for behaviour, IN scope for the new field.** | CR 707.10a/115.7d: a copy's "you may choose new targets" is legally satisfiable by leaving them unchanged, so today's behaviour is CR-correct-if-unhelpful. But `copy.rs:159-…` builds the copy field-by-field with a CR rationale per field and **must propagate `target_requirements`** (CR 707.10: the copy has the same characteristics and the same targets, so it must carry the same targeting requirements — otherwise a copy would be retargetable with no legality basis). |
| 9 | PB-ENG2 established **12** stack-push/announce sites; find them by the same method and report your own number | **The 12-site census is the wrong census for this batch, and I report two different measured numbers instead.** | (a) Sites that **write `StackObject.targets`**, i.e. that must also write `target_requirements` with a real value: **9** — `casting.rs:4556`, `engine.rs:3703`, `copy.rs:163`, `abilities.rs:1405`, `:1788`, `:2003`, `:8809`, `:9431`, `:10932` (measured by `Grep '\.targets = \|targets: '` over `crates/engine/src`). (b) Sites that **construct a `StackObject` struct literal**, i.e. that break the build until the field is added: **~42** — 9 in `crates/engine/src` production (`casting.rs:4552`, `resolution.rs:8501`, `engine.rs:3121`, `:3185`, `:3695`, `:3867`, `copy.rs:159`, `:392`, `:630`), 3 in `casting.rs`'s in-source `mod tests` (`:8198`, `:8494`, `:8579`), and ~30 in `crates/engine/tests/`. `StackObject::trigger_default` (`stack.rs:517`) is a **function**, so its ~20 callers (including the only `crates/simulator` and `crates/view-model` uses) need no edit — only the function body does. **Re-measure with `git grep -n 'StackObject {'` before starting; do not trust ~42.** |
| 10 | *(brief question)* Is a new `StackObject` field a wire change under SR-8? | **NO — and this is machine-decided, not a judgement call.** | `crates/engine/tests/core/protocol_schema.rs:116-117`: `const CLOSURE_MUST_NOT_CONTAIN: [&str; 4] = ["GameState", "PlayerState", "StackObject", "CardDefinition"];`. `StackObject` is *required to be absent* from the wire closure; if it were reachable from `Command`/`GameEvent`/`ReplayLog` the gate would already be red. So the protocol fingerprint cannot move. **PROTOCOL 35 expected unmoved — still gate-EXECUTED, never asserted (§7).** |
| 11 | HASH 73 → 74 predicted | **CONFIRMED as the correct prediction, and the bump is FORCED rather than optional** | `hash.rs:4336-4452` is `impl HashInto for StackObject`; `hash_schema.rs:1305` pins `const NOT_HASHED: &[(&str,&str)] = &[]` (every field of every hashed struct is fed to `HashInto`) and `:1309-1316` lists `StackObject` in `COVERAGE_MUST_INCLUDE`. Adding an unhashed field makes that gate red until the field is either hashed or allowlisted with a written rationale. It must be **hashed**: it changes what a legal retarget is, so two states differing only in it are genuinely different positions. |
| 12 | *(brief)* `StackObject` is defined at `crates/card-types/src/state/stack.rs:159` | **CONFIRMED** (`:158-500`, `impl` block `:501-570`, `trigger_default` `:517-569`) | — |
| 13 | **NEW — not in the brief.** The DSL's own doc *already describes the behaviour this batch implements* | Finding, recorded | `crates/card-types/src/cards/card_definition.rs:2457-2460`: *"Deterministic fallback for `must_change: true`: retargets to the effect's controller (if legal). If the controller is not a legal target, picks the first legal alternative (smallest PlayerId/ObjectId)."* At HEAD **"legal" is false for objects and for players alike** — the doc has been aspirationally wrong since PB-J. This is the `memory/conventions.md` "aspirationally-wrong comment is a correctness hazard" class, and it is also useful: it is contemporaneous evidence that CR 115.7a legality was always the intent. The doc still needs a small edit (§3.6) for the all-or-nothing clause. |
| 14 | **NEW — not in the brief.** `PB-DX25b`'s R4 gate carries a `body.len() >= 200` non-vacuity floor over the `Effect::ChangeTargets` arm body | Risk, must be measured | `pb_dx25b_announced_target_roster.rs:408-414`. This batch **shrinks that arm substantially** (the whole candidate enumeration moves into `rules/retarget.rs`). If the comment-stripped body drops below 200 chars the gate goes red for a reason unrelated to its invariant. Re-measure and re-aim **deliberately, with a revert proof**, exactly as PB-DX25b re-aimed PB-DX25's G2 (§5.4, §8 R7). |
| 15 | **NEW — not in the brief.** Six existing tests will go RED and must be repaired with real requirement lists | Expected, and it is the batch's own headline non-vacuity evidence | `crates/engine/tests/rules/copy_redirect.rs`'s five `ChangeTargets` tests (`:280`, `:328`, `:371`, `:412`, `:451`) plus `:544`, and `pb_ef11_spell_single_target.rs:372`. Every one of them builds a `StackObject` by hand with **no requirement list at all** and asserts a redirect happens. Under fail-closed semantics (§3.4) they will all stop redirecting. That is the point: **each one was asserting an unfiltered redirect.** Repair by giving each fixture the requirement the pretend-spell would really have had. |

---

## §2 Census — every consumer of the "which object or player may become the new target" decision

### 2.1 Method (re-derivable, not to be trusted)

PB-DX25's durable lesson is *"an enumeration is only as wide as the variant list it walks"*, and
PB-DX25b's is *"the brief's site list is a FLOOR"*. This census is built three independent ways and
the results reconciled:

1. **By the decision's own vocabulary.** `Grep 'new_targets|choose_new_targets|retarget|
   TargetsChanged'` over every `crates/*/src/**`. Result: 21 hits, all accounted for in §2.2/§2.3.
2. **By the mutation.** Every write to `StackObject.targets` after the object is on the stack:
   `Grep '\.targets = ' crates/engine/src`. Result: exactly one post-push write —
   `effects/mod.rs:7668`. Every other `.targets =` is a *construction-time* write (§2.4).
3. **By the event.** Every emitter of `GameEvent::TargetsChanged`: exactly one
   (`effects/mod.rs:7670`).

All three agree. **There is exactly ONE retarget decision site in the tree**, and it has two
branches. That is a stronger statement than "the brief named one of two", and it is what makes the
"encode it once" fix small.

### 2.2 IN SCOPE — the decision itself

| # | Site | What it decides | Verdict at HEAD |
|---|---|---|---|
| D1 | `effects/mod.rs:7625-7660` — `ChangeTargets` / `Target::Object` | which object becomes the new target | **live-wrong** (the seed). No requirement, protection, hexproof, shroud, controller or type check. `zone_at_cast: None` widens the candidate set to every zone (§1 fact 2). |
| D2 | `effects/mod.rs:7589-7624` — `ChangeTargets` / `Target::Player` | which player becomes the new target | **live-wrong, two ways** (§1 fact 7): `has_conceded` unchecked (CR 104.3a), and no requirement check (`TargetOpponent`). **Brief did not name it; IN SCOPE** — same function, same rule, same commit, and a half-fix leaves a live defect one branch away. |
| D3 | `effects/mod.rs:7663-7678` — the `changed` write + event | whether a partial change is committed | **CR 115.7a's all-or-nothing clause is unimplemented** (`changed` is set per-target). IN SCOPE by construction: the fix's shape decides it either way, so it must be decided deliberately (§3.5). |

### 2.3 IN SCOPE for the new FIELD only — not for behaviour

| # | Site | Action |
|---|---|---|
| P1 | `copy.rs:159-…` `copy_spell_on_stack` | propagate `target_requirements: original.target_requirements.clone()` with a CR 707.10 rationale line, matching the file's per-field idiom. **`_choose_new_targets` stays unused** — CR 115.7d permits the leave-unchanged fallback; wiring it is a behaviour change on a path this batch was not dispatched for. Recorded as `OOS-DX25c-2`. |
| P2 | The 9 `.targets`-writing sites (§1 fact 9a) | write the requirement list that was validated **at that site**. |
| P3 | The ~42 `StackObject { … }` literals (§1 fact 9b) | `target_requirements: vec![]` where the object announces no targets; the real list where it does. |

### 2.4 OUT OF SCOPE — stated, with the reason

| Site | Why out |
|---|---|
| `Effect::CopySpellOnStack` (`effects/mod.rs:7495-7530`) | makes copies; chooses no targets. PB-DX25b already routed its lookup. Untouched. |
| `casting.rs`'s C1/C2 arms, `Effect::CounterSpell`, `resolution.rs::counter_stack_object`, `abilities.rs:6747`, Ward's `DeclaredTarget` fallback (`effects/mod.rs:~7690`), `PlayerTarget::ControllerOf` | PB-DX25/DX25b territory. **Byte-unchanged; prove with `git diff`.** |
| `mtg_simulator::invariants::stack_card_of` | deliberately duplicated, `stack_registry.rs:36-48`. Untouched. |
| `crates/simulator/src/targeting.rs::plan_targets` | announces the *Misdirection* target (which spell to retarget). The retarget itself is engine-side. **No production line changes**; it is the *channel* the AC-6304 probe drives (§5.3). |
| `deflecting_swat` (`must_change: false`, CR 115.7d) | deterministic no-op before and after. `OOS-DX25b-4` stays open. The R2 roster message must keep saying membership ≠ works. |
| `hydroelectric_specimen.rs:27-33` | its `Completeness::partial(...)` string says *"must_change retargets to the effect's controller"* — still substantially true after this batch (the controller preference is **kept**, §3.3). **Leave byte-unchanged**; editing a `partial(...)` string is a card-def content edit and PB-DX25b's C2 already learned that lesson the hard way. |

### 2.5 Documentation consumers whose text becomes false

| File:line | Today | Action |
|---|---|---|
| `crates/simulator/src/decision_coverage.rs:122-126` | `("change_targets", "AutoChosen -- declines an optional retarget and picks the lowest id for a mandatory one, inline (CR 115.7d)")` | reword to "...picks the lowest-id **legal** candidate...". **Class stays `AutoChosen`** (the engine still chooses inline, with no artefact), so `MAX_AUTO_CHOSEN_COMPLETE_UNION` (`decision_gate.rs`, pinned 80) is **unmoved**. `runtime_decision_coverage_roster_matches_rows` compares **ids**, not reasons (PB-DX32 Stage 6), so the reword cannot redden it — verify by execution. **This is the batch's only `crates/simulator` line.** |
| `crates/engine/tests/core/decision_gate.rs` `ROWS` `change_targets` row | same wording | same reword. `("Bolt Bend"/"Deflecting Swat"/"Misdirection", &["change_targets"], None)` rows at `:391`/`:403`/`:435` stay unchanged (the decision is still auto-chosen). |
| `crates/card-types/src/cards/card_definition.rs:2448-2468` | already claims "first **legal** alternative" (§1 fact 13) | correct the all-or-nothing sentence and state that legality is now delegated (§3.6). **The batch's only `crates/card-types` production line beyond the struct field itself.** |

---

## §3 The structural fix

### 3.1 The stored requirement list

**File**: `crates/card-types/src/state/stack.rs`, in `pub struct StackObject` (after `targets`, so
the two read together).

```rust
    /// CR 115.7a / CR 115.7e: the `TargetRequirement` list this stack object's
    /// targets were validated against at announcement time (CR 601.2c / 602.2b).
    ///
    /// Recorded, never re-derived. `casting::handle_cast_spell` chooses between
    /// the flat `Spell.targets` list, the per-mode slice
    /// (`ModeSelection.mode_targets`), the aftermath half's list, the
    /// Aura-synthesised list (CR 303.4a) and the empty list (overload,
    /// CR 702.96b) BEFORE it validates; the result of that choice is what lands
    /// here, so a later reader cannot disagree with the validation that actually
    /// happened. Re-deriving it from the card definition at read time is wrong
    /// for at least the aftermath case (a spell cast as its aftermath half sits
    /// in `ZoneId::Stack`, so the `casting_with_aftermath` test that selects the
    /// aftermath list cannot be reproduced from the stack-resident object).
    ///
    /// Empty means "this stack object announced no targets" OR "no list was
    /// recorded at this push site". Readers must FAIL CLOSED on a non-empty
    /// `targets` with an empty list — see `rules::retarget`.
    ///
    /// Propagated to copies (CR 707.10: a copy has the same characteristics and
    /// the same targets, so it has the same targeting requirements).
    #[serde(default)]
    pub target_requirements: Vec<TargetRequirement>,
```

`TargetRequirement` is already in scope in this file's imports chain (`SpellTarget` and
`AdditionalCost` are); confirm the exact `use` path at implement time.

**Population**, at the 9 writing sites (§1 fact 9a):

| Site | Value |
|---|---|
| `casting.rs:4552-4556` (`handle_cast_spell`) | `mode_targets_active.clone().unwrap_or_else(\|\| requirements.clone())` — the exact expression chosen at `:3696-3727`. Both bindings are in scope at `:4552` (same function). **Hoist the choice into a single `let announced_requirements = …;` at `:3696` and use it in BOTH places**, so the validated list and the recorded list are the same value, not two expressions that happen to agree. |
| `engine.rs:3695-3703` (loyalty ability) | `ability_targets.clone()` (`:3623`, the list `:3631-3638` validated against). |
| `copy.rs:159-163` | `original.target_requirements.clone()` (CR 707.10). |
| `abilities.rs:1396-1405` (activated ability) | the same `active_reqs` / `target_requirements` the `:487`/`:501` validation used. |
| `abilities.rs:1788`, `:2003`, `:8809`, `:9431`, `:10932` | the list that governed those targets, where one exists; `vec![]` **with a one-line comment saying why** where none does. |

The other ~33 literal sites get `target_requirements: vec![]`. **Six test fixtures must get real
values** (§1 fact 15).

### 3.2 The shared decision — a new module

**File**: `crates/engine/src/rules/retarget.rs` (new). `pub mod retarget;` added to
`rules/mod.rs` between `replacement` and `resolution`.

**Why NOT `state::stack_registry`** (the brief's expectation, argued rather than ignored): that
module is deliberately a **pure classification module with no `GameState` dependency**
(`stack_registry.rs` module doc; PB-DX25b plan §3.1 states it as a design property and the
`&imbl::Vector<StackObject>` parameter shape enforces it). This decision needs `&GameState` **and**
`rules::casting::validate_targets_inner` — putting it there would make `state` depend on `rules`,
inverting the crate's layering and falsifying a module doc that the last two batches leaned on.
**Why not `rules/casting.rs`**: it is already ~8,700 lines, and a separately-named module is what
makes the R4-style source gate expressible by name. **Why not inline in `effects/mod.rs`**: that is
where the defect is; a decision that is supposed to be made once must be extractable and gateable.

**API** (all `pub(crate)`):

```rust
/// CR 115.7a — the new target set for a stack object whose targets are being
/// changed, or `None` if CR 115.7a's fallback applies (leave everything alone).
///
/// `chooser` is the player the changing effect instructs to choose (CR 115.7:
/// "allow a PLAYER to change the target(s)"), i.e. `EffectContext.controller`.
/// It is used ONLY to order candidates — never to decide legality. Legality is
/// decided relative to the VICTIM's controller, because CR 109.5 makes "you" on
/// the victim spell mean the victim spell's controller. Conflating the two is
/// the class of error that made Misdirection redirect a "target opponent" spell
/// onto its own caster.
pub(crate) fn plan_target_change(
    state: &GameState,
    stack_index: usize,
    chooser: PlayerId,
) -> Option<Vec<SpellTarget>>;

/// The candidate universe, in deterministic order. Mirrors
/// `rules::queries::legal_targets_per_slot`'s universe exactly (see its doc for
/// why those three zones and no others), with one inherited preference: the
/// chooser is offered first. Public to the crate so the gate and the probes can
/// assert the two universes are the same set.
pub(crate) fn retarget_candidates(state: &GameState, chooser: PlayerId) -> Vec<Target>;
```

**`plan_target_change` body, in order**:

1. `let so = state.stack_objects.get(stack_index)?;` — clone what is needed, drop the borrow.
2. `if so.targets.is_empty() { return None; }` (unchanged from HEAD).
3. `let reqs = so.target_requirements.clone(); if reqs.is_empty() { return None; }`
   — **fail closed** (§3.4).
4. `let victim_card = state::stack_registry::card_in_stack_zone(&so.kind);`
   `let source_chars = victim_card.and_then(|id| rules::layers::calculate_characteristics(state, id));`
   — CR 702.16b protection checks read the *victim spell's* characteristics, which is what
   `handle_cast_spell` passed as `Some(&chars)` at `casting.rs:3726`. `victim_card` also serves as
   `self_id` (CR 601.2c self-targeting prevention, and `TargetFilter.exclude_self`), matching
   `casting.rs:3726`'s `card` argument. For a non-card-owning stack kind both are `None`; that case
   is unreachable here (§8 R2) and is recorded as `OOS-DX25c-3`, whose fix is the
   `stack_registry::source_of` sibling `OOS-DX25-4` already asks for.
5. `let candidates = retarget_candidates(state, chooser);`
6. Greedy per index, each trial validated as a **whole set** (CR 115.7e):
   ```rust
   let mut next: Vec<Target> = so.targets.iter().map(|st| st.target).collect();
   for i in 0..next.len() {
       let current = so.targets[i].target;
       let pick = candidates.iter().find(|c| {
           **c != current && {
               let mut trial = next.clone();
               trial[i] = **c;
               casting::validate_targets_inner(
                   state, &trial, &reqs, so.controller, source_chars.as_ref(), victim_card,
               ).is_ok()
           }
       })?;                       // CR 115.7a: one index with no legal change ⇒ change NOTHING
       next[i] = *pick;
   }
   ```
7. **CR 115.7e final-set re-validation** (the greedy loop validated mixed sets, not the final one):
   `casting::validate_targets_inner(state, &next, &reqs, so.controller, source_chars.as_ref(), victim_card).ok()?;`
8. Rebuild `zone_at_cast` from the **new** target's actual zone — `Target::Object(id) =>
   state.objects.get(&id).map(|o| o.zone)`, `Target::Player(_) => None` — mirroring
   `casting.rs:6345` and `engine.rs:3683-3689`. **This is a correction**: HEAD copies the
   *original* target's `zone_at_cast` onto the new target (`effects/mod.rs:7655`), which is a
   latent CR 608.2b bug the moment the new target is in a different zone. Harmless at HEAD only
   because the same-zone filter made it self-consistent; that filter is being removed.

**`retarget_candidates` order** (deterministic, and chosen to preserve every currently-legal
observable):

1. `Target::Player(chooser)` if `!has_lost && !has_conceded` — the preference inherited from
   `effects/mod.rs:7600-7601`. **Kept deliberately**: this batch changes *legality*, not
   *preference*, and dropping it would move T1/T2 and five `copy_redirect.rs` fixtures for a reason
   unrelated to the seed.
2. remaining `state.turn.turn_order` players, in seat order, alive by both flags
   (`queries.rs:223-229`'s exact test).
3. `state.objects()` in ascending `ObjectId` (`imbl::OrdMap` iteration order) whose zone is
   `Battlefield | Stack | Graveyard(_)` — `queries.rs:230-237`'s exact universe.

### 3.3 What changes observably, and what deliberately does not

* **Players and objects become one candidate universe.** A `TargetCreatureOrPlayer` spell targeting
  a player can now be redirected to a creature and vice versa. That is CR 115.7a as written ("another
  legal target", with no kind restriction) and is *why* the two branches collapse into one decision
  rather than two. It is also a widening, so it needs a probe (§5.2 T6).
* **The `zone_at_cast` same-zone filter is dropped.** CR 115.7a imposes no such restriction;
  Misdirection's 2004-10-04 ruling ("can be changed to any other spell on the stack") and CR 115.4
  both describe legality, not zone identity. The zone restriction that *does* exist is inside each
  `TargetRequirement` arm (`TargetCreature` requires Battlefield, `TargetSpell` requires Stack,
  `TargetCardInGraveyard` requires a graveyard — `queries.rs:187-196` enumerates the mapping), and
  it is now enforced by the validator rather than approximated by a zone equality.
* **The `ctx.controller` preference survives** (§3.2). Stated so nobody reads its survival as an
  oversight.
* **`must_change: false` is unchanged.** CR 115.7d's leave-everything-unchanged fallback is legal;
  `OOS-DX25b-4` stays open and the R2 roster message must keep saying so.
* **`Effect::CopySpellOnStack` is unchanged.** `OOS-DX25c-2`.

### 3.4 Fail-closed on a missing requirement list — argued

An empty `target_requirements` with a non-empty `targets` means *"nobody recorded what this object
was validated against."* Three options were considered:

* *(i) treat it as "no requirements" and let `validate_targets_inner` do existence-only validation*
  (`casting.rs:6077-6079`). **Rejected**: that is exactly HEAD's unfiltered behaviour, reintroduced
  behind a friendlier-looking call. A gap in the population would then be invisible.
* *(ii) `Option<Vec<TargetRequirement>>` to distinguish "none" from "unknown".* Rejected as
  redundant: `targets.is_empty()` already discriminates the only benign empty case, and an `Option`
  adds a serde/hash shape for a distinction one guard makes.
* **(iii) fail closed — no requirements recorded ⇒ no change.** **Chosen.** It can only ever produce
  "the target was not changed", which CR 115.7a explicitly sanctions as the fallback; it can never
  produce wrong game state; and it converts every population gap into a *visible, testable* loss of
  function rather than a silent illegal redirect. It is also what makes §1 fact 15's six red tests
  informative instead of merely inconvenient.

### 3.5 CR 115.7a's all-or-nothing clause — decided explicitly

CR 115.7a: *"If all the targets aren't changed to other legal targets, none of them are changed."*
HEAD sets `changed` per-target and commits partial changes.

**Decision: IMPLEMENT it, because the chosen fix shape decides it either way and the honest choice
is the CR one.** Step 6's `?` makes any index with no legal replacement abort the whole plan, and
step 7 re-validates the final set; either failure returns `None` and nothing is written.

**Its reachability is measurable and is predicted to be zero**, which must be *stated* so nobody
reads this as a claimed card-yield improvement. Every `must_change: true` corpus user
(`misdirection`, `bolt_bend`, `untimely_malfunction`) requires the victim to have **exactly one**
target (`TargetSpellWithSingleTarget` / `TargetSpellOrAbilityWithSingleTarget`); with n = 1
all-or-nothing is vacuous. The only `TargetSpell`-without-single-target user is `deflecting_swat`,
and its `must_change: false` makes the whole branch a no-op. **R2 must measure and pin this**, not
assume it.

**Known incompleteness, stated rather than glossed**: the per-index search is **greedy**, so for
n > 1 it can fail to find a legal assignment that exists (Bolt Bend's 2024-11-08 ruling: *"You must
change the target if possible"*). A complete search is combinatorial in the candidate universe
(~100 objects), the failure direction is "leave unchanged" — which CR 115.7a itself prescribes as
the fallback — and the population is empty. Filed as `OOS-DX25c-1` with the reachability measurement
attached.

**115.7b / 115.7c are NOT implemented and are not needed.** No corpus card says "change a target"
or "change any targets" — R2 must confirm this by enumeration, not by reading this sentence.
**115.7f (divided/distributed effects) is not implemented and is not perturbed**: the redirect only
rewrites `SpellTarget.target`, never any division, so the original division is preserved by
construction. Say so; do not claim it as work.

### 3.6 The `effects/mod.rs` arm after the change

The `Target::Player` and `Target::Object` match arms, the candidate builds, the `has_lost` checks
and the `changed` flag all **disappear**. What remains inside `Effect::ChangeTargets` (after the
existing `pos` resolution and the `must_change` guard):

```rust
let Some(new_targets) = crate::rules::retarget::plan_target_change(state, pos, ctx.controller)
else {
    continue;                      // CR 115.7a fallback: targets unchanged, no event
};
let old_targets = state.stack_objects[pos].targets.clone();
let real_stack_id = state.stack_objects[pos].id;   // PB-DX25b E5: the STACK-ENTRY id
if let Some(so) = state.stack_objects.get_mut(pos) {
    so.targets = new_targets.clone();
}
events.push(GameEvent::TargetsChanged { stack_object_id: real_stack_id, old_targets, new_targets });
```

`GameEvent::TargetsChanged`'s shape, its `stack_object_id` contract and PB-DX25b's E5 justification
comment are **byte-preserved**. No event, `Command`, `Effect` or `TargetRequirement` shape changes.

`card_definition.rs:2457-2460`'s doc gains one sentence: that CR 115.7a is all-or-nothing across
targets, that legality is delegated to the same validator the cast used, and that the
controller-first behaviour is a preference among legal candidates.

### 3.7 What a future change is forced to do

* A new `StackObject` field is still forced into the hash by `hash_schema.rs`'s `NOT_HASHED = []`
  coverage gate (§1 fact 11).
* A new stack-push site that sets `targets` without `target_requirements` **fails closed** (§3.4)
  and is caught by R3 (§5.4), which asserts every `StackObject` literal in `crates/engine/src` that
  writes a non-empty `targets` also writes `target_requirements`.
* A future author who re-open-codes a candidate scan inside the `ChangeTargets` arm is caught by R4
  (zero `state.objects` enumerations in that arm, ≥1 call to `retarget::plan_target_change`).
* **Residual, stated up front rather than after a reviewer finds it** (PB-DX25b's R5 lesson): none
  of these gates sees a *brand-new* retarget decision invented somewhere else in the tree. §2.1's
  three-way method is re-derivable but not machine-run. R5 (§5.4) narrows this by forbidding a
  second `GameEvent::TargetsChanged` emitter, which is the one thing any real second retarget site
  would have to do — but a site that mutates `StackObject.targets` without emitting the event is
  invisible to it, and `crates/engine/src` is not `pub`-sealed against that.

---

## §4 CR grounding

### 4.1 CR 115.7 (verbatim, MCP, 2026-08-06)

> **115.7.** Some effects allow a player to change the target(s) of a spell or ability, and other
> effects allow a player to choose new targets for a spell or ability.
>
> **115.7a** If an effect allows a player to "change the target(s)" of a spell or ability, each
> target can be changed only to another legal target. If a target can't be changed to another legal
> target, the original target is unchanged, even if the original target is itself illegal by then.
> If all the targets aren't changed to other legal targets, none of them are changed.
>
> **115.7b** If an effect allows a player to "change a target" of a spell or ability, the process
> described in rule 115.7a is followed, except that only one of those targets may be changed (rather
> than all of them or none of them).
>
> **115.7c** If an effect allows a player to "change any targets" of a spell or ability, the process
> described in rule 115.7a is followed, except that any number of those targets may be changed
> (rather than all of them or none of them).
>
> **115.7d** If an effect allows a player to "choose new targets" for a spell or ability, the player
> may leave any number of the targets unchanged, even if those targets would be illegal. If the
> player chooses to change some or all of the targets, the new targets must be legal and must not
> cause any unchanged targets to become illegal.
>
> **115.7e** When changing targets or choosing new targets for a spell or ability, only the final set
> of targets is evaluated to determine whether the change is legal.
>
> **115.7f** A spell or ability may "divide" or "distribute" an effect (such as damage or counters)
> among one or more targets. When changing targets or choosing new targets for that spell or
> ability, the original division can't be changed.

**Engine mapping**: `must_change: true` = 115.7a (`misdirection`, `bolt_bend`,
`untimely_malfunction` mode 1); `must_change: false` = 115.7d (`deflecting_swat`, a no-op,
`OOS-DX25b-4`). 115.7b/115.7c: no corpus user (R2 measures it). 115.7e is what licenses
set-level validation and *forbids* per-index requirement matching (§1 fact 6). 115.7f is
preserved by construction (§3.5).

### 4.2 CR 115.3 (verbatim, MCP)

> **115.3.** The same target can't be chosen multiple times for any one instance of the word
> "target" on a spell or ability. If the spell or ability uses the word "target" in multiple places,
> the same object or player can be chosen once for each instance of the word "target" (as long as it
> fits the targeting criteria). **This rule applies both when choosing targets for a spell or ability
> and when changing targets or choosing new targets for a spell or ability (see rule 115.7).**

The emphasised sentence is the reason to delegate rather than re-derive: `validate_targets_inner`
calls `enforce_inter_target_distinctness` at `casting.rs:6154`, so CR 115.3-at-retarget comes for
free. HEAD implements none of it.

### 4.3 CR 109.5 (verbatim, MCP)

> **109.5.** The words "you" and "your" on an object refer to the object's controller, its would-be
> controller (if a player is attempting to play, cast, or activate it), or its owner (if it has no
> controller). …

This is the authority for `caster = so.controller` (the **victim's** controller) rather than
`ctx.controller` (the Misdirection caster) in §3.2 step 6, and for keeping the two apart in the
API's own parameter names.

### 4.4 CR 115.4 (verbatim, MCP)

> **115.4.** Some spells and abilities that refer to damage require "any target," "another target,"
> "two targets," or similar rather than "target [something]." These targets may be creatures,
> players, planeswalkers, or battles. Other game objects, such as noncreature artifacts or spells,
> can't be chosen.

`TargetRequirement::TargetAny` is the engine's encoding. It is the reason the unified candidate
universe (§3.3) is CR-correct rather than merely convenient: a "3 damage to any target" spell
redirected off a player really can land on a creature.

### 4.5 Rulings worth encoding (MCP; CR governs, rulings only locate edge cases)

* **Misdirection 2004-10-04**: *"If there is no other legal target for the spell, this does not
  change the target."* → the inverted T9's exact assertion.
* **Misdirection 2004-10-04**: *"This does not check if the current target is legal. It just checks
  if the spell has a single target."* → the `TargetSpellWithSingleTarget` announcement is unchanged
  by this batch; do not add a current-target legality check.
* **Misdirection 2004-10-04**: *"You can choose to make a spell on the stack target this spell (if
  such a target choice would be legal had the spell been cast while this spell was on the stack)."*
  → Misdirection itself is a legal candidate. Delegating to `validate_targets_inner` gets this
  right for free (the Misdirection card is in `ZoneId::Stack`), and it is worth one probe (§5.2 T7)
  because the naive same-zone-plus-lowest-id scan also happened to allow it and a reviewer will
  want to know it survived.
* **Misdirection 2004-10-04**: *"You can't make a spell which is on the stack target itself."* →
  `self_id = victim_card` (§3.2 step 4).
* **Misdirection 2004-10-04**: *"Once the spell resolves, the new target is considered to be
  targeted by the deflected spell. This will trigger any effects which trigger on being targeted."*
  → **NOT implemented and NOT in scope**: no `PermanentTargeted` / `GameEvent::TargetsAnnounced` is
  emitted for the new target. Pre-existing; compounds with `OOS-DX25-2`/`OOS-ENG2-1`. Filed as
  `OOS-DX25c-4` so it is not mistaken for something this batch broke.
* **Bolt Bend 2024-11-08**: *"You must change the target if possible. However, you can't change the
  target to an illegal target. If there are no legal targets to choose from, the target isn't
  changed."* → the one-sentence statement of this whole batch, and the source of `OOS-DX25c-1`'s
  greedy-incompleteness caveat.

### 4.6 CR 608.2b — why `zone_at_cast` must be rebuilt

> **608.2b** If the spell or ability specifies targets, it checks whether the targets are still
> legal. A target that's no longer in the zone it was in when it was targeted is illegal. …

`resolution.rs::is_target_legal` compares the live zone against `SpellTarget.zone_at_cast`. Writing
the *original* target's zone onto a *new* target (HEAD, `effects/mod.rs:7655`) is therefore wrong
the moment the two differ. §3.2 step 8.

---

## §5 Test plan

New files:

* `crates/engine/tests/primitives/pb_dx25c_retarget_legality.rs`
  (+ `mod` line in `crates/engine/tests/primitives/main.rs` — **SR-9a: a dropped `mod` line
  silently deletes the whole file's coverage**)
* `crates/engine/tests/core/pb_dx25c_retarget_roster.rs`
  (+ `mod` line in `crates/engine/tests/core/main.rs`)
* `crates/simulator/tests/pb_dx25c_bot_retarget_is_legal.rs`

Modified: `pb_dx25b_announced_stack_target_space.rs` (T9 inverted), `tests/rules/copy_redirect.rs`
(6 fixtures), `tests/primitives/pb_ef11_spell_single_target.rs` (1 fixture + the shared
`make_stack_object` literal), `tests/core/pb_dx25b_announced_target_roster.rs` (R4 floor, §5.5),
plus the ~30 test-side `StackObject` literals.

**Hard constraint, stated because the batch will be judged on it (PB-DX25b's durable lesson):
every legality probe below reaches the code through a real `Command::CastSpell` and real
`PassPriority` resolution.** A hand-built `StackObject` may appear only where a real cast cannot
produce the configuration, and where it does the test's own doc must say so and say what it
therefore does *not* prove.

### 5.1 The inverted pin (AC 6303)

**T9-inverted** — `pb_dx25b_announced_stack_target_space.rs::t9_object_target_redirect_ignores_
the_original_requirement`, renamed to
`t9_object_target_redirect_obeys_the_original_requirement`.

The fixture is unchanged (§1 fact 3): p1's board is one land + one creature; p2 casts "Destroy
target creature" (`TargetRequirement::TargetCreature`) at the creature; p1 Misdirects it.
**After this batch the only creature on the board IS the current target, so CR 115.7a's fallback
applies.** New assertions, each with its CR cite:

1. **No `GameEvent::TargetsChanged` is emitted** — CR 115.7a, "the original target is unchanged".
   (HEAD emits one; this alone is red at HEAD.)
2. The bystander **land survives** on the battlefield — the inversion of the old assertion, quoted
   in the message alongside the old text so the diff is legible to a reviewer.
3. The **creature is destroyed** — proving the fallback left the original target intact rather than
   producing a fizzle.
4. The doc comment is rewritten: the wrong-way-round banner and the "successor batch must invert
   this" instruction are **removed**, replaced by a note recording that PB-DX25c inverted it and
   that `OOS-DX25b-3` is closed. Leaving a stale "must invert" instruction on a test that has been
   inverted is the `memory/conventions.md` aspirationally-wrong-comment hazard in its purest form.

**T9b — the same board with a second creature.** Add one creature controlled by p2. Assert the
redirect **does** fire, that `TargetsChanged.new_targets[0]` is that creature (not the land), that
its `zone_at_cast` is `Some(Battlefield)`, and that the land survives. This is the half that proves
the fix is not simply "never change anything"; without it, a `plan_target_change` that always
returned `None` would pass T9-inverted.

### 5.2 New probes — `pb_dx25c_retarget_legality.rs`

All use real casts. Each names the CR rule and the branch it discriminates.

* **T1 — hexproof (CR 702.11b) blocks the redirect.** Victim "destroy target creature" at p1's
  vanilla creature; the only other creature is p2's **hexproof** creature. Misdirection resolves →
  no `TargetsChanged`, the vanilla creature dies. Discriminates the *characteristic* half of
  `validate_object_satisfies_requirement`, which is a different code path from the type check T9
  exercises, and is one of the four checks the registry row names.
* **T2 — protection from blue (CR 702.16b) blocks the redirect.** Same shape, with the alternative
  creature having protection from the victim spell's colour. Discriminates the `source_chars`
  argument of §3.2 step 4 — the *only* probe that does, so if `source_chars` is passed as `None`
  this is the test that catches it.
* **T3 — CR 115.7a on the PLAYER branch, requirement half.** 3 players. p1 casts a purpose-built
  "target opponent loses 3 life" (`TargetRequirement::TargetOpponent`) at p2, then Misdirects
  **their own spell**. At HEAD the controller preference picks p1 — not an opponent of p1, illegal.
  After: p1 is rejected, p3 is chosen. Assert p3's life fell and p1's did not.
* **T4 — CR 104.3a / CR 115.7a on the PLAYER branch, `has_conceded` half.** 4 players, turn order
  [p1,p2,p3,p4]. p2 concedes (`Command::Concede` — verify it sets `has_conceded` and not `has_lost`
  *in the test*, so the probe documents its own premise). p3 casts a `TargetPlayer` spell at **p1**;
  p1 Misdirects it. The chooser-first candidate is p1 itself (= current target, skipped), so the
  scan reaches p2 first. HEAD picks the conceded p2; after, p2 is rejected and p3 is chosen.
* **T5 — CR 115.3 distinctness at retarget.** A victim with two mandatory slots that must be
  distinct (`TargetPermanentDistinctFrom`) — construct it if the corpus has none; assert the
  redirect never produces a set violating distinctness. If the shape proves unbuildable through a
  real cast, **say so in the test doc and drop the probe**, do not fake it with a hand-built stack
  object.
* **T6 — CR 115.4 / CR 115.7a cross-kind redirect.** Victim "deal 3 damage to any target"
  (`TargetAny`) at a player; the redirect may legally land on a creature. Asserts the unified
  candidate universe (§3.3) and pins the ordering rule (players before objects) so a future
  reordering is a visible decision.
* **T7 — Misdirection is itself a legal candidate** (2004-10-04 ruling). Victim `TargetSpell`-style
  or `TargetAny`? — use a victim whose requirement admits a spell on the stack, cast Misdirection,
  and assert the redirect can land on the Misdirection card. If no corpus-shaped victim admits it,
  build one; the point is that delegation did not silently narrow the candidate set to
  non-stack objects.
* **T8 — CR 601.2c self-targeting is still refused.** The victim spell cannot be retargeted onto
  its own card (`self_id`). Discriminates §3.2 step 4's `victim_card` argument.
* **T9c — the fail-closed guard.** The only probe permitted a hand-built `StackObject`: a spell
  entry with non-empty `targets` and empty `target_requirements`. Assert **no change and no event**.
  Its doc must state plainly that this configuration is unreachable through any real cast after
  §3.1's population work, that it exists to pin §3.4's decision, and that it therefore proves
  nothing about the production path.
* **T10 — `HashInto` field coverage.** A direct unit test that two `StackObject`s differing **only**
  in `target_requirements` hash differently. Required because `canonical_fixture()` cannot populate
  `stack_objects` (`hash_schema.rs:713-726`), so the field's own bytes are otherwise inside **no**
  gate — the v73 row's situation verbatim (`hash.rs:743-756`).
* **T11 — CR 608.2b `zone_at_cast` is rebuilt.** Assert the new `SpellTarget.zone_at_cast` is the
  **new** target's zone, and construct one case where old and new zones differ (a `TargetAny`
  victim redirected from a player, `zone_at_cast: None`, to a creature, `Some(Battlefield)`).

### 5.3 The bot-path probe (AC 6304) — `crates/simulator/tests/pb_dx25c_bot_retarget_is_legal.rs`

**S1 — the redirect is legal when the whole chain is driven the way a bot drives it.** Build a real
2-player fixture through the simulator's own machinery. Reach the Misdirection cast through
`crates/simulator/src/legal_actions` + `targeting::plan_targets` (**not** a hand-built
`Command::CastSpell`), submit the resulting command through the real engine, resolve, and assert:

1. `plan_targets` announced the victim **spell** (non-vacuity anchor — if it announced nothing the
   rest of the test is meaningless, and PB-DX25's T6 lesson is that a probe must not compare a
   fixture to itself);
2. the post-resolution target of the victim satisfies its own requirement, checked by calling
   `mtg_engine::rules::queries::legal_targets_per_slot` for that requirement and asserting
   membership — i.e. the assertion is made against the **offer layer's** answer, not against a
   literal, so it cannot drift from what the engine considers legal;
3. `mtg_simulator::invariants::check_all` reports **zero** violations on the final state.

**S2 — a fuzz-shaped run reaching the arm produces no illegal retarget.** Play N fuzz-shaped games
(`build_fuzz_state`, `RandomBot`, the `pb_dx32_fuzz_output.rs` idiom) and assert zero violations
of a new, narrow check. **Only do this if a measured run actually reaches `Effect::ChangeTargets`** —
measure first (the corpus has 4 such defs out of 1,133 `Complete`, so the arm may never fire at a
practical budget). If it does not, **say so with the measurement and ship S1 alone**; a fuzz probe
that never reaches its subject is a green test that means nothing, which is `OOS-SIM3-3`'s whole
lesson.

### 5.4 The roster / gate file — `crates/engine/tests/core/pb_dx25c_retarget_roster.rs`

Modelled on `pb_dx25b_announced_target_roster.rs` (read it first). **Reuse its `strip_comments`
(line **and** block — PB-DX32 M8), `balanced_body`, `extract_match_arm_body` and its
`sanitized_debug` corpus walker** rather than writing new ones; the sanitized-Debug walker is the
PB-DX25b review's E3 answer to hand-written walkers and its blind spot is already documented.

* **R1 — `must_change: true` corpus roster, with the single-target claim measured.** Enumerate
  `all_cards()`; pin the exact NAME set of defs carrying `Effect::ChangeTargets { must_change: true }`,
  and **in the same test** pin, for each, whether its `TargetRequirement` is one of the two
  single-target variants. This is what makes §3.5's "all-or-nothing is unreachable" a measurement
  rather than a claim. Non-vacuity floor `all_cards().len() >= 1_700` **in the same test** (PB-DX24
  R2 lesson). Recon says `{Bolt Bend, Misdirection, Untimely Malfunction}` — **re-measure; do not
  hard-code from this plan.**
* **R2 — the 115.7b/115.7c population is EMPTY, with a liveness control.** The DSL has no
  representation for "change a target" / "change any targets", so this is a pin on the *absence* of
  a need. An empty pin needs a walker-liveness control, not just a corpus floor (PB-DX25b R3): in
  the same test, assert the identical walker returns a **non-empty** set for a control needle known
  to be common. Also pin `deflecting_swat` as the sole `must_change: false` user, with the message
  restating `OOS-DX25b-4` ("membership here does not mean it works").
* **R3 — the population gate.** Scan `crates/engine/src` (comment-stripped) for every
  `StackObject {` literal and every `StackObject::trigger_default(` call whose surrounding statement
  later assigns `.targets`; assert each is paired with a `target_requirements` write. State the
  residual honestly: this is a *textual* pairing check over a fixed file set, it cannot prove the
  recorded list is the *right* one, and the thing that actually protects correctness is §3.4's
  fail-closed guard plus T9c. **Do not let this gate be described as proving population
  correctness.**
* **R4 — the arm contains no second decision.** Over `effects/mod.rs`'s `Effect::ChangeTargets` arm
  body (comment-stripped): (a) ≥1 occurrence of `retarget::plan_target_change`; (b) **zero**
  occurrences of `state.objects` / `.objects.iter()` / `state.players` / `has_lost` /
  `candidates.sort()`; (c) a re-measured size floor. Residual, stated in the gate's own doc: it sees
  only the arm it names.
* **R5 — one emitter.** `GameEvent::TargetsChanged` is constructed at exactly **one** place in
  `crates/engine/src` (comment-stripped, directory walk in R5-of-DX25b's style). This is the
  narrowest available machine check on §2.1's census, and its limit must be written down: a second
  retarget that mutates `StackObject.targets` **without** emitting the event is invisible to it, and
  nothing in the tree closes that. PB-DX25b's reviewer defeated its R5 three ways and the accepted
  answer was honest disclosure of the residual, not a bigger regex — the same standard applies here.
* **R6 — the two candidate universes agree.** A behavioural (not textual) gate: on a purpose-built
  multi-zone state, assert
  `retarget::retarget_candidates(state, chooser)` equals, as a set, the union of
  `queries::legal_targets_per_slot`'s candidate universe. Proves §3.3's "same universe" claim by
  execution rather than by comment, and reddens if either side's zone list drifts.

### 5.5 Existing tests expected to move (each handled deliberately, none silently)

| Test | Expected | Required handling |
|---|---|---|
| `pb_dx25b…::t9_object_target_redirect_ignores_the_original_requirement` | **RED**, by design | invert + rename (§5.1); record the pre-inversion failure text verbatim |
| `copy_redirect.rs::test_change_targets_must_change_redirects_to_new_player` (`:280`), `…_no_alternative_leaves_unchanged` (`:328`), `…_accepts_single_target_spell` (`:412`), `…_object_redirect` (`:451`), `…_redirects_single_target_spell_by_stack_entry_id` (`:544`) | **RED** (fail-closed: their fixtures record no requirements) | give each fixture the requirement its pretend-spell would really have carried (`TargetPlayer` for the player ones, `TargetCreature` for `:451`); record each pre-repair failure text — this set **is** the batch's headline non-vacuity evidence. `:371` (`must_change: false`) should stay **GREEN**; if it moves, that is a finding. |
| `copy_redirect.rs::test_change_targets_object_redirect` (`:451`) additionally | its `source` id is not in `state.objects`, so `source_chars` is `None` | keep it that way and note in the doc that this fixture therefore does **not** exercise the CR 702.16b protection path (T2 does) |
| `pb_ef11_spell_single_target.rs::test_misdirection_retargets_single_target_spell` (`:372`) | **RED** | same repair |
| `pb_dx25b…::t1`, `::t2`, `::t10` | expected **GREEN** (real casts, so requirements are populated; the redirect targets stay legal) | if any moves, stop and diagnose — it means the population or the preference ordering is wrong, not that the test needs adjusting |
| `pb_dx25b_announced_target_roster.rs::r4_…` `body.len() >= 200` floor (`:408-414`) | possibly **RED** (the arm shrinks) | re-measure the stripped body; re-aim the floor as a **deliberate, revert-proven** edit (PB-DX25b's own G2 precedent), never by deleting the floor |
| `core::hash_schema` sentinels, `pb_ef11_spell_single_target.rs:331` (`HASH_SCHEMA_VERSION == 73`) and every other 73-sentinel | **RED** | re-pin to 74 by **symbol**, after computing the value from the failing gate's own output (§7) |
| `core::decision_gate` / `pb_dx32_fuzz_output` decision-coverage gates | expected **GREEN** (ids unchanged) | execute, do not assume |
| Golden scripts | none expected | verify with `SCRIPT_FILTER`, **not** by starting the replay-viewer HTTP server (gotchas-infra: SIGKILL 137) |

---

## §6 Revert matrix

Every row must be **executed**: apply the mutation, confirm the rebuild actually happened
(`Compiling mtg-engine` present in captured output — a stale binary faking a pass is the PB-DX32 R7
class), capture the failure text verbatim into `memory/primitives/pb-DX25c-execution-notes.md`, then
restore and confirm `git diff` clean before the next row.

| # | Production edit | Mutation | Test that must redden |
|---|---|---|---|
| V1 | `retarget::plan_target_change` step 6's `validate_targets_inner` trial | replace the trial with `true` (accept any candidate ≠ current) | `t9_…obeys…` (land dies again), T1, T2, T3 |
| V2 | step 6's `?` (CR 115.7a all-or-nothing / per-index fallback) | replace with "skip this index and continue" | `t9_…obeys…` (a `TargetsChanged` fires with an unchanged target set) |
| V3 | step 7's final-set re-validation | delete it | T5 if buildable; otherwise **record that V3 has no discriminating test** and say so in the notes rather than inventing one — an undiscriminated line is a finding, not a formality |
| V4 | step 4's `source_chars` | pass `None` | **T2 only** (this is the sole probe covering it) |
| V5 | step 4's `victim_card` as `self_id` | pass `None` | T8 |
| V6 | step 6's `caster` argument | pass `chooser` instead of `so.controller` (the CR 109.5 confusion) | T3 |
| V7 | `retarget_candidates`' `has_conceded` conjunct | drop it | T4 |
| V8 | `retarget_candidates`' object arm | drop it (players only) | T6, R6 |
| V9 | `retarget_candidates`' chooser-first preference | drop it | `pb_dx25b…::t1` (life total lands on a different player) — **executing this is how the plan's claim that the preference is load-bearing for existing tests gets verified rather than assumed** |
| V10 | §3.4's fail-closed guard | let an empty `reqs` fall through to `validate_targets_inner` | T9c |
| V11 | §3.2 step 8's `zone_at_cast` rebuild | write the original target's zone | T11 |
| V12 | `StackObject.target_requirements` in `HashInto` | delete the `hash_into` feed | T10 **and** `core::hash_schema`'s `NOT_HASHED`/coverage gate — record BOTH failures, since the gate firing is the SR-19 forcing function this plan claims exists |
| V13 | `copy.rs`'s `target_requirements` propagation | set `vec![]` | a new probe: copy a targeted spell, then redirect the copy — must go from "redirects legally" to "does not redirect". If no such probe is buildable (a copy is not announceable, `OOS-DX25b-2`), **say so and mark V13 undiscriminated**; do not manufacture a hand-built fixture and call it coverage |
| V14 | `casting.rs`'s hoisted `announced_requirements` | record `requirements` unconditionally (ignoring `mode_targets_active`) | `pb_dx25b…::t10_untimely_malfunction_mode1_target_index` |
| V15 | R4 gate | insert a `state.objects.iter()` scan into the `ChangeTargets` arm | R4 |
| V16 | R4/R5 comment stripping | wrap a required needle in `/* */` | R4 / R5 (the PB-DX32 M8 class — must still redden) |
| V17 | R2's liveness control | make the walker return an empty set unconditionally | R2's control assertion |
| V18 | R1's pinned name set | pin it one member short | R1 |
| V19 | R6 | narrow `retarget_candidates`' zone list to `Battlefield` | R6 |

**Mandatory A/B, executed and recorded**: `git stash` the whole batch and run
`cargo test -p mtg-engine --test primitives pb_dx25c`. Expect a **compile failure** (the module and
the field do not exist at HEAD) — the strongest available form of "none of these probes passes at
HEAD", and the same evidence shape PB-DX25b recorded. Additionally, before touching T9, run it
unchanged at HEAD and record that it is **green** — that is the wrong-way-round pin doing its job,
and the close-out should quote it.

---

## §7 Gates and measurements — every one EXECUTED, none predicted

**Predicted**: PROTOCOL **35 unmoved** (mechanism: `StackObject` is in
`protocol_schema.rs`'s `CLOSURE_MUST_NOT_CONTAIN`, §1 fact 10). HASH **73 → 74**, forced.
Coverage **1,133/1,803 = 62.8% unmoved**.

**HASH bump procedure** (`hash.rs:807-813`'s own three steps, plus what the last five batches
learned):

1. Bump `HASH_SCHEMA_VERSION` to **74** and append a `- 74:` History doc line **modelled on the
   v73 row (`hash.rs:743-756`), which is the closest precedent** — same shape of claim, same
   fixture limitation. It must state: the new field, its `#[serde(default)]`, its reachability from
   `GameState` via `stack_objects: Vector<StackObject>`; that **`decl_fingerprint` MOVES** (new
   field in the serde closure); that **`stream_fingerprint` moves by the v40 mechanism only**
   (`HASH_SCHEMA_VERSION` is the stream's first byte) because `canonical_fixture()` cannot populate
   `stack_objects` (`hash_schema.rs:713-726`'s five named exclusions), so this is the
   v69/v72/v73 version-sentinel-byte-only case and the field's own bytes are covered **by T10 and
   by nothing else**; and that `PROTOCOL_VERSION` is **UNMOVED**, with the
   `CLOSURE_MUST_NOT_CONTAIN` mechanism named.
2. Append the `HASH_SCHEMA_HISTORY` row with fingerprints **read off the failing gate's own
   output** — computed, never hand-derived.
3. Re-pin every `HASH_SCHEMA_VERSION == 73` sentinel **by symbol** (`git grep -n 'HASH_SCHEMA_VERSION'`),
   then confirm by a full `--workspace --no-fail-fast` run that the residual list is empty.
   PB-DX6's 13-sentinel re-pin is the procedure of record.
4. Note in the row that `loop_detection.rs:144-146` folds the whole `StackObject` via
   `so.hash_into`, so the new field enters `compute_mandatory_state_hash` automatically; this adds
   **no** CR 104.4b false-negative risk because the field is fixed at construction and never
   mutated (the same argument v70 made for `affected_set`, and the opposite of PB-DP9's excluded
   fields).

**To execute and record** (all of them; `cargo test --workspace --no-fail-fast` **to a file**,
never `| tail` — a tail pipe hid a compile failure and faked a green run on 2026-08-02):

```
cargo test -p mtg-engine --test core hash_schema        # expect 74 after the bump
cargo test -p mtg-engine --test core protocol_schema    # expect 35, UNMOVED
cargo test -p play-server                                # expect unmoved (80/0 at PB-DX25b close)
cargo test -p mtg-simulator
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check ; tools/check-defs-fmt.sh              # SR-35: fmt checks 0 of 1,803 defs
cargo build --workspace
python3 tools/authoring-report.py                        # body must be byte-identical
cargo bench -p mtg-engine --bench engine_perf -- "full_turn_4p|priority_cycle_4p"
```

**Scope diffs to run and report**:

* `git diff main..HEAD --numstat -- crates/view-model/ tools/` → expected **EMPTY**.
* `git diff main..HEAD --numstat -- crates/simulator/` → expected **exactly one file**,
  `src/decision_coverage.rs`, doc-text only (§2.5), plus the new test file.
* `git diff main..HEAD -- crates/card-defs/` → expected **comment-only**, verified **per line**
  (PB-DX25b §14's standard), on `misdirection.rs` and `bolt_bend.rs` only. Coverage proven unmoved
  by **regeneration to a byte-identical body**, not by an empty diff.
* `git diff main..HEAD --numstat -- crates/card-types/` → two files
  (`src/state/stack.rs`, `src/cards/card_definition.rs`), one new field + doc text.

**Performance note worth measuring rather than assuming**: `plan_target_change` costs up to
(candidates × targets) `validate_targets_inner` calls, each of which may run
`calculate_characteristics`. On a four-player board that is the same order as
`queries::legal_targets_per_slot`'s documented ~200 layer computations — but it now runs during
**resolution**, not during a UI query. It fires once per `Effect::ChangeTargets` resolution, which
no benchmark exercises, so `full_turn_4p` / `priority_cycle_4p` are expected within noise; if they
move, the call landed somewhere it should not have.

**Predicted test delta**: **+18 to +22** (11-13 in `pb_dx25c_retarget_legality.rs`, 6 in the roster
file, 1-2 simulator, T9b). Every repaired existing test is a modification, not an addition.
PB-DX25b's own execution notes record that its plan's headline range overshot the truth; measure
the pre-edit baseline **on this branch before any edit** and report the delta against that, not
against 4,469.

---

## §8 Risks, residuals, and what this batch does NOT deliver

**R1 — the batch's own subject matter, turned on itself: the census could still be short.** §2.1
used three independent methods and they agree on one decision site. R5 machine-checks the narrowest
version of that (one `TargetsChanged` emitter). Nothing catches a future site that mutates
`StackObject.targets` without emitting the event. **Stated here and in R5's own doc**, not glossed.

**R2 — the ability half remains unreachable (`OOS-DX25b-1`), and this batch must not claim
otherwise.** A `ChangeTargets` victim is always a `Spell` / `MutatingCreatureSpell`, because the
only route to one is an announced card id resolved through `stack_index_for_announced_target`, and
an ability's stack entry is never in `state.objects`. Consequence: §3.2 step 4's `victim_card ==
None` path is **dead code today**, and `self_id`/`source_chars` would both be wrong-ish for an
ability if it ever became reachable (the right values are the ability's *source permanent*, which
needs the `stack_registry::source_of` sibling `OOS-DX25-4` already asks for). Filed as
`OOS-DX25c-3`. **The runner should grep its own diff for "or ability" before committing**, exactly
as PB-DX25b was told to.

**R3 — greedy search is incomplete for n > 1 targets** (§3.5). Population measured as **zero** by
R1; failure direction is CR 115.7a's own fallback. `OOS-DX25c-1`.

**R4 — a redirect still fires no "becomes the target of" trigger** (Misdirection 2004-10-04 ruling;
CR 702.21a Ward). Pre-existing, compounds with `OOS-DX25-2` and `OOS-ENG2-1`. **This batch makes it
more visible, not worse**: before it, the redirect landed on an arbitrary object and fired nothing;
after it, the redirect lands on a *legal* object and fires nothing. `OOS-DX25c-4`.

**R5 — `deflecting_swat` still gains nothing** (`OOS-DX25b-4`) and `Effect::CopySpellOnStack` still
never chooses new targets (`OOS-DX25c-2`). Neither may be counted in the close-out.

**R6 — the `zone_at_cast: None` widening (§1 fact 2) is closed as a side effect, and that should be
said rather than discovered.** At HEAD a target recorded with `zone_at_cast: None` made the
candidate set *every object in the game, in every zone* — hands and libraries included. The new
candidate universe is the three public zones (`queries.rs:230-237`), so the widening is gone. It was
never a hidden-information *leak* (nothing was rendered), but it was a wrong-game-state path, and
it is a strictly better story than the one the registry row tells.

**R7 — PB-DX25b's R4 floor will need re-aiming** (§1 fact 14, §5.5). Handle it as a deliberate,
revert-proven gate edit and record it in the execution notes, exactly as PB-DX25b handled PB-DX25's
G2. **Weakening a shipped gate silently is the one outcome this batch cannot ship.**

**R8 — ~42 struct literals is a large mechanical diff and mechanical diffs hide things.** Every
`target_requirements: vec![]` in a *production* file must carry a one-line reason. The
test-file ones need none. Run `cargo build --workspace` early and often; it is what catches the
`crates/view-model` and `tools/tui` sites if any turn out to construct a literal after all (measured
today: they do not — both use `trigger_default`).

**R9 — coverage and the fuzz seed pool.** 0 `completeness` flips predicted, so
`pb_dx32_fuzz_output.rs::test_dx32_fuzz_deck_pool_size_is_pinned` (`CORPUS_COMPLETE = 1133`) must
stay green. If it moves, a card-def edit went beyond comment-only and every recorded fuzz seed has
been re-rolled (`OOS-CARDS2-3`). Verify by execution, not by inspecting the diff.

**R10 — candidate seeds to file (grep the registry FIRST; dispatch hygiene 5).**
`OOS-DX25c-1` (greedy incompleteness for n > 1, with its measured-zero population),
`OOS-DX25c-2` (`copy_spell_on_stack`'s `_choose_new_targets` still unwired, CR 707.10a/115.7d),
`OOS-DX25c-3` (an ability victim's `self_id`/`source_chars` would be wrong; blocked behind
`OOS-DX25b-1`; wants `stack_registry::source_of` from `OOS-DX25-4`),
`OOS-DX25c-4` (a redirect fires no becomes-the-target trigger).
**`OOS-DX25b-3` is CLOSED**, and its row must carry corrections to its own claims: the line range
`:7619-7654` is stale, and the row omits the `Target::Player` half entirely (§1 fact 7) and the
`zone_at_cast: None` widening (R6). `OOS-DX25b-1`, `-2`, `-4`, `-5` all stay **open**.

---

## §9 Verification checklist

- [ ] Pre-edit baseline measured **on this branch before any edit**, `cargo test --workspace
      --no-fail-fast` to a **file**; residual list recorded
- [ ] T9 run **unchanged at HEAD** and recorded green (the wrong-way-round pin doing its job)
- [ ] §1's line-number and premise corrections re-checked against source, not copied from this plan
- [ ] Field added, hashed, propagated; `git grep -n 'StackObject {'` count recorded before and after
- [ ] Every production `target_requirements: vec![]` carries a reason
- [ ] `rules/retarget.rs` created; `pub mod retarget;` added; `effects/mod.rs`'s arm reduced to §3.6
- [ ] The out-of-scope sites (§2.4) byte-unchanged — prove with `git diff`
- [ ] Every revert in §6 executed, rebuild confirmed, failure text captured, revert restored clean;
      **any undiscriminated row reported as a finding, not omitted**
- [ ] `git stash` A/B recorded (compile failure at HEAD)
- [ ] T9 inverted **and renamed**, its "successor must invert" instruction removed
- [ ] The six red fixtures repaired with real requirement lists; each pre-repair failure recorded
- [ ] PB-DX25b's R4 floor re-measured and, if moved, re-aimed with its own revert proof
- [ ] `mod` lines added to `tests/primitives/main.rs` and `tests/core/main.rs` (SR-9a)
- [ ] HASH **74** gate-COMPUTED (history row + fingerprints from the gate's own output + every
      sentinel re-pinned by symbol); PROTOCOL **35** gate-EXECUTED and unmoved
- [ ] `cargo test -p play-server`, `-p mtg-simulator`, `core decision_gate`,
      `pb_dx32_fuzz_output` all executed and reported
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35)
- [ ] `cargo build --workspace`
- [ ] Coverage proven unmoved by **regeneration** to a byte-identical body
- [ ] Card-def diff verified **per line** to be comment-only
- [ ] Benches spot-checked; `CORPUS_COMPLETE` gate green
- [ ] `OOS-DX25b-3` CLOSED in `docs/audits/decision-point-audit.md` **with corrections to its own
      claims** (stale line range; the missing `Target::Player` half; the `zone_at_cast: None`
      widening)
- [ ] `OOS-DX25c-1..4` filed — **grep the registry for each ID first** (dispatch hygiene 5)
- [ ] `memory/primitives/pb-DX25c-execution-notes.md` written: revert matrix results, every
      measurement, the A/B, every roster population including the zeros, and every place this plan
      turned out to be wrong
- [ ] v3 queue row 7c struck; CLAUDE.md delta appended as a **NEW short bullet** (never grow a
      line); `memory/workstream-state.md` handoff
