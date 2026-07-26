# Primitive WIP — PB-DP2 (DP-2 · the mulligan is a content no-op) · PLAN

<!-- last_updated: 2026-07-26 -->

- **PB**: PB-DP2 — a mulligan must actually shuffle and `cards_to_bottom` must land on
  the library **bottom**. Both halves are **CR 103.5** (one sentence), with 103.5c
  supplying the multiplayer free-first-mulligan adjustment. The task title and criterion
  5519 cite "CR 103.4b" — that is stale; live CR 103.4b is the Vanguard starting life
  total. Verified against the live CR twice (plan + review).
- **Task**: `scutemob-150`
- **Branch**: `feat/pb-dp2-mulligan-is-a-content-no-op-bottomed-cards-go-to-libr`
- **Class**: CORRECTNESS (live-wrong, core-reachable — no card required; Tier 0)
- **Phase**: fix — COMPLETE. Review cycle closed 2026-07-26: 0 HIGH, 1 MEDIUM (filed as
  seed `OOS-DP2-7`), 6 LOW (2 filed as seeds `OOS-DP2-8` and an `OOS-DP2-4` addendum,
  2 doc/comment fixes applied, 1 declined-with-reason, 1 audit §4.4 cite fix). See
  `memory/primitives/pb-review-DP2.md` "Fix-cycle dispositions" for the full table.
- **Binding spec**: `docs/audits/decision-point-audit.md` §5 (DP-2 row, line ~429),
  §7 (OOS-M11-1 assessment, line ~495), §8 (PB-DP2 row, line ~561), §9 re-audit
  trigger "After a `Zone` API change" (line ~744)
- **Plan file**: `memory/primitives/pb-plan-DP2.md`
- **Review file**: `memory/primitives/pb-review-DP2.md`

## The two defects (as filed by the audit)

**(a) `handle_keep_hand` bottoms to the TOP.** `crates/engine/src/rules/commander.rs`
`handle_keep_hand` (~`:886-890`) moves each `cards_to_bottom` entry with
`state.move_object_to_zone(obj_id, ZoneId::Library(player))`. `Zone::insert` on an
ordered zone is `push_back` (`crates/card-types/src/state/zone.rs:109`) and
`Zone::top()` is `v.last()` (`:159-164`) — so the cards a player "bottoms" during the
London mulligan are the next cards they draw. `Zone::push_front` (`:187`) is the bottom
end, and `GameState::move_object_to_bottom_of_zone`
(`crates/engine/src/state/mod.rs:1610`, uses `push_front` at `:1792`) already exists as
the correct helper — `rules/copy.rs:484` cites it for the cascade bottom-write
(MR-M9.4-08). This is the same top/bottom inversion class PB-RS1 swept; the mulligan
was simply not in that sweep's roster.

**(b) `handle_take_mulligan` never permutes.** Same file, `~:808-848`: hand objects are
moved to the library (landing on top, in hand order), a `GameEvent::LibraryShuffled` is
pushed with **no permutation performed**, and 7 cards are drawn back off the top — so the
same seven cards return, reversed. Two problems: the CR 103.5 shuffle does not happen,
and the emitted event is a phantom (Architecture Invariant 4: "events are the single
source of truth for what happened").

## Wire expectation — **NO PROTOCOL bump, NO HASH bump** (PROTOCOL 27 / HASH 63)

The §8 PB-DP2 row predicts "(b) needs a seed on `GameState` ⇒ **HASH bump**". §7
supersedes that: the engine already has a deterministic seeded PRNG pattern seeded from
the **existing** `state.timestamp_counter` field —
`effects/mod.rs:8697-8703` (`move_zone_all_then_shuffle`), also `:3049`, `:3148`:

```rust
// MR-M7-17: seed from timestamp_counter (not entropy) for deterministic replay.
let seed = state.timestamp_counter;
state.timestamp_counter += 1;
if let Some(zone) = state.expect_zone_mut(&lib_zone) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    zone.shuffle(&mut rng);
}
```

Reusing that pattern adds **no new `GameState` field**, no `Command` field, no `Effect`
variant, no `GameEvent` variant. Therefore **neither fingerprint moves**. If the plan
concludes a schema fingerprint must be re-pinned — and especially if a **PROTOCOL** bump
appears needed — **STOP AND RE-SCOPE** (explicit task directive).

## Acceptance criteria (ESM `scutemob-150`)

1. (5519) `handle_keep_hand` puts `cards_to_bottom` on the library **BOTTOM**
   (`push_front`), with a test asserting **library position**, citing CR 103.4b.
2. (5520) `handle_take_mulligan` performs a **real seeded permutation** of the library;
   `LibraryShuffled` is no longer phantom; a test proves the drawn hand **can differ**
   and that the permutation is **deterministic per seed**, citing CR 103.5.
3. (5521) OOS-M11-1 marked closed in its seed inventory; audit DP-2 row and PB-DP2 row
   updated.
4. (5522) `cargo test --all`, `clippy -D warnings`, `fmt` + `tools/check-defs-fmt.sh`
   all clean; wire expectation honored.

## Existing test surface

`crates/engine/tests/rules/commander.rs:1400-1495` — the mulligan tests assert hand
counts and event shapes only; they never look at library position or library order.
New probes must fail-before / pass-after.

## Coordination note

M11-local Session 2 routes *around* (b) with a pregame `redeal`. Coordinate, do not
block on it — this PB fixes the engine path regardless.
