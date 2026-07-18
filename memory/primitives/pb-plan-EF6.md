# Primitive Batch Plan: PB-EF6 — `TargetRequirement::TargetOpponent`

**Generated**: 2026-07-18
**Primitive**: New unit variant `TargetRequirement::TargetOpponent` — an opponent-restricted
player target ("target opponent …"). Threads the source's controller (`caster` / `trigger.controller`)
into player-target validation and into the trigger auto-target picker so an opponent-only clause
can be authored without permitting an illegal self-target.
**CR Rules**: CR 102.4 (opponent definition), CR 115.1 / 115.3 (targeting), CR 601.2c (target
announcement / illegal-target rejection), CR 603.3d (trigger removed if no legal target)
**Cards affected**: 8 touched — **3 clean flips to Complete** (shaman_of_the_pack, raiders_wake,
vengeful_bloodwitch), **1 correctness fix staying Complete** (fell_specter), **3 minimal honest
target-fixes staying non-Complete** (blood_tribute, blessed_alliance, forbidden_orchard), **1 minimal
emblem target-fix staying known_wrong** (ajani_sleeper_agent). 1 explicitly left untouched
(flare_of_malice — wrong-oracle, full re-author needed).
**Dependencies**: none (PB-AC6 `Condition::YouAttackedThisTurn` and `TriggerCondition::AtBeginningOfYourEndStep`
already shipped; PB-EF2 `TokenSpec.recipient` already shipped — relevant only to the optional
forbidden_orchard token-recipient tidy).
**Deferred items from prior PBs**: closes EF-W-PB2-2. Does NOT close EF-W-PB2-3 (`AddManaAnyColor`,
forbidden_orchard's surviving blocker).

---

## MANDATORY roster-recall / TODO sweep result (run before finalizing roster)

Sweep 1 — `TargetOpponent` named in a def comment (`grep TargetOpponent crates/card-defs/src/defs/`):
forbidden_orchard, shaman_of_the_pack, raiders_wake, ajani_sleeper_agent. (All 4 coordinator-named.)

Sweep 2 — oracle "target opponent" modeled with the wrong `TargetRequirement::TargetPlayer`
(`grep -il "target opponent"` ∩ uses `TargetRequirement::TargetPlayer`): **5 files** —
fell_specter, blessed_alliance, blood_tribute, flare_of_malice, vengeful_bloodwitch.

**Roster-recall finding (NOT in the coordinator brief — forced adds):**
- **vengeful_bloodwitch.rs** (`known_wrong`) — its marker literally states the ONLY blocker is
  "TargetRequirement has no opponent-only player variant, so the controller is an
  illegal-but-accepted target (CR 115.1)." Death trigger `WheneverCreatureDies{controller:You}`,
  `LoseLife{DeclaredTarget 0}` + `GainLife{Controller}` — all expressible. **CLEAN FLIP →
  Complete.** This is a 3rd coverage-mover the brief's 2-flip estimate missed.
- **fell_specter.rs** (currently **Complete**) — ships `TargetRequirement::TargetPlayer` for oracle
  "target opponent discards a card." Latent legal-but-wrong self-target bug on a shipped-Complete
  def. **Forced correctness fix** (TargetPlayer → TargetOpponent); stays Complete, no coverage
  movement, but removes a real KI-1 wrong-target.

Both are added to the Card Definition Fixes section with the note *"added via pre-existing
TODO/oracle sweep — not in original PB brief."* The other 14 "target opponent" oracle files model
the clause without a player `TargetRequirement` (each-opponent / reminder text) and are unaffected
(grep-verified: only the 5 above use `TargetRequirement::TargetPlayer`).

---

## Confirmed: NO teams model exists (DECISION 2 premise verified)

- `PlayerState` (`crates/card-types/src/state/player.rs:281`) has **no** `team` / `teammate` /
  `team_id` field (full struct inspected).
- No `team` / `teammate` symbol anywhere in `crates/card-types/src/state/` except an unrelated
  `ControllerRestriction::Opponent` enum (used by `EnchantFilter`) and keyword doc-comments.
- The engine-wide idiom for "opponent" is `p != controller` (+ `!has_lost`), used identically at
  `effects/mod.rs:3769` (`AnOpponent`), `:6327` (`EffectTarget::EachOpponent`), `:6507`
  (`PlayerTarget::EachOpponent`), and the trigger auto-picker `abilities.rs:6885`.

**Definitive answer: opponent of `caster` = `id != caster`.** Commander is free-for-all; every
non-controller player is an opponent. The new validation arm and auto-picker arm both use this,
mirroring the existing `EachOpponent` idiom. If a teams model is ever added, these two sites (and
the existing `EachOpponent` resolvers) all change together.

---

## CR Rule Text (from MCP lookup — authoritative)

- **CR 102.2** In a two-player game, a player's opponent is the other player.
- **CR 102.3** In a multiplayer game between teams, a player's teammates are the other players on
  their team, and the player's opponents are all players not on their team. *(No teams here → every
  non-controller is an opponent.)*
- **CR 115.1** Some spells and abilities require their controller to choose one or more targets…
  These targets are declared as part of the process of putting the spell or ability on the stack.
- **CR 115.3** The same target can't be chosen multiple times for any one instance of "target".
- **CR 601.2c** The player announces their choice of an appropriate object or player for each
  target the spell requires… The chosen objects and/or players each become a target. *(Opponent-ness
  is a declaration-time restriction — an illegal target is rejected here.)*
- **CR 603.3d** The remainder of the process for putting a triggered ability on the stack is
  identical to 601.2c–d. **If a choice is required when the triggered ability goes on the stack but
  no legal choices can be made for it… the ability is simply removed from the stack.** *(→ auto-picker
  must contribute NO candidate and skip the trigger when the source has no opponent — never fall back
  to self.)*
- **CR 608.2b** (resolution re-check) A target that's no longer legal is illegal. *(Opponent-ness
  cannot change — no team changes — so no resolution-time re-check is needed; DECISION 4.)*

---

## Engine Changes (step-ordered)

### Step 1 — Add the enum variant (DSL)

**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: Add a unit variant `TargetOpponent` to `enum TargetRequirement`, immediately after the
`UpToN { … }` variant (currently ends at **line 2875**, enum closes at **2876**). Discriminant-agnostic
(hash arm is explicit in Step 2). Doc-comment MUST cite CR 102.4 / 115.1 / 601.2c / 603.3d:

```
/// "target opponent" — a player who is an opponent of the source's controller
/// (CR 102.3/102.4: opponent = any player not on your team; no teams model exists,
/// so opponent = any player other than the controller). Validated at declaration
/// time (CR 601.2c) — a self-target is illegal. For triggered abilities the auto-
/// target picker selects the first active opponent, or removes the trigger if the
/// source has no opponent (CR 603.3d). Discriminant 18 (state/hash.rs).
TargetOpponent,
```

### Step 2 — Hash discriminant (COMPILE-ERROR gate — exhaustive match, no `_`)

**File**: `crates/engine/src/state/hash.rs`
**Action**: In `impl HashInto for TargetRequirement` (match at **5017**), add a new arm after the
`UpToN` arm (**line 5058**), before the closing brace at **5059**:

```
// PB-EF6: TargetOpponent -- CR 102.3 / 601.2c (discriminant 18)
TargetRequirement::TargetOpponent => 18u8.hash_into(hasher),
```

This match is exhaustive (no wildcard) → omitting the arm is a compile error. Discriminant **18**
(current max UpToN = 17).

### Step 3 — Player-target validation: thread `caster`, add the restriction (CR 601.2c)

**File**: `crates/engine/src/rules/casting.rs`
**Action A — signature**: `fn validate_player_satisfies_requirement` (**line 6074**) gains a
`caster: PlayerId` parameter.
**Action B — arm**: in its `match req` (**6078**), add before the `_ =>` catch-all (**6085**):

```
// CR 102.3 / 115.1 / 601.2c: "target opponent" — the caster may not choose themselves.
TargetRequirement::TargetOpponent => {
    if id != caster {
        Ok(())
    } else {
        Err(GameStateError::InvalidTarget(format!(
            "player {:?} cannot be the target of 'target opponent' (self, CR 102.3/601.2c)",
            id
        )))
    }
}
```

**Action C — UpToN delegation** (**line 6084**): pass `caster` through:
`TargetRequirement::UpToN { inner, .. } => validate_player_satisfies_requirement(id, inner, caster),`

**Action D — call sites** (both already have `caster` in scope):
- **line 5829** inside the `target_satisfies` closure: `validate_player_satisfies_requirement(*id, req, caster).is_ok()` (`caster` is captured — used at 5831).
- **line 6016**: `validate_player_satisfies_requirement(*id, r, caster)?;` (`caster` in scope — used at 5994/6049).

### Step 4 — Object-target validation: reject player-req on object (COMPILE-ERROR gate)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: `validate_object_satisfies_requirement`'s `let valid = match req` (**6174**) is
exhaustive (no `_`). Add, next to the existing `TargetRequirement::TargetPlayer => false` at
**6351**:

```
// Player requirement — object target is illegal (CR 601.2c).
TargetRequirement::TargetPlayer | TargetRequirement::TargetOpponent => false,
```

(Combine with the existing TargetPlayer arm, or add a sibling arm.)

### Step 5 — Trigger auto-target pickers: own arm, NO self-fallback (CR 603.3d) — CORRECTNESS, not compile

**File**: `crates/engine/src/rules/abilities.rs`
Both matches have a `_` catch-all that routes to a **battlefield object scan** — so a missing arm
silently mis-targets rather than failing to compile. Explicit arms are load-bearing.

**Action A — outer picker** (`match req`, **6873**; player family at **6876–6903**, `_` battlefield
scan at **7017**): add a dedicated arm (place beside the TargetPlayer family, before `_`):

```
// PB-EF6: CR 603.3d — pick the first active opponent; if the source has NO opponent,
// contribute no candidate (None) so the trigger is removed from the stack. NEVER fall
// back to trigger.controller (that would be an illegal self-target).
TargetRequirement::TargetOpponent => state
    .turn
    .turn_order
    .iter()
    .find(|&&p| {
        p != trigger.controller
            && state
                .expect_player(p)
                .map(|pl| !pl.has_lost && !pl.has_conceded)
                .unwrap_or(false)
    })
    .copied()
    .map(|p| SpellTarget { target: Target::Player(p), zone_at_cast: None }),
```

Note this is the SAME opponent-finding expression as the existing TargetPlayer family (**6881–6892**)
but WITHOUT the `.or_else(|| … trigger.controller)` self-fallback (**6893–6898**).

**Action B — UpToN-inner picker** (**6982**; player-inner family at **6983–6986**, `_ => None` at
**7013**): add a `TargetRequirement::TargetOpponent` arm inside the inner match, mirroring the
player-inner picker (**6988–7009**) but WITHOUT the `.or_else` self-fallback — returns the mapped
opponent `SpellTarget`, or `None` (UpToN contributes 0 targets if no opponent — correct for optional).

### Step 6 — Resolution re-check: NO change (DECISION 4 — confirmed)

**File**: `crates/engine/src/rules/resolution.rs` — `is_target_legal` (**7783**) re-checks only that
a `Target::Player` is `!has_lost && !has_conceded`. Opponent-ness is a CR 601.2c/115.1
declaration-time restriction; it cannot change at resolution (no team changes exist), and a departed
opponent is already caught by the `has_lost`/`has_conceded` check. **Do not modify.** (Confirmed by
reading the fn — it stores no caster and no requirement, by design.)

### Step 7 — Exhaustive-match / requirement-listing sweep (`cargo build --workspace` is the gate)

Verified full list of sites that match/enumerate `TargetRequirement` variants:

| File | Match / site | Line | Action | Kind |
|------|--------------|------|--------|------|
| `crates/engine/src/state/hash.rs` | `impl HashInto` | 5017 | add discriminant 18 | **compile-error** if missed |
| `crates/engine/src/rules/casting.rs` | `validate_object_satisfies_requirement` `valid` match | 6174 | add `=> false` | **compile-error** if missed |
| `crates/engine/src/rules/casting.rs` | `validate_player_satisfies_requirement` | 6078 | add restriction arm + `caster` param | has `_`; MUST add (else self rejected wrongly) |
| `crates/engine/src/rules/abilities.rs` | outer auto-picker | 6873 | add opponent arm | has `_`; MUST add (else mis-scans battlefield) |
| `crates/engine/src/rules/abilities.rs` | UpToN-inner picker | 6982 | add opponent arm | has `_ => None`; MUST add |

**Confirmed NO change needed / NOT present:**
- **simulator** (`crates/simulator/`) — `grep TargetRequirement` → **0 matches**. `LegalActionProvider`
  does NOT enumerate player targets by `TargetRequirement`, so bots cannot offer an illegal
  self-target for TargetOpponent (no SG-1-class hazard). Confirmed.
- **TUI** (`tools/tui/`) and **replay-viewer** (`tools/replay-viewer/`) — `grep TargetRequirement` →
  **0 matches**. Those view-models match on `StackObjectKind`/`KeywordAbility`, never on
  `TargetRequirement`. No display arm needed.
- **replay harness** `translate_player_action` / `resolve_targets` — resolves explicit
  `Target::Player` from the script JSON; it does not map a `TargetRequirement` to a player. The
  requirement is validated downstream by `validate_targets` (Step 3), so **no harness change**
  (DECISION 5). A script that self-targets a TargetOpponent spell will now be rejected by the same
  validation the engine uses — correct.

### Step 8 — Wire bump (machine-forced; re-pin from FAILING gate output, never hand-guess)

`TargetRequirement` is reachable from `Characteristics → …abilities → AbilityDefinition.targets`
(in the GameState hash closure) **and** from the card DSL in the SR-8 protocol closure. Adding a
variant moves both digests.

**Expected: PROTOCOL 10 → 11, HASH 48 → 49** (both machine-forced by `tests/core/protocol_schema.rs`
and `tests/core/hash_schema.rs` + the `HASH_SCHEMA_VERSION` sentinels).

Procedure (run the impl first, THEN the gates emit the new digests):
1. **PROTOCOL** — `crates/engine/src/rules/protocol.rs`:
   - bump `PROTOCOL_VERSION` `10 → 11` (**line 118**);
   - add a `/// - 11: PB-EF6 (2026-07-18) — TargetRequirement gains unit variant TargetOpponent …`
     History line above 118 (after the `- 10:` block at 113–117);
   - set `PROTOCOL_SCHEMA_FINGERPRINT` (**line 135**) to the recomputed digest from the
     `protocol_schema.rs` failure text;
   - **append** a `ProtocolEpoch { version: 11, fingerprint: "<new>" }` row to `PROTOCOL_HISTORY`
     (**begins line 188** — never edit an existing row);
   - update the `protocol_version_sentinel` and the FROZEN prefix digest in `tests/core/protocol_schema.rs`.
2. **HASH** — `crates/engine/src/state/hash.rs`:
   - bump `HASH_SCHEMA_VERSION` `48 → 49` (**line 435**);
   - add its `- 49:` History line;
   - **append** a `HashSchemaEpoch { version: 49, decl_fingerprint, stream_fingerprint }` row to
     `HASH_SCHEMA_HISTORY` (read BOTH digests from the `hash_schema.rs` failure text);
   - bump the scattered `assert_eq!(HASH_SCHEMA_VERSION, 48)` sentinels (grep the suite; e.g.
     `pbt_up_to_n_targets.rs` asserts the current value) to 49.

If EITHER gate unexpectedly does NOT move, stop and investigate — DECISION 6 expects both; a
non-moving PROTOCOL digest would mean TargetRequirement is not in the protocol closure (contradicts
the recorded closure shape) and must be reconciled before shipping.

---

## Card Definition Fixes

### shaman_of_the_pack.rs  (`inert` → **Complete** — CLEAN FLIP)
**Oracle**: "When this creature enters, target opponent loses life equal to the number of Elves you
control."
**Current**: `abilities: vec![]`, `Completeness::inert(...)` citing EF-W-PB2-2.
**Fix**: author the ETB trigger (pattern: fell_specter.rs ETB), drop the inert marker (→ default
Complete), remove the stale header comment:
```
AbilityDefinition::Triggered {
    once_per_turn: false,
    trigger_condition: TriggerCondition::WhenEntersBattlefield,
    effect: Effect::LoseLife {
        player: PlayerTarget::DeclaredTarget { index: 0 },
        amount: EffectAmount::PermanentCount {
            filter: TargetFilter {
                has_subtype: Some(SubType("Elf".to_string())),
                controller: TargetController::You,
                ..Default::default()
            },
            controller: PlayerTarget::Controller,
        },
    },
    intervening_if: None,
    targets: vec![TargetRequirement::TargetOpponent],
    modes: None,
    trigger_zone: None,
}
```
Verified shapes: `TriggerCondition::WhenEntersBattlefield` (fell_specter), `Effect::LoseLife{player,amount}`,
`EffectAmount::PermanentCount{filter: TargetFilter, controller: PlayerTarget}` (card_definition.rs:2576;
eomer_king_of_rohan.rs:41). Count is subtype-only ("Elves you control" = any Elf permanent; do NOT add
`has_card_type: Creature` — oracle counts Elves, not Elf creatures).

### raiders_wake.rs  (`partial` → **Complete** — CLEAN FLIP)
**Oracle**: "Whenever an opponent discards a card, that player loses 2 life.\nRaid — At the beginning
of your end step, if you attacked this turn, target opponent discards a card."
**Current**: first ability authored; Raid half omitted; `Completeness::partial(...)`.
**Fix**: add the Raid ability after the existing trigger; drop the partial marker (→ Complete);
remove the ENGINE-BLOCKED header/inline comments:
```
AbilityDefinition::Triggered {
    once_per_turn: false,
    trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
    effect: Effect::DiscardCards {
        player: PlayerTarget::DeclaredTarget { index: 0 },
        count: EffectAmount::Fixed(1),
    },
    intervening_if: Some(Condition::YouAttackedThisTurn),
    targets: vec![TargetRequirement::TargetOpponent],
    modes: None,
    trigger_zone: None,
}
```
Verified: `TriggerCondition::AtBeginningOfYourEndStep` ✓, `Condition::YouAttackedThisTurn` ✓ (PB-AC6),
`Effect::DiscardCards{player: DeclaredTarget, count}` ✓ (fell_specter ETB precedent). Runner: confirm
the exact `intervening_if` variant name (`Condition::YouAttackedThisTurn`) against a PB-AC6 card that
uses it; adjust if the field spelling differs.

### vengeful_bloodwitch.rs  (`known_wrong` → **Complete** — CLEAN FLIP, roster-recall add — not in original brief)
**Oracle**: "Whenever Vengeful Bloodwitch or another creature you control dies, target opponent loses
1 life and you gain 1 life."
**Current**: authored with `targets: vec![TargetRequirement::TargetPlayer]`; `Completeness::known_wrong(...)`
whose sole cited blocker is the missing opponent variant.
**Fix**: change `targets` (**line 45**) `TargetPlayer` → `TargetOpponent`; delete the known_wrong
marker (→ Complete); update the inline comment at line 25 ("approximated as DeclaredTarget" — the
approximation is now exact). No other change; the death-trigger auto-picker (Step 5A) supplies the
opponent.

### fell_specter.rs  (stays **Complete** — correctness fix, roster-recall add — not in original brief)
**Oracle**: "Flying\nWhen this creature enters, target opponent discards a card.\nWhenever an opponent
discards a card, that player loses 2 life."
**Current**: **Complete**, but ETB uses `targets: vec![TargetRequirement::TargetPlayer]` (**line 33**)
for "target opponent" — a latent legal-but-wrong self-target.
**Fix**: change ETB `targets` `TargetPlayer` → `TargetOpponent`. Stays Complete (no coverage
movement); removes the KI-1 wrong-target. The second trigger (`WheneverOpponentDiscards`, no targets)
is unchanged.

### blood_tribute.rs  (stays `partial` — minimal honest target fix)
**Oracle**: "…Target opponent loses half their life, rounded up. If this spell was kicked, you gain
life equal to the life lost this way."
**Current**: `partial` (effect is `Effect::Nothing`); real blocker `EffectAmount::HalfLife`;
`targets: vec![TargetRequirement::TargetPlayer]` (**line 29**).
**Fix**: change `targets` `TargetPlayer` → `TargetOpponent` (correct the target restriction now that
it exists). **Keep `Completeness::partial`** — surviving blocker is `EffectAmount::HalfLife`
(+ non-mana Kicker cost + "if kicked" conditional). Update the partial-marker note to keep the
HalfLife blocker as the reason (it is not TargetOpponent).

### blessed_alliance.rs  (stays `partial` — minimal honest target fix, LOW priority)
**Oracle mode 2**: "Target opponent sacrifices an attacking creature of their choice." (mode 0 is
"Target player gains 4 life" — genuinely any player, leave as `TargetPlayer`.)
**Current**: `partial` (blocked on the Escalate + `mode_targets` design conflict); the flat targets
list uses `TargetPlayer` at index 3 (**line 58**) for the mode-2 opponent.
**Fix**: change index-3 requirement (**line 58**) `TargetPlayer` → `TargetOpponent`; **keep index-0
as `TargetPlayer`** (mode 0 targets any player). **Keep `Completeness::partial`** — surviving blocker
is the Escalate/`mode_targets` incompatibility (unchanged). This is a correctness tidy only; if the
runner judges it risks confusing the flat-target approximation, it may defer — but the honest change
is index-3 → TargetOpponent.

### forbidden_orchard.rs  (stays `known_wrong` — recommend MINIMAL change; surviving blocker EF-W-PB2-3)
**Oracle**: "{T}: Add one mana of any color.\nWhenever you tap this land for mana, target opponent
creates a 1/1 colorless Spirit creature token."
**Current**: `known_wrong`; two defects — (a) token minted for the CONTROLLER not the target opponent
(inverts the drawback), and (b) `Effect::AddManaAnyColor` stub adds Colorless (EF-W-PB2-3, STILL
OPEN → barred from Complete).
**Recommended minimal honest change**: fix BOTH target-side defects now (they are real wrong-state):
1. `targets` (**line 62**) `TargetPlayer` → `TargetOpponent`;
2. route the token to the target opponent via PB-EF2 `TokenSpec.recipient: PlayerTarget::DeclaredTarget { index: 0 }`
   (removing the "Spirit for controller" inversion).
**Keep `Completeness::known_wrong`**, but rewrite the marker so the SOLE surviving blocker is
EF-W-PB2-3 (`Effect::AddManaAnyColor` adds Colorless, not any color) — no longer the token recipient
or the target. Rationale: this converts a two-defect known_wrong into a one-defect known_wrong with
an accurate marker, and exercises the new variant in the corpus. Runner: verify `TokenSpec.recipient`
exists and defaults to `Controller` (PB-EF2, shipped `scutemob-102`) before wiring it; if the
recipient field's spelling differs, correct only the `targets` line and leave the marker citing both
the recipient gap and AddManaAnyColor.

### ajani_sleeper_agent.rs  (stays `known_wrong` — minimal emblem target fix only)
**Oracle -6 emblem**: "Whenever you cast a creature or planeswalker spell, target opponent gets two
poison counters."
**Current**: `known_wrong` with MANY blockers (+1/-3 are `Sequence(vec![])` no-ops; emblem lacks the
creature/pw spell-type filter; fires on AnySpellCast; Compleated unimplemented).
**Fix (minimal)**: in the emblem's `TriggeredAbilityDef`, change `targets` (**line 87**) `TargetPlayer`
→ `TargetOpponent`; remove the two "TODO: Should be TargetOpponent" comments (**64, 86**). **Keep
`Completeness::known_wrong`** and keep the marker naming all the surviving blockers (drop only the
"targets any player rather than an opponent" clause from the marker text, since that is now fixed).

### flare_of_malice.rs  (LEAVE UNTOUCHED)
Marker documents the def is authored against oracle text this card does not have (real card is
"Each opponent sacrifices … greatest mana value"). Not a single-"target opponent" card at all; needs
a full re-author (greatest-MV selection + sac-a-nontoken-black-creature alt cost). Out of scope for
PB-EF6 — do not change its `TargetPlayer`.

---

## New Card Definitions

None. All 8 touched cards already exist.

---

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_ef6_target_opponent.rs` (new).
**Register**: add `mod pb_ef6_target_opponent;` to `crates/engine/tests/primitives/main.rs` after
line 26 (keep alphabetical-ish grouping with the other `pb_ef*` mods).

Module doc-comment cites CR 102.3 / 115.1 / 601.2c / 603.3d and notes HASH 48→49, PROTOCOL 10→11.
Pattern: follow `pbt_up_to_n_targets.rs` (GameStateBuilder + `process_command` + `Command::CastSpell`
/ `CastSpellData`, `Target::Player`).

Tests:
- `test_target_opponent_spell_accepts_opponent_rejects_self_4p` — **the required 4-player test.**
  Build a 4-player game; a test-only spell CardDefinition with
  `AbilityDefinition::Spell { targets: vec![TargetRequirement::TargetOpponent], … }` in
  caster P1's hand. Assert: `CastSpell` targeting P2 (opponent) → `Ok` and the spell hits the stack;
  `CastSpell` targeting P1 (self) → `Err(GameStateError::InvalidTarget)`; also assert targeting P3/P4
  (other opponents) → `Ok`. CR 601.2c. Exercises the Step 3 validation path directly.
- `test_target_opponent_decoy_self_must_be_rejected` — **the decoy that fails on EXACTLY the
  opponent-ness field.** Identical to above but asserts ONLY that the self-target `CastSpell` is
  rejected. This test PASSES if and only if TargetOpponent restricts to non-caster; it FAILS if the
  arm is implemented as `Ok(())`-for-everyone (i.e. accidentally aliased to TargetPlayer). Comment the
  decoy intent explicitly so a future edit can't silently weaken it.
- `test_target_opponent_hashes_distinctly` — assert `HASH_SCHEMA_VERSION == 49` and that a
  `TargetRequirement::TargetOpponent` requirement hashes differently from `TargetPlayer` and `UpToN`
  (mirror the PB-T hash-distinctness test).
- `test_shaman_of_the_pack_etb_targets_opponent_loses_life` — **non-vacuous integration.** 2+ player
  game, P1 controls N other Elves; resolve shaman's ETB; assert the auto-picked target is an OPPONENT
  (not P1) and that opponent's life dropped by exactly N (0-Elf and N-Elf cases). CR 603.3d + the
  PermanentCount count. Proves the flip produces correct game state.
- `test_raiders_wake_raid_targets_opponent_discards` — end-step trigger with `YouAttackedThisTurn`
  true: assert an opponent (not P1) discards exactly 1; and with attacked=false the trigger does not
  fire (intervening-if). Non-vacuous.
- `test_vengeful_bloodwitch_death_trigger_targets_opponent` — a creature P1 controls dies; assert an
  opponent (not P1) loses 1 and P1 gains 1. Proves known_wrong→Complete is correct, not vacuous.
- `test_fell_specter_etb_no_longer_self_targetable` — assert the ETB auto-picker selects an opponent
  (regression pin for the corrected latent bug).
- `test_target_opponent_trigger_no_opponent_removed_from_stack` — a contrived 1-opponent game where
  the only opponent has `has_lost = true`: assert the TargetOpponent trigger contributes no target and
  is removed from the stack (CR 603.3d), and does NOT fall back to targeting the controller (DECISION
  3). This is the decoy for the auto-picker's no-self-fallback.

---

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check -p mtg-card-types && cargo check -p mtg-engine`)
- [ ] `TargetOpponent` added to enum with CR-citing doc-comment
- [ ] Hash discriminant 18 added (exhaustive match)
- [ ] `validate_player_satisfies_requirement` threads `caster`; both call sites updated; UpToN delegates `caster`
- [ ] Object-side `valid` match rejects `TargetOpponent`
- [ ] Both auto-target pickers have an opponent arm with NO self-fallback
- [ ] `is_target_legal` UNCHANGED (DECISION 4)
- [ ] 3 clean flips → Complete (shaman, raiders_wake, vengeful_bloodwitch); markers removed
- [ ] fell_specter corrected (TargetPlayer→TargetOpponent), stays Complete
- [ ] blood_tribute / blessed_alliance(idx3) / forbidden_orchard / ajani target-fixed, markers rewritten to surviving (non-TargetOpponent) blockers
- [ ] flare_of_malice untouched
- [ ] No remaining `TargetOpponent`-blocker TODO comments in the 8 touched defs
- [ ] PROTOCOL 10→11 + HASH 48→49, both re-pinned from FAILING gate output; history rows appended; sentinels bumped
- [ ] New test module registered in `primitives/main.rs`; all new tests pass (`cargo test -p mtg-engine --test primitives pb_ef6`)
- [ ] `cargo test --all` (includes `core protocol_schema`, `core hash_schema`, `core card_defs_fmt`)
- [ ] Clippy clean (`cargo clippy --all-targets -- -D warnings`)
- [ ] `cargo build --workspace` (the seal + the exhaustive-match gate across simulator/TUI/viewer)
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — the defs are only checked by the script)

## Risks & Edge Cases

- **Auto-picker silent mis-route**: the two abilities.rs matches have `_` catch-alls, so a missing
  TargetOpponent arm compiles but routes to a battlefield-object scan (returns no player → trigger
  skipped, or worse). Step 5 arms are mandatory; the non-vacuous integration tests are the guard.
- **No self-fallback vs mandatory trigger**: shaman/raiders/vengeful are mandatory "target opponent"
  triggers. In any live multiplayer game an active opponent always exists, so removal-from-stack is
  an edge only reachable in contrived states (last opponent lost). The dedicated
  `no_opponent_removed_from_stack` test pins CR 603.3d and the no-self-fallback.
- **`caster` threading**: `validate_player_satisfies_requirement` is only called from the two casting
  sites (grep-confirmed, engine-wide). Both have `caster` in scope. No hidden caller.
- **Wire double-bump**: if only one of PROTOCOL/HASH moves, investigate before re-pinning — do not
  silently re-pin one and skip the other. Re-pin values come ONLY from the failing gate text.
- **forbidden_orchard recipient wiring**: depends on PB-EF2 `TokenSpec.recipient` being present and
  defaulting to `Controller`. If the field name differs, do the `targets` fix only and leave the
  recipient gap in the marker — do not fabricate a field.
- **blessed_alliance flat-target subtlety**: only index 3 becomes TargetOpponent; index 0 stays
  TargetPlayer. Mixing is fine (each slot is validated against its own requirement), but the runner
  must not blanket-replace all `TargetPlayer` in that file.
- **matches_filter subtype spelling**: shaman's `SubType("Elf")` must match the corpus subtype
  spelling; Elf is a standard creature subtype — verify against another Elf def if the count reads 0
  in the integration test.
```