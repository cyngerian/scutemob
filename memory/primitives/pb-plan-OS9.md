# Primitive Batch Plan: PB-OS9 — Lieutenant / "you control your commander" condition (OOS-EF3b-1)

**Generated**: 2026-07-19
**Primitive**: `Condition::YouControlYourCommander` — a new `Condition` variant that is
true when the effect's controller currently **controls** (CR 903.3d) at least one
commander **they own** that is on the battlefield (phased-in). Serves BOTH the
intervening-if-on-a-triggered-ability shape and the continuous-grant ("as long as")
shape through the engine's two existing condition evaluators.
**CR Rules**: 903.3d (controlling a commander), 903.3 (commander designation is an
attribute of the card), 603.4 (intervening-if re-checked at resolution), 604.2
(conditional static abilities), 611.2a (control), 702.121a/b (Melee — the granted
keyword whose trigger now fires post-PB-EF3b).
**Cards affected**: 3 (3 existing partial→Complete fixes; 0 new). `legion_lieutenant` OUT.
**Dependencies**: PB-EF3b (granted keyword-triggers fire — required for Skyhunter's
granted Melee to actually trigger; SHIPPED `scutemob-104`). No other prerequisites.
**Deferred items from prior PBs**: none pulled in. Explicitly does NOT touch
OOS-EF9-1's WhileSourceOnBattlefield reversion half or OOS-OS7-2's 611.2c edge.

**TODO sweep (roster-recall gate)**: grepped `crates/card-defs/src/defs/` for
`Lieutenant`/`lieutenant` and for `YouControlYourCommander` / `control your commander`.
Result: exactly **4** Lieutenant files — `skyhunter_strike_force.rs`,
`loyal_apprentice.rs`, `siege_gang_lieutenant.rs` (all three name
`Condition::YouControlYourCommander` in their TODO/blocked notes — forced adds), and
`legion_lieutenant.rs` (name-only, no ability — OUT, see below). No other file
self-identifies as needing this primitive. Roster is closed at 3.

---

## Primitive Specification

The Lieutenant ability word (not a keyword; CR has no "Lieutenant" keyword) gates an
effect on "if/as long as you control your commander". Two syntactic shapes exist in the
corpus and BOTH must work:

1. **Intervening-if on a triggered ability** — `loyal_apprentice`, `siege_gang_lieutenant`:
   "At the beginning of combat on your turn, if you control your commander, create …".
   In the DSL this is `AbilityDefinition::Triggered { trigger_condition:
   AtBeginningOfCombat, intervening_if: Some(Condition::YouControlYourCommander), … }`.
   The DSL `intervening_if: Option<Condition>` is evaluated at **resolution time** via
   `crate::effects::check_condition` (resolution.rs:2125-2135) — exactly matching the
   Gatherer ruling "If you don't control your commander as the lieutenant ability
   resolves, you won't get its effect."

2. **Continuous-grant condition** — `skyhunter_strike_force`: "As long as you control
   your commander, other creatures you control have melee." In the DSL this is
   `AbilityDefinition::Static(ContinuousEffectDef { …, condition:
   Some(Condition::YouControlYourCommander) })`. `ContinuousEffectDef.condition` is
   evaluated at **layer-application time** via `crate::effects::check_static_condition`
   (layers.rs:558), which re-evaluates on every characteristics recompute — so it drops
   automatically the moment the commander leaves the battlefield or changes control (no
   bespoke reconcile site needed; this is the same machinery `WhileSourceOnBattlefield`
   grants use, but we are NOT widening into that SBA-removal path).

**Semantics (CR 903.3d):** "If an effect refers to controlling a commander, it refers to
a permanent on the battlefield that is a commander." Combined with "**your** commander"
(a commander the player **owns**): the condition is true iff there exists a battlefield,
phased-in object whose `controller == ctx.controller` AND whose `card_id` is in
`state.players[ctx.controller].commander_ids`. Because `commander_ids` is per-owner,
membership in the controller's own `commander_ids` inherently encodes ownership.

This is deliberately **stricter** than the existing CommanderFreeCast check
(`casting.rs:2353`, CR 118.9 "if you control **a** commander" — ANY player's commander).
Lieutenant is "**your** commander" — do NOT reuse or generalize the free-cast predicate.

Consequences (all decoy-pinned below):
- Opponent controls YOUR commander (they stole it) → `obj.controller != ctx.controller`
  → **false** (Lieutenant OFF). Correct.
- You stole your commander back → `obj.controller == ctx.controller` and card owned by
  you → **true** (ON). Correct.
- You control an OPPONENT's commander but not your own → their card is in THEIR
  `commander_ids`, not yours → **false** (OFF). Correct ("your commander").
- Multiple commanders (partner) → `.any()`; controlling one suffices (ruling: "you need
  to control only one"). Correct.
- Commander in the command zone / graveyard / exile → not on battlefield → **false**.

## CR Rule Text

**903.3d** — "If an effect refers to controlling a commander, it refers to a permanent
on the battlefield that is a commander. If an effect refers to casting a commander, it
refers to a spell that is a commander. If an effect refers to a commander in a specific
zone, it refers to a card in that zone that is a commander."

**903.3** — "Each deck has a legendary card designated as its commander. … This
designation is not a characteristic of the object represented by the card; rather, it is
an attribute of the card itself. The card retains this designation even when it changes
zones." (This is why membership keys off `card_id ∈ commander_ids`, not an object flag.)

**603.4** — an intervening-if triggered ability checks its condition when it would
trigger AND again when it resolves; if false at resolution it is removed from the stack
with no effect. (The engine's DSL-Condition intervening-if path checks only at
resolution — a pre-existing, accepted engine behavior shared by Acererak, Delver,
Legion's Landing; not in scope to change.)

**604.2** — conditional ("as long as") static abilities function only while their
condition is met; the condition is continuously evaluated.

## Engine Changes

### Change 1: Add the `Condition` variant

**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: Add a new unit variant `YouControlYourCommander` to `enum Condition`
(currently ends at line 3892, last existing variant `YouAttackedWithNOrMore(u32)`).
Doc-comment it: CR 903.3d — true when the effect controller controls (on the
battlefield, phased-in) at least one commander they own. Distinct from CR 118.9
"control a commander" (any owner).
**Pattern**: Follow the sibling unit variants (`SourceIsSolved`, `WasCast`).

### Change 2: Real evaluator arm in `check_condition`

**File**: `crates/engine/src/effects/mod.rs` (`fn check_condition`, match at line 8868)
**Action**: Add the arm (reads only `ctx.controller` + `state`, so it also works from
the static path — see Change 3):

```rust
// CR 903.3d + "your commander" (owned): true iff the controller controls, on the
// battlefield (phased-in), at least one commander card they OWN.
Condition::YouControlYourCommander => {
    match state.players.get(&ctx.controller) {
        Some(ps) => {
            let cmd_ids = &ps.commander_ids;
            state.objects.values().any(|obj| {
                obj.zone == ZoneId::Battlefield
                    && obj.is_phased_in()
                    && obj.controller == ctx.controller
                    && obj
                        .card_id
                        .as_ref()
                        .map(|cid| cmd_ids.contains(cid))
                        .unwrap_or(false)
            })
        }
        None => false,
    }
}
```

**CR**: 903.3d / 903.3. Mirror of the `controls_commander` closure at `casting.rs:2353`
but restricted to the controller's OWN `commander_ids` (that predicate scans ALL
players' `commander_ids`; do not copy it verbatim).

### Change 3: Continuous-grant (static) path — NO new arm required

**File**: `crates/engine/src/effects/mod.rs` (`fn check_static_condition`, line 9307)
**Action**: **None.** `check_static_condition` explicit-matches only the variants that
need a static-specific implementation and delegates everything else through its `_ =>`
fallback (line 9375), which builds a minimal `EffectContext` populated with
`controller` + `source` and calls `check_condition`. Since our arm reads only
`ctx.controller` + `state`, the fallback evaluates it correctly. The runner should add a
one-line comment at the `_ =>` fallback noting `YouControlYourCommander` is handled there
(no code change), and add a test that exercises the static path (Skyhunter) to prove it.

### Change 4: Hash arm (GameState hash closure)

**File**: `crates/engine/src/state/hash.rs` (`impl HashInto for Condition`, match at
line 5800; last discriminant is `50` at line 5974)
**Action**: Add `Condition::YouControlYourCommander => 51u8.hash_into(hasher),` with a
`// PB-OS9 / CR 903.3d (discriminant 51)` comment.

### Change 5: Wire fingerprint re-pin (machine-forced — see wire verdict below)

See the "Wire Impact" section for the definitive verdict and the exact re-pin steps.

### Exhaustive match sites for the new variant

`Condition` has exactly **two** exhaustive matches in the workspace (confirmed by
grepping for the `Condition::Always` / `Condition::SacrificeFired` arms — every
exhaustive match contains them). `check_static_condition` is a partial match with a
catch-all and does NOT count. No TUI / replay-viewer arm is needed (those match on
`StackObjectKind` / `KeywordAbility`, never `Condition`).

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/effects/mod.rs` | `check_condition` | 8868 | Add real arm (Change 2) |
| `crates/engine/src/state/hash.rs` | `HashInto for Condition` | 5800 | Add discriminant 51 (Change 4) |
| `crates/engine/src/effects/mod.rs` | `check_static_condition` | 9313 | none — `_ =>` fallback (Change 3) |

## Wire Impact — DEFINITIVE VERDICT (AC 5083)

**STOP-AND-FLAG: a wire bump IS forced. Take the single batched bump.**

- **`Condition` is inside the SR-8 PROTOCOL closure.** Confirmed from `protocol.rs`
  history: v5 note "`Condition` was already in the closure (reachable via
  `Effect::Conditional`)"; v21/PB-OS6 added two `Condition` variants and the machine
  gate forced PROTOCOL 20→21. Adding `YouControlYourCommander` moves the shape digest
  (type count unchanged; `Condition`'s declared shape moves).
  → **PROTOCOL_VERSION 23 → 24** (machine-forced by `tests/core/protocol_schema.rs`).
- **`Condition` is inside the GameState hash closure** (it has a `HashInto` impl and is
  reachable from `GameState.continuous_effects` / card registry). New variant +
  discriminant moves both the decl and stream digests.
  → **HASH_SCHEMA_VERSION 60 → 61** (machine-forced by `tests/core/hash_schema.rs`).

Current fingerprints read from source:
- `PROTOCOL_VERSION = 23` (`protocol.rs:220`);
  `PROTOCOL_SCHEMA_FINGERPRINT = "553f2ff2e54c7de707209b79db7f8bca0fc0c37405871a0c1b31c431e6dedb32"`
  (`protocol.rs:237`).
- `HASH_SCHEMA_VERSION = 60` (`hash.rs:543`); history tail is version 60.

### Re-pin steps (one commit, both axes)

PROTOCOL (in `crates/engine/src/rules/protocol.rs`):
1. Bump `PROTOCOL_VERSION` 23 → **24**; add a `- 24: PB-OS9 …` History line above line
   220 (note: `Condition` already in closure; declared shape moved).
2. Set `PROTOCOL_SCHEMA_FINGERPRINT` to the value printed by the failing
   `tests/core/protocol_schema.rs`.
3. **Append** a `ProtocolEpoch { version: 24, fingerprint: <same> }` row to
   `PROTOCOL_HISTORY` (never edit an existing row).
4. Update `protocol_version_sentinel` (`tests/core/protocol_schema.rs:872`, `23`→`24`)
   and the `FROZEN_HISTORY_PREFIX_DIGEST` / frozen-prefix pin per the failure text.

HASH (in `crates/engine/src/state/hash.rs`):
5. Bump `HASH_SCHEMA_VERSION` 60 → **61**; add a `- 61: PB-OS9 …` History line.
6. **Append** a `HashSchemaEpoch { version: 61, decl_fingerprint, stream_fingerprint }`
   row (both values from the failing `tests/core/hash_schema.rs` message).
7. Update the `hash_schema.rs:1194` gate sentinel (`60`→`61`).

Bulk sentinel update (both axes): global-replace every stale live sentinel:
- `HASH_SCHEMA_VERSION, 60` / `HASH_SCHEMA_VERSION, 60u8` → `61` (≈40 sites across
  `crates/engine/tests/**`; full list enumerated below — the grep in this plan captured
  them all).
- `PROTOCOL_VERSION, 23` → `24` (sites: `pb_ef12_any_color_choice.rs:363`,
  `pb_os7_…:699`, `pb_os6_…:874`, `pb_os5_…:716`, `pb_os8_…:1070`, `pb_ef7_…:242`,
  `pb_ef10_…:1595`, plus the gate at `protocol_schema.rs:872`).

The runner should add its own PB-OS9 sentinels (both `PROTOCOL_VERSION, 24` and
`HASH_SCHEMA_VERSION, 61u8`) in the new test file.

## Card Definition Fixes

### skyhunter_strike_force.rs  (partial → Complete)
**Oracle**: "Flying / Melee (…) / Lieutenant — As long as you control your commander,
other creatures you control have melee."
**Current**: Flying + printed Melee modeled; Lieutenant anthem omitted (blocked, see
top-of-file ENGINE-BLOCKED comment).
**Fix**: Add a third ability — `AbilityDefinition::Static(ContinuousEffectDef {
layer: <ability-adding layer, same layer used by other AddKeyword grants — verify
against a reference AddKeyword static>, modification:
LayerModification::AddKeyword(KeywordAbility::Melee), filter:
EffectFilter::OtherCreaturesYouControl, duration: <whichever WhileOnBattlefield-style
duration the existing AddKeyword statics use>, condition:
Some(Condition::YouControlYourCommander) })`. Post-PB-EF3b the granted Melee synthesizes
its attack trigger, so the anthem now actually fires. Remove the ENGINE-BLOCKED comment
and the OOS-EF3b-1 blocker note; set `Completeness::Complete`.

### loyal_apprentice.rs  (partial → Complete)
**Oracle**: "Haste / Lieutenant — At the beginning of combat on your turn, if you
control your commander, create a 1/1 colorless Thopter artifact creature token with
flying. That token gains haste until end of turn."
**Current**: only `Keyword(Haste)`; Lieutenant TODO.
**Fix**: Add `AbilityDefinition::Triggered { trigger_condition:
TriggerCondition::AtBeginningOfCombat, intervening_if:
Some(Condition::YouControlYourCommander), effect: <CreateToken of one 1/1 colorless
Thopter artifact-creature token with flying>, … }`. `AtBeginningOfCombat` is already
"on your turn" (DSL doc, card_definition.rs:3348). Token flying via `TokenSpec.keywords`.
Haste-until-EOT: prefer a true UEOT grant on the created token if a
`LastCreatedPermanent`-style reference is available (loyal_apprentice's own note cites
`EffectTarget::LastCreatedPermanent`); a permanent-haste `TokenSpec.keywords` entry is a
functionally-equivalent fallback (a token's haste is unobservable after the turn it is
created — it loses summoning sickness anyway). Set `Completeness::Complete`.

### siege_gang_lieutenant.rs  (partial → Complete)
**Oracle**: "Lieutenant — At the beginning of combat on your turn, if you control your
commander, create two 1/1 red Goblin creature tokens. Those tokens gain haste until end
of turn. / {2}, Sacrifice a Goblin: This creature deals 1 damage to any target."
**Current**: the `{2}, Sacrifice a Goblin` activated ability is modeled and correct; the
Lieutenant trigger is a TODO.
**Fix**: Prepend `AbilityDefinition::Triggered { trigger_condition:
AtBeginningOfCombat, intervening_if: Some(Condition::YouControlYourCommander), effect:
<CreateToken count 2, 1/1 red Goblin creature tokens, each gains haste UEOT> }`. Keep the
existing activated ability unchanged. Same token-haste guidance as loyal_apprentice
(here `TokenSpec.count = 2`). Set `Completeness::Complete`.

### legion_lieutenant.rs — OUT OF SCOPE (confirmed via MCP)
Oracle: "Other Vampires you control get +1/+1." No Lieutenant ability, no commander
condition — "Lieutenant" appears only in the card's NAME. Do not touch. Record in the
close-out: "legion_lieutenant confirmed name-only, OUT."

## New Card Definitions

None. Roster is closed at the 3 existing partials above.

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_os9_lieutenant_commander_control.rs`
(register its `mod` line in the primitives group per SR-9a; never add a top-level
`tests/*.rs`).

**Tests to write** (every decoy must be non-vacuous — assert the OFF branch would
otherwise fire):

- `test_you_control_your_commander_true_on_battlefield` — controller's own commander is
  on the battlefield under their control → `check_condition` returns true (CR 903.3d).
- `test_you_control_your_commander_false_in_command_zone` — commander in the command zone
  (not on battlefield) → false. (Non-vacuous: put an ordinary creature the controller
  controls on the battlefield so the object scan is non-empty.)
- `test_you_control_your_commander_drops_when_commander_dies` — commander on battlefield,
  condition true; destroy it (SBA batch) → same-batch recompute → false. Assert the
  drop occurs in the same SBA pass, not a later one.
- `test_stolen_commander_decoy_lieutenant_off` — **STOLEN commander decoy**: p1's
  commander is on the battlefield but `controller == p2` (p2 stole it) → for p1's
  ctx.controller the condition is **false** (Lieutenant OFF); for p2 the condition is
  also false because that card is in p1's `commander_ids`, not p2's. Assert BOTH. Then
  return control to p1 → true (stole-back). (Non-vacuous: verify it would be true if p1
  controlled it.)
- `test_control_opponents_commander_only_still_off` — p1 controls p2's commander but NOT
  p1's own commander → false ("your commander", CR 903.3d + owned). Distinct from the
  CR 118.9 free-cast predicate which WOULD be satisfied here.
- `test_multiple_commanders_control_one_suffices` — partner pair; controlling one of two
  owned commanders → true (ruling: "you need to control only one").
- `test_lieutenant_intervening_if_at_resolution_siege_gang` — integration: cast/enter
  `siege_gang_lieutenant`, control your commander, begin combat → trigger resolves →
  two Goblin tokens created; then a variant where the commander is removed in response
  before resolution → trigger resolves with NO tokens (CR 603.4 resolution-time check).
- `test_skyhunter_continuous_grant_active_and_drops` — integration (static path via
  `check_static_condition` fallback): `skyhunter_strike_force` on the battlefield, a
  second creature you control, and your commander on the battlefield → the second
  creature has Melee (assert via calculated characteristics AND that its Melee attack
  trigger fires post-PB-EF3b). Then move the commander off the battlefield → recompute →
  the second creature no longer has Melee (grant drops). **STOLEN-commander sub-case**:
  opponent gains control of your commander → grant drops.
- `test_loyal_apprentice_thopter_token_with_haste` — integration: control your commander,
  begin combat → one 1/1 colorless flying Thopter token created; assert it can attack
  this turn (haste) and is an artifact creature.
- Version sentinels in this file: `assert_eq!(PROTOCOL_VERSION, 24)` and
  `assert_eq!(HASH_SCHEMA_VERSION, 61u8)`.

**Pattern**: Follow `pb_os6_dfc_flip_conditions.rs` (Condition-variant + intervening-if
tests, incl. its `PROTOCOL_VERSION`/`HASH_SCHEMA_VERSION` sentinels) and
`pb_os7_defending_player_continuous_filter.rs` (continuous-effect condition drop-on-
change tests). For commander-control fixture setup, mirror
`tests/mechanics_a_d/domain_and_freecast.rs` (it already builds "p1 controls a commander
on the battlefield" and the stolen-commander cross-control scenario at lines 800-853).

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] `Condition::YouControlYourCommander` added; `check_condition` arm added; hash
      discriminant 51 added
- [ ] `check_static_condition` fallback verified to route the variant (no arm, comment
      only) — proven by the Skyhunter static test
- [ ] PROTOCOL 23→24 re-pinned (version + fingerprint + history row + sentinel + frozen
      prefix); HASH 60→61 re-pinned (version + history row with both fingerprints +
      gate sentinel); all ~40 `HASH_SCHEMA_VERSION` and all `PROTOCOL_VERSION` live
      sentinels bumped
- [ ] 3 card defs flipped to `Complete` (skyhunter, loyal_apprentice, siege_gang); no
      remaining TODO/ENGINE-BLOCKED text; `legion_lieutenant` untouched (documented OUT)
- [ ] New test file registered via `mod` (SR-9a); all decoys non-vacuous
- [ ] `cargo test --all` green (incl. `core/protocol_schema`, `core/hash_schema`,
      `card_defs_fmt`)
- [ ] `cargo clippy -- -D warnings` clean; `cargo fmt --check` + `tools/check-defs-fmt.sh`
- [ ] `cargo build --workspace` (seal + exhaustive-match gate)

## Risks & Edge Cases

- **Wire bump is unavoidable and multi-file.** The ~40 `HASH_SCHEMA_VERSION` sentinels
  are the #1 churn source; a missed one reddens the suite. Do the global replace, then
  re-pin the two gate fingerprints from the failing test output (don't hand-compute).
- **"your commander" vs "a commander" divergence.** The single biggest correctness trap:
  do NOT reuse `casting.rs`'s CommanderFreeCast predicate (any owner). Lieutenant is
  strictly owned-by-controller. The `test_control_opponents_commander_only_still_off`
  decoy exists to catch this.
- **PB-EF3b dependency.** Skyhunter's granted Melee only fires because PB-EF3b synthesizes
  keyword-derived triggers for continuous-effect-added keywords. If that synthesis has a
  gap for Melee-via-static-with-condition, the Skyhunter trigger test will expose it —
  treat a failure as a PB-EF3b interaction to file (OOS-EF3b-2 already tracks extending
  the helper), not a reason to author Skyhunter wrong.
- **Resolution-only intervening-if check.** The DSL Condition intervening-if is checked
  at resolution, not at trigger-enqueue (existing engine behavior; the enqueue-time
  `check_intervening_if` uses the separate 2-variant `InterveningIf` enum and is not
  touched). This is CR-defensible (the ruling emphasizes the resolution check) and
  consistent with Acererak/Delver/Legion's Landing. Do NOT try to also wire it into the
  enqueue path — that is scope creep and would require converting `InterveningIf`.
- **Re-entrancy of `check_static_condition`.** The Skyhunter path evaluates the condition
  during `calculate_characteristics`; the arm only scans OTHER objects' zone/controller/
  card_id (no characteristics recompute of the object under calculation), so it is
  re-entrant-safe (same guarantee documented for `YouControlNOrMoreWithFilter`).
- **Token haste truthfulness.** Prefer a real UEOT grant on the created token; the
  permanent-haste `TokenSpec.keywords` fallback is functionally equivalent (unobservable
  difference) and acceptable under the no-wrong-game-state guardrail, but note the choice
  in each card's completeness rationale.

## Discounted ship

**~3 clean flips** (skyhunter_strike_force, loyal_apprentice, siege_gang_lieutenant),
slightly above the queue's ~1-2 estimate because all three corpus Lieutenant-ability
cards are unblocked by this one primitive and their token/anthem halves were already
expressible. `legion_lieutenant` is name-only and OUT.
