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
