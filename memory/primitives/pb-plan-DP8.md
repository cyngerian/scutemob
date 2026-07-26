# Primitive Batch Plan: PB-DP8 — Triggered-ability targets become a player choice (CR 603.3d)

**Generated**: 2026-07-26
**Task**: `scutemob-156` · branch `feat/pb-dp8-triggered-ability-target-choice-surface-the-84-def-ag`
**Primitive**: a second blocking pending decision — `GameState.pending_trigger_targets` +
`GameEvent::TriggerTargetChoiceRequired` → `Command::ChooseTriggerTargets` — that **suspends
`abilities::flush_pending_triggers` mid-batch** and resumes it when the trigger's controller
answers.
**Finding**: DP-6 (`docs/audits/decision-point-audit.md` §5 line 457, §7 line 570 = **OOS-M11-4**)
**CR Rules**: **603.3d**, 603.3, 603.3a, **603.3b**, 603.3c, **601.2c**, 101.4, 700.2b,
**800.4d**, 800.4g, 800.4j, 104.3a, 117.3, 613.1f
**Class**: AGENCY, not a rules fix. The existing first-match fallback at
`crates/engine/src/rules/abilities.rs:7308-7798` is **CR 603.3d-compliant** and is **preserved**
verbatim as the exported bot/harness default.
**Cards affected**: **74 effectively-`Complete` defs** (derived below, §1.4 — *not* the audit's 84;
the derivation and its falsifier are stated). **0 card-def source edits, 0 completeness flips.**
**Wire**: **PROTOCOL 28 → 29** and **HASH 65 → 66** — both expected, both with a stated falsifier (§6).
**Baseline**: PROTOCOL **28** (`rules/protocol.rs:268`), HASH **65** (`state/hash.rs:607`),
tests **3,837**.
**Dependencies**: PB-DP7 (`BlockingDecision` + `blocking_decision` + the admission gate + the
`GameState::blocking_decision()` public accessor), PB-DP1 (priority-to-actor), M11-local S1
(`LocalGame`).
**Deferred items carried in**: **OOS-DP3-4** (modal triggered abilities) — explicit **OUT**, argued
in §9. **OOS-DP7-5** (`PendingDecision.actions` out of runway) — acknowledged, worked around, not
solved; argued in §7.2.

---

## 0. Executive summary — the six decisions that carry this batch

1. **The engine never learns what a human is.** The block is unconditional; a pure exported helper
   `abilities::default_trigger_targets` supplies the pre-PB-DP8 pick, and the *caller*
   (`StubProvider` → bot → `LocalGame`, the replay-harness pump, the TUI key) submits it as a real
   `Command`. This is DP-7's pattern and it is the answer to the brief's "crux" question: seat kind
   is a `crates/simulator` concept (`LocalGame.human_seats`) and must stay one, or Architecture
   Invariant 1 and SR-9b both break. §5.
2. **The pending state is an `Option`, and the plurality lives inside it.** CR 603.3b's batch is
   answered as a *sequence* of round-trips, but exactly one question is outstanding at any moment,
   because the flush is sequential and the admission gate freezes everything else. The entry owns
   the trigger being asked about **and the un-flushed tail of its own batch**. §2.
3. **Resumption replays, it does not re-derive.** `flush_pending_triggers` drains
   `state.pending_triggers` up front, so a naive pause loses the tail. The entry carries
   `remaining: Vector<PendingTrigger>` in already-APNAP-sorted order; the resume re-enters a new
   private `flush_sorted(state, sorted, presupplied_head_targets)` with
   `[entry.trigger] ++ entry.remaining` and the answer bound to the head. Triggers already on the
   stack stay there and are never revisited. §3.
4. **The consult-site set is derived, not enumerated — and DP-7's answer does not transfer.**
   `flush_pending_triggers` has **6** production call sites on this branch. **Five of them are one
   site**: every one of the **29** `check_and_flush_triggers(&mut state, &mut events);` calls in
   `process_command` is immediately followed by `all_events.extend(events);` and the end of its match
   arm, and `process_command`'s tail (`engine.rs:722-726`) only records history — so **nothing
   executes after a suspended flush on any command path** and **no guard is needed there at all**.
   The four that *do* need a guard are the four that grant priority afterwards. §4.
5. **A choice with exactly one legal answer is not a choice.** When every required slot has exactly
   one candidate and no slot is optional, the announcement is determined (CR 601.2c) and the engine
   places the trigger directly with no round trip. This is CR-justified independently, and it is also
   the single thing that keeps this batch's test/script/fuzzer fallout tractable. §5.3.
6. **OOS-DP3-4 (modal triggered abilities) is OUT**, for a reason that is about CR 700.2c and the
   DSL's shape, not about scheduling. §9.

---

## 1. The finding, re-verified on this branch

### 1.1 The site, and why it is compliant

`crates/engine/src/rules/abilities.rs:7308-7798`, the
`PendingTriggerKind::Normal | PendingTriggerKind::CardDefETB` arm of `flush_pending_triggers`'s
`trigger_targets_opt` chain. For each `TargetRequirement` of the ability it computes exactly one
`candidate: Option<SpellTarget>` and pushes it; if any required slot yields `None` it sets
`all_satisfied = false` and the trigger is `continue`d off the stack (`:7788-7806`).

Re-verified, all four compliance claims from audit §7 hold on this branch:

| claim | site | verdict |
|---|---|---|
| layer-resolved characteristics | `layers::expect_characteristics(state, obj.id)` at **`:7571`** | **confirmed** (audit cites `:7434` — stale by 137 lines) |
| protection/hexproof/shroud honoured | `super::validate_target_protection(...)` at **`:7573-7582`** | **confirmed** (audit cites `:7438-7450`) |
| never self-targets a `TargetOpponent` | **`:7410-7425`** (and the `UpToN{inner: TargetOpponent}` twin at `:7537-7552`) — no `.or_else(self)` | **confirmed** (audit cites `:7272-7291`) |
| no legal candidate ⇒ removed from the stack | `:7788-7791` + `:7802-7806` | **confirmed** |

**So nothing here is a rules bug and nothing here may be "fixed".** The batch adds a question in
front of the fallback and leaves the fallback as the answer of last resort.

> CR 603.3d — *"The remainder of the process for putting a triggered ability on the stack is
> identical to the process for casting a spell listed in rules 601.2c–d. If a choice is required
> when the triggered ability goes on the stack but no legal choices can be made for it, or if a rule
> or a continuous effect otherwise makes the ability illegal, the ability is simply removed from the
> stack."*
>
> CR 601.2c — *"The player announces their choice of an appropriate object or player for each target
> the spell requires. … The same target can't be chosen multiple times for any one instance of the
> word 'target' on the spell."*
>
> CR 603.3b — *"If multiple abilities have triggered since the last time a player received priority,
> the abilities are placed on the stack in a two-part process. First, each player, in APNAP order,
> puts each triggered ability they control with a trigger condition that isn't another ability
> triggering on the stack in any order they choose. (See rule 101.4.) Second, each player, in APNAP
> order, puts all remaining triggered abilities they control on the stack in any order they choose.
> …"*
>
> CR 603.3a — *"A triggered ability is controlled by the player who controlled its source at the
> time it triggered …"* — this is why the entry names `trigger.controller` and nobody else.
>
> CR 800.4d — *"If an object that would be owned by a player who has left the game would be created
> in any zone, it isn't created. **If a triggered ability that would be controlled by a player who
> has left the game would be put onto the stack, it isn't put on the stack.**"* — the CR-supplied
> answer to the brief's liveness question (§8).
>
> CR 800.4j — *"If a player leaves the game during their turn, that turn continues to its completion
> without an active player. …"*
>
> CR 104.3a — *"A player can concede the game at any time. A player who concedes leaves the game
> immediately."*

### 1.2 What is IN scope at the site, and what is not

The `trigger_targets_opt` chain has eight branches. Only the last real one is a choice:

| branch | line | in scope? | why |
|---|---|---|---|
| `trigger.targeting_stack_id` (ward) | `:7231-7237` | **no** | CR 702.21a determines the target; nothing is chosen |
| `trigger.triggering_player` | `:7238-7242` | **no** | CR 102.2/603.2 determines it |
| `trigger.defending_player_id` (annihilator/dethrone/training/afflict) | `:7243-7282` | **no** | CR 702.86a/508.5 determine it |
| `trigger.exalted_attacker_id` | `:7283-7292` | **no** | CR 702.83a determines it |
| `PendingTriggerKind::Provoke` | `:7293-7307` | **no** | CR 702.39a — the provoked creature was chosen at declare-attackers |
| **`Normal` / `CardDefETB` with non-empty `ability_targets`** | **`:7308-7798`** | **YES** | CR 603.3d/601.2c — the controller announces |
| `Normal`/`CardDefETB` with empty `ability_targets` | `:7360-7362` | no | nothing to choose |
| `else` (all other `PendingTriggerKind`s) | `:7799-7801` | no | keyword machinery |
| `PendingTriggerKind::Modular` (separate, later) | `:7868-7931` | **no — seeded** | CR 702.43a *is* a real choice ("target artifact creature") auto-picked lowest-`ObjectId` at `:7877-7888`, but it lives in the `kind` match with its own `continue`, not in `trigger_targets_opt`. Out of scope; **OOS-DP8-3** |

### 1.3 One pre-existing defect found while reading the site

`TargetRequirement::TargetPermanentDistinctFrom(_)` is handled at `:7598` as
`=> true` — *"distinctness enforced at declaration validation (casting.rs), not here."* That is true
for spells and false for triggers: nothing validates trigger declarations. So a trigger declaring two
`TargetPermanentDistinctFrom` slots gets the **same** object in both, because both slots run the same
`.find` on the same ascending `OrdMap`. Corpus exposure: zero (`rg 'TargetPermanentDistinctFrom'
crates/card-defs/src/defs/` returns nothing inside a `Triggered`). Seeded as **OOS-DP8-4**; PB-DP8
adds only the narrow cross-slot duplicate rejection described in §5.5 item 8.

### 1.4 The card roster — derived, with the method and the discrepancy stated

**The audit's "84" is not reproduced. I derive 74.** Method, mechanical, on this branch:

The site is reached only for a card-def `AbilityDefinition::Triggered` whose `targets` is non-empty
(both the `Normal` runtime path at `:7329-7333` and the `CardDefETB` registry path at `:7340-7354`
read that same field — and `testing::replay_harness::build_face_ability_vectors` forwards
`targets: targets.clone()` at every lowering site, e.g. `:2271`, `:2309`, `:2336`, so the lowered
`WhenDies`/`WhenAttacks`/`WhenBlocks`/`WhenDealsCombatDamageToPlayer` families reach it too).
`intervening_if` immediately precedes `targets` in the variant declaration
(`crates/card-types/src/cards/card_definition.rs:338-365`) and in 551 of the 577 `intervening_if:`
occurrences in the corpus the next line is `targets:` (the other 26 are multi-line
`intervening_if: Some(Condition::X {` bodies).

| step | pattern | result |
|---|---|---|
| a | `AbilityDefinition::Triggered {` in `crates/card-defs/src/defs/` | 570 abilities / 511 files |
| b | multiline `intervening_if: None,\s+targets: vec!\[(\s*\n\|[A-Za-z])` | **89 abilities / 87 files** |
| c | multiline `intervening_if: Some\((?s:.)*?targets: vec!\[(\s*\n\|[A-Za-z])` | 7 files, of which **2 are false positives** verified by reading (`thaumatic_compass.rs` — its Triggered is `targets: vec![]`, the lazy match ran on into a back-face `Activated`; `siege_gang_lieutenant.rs` — same shape). Real: `raiders_wake.rs`, `vivisection_evangelist.rs`, `nullpriest_of_oblivion.rs`, `thieving_skydiver.rs` (`known_wrong`), `tatyova_steward_of_tides.rs` (`partial`) |
| d | of (b), how many carry a non-`Complete` marker: multiline `…targets: vec!\[…(?s:.)*?completeness: Completeness::(partial\|known_wrong\|inert)` | **16 files** (`sun_titan`, `skullsnatcher`, `shriekmaw`, `sheoldred_whispering_one`, `retreat_to_coralhelm`, `patron_of_the_vein`, `orcish_bowmasters`, `niv_mizzet_parun`, `mortuary_mire`, `kogla_the_titan_ape`, `junji_the_midnight_sky`, `ink_eyes_servant_of_oni`, `glissa_sunslayer`, `gilded_drake`, `den_protector`, `boggart_shenanigans`) |
| **=** | **(87 − 16) + (5 − 2)** | **71 + 3 = 74 effectively-`Complete` defs** |

**This is a grep estimate and SR-36 forbids shipping a roster derived from grep.** The runner's
first task is to re-derive it by enumeration, which is the authoritative method:

> Write a `#[test]` in the new test file that walks `mtg_card_defs::all_cards()`, and for every
> `CardDefinition` (including `back_face` and `adventure_face` via `effective_abilities(true)` /
> `effective_abilities(false)`) counts those with an `AbilityDefinition::Triggered { targets, .. }`
> where `!targets.is_empty()` **and** `completeness == Completeness::Complete`. Print the count and
> the names. Pin the number with `assert!(n >= 60)` (a `>=` assertion, so the authoring campaign does
> not redden it), and **write the number this test prints into the PB's commit message and into the
> audit's §5 DP-6 row.** If it is 84, my grep undercounted and I want that recorded; if it is 74, the
> audit's row is corrected. Either way the number in the doc becomes a fact instead of a claim
> (PB-DP6's headline lesson).

Spot-check of what the roster actually contains, so the reviewer can sanity-check it: the ten karoo
lands (`azorius_chancery`, `boros_garrison`, `dimir_aqueduct`, `golgari_rot_farm`, `gruul_turf`,
`izzet_boilerworks`, `orzhov_basilica`, `rakdos_carnarium`, `selesnya_sanctuary`,
`simic_growth_chamber` — *"When this land enters, return a land you control to its owner's hand"*,
today the engine picks which of your lands bounces), `ravenous_chupacabra`, `acidic_slime`,
`reclamation_sage`, `eternal_witness`, `shriekmaw`, `warstorm_surge`, `dragon_tempest`,
`aura_shards`, `ojutai_soul_of_winter`, `sword_of_fire_and_ice` + 4 sibling Swords.

**Card-def yield: 0 edits, 0 completeness flips.** This is an engine-agency batch; the 74 defs are
already `Complete` and already legal — they simply stop having their target chosen for them.

---

## 2. The pending state — shape, and why `Option` is right

### 2.1 The types

**File**: `crates/card-types/src/state/stubs.rs` (home chosen to match `PendingTrigger` and
`PendingCleanupDiscard`).

```rust
/// CR 603.3d / CR 601.2c (PB-DP8 / DP-6): one target slot of a triggered ability
/// whose controller must announce a choice as it is put on the stack.
///
/// Reachable from `GameEvent::TriggerTargetChoiceRequired`, so this type IS in the
/// SR-8 wire closure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerTargetOption {
    /// True iff the requirement is `TargetRequirement::UpToN` — CR 601.2c's
    /// "up to": the slot may legally be answered with zero targets.
    pub optional: bool,
    /// Every legal choice for this slot, derived with the SAME predicates the
    /// CR 603.3d auto-fallback uses (see `abilities::trigger_target_candidates`).
    /// Deterministic order: `state.objects` is an `OrdMap` (ascending `ObjectId`)
    /// and `state.turn.turn_order` is a `Vec` in seat order.
    pub candidates: Vec<SpellTarget>,
    /// The pre-PB-DP8 auto-pick for this slot, byte-identical to what the
    /// first-match fallback produced. `None` only for an `optional` slot the old
    /// code skipped. When `Some(t)`, `t` is always present in `candidates`
    /// (debug-asserted). This is NOT always `candidates[0]`: for player-targeting
    /// requirements the old code preferred the first live OPPONENT and only then
    /// fell back to the controller, while `candidates` legally contains every live
    /// player (CR 601.2c) — which is exactly the agency this batch restores.
    pub default: Option<SpellTarget>,
}

/// CR 603.3d / CR 603.3b (PB-DP8 / DP-6): the suspended trigger flush.
///
/// Reachable only from `GameState` — never from `Command`/`GameEvent`/`ReplayLog`
/// — so it contributes nothing to `PROTOCOL_SCHEMA_FINGERPRINT` (the `PendingDraw`
/// and `PendingCleanupDiscard` precedent).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingTriggerTargets {
    /// Monotonic, unique for the whole game. Taken by incrementing
    /// `GameState.timestamp_counter` at the moment the flush suspends (the same
    /// counter `next_object_id` uses, `state/mod.rs:940-943`). The answering
    /// command must quote it: this is the MOMENT guard, not a payload guard
    /// (PB-DP7 lesson 2).
    pub choice_id: u64,
    /// CR 603.3a: the controller of `trigger`, and the ONLY player who may answer.
    pub player: PlayerId,
    /// The trigger being put on the stack right now. It is NOT in
    /// `GameState.pending_triggers` while this entry exists — the entry owns it.
    pub trigger: PendingTrigger,
    /// CR 603.3b: the rest of THIS batch, already APNAP-sorted, none of which has
    /// been put on the stack. The resume continues through these in order.
    pub remaining: Vector<PendingTrigger>,
    /// One entry per `TargetRequirement` of the trigger's ability, in declaration
    /// order.
    pub slots: Vector<TriggerTargetOption>,
}
```

**File**: `crates/engine/src/state/mod.rs`, beside `pending_cleanup_discard` (`:144-152`):

```rust
/// CR 603.3d (PB-DP8 / DP-6): the suspended trigger flush, if any. At most one
/// can be outstanding — CR 603.3b's batch is answered as a SEQUENCE of
/// round-trips, one at a time, and this entry carries the un-flushed tail.
/// See `rules::engine::blocking_decision`.
#[serde(default)]
pub(crate) pending_trigger_targets: Option<PendingTriggerTargets>,
```

plus, beside `pending_cleanup_discard()` (`:459-475`):

```rust
/// Read-only access to the `pending_trigger_targets` field (CR 603.3d, PB-DP8).
/// No `_mut` accessor (SR-3). Consumers OUTSIDE this crate must read
/// `blocking_decision()` instead — the liveness-filtered predicate — per
/// PB-DP7's fix-cycle Finding 4.
pub fn pending_trigger_targets(&self) -> Option<&PendingTriggerTargets> { … }
```

and `crates/engine/src/state/builder.rs` initialises it to `None` beside `pending_cleanup_discard`
(`:322`).

### 2.2 Why `Option`, not `Vector` — correcting the inherited spec

`pb-plan-DP7.md` §1.5 predicts *"DP-8's pending state is a `Vector`"*. **That prediction is wrong,
and the reason it is wrong is the interesting part.** §1.5 correctly identifies that the *decision*
is plural (CR 603.3b puts a whole batch on the stack, controlled by different players) and correctly
concludes it is answered as a *sequence*. But "answered as a sequence" is precisely the property that
makes the outstanding set a **singleton**:

- CR 603.3d attaches the choice to *"when the triggered ability goes on the stack"* — a per-ability
  moment, not a per-batch one. The batch is placed one ability at a time.
- CR 603.3b orders those moments totally (APNAP, then within-controller). There is never a legal
  moment at which two controllers are simultaneously owed a CR 603.3d announcement.
- Asking in parallel would be *wrong*, not merely awkward: a later chooser is entitled to see the
  earlier triggers already on the stack (they are public information the CR gives them).
- The admission gate freezes the game between questions, so a `Vector` could never hold two live
  entries anyway; it would only ever hold one, and would additionally have to invent an ordering
  policy the CR already supplies.

So the plural half lives in `remaining: Vector<PendingTrigger>` *inside* the singleton entry, where
it is what it actually is: the un-flushed tail of one CR 603.3b batch. The hash consequence is the
same either way (one new `GameState` field, one HASH bump), so nothing is bought by the `Vector`.

**Falsifier**: if any CR rule permitted two controllers to announce trigger targets simultaneously,
`Option` would be wrong. CR 603.3b's two-part APNAP process forbids it.

### 2.3 SR-19 / OOS-DP7-11 — write the `HashInto` impls with BARE names

`crates/engine/tests/core/hash_schema.rs`'s `every_hashed_struct_field_is_hashed_or_allowlisted`
gate looks impl bodies up by the **bare** struct name. `impl HashInto for crate::state::stubs::Foo`
silently falls out of the gate with no diagnostic (**OOS-DP7-11**, demonstrated by PB-DP7).

Therefore, in `crates/engine/src/state/hash.rs`, beside `impl HashInto for PendingCleanupDiscard`
(`:3003-3008`), write **exactly**:

```rust
impl HashInto for TriggerTargetOption { … }        // optional, candidates, default — all three
impl HashInto for PendingTriggerTargets { … }      // choice_id, player, trigger, remaining, slots — all five
```

with the bare type names and no path qualification. `PendingTrigger` (`:3071`), `SpellTarget`
(`:3941`) and `Target` (`:3927`) already have `HashInto`, so the bodies are one line per field. The
`NOT_HASHED` allowlist (`hash_schema.rs:1252`) is empty and **must stay empty**.

**Runner obligation (a gate cited in a comment is a claim, PB-DP7 lesson 4):** after writing them,
temporarily delete one `hash_into` line from each new impl, run
`cargo test --test <hash gate target> every_hashed_struct_field_is_hashed_or_allowlisted`, confirm it
**fails by name**, then restore. Record the result in the commit message. Do not assert gate coverage
in a source comment without having done this.

Fold into `public_state_hash` beside `:7786`:
```rust
self.pending_trigger_targets.hash_into(&mut hasher);   // blanket Option impl at :999-1009
```
and mirror into `rules/loop_detection.rs`'s mandatory-state fingerprint beside its
`pending_cleanup_discard` line (`:151-156`) — a suspended flush is a distinct position.

---

## 3. Partial-flush resumption (the thing DP-7 got for free)

### 3.1 Why the current function cannot pause

`flush_pending_triggers` (`abilities.rs:7084`) **drains first**:

```rust
let pending: Vec<PendingTrigger> = state.pending_triggers.iter().cloned().collect();
state.pending_triggers = imbl::Vector::new();          // :7098-7099
```

then sorts (`:7101-7109`) and iterates. A `return` mid-loop therefore destroys every unprocessed
trigger. That is the whole of the resumption problem, and it is why `cleanup_actions`'s
idempotence-at-max-hand-size trick does not transfer.

### 3.2 The shape

Split the function into a public entry point and a private worker:

```rust
// crates/engine/src/rules/abilities.rs

/// CR 603.3. Unchanged public signature and unchanged semantics EXCEPT that the
/// batch may now suspend: if this returns with `state.pending_trigger_targets`
/// `Some`, the batch is INCOMPLETE and the caller must not grant priority or
/// advance (see the four guarded call sites in the plan's §4.1).
pub fn flush_pending_triggers(state: &mut GameState) -> Vec<GameEvent> {
    // A suspended flush must never be re-entered; the caller's guard should have
    // prevented it, so this is an engine bug, not a fizzle (SR-4).
    debug_assert!(state.pending_trigger_targets.is_none(),
        "flush_pending_triggers re-entered while a CR 603.3d target choice is outstanding");
    if state.pending_trigger_targets.is_some() { return Vec::new(); }
    if state.pending_triggers.is_empty() { return Vec::new(); }
    // ... existing CR 603.2d trigger_doublers retain (:7090-7096) ...
    let mut sorted: Vec<PendingTrigger> = state.pending_triggers.iter().cloned().collect();
    state.pending_triggers = imbl::Vector::new();
    let apnap = apnap_order(state);
    sorted.sort_by_key(|t| apnap.iter().position(|&p| p == t.controller).unwrap_or(usize::MAX));
    flush_sorted(state, sorted, None)
}

/// CR 603.3d resume: continue a suspended batch. `head_targets` are the answered
/// targets for `sorted[0]`; every later trigger derives its own normally.
fn flush_sorted(
    state: &mut GameState,
    sorted: Vec<PendingTrigger>,
    head_targets: Option<Vec<SpellTarget>>,
) -> Vec<GameEvent> { /* the existing :7110-7564 loop body, with the two edits below */ }

/// Called by `handle_choose_trigger_targets` once the answer validates.
pub(crate) fn resume_trigger_flush(
    state: &mut GameState,
    chosen: Vec<SpellTarget>,
) -> Vec<GameEvent> {
    let entry = state.pending_trigger_targets.take()
        .expect("caller validated the entry exists");
    let mut sorted = vec![entry.trigger];
    sorted.extend(entry.remaining.iter().cloned());
    flush_sorted(state, sorted, Some(chosen))
}
```

Two edits inside the loop body:

- **Edit A (the pause).** In the `Normal | CardDefETB` arm (`:7308-7798`), replace the per-requirement
  `candidate` computation with a call to the extracted
  `trigger_target_candidates(state, &trigger, req) -> TriggerTargetOption` (§5.2), collect the slots,
  and branch:
  - any **required** slot with `candidates.is_empty()` ⇒ CR 603.3d, `continue` (unchanged behaviour);
  - `head_targets` is `Some(..)` **and this is the first iteration** ⇒ use it, and set
    `head_targets = None` so it cannot leak to a later trigger;
  - `trigger_target_choice_is_forced(&slots)` (§5.3) ⇒ use each slot's sole candidate directly, no
    round trip;
  - `trigger.controller` has left the game (`has_lost || has_conceded`) ⇒ use
    `default_trigger_targets(&slots)`; **do not ask a player who cannot answer** (PB-DP7 Finding 1
    applied preemptively — see §8);
  - otherwise ⇒ **suspend**: build the entry from `trigger` + the *not-yet-iterated* remainder of
    `sorted` + `slots`, push `GameEvent::TriggerTargetChoiceRequired`, and `return events`.
- **Edit B (the resume must not re-sort or re-drain).** `flush_sorted` never touches
  `state.pending_triggers` and never sorts; both belong to the public entry point. `remaining` is
  already sorted, so the batch's CR 603.3b order survives a pause byte-for-byte.

**Implementation note for the runner:** the loop consumes `sorted` with `for trigger in sorted`. To
capture the remainder, convert to an index loop (`let mut i = 0; while i < sorted.len() { let trigger
= sorted[i].clone(); i += 1; … }`) and build `remaining` from `sorted[i..]`. Do **not** restructure
the ~750-line `kind` match at `:7814-8472`; it is untouched by this batch.

### 3.3 Why replay, not re-derive, is correct — and what could falsify it

On resume, `flush_sorted` re-runs, for the head trigger, the once-per-turn gate (`:7119-7171`), the
doubling computation (`:7176-7180`) and `has_ability_targets` (`:7200-7229`) — but **not** the target
derivation (the answer is supplied). Those three are pure functions of `state`, and `state` cannot
have changed: `process_command`'s admission gate rejects every command except the answer and
`Concede` while `blocking_decision()` is `Some`. So the re-run is provably identical, and carrying
`additional_count`/`once_per_turn` in the entry would be redundant state.

For triggers *after* the head, targets ARE re-derived at their own turn — which is what CR 603.3d
requires ("as it goes on the stack"), and it is why the earlier triggers now being on the stack is
correct rather than a hazard.

**Falsifier**: if any code path could mutate `GameState` between the pause and the resume, the
re-derivation could disagree with what was offered and the SR-38 contract would break. The only such
path would be a command that slips the admission gate. T11 pins that it does not.

**Triggers already placed stay placed.** The stack objects pushed before the pause are real and
final; the resume never revisits them. T7 asserts "each trigger appears on the stack exactly once".

---

## 4. Consult sites — derived, complete, and *not* DP-7's set

### 4.1 The derivation

DP-7's completeness argument was: *nothing advances a step or a turn except the call sites of
`advance_step`/`advance_turn`*. That argument does not apply here. The correct one is:

> **A suspended flush returns control to the statement after the `flush_pending_triggers` call.
> Nothing else in the engine is running. So the guard set is exactly "every statement that executes
> after a `flush_pending_triggers` call, within the same `process_command` invocation".**

`rg 'flush_pending_triggers\(' crates/*/src` returns **6** production call sites (plus the definition
at `abilities.rs:7084`). For each, what runs after:

| # | site | enclosing fn | what runs after the flush | verdict |
|---|---|---|---|---|
| 1 | `rules/engine.rs:60` | `check_and_flush_triggers` | `events.extend(trigger_events); }` — returns unit into a `process_command` match arm | **no guard needed**, see §4.2 |
| 2 | `rules/engine.rs:2169` | `enter_step`, Cleanup branch | `had_events` → `cleanup_sba_rounds += 1` → loop detection → **`grant_initial_priority` + `priority_holder = Some(active)`** (`:2188-2194`) | **GUARD** |
| 3 | `rules/engine.rs:2210` | `enter_step`, has-priority branch | loop detection → **`grant_initial_priority` / `next_priority_player`** (`:2228-2250`) | **GUARD** |
| 4 | `rules/combat.rs:760` | `handle_declare_attackers` | `players_passed = OrdSet::new()`, **`priority_holder = Some(player)`**, `PriorityGiven` (`:765-767`) | **GUARD** |
| 5 | `rules/combat.rs:1538` | `handle_declare_blockers` | `players_passed = OrdSet::new()`, **`priority_holder = Some(active)`**, `PriorityGiven` (`:1544-1548`) | **GUARD** |
| 6 | `rules/resolution.rs:7799` | resolution tail | `players_passed = OrdSet::new()`, **`priority_holder = Some(active)`**, `PriorityGiven` (`:7802-7805`) | **GUARD** |

The guard is identical at all four:

```rust
// CR 603.3 / CR 603.3d (PB-DP8): the batch suspended on a target choice. CR 603.3b
// gives priority only AFTER every triggered ability of this batch is on the stack,
// so stop here without granting it. `handle_choose_trigger_targets` resumes.
if state.pending_trigger_targets.is_some() {
    return Ok(events);        // or `return events;` where the fn is infallible
}
```

Use the **raw field** inside `crates/engine` (it is the engine's own state and the liveness filter is
irrelevant: a dead controller is never asked, §8). Everything **outside** `crates/engine` reads
`GameState::blocking_decision()` (PB-DP7 fix-cycle Finding 4).

### 4.2 Why site #1 — the ~20-command path DP-7's §1.5 warned about — needs no guard

This is the load-bearing part of the derivation and it is a mechanical fact, not a hope.
`check_and_flush_triggers` is called from **29** sites in `process_command`
(`engine.rs:215, 257, 288, 377, 391, 400, 412, 428, 486, 500, 509, 518, 529, 537, 546, 555, 564, 578,
591, 607, 621, 635, 644, 657, 663, 669, 687, 707, 718`). `rg -A4` over every one of them shows the
**identical** two lines follow:

```rust
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
```

(the single exception, `:518` `PlotCard`, adds only a comment). `process_command`'s tail after the
`match` (`engine.rs:721-726`) is:

```rust
    }
    // Record events in history
    for event in &all_events { state.history.push_back(event.clone()); }
    Ok((state, all_events))
```

— no priority grant, no SBA, no step advance. So on every one of those 29 paths the command simply
returns with the entry recorded, and from that instant the admission gate owns the game. **29 sites,
0 guards.** DP-7's §1.5 predicted this would be the expensive part; it is the free part, and the
reason is that PB-DP1 already moved priority assignment *into* the individual handlers, ahead of the
flush.

### 4.3 ALREADY-SAFE, argued, no edit

| site | why it cannot step over a suspended flush |
|---|---|
| `process_command`'s admission gate (`engine.rs:169-178`) | one entry point (Architecture Invariant 3 / SR-3); §4.4 extends its allow-list |
| `priority::pass_priority` (`priority.rs:26-33`) | unreachable — the admission gate rejects `PassPriority` before it |
| `handle_all_passed` (`engine.rs:2088-2121`) | reachable only from `handle_pass_priority` and `handle_concede`; the former is gated, the latter is handled in §8 |
| `enter_step`'s existing PB-DP7 progress gate (`engine.rs:2156`) | it runs *before* the flushes at `:2169`/`:2210` in the same loop iteration, so it catches a *pre-existing* entry but not one created during this iteration — which is exactly why guards #2 and #3 exist. **Do not delete or move it**; it is DP-7's gate and it also covers the new variant for free (it reads `blocking_decision`) |
| `sba::check_and_apply_sbas` | never flushes and never grants priority |
| `rules/turn_structure.rs::advance_step` / `advance_turn` | pure `&GameState -> TurnState` |
| the 14 direct test call sites | not production; §11.2 lists them as fallout |

### 4.4 Admission gate — who may act while suspended

`engine.rs:169-178`, extend the allow-list (this is the **one** site a new `BlockingDecision` variant
must always edit; PB-DP7's fix-cycle Finding 3 says so and the source comment at `:63-99` says so):

```rust
let allowed = matches!(&command, Command::Concede { .. })
    || matches!(&command, Command::DiscardToHandSize { player, .. } if *player == decision.player())
    || matches!(&command, Command::ChooseTriggerTargets { player, .. } if *player == decision.player());
```

| command | while suspended | why |
|---|---|---|
| `ChooseTriggerTargets` from `decision.player()` | **accepted** | it is the answer |
| `Concede` (any player) | **accepted** | CR 104.3a — available at all times; §8 handles the consequences |
| everything else, incl. `PassPriority` and `TapForMana` | **rejected**, `BlockedByPendingDecision` | CR 603.3 gives priority only *after* the batch is on the stack; nobody has priority mid-flush, and CR 605.3a requires priority to activate a mana ability |

`GameStateError::BlockedByPendingDecision { player, decision: String }` already exists
(`state/error.rs:94-102`) and is outside the wire closure — no new error variant needed.

---

## 5. The choice itself: candidates, defaults, and the forced-choice narrowing

### 5.1 Human vs bot at the engine boundary — the crux, resolved

**The engine does not, and must not, know which seats are human.** Restated as three facts:

1. Architecture Invariant 1 makes the engine a pure library with no notion of a client. A
   `human_seats` concept in `crates/engine` would be a UI concept in the rules layer.
2. SR-9b's cross-regime determinism and the replay log both require that *every* choice becomes a
   `Command`. If the engine auto-picked for bot seats and blocked for human seats, a bot game and a
   human game would produce different command traces for the same decisions and the replay log would
   no longer be a complete record.
3. The DP-7 pattern already encodes the answer: *the engine never auto-picks on a decision path; a
   pure exported helper supplies the deterministic default; the caller submits it as a real command.*

So PB-DP8 blocks unconditionally, and `crates/simulator` decides who answers: `LocalGame.human_seats`
routes to `AwaitingHuman`, everything else submits `default_trigger_targets`. The *cost* of that
choice is stated honestly in §5.3 and §10.

### 5.2 Extract the predicate; do not fork it

**New, in `crates/engine/src/rules/abilities.rs`:**

```rust
/// CR 603.3d / CR 601.2c: every legal choice for one target slot of a triggered
/// ability, plus the pre-PB-DP8 auto-pick.
///
/// This is the SAME code the first-match fallback used, refactored from
/// `.find(<pred>)` into `.filter(<pred>).collect()` + `default`. Validation of a
/// submitted answer is membership in `candidates` — the predicate is never
/// re-implemented and never re-run against a possibly-different state.
pub(crate) fn trigger_target_candidates(
    state: &GameState,
    trigger: &PendingTrigger,
    req: &TargetRequirement,
) -> TriggerTargetOption
```

Mechanics, per requirement family (line numbers are the current fallback's):

| family | `candidates` | `default` |
|---|---|---|
| `TargetPlayer`, `TargetCreatureOrPlayer`, `TargetAny`, `TargetPlayerOrPlaneswalker` (`:7378-7405`) | **union**: every live player, **plus** the battlefield objects the object-arm's own match already accepts for that requirement (`TargetCreatureOrPlayer => is_creature` `:7603`, `TargetAny => is_creature \|\| is_planeswalker` `:7688-7690`, `TargetPlayerOrPlaneswalker => is_planeswalker` `:7691-7693`) | today's pick: first live opponent in `turn.turn_order`, else the controller |
| `TargetOpponent` (`:7410-7425`) | every live opponent. **No self-fallback, ever** (PB-EF6, CR 102.3/601.2c) | `candidates.first()` |
| `TargetCardInYourGraveyard` (`:7427-7466`), `TargetCardInGraveyard` (`:7467-7496`) | `.filter` instead of `.find`, predicate verbatim | `candidates.first()` |
| `UpToN { inner }` (`:7503-7557`) | delegate to `inner`, `optional: true` | today's: player-inner ⇒ first opponent (or controller for the non-`TargetOpponent` player kinds); permanent-inner ⇒ **`None`** (the old code contributed 0 targets) |
| everything else — the battlefield scan (`:7559-7783`) | `.filter` instead of `.find`, the ~220-line predicate verbatim including `layers::expect_characteristics`, `validate_target_protection`, `matches_filter`, `TargetController`, `exclude_self`, combat-role, tapped/untapped | `candidates.first()` |

**The player-arm/object-arm union is a deliberate, CR-correct widening, and it makes currently-dead
code live.** Today the player arm returns first for `TargetCreatureOrPlayer` / `TargetAny` /
`TargetPlayerOrPlaneswalker`, so the object arm's matching branches at `:7603`, `:7688-7693` are
**unreachable**. CR 601.2c makes both kinds legal, so a human choosing must be offered both. Because
`default` still comes from the player arm, **no bot behaviour changes** (T13). Pinned by T14.

`debug_assert!(opt.default.map_or(true, |d| opt.candidates.contains(&d)))` in every construction
path — a `default` outside `candidates` would make the SR-38 contract false.

**Also new, pure and exported:**

```rust
/// CR 603.3d (PB-DP8): the deterministic default answer — byte-identical to the
/// pre-PB-DP8 first-match auto-pick, because each slot's `default` IS the value the
/// old `candidate` expression produced.
///
/// THE ENGINE NEVER CALLS THIS ON A DECISION PATH. It exists so `StubProvider`,
/// the replay harness and the TUI cannot drift from one another (SR-38).
pub fn default_trigger_targets(slots: &[TriggerTargetOption]) -> Vec<Vec<Target>>
```
(one inner `Vec` per slot; `vec![]` where `default` is `None`).

### 5.3 The forced-choice narrowing

```rust
/// CR 601.2c: an announcement with exactly one legal answer is determined. When
/// every slot is required and has exactly one candidate, there is nothing for the
/// controller to decide, so the engine places the trigger directly rather than
/// spending a wire round trip on a question with one answer.
fn trigger_target_choice_is_forced(slots: &[TriggerTargetOption]) -> bool {
    slots.iter().all(|s| !s.optional && s.candidates.len() == 1)
}
```

**Argument.** CR 601.2c requires the controller to *announce* a choice; it does not give them a
choice where the rules leave one legal option. The only way "one candidate" could still be a real
decision is if the slot were optional — a player may prefer to target nothing — and `optional` is
exactly `TargetRequirement::UpToN`, which the predicate excludes. There is a second way a
one-candidate slot could still be a decision: a *"you may"* trigger, where the player declines the
whole ability. That is **DP-12**, which has no DSL representation at all (19 defs marked
`known_wrong`), so it cannot be reached from here.

**Why it matters practically, stated so a reviewer can weigh it:** without this narrowing, every
Ravenous Chupacabra ETB in a one-opponent-creature board, every karoo land with one other land, and
every unit test with a single legal target becomes a wire round trip. That is thousands of extra
commands across the golden-script corpus and the existing engine test suite, all of them answerable
in exactly one way. The narrowing removes essentially all of the churn while removing **none** of the
agency.

**Falsifier**: the narrowing is wrong if any CR rule lets a controller decline a required target with
one legal candidate. CR 601.2c ("The player announces their choice of an appropriate object or player
**for each target the spell requires**") does not. T5 pins it; T14's optional case pins the
complement.

### 5.4 The wire shapes

**`Command::ChooseTriggerTargets`** — `crates/engine/src/rules/command.rs`, appended after
`DiscardToHandSize`:

```rust
/// CR 603.3d / CR 601.2c (PB-DP8 / DP-6): the trigger's controller announces its
/// targets as it is put on the stack.
///
/// Sent in response to `GameEvent::TriggerTargetChoiceRequired`. `choice_id` must
/// equal the outstanding entry's — it is the MOMENT guard, so an answer to a
/// superseded question in the same CR 603.3b batch is rejected rather than applied
/// to the wrong trigger.
///
/// `targets` has exactly one inner `Vec` per slot of the offered `slots`, in the
/// same order: exactly one `Target` for a required slot, zero or one for an
/// `optional` slot (CR 601.2c "up to"). Only the `Target` identity is carried; the
/// engine re-derives `zone_at_cast` from its own candidate set, so a client cannot
/// tamper with it.
ChooseTriggerTargets {
    player: PlayerId,
    choice_id: u64,
    targets: Vec<Vec<Target>>,
},
```

`Target` is already in the closure (`command.rs:10`, used by `ActivateAbility.targets` `:104` and
`CastSpellData.targets` `:674`); `u64` and `PlayerId` likewise. **Closure type count unchanged by the
`Command`.**

**`GameEvent::TriggerTargetChoiceRequired`** — `crates/engine/src/rules/events.rs`, appended at the
**end** of the enum after `CleanupDiscardChoiceRequired` (`:1373-1380`). **Discriminant 130**
(current max is 129, `hash.rs:5341-5351`).

```rust
/// CR 603.3d (PB-DP8 / DP-6): the controller of a triggered ability must announce
/// its targets before it goes on the stack. The engine BLOCKS — the CR 603.3b batch
/// is suspended, no priority is granted, and `process_command` rejects every command
/// except `Command::ChooseTriggerTargets` from `player` and `Command::Concede` —
/// until the answer arrives.
///
/// `slots` is one `TriggerTargetOption` per `TargetRequirement`, in declaration
/// order, each carrying the full legal candidate set (so a client can render the
/// picker with no second query) and the engine's deterministic default (so a bot,
/// the replay harness or a minimal TUI can answer in one step, SR-38).
///
/// Emitted only when the choice is real: a slot with no legal candidate removes the
/// trigger instead (CR 603.3d), and a fully forced choice is placed directly.
///
/// Hidden information (Architecture Invariant 7): unlike
/// `CleanupDiscardChoiceRequired`, every id here names a public-zone object (the
/// battlefield or a graveyard) or a player, so `reveals_hidden_info()` is `false`
/// and no M10 private-to filter is owed. See the plan's §12.
///
/// Discriminant: 130.
TriggerTargetChoiceRequired {
    player: crate::state::player::PlayerId,
    choice_id: u64,
    source_object_id: crate::state::game_object::ObjectId,
    ability_index: usize,
    slots: Vec<crate::state::stubs::TriggerTargetOption>,
},
```

`TriggerTargetOption` (and, through it, `SpellTarget`) **join** the wire closure — a genuine type-count
change, unlike PB-DP7's. Say so in the commit message.

### 5.5 Validation list for `handle_choose_trigger_targets`

**File**: `crates/engine/src/rules/abilities.rs` (beside the flush machinery), dispatched from
`engine.rs`'s match with `validate_player_exists` (the `ChooseDredge`/`DiscardToHandSize` precedent)
and `loop_detection::reset_loop_detection` (CR 104.4b — a target announcement is a meaningful player
choice, matching `ChooseDredge` `engine.rs:439`).

In order, **all before any mutation**:

1. `validate_player_exists(&state, player)`.
2. An entry exists — else `InvalidCommand("no trigger-target choice is pending")`.
3. `entry.player == player` — else `InvalidCommand(..)` naming both. **SR-29 trust boundary.** Note
   that the admission gate at `engine.rs:169-178` already rejects a foreign sender with
   `BlockedByPendingDecision`, so this check is only reachable by a direct handler call — which is
   exactly the hole PB-DP7's review Finding 12 found. **T9 must assert the specific error for each
   path, not merely `is_err()`.**
4. `choice_id == entry.choice_id` — else `InvalidCommand("stale trigger-target choice: expected N")`.
   **This is the moment guard**, and it is what makes an answer to question *k* inapplicable to
   question *k+1* of the same batch.
5. `targets.len() == entry.slots.len()` — else `InvalidCommand(..)`.
6. Per slot *i*: `targets[i].len() == 1` for a required slot; `<= 1` for an `optional` slot
   (CR 601.2c "up to"). Else `InvalidCommand(..)` naming the slot index.
7. Per submitted `Target t` in slot *i*: find the unique `SpellTarget` in
   `entry.slots[i].candidates` whose `.target == t`; absent ⇒
   `InvalidCommand("target is not a legal choice for slot i (CR 603.3d)")`. **This IS the CR 603.3d
   legality check** — the candidate set was built by `validate_target_protection` +
   `layers::expect_characteristics` + `matches_filter` and nothing can have changed since (§3.3), so
   re-running the predicate here would add no safety and could only introduce a disagreement with
   what was offered (SR-38). The engine takes `zone_at_cast` from the candidate, never from the wire.
8. Cross-slot distinctness, **narrow**: if two slots' requirements are both
   `TargetPermanentDistinctFrom(_)` and the submitted `Target`s are equal, reject
   (CR 601.2c). The general "same target can't be chosen multiple times for **any one instance** of
   the word target" case is per-slot and is covered by (6). See §1.3 / OOS-DP8-4.
9. Only then: `resume_trigger_flush(state, chosen)`.

**State untouched on rejection.** `process_command` takes `GameState` by value (`engine.rs:152-156`)
and every `?` discards the locally-mutated copy, so the caller's state is untouched by construction —
provided the handler validates before mutating, which (1)-(8) enforce. T3 and T8 assert
byte-identical `public_state_hash` across a rejection, and this is the property
`LocalGame::submit`'s contract (`local_game.rs:453-456`) and ESM criterion 5545 depend on.

### 5.6 `BlockingDecision` variant

`crates/engine/src/rules/engine.rs:100-126`:

```rust
pub enum BlockingDecision {
    CleanupDiscard { player: PlayerId, count: u32 },
    /// CR 603.3d (PB-DP8 / DP-6): `player` must announce the targets of the
    /// triggered ability from `source` before the CR 603.3b batch can continue.
    TriggerTargets { player: PlayerId, choice_id: u64, source: ObjectId },
}
```

Keeps `Copy`. `player()` gains an arm; `Display` gains an arm. `blocking_decision`
(`engine.rs:133-147`) gains a second lookup with the same liveness filter — but the filter is
belt-and-braces here, because a dead controller is never asked in the first place (§8).

**Correct PB-DP7's aspirational comment while you are in the file.** The doc block at
`engine.rs:63-99` says a new variant needs "no new consult site". Now that a second variant exists,
rewrite it to name what a second variant actually costs: the admission-gate allow-list (§4.4), the
`handle_concede` clear (§8), the two by-name hash lines (§2.3), **and** — new, found by this batch —
`crates/simulator/src/local_game.rs:335-338`, which today maps *any* `BlockingDecision` to
`DecisionKind::CleanupDiscard` without matching on the variant (§7.3).

---

## 6. Wire expectation, and what would falsify each half

### 6.1 `PROTOCOL_VERSION` 28 → 29 — expected

Two new wire-frame variants (`Command::ChooseTriggerTargets`, `GameEvent::TriggerTargetChoiceRequired`)
**and** two new types entering the closure (`TriggerTargetOption`, `SpellTarget`).

Procedure, verbatim from `protocol.rs:323-334`, in **one** commit:
1. `PROTOCOL_VERSION` 28 → **29** at `protocol.rs:268`, plus a `- 29:` History line above it.
2. **Append** `ProtocolEpoch { version: 29, fingerprint: <gate-computed> }` to `PROTOCOL_HISTORY`
   (array starts `protocol.rs:338`; **never edit an existing row**) and set
   `PROTOCOL_SCHEMA_FINGERPRINT` (`:285-286`) to the same value.
3. Re-pin `protocol_version_sentinel` (`crates/engine/tests/core/protocol_schema.rs:868`, currently
   `28`) and `FROZEN_HISTORY_PREFIX_DIGEST` (`protocol_schema.rs:148-149`).

**Never hand-invent a fingerprint** — both values are printed by the failing gate.

**Falsifier**: PROTOCOL stays 28 only if the answer reuses an existing `Command` variant and the
question reuses an existing `GameEvent`. There is a superficially tempting candidate —
`Command::ActivateAbility { targets: Vec<Target>, .. }` — and it is **wrong**: it carries a flat
`Vec<Target>` with no slot structure (so "up to N" is unrepresentable), no `choice_id` (so no moment
guard), and it would make the DP-24 accepted-and-discarded-field problem worse rather than better.
Reject it explicitly and record the rejection.

### 6.2 `HASH_SCHEMA_VERSION` 65 → 66 — expected

`GameState` gains `pending_trigger_targets: Option<PendingTriggerTargets>`; two new hashed structs;
`GameEvent` gains discriminant 130.

1. `HASH_SCHEMA_VERSION` 65 → **66** (`hash.rs:607`) + a `- 66:` History line.
2. **Append** `HashSchemaEpoch { version: 66, decl_fingerprint, stream_fingerprint }` after the v65
   row (`hash.rs:917-926`), both gate-computed, no existing row edited.
3. `GameEvent` hashing match: append a `130u8` arm after `CleanupDiscardChoiceRequired`
   (`hash.rs:5341-5351`). **That match has no `_` arm** — a miss is a compile error, which is the
   point.
4. `HashInto` impls per §2.3 (bare names).
5. `public_state_hash` + `loop_detection.rs` mirror per §2.3.
6. Re-pin the `HASH_SCHEMA_VERSION` sentinel (`crates/engine/tests/core/hash_schema.rs:1198`) and
   `FROZEN_HISTORY_PREFIX_DIGEST` (`hash_schema.rs:190-191`).

**Falsifier**: HASH stays 65 only if the suspended flush could live outside `GameState`. It cannot —
`process_command(state: GameState, …) -> Result<(GameState, …)>` is the only carrier between two
commands, and the un-flushed tail (`remaining`) has nowhere else to exist. Re-deriving it from
`pending_triggers` is not an option either: the drain has already happened, and a "leave them in
`pending_triggers`" variant would let the resume re-sort them against a batch boundary that has
already been fixed (CR 603.3b), and would make the entry invisible to `loop_detection`.

### 6.3 The ~53 scattered sentinel copies (OOS-DP7-8) — enumerated so the runner does not discover them one build at a time

Re-pin `28 → 29` and `65u8 → 66u8`:

| file | line(s) | which |
|---|---|---|
| `crates/engine/tests/core/protocol_schema.rs` | 868 | PROTOCOL (canonical) + `FROZEN_HISTORY_PREFIX_DIGEST` at 148-149 |
| `crates/engine/tests/core/hash_schema.rs` | 1198 | HASH (canonical) + `FROZEN_HISTORY_PREFIX_DIGEST` at 190-191 |
| `crates/engine/tests/primitives/pb_os6_dfc_flip_conditions.rs` | 876, 880 | both |
| `crates/engine/tests/primitives/pb_os10_singleton_cleanup.rs` | 94, 100 | both |
| `crates/engine/tests/primitives/pb_ef7_modal_activated.rs` | 239, 244 | both |
| `crates/engine/tests/primitives/pb_os7_defending_player_continuous_filter.rs` | 692, 699 | both |
| `crates/engine/tests/primitives/pb_os8_look_at_top_then_place.rs` | 1176, 1181 | both |
| `crates/engine/tests/primitives/pb_ef10_sacrifice_driven_amounts.rs` | 1597, 1602 | both |
| `crates/engine/tests/primitives/pb_os9_lieutenant_commander_control.rs` | 887, 891 | both |
| `crates/engine/tests/primitives/pb_os5_relative_attacker_count.rs` | 716, 721 | both |
| `crates/engine/tests/primitives/pb_ef12_any_color_choice.rs` | 373 | PROTOCOL |
| `crates/engine/tests/primitives/pbp_power_of_sacrificed_creature.rs` | 795 | HASH |
| `crates/engine/tests/primitives/primitive_pb_xa.rs` | 93 | HASH |
| `crates/engine/tests/primitives/primitive_pb_xs.rs` | 69 | HASH |
| `crates/engine/tests/primitives/primitive_pb_ewcd.rs` | 143 | HASH |
| `crates/engine/tests/primitives/pb_ef6_target_opponent.rs` | 280 | HASH |
| `crates/engine/tests/primitives/primitive_pb_cc_c_followup.rs` | 402 | HASH |
| `crates/engine/tests/primitives/primitive_pb_lki_power.rs` | 389 | HASH |
| `crates/engine/tests/primitives/primitive_pb_oos_lki_power_3.rs` | 66 | HASH |
| `crates/engine/tests/primitives/pb_ac9_wheel_and_misc.rs` | 127 | HASH |
| `crates/engine/tests/primitives/pbn_subtype_filtered_triggers.rs` | 568 | HASH |
| `crates/engine/tests/primitives/primitive_pb_eat.rs` | 140 | HASH |
| `crates/engine/tests/primitives/pb_ac1_untap_counter.rs` | 94 | HASH |
| `crates/engine/tests/primitives/pb_ef1_exclude_self_enforcement.rs` | 165 | HASH |
| `crates/engine/tests/primitives/primitive_pb_ewc.rs` | 400 | HASH |
| `crates/engine/tests/primitives/primitive_pb_xs_e.rs` | 165 | HASH |
| `crates/engine/tests/primitives/pb_ac4_per_mode_targeting.rs` | 701 | HASH |
| `crates/engine/tests/primitives/pbt_up_to_n_targets.rs` | 412, 866 | HASH ×2 |
| `crates/engine/tests/primitives/pb_ac6_phase_action_conditions.rs` | 182 | HASH |
| `crates/engine/tests/primitives/primitive_pb_xa2.rs` | 107 | HASH |
| `crates/engine/tests/primitives/primitive_pb_lki_cc.rs` | 443 | HASH |
| `crates/engine/tests/primitives/pbd_damaged_player_filter.rs` | 611 | HASH |
| `crates/engine/tests/primitives/pb_ef11_spell_single_target.rs` | 336 | HASH |
| `crates/engine/tests/primitives/pb_ac5_alt_costs.rs` | 408 | HASH |
| `crates/engine/tests/primitives/pb_ef2_create_token_recipient.rs` | 263 | HASH |
| `crates/engine/tests/primitives/primitive_pb_cc_a.rs` | 101 | HASH |
| `crates/engine/tests/primitives/pb_ac8_restrictions_and_wingame.rs` | 165 | HASH |
| `crates/engine/tests/primitives/primitive_pb_ts.rs` | 369 | HASH |
| `crates/engine/tests/primitives/pb_ef11_wheel_greatest_discarded.rs` | 91 | HASH |
| `crates/engine/tests/primitives/pb_ac7_type_change_ability_removal.rs` | 961 | HASH |
| `crates/engine/tests/primitives/pb_ac3_dynamic_pt_counts.rs` | 885 | HASH |
| `crates/engine/tests/rules/loyalty_target_validation.rs` | 355 | HASH |
| `crates/engine/tests/casting/optional_cost_and_counter_tax.rs` | 1139 | HASH |
| `crates/engine/tests/mechanics_e_l/effect_sacrifice_permanents_filter.rs` | 136 | HASH |

**53 assertions across 44 files.** Do not add a 54th in the new PB-DP8 test file — OOS-DP7-8 is a
standing complaint about exactly this growth, and adding to it while citing it would be poor form.

### 6.4 Bump both in one commit

Both gates fail on the first `cargo test --all`. Take all three fingerprints (one protocol, two hash)
plus both frozen-prefix digests from the failure texts and say in the commit message that every one
is gate-computed.

---

## 7. Consumers — the driving loops, not just the consult sites

PB-DP7's closing review found that *"a gate that stops the engine also stops every loop built on top
of it"*, and that the plan had listed the TUI's display and input surfaces while missing its loop.
This section enumerates loops first.

### 7.1 The five driving loops

| loop | file | status after PB-DP7 | PB-DP8 work |
|---|---|---|---|
| TUI auto-pass | `tools/tui/src/play/mod.rs:117-146` via `PlayApp::should_stop_auto_pass` (`app.rs:358-370`) | **already generalised** — it reads `self.state.blocking_decision().is_some()` and stops for *any* variant | **none**. Verify by reading; do not "improve" it |
| TUI bot loop | `tools/tui/src/play/mod.rs:83-115` via `PlayApp::acting_player` (`app.rs:234-266`) | **already generalised** — `if let Some(decision) = self.state.blocking_decision() { return decision.player(); }` | **none** |
| `LocalGame::advance` | `crates/simulator/src/local_game.rs:271-451` | partially — the branch at `:335-338` reads `blocking_decision()` but hard-maps the result to `DecisionKind::CleanupDiscard` | **exhaustive `match`** — §7.3 |
| `GameDriver::run_game` | `crates/simulator/src/driver.rs:62-126` | re-expressed on `LocalGame`; `:122-125` asserts `AwaitingHuman` unreachable with empty `human_seats` | **none directly** — covered by `LocalGame` + `StubProvider` + the bots. But T17 must prove it, because a provider gap here turns into `unreachable!()` |
| `mtg-fuzzer` | `crates/simulator/src/bin/fuzzer.rs:374-375` | uses `GameDriver::run_game` | **none directly**; §10 |
| replay-harness script driver | `crates/engine/src/testing/replay_harness.rs` + the script runner | DP-7 relied on "no script reaches a cleanup discard" | **a pump is mandatory** — §7.5 |

### 7.2 `LegalAction`

`crates/simulator/src/legal_actions.rs`, appended to the enum (currently ends at `DiscardToHandSize`
`:151-155`):

```rust
/// CR 603.3d / CR 601.2c (PB-DP8 / DP-6): announce the targets of a triggered
/// ability being put on the stack. `slots` is the full per-slot candidate set so a
/// human client can render a real picker; `targets` is the deterministic default
/// (`mtg_engine::rules::abilities::default_trigger_targets`), which the engine is
/// guaranteed to accept (SR-38: never offer an action the engine rejects).
ChooseTriggerTargets {
    choice_id: u64,
    source: ObjectId,
    slots: Vec<TriggerTargetOption>,
    targets: Vec<Vec<Target>>,
},
```

Exactly one such action is offered, and the provider block early-returns.

`StubProvider::legal_actions` (`legal_actions.rs:206-245`): the existing PB-DP7 block already opens
with `if let Some(decision) = state.blocking_decision() { match decision { … } return actions; }`
and the `match` is **exhaustive**, so adding a `BlockingDecision::TriggerTargets` arm is
compile-forced. Fill it by reading `state.pending_trigger_targets()` for the slot payload (the raw
accessor is correct here — `blocking_decision()` already applied the liveness filter to decide we are
blocked at all).

**OOS-DP7-5 is acknowledged, not solved.** A trigger-target answer is a per-slot structure, and it
fits `PendingDecision.actions: Vec<LegalAction>` only because the whole structure is packed into one
`LegalAction` — the same trick PB-DP7 used and the one OOS-DP7-5 says has run out of runway. That
seed recommends reshaping to `payload: DecisionPayload` **before** PB-DP8; that reshape is M11-local
Session 3/5's call and is not taken here. What PB-DP8 adds is a second data point that the flat list
is wrong, and it does so without making the eventual reshape harder (the payload is a named struct,
not a tuple).

### 7.3 `DecisionKind` and `LocalGame`

`crates/simulator/src/local_game.rs:105-116` — add `TriggerTargets`. The enum is already
`#[non_exhaustive]` (PB-DP7, audit §9.4 rec 1). Update its doc comment (`:92-102`), which currently
says it *"does NOT yet reach the trigger-time (PB-DP8, CR 603.3d) … decision class"*.

**`advance()`'s acting-player chain (`:335-338`) is a latent bug this batch must fix:**

```rust
let (acting_player, forced_kind) = if let Some(decision) = self.state.blocking_decision() {
    (decision.player(), Some(DecisionKind::CleanupDiscard))     // <-- ignores the variant
} else if …
```

It ignores the variant and would silently label a trigger-target decision `CleanupDiscard`, handing a
browser client the wrong picker. Convert to an exhaustive `match decision { … }`. Note that
`BlockingDecision` is a plain `enum` with no `#[non_exhaustive]`, so this becomes compile-forced for
every future variant — which is the point.

Nothing else in `LocalGame` changes. Specifically:
- `submit()` (`:457-526`) needs no change; `command_player` (`:559-563`) extracts `player` from the
  externally-tagged JSON and works for `ChooseTriggerTargets { player, .. }`. Its pinning test
  (`test_command_player_extracts_acting_player`) should gain the new variant.
- The empty-legal-actions auto-pass (`:363-381`) is unreachable while blocked — the provider always
  offers exactly one action for the entry's player.
- The idempotence guard (`:288-292`) already makes a repeated `advance()` return the same `seq`
  (T16).
- The bot-command-rejected fallback (`:426-449`) issues `PassPriority`, which the admission gate
  rejects ⇒ `Halted(EngineError)`. That is **OOS-DP7-12**'s shape, now with a second decision class
  behind it. Add a sentence to the `LegalActionProvider` trait doc making the obligation explicit:
  *"a provider MUST offer an answer for every `BlockingDecision` variant; failing to do so converts a
  recoverable state into a dead game."*

### 7.4 Bots

Both match `LegalAction` **exhaustively with no `_` arm** and are compile-forced:
- `crates/simulator/src/random_bot.rs::action_to_command` (`:128-371`) → `Command::ChooseTriggerTargets
  { player, choice_id: *choice_id, targets: targets.clone() }`. **It must submit the offered default
  verbatim, not randomise**: randomising would be a legitimate improvement to bot play and a
  *disaster* for this batch, because it would change fuzzer outcomes on every seed and destroy the
  A/B oracle (§10). Seed the improvement as **OOS-DP8-1**.
- `crates/simulator/src/heuristic_bot.rs` scorer (`:107-111` neighbourhood) → score 100, matching the
  `DiscardToHandSize` arm's precedent and rationale.

### 7.5 Replay harness — the pump is mandatory

DP-7 could rely on "no approved script reaches a cleanup discard with an oversized hand". **PB-DP8
cannot.** 74 `Complete` defs carry targeted triggers and there are 210 approved scripts; some will
reach a non-forced choice, and without a pump the script halts and every later action returns
`CommandRejected(BlockedByPendingDecision)`.

1. **New helper in `crates/engine/src/testing/replay_harness.rs`:**
   ```rust
   /// CR 603.3d (PB-DP8): answer every outstanding blocking decision with the
   /// engine's own deterministic default, until none remains. Used by the script
   /// driver so an existing golden script that reaches a targeted trigger keeps its
   /// pre-PB-DP8 behaviour byte-for-byte. A script that wants to CHOOSE uses
   /// `ScriptAction::PlayerAction { action: "choose_trigger_targets", .. }` instead;
   /// the driver skips this pump when that is the next action.
   pub fn auto_answer_blocking_decisions(state: &mut GameState) -> Vec<GameEvent>
   ```
   It loops: read `state.blocking_decision()`; for `TriggerTargets`, build
   `default_trigger_targets(&entry.slots)` and run it through `process_command` (not through the
   handler directly — the pump must exercise the same path a client does); for `CleanupDiscard`, the
   existing `default_cleanup_discard`. Bounded by a `debug_assert` counter so a mis-implemented
   default cannot loop forever.
2. **Call it from the script driver** — the runner must locate the per-step loop that consumes
   `GameScript.actions` (it lives with `replay_script()`, which the module doc at `:22-23` says stays
   in the test file) and invoke the pump **after applying each action**, skipping when the *next*
   action is `PlayerAction { action: "choose_trigger_targets", .. }`.
3. **New action arm** beside `"discard_to_hand_size"` (`:917-927`):
   `"choose_trigger_targets"` → resolve a new `#[serde(default)] trigger_targets: Vec<Vec<ActionTarget>>`
   field against the outstanding entry's candidate sets; empty ⇒ the default.
4. **`crates/engine/src/testing/script_schema.rs`** — `ScriptAction::PlayerAction` is
   `#[serde(deny_unknown_fields)]`, so declare the new field (follow the `discard_cards` pattern at
   `:482-489`) and extend the `action:` doc list at `:254-271`.
5. **SR-9c**: no new *assertion* path, so no `check_assertions` work. Recommended (not required): one
   new approved script that actually chooses a non-default target — the JSON regime is the best place
   to document that the choice is real.

### 7.6 TUI surfaces

- `tools/tui/src/play/input.rs` — a `'t'` key mirroring the `'d'` key at `:43-62`: find
  `LegalAction::ChooseTriggerTargets` in `legal`, submit `Command::ChooseTriggerTargets` with the
  offered default. A real picker is M11-local Session 7 (**OOS-DP8-2**, the sibling of OOS-DP7-6).
  The TUI has no exhaustive `LegalAction` match (it uses `matches!` probes), so nothing
  compile-breaks and a missing key is a *hang* — this is not optional.
- `tools/tui/src/play/panels/action_menu.rs:139-142` — a `[t]` hint beside the `[d]` hint.
- `tools/tui/src/play/app.rs` event formatter (`:603-608` neighbourhood, below a `_ => String::new()`
  catch-all) — a display arm for `TriggerTargetChoiceRequired`.

### 7.7 Replay viewer

- `tools/replay-viewer/src/view_model.rs` matches `StackObjectKind` and `KeywordAbility`
  exhaustively; **neither moves in this batch**, so there is likely nothing to do. **Verify with
  `cargo build --workspace`; do not assume** (the standing ~50%-miss warning).
- `tools/replay-viewer/frontend/src/lib/eventFormat.js` — add a
  `case 'TriggerTargetChoiceRequired':` beside the `'CleanupDiscardChoiceRequired'` cases at `:60-61`
  (display) and `:438` (category). **JS has no compile gate; this is the easiest silent miss in the
  batch.**

---

## 8. Liveness and the moment guard (PB-DP7 lessons 1 and 2, applied preemptively)

### 8.1 "What if the player it names leaves the game?"

The CR answers this directly and better than DP-7's ad-hoc filter did:

> **CR 800.4d** — *"If a triggered ability that would be controlled by a player who has left the game
> would be put onto the stack, it isn't put on the stack."*

So a departed controller's trigger is **dropped**, not defaulted, and CR 800.4g's "the controller
chooses another player to make that choice" does not arise — there is no ability left to choose for.

Three states, three answers:

1. **The controller is already `has_lost`/`has_conceded` when the flush reaches their trigger.** Do
   not record an entry — nobody can answer it (this is DP-7 Finding 1's exact failure mode). Use
   `default_trigger_targets(&slots)` and place the trigger, i.e. **today's behaviour, unchanged**.
   *Deliberately not* the CR 800.4d drop: dropping it would be a behaviour flip on a path PB-DP8 is
   not chartered to change, and the current engine never implements CR 800.4a/800.4d at all (a dead
   player's objects persist — **OOS-DP7-9**). Seeded as **OOS-DP8-5** with the CR 800.4d citation.
2. **The controller concedes while their entry is outstanding.** `handle_concede`
   (`engine.rs:2286-2305` neighbourhood, beside the existing `pending_cleanup_discard` clear) must,
   **before** its `handle_all_passed` / advance-turn logic:
   - take `state.pending_trigger_targets` if its `player == player`;
   - **drop `entry.trigger`** (CR 800.4d — this one *is* the CR-correct drop, because the player has
     left *now*, mid-announcement, and the ability was never put on the stack);
   - drop from `entry.remaining` every trigger whose `controller == player` (same rule);
   - `flush_sorted(state, entry.remaining_filtered, None)` to finish the batch, which may legitimately
     suspend again on a *different* player's trigger — that is correct and the game stays blocked on
     them;
   - extend `events` with the result.

   Without this, `remaining` is lost, CR 603.3b's batch is silently truncated, and CR 800.4j's "that
   turn continues to its completion" is violated. **This is DP-7 Findings 1 + 5 in a single site;
   write it in the implement phase, not the fix phase.**
3. **Another player concedes while the entry is outstanding.** The entry is untouched, no advance
   happens, the block persists. Correct.

`blocking_decision`'s liveness filter still applies to the new variant (belt and braces), and the
field must still be **cleared** on concede even though the filter would hide it, or it pollutes the
state hash forever.

### 8.2 The moment guard

`choice_id`, validation item (4) in §5.5. This is the analogue of PB-DP7 Finding 2's missing
`turn.step` check, and it is stronger: DP-7's guard was "is this the right *moment in the turn*";
`choice_id` is "is this the right *question*", which is what a sequence of questions inside one batch
requires. A `turn.step` check would be useless here — a flush can suspend in any step.

`LocalGame`'s `seq` is a *second*, simulator-level guard covering a different failure (a stale browser
tab). Neither replaces the other.

---

## 9. Scope call on OOS-DP3-4 (modal triggered abilities) — **OUT**

The audit says *"Adjacent to DP-6 … bundle with PB-DP8."* I am not bundling it, and the reasons are
about the rules and the DSL, not the schedule.

**What it is.** `rules/abilities.rs:8496-8541` sets `stack_obj.modes_chosen = vec![0]` for every modal
trigger with at least one mode; its `if min_modes == 0 { vec![0] } else { vec![0] }` has two identical
branches, and CR 700.2b's *"If no mode is chosen, the ability is removed from the stack"* is never
implemented.

**Corpus size, derived**: multiline `intervening_if:(?s:.){0,4000}?modes: Some\(` over
`crates/card-defs/src/defs/` returns **8 files** — `retreat_to_kazandu`, `retreat_to_coralhelm`,
`felidar_retreat`, `glissa_sunslayer`, `junji_the_midnight_sky`, `umezawas_jitte`,
`hullbreaker_horror`, `shambling_ghast` — of which `glissa_sunslayer` and `junji_the_midnight_sky`
carry non-`Complete` markers and `umezawas_jitte` is plausibly an *activated* modal caught by the lazy
match. So roughly **5 effectively-`Complete` defs**.

**Why out:**

1. **It is a different question with a different CR** (700.2b / 603.3c: a set of mode indices) and
   needs its own `Command` payload. Nothing about it reuses `ChooseTriggerTargets`'s slot structure.
2. **CR 700.2c orders the two questions and this DSL cannot express the ordering.** *"If a spell or
   ability targets one or more targets only if a particular mode is chosen for it, its controller
   will need to choose those targets only if they chose that mode."* Modes must be announced
   **before** targets, and the target requirements must then be derived **from the chosen mode**. In
   the current DSL `targets` is a field of the *ability*, not of the mode
   (`card_definition.rs:338-365`), so today the dependency does not bite — but designing the
   mode round trip now would bake in a shape that must be redesigned the moment per-mode targets
   exist. That is a bad trade for 5 cards.
3. **CR 700.2b's "removed from the stack" is a rules fix, not an agency fix.** PB-DP8's governing
   invariant is *"the compliant CR 603.3d fallback is preserved"*. Folding in a rule the engine has
   never implemented mixes classes and makes the review harder to reason about — PB-DP6's lesson
   about false negatives hiding inside a class-A row.
4. **Blast radius.** PB-DP8 already spans a PROTOCOL bump, a HASH bump, 4 guarded call sites, 14
   direct test call sites, 5 driving loops, a harness pump, and 53 sentinel re-pins. Adding a second
   roster on top is how a batch stops being reviewable.

**Recommendation**: rank it as **PB-DP8b**, immediately after this batch, where it costs one more
`BlockingDecision` variant, one more `Command`, one more `LegalAction` arm and **zero new consult
sites** — precisely the reuse this batch's machinery is meant to buy. Leave OOS-DP3-4 open with a
note pointing here.

---

## 10. Determinism, and the fuzzer

**The default is a pure function of `GameState`**, so the class of determinism that matters is
preserved by construction:
- `state.objects` is an `imbl::OrdMap` — `.iter()` is ascending by `ObjectId`, so `.filter().collect()`
  yields the same order `.find()` walked, and `candidates[0]` is the old `.find()` result.
- `state.turn.turn_order` is a `Vec` in seat order.
- No `HashSet`/`HashMap` iteration is involved anywhere in this path.
- `choice_id` comes from `timestamp_counter`, which is already deterministic and already hashed.

**What genuinely changes for a bot game**, stated so the runner expects it and does not paper over it:
- One extra `Command` per *non-forced* targeted trigger. `LocalGame.command_count` (`:164`) rises and
  moves games toward `limits.max_commands`. The fuzzer sets `max_consecutive_passes: 500`
  (`driver.rs:74`) and takes `max_commands` from its CLI — **check the default and raise it if the
  A/B shows games truncating**. A truncated game is a *false* determinism signal.
- `consecutive_passes` resets on each answer, making the `max_consecutive_passes` valve slightly less
  likely to trip. Both are safety valves, not semantics.
- `loop_detection::reset_loop_detection` fires on each answer (CR 104.4b), changing
  `loop_detection_hashes` in pathological games. Correct, not a regression.
- The forced-choice narrowing (§5.3) is what keeps all of the above small.

**Fuzzer A/B, and its honest limits.** **OOS-DP3-9**: `mtg-fuzzer` aborts with a stack overflow at
~15 games on `main` and long games flood `stack_consistency` violations; **OOS-M11-3**: the fuzzer is
not run-to-run deterministic in 150-200+ turn games. Both are **pre-existing**. Do not chase them and
do not let them mask a regression. The usable check is a **fixed-seed A/B**: run N seeds on `main`
and on the branch with `--games` small enough to complete, and confirm winner and turn count match for
every seed that completes on both. Any *winner* or *turn-count* change on a completing seed is a bug.
If the A/B cannot be run at all, say so in the commit message (PB-DP7's runner did, and its review
accepted it) and lean on T17.

---

## 11. Tests

**New file**: `crates/engine/tests/primitives/pb_dp8_trigger_target_choice.rs`, registered in
`crates/engine/tests/primitives/mod.rs`. **SR-9a: never add a top-level `tests/*.rs`; a dropped `mod`
line silently deletes coverage.**
**Simulator tests**: `crates/simulator/tests/local_game.rs` (the established home — PB-DP7's T14/T15
moved there) and `crates/simulator/src/legal_actions.rs`'s `mod tests`.

**Shared helper** (put it in the new test file and re-export, or in the primitives test module's
common helpers):
```rust
/// Answer any outstanding CR 603.3d target choice with the engine's own default,
/// through `process_command`. Panics if nothing is pending — so it can never mask a
/// missing block (the `answer_pending_cleanup_discard` precedent from PB-DP7).
fn answer_pending_trigger_targets(state: GameState) -> (GameState, Vec<GameEvent>)
```

### 11.1 New tests, with per-test fail-before predictions

Every row cites CR 603.3d; rows about sequencing also cite CR 603.3b.

| # | test | asserts | fail-before probe (expressible on `main`) |
|---|---|---|---|
| T1 | `test_dp8_flush_blocks_on_a_real_target_choice` | 2 legal creature targets for one ETB trigger; after the flush: `pending_trigger_targets().is_some()`, **no** `StackObjectKind::TriggeredAbility` on the stack, no `PriorityGiven`, exactly one `TriggerTargetChoiceRequired` whose `slots[0].candidates.len() == 2`. **ESM "a test observes the block".** CR 603.3d | assert `state.stack_objects().is_empty()` after the flush — **fails today** (the trigger is already on the stack with an auto-picked target) |
| T2 | `test_dp8_chosen_target_is_honoured_not_first_match` | answer naming the **higher**-`ObjectId` creature; the stack object's `targets[0]` is that creature | assert the trigger's target is the higher id — **fails today** (`.find` takes the lowest) |
| T3 | `test_dp8_illegal_target_rejected_state_untouched` | a `Target` not in `slots[0].candidates` (a hexproof creature, an opponent's creature under a `TargetController::You` filter, a graveyard card for a battlefield slot) → `Err`; `public_state_hash` byte-identical before/after. CR 603.3d + 601.2c | new-surface-only |
| T4 | `test_dp8_no_legal_candidate_still_removes_the_trigger` | a required slot with zero candidates ⇒ **no question**, no stack object, `AbilityTriggered` absent. CR 603.3d | **passes today** — regression guard for the compliant fallback the batch must not break |
| T5 | `test_dp8_forced_single_candidate_asks_nothing` | exactly one legal target, no optional slot ⇒ no `TriggerTargetChoiceRequired`, trigger on the stack targeting it, priority granted normally. CR 601.2c (§5.3) | passes today by outcome; the *absence of a question* is new-surface |
| T6 | `test_dp8_apnap_sequence_across_two_controllers` | P1 (active) and P2 each control a targeted trigger with 2 candidates. Question 1 names **P1** (CR 101.4/603.3b); answer; question 2 names **P2**; answer; both on the stack, P1's below P2's; two distinct `choice_id`s. CR 603.3b | the stack-order half is expressible today (assert AP's trigger is at the bottom) and **passes**; the two-question half is new-surface |
| T7 | `test_dp8_resume_after_partial_flush_places_each_trigger_exactly_once` | 3 triggers, only the 2nd needs a choice. At the pause: trigger 1 on the stack, 2 and 3 nowhere, `pending_triggers` **empty**, `entry.remaining.len() == 1`. After the answer: all 3 on the stack, in batch order, **each exactly once**; `pending_trigger_targets` `None` | new-surface; the "exactly once" clause is the guard against the drain/replay bug §3.1 describes |
| T8 | `test_dp8_stale_choice_id_rejected` | answer with `choice_id + 1`, and with `choice_id` of a *previous* question in the same batch → `Err`, hash unchanged; answering with no entry → `Err`. **PB-DP7 lesson 2.** | new-surface |
| T9 | `test_dp8_sender_validation` | (a) `ChooseTriggerTargets` from a non-controller through `process_command` → `Err(BlockedByPendingDecision)` (the admission gate); (b) the same through a **direct call** to `handle_choose_trigger_targets` → `Err(InvalidCommand)` (the SR-29 check). **Assert the specific error in each case** — PB-DP7 review Finding 12 | new-surface |
| T10 | `test_dp8_controller_concedes_mid_choice` | P2's trigger is the outstanding question, with one more P1 trigger in `remaining`. P2 concedes → entry cleared, **P2's trigger never reaches the stack** (CR 800.4d), P1's trigger **does** (CR 603.3b batch completes), game advances, no hang. CR 104.3a/800.4d/800.4j | new-surface |
| T10b | `test_dp8_dead_controller_is_never_asked` | controller marked `has_lost` before the flush ⇒ no entry, no event, trigger placed with the default (today's behaviour). §8.1 case 1 | passes today by outcome; the *absence of an entry* is the assertion |
| T11 | `test_dp8_admission_gate_while_suspended` | `PassPriority`, `CastSpell`, `TapForMana`, `PlayLand` from **any** seat → `Err(BlockedByPendingDecision)`; `public_state_hash` unchanged in every case. CR 603.3 | new-surface |
| T12 | `test_dp8_no_priority_granted_while_suspended` | suspend from **`handle_declare_attackers`** (guard #4): assert no `PriorityGiven`, `priority_holder` unchanged; after the answer, `PriorityGiven { player: attacker }` appears exactly once. Repeat for `handle_declare_blockers` (#5) and the resolution tail (#6). CR 603.3/117.3a | new-surface; this is the §4.1 guard set's own test |
| T13 | `test_dp8_default_reproduces_pre_pb_behaviour` | for each requirement family in §5.2 (player, opponent, own-graveyard, any-graveyard, `UpToN` player-inner, `UpToN` permanent-inner, battlefield-with-filter), `default_trigger_targets` equals the value the pre-PB `.find` chain produced. The determinism pin | passes by construction; it is the pin |
| T14 | `test_dp8_candidate_set_is_wider_than_the_default` | (a) `TargetCreatureOrPlayer` offers **players and creatures**, `default` is the first opponent; (b) `TargetPlayer` offers the controller too, `default` is an opponent; (c) `UpToN{TargetCreature}` is `optional: true` with `default: None`, and an empty answer for it is **accepted**. CR 601.2c | new-surface (the object-arm branches are unreachable today) |
| T15 | `test_dp8_target_opponent_never_self_and_never_asks_when_alone` | 1v1, opponent dead ⇒ trigger removed, no question (PB-EF6 regression guard); 4-player ⇒ 3 candidates, none of them the controller. CR 603.3d/102.3 | the "never self" half **passes today**; the candidate-count half is new |
| T16 | `test_dp8_local_game_awaits_human` (simulator) | human seat: `advance()` → `AwaitingHuman { kind: DecisionKind::TriggerTargets, player, actions.len() == 1 }`; a second `advance()` returns the **same** `seq`; `submit(seq, cmd naming another seat)` → `BadParams`; `submit(stale_seq, ..)` → `StaleDecision`; a correct `submit` proceeds. **Must assert `kind == TriggerTargets`** — that is §7.3's latent bug | new-surface |
| T17 | `test_dp8_bot_game_never_halts_on_a_trigger_target` (simulator) | bot-only `LocalGame`, seeded, a deck with a targeted-trigger `Complete` def, ≥5 turns: never `Halted`, at least one `ChooseTriggerTargets` in the journal | new-surface; guards the `driver.rs:122` `unreachable!()` |
| T18 | `test_dp8_stub_provider_offers_only_the_answer` (simulator) | the blocked player gets exactly one action, `ChooseTriggerTargets`, with `targets.len() == slots.len()`; every other player gets `[]`; and the offered default is **accepted by `process_command`** (SR-38) | new-surface |
| T19 | `test_dp8_roster_enumeration` | the `all_cards()` walk of §1.4: prints the roster, `assert!(n >= 60)` | new-surface; **its printed number is the deliverable** |

### 11.2 Existing tests and scripts predicted to change

**None of these may be repaired by weakening an assertion.** Every one is "the test now *chooses*
instead of relying on the auto-picker", answered with `answer_pending_trigger_targets`.

**Direct `flush_pending_triggers` call sites (14, mechanically enumerable):**
`tests/primitives/pb_ef6_target_opponent.rs:573`, `:702`;
`tests/primitives/pb_ac1_untap_counter.rs:320`, `:396`, `:767`, `:793`, `:872`;
`tests/primitives/primitive_pb_lki_cc.rs:603`, `:716`, `:811`;
`tests/primitives/pb_ac2_card_integration.rs:618`;
`tests/primitives/pb_ef4_triggering_creature_subject_source.rs:104`;
`tests/primitives/pb_ac8_restrictions_and_wingame.rs:385`, `:420`;
`tests/primitives/pb_os9_lieutenant_commander_control.rs:657`, `:716`, `:790`;
`tests/mechanics_e_l/encore.rs:438`; `tests/rules/delayed_triggers.rs:578`.
Most build boards with a single legal target and are unaffected by §5.3's narrowing — **verify, do
not assume**; `pb_ef6_target_opponent.rs` in a 4-player fixture is the most likely to now ask.

**Indirect fallout** is not enumerable by grep: any test whose board reaches a targeted trigger with
2+ candidates now suspends. The procedure is: run `cargo test --all`, and for each failure decide
whether the right repair is (a) insert the answer helper, or (b) recognise that the narrowing should
have applied and the candidate set is wrong. **(b) is a bug in this batch, not in the test** — treat
every "the candidate set is bigger than I expected" failure as a finding before treating it as
fallout.

**Golden scripts**: run the full suite (`cargo test --test run_all_scripts`), 210 approved, **0 new
skips** (SR-9c). If any script halts, the §7.5 pump is not wired correctly — fix the pump, do not
edit the script.

---

## 12. Hidden information (Architecture Invariant 7)

`GameEvent::TriggerTargetChoiceRequired.slots[..].candidates` carries `SpellTarget`s naming:
- battlefield objects (public zone),
- graveyard cards (public zone — CR 400.2),
- players.

**No hidden-zone object can appear**: the requirement families that reach the choice are
battlefield-scanning, graveyard-scanning, or player-picking; there is no library-, hand- or
exile-scanning `TargetRequirement` in the fallback's match, and the `_ =>` battlefield arm filters on
`obj.zone != ZoneId::Battlefield` (`:7565`).

**Decision**: `reveals_hidden_info()` → **`false`**, i.e. leave it on the `_ => false` catch-all at
`events.rs:1382` neighbourhood. Record the decision in the variant's doc comment (as above) rather
than only in this plan, so a later reader sees it was deliberate.

**`private_to()` — a falsified premise, recorded.** CLAUDE.md's Architecture Invariant 7 says private
events go to the relevant player *"via `GameEvent::private_to() -> Option<PlayerId>`"*. **That method
does not exist**: `rg 'private_to' crates/` returns **zero matches** on this branch. It is a design
statement about the unbuilt M10 network layer, not a surface PB-DP8 can populate. So the honest answer
to the brief's `private_to()` question is: *there is nothing to answer, and the invariant's phrasing
is stale.* Seeded as **OOS-DP8-6**.

**Sibling check (OOS-DP7-3's standing obligation)**: `AbilityTriggered` (the event this batch's
successful path still emits) returns `false` — correct, it names public stack/permanent ids.
`CleanupDiscardChoiceRequired` returns `false` and **should be private-to-`player`** when the M10
filter exists — that is OOS-DP7-3(b), unchanged and not fixed here.

---

## 13. Pre-survey bullets that turned out to be WRONG

Verified against source on this branch, 2026-07-26. Confirmations are listed only where load-bearing.

1. **The inherited spec's headline prediction is wrong.** `pb-plan-DP7.md` §1.5 and the audit's §8
   PB-DP7 row both say *"DP-8's pending state is a `Vector`"*. It is an `Option`, and §2.2 gives the
   CR argument: "answered as a sequence" is exactly what makes the outstanding set a singleton. The
   plurality is real and lives in `remaining` **inside** the entry.
2. **The audit's card count is not reproduced.** §5 DP-6 says **84**; a mechanical corpus derivation
   gives **74** effectively-`Complete` defs (§1.4). Two of the seven `intervening_if: Some` hits are
   regex false positives verified by reading (`thaumatic_compass.rs`, `siege_gang_lieutenant.rs` —
   both have `targets: vec![]` on the Triggered and the lazy match ran on into a later `Activated`).
   The number is still a grep estimate; §1.4 prescribes the `all_cards()` enumeration that makes it a
   fact (SR-36).
3. **The audit's DP-6 site cites are stale by ~137 lines.** §7 cites `:7434` for
   `expect_characteristics`, `:7438-7450` for `validate_target_protection`, `:7272-7291` for the
   `TargetOpponent` self-guard; the real lines are `:7571`, `:7573-7582`, `:7410-7425`. The stated
   range `:7174-7500` also undershoots — the arm runs to `:7798`. Same class as **OOS-DP6-8**: *a site
   cite in that document is a snapshot.*
4. **The `flush_pending_triggers` guard set is far smaller than the inherited spec predicted.** §1.5
   warns that the guard set is *"~20 command paths plus both `enter_step` branches"*. It is **four**
   guards. All 29 `check_and_flush_triggers` sites need none, because every one is followed by exactly
   `all_events.extend(events);` and `process_command`'s tail (`engine.rs:722-726`) only pushes history
   (§4.2). PB-DP1's priority-to-actor work is what made that true — the handlers now assign priority
   *before* the flush.
5. **The TUI needs no loop work.** PB-DP7's closing review (lesson 4) says PB-DP8 *"must enumerate the
   driving loops"*, implying the TUI's auto-pass loop is at risk again. It is not: the fix cycle wrote
   `should_stop_auto_pass` (`app.rs:358-370`) and `acting_player` (`app.rs:253-255`) against
   `blocking_decision()`, so both generalise to a new variant for free. Only the input key, the menu
   hint and the event formatter need arms. **Verified by reading, not assumed.**
6. **`LocalGame` has a latent variant-blindness bug that PB-DP7's review did not catch.**
   `local_game.rs:335-338` does `if let Some(decision) = self.state.blocking_decision() { (decision.player(), Some(DecisionKind::CleanupDiscard)) }`
   — it reads the liveness-filtered predicate (correct) and then hard-codes the *kind* (wrong).
   PB-DP8 must convert it to an exhaustive `match`. Contrast `StubProvider::legal_actions`
   (`legal_actions.rs:224-245`), which *does* `match decision` exhaustively and is therefore
   compile-forced.
7. **`GameEvent::private_to()` does not exist**, despite Architecture Invariant 7 naming it as the
   mechanism. Zero matches in `crates/`. §12.
8. **`SpellTarget` is not currently in the wire closure**; `command.rs` has zero `SpellTarget`
   references and carries bare `Target` (`:104`, `:280`, `:644`, `:674`). So this batch's closure
   **type count changes** (by `TriggerTargetOption` and `SpellTarget`), unlike PB-DP7's, where it did
   not. Say so in the `- 29:` History line.
9. **The object-arm branches for `TargetCreatureOrPlayer` / `TargetAny` /
   `TargetPlayerOrPlaneswalker` (`:7603`, `:7688-7693`) are dead code today** — the player arm at
   `:7378-7405` returns first for all three. They are not vestigial; they are the other half of the
   CR 601.2c candidate set, and §5.2's union is what makes them live.
10. **`build_face_ability_vectors` *does* forward trigger targets** (`replay_harness.rs:2271`, `:2309`,
    `:2336`, and the `WhenDealsCombatDamageToPlayer` block at `:2347+` — `targets: targets.clone()`
    at every site). OOS-DP6-1's finding that it hardcodes `intervening_if: None` might have suggested
    it drops `targets` too; it does not, so the lowered `WhenDies`/`WhenAttacks`/`WhenBlocks` families
    genuinely reach the DP-6 site and are inside the 74.
11. **`TargetPermanentDistinctFrom` distinctness is unenforced for triggers**, contradicting its own
    comment at `:7595-7598` ("enforced at declaration validation (casting.rs)"). Zero corpus exposure;
    §1.3, **OOS-DP8-4**.

**Confirmed as stated** (recorded so the reviewer knows the checks ran): the four CR 603.3d-compliance
claims of audit §7 (§1.1); `state.objects` is an `imbl::OrdMap` so `.iter()` is ascending;
`timestamp_counter` is the monotone counter `next_object_id` increments (`state/mod.rs:940-943`);
`GameEvent`'s max hashed discriminant is **129** (`hash.rs:5341`), so 130 is next; `PendingTrigger`,
`SpellTarget`, `Target`, `TargetRequirement` all already have `HashInto`;
`GameStateError::BlockedByPendingDecision` exists and is outside the wire closure
(`state/error.rs:94-102`); `blocking_decision()` is already a public `GameState` accessor
(`state/mod.rs:487-489`) and all three PB-DP7 consumers already read it.

---

## 14. Seeds to file in `docs/audits/decision-point-audit.md` §8.1

| seed | finding | class |
|---|---|---|
| **OOS-DP8-1** | **`RandomBot` answers a CR 603.3d choice with the engine's default, not randomly.** Correct for this batch (a random answer would change every fuzzer seed's outcome and destroy the A/B oracle), and wrong for bot quality: the whole point of DP-6 is that first-match is a bad pick, and the bot now makes it deliberately. A randomising `RandomBot` and a target-evaluating `HeuristicBot` belong together with a fuzzer-baseline re-pin. | simulator quality, deferred |
| **OOS-DP8-2** | **The TUI answers with the default, not a picker** — the sibling of OOS-DP7-6. `tools/tui/src/play/input.rs`'s new `'t'` key submits `default_trigger_targets` verbatim, i.e. exactly the pre-PB-DP8 auto-pick, so a human at the TUI seat gets the old behaviour plus a keystroke. The real picker is M11-local Session 7, which per audit §9.4 rec 9 must also stop believing `TargetPicker.svelte` covers trigger targets. | UX gap, M11-local |
| **OOS-DP8-3** | **Modular's trigger target is still auto-picked** (`rules/abilities.rs:7877-7888`, CR 702.43a "target artifact creature", lowest `ObjectId` on the battlefield). It sits in the `kind` match with its own `continue`, not in `trigger_targets_opt`, so PB-DP8's machinery does not reach it. One more slot-derivation call site; cheap once this batch lands. | agency loss, narrow |
| **OOS-DP8-4** | **`TargetPermanentDistinctFrom` distinctness is unenforced for triggered abilities and the comment says otherwise.** `abilities.rs:7595-7598` returns `true` for the requirement and says distinctness is "enforced at declaration validation (casting.rs)" — which is true for spells and false for triggers, where nothing validates a declaration. Two such slots on one trigger get the same object today. Zero corpus exposure (no `Triggered` def uses it). PB-DP8 adds only the narrow cross-slot rejection in the answering handler; the auto-fallback keeps the defect. Aspirationally-wrong comment ⇒ correctness hazard per `memory/conventions.md`. | correctness, latent |
| **OOS-DP8-5** | **CR 800.4d is unimplemented for the auto-fallback path.** A trigger controlled by a player who has left the game is still put on the stack; PB-DP8 only declines to *ask* such a player (and does drop the trigger on the concede-while-blocked path, where the departure is simultaneous with the announcement). Root cause is **OOS-DP7-9** (CR 800.4a object removal unimplemented; a dead player's objects persist). Closing it is a behaviour flip on every dead-player trigger and needs its own roster. | correctness, deferred |
| **OOS-DP8-6** | **`GameEvent::private_to()` does not exist.** CLAUDE.md's Architecture Invariant 7 names it as the mechanism by which private events reach only the relevant player; `rg 'private_to' crates/` returns zero. Today the only hidden-info surface is the boolean `reveals_hidden_info()`, which cannot express "this event is for one seat". M10's filter needs the method, and **OOS-DP7-3(b)** (`CleanupDiscardChoiceRequired` broadcasting a hand's `ObjectId` set) is already waiting on it. Minimum action: correct the invariant's wording so it stops asserting a surface that does not exist. | documentation-vs-code / M10-gated |
| **OOS-DP8-7** | **`OOS-DP3-4` (modal triggered abilities) is deliberately not bundled here — rank it as PB-DP8b.** ~5 effectively-`Complete` defs. Reasons in this plan's §9: it is CR 700.2b/603.3c not 603.3d, CR 700.2c orders modes before targets in a way the ability-level `targets` field cannot express, and its "no mode chosen ⇒ removed from the stack" half is an unimplemented *rule* rather than an agency gap. On top of PB-DP8's machinery it costs one `BlockingDecision` variant, one `Command`, one `LegalAction` arm and **zero new consult sites**. | scope call, ranked candidate |
| **OOS-DP8-8** | **§10's re-audit triggers are due again.** A new `Command` (DP-24's accepted-and-discarded-field check: the answer is **not** one — `choice_id` is validated, every `Target` is validated against the offered candidate set, and `zone_at_cast` is deliberately taken from the engine rather than the wire) and a new `GameEvent` (the `reveals_hidden_info` sweep, answered in §12). Also still owed from OOS-DP7-7: §3.1's 277-def re-derivation. | bookkeeping |

**Audit cross-references to update when this ships**: §5 **DP-6** row (SHIPPED banner, the corrected
card count, the corrected site lines from §13 item 3), §7 **OOS-M11-4** (CLOSED), §8 the **PB-DP8**
row (wire prediction confirmed on both halves; the `Vector`-vs-`Option` correction; the four-guard
result vs the "~20 command paths" prediction), §8's sequencing note (point PB-DP9 at this plan's §3
for the resume shape *and* at `pb-plan-DP7.md` §1.6 for why DP-9 still does not inherit it), §9.3/§9.4
recs 1/2/5/9, §10.

---

## 15. Risks and edge cases

1. **A missed guard grants priority mid-batch.** If any of the four §4.1 guards is omitted, the game
   grants priority with the CR 603.3b batch half-placed, the admission gate then rejects the
   resulting `PassPriority`, and the game *looks* stuck rather than *is* blocked. T12 pins all four.
   The mechanical check: after implementing, `rg 'flush_pending_triggers\(' crates/*/src` must return
   6 sites, and each of the four non-`check_and_flush_triggers` ones must have the guard immediately
   after.
2. **A missed consumer deadlocks a whole regime.** Five consumers must answer:
   `StubProvider`, `RandomBot`, `HeuristicBot`, the TUI key, and the harness pump. The first three are
   compile-forced (exhaustive matches with no `_` arm). **The TUI key and the harness pump are not**,
   and the harness pump is the higher risk because it is the one PB-DP7 did not need. A missing pump
   turns 210 green scripts red at once — which is loud, so it fails safe.
3. **The resume loses the tail.** The single hardest thing in this batch. `flush_pending_triggers`
   drains before iterating (§3.1), so `remaining` must be captured from the *not-yet-iterated* portion
   of `sorted`, and `flush_sorted` must not re-drain or re-sort. T7's "each trigger appears on the
   stack exactly once" is the assertion that catches every version of getting this wrong.
4. **The candidate-set widening is the batch's largest behaviour surface.** Turning `.find` into
   `.filter().collect()` for the ~220-line battlefield predicate is mechanical but long, and a
   transcription slip changes which targets are *legal*, not merely which is picked. Mitigation: T13
   pins that `default` is unchanged for every family, so any behaviour change shows up as a
   `default` mismatch rather than silently.
5. **Test fallout is not enumerable in advance.** §11.2 lists the 14 direct call sites; the rest are
   discovered by running the suite. The forced-choice narrowing (§5.3) is what keeps this bounded, and
   it is therefore load-bearing for schedule as well as for CR fidelity. **Treat every unexpected
   suspension as a possible candidate-set bug before treating it as fallout.**
6. **The concede path is the one place two CR rules meet.** CR 800.4d (drop the trigger) and CR 800.4j
   (the turn completes) both apply, and the batch's `remaining` must survive the concede. §8.1 case 2
   is the site; T10 is the pin. PB-DP7 shipped the analogous bug (Findings 1 and 5) and had to fix it
   in the fix cycle — do it in the implement phase here.
7. **Two version bumps in one commit, five gate-computed fingerprints.** The most likely process error
   in the batch is hand-editing a fingerprint or editing an existing history row. All of it is
   machine-caught (`history_is_append_only`, `frozen_prefix_is_pinned`,
   `declaration_fingerprint_is_pinned`) — read the failures, do not guess.
8. **The SR-19 gate can report success while checking nothing.** OOS-DP7-11. Write both `HashInto`
   impls with bare names and run the delete-a-field demonstration (§2.3). A comment claiming gate
   coverage without that demonstration is exactly the failure PB-DP7's closing review caught.
9. **`Vec<Vec<Target>>` on the wire is a validation surface.** Every SR-29 lesson applies: check the
   sender, check the `choice_id`, check the slot count, check per-slot cardinality, check membership.
   `handle_keep_hand` (OOS-DP2-1, checks only the count) is the cautionary tale.
10. **Fuzzer signal is already degraded.** OOS-DP3-9 and OOS-M11-3 mean a clean fuzzer A/B may be
    impossible. Do not chase them; do not let them mask a regression; say in the commit message
    exactly what was and was not run.

---

## 16. Verification checklist

- [ ] `cargo build --workspace` clean after **every** phase (SR-8; `tools/replay-viewer` and
      `tools/tui` are the two runners miss ~50% of the time)
- [ ] `cargo test --all` green — includes `tools/check-defs-fmt.sh` via `core card_defs_fmt` (SR-35)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh`
- [ ] `rg 'flush_pending_triggers\(' crates/*/src` returns **6** sites; the four in §4.1 each carry
      the suspension guard immediately after; the `check_and_flush_triggers` site carries **none**
      and the reason is in a comment
- [ ] `PROTOCOL_VERSION == 29`, fingerprint **gate-computed**, `PROTOCOL_HISTORY` row **appended**,
      `protocol_version_sentinel` + `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned
- [ ] `HASH_SCHEMA_VERSION == 66`, **both** fingerprints gate-computed, `HASH_SCHEMA_HISTORY` row
      **appended**, sentinel + `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned
- [ ] all **53** scattered sentinels from §6.3 re-pinned; **no new sentinel added** (OOS-DP7-8)
- [ ] `state/hash.rs` `GameEvent` match gained a `130u8` arm (no `_` arm exists — a miss is a compile
      error, which is the point)
- [ ] `HashInto for TriggerTargetOption` and `for PendingTriggerTargets` written with **bare** names;
      the delete-a-field demonstration run and its result recorded (§2.3, OOS-DP7-11)
- [ ] `NOT_HASHED` allowlist still empty
- [ ] `GameState` still sealed: new field `pub(crate)`, read accessor, **no** `_mut` accessor (SR-3)
- [ ] `pending_trigger_targets` folded into `public_state_hash` **and** `loop_detection.rs`'s
      mandatory-state fingerprint
- [ ] `local_game.rs`'s acting-player chain is an **exhaustive `match`** on `BlockingDecision`, not a
      hard-coded `DecisionKind` (§7.3 / §13 item 6)
- [ ] `random_bot::action_to_command` and `heuristic_bot`'s scorer gained `LegalAction` arms
      (compile-forced) and the bot submits the **default verbatim**
- [ ] TUI: `'t'` key, `[t]` menu hint, event-formatter arm; auto-pass loop and `acting_player`
      **verified unchanged and still correct**
- [ ] `eventFormat.js` gained a `TriggerTargetChoiceRequired` case in **both** places (`:60-61`
      display, `:438` category) — no compile gate, verify by reading
- [ ] the replay-harness pump is wired into the script driver, and the full golden-script suite runs:
      `cargo test --test run_all_scripts` — 210 approved, **0 new skips** (SR-9c)
- [ ] T19's `all_cards()` roster enumeration run; **the printed count written into the commit message
      and into audit §5's DP-6 row** (SR-36 — never ship a grep-derived roster)
- [ ] `debug_assert` that every slot's `default`, when `Some`, is a member of its `candidates`
- [ ] fixed-seed fuzzer A/B vs `main` on seeds that complete on both; `max_commands` checked for
      truncation; what was and was not run stated in the commit message (OOS-DP3-9 / OOS-M11-3 are
      pre-existing — do not chase, do not let them mask)
- [ ] 0 card-def source edits, 0 completeness flips — assert by `git diff --stat -- crates/card-defs/`
- [ ] `PB-DP7`'s `BlockingDecision` doc block (`engine.rs:63-99`) updated to name the **four**
      per-variant obligations a second variant actually hits (§5.6)
- [ ] `docs/audits/decision-point-audit.md` §5 DP-6 / §7 OOS-M11-4 / §8 PB-DP8 row + sequencing note /
      §9.3 / §9.4 / §10 updated; seeds **OOS-DP8-1..8** filed in §8.1
- [ ] `memory/workstream-state.md` handoff + CLAUDE.md Current State snapshot delta
