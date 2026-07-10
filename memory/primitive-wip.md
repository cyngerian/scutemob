# Primitive WIP: PB-AC9 — Misc & mana (final AC-chain batch)

batch: PB-AC9
title: Misc & mana (final AC-chain batch)
cards_affected: 10 fully clean + 1 partial-correctness upgrade (Reforge the Soul)
started: 2026-07-10
phase: implement (complete — pending review)
plan_file: memory/primitives/pb-plan-AC9.md
recon_file: memory/primitives/pb-recon-AC9.md

## Scope decision (confirmed from plan, verified again during implementation)

Three of five briefed primitives already existed and were correctly dropped from
scope by the plan: `Effect::RollDice` (d20 + results table), `ReplacementModification::DoubleTokens`
(token doubling), `Effect::AddManaFilterChoice` (multi-output filter mana, M10-deferred).
`SearchLibrary` multi-name dropped (zero roster). Confirmed both by grep and by using
these primitives directly in the card-def backfill (RollDice in 3 dragon defs;
DoubleTokens in 3 enchantment defs) — no re-implementation occurred.

## Implementation progress (2026-07-10)

- [x] `Effect::WheelHand { player, disposal, draw }` + `WheelDisposal`
      (Discard / ShuffleHandIntoLibrary / ShuffleHandAndGraveyardIntoLibrary) +
      `WheelDraw` (ThatMany / Fixed(u32)) — `cards/card_definition.rs` enum defs;
      dispatch in `effects/mod.rs` (snapshots hand size BEFORE disposal); new
      helper `move_zone_all_then_shuffle()` for the two shuffle dispositions
      (mirrors the existing `Effect::Shuffle` seed_from_u64(timestamp_counter)
      pattern). Routes Discard through the existing `discard_cards()` helper
      (Madness → exile preserved). Exported from `cards/helpers.rs`.
- [x] `Effect::SetNoMaximumHandSize { player }` + persistent
      `PlayerState.no_max_hand_size_permanent: bool` — `state/player.rs` field;
      `state/builder.rs:257` literal updated; dispatch in `effects/mod.rs`;
      cleanup recompute in `rules/turn_actions.rs` changed from
      `ps.no_max_hand_size = has_no_max;` to
      `ps.no_max_hand_size = has_no_max || ps.no_max_hand_size_permanent;`
      (the `calculate_characteristics()` layer-correctness scan above it,
      fixed by PB-AC8, was left untouched — verified NOT regressed).
- [x] hash.rs: `Effect::WheelHand` (disc 91) + `Effect::SetNoMaximumHandSize`
      (disc 92) hash arms; `HashInto for WheelDisposal` + `HashInto for WheelDraw`
      sub-enum impls; `PlayerState.no_max_hand_size_permanent` hashed;
      `HASH_SCHEMA_VERSION` 35 → 36 with changelog block. Bulk-updated all 27
      test-file `35u8` sentinel assertions → `36u8` (hash.rs itself excluded —
      its own internal `35u8` discriminants for unrelated enums are untouched).
- [x] Card fixes (10 fully clean, matching plan's yield table exactly):
      `incendiary_command.rs` (mode 3 → WheelHand), `shattered_perception.rs`,
      `winds_of_change.rs`, `echo_of_eons.rs` (WheelHand family);
      `ancient_silver_dragon.rs` (Sequence[RollDice, SetNoMaximumHandSize]);
      `parallel_lives.rs`, `anointed_procession.rs`, `doubling_season.rs`
      (token + counter doubling replacement registration);
      `ancient_copper_dragon.rs`, `ancient_gold_dragon.rs` (RollDice → CreateToken
      with `EffectAmount::LastDiceRoll`).
- [x] `reforge_the_soul.rs` — PARTIAL correctness upgrade only: swapped the
      wrong-game-state `DiscardCards{Fixed(7)}` for `WheelHand{Discard, Fixed(7)}`
      (now discards the WHOLE hand, not exactly 7). Miracle TODO deliberately
      retained — card stays PARTIAL, NOT counted as clean.
- [x] Token-doubling completeness pass (§5, separate scope from the two new
      primitives) — `apply_token_creation_replacement()` was wired at only 2 of
      13 actual `GameEvent::TokenCreated` emission sites (verified by grep, not
      "~13" from the plan — exactly 13). Wired the remaining 9 (plan's table
      named 8 classes across `resolution.rs`/`effects/mod.rs`; a systematic final
      grep sweep found 2 more the plan's table missed — `Effect::Investigate`
      and `Effect::Amass` — both wired too, each with its own regression test):
      `CreateTokenAndAttachSource` (Living Weapon), Squad (batch-level doubling),
      Offspring (single-instance), Myriad (per-opponent-instance), Embalm,
      Eternalize, Encore (per-opponent-instance), Gift Food/Treasure (keyed to
      **recipient**, not the gifting spell's controller — the exact subtlety the
      plan flagged), Investigate (per-instance), Amass (single-instance; doubled
      Army token created, only the FIRST receives the amass counters — a design
      call I made by analogy to the existing Living Weapon "only the first gets
      equipped" precedent comment, NOT verified against a specific CR ruling for
      the Amass+Doubling-Season interaction; flagged for reviewer attention).
      Plan's original table mislabeled one site as "Populate" — no `Populate`
      keyword/mechanic exists anywhere in the engine; that site is actually Squad.
- [x] Unit tests: new file `crates/engine/tests/pb_ac9_wheel_and_misc.rs` (18
      tests: 7 WheelHand, 4 SetNoMaximumHandSize incl. mutation-verified hash
      test, 1 hash sentinel, 6 card integration). Plus 7 token-doubling
      completeness regression tests added to existing keyword test files:
      `squad.rs` (+1), `myriad.rs` (+1), `living_weapon.rs` (+1), `gift.rs` (+2,
      including the negative control proving recipient-vs-giver controller
      keying), `investigate.rs` (+1), `amass.rs` (+1). Total: 25 new tests.
- [x] Gates — ALL GREEN:
      `cargo build --workspace` clean (confirms no TUI/replay-viewer arms needed —
      the plan's positive assertion held, verified: neither file matches on
      `Effect::`, only `StackObjectKind`/`KeywordAbility`).
      `cargo test --all`: 3062 → 3087 (+25), 0 failed.
      `cargo clippy --all-targets -- -D warnings`: clean.
      `cargo fmt --check`: clean (after 2 `cargo fmt` auto-fix passes for the
      manually-inserted loop-wrapping code in `resolution.rs` and 2 new test files).
      `python3 tools/authoring-report.py`: clean 973 → 982 (55.7% → 56.2%),
      matches plan's expectation of +9 clean, +1 partial→partial exactly.
      Zero remaining `TODO`/`ENGINE-BLOCKED` markers in the 10 clean cards
      (verified by grep); Reforge the Soul retains exactly 1 (Miracle).

## Deviations / findings for the reviewer

1. **Gotcha discovered (test-writing, not engine)**: `next_object_id()` shares
   the SAME counter as `timestamp_counter` (dice-roll/shuffle seeding). A test
   that builds many pre-placed objects via `GameStateBuilder` and THEN forces
   `state.timestamp_counter` to a small fixed value (to control a dice roll)
   will collide new object ids with existing object ids, silently corrupting
   the state (`OrdMap::insert` overwrite). Hit this in
   `test_ancient_silver_dragon_draw_and_no_max` (20 library objects + forced
   `timestamp_counter = 9` → only 6/10 draws succeeded before silent
   corruption). Fixed by forcing a value well ABOVE the post-build object
   count instead (`1009`, chosen so `1009 % 20 == 9` for the same roll).
   Recommend adding this to `memory/gotchas-infra.md` if not already covered.
2. **Amass + Doubling Season interaction**: no MCP CR/ruling lookup was
   performed for this specific interaction (Amass wasn't in the plan's scope
   at all — found via a final completeness sweep). Implemented as "doubled
   Army token creation, counters go on the first only," by analogy to the
   Living Weapon precedent already in the codebase. **Flag for review** —
   verify against the actual Amass + Doubling Season ruling if one exists.
3. **Plan's site-table label was wrong** for the `resolution.rs:4739`-ish
   site: labeled "Populate (SOK)" — no `Populate` keyword/mechanic exists in
   this engine at all (grep-confirmed). That site is Squad's ETB trigger. Not
   a blocker — Squad was wired regardless, just noting the plan's minor label
   error (consistent with the plan's own admission that it overturned the
   recon's table once already).
4. **Two additional token-doubling sites found beyond the plan's table**:
   `Effect::Investigate` (effects/mod.rs) and `Effect::Amass` (effects/mod.rs).
   The plan said "~13" `TokenCreated` sites; the table only named 10 (2
   pre-wired + 8 to wire). A final grep-count found exactly 13 total sites —
   these 2 were the gap. Wired both, each with a regression test, since they
   were low-risk (same `apply_token_creation_replacement` chokepoint pattern)
   and genuinely in-scope for "route every applicable site" per the task brief.

## Task reference

- Branch: feat/pb-ac9-misc-mana
- No ESM task ID recorded in this worktree's context at session start.

## Gates

- cargo build --workspace
- cargo test --all
- cargo clippy --all-targets -- -D warnings
- cargo fmt --check
- python3 tools/authoring-report.py → clean coverage delta posted above

## Commit prefix

W6-prim:
