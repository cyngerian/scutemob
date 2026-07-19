# Primitive WIP — PB-OS8 (LookAtTopThenPlace) — phase: fix-done / ready

<!-- last_updated: 2026-07-19 -->

**Batch**: PB-OS8 — `Effect::LookAtTopThenPlace` (OOS-EF10-1 + `min_cmc_amount` rider + PB-OS6's
deferred sub-primitive (d), growing_rites ETB).
**Task**: `scutemob-138` — branch `feat/pb-os8-lookattopthenplace-oos-ef10-1-mincmc-os6-deferred-d-b`.
**Phase**: plan DONE → implement DONE (all 13 steps) → review DONE → fix DONE → **ready** (not committed).
**Plan**: `memory/primitives/pb-plan-OS8.md` (read it — full CR, algorithm, wire sites, verdicts).

## Review outcome (2026-07-19) — needs-fix (0 HIGH / 1 MEDIUM / 3 LOW)

Review file: `memory/primitives/pb-review-OS8.md`. Engine primitive CR-correct on every focus axis
(top-N scoping, LKI-before-cap ordering, ≤1 cardinality, deterministic remainder, decline path,
min/max mirror, hash coverage, single justified wire bump, no new leak). Both card flips match
oracle exactly and are `Complete`; birthing_pod (inert) + muxus (partial) honestly deferred with
correct seeds. Only gaps are in test coverage.

**Fix directives:**
1. **MEDIUM (F3)** — add a top-N truncation decoy: library of `count + 2`, a matching creature just
   OUTSIDE the top-N window must stay untouched (not placed, not bottomed) while an in-window match
   is placed. This is the core `LookAtTopThenPlace`-vs-`SearchLibrary` distinction and is currently
   never behaviorally exercised (all test libraries hold exactly `count` cards).
2. **LOW (F4)** — add a zero-count / empty-library edge test (nothing placed/bottomed, `place_cost`
   unpaid).
3. **LOW (F1, F2)** — optional doc-only comment clarifications (empty-library vs whiff cost
   asymmetry; `optional` field currently inert). No code change required.

## Fix outcome (2026-07-19) — all 4 findings addressed

1. **MEDIUM (F3, AC 5077) — DONE.** Added
   `test_look_place_truncates_at_top_n_leaves_out_of_window_match_untouched`: `count=3`, library
   of 5 cards (`count + 2`), a matching creature at index 0 (in-window) and a second matching
   creature at index 3 (`== count`, one past the window). Asserts the in-window match is placed;
   the out-of-window match keeps its ORIGINAL `ObjectId` still live in `Library(p1)` (would have
   been retired under CR 400.7 had it been bottomed), and no `ObjectPutOnLibrary`/
   `ObjectReturnedToHand` event ever references its id. Locks the top-N boundary.
2. **LOW (F4) — DONE.** Added `test_look_place_empty_library_places_nothing_and_skips_cost`: empty
   library + `place_cost: Sacrifice(creature)` with a sacrificeable creature present on the
   battlefield; asserts nothing placed, nothing bottomed, the sacrifice is NOT paid (creature
   stays on battlefield), `events.is_empty()`, and `ctx.sacrifice_fired` stays false.
3. **LOW (F1, F2) — DONE.** Added doc comments in `effects/mod.rs`: at the `optional: _` field
   (inert this milestone, reserved for M10 interactive decline) and at the `top_ids.is_empty()`
   guard (documents the empty-library-vs-whiff cost-payment asymmetry as intentional, not a bug).
   No behavioral change.

**Result**: 15 tests total in `pb_os8_look_at_top_then_place.rs` (13 original + 2 new), all
passing. Full suite (`cargo test --all`), `cargo clippy --workspace -- -D warnings`,
`cargo fmt --check`, and `tools/check-defs-fmt.sh` all clean after the fixes. Not committed.

## Key plan outcomes (verified against oracle via MCP)

- **New `Effect::LookAtTopThenPlace { player, count, filter, place_cost: Option<Box<Cost>>,
  destination, rest_to, optional }`** — put-≤1 sibling of the EXISTING `Effect::RevealAndRoute`
  (which puts ALL matches). Model the executor directly on RevealAndRoute (reuse
  `try_pay_optional_cost`, `resolve_zone_target`, `dest_tapped`, `zone_move_event`,
  `object_ids().take(n)`, ObjectId-ascending determinism). `place_cost` = interposed sacrifice
  paid AFTER the look (Birthing Ritual), captures sacrifice LKI into ctx so `max_cmc_amount`
  resolves. NO new `private_to` (doesn't exist; deferred M10). NO `rand`.
- **New `TargetFilter.min_cmc_amount: Option<Box<EffectAmount>>`** (runtime floor; static
  `min_cmc` already exists). Honored in SearchLibrary + LookAtTopThenPlace.
- **Effect discriminant 96** (current max 95 = RemoveFromCombat).
- **WIRE**: single batched **PROTOCOL 22→23 + HASH 59→60** (both forced: Effect variant +
  TargetFilter field, both in SR-8 closure). Sentinel/fingerprint sites enumerated in the plan.

## Roster (discounted ship = 2)

- **FLIP** `birthing_ritual` (inert→Complete) — LookAtTopThenPlace count 7, place_cost
  Sacrifice(creature), max_cmc_amount = 1 + ManaValueOfSacrificedCreature, dest Battlefield,
  rest_to Library Bottom, optional; + existing end-step trigger + intervening-if(creature,count 1).
- **FLIP** `growing_rites_of_itlimoc` (partial→Complete) — ETB LookAtTopThenPlace count 4,
  filter creature, dest Hand, place_cost None, rest_to Library Bottom, optional.
- **STAYS PARTIAL** `birthing_pod` — SearchLibrary card (NOT the dig); min_cmc_amount available
  but SECOND blocker = Phyrexian mana in activated cost is unsupported (abilities.rs has 0
  phyrexian; 2-life path is casting.rs-only). Doc note + new seed **OOS-OS8-1**.
- **STAYS PARTIAL** `muxus_goblin_grandee` — put-MULTIPLE = existing `RevealAndRoute`, NOT
  LookAtTopThenPlace. Re-point note to RevealAndRoute + seed **OOS-OS8-2**; do NOT author here.
- **STAY PARTIAL** (sweep, independent blockers) `carth_the_lion`, `nissa_resurgent_animist`,
  `narset_parter_of_veils`, `harald_king_of_skemfar`, `bounty_of_skemfar`, `satoru_umezawa`, +
  split/multi-dest cards. Verify at implement; do NOT expand roster without escalating.

## Implement checklist (numbered — see plan §Verification Checklist for detail)

1. [x] Add Effect variant + TargetFilter.min_cmc_amount (card_definition.rs). `check -p mtg-card-types` clean.
2. [x] hash.rs: Effect HashInto arm (disc 96) + min_cmc_amount HashInto line.
3. [x] effects/mod.rs: LookAtTopThenPlace executor (algorithm in plan §Change 3). `check -p mtg-engine` clean.
4. [x] effects/mod.rs: min_cmc_amount runtime floor in SearchLibrary.
5. [x] `cargo build --workspace` clean — no exhaustive-match errors anywhere (no tool arm needed).
6. [x] Author birthing_ritual + growing_rites flips (both Completeness::Complete). `check -p mtg-card-defs` clean.
7. [x] Doc-only: birthing_pod note (2 blockers + OOS-OS8-1), muxus note (RevealAndRoute + OOS-OS8-2). Both STAY PARTIAL/inert, no behavioral change.
8. [x] PROTOCOL 22→23 + HASH 59→60 lockstep; re-pinned PROTOCOL_SCHEMA_FINGERPRINT
   (553f2ff2e54c7de707209b79db7f8bca0fc0c37405871a0c1b31c431e6dedb32), appended
   PROTOCOL_HISTORY row 23, re-pinned FROZEN_HISTORY_PREFIX_DIGEST
   (3dc31d4a8eabb7a3ca8d9c4d32b14d4a033a7281a7e21c4d14ca1301903f4501) in
   core/protocol_schema.rs; appended HASH_SCHEMA_HISTORY row 60
   (decl 73e6f285645f087b7aa2346b0b84ba0394f51c6d9ebfff6c18cafc926f665728, stream
   55fa8a7776b413a2bb66ffda61d0d51bf52687e450ae12cd36e697879ff351c1), re-pinned
   FROZEN_HISTORY_PREFIX_DIGEST (bfbb4b54c9603676871e6530780d377aaa436c8fd770f32c7c5945318a850b52)
   in core/hash_schema.rs. Bumped all 6 `PROTOCOL_VERSION, 22` sentinels and all 39
   `HASH_SCHEMA_VERSION, 59u8` sentinels (grep-driven, whole-repo verified clean of
   leftovers). `cargo build --workspace` clean.
9. [x] Wrote tests/primitives/pb_os8_look_at_top_then_place.rs (13 tests, all passing) +
   mod line in primitives/main.rs.
10. [x] `cargo test --all` green (incl. check-defs-fmt via `core card_defs_fmt`). Hit and
   fixed one unexpected gate: SR-25 `bare_lookup_ratchet` ceiling for effects/mod.rs bumped
   109→110 (one new NONSWALLOW predicate read in the LookAtTopThenPlace executor, exact
   copy of RevealAndRoute's existing idiom). Suite now all green.
11. [x] clippy -D warnings + fmt clean. Hit and fixed one unplanned knock-on: adding
   `min_cmc_amount` pushed `TargetFilter`'s size over clippy's `large_enum_variant` gap
   for `Cost::Sacrifice(TargetFilter)` vs `Cost::Mana(ManaCost)` (208 > 200-byte
   threshold). Boxing `TargetFilter` in `Cost::Sacrifice` would touch ~84 call sites
   across 62 files (far outside scope) — used `#[allow(clippy::large_enum_variant)]` on
   `Cost` instead, matching existing precedent at `rules/events.rs` /
   `testing/script_schema.rs` for the same lint. `cargo fmt` reformatted the new test
   file's two long `format!()` calls (mechanical, no logic change). Re-ran full suite +
   `cargo build --workspace` after both fixes — all green.
12. [x] Closed OOS-EF10-1 in ef-batch-plan-2026-07-17.md §12 + oos-retriage-plan-2026-07-18.md
    §3 PB-OS8 block + queue table; PB-OS6 deferred (d) closed (growing_rites Complete). Filed
    OOS-OS8-1 (Phyrexian mana in activated costs) and OOS-OS8-2 (muxus via RevealAndRoute) in
    the retriage doc. workstream-state.md "In flight / next" updated. Confirmed `all_cards()`
    enumerates both flipped defs (proven by the two card-integration tests, which panic via
    `real_card_spec` if the def is missing).
13. [x] No leftover LookAtTopThenPlace/OOS-EF10-1 TODOs in birthing_ritual.rs or
    growing_rites_of_itlimoc.rs (grep clean); birthing_pod/muxus notes reworded (not removed).

## Close-out riders
- OOS-EF10-1 → close in ef-batch-plan §12 + OS plan §3.
- PB-OS6 deferred (d) → close (growing_rites Complete).
- New seeds: OOS-OS8-1 (Phyrexian mana in activated ability costs), OOS-OS8-2 (muxus ETB via RevealAndRoute).
