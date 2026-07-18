# Primitive Batch Review: PB-EF10 — sacrifice-driven `EffectAmount` / runtime `max_cmc` / "if you do" `Condition`

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: scutemob-111
**CR Rules**: 608.2b, 608.2c, 608.2h, 608.2i, 701.21a, 202.3, 613.1d, 400.7, 118.8
**Engine files reviewed**: `effects/mod.rs` (resolve_amount, SearchLibrary, SacrificePermanents, sacrifice_permanents_for_player, MoveZone, check_condition, EffectContext), `rules/abilities.rs` (activated-cost capture), `rules/casting.rs` (spell-additional-cost capture), `rules/resolution.rs` (2 copy-into-ctx sites), `state/hash.rs` (all new HashInto arms + version), `rules/protocol.rs` (PROTOCOL_VERSION + history), `card-types/src/state/types.rs` (`SacrificedCreatureLki`, `AdditionalCost::Sacrifice`)
**Card defs reviewed**: momentous_fall, eldritch_evolution, victimize (new Complete), miren_the_moaning_well, diamond_valley (forced-add flips), birthing_ritual (inert), disciple_of_freyalise (stays partial)
**Tests reviewed**: `crates/engine/tests/primitives/pb_ef10_sacrifice_driven_amounts.rs` (15 tests)

## Fix phase (2026-07-18) — both LOW findings resolved (commit `bcf9eb8a`)

- **LOW #1 FIXED** (`casting.rs`): the second `expect_characteristics` call is gone;
  `sac_lki_power` is now `sac_lki.power` (one layer calc). No behavior change; suite green.
- **LOW #2 FIXED** (`effects/mod.rs` MoveZone `dest_tapped`): added the dedicated regression
  test `test_move_zone_returns_to_battlefield_tapped` in
  `crates/engine/tests/primitives/pb_ef10_sacrifice_driven_amounts.rs`. It moves a graveyard
  card to the battlefield via `Effect::MoveZone { target: Source, to: Battlefield { tapped } }`
  (reassembling_skeleton's shape) and asserts the returned object's `status.tapped == want_tapped`
  for BOTH `true` and `false`, so a MoveZone that drops `dest_tapped` fails independently of
  Victimize. Suite: 3452 → **3453** passing. All gates re-run green.

## Verdict: needs-fix (LOW only — functionally correct and mergeable) — RESOLVED

The batch is correct across every scrutinized area. All three sub-gaps chain-verify against CR
and MCP oracle text; all three capture sites read layer-resolved characteristics before
`move_object_to_zone`; the `PowerOfSacrificedCreature` reads were migrated to `.power` without
desync; the three decoys are genuinely non-vacuous; version bumps are machine-forced and
history rows appended; EF-EF1-A is untouched (Step 3.4 correctly skipped). No HIGH or MEDIUM
findings. Two LOW polish items only (a redundant `expect_characteristics` call in casting.rs, and
the absence of a dedicated regression test for the MoveZone/`reassembling_skeleton` tapped-return
correction). The MoveZone `dest_tapped` bonus fix is correct and — a notable positive — silently
fixes shipped-Complete `reassembling_skeleton`, which was entering untapped against its oracle
("return this card... to the battlefield tapped").

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `rules/casting.rs:4231-4251` | **Redundant layer calculation.** `sac_lki_power` (i32) and `sac_lki` each call `expect_characteristics(state, sac_id)` separately; `sac_lki_power == sac_lki.power`. **Fix (optional):** derive `sac_lki_power` from `sac_lki.power` and drop the second `expect_characteristics` call. Correctness unaffected. |
| 2 | LOW | `effects/mod.rs:2774` (MoveZone dest_tapped) | **Bonus fix under-tested for its full blast radius.** The `dest_tapped(to)` application is exercised only via the two Victimize integration tests. It also corrects shipped-Complete `reassembling_skeleton.rs:35` (`to: Battlefield { tapped: true }`), which was entering untapped — a genuine pre-existing bug fix, not a regression. **Fix (optional):** add a direct MoveZone-returns-tapped unit test or a `reassembling_skeleton` regression test pinning the corrected behavior. |

## Card Definition Findings

None. All six reviewed defs match oracle text and chain-verify.

## Area-by-Area Scrutiny (as directed)

### 1. LKI capture correctness (CR 608.2b/608.2h/608.2i) — CLEAN
All three capture sites read **layer-resolved** characteristics **before** the zone move:
- `abilities.rs:1021-1037` (activated cost): uses `calculate_characteristics`/`expect_characteristics` (`resolved`), captures power/toughness/mana_value with `obj.characteristics` LKI fallback, pushes struct before the zone move at ~1044+. Rename `sacrificed_lki_powers → sacrificed_lki` complete; assignment at `abilities.rs:1292`.
- `casting.rs:4234-4251` (spell additional cost): `expect_characteristics(state, sac_id)` before the move; patches `AdditionalCost::Sacrifice.lki` in place with correct `resize`-to-len then `lki[pos]=` (parallel-to-`ids` contract honored).
- `sacrifice_permanents_for_player` `effects/mod.rs:7814-7830` (resolution effect): `calculate_characteristics(state, id)` (with `obj.characteristics` fallback) captured before `expect_move_object_to_zone`.

No site reads a post-move graveyard object. The `PowerOfSacrificedCreature` resolver was correctly migrated to `.map(|l| l.power)` (`effects/mod.rs:7323-7327`) — no copy-paste desync with the new `.toughness` (7331-7335) / `.mana_value as i32` (7339-7343) arms. The two `resolution.rs` copy-into-ctx sites (393-405 for spells via `find_map` on the first `AdditionalCost::Sacrifice`; 1870 for activated via `stack_obj.sacrificed_creature_lki.clone()`) are both correct.

### 2. `sacrifice_permanents_for_player` return + overwrite semantics — CLEAN
`Vec<SacrificedCreatureLki>` is pushed **only** on the two success arms where the zone move
completed (`Redirect` inside `if let Some((new_id,_))` at 7861→7865; `Proceed` inside
`if let Some((new_id,_))` at 7937→7941). The `ChoiceRequired` arm (7916-7934) defers and does
**not** push — so `sacrifice_fired` is never falsely set for a deferred sacrifice. In
`Effect::SacrificePermanents` (3400-3431), `ctx.sacrifice_fired = !sacrificed.is_empty()` and
`ctx.sacrificed_creature_lki = sacrificed` (overwrite, not append) correctly model "if you do"
referring to *this* instruction. Per-resolution scoping (fresh `EffectContext` per resolution)
means no stale latching. The documented single-`SacrificePermanents`-per-resolution limitation is
acceptable for all candidates. Note: the `Redirect`-to-Command-zone case (commander) still pushes
LKI — correct, since the commander *was* sacrificed before the 903.9 zone-replacement redirect.

### 3. `Effect::MoveZone` `dest_tapped()` bonus fix — CORRECT, no regression
`dest_tapped(to)` returns `Some(bool)` only for `ZoneTarget::Battlefield { tapped }` and `None`
otherwise, so non-battlefield MoveZone destinations are untouched. `Battlefield { tapped: false }`
sets `status.tapped = false` — a no-op against the default-untapped new object (CR 400.7). The
only behavioral change is `Battlefield { tapped: true }` now taps, which is what every such caller
wrote and intended. Blast radius across card defs: only `victimize.rs` (new) and
`reassembling_skeleton.rs:35` use MoveZone `to: Battlefield { tapped: true }`; the latter's oracle
is "return this card... to the battlefield **tapped**" — the fix **corrects a pre-existing bug**
(it was entering untapped). All other `Battlefield { tapped: true }` sites are SearchLibrary
`destination:`/`matched_dest:`, which already tapped via the separate `matched_tapped` path
(`effects/mod.rs:4838`). Tests pass (3452), so no test encoded the old untapped bug. Under-tested
only for a dedicated `reassembling_skeleton` regression (LOW #2).

### 4. Victimize vs oracle + ruling 2020-11-10 — CLEAN
`victimize.rs` models `Sequence[SacrificePermanents{Fixed(1), creature}, Conditional{SacrificeFired, if_true: Sequence[MoveZone(DeclaredTarget 0, Battlefield{tapped:true}, controller=Controller), MoveZone(DeclaredTarget 1, ...)], if_false: Nothing}]`. Oracle order (sacrifice → "if you do" → return tapped) is preserved. Each returned card is tapped (`dest_tapped` fix) and under the controller (`controller_override: Some(Controller)`; also owner-correct since targets are in *your* graveyard). One-illegal-target still sacs + returns the other (DeclaredTarget for the illegal slot resolves to nothing via the engine's resolution-time target-legality check — confirmed empirically by `test_victimize_one_illegal_target_still_sacs_and_returns_other`). Both-illegal → spell doesn't resolve → no sac is existing engine behavior. Integration tests cover the sac-fires, no-creature, and one-illegal-target cases.

### 5. Birthing Ritual = inert — CORRECT
`birthing_ritual.rs` has `abilities: vec![]` and no other behaviour-bearing field →
`registers_no_behavior` true → `Completeness::inert` is the taxonomy-mandated marker (the plan's
"partial" was wrong; `card_registry_gate::test_no_behavior_defs_are_inert_not_partial_or_known_wrong`
enforces `inert`, and the runner correctly flipped it). OOS-EF10-1 is a **real** blocker, precisely
named: no `Effect` scopes candidates to a looked-at top-N subset (not the whole library), places at
most one matching a runtime MV cap, and sends the remainder to the bottom in random order.
`SearchLibrary` searches the whole library and has no bottom-randomize destination — using it would
be legal-but-wrong. Not an under-attempt: the trigger, intervening-if, optional sacrifice, and
runtime MV cap are all otherwise expressible after this PB; only the dig is missing.

### 6. Decoys non-vacuous — CLEAN (3/3)
- `test_toughness_amount_reads_toughness_not_power` (271): sacs a 1/3, asserts gain **3**. A `.power` copy-paste would gain 1 → assert fails. Non-vacuous.
- `test_search_cap_uses_both_terms` (498): cap = `Sum(Fixed(2), ManaValueOfSacrificedCreature=3)` = 5, card MV-5, asserts found. Drop `+2` → cap 3 → rejected; drop sac-MV → cap 2 → rejected. Pins **both** summands. Non-vacuous.
- `test_sacrifice_fired_false_when_none_available` (757): no creature; asserts `+7` branch did NOT run AND `!ctx.sacrifice_fired`. An unconditional `sacrifice_fired = true` fails both asserts. Non-vacuous.

### 7. Version bumps — CLEAN
`PROTOCOL_VERSION = 15` (protocol.rs:152) with an **appended** `ProtocolEpoch { version: 15, ... }`
row (298-309; v14 row intact above it). `HASH_SCHEMA_VERSION = 53` (hash.rs:482) with the `- 53:`
history docstring (464-481) and appended history row. All new HashInto arms present and
non-colliding: `SacrificedCreatureLki` impl (3870-3876); `AdditionalCost::Sacrifice { ids, lki }`
(3880-3889); `StackObject.sacrificed_creature_lki` (3784-3785); `EffectAmount` discriminants 22/23
(5402/5405, after HandSize=21, no collision with PowerOf=15); `Condition::SacrificeFired = 48`
(5824, after 47); `TargetFilter.max_cmc_amount` (5132). `EffectContext.sacrifice_fired`/
`sacrificed_creature_lki` correctly **not** hashed (transient scratch). Sentinel test
`test_pb_ef10_version_sentinels` asserts 15/53. WIP records fingerprints copied from failing gates,
not hand-guessed.

### 8. EF-EF1-A NOT regressed — CLEAN
Step 3.4 skipped per plan permission. `pay_optional_cost`'s `Cost::Sacrifice` arm
(`effects/mod.rs:8083-8090`) calls `sacrifice_permanents_for_player` and discards the return with
`let _ =` — the optional-cost path signature and behavior are unchanged; `exclude_self` source
threading intact. `disciple_of_freyalise.rs` stays `Completeness::partial` with the EF-EF1-A note
intact. ziatora/disciple behave exactly as before. EF-EF1-A remains filed, not broken.

## Card Def Chain-Verification (filter → amount → effect → cost)

### momentous_fall.rs — Complete ✓
Oracle: "As an additional cost, sacrifice a creature. You draw cards equal to the sacrificed
creature's power, then gain life equal to its toughness." →
`spell_additional_costs: [SacrificeCreature]` (LKI captured casting.rs) → `Sequence[DrawCards{PowerOfSacrificedCreature}, GainLife{ToughnessOfSacrificedCreature}]`. Order and both LKI reads correct. Integration test sacs a 3/4 → draw 3, gain 4.

### eldritch_evolution.rs — Complete ✓
Oracle: "...sacrifice a creature. Search library for a creature card with MV X or less, X = 2 + sac
creature's MV. Put onto battlefield, then shuffle. Exile Eldritch Evolution." →
`SacrificeCreature` cost → `SearchLibrary{filter: has_card_type=Creature, max_cmc_amount: Sum(Fixed(2), ManaValueOfSacrificedCreature)}` → `Effect::Shuffle` → `self_exile_on_resolution`. The explicit `Effect::Shuffle` (plan deviation) is **correct and single** — `shuffle_before_placing: false` means SearchLibrary does not shuffle, so no double-shuffle; `Effect::Shuffle` (effects/mod.rs:3064) is a real implemented effect. Order (place, then shuffle) matches oracle. Integration test sacs MV-2 (cap 4): MV-4 enters, MV-5 does not, spell exiled.

### victimize.rs — Complete ✓
See area 4.

### miren_the_moaning_well.rs — Complete ✓ (forced-add)
Oracle: "{T}: Add {C}. / {3},{T},Sacrifice a creature: You gain life equal to the sacrificed
creature's toughness." → ability 0 = `{T}: Add {C}` (mana ability, filtered out of
`activated_abilities` by enrich); ability 1 (activation index 0) = `Sequence[Mana{3}, Tap, Sacrifice(creature)] → GainLife{ToughnessOfSacrificedCreature}`. Exact oracle match. Layer-resolved capture proven by `test_toughness_of_sacrificed_creature_reads_layer_resolved` (anthem-boosted 2/4 → gain 4).

### diamond_valley.rs — Complete ✓ (forced-add)
Oracle: "{T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness."
(no mana ability). → single `Activated{Sequence[Tap, Sacrifice(creature)] → GainLife{ToughnessOfSacrificedCreature}}`. Exact match; the note correctly records it has no `{T}: Add {C}`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 608.2b (LKI power/toughness) | Yes | Yes | `test_toughness_of_sacrificed_creature_basic`, momentous_fall |
| 608.2h/608.2i (LKI mana value, look-back) | Yes | Yes | anthem-resolved test; eldritch_evolution MV cap |
| 608.2c (sequential "if you do") | Yes | Yes | `test_sacrifice_fired_true/false`, victimize |
| 202.3 (mana value) | Yes | Yes | `test_search_max_cmc_amount_caps_by_runtime_value` |
| 701.21a (sacrifice keyword action) | Yes (reused) | Yes | victimize sac + return |
| 613.1d (layer-resolved chars) | Yes | Yes | anthem-boosted toughness = 4 |
| 400.7 (object identity / LKI before move) | Yes | Yes (implicit) | all 3 capture sites pre-move |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| momentous_fall | Yes | 0 | Yes | draw=power, gain=toughness |
| eldritch_evolution | Yes | 0 | Yes | explicit Shuffle, single, correct |
| victimize | Yes | 0 | Yes | sac→if-you-do→return tapped/controlled |
| miren_the_moaning_well | Yes | 0 | Yes | forced-add; toughness outlet |
| diamond_valley | Yes | 0 | Yes | forced-add; no mana ability |
| birthing_ritual | Yes (inert) | 1 (allowed, inert) | N/A (inert) | OOS-EF10-1 real + named |
| disciple_of_freyalise | Yes (partial) | note only | N/A (partial) | EF-EF1-A intact, not regressed |

## Notes / Informational (not defects)

- **Semantic improvement, no regression**: `Effect::SacrificePermanents` now populates
  `ctx.sacrificed_creature_lki`/`sacrifice_fired` where it previously left them untouched. This is
  strictly more correct for resolution-sac cards; no shipped card combines a resolution
  `SacrificePermanents` with a subsequent `PowerOf/ToughnessOfSacrificedCreature` read on the
  *cost* creature, so nothing regresses (3452 tests green).
- **reassembling_skeleton corrected**: the MoveZone `dest_tapped` fix aligns it with oracle
  ("return... tapped"). Positive side effect; captured under LOW #2 only for the missing dedicated
  regression test.
