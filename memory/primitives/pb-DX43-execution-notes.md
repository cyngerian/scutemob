# PB-DX43 execution notes — test authoring (`scutemob-213`)

Scope of this note: the two test files this session owns —
`crates/engine/tests/rules/pb_dx43_intrinsic_land_mana.rs` (13 probes, P1-P13) and
`crates/engine/tests/core/pb_dx43_land_type_roster.rs` (5 rows, R1-R5) — plus the revert matrix
proving every probe/row discriminates real behaviour. The third owned file
(`crates/simulator/tests/pb_dx43_intrinsic_mana_channel.rs`) belongs to a different session and
was **not** touched here.

## Result

- `cargo test -p mtg-engine --test rules pb_dx43` — **13 / 13 passing**.
- `cargo test -p mtg-engine --test core pb_dx43` — **5 / 5 passing**.
- `cargo test -p mtg-engine --test rules pb_dx27_blood_moon_type_scope` — **9 / 9 passing**
  (unweakened regression suite, re-verified after every revert row below).
- `cargo clippy -p mtg-engine --test rules --test core -- -D warnings` — clean.
- `cargo fmt --check -p mtg-engine` — clean.
- R3 measured population (printed): **46** `Complete` defs carry a basic land subtype in their
  type line — matches the plan's stated figure exactly.

No probe went red against the shipped implementation. No probe was weakened.

## Revert matrix

Every row was executed: the named line(s) were edited, the affected test target was run and its
output captured, then the edit was reverted and `git diff --stat` on the touched source file was
confirmed empty (byte-identical to the committed state) before moving to the next row.

| # | What was reverted | File : line(s) | Tests observed RED | Tests confirmed still GREEN | Restored? |
|---|---|---|---|---|---|
| 1 | Disabled the whole derivation (`derive_intrinsic_land_mana_abilities` becomes an early `return;`) | `layers.rs` — body of `derive_intrinsic_land_mana_abilities` (~1635) | P1, P2, P3, P4, P6, P7, P10, P12, R5 | P5, P8, P9, P11, P13, R1, R2, R3, R4 | Yes |
| 2 | Removed the D4 idempotence check — `push_back` runs unconditionally instead of gated on `!already_present` | `layers.rs` — `derive_intrinsic_land_mana_abilities` (~1644-1650) | P5, R3, R4 | P1, P2, P3, P4, P6-P13, R1, R2 | Yes |
| 3 | Moved the derivation call from end-of-Layer-4 to AFTER the whole layer walk (past Layer 6) | `layers.rs` — call site (~508-510) moved to just before `Some(chars)` (~613) | P9 | P1-P8, P10-P13, R1-R5 | Yes |
| 4 | Removed the `chars.card_types.contains(&CardType::Land)` guard | `layers.rs` — top of `derive_intrinsic_land_mana_abilities` (~1636-1638) | P11 | P1-P10, P12, P13, R1-R5 (P8 unaffected — see note below) | Yes |
| 5 | Weakened D4: `activation_condition.is_none()` conjunct replaced with `(.. \|\| true)` | `layers.rs` — `discharges_intrinsic_mana_ability` (~1688) | P12 (the conditioned-ability half) | everything else | Yes |
| 5b | Weakened D4: `*life_cost == 0` conjunct replaced with `(.. \|\| true)` | `layers.rs` — `discharges_intrinsic_mana_ability` (~1686) | P12 (the costed-ability half — confirmed the failure moved from the conditioned assertion to the costed one) | everything else | Yes |
| 6 | Removed the `sets_a_basic_type` precondition on `SetLandTypes`'s CR 305.7 ability-clearing (clearing now runs unconditionally, even for a nonbasic payload like Gate) | `layers.rs` — `SetLandTypes` arm (~1900-1911) | P13 | P1-P12, R1-R5, and `pb_dx27_blood_moon_type_scope.rs` t1-t9 (unaffected — its own fixtures always set a basic type) | Yes |
| 7 | Fixture-level (not a source revert): reproduced the **pre-PB-DX43 architecture** as a temporary standalone test — CR 305.7 removal modelled as a separate Layer-6 `RemoveAllAbilities` (mimicking Blood Moon's old removal component, timestamp 10) alongside a Layer-6 `AddManaAbility(Red)` (mimicking Blood Moon's old own grant, timestamp 15, deliberately later so it survives its own removal) and an external Layer-6 `AddManaAbility(Green)` grant at timestamp 1 (earliest) | `pb_dx43_intrinsic_land_mana.rs` — temporary `revert_row_7_pre_pb_dx43_shape_strips_earlier_third_party_grant`, added, run, and deleted | The temporary test's own assertion (`!chars.mana_abilities... Green`) — i.e. the reproduction **confirmed the historical violation occurs**: Blood Moon's own red grant survives (ordered to), but the earlier external green grant is wiped. This is the executed proof P7's doc comment cites. | n/a (temporary test, not part of the shipped suite) | Yes — deleted after one run; `git diff` on the test file was checked empty relative to its final committed content |
| 8a | Shrank `LAND_TYPE_CONFERRING_VARIANTS` from 4 to 3 (removed `"SetLandTypes"`) | `pb_dx43_land_type_roster.rs` — R1's variant list (~80-81) | R1 (population drops from 5 to 3 — Blood Moon and Magus of the Moon, which use ONLY `SetLandTypes`, disappear) | R2-R5 | Yes |
| 8b | Shrank `TOKEN_SPEC_FIELDS` from 17 to 16 (removed `"recipient"`) | `pb_dx43_land_type_roster.rs` — R2's field fingerprint (~178-195) | R2 (fingerprint no longer matches ANY `TokenSpec` node — population goes vacuous, `{}` vs the pinned `{Awaken the Woods, Overlord of the Hauntwoods}`) | R1, R3-R5 | Yes |

### Row 1 detail (measured output)

With the derivation disabled entirely, 8 of 13 rules-probes and 1 of 5 core-rows went red:

```
FAILED: p1_urborg_grants_swamp_intrinsic_black_to_a_plains
FAILED: p2_yavimaya_grants_forest_intrinsic_green_to_a_plains
FAILED: p3_dryad_grants_all_five_basics_only_to_lands_its_controller_controls
FAILED: p4_two_moons_together_grant_exactly_one_red_mana_ability
FAILED: p6_ancient_den_under_blood_moon_loses_white_and_has_exactly_red
FAILED: p7_earlier_timestamped_layer_six_grant_survives_blood_moons_layer_four_clearing
FAILED: p10_multi_basic_land_gets_both_colors_in_cr_305_6_order
FAILED: p12_conditioned_or_costed_existing_ability_does_not_discharge_the_intrinsic
FAILED (core): r5_forest_dryad_token_gains_tap_add_green_for_free
```

P5 (Swamp keeps its printed ability regardless of whether the derivation runs — nothing is
added OR removed either way), P8 (face-down; see below), P9 (a printed ability, not a derived
one, is what `RemoveAllAbilities` strips here), P11 (nothing is derived for a non-Land object
whether or not the derivation runs), and P13 (nonbasic payload; nothing to derive either way)
are **not** discriminated by Row 1 — each is discriminated by a different row instead (P5/R3/R4
by Row 2; P9 by Row 3; P11 by Row 4; P13 by Row 6).

## Honest disclosure: P8 is UNDISCRIMINATED by any row in this batch's own code

`p8_face_down_swamp_derives_nothing` did **not** go red under any revert attempted, including
Row 1 (disable the whole derivation) and Row 4 (remove the Land-card-type guard). The reason is
structural, not a gap in the revert attempts: CR 708.2a's face-down blank
(`layers.rs:329-342`) is **pre-existing** code that runs **before** the Layer-4 loop even
starts, and unconditionally sets `chars.card_types = {Creature}` and `chars.subtypes = {}`.
By the time `derive_intrinsic_land_mana_abilities` would run (end of Layer 4), a face-down
object already has neither a Land card type nor any subtype — the derivation has nothing to
see regardless of whether its own guard, its own idempotence check, or its own existence is
reverted.

P8 is therefore not a discriminator of anything PB-DX43 added; it is a **regression pin**
protecting the ORDERING invariant D6 relies on (face-down blanking must keep running before the
Layer-4 loop) against a future reordering that no line in this batch's diff could introduce. It
is kept because D6 explicitly claims this "falls out for free" and the plan says to assert that
rather than assume it — the honest disclosure is that "falls out for free" also means "cannot be
un-proven by reverting this batch's own lines."

## Notes on design decisions the reverts confirmed

- **D1 (placement at end-of-Layer-4)** is load-bearing exactly once, at P9/Row 3: moving the
  call past the whole layer walk makes the derived ability immune to a Layer-6
  `RemoveAllAbilities`, which is the CR-wrong shape the plan's §5 D1 argument predicted.
- **D3/D4 idempotence** is load-bearing at P5/R3/R4/Row 2, and R4's own printed numbers
  (Everywhere token 5 → 10 without it) are an exact, measured confirmation of D3's point (2)
  in the plan ("closes OOS-DX27-10 WITHOUT a push_back dedup guard... two moons... exactly
  one, not two" — and separately, the Everywhere token's OWN five hand-authored abilities
  would double without the SAME idempotence check).
- **D2 (CR 305.7 removal folded into the Layer-4 `SetLandTypes` arm, gated on
  `sets_a_basic_type`)** is load-bearing at P13/Row 6 for its precondition, and at P7/Row 7 for
  its placement (Layer 4, not a separate Layer-6 static) — Row 7's fixture-level reproduction
  is the only row in this matrix that is not a source-line revert, because the alternative
  (Layer-6 clearing, timestamp-ordered against unrelated Layer-6 grants) is a design this
  batch's shipped code no longer contains anywhere to revert into; reproducing it as a
  standalone fixture was the closest executable proof.
- **The `BASIC_LAND_TYPES` iteration order (D5)** was proven by P10's own construction
  (Forest+Plains, chosen specifically because alphabetical order and CR 305.6 order disagree
  on this pair) rather than by a revert row — no single-line revert of `BASIC_LAND_TYPES`'s
  array order would be meaningful without also breaking every other probe that reads it, so
  P10 stands on construction rather than negative-control evidence.

## Population figures printed by the tests themselves (not transcribed)

- R1 (payload-derived conferring population): 5 — `Blood Moon`, `Magus of the Moon`,
  `Urborg, Tomb of Yawgmoth`, `Yavimaya, Cradle of Growth`, `Dryad of the Ilysian Grove`.
- R2 (inverse `TokenSpec`-derived population): 2 — `Awaken the Woods`,
  `Overlord of the Hauntwoods`.
- R3 (printed-basic-land-subtype population): **46** `Complete` defs (`eprintln!`'d by the test
  itself under `--nocapture`), all ability-index-neutral.

---

# Part 2 — coordinator record (`scutemob-213`)

Everything below was measured by the coordinating session: the census, the design decision, the
third test file, and the batch-level gates. Written after Part 1 so the two do not contradict each
other; where they disagree, Part 2 is later.

## The census — the memo's 5 is a floor, and it is short by three

The v4 memo (`memory/primitives/seed-rerank-2026-08-14.md` §2.1) publishes its derivation rule:
scan `crates/card-defs/src/defs/*.rs` comment-stripped for a land-type-conferring
`LayerModification` whose payload names a basic land subtype. **Re-run at HEAD it reproduces the
memo exactly** — 6 hits, minus `awaken_the_ancient` (Mountain appears only in its `EnchantFilter`)
= 5. Dispatch hygiene 6 says treat a known-site list as a floor, so a second axis was run.

**The inverse axis starts from the printed card, not from the layer modification**, and finds
three the payload rule structurally cannot see, because a token grants its types through a
`TokenSpec` and never through a `LayerModification` at all:

| def | shape | `Complete`? | disposition |
|---|---|---|---|
| `awaken_the_woods` | "Forest Dryad land" token, `mana_abilities: vec![]` | yes (`#[default]` derive) | **4th live-wrong def — fixed for free** (R5, C5) |
| `overlord_of_the_hauntwoods` | Everywhere token: 5 basic subtypes **and** 5 hand-authored abilities | yes (explicit) | **3rd double-grant risk — proven not to double** (R4: exactly 5, not 10) |
| `leyline_of_the_guildpact` | prints the clause, authors nothing | `Inert` | out of scope; filed `OOS-DX43-1` |

So the class is **8 defs, not 5**. This is PB-DX26's lesson arriving again: *a roster derived from
one declaration construct measures that construct.* Both axes are now standing roster rows (R1
payload, R2 inverse) so neither half can silently regrow.

## The design decision, and the thing that forced it

CR 305.6's intrinsic ability is a consequence of the type change, so it belongs to **layer 4**
(CR 613.1d) — which means a layer-6 ability removal must still be able to strip it (CR 613.1f).
That is P9, and it is the probe that makes the placement decision real rather than stylistic.

**That reading then forces a second conclusion the brief did not state.** Deleting only the moons'
`AddManaAbility` — which is literally what criterion 6507 asks for — would have left each moon's
own **layer-6 `RemoveAllAbilities`** wiping the **layer-4** derived ability. Blood Moon would have
stopped working entirely, and `pb_dx27_blood_moon_type_scope::t6` would have caught it. So CR
305.7's ability-LOSS half moved into the `SetLandTypes` primitive itself, conditioned on the
payload containing a basic land type (CR 305.7's own precondition — P13 proves a `Gate` payload
triggers neither the clearing nor any derivation), and both moons drop **two** statics apiece.

**This closes a latent CR 305.7 violation nobody had filed.** The rule's final sentence is *"Note
that this doesn't remove any abilities that were granted to the land by other effects."* A blanket
**layer-6** `RemoveAllAbilities` is timestamp-ordered against every other layer-6 effect, so it
could strip an earlier-timestamped grant from another source — Cryptolith Rite, Chromatic Lantern,
The World Tree, Bootleggers' Stash and Wrenn and Realmbreaker all grant into
`LandsYouControl`/`AllLands`. Moving the removal to layer 4 makes every layer-6 grant survive
regardless of timestamp. That is **P7**, and it fails on the pre-PB-DX43 shape.

## The basics decision (criterion 6508), decided by a gate rather than by taste

**Keep** the hand-authored `{T}: Add` on `swamp.rs` et al.; make the derivation idempotent instead.

1. `crates/engine/tests/core/effect_choose_gate.rs::every_complete_land_registers_each_printed_tap_mana_color`
   compares a def's **oracle text** against its **`enrich_spec_from_def` lowering** — a pure
   registry path with no `GameState` and no layers. Deleting a basic's printed ability makes it
   report `missing {B}` and go red, **correctly**: a def that prints "{T}: Add {B}" and lowers
   nothing is a def whose spec lies about the card. CR 305.6 says a Swamp does not *need* the
   printed text; it does not say a Swamp that *has* it should stop declaring it.
2. `Command::TapForMana.ability_index` is a **dense index** into `mana_abilities`
   (`rules/command.rs:25-29`, consumed `rules/mana.rs:152-160`). Idempotence keeps the printed
   ability at index 0; deletion would move every basic land's ability from base into a derived
   append — the `OOS-DX26-3` hazard, on the most common object in the game.
3. `rules/face.rs:115` and `rules/resolution.rs:891` **rebuild base `mana_abilities` from the
   def** at every face change. A basic land declaring nothing would have an empty base vector there.

Proven by R3 across all **46** `Complete` defs that print a basic land subtype: resolved
`mana_abilities` equals the base spec's exactly — same length, same order, nothing added, no index
moved. Independently, no such def carries a conditioned or costed mana ability, so D4's exclusion
never fires on the live corpus (it is there for the case P12 constructs).

## Revert matrix — `crates/simulator/tests/pb_dx43_intrinsic_mana_channel.rs` (8 probes)

Each row: edit, run, capture, restore, confirm `git diff` empty against `HEAD`.

| # | What was reverted | Tests observed RED | Restored? |
|---|---|---|---|
| V1 | `derive_intrinsic_land_mana_abilities` call disabled (`&& false` on the `EffectLayer::TypeChange` guard) | C1, C2, C3, C4, C5, C6b | yes |
| V2 | idempotence dropped (`already_present` guard bypassed) | C1, C3, C4, C6 | yes |
| V3 | `SetLandTypes`' CR 305.7 clearing disabled (`sets_a_basic_type && false`) | C6b | yes |
| V4 | card-def revert: Dryad's filter widened `LandsYouControl` → `AllLands` | C4b | yes |

**All 8 probes discriminate; 0 UNDISCRIMINATED.** C6 and C4b are invariance probes and are
correctly green under V1 — they are discriminated by V2 and V4 respectively, which is why those
two rows exist.

## Two fixture defects this file found in itself, both worth carrying

1. **`GameStateBuilder::build()` never registers static continuous effects.** Nothing does until a
   permanent actually enters through `Command::PlayLand` (`rules/lands.rs`) or spell resolution.
   A conferring permanent dropped straight onto the battlefield by the builder therefore registers
   **no `ContinuousEffect` at all** — Urborg sat there conferring nothing and the first draft of
   this file failed on all three staples **for a reason it did not describe**. That is the mirror
   image of PB-DX25b's fixture, which made a probe *pass* by removing the only condition under
   which the code was wrong. Fixed by entering the card through the real command path, which is
   strictly stronger evidence: the probes now prove *play Urborg, THEN tap your Plains for `{B}`*.
   Filed `OOS-DX43-6`. Note Part 1's P1-P3 take the other route — hand-built `ContinuousEffect`
   fixtures mirroring each card's static — so the two files cover each other: Part 1 pins the
   mechanism, Part 2 pins the real cards.
2. **An offer-layer assertion about a player who does not hold priority is structurally vacuous.**
   `StubProvider::legal_actions` returns nothing at all for such a player, so C4b's first draft
   would have read 0 offers whatever the derivation did. It reads the mana solver instead, which
   filters on `obj.controller` rather than on priority. Filed `OOS-DX43-7`.

## Batch-level gates (all executed)

- **Tests: 4,749 / 0 / 5** full-workspace (`--workspace --no-fail-fast` to a file), **50**
  result-producing targets (49 → 50: the new simulator test binary), residual list empty.
  **+28 over the 4,721 pre-edit baseline measured on this branch before any edit**, itemised by
  test NAME by set-diffing the two run logs, with **ZERO removals**: 13 in
  `rules/pb_dx43_intrinsic_land_mana.rs`, 8 in `simulator/tests/pb_dx43_intrinsic_mana_channel.rs`,
  5 in `core/pb_dx43_land_type_roster.rs`, 2 in `card-types`' new `basic_land_types_tests`.
- **PROTOCOL 37 / HASH 76 both UNMOVED**, gate-executed (`hash_schema` 36/36,
  `protocol_schema` 17/17). The prediction was recorded in the plan (D7) **before** any code
  change and held; the numbers here are read off the gates, not off the prediction.
- **Coverage 1,136/1,803 = 63.0%, 0 flips**, proven by regenerating `tools/authoring-report.py`
  and diffing: clean 1,136 / todo 520 / empty 147 all identical, every changed line self-dating
  (timestamp, git SHA, recent-commit list, day-window counts). Churn reverted.
- `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs).
- `git diff --numstat` for the implementation commit: `layers.rs` +174/−2, `types.rs` +75/−0,
  `blood_moon.rs` +33/−42, `magus_of_the_moon.rs` +25/−38, `tools/play-server/src/main.rs` +15/−1.
  **`crates/view-model` and `crates/simulator/src` are both 0** — the derivation is reachable
  through every consumer without a single production line outside the engine, because every one
  of them already reads layer-resolved characteristics.

## One seeded constant moved, and it was predicted

`tools/play-server/src/main.rs`'s `UI3_SPLIT_COMBAT_SEED` **28 → 32**. The plan's own §7 point 6
named this hazard in advance ("the derived ability makes lands offer a `TapForMana` where they
offered none; re-observe any seeded constant rather than editing it to taste"). Re-observed by an
executed sweep over seeds 0..80; hits satisfying both halves are 32, 47, 48, 79, and 32 is the
lowest. The constant's doc records the re-observation in the file's established convention rather
than silently replacing the number — this is its fourth.
