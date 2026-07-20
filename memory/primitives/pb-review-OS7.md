# Primitive Batch Review: PB-OS7 — Defending-player-scoped continuous EffectFilter

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 508.4 (defending player), CR 611.2a/611.2c/611.2f (continuous effect from a
resolving ability), CR 613.1c/613.4c (Layer 7c P/T), CR 514.2 (until-end-of-turn cleanup),
CR 704.5f (0-toughness SBA), CR 205.3m (Dragon subtype). Ruling: Silumgar 2014-11-24.
**Engine files reviewed**:
- `crates/card-types/src/state/continuous_effect.rs` (new `EffectFilter` variant)
- `crates/engine/src/effects/mod.rs` (`ApplyContinuousEffect` substitution arm)
- `crates/engine/src/rules/layers.rs` (`filter_matches` guard)
- `crates/engine/src/rules/abilities.rs` (defending-player capture, trigger-filter enforcement)
- `crates/engine/src/state/hash.rs` (HashInto + HASH 58→59 + epoch)
- `crates/engine/src/rules/protocol.rs` (PROTOCOL 21→22 + history)
- `crates/engine/tests/core/protocol_schema.rs`, `crates/engine/tests/core/hash_schema.rs` (sentinels + frozen-prefix)
**Card defs reviewed**: 1 — `crates/card-defs/src/defs/silumgar_the_drifting_death.rs`
**Tests reviewed**: `crates/engine/tests/primitives/pb_os7_defending_player_continuous_filter.rs` (9 tests)

## Verdict: needs-fix

No HIGH findings. The primitive, its substitution, the wire bumps, and the Silumgar card def are
all correct and internally consistent with the engine's established patterns; all 9 tests probe by
execution, cite CR, and use non-vacuous decoys. The card is legitimately `Complete` and ships.
Three findings remain, none ship-blocking: (1) **MEDIUM** — a genuine CR 611.2c divergence in how
the locked `CreaturesControlledBy(pid)` filter re-evaluates membership live rather than locking the
affected *set* at resolution; this is a **pre-existing, engine-wide limitation** shared by every
resolution-generated mass P/T effect (Golgari Charm, Eyeblight Massacre, Infest-style wipes), so
PB-OS7 correctly follows precedent — BUT the plan's design note actively mischaracterizes the live
behavior as CR-correct, which must be corrected and tracked as a seed rather than absorbed as fact.
(2) **LOW** — both `FROZEN_HISTORY_PREFIX_DIGEST` explanatory comments are stale (still describe the
OS6 bump). (3) **LOW** — no negative test for a non-Dragon attacker / planeswalker-attack path.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `effects/mod.rs:3181` (+ `layers.rs:646`) | **CR 611.2c affected-set is not locked at resolution.** The `CreaturesControlledBy(pid)` layer arm re-checks `o.controller == pid` live every `calculate_characteristics`, so a creature entering the defending player's control after the trigger resolves is wrongly debuffed and a creature leaving that player's control wrongly loses the −1/−1. CR 611.2c fixes the *set of objects*, not just the player. **Pre-existing systemic limitation** (Golgari Charm / Eyeblight Massacre use the same live filters); do NOT engine-fix in this PB. **Fix:** correct the plan's false "CR-correct" claim (see Details) and file a tracking seed (OOS) for resolution-time set-snapshot semantics for the whole class. |
| 2 | LOW | `tests/core/protocol_schema.rs:145-153`, `tests/core/hash_schema.rs:181-187` | **Stale frozen-prefix comments.** Both `FROZEN_HISTORY_PREFIX_DIGEST` comment blocks still describe the OS6 bump (protocol "20→21", prefix ending v20; hash "57→58", v57 joining). After the OS7 bump v21/v58 have joined their frozen prefixes. The digest *values* were re-pinned (WIP attests it; the `frozen_prefix` gates are in `cargo test --all`, reported green), only the prose is stale. **Fix:** update both comment blocks to the OS7 bump (protocol 21→22, v21 joins prefix `[2..21]`; hash 58→59, v58 joins). Re-confirm the two `frozen_prefix` gates are green — if either is red the values were NOT re-pinned and this escalates to HIGH. |

### Engine changes verified correct (no finding)

- **Placeholder variant** (`continuous_effect.rs:142-154`): unit variant `CreaturesControlledByDefendingPlayer`
  added after `TriggeringCreature`, doc-commented as a DSL-only placeholder that never appears in a
  stored `ContinuousEffect`. Correct, mirrors `Source`/`TriggeringCreature`.
- **Substitution / footgun** (`effects/mod.rs:3181-3184`): `Some(pid) => CreaturesControlledBy(pid)`,
  `None => return`. **The controller-fallback footgun is genuinely avoided** — no `unwrap_or(ctx.controller)`,
  no `PlayerId(0)` sentinel. Traced `ctx.defending_player`: populated per-attacker at
  `abilities.rs:4105-4113`; a `Player(pid)` attack yields `Some(pid)`, a **planeswalker attack yields
  `Some(pw.controller)`** (`abilities.rs:4107-4109`) — CR 508.4 correct, so attacking a planeswalker
  does NOT wrongly skip the debuff. `None` only occurs off the attack-trigger path (never for Silumgar),
  where skipping is correct. `PlayerId(0)` cannot leak the controller here.
- **Layer guard** (`layers.rs:668`): `=> false` unreached guard present in the exhaustive `filter_matches`.
- **Trigger subtype filter honored** (`abilities.rs:6403-6412`): the `WheneverCreatureYouControlAttacks { filter }`
  Dragon filter is applied via `matches_filter(&attacking_chars, creature_filter)` on the attacker's
  layer-resolved characteristics — a non-Dragon attacker will NOT trigger. Correct (untested negatively — see Finding 3).
- **HASH bump complete**: discriminant 36 (`hash.rs:2167`), `HASH_SCHEMA_VERSION = 59` (`:531`),
  `- 59:` history doc line (`:523-530`), v59 `HashSchemaEpoch` row with both fingerprints re-pinned
  (`:787-794`), ~40 test sentinels swept (0 stale `HASH_SCHEMA_VERSION, 58` remain).
- **PROTOCOL bump correct and correctly justified**: the plan predicted NO bump; the machine gate
  forced 21→22. The runner **correctly stopped-and-flagged** (WIP §PROTOCOL DEVIATION) rather than
  silently absorbing, and correctly diagnosed the root cause — PB-EF9 (v14) pulled `ContinuousEffectDef`
  (and thus its sibling `EffectFilter`) into the wire closure via `Effect::ApplyContinuousEffect`, so
  the PB-EF4-era "EffectFilter off the wire closure" note went stale. `PROTOCOL_VERSION = 22`,
  fingerprint re-pinned, v22 `PROTOCOL_HISTORY` row appended (`protocol.rs:408-413`, fingerprint matches
  the const), `protocol_version_sentinel` = 22, 5 test sentinels swept. The correction is documented
  inline at `protocol.rs:196-212`. This is exactly the required behavior.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | LOW | `silumgar_the_drifting_death.rs` | **Test-coverage gap (not a def defect).** The def is oracle-perfect. Missing negative tests: (a) a non-Dragon creature you control attacking does NOT trigger the −1/−1 (Dragon subtype filter); (b) attacking a planeswalker scopes to the PW's controller. Both mechanisms are verified correct in source and share PB-N's tested path. **Fix (optional):** add the two tests. |

### Card def verified correct (no blocking finding)

Oracle (MCP 2026-07-19): "{4}{U}{B} Legendary Creature — Dragon 3/7. Flying, hexproof. Whenever a
Dragon you control attacks, creatures defending player controls get -1/-1 until end of turn."
Def matches exactly: `full_types(Legendary, Creature, Dragon)`, `power 3 / toughness 7`,
`ManaCost { generic 4, blue 1, black 1 }`, `Keyword(Flying)` + `Keyword(Hexproof)`, per-Dragon
`WheneverCreatureYouControlAttacks { filter: has_subtype Dragon }` →
`ApplyContinuousEffect(ModifyBoth(-1), CreaturesControlledByDefendingPlayer, UntilEndOfTurn)`.
`completeness: Complete`, no TODOs, no gated stubs, no barred `Choose`/`MayPayOrElse`/`AddMana*`.
Silumgar is itself a Dragon so it triggers off its own attack (proven by test 1). Karazikar correctly
NOT authored; OOS-OS7-1 filed in the plan.

### Finding Details

#### Finding 1: CR 611.2c affected-set is not locked at resolution (MEDIUM, pre-existing/systemic)

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:3181` (substitution) → `crates/engine/src/rules/layers.rs:646` (live re-eval)
**CR Rule**: 611.2c — "If a continuous effect generated by the resolution of a spell or ability
modifies the characteristics ... of any objects, the set of objects it affects is determined when
that continuous effect begins. After that point, the set won't change. (Note that this works
differently than a continuous effect from a static ability.)"
**Issue**: Silumgar's −1/−1 is a continuous effect generated by the RESOLUTION of a triggered ability
that modifies characteristics (Layer 7c). Per CR 611.2c its affected *set of creatures* must be locked
at resolution. The implementation instead stores `CreaturesControlledBy(pid)` and the layer arm
(`layers.rs:646-654`) re-evaluates `o.controller == pid` live at every characteristics calculation,
which only locks the PLAYER, not the SET. Reachable wrong game states: (a) the defending player
flashes in / is given a creature after the trigger resolves → it is wrongly debuffed (and may wrongly
die to SBA); (b) a debuffed creature changes controller away from the defending player → it wrongly
loses the −1/−1 (CR 611.2c says it keeps it). The plan's design note (`pb-plan-OS7.md:95-101`,
`:383-386`) asserts the opposite — "CR 611.2c only fixes the *player*, not the membership ... affects
the set that matches the description at each moment." **That is a misreading of CR 611.2c** and
violates the standing verify-CR-before-implement discipline (`memory/feedback_verify_cr_before_implement.md`).
**Context / why MEDIUM not HIGH**: This is NOT introduced by PB-OS7. Every resolution-generated mass
P/T effect in the corpus uses the same live-membership filters and has the same divergence — verified:
`golgari_charm.rs:32-40` (`AllCreatures`), `eyeblight_massacre.rs:19-27` (`AllCreaturesExcludingSubtype`).
The engine's `ApplyContinuousEffect`+`UntilEndOfTurn` model has no resolution-time set-snapshot
mechanism. PB-OS7 correctly follows precedent; a real fix is an engine-wide subsystem (new snapshot
filter capturing the object-id set at resolution) and is out of scope per implement-phase-default-to-defer.
The 9 shipped tests are all valid (static boards, no mid-turn control change) — no test is wrong.
**Fix**: (1) Do NOT engine-fix in this PB. (2) Correct the plan's design note and any "CR-correct"
justification so it reads as a known CR 611.2c divergence, not correct behavior (per conventions.md
"aspirationally-wrong comments are correctness hazards"). The card header comment cites only 611.2a and
does not claim set-locking, so it is acceptable, but add one line noting the entering/leaving-control
edge case is a tracked 611.2c limitation. (3) File a tracking seed (e.g. OOS-OS7-3) for resolution-time
affected-set snapshot semantics covering the whole class of resolution-generated P/T effects.

> ℹ️ **Resolution (verified `scutemob-142`, 2026-07-19):** this step's content was filed as
> **OOS-OS7-2**, not OOS-OS7-3 — `pb-plan-OS7.md:321` had concurrently renumbered the unrelated
> `CreaturesControlledByTargetPlayer` note into `OOS-OS7-3`, so the ID was double-proposed. The
> 611.2c seed is canonical at `oos-retriage-plan-2026-07-18.md:424-444`. **`OOS-OS7-3` was never
> formally filed and must not be reused**; the orphaned filter note is refiled as **OOS-RS-5**.
> See `memory/primitives/rider-seed-triage-2026-07-19.md` §1b.

#### Finding 2: Stale frozen-prefix digest comments (LOW)

**Severity**: LOW
**File**: `crates/engine/tests/core/protocol_schema.rs:145-153`; `crates/engine/tests/core/hash_schema.rs:181-187`
**Issue**: Both `FROZEN_HISTORY_PREFIX_DIGEST` blocks still narrate the OS6 re-pin (protocol "20→21",
prefix `[version 2 ... version 20]`; hash "57→58", "version 57 ... joined the frozen prefix"). After
OS7's 21→22 and 58→59 bumps, versions 21 and 58 have joined their respective frozen prefixes, so the
prose is factually wrong about which prefix each digest covers. The runner attests (WIP §PROTOCOL
DEVIATION) that the protocol frozen-prefix value was re-pinned, and both `frozen_prefix` gates are part
of `cargo test --all` (reported 3547 green), so the *values* are correct — only the comments lag.
**Fix**: Update both comment blocks to describe the OS7 bump (protocol: 21→22, v21 joins prefix `[2..21]`;
hash: 58→59, v58 joins prefix `[..58]`). Before editing, re-run the two `frozen_prefix` gates to confirm
green; if either is RED the digest value was never re-pinned and this escalates to HIGH (re-pin from the
gate's printed value).

#### Finding 3: Missing negative tests (LOW)

**Severity**: LOW
**File**: `crates/engine/tests/primitives/pb_os7_defending_player_continuous_filter.rs`
**Issue**: All attackers in the suite are Dragons, and all attacks target players. No test proves
(a) a non-Dragon creature you control attacking does NOT trigger the −1/−1 (the `has_subtype Dragon`
trigger filter), nor (b) a Dragon attacking a planeswalker scopes the −1/−1 to that planeswalker's
controller. Both are verified correct in source (`abilities.rs:6403-6412` filter enforcement;
`abilities.rs:4107-4109` planeswalker→controller capture) and share PB-N's tested machinery, so risk
is low. Plan §Risks acknowledges the planeswalker path is untested.
**Fix (optional)**: Add a non-Dragon-attacker negative test and a planeswalker-attack scoping test.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 508.4 (defending player, incl. PW→controller) | Yes | Partial | Player defender tested (all tests); PW path verified in source, untested (Finding 3) |
| 611.2a (continuous effect from resolving ability) | Yes | Yes | tests 1-4 |
| 611.2c (affected-set locked at resolution) | **No (divergence)** | No | Finding 1 — live membership; pre-existing systemic limitation |
| 613.4c (Layer 7c −1/−1) | Yes | Yes | `calculate_characteristics` reads in every test |
| 514.2 (until-end-of-turn expiry) | Yes | Yes | test_os7_until_end_of_turn_expiry |
| 704.5f (0-toughness SBA) | Yes | Yes | test_os7_toughness_death_sba_defender_vs_bystander |
| 205.3m (Dragon subtype trigger filter) | Yes | Partial | positive only; non-Dragon negative untested (Finding 3) |
| Ruling 2014-11-24 (per-Dragon / per-defender scope) | Yes | Yes | tests 3 (same-defender −2/−2) + 4 (different-defender independent scope) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| silumgar_the_drifting_death | Yes (exact) | 0 | Yes* | *Correct except the engine-wide CR 611.2c entering/leaving-control edge case (Finding 1); legitimately `Complete` |

## Wire Bump Completeness

| Item | Present | Correct |
|------|---------|---------|
| `EffectFilter` variant (discriminant 36) | Yes | Yes |
| HASH_SCHEMA_VERSION 58→59 | Yes | Yes |
| HASH `- 59:` history line | Yes | Yes |
| HASH v59 epoch row (both fingerprints) | Yes | Yes |
| HASH test sentinels (~40) swept | Yes | Yes (0 stale) |
| PROTOCOL_VERSION 21→22 (machine-forced) | Yes | Yes — correctly stopped-and-flagged per plan |
| PROTOCOL fingerprint re-pin | Yes | Yes (matches v22 history row) |
| PROTOCOL v22 history row | Yes | Yes |
| PROTOCOL sentinels (5) swept | Yes | Yes |
| FROZEN_HISTORY_PREFIX_DIGEST re-pinned (both) | Value: yes / Comment: **stale** | Finding 2 |

## Notes for the runner

- **No HIGH; the card ships.** The primitive, substitution, footgun avoidance, wire bumps, card def,
  and 9 tests are correct. The PROTOCOL deviation handling was exemplary (flagged, not silently absorbed).
- **Do NOT attempt to fix Finding 1 in this PB.** It is a pre-existing engine-wide limitation. The
  in-scope action is: correct the plan's CR-611.2c mischaracterization + file a tracking seed. Hacking a
  snapshot filter here would be an out-of-scope micro-PB (implement-phase-default-to-defer).
- Finding 2 is a 2-comment edit + a green-gate re-confirmation. Finding 3 is optional test hardening.
