# Primitive WIP — PB-DP1 (DP-1 · priority after cast/activate/special action) · FIX CYCLE COMPLETE

<!-- last_updated: 2026-07-26 -->

- **PB**: PB-DP1 — priority after casting a spell, activating an ability, or taking a
  special action goes to the **actor**, not the active player (CR 117.3c / 116.3)
- **Task**: `scutemob-149`
- **Branch**: `feat/pb-dp1-priority-after-castactivatespecial-action-goes-to-the`
- **Class**: CORRECTNESS (live-wrong, core-reachable — no card required)
- **Phase**: FIX — review findings applied (0 HIGH / 3 MEDIUM / 8 LOW, all 11
  dispositioned below). Implement phase (plan steps 1-14) was complete before this
  cycle; this cycle also completed plan step 16 (audit doc close-out, seeds filed) as
  directed by review LOW 11. Full gates re-run clean: `cargo test --all --no-fail-fast`
  3,721/3,721 passed (baseline 3,713 + 8 net-new fix-cycle probes), `cargo clippy
  --workspace --all-targets -- -D warnings` clean, `cargo build --workspace` clean,
  `cargo fmt --check` clean, `tools/check-defs-fmt.sh` clean (1,804 defs). PROTOCOL 27 /
  HASH 63 confirmed unchanged (`protocol_schema`/`hash_schema` gates green).
- **Binding spec**: `docs/audits/decision-point-audit.md` §4.1, §4.12, §5 Tier 0 (DP-1 row), §8 (PB-DP1 row)
- **Plan file**: `memory/primitives/pb-plan-DP1.md`
- **Review file**: `memory/primitives/pb-review-DP1.md`
- **Wire expectation**: **NO PROTOCOL bump, NO HASH bump** (PROTOCOL 27 / HASH 63 unchanged).
  This PB changes *which `PlayerId` is written into the existing `Turn::priority_holder`
  field*. No `Command` field, no `Effect` variant, no `GameEvent` variant, no shape change.
  **If a schema fingerprint must be re-pinned, STOP and re-scope** (explicit task directive).

## CR text — verified via mtg-rules MCP 2026-07-26 (authoritative; do not re-derive)

- **CR 117.3a** — "The active player receives priority at the beginning of most steps and
  phases, after any turn-based actions … have been dealt with and abilities that trigger at
  the beginning of that phase or step have been put on the stack."
- **CR 117.3b** — "The active player receives priority after a spell or ability (other than
  a mana ability) resolves."
- **CR 117.3c** — "If a player has priority when they cast a spell, activate an ability, or
  take a special action, **that player** receives priority afterward."
- **CR 117.3d** — passing: the next player in turn order receives priority.
- **CR 116.3** — "If a player takes a special action, that player receives priority afterward."
- **CR 601.2i** (actual text) — "Once the steps described in 601.2a–h are completed, effects
  that modify the characteristics of the spell as it's cast are applied, then the spell
  becomes cast. Any abilities that trigger when a spell is cast or put onto the stack trigger
  at this time. **If the spell's controller had priority before casting it, they get priority.**"
- **CR 602.2** has exactly two subrules, **602.2a** and **602.2b**. **CR 602.2e DOES NOT
  EXIST.** 602.2b is the correct citation: "The remainder of the process for activating an
  ability is identical to the process for casting a spell listed in rules 601.2b–i."
- **CR 116.3b DOES NOT EXIST either** — CR 116.3 has no subrules. Several comments in
  `resolution.rs` and `abilities.rs` cite it. Third bogus citation, found by this sweep;
  fold it into the citation fix.

## Sweep result — all 34 `priority_holder =` assignments in `crates/engine/src/`

`grep -rn "priority_holder\s*=" crates/engine/src/` → 34 sites across 6 files
(engine.rs 13, abilities.rs 12, resolution.rs 4, turn_structure.rs 2, combat.rs 2,
casting.rs 1). Classified below. **The audit's roster is a starting point and contains at
least two false positives** (Group C).

### Group A — MUST CHANGE: actor receives priority (CR 117.3c / 601.2i / 602.2b / 116.3)

| site | context | actor |
|---|---|---|
| `rules/casting.rs:4715` | cast a spell | spell's controller/caster |
| `rules/abilities.rs:1387` | activate ability (generic) | activating player |
| `rules/abilities.rs:1552` | activate (2) | activating player |
| `rules/abilities.rs:1753` | activate (3 — forecast region) | activating player |
| `rules/abilities.rs:1967` | activate (4) | activating player |
| `rules/abilities.rs:2102` | activate (5) | activating player |
| `rules/abilities.rs:2341` | activate (6 — ninjutsu region) | activating player |
| `rules/abilities.rs:2504` | activate (7) | activating player |
| `rules/abilities.rs:2681` | activate (8) | activating player |
| `rules/abilities.rs:2857` | activate (9) | activating player |
| `rules/abilities.rs:8791` | activate (crew region) | activating player |
| `rules/abilities.rs:9000` | activate (saddle region) | activating player |
| `rules/abilities.rs:9202` | activate (12) | activating player |
| `rules/engine.rs:1461` | `handle_activate_craft` — craft is an **activated ability** (CR 702.167a) | activating player |

Every one of these already has the actor's `PlayerId` in scope (the command carries it) —
verify per-site; do not assume.

### Group B — JUDGEMENT CALL, planner must rule (cost payments; not in CR 117.3's list)

| site | enclosing fn | note |
|---|---|---|
| `rules/engine.rs:757` | echo payment (before `handle_pay_cumulative_upkeep` at :768) | Paying echo is not a cast, an activation, or a special action. CR grants no priority for it. Decide: actor-by-analogy, or leave AP with an honest comment. Either way the citation must not claim a rule that doesn't say it. |
| `rules/engine.rs:958` | `handle_pay_cumulative_upkeep` | same class |
| `rules/engine.rs:1072` | `handle_pay_recover` | same class |

### Group C — CORRECT AS-IS, DO NOT CHANGE (CR 117.3a / 117.3b / 117.3d)

| site | context | why correct |
|---|---|---|
| `rules/engine.rs:1759`, `:1805` | `enter_step` | CR 117.3a — AP gets priority at step start. **Audit listed both; both are false positives.** |
| `rules/combat.rs:1373` | `handle_declare_blockers` | Declaring blockers is a **turn-based action** (CR 509.1); CR 117.3a gives AP priority after it. **Audit listed it; false positive.** |
| `rules/combat.rs:680` | `handle_declare_attackers` | already `Some(player)`; the declaring player IS the AP (CR 508.1). Already correct. |
| `rules/resolution.rs:114` | fizzle | CR 117.3b |
| `rules/resolution.rs:7744` | after resolution | CR 117.3b |
| `rules/resolution.rs:7983` | after countering | CR 117.3b |
| `rules/resolution.rs:5175` | cipher free-cast **during resolution** | No player holds priority mid-resolution, so CR 117.3c's antecedent ("if a player has priority when they cast") is false. AP is right; only the bogus `CR 116.3b` citation needs fixing. |
| `rules/turn_structure.rs:103`, `:140` | step / turn advance | sets `None` |
| `rules/engine.rs:1636`, `:1641` | `handle_pass_priority` | CR 117.3d |
| `rules/engine.rs:1812`, `:1815`, `:1883`, `:1893` | `handle_concede` | priority reassignment on player loss |

### Group D — MISSING SITES: special actions that never touch priority at all (CR 116.3)

The sweep found **no** `priority_holder` write in these handlers. Under CR 116.3 the actor
should receive priority afterward (and `players_passed` should reset — a game action
occurred). Planner must rule in-scope vs. seed **per item**, with reasoning:

- `rules/lands.rs` — `PlayLand` (CR 116.2a)
- `rules/engine.rs:1469` `handle_turn_face_up` — turn a face-down creature face up (CR 116.2b)
- `rules/foretell.rs` — foretell (CR 116.2h)
- `rules/plot.rs` — plot (CR 116.2k)
- `rules/suspend.rs` — exile a suspend card (CR 116.2f)
- `rules/commander.rs` — `BringCompanion` (CR 116.2g)

## PRESERVE (explicit task directive)

**Mana abilities do NOT reset `players_passed` and must not disturb the priority holder.**
Documented, correct behaviour (CR 117.3b's parenthetical "other than a mana ability").
A regression test pins it.

## Expected fallout (explicit task directive)

Wide. Many engine tests and JSON golden scripts encode the old active-player behaviour.
Every updated test/script must **cite CR 117.3c**. The reviewer must confirm no script
silently retains the inversion (PB-RS1 precedent). Simulator / M11-local `LocalGame` parity
and trace baselines will move — expected and correct; update knowingly, with citation.

## Steps

- [x] 1. Plan phase (`primitive-impl-planner`) → `memory/primitives/pb-plan-DP1.md`
- [x] 2. Probe tests written FIRST, verified FAILING pre-fix
- [x] 3. Group A fixes: casting.rs + CR 601.2i citation
- [x] 4. Group A fixes: abilities.rs + CR 602.2e → 602.2b citation
- [x] 5. Group A fixes: engine.rs craft; Group B ruling applied; Group D disposition
- [x] 6. Regression tests (non-active caster; respond to own spell; mana-ability preservation)
      — covered by probes P1/P2/P7 in the Step-1 probe file (all green post-fix)
- [x] 7. Test + golden-script fallout triage, every update citing CR 117.3c — **DONE**
      this session (`scutemob-149`, plan steps 12-14). Full triage recorded below.
- [x] 8. Simulator / LocalGame baselines updated knowingly — **DONE**. No baseline
      needed rewriting; verified WHY by reading the two derive-from-`priority_holder`
      sites (see plan-step-14 section below).
- [x] 9. PROTOCOL 27 / HASH 63 confirmed unchanged (wire sentinels green); **full gate
      suite now run**: `cargo test --all --no-fail-fast` → 271/271 targets green, 0
      failures. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build
      --workspace`, `cargo fmt --check` deferred to the fix/review-close session per
      the task's scope (test + script triage only) — not run this session.
- [x] 10. `primitive-impl-reviewer` pass, findings dispositioned — **DONE**. 0 HIGH / 3
      MEDIUM / 8 LOW, verdict needs-fix. All 11 applied this fix cycle — see
      "Fix cycle" section below.
- [x] 11. Close-out: audit doc §5/§8 rows shipped; seeds filed — **DONE**. See "Fix
      cycle" section below (this doubles as plan step 16, per review LOW 11's
      directive to run it).

## Steps 12-14 triage session — scutemob-149 (test + golden-script fallout)

**Starting fallout** (from `cargo test --all --no-fail-fast` run at session start): 19 unit
test failures across `casting`, `core`, `mechanics_a_d`, `mechanics_e_l`, `mechanics_m_z`,
`primitives`, plus 15 golden-script failures in `run_all_approved_scripts` (1 corpus-wide
test). All 19 unit-test failures and all 15 script failures were **test-encoded-the-bug**
per the plan §9.4 rule — **zero real regressions**.

### Root-cause cluster: countered-spell tests/scripts (the dominant failure mode)

10 of 19 unit-test failures (`aftermath`, `buyback`, `flashback`, `jump_start`, `madness`,
`retrace`, `pb_ac5_alt_costs` ×2, `pb_ef2_create_token_recipient` ×3 — one file had 3
affected tests) plus 12 of 15 script failures share ONE root cause: a non-active player
casts a counterspell (or any instant) in response, and the test/script's hardcoded
`pass_all`/`priority_round` player list started with the ACTIVE player (the old,
buggy hand-off target) instead of the ACTUAL actor. Per CR 117.3c the actor now
correctly retains priority, so the first `PassPriority` in these lists failed with
`NotPriorityHolder`.

**Fix pattern** (verified empirically via debug prints on `aftermath::test_aftermath_exile_on_counter`
before generalizing — see evidence below): reorder the pass list to start with the
actor, then the other active player(s) in APNAP order. For a 2-object stack where the
top resolution removes BOTH objects (e.g., Counterspell resolving counters and removes
its target too), the correct 4-pass shape is `[actor, other, other, actor]` — NOT
`[actor, other, actor, other]` — because after the single resolution empties the whole
stack, CR 117.3b (Group C, untouched by PB-DP1) hands priority back to the ACTIVE
player, and the remaining two passes drain that now-empty-stack round starting from
the active player, not the actor.

**Evidence of the empirical verification** (not just theory): added temporary
`eprintln!` debug prints to `aftermath::test_aftermath_exile_on_counter`'s `pass_all`
loop, which showed `priority_holder` was correctly `Some(PlayerId(2))` right after the
Counterspell cast, then flipped to `Some(PlayerId(1))` after the SECOND pass in the
sequence (once the top-of-stack resolution completed and Group C's CR 117.3b handoff
fired) — confirming the `[p2, p1, p1, p2]` shape empirically, not just by inference.
Debug prints were removed before committing.

Fixed files (unit tests): `crates/engine/tests/mechanics_a_d/aftermath.rs`,
`crates/engine/tests/mechanics_a_d/buyback.rs`, `crates/engine/tests/mechanics_e_l/flashback.rs`,
`crates/engine/tests/mechanics_e_l/jump_start.rs`, `crates/engine/tests/mechanics_m_z/madness.rs`
(2-player, single-pass-per-side shape `[actor, other]`, not 4-pass — no second stack
object), `crates/engine/tests/mechanics_m_z/retrace.rs` (only the FIRST of two
`pass_all` calls needed reordering — the second, post-resolution round was already
correct since Group C restores AP priority), `crates/engine/tests/primitives/pb_ac5_alt_costs.rs`
(`test_force_of_negation_counters_and_exiles` — its OWN comment cited the pre-fix rule
verbatim: `"CR 601.2i: CastSpell resets priority to the ACTIVE player"` — direct textual
proof this test encoded the bug; `test_warp_countered_spell_not_exiled` — same shape),
`crates/engine/tests/primitives/pb_ef2_create_token_recipient.rs` (3 tests, all
`counter_scenario`-based, all fixed with the same `[p2, p1, p1, p2]` reorder),
`crates/engine/tests/primitives/pbt_up_to_n_targets.rs` (4-player variant —
`[p2, p3, p4, p1]` for the first round; the SECOND round in the same test was already
correct as `[p1, p2, p3, p4]` since it runs after a resolution, Group C), and
`crates/engine/tests/primitives/pb_ac4_per_mode_targeting.rs` (2 tests — only the FIRST
`pass_all` per test needed reordering to `[p2, p1]`; Modal Strike stays on the stack
after the response resolves, so the SECOND round is Group C, unaffected).

Fixed scripts (golden corpus, `test-data/generated-scripts/`): `tokens/001_swan_song_creates_bird.json`,
`baseline/019_pass_priority_with_stack_item.json`, `stack/030_counterspell_counters_wrath.json`,
`stack/002_counterspell_counters_spell.json`, `stack/010_negate_counters_noncreature.json`,
`stack/015_supreme_verdict_uncounterable.json`, `stack/045_swan_song_counters_damnation.json`,
`stack/044_negate_counters_harmonize.json`, `stack/043_two_spells_lifo_order.json`,
`stack/062_rancor_aura_attach_and_return.json`, `stack/006_arcane_denial_counters_spell.json`,
`stack/165_umbra_armor_hyena_umbra.json`, `stack/198_morph_face_down_creature_dies_reveal.json`,
`layers/081_bestow_aura_then_falls_off.json` — all fixed by reordering the single
`priority_round`'s `players` list to start with the actual actor, each with a new
`note` citing CR 117.3c. Any SECOND `priority_round` in the same script (post-resolution,
Group C) was left unchanged, with a clarifying-only comment added in a couple of cases
(`015`, `043`) to record why it didn't need reordering.

### The one script needing more than a reorder: `stack/066_krosan_grip_split_second_blocks_counterspell.json`

This script does not fit the simple reorder pattern. p2 casts Krosan Grip
(split second) in response to nothing — p2 is the actor, priority correctly stays
with p2 (CR 117.3c). The script's ORIGINAL sequence then has p1 (who does NOT hold
priority) immediately activate Sol Ring's mana ability. `rules/mana.rs::handle_tap_for_mana`
requires `state.turn.priority_holder == Some(player)` (CR 605.3b, unchanged by this PB
— PRESERVE) — a real, pre-existing engine gate, not something PB-DP1 introduced. Under
the pre-fix (buggy) engine, priority had incorrectly reverted to the active player p1
immediately after p2's cast, which is exactly why p1's mana-tap "worked" before. Under
the CR-correct fix, p1 does not hold priority at that point and genuinely cannot
activate a mana ability yet.

**Fix**: inserted an explicit `p2` `priority_pass` action between the Krosan Grip cast
and P1's mana-tap (p2 must actively hand priority to p1 before p1 can act), then
reduced the trailing `priority_round` from `[p1, p2]` to `[p1]` alone — since p2's
explicit pass already contributed to `players_passed` (CR 117.3b parenthetical: a mana
ability does not disturb `priority_holder` or reset `players_passed`), only p1's own
pass is needed to complete the all-pass round. This is still functionally a
test-encoded-the-bug fix (the old sequence relied on the bug to let p1 act), just one
that needed an inserted action rather than a bare reorder. Verified: script passes,
0 remaining script failures.

### Group C presumption discharge: `resolution::test_608_1_priority_goes_to_active_player_after_resolution`

Per the task's flagged concern, this test carries a presumption of being a real
regression (it asserts Group C / CR 117.3b behavior, which PB-DP1 does not touch).
**Presumption discharged, not a real regression**: the test's SETUP casts an instant as
p2 (a non-active-player cast) and then drives the stack to resolution via
`pass_all_four(state, [p1, p2, p3, p4])` — a hardcoded 4-player pass list that, pre-fix,
"worked" only because priority incorrectly reverted to the active player p1 right after
the cast. Post-fix, the actor p2 correctly retains priority (CR 117.3c), so the pass
list needed reordering to `[p2, p3, p4, p1]` (actor first, then APNAP wrap). The
test's actual ASSERTION — `final_state.turn().priority_holder == Some(p1)` AFTER all
four players pass and the stack resolves — is CR 117.3b (Group C) and is byte-identical
in meaning before and after this fix; it was never touched. Only the pass-sequence
SETUP needed to change, and only because of the (in-scope) Group A cast-priority fix
upstream of it, not because Group C's own logic moved. Sibling test
`test_608_1_instant_resolves_to_graveyard` in the same file had the identical shape and
received the identical fix (`[p1,p2,p3,p4]` → `[p2,p3,p4,p1]`).

### `casting.rs` cluster (test-encoded-the-bug, most direct)

`casting::test_cast_spell_instant_during_opponents_upkeep` and
`casting::test_cast_spell_priority_resets_to_active_player` both had p2 (non-active,
holding priority via a manually-seeded `state.turn_mut().priority_holder = Some(p2)`)
cast an instant and then assert `priority_holder == Some(p1)` afterward — the bug,
verbatim. Fixed both assertions to `Some(p2)`, cited CR 117.3c, and renamed the second
test from `test_cast_spell_priority_resets_to_active_player` to
`test_cast_spell_priority_retained_by_actor_after_casting` (the old name described the
bug's behavior as the intended one).

### Simulator / LocalGame (plan step 14) — verified, not just reported green

`cargo test -p mtg-simulator` was green with NO changes needed. Verified the plan's
claimed explanation by reading both sites directly (not just trusting the plan):
- `crates/simulator/src/legal_actions.rs:191-192`: `if state.turn().priority_holder !=
  Some(player) { return actions; }` — gates legal-action enumeration purely off
  `priority_holder`, with no active-player special-casing anywhere in the function.
- `crates/simulator/src/local_game.rs:310`: `else if let Some(priority) =
  self.state.turn().priority_holder { (priority, None) }` — derives the acting seat
  directly from `priority_holder` in `advance()`'s seat-resolution chain.

Both sites treat `priority_holder` as the single source of truth for "who acts next"
and contain no hardcoded assumption that it's the active player. This confirms the
plan's forecast: since PB-DP1 only changes WHICH `PlayerId` gets written into
`priority_holder` (not the field's meaning or type), both call sites automatically
followed the fix with zero code changes needed. `test_local_game_bot_only_matches_game_driver_for_fixed_seeds`
(the self-referential `GameDriver` vs `LocalGame` parity test) stayed green because
BOTH paths move identically under the fix (same underlying `process_command` logic).

### PRESERVE gate — confirmed intact

`test_dp1_mana_ability_does_not_reset_players_passed` (P7) was NOT touched and was
green throughout this session (confirmed in the probe re-runs during the implement
phase and never revisited here). No code in `mana.rs` changed during this triage
session — only script `066`'s ACTION SEQUENCE (adding an explicit p2 pass before p1's
mana tap) changed, never the mana-ability activation gate itself.

### Final tally

- **19 tests updated (bug-encoded) / 15 scripts updated (bug-encoded) / 0 real
  regressions.**
- Full suite: `cargo test --all --no-fail-fast` → **271/271 targets green, 0 failures**
  (271 = the workspace's full binary count including doc-tests; grepped for
  `FAILED`/`error[` in the captured log — zero hits).
- Golden scripts: `run_all_approved_scripts` → 211/271 discovered scripts ran and
  passed, 60 retired (pre-existing, unrelated to this PB — reasons unchanged), 0
  skipped silently.
- `cargo test -p mtg-simulator` → all green, 0 changes needed (9 `local_game.rs` tests
  + others).
- PROTOCOL 27 / HASH 63 unchanged — no sentinel re-pinned; no wire-shape file touched.
- No file under `crates/card-defs/` touched; `docs/authoring-status.md` untouched.

## Implement session (plan `pb-plan-DP1.md` steps 1-11) — scutemob-149

Scope for this invocation was explicitly capped at the plan's Steps 1-11 (probes,
Group A, Group B ruling, Group D, Group C citations, optional mana.rs comment, probes
GREEN). Steps 12-16 (full-suite fallout triage, golden scripts, simulator, gates,
close-out) are dispatched separately and are NOT covered by this session.

### Plan step 1 — probe file created, RED captured before any engine edit

File: `crates/engine/tests/primitives/pb_dp1_actor_priority.rs`, registered via
`mod pb_dp1_actor_priority;` in `crates/engine/tests/primitives/main.rs` (alphabetically
between `pb_ac9_wheel_and_misc` and `pb_ef10_sacrifice_driven_amounts`, per plan §3).
9 probes P1-P9. Verbatim pre-fix run:

```
running 9 tests
test pb_dp1_actor_priority::test_dp1_mana_ability_does_not_reset_players_passed ... ok
test pb_dp1_actor_priority::test_dp1_non_active_player_crewing_retains_priority ... FAILED
test pb_dp1_actor_priority::test_dp1_foretell_resets_players_passed ... FAILED
test pb_dp1_actor_priority::test_dp1_non_active_player_cycling_retains_priority ... FAILED
test pb_dp1_actor_priority::test_dp1_active_player_casting_still_holds_priority ... ok
test pb_dp1_actor_priority::test_dp1_special_action_actor_holds_priority_after_turn_face_up ... ok
test pb_dp1_actor_priority::test_dp1_actor_can_respond_to_own_activated_ability ... FAILED
test pb_dp1_actor_priority::test_dp1_non_active_player_casting_instant_retains_priority ... FAILED
test pb_dp1_actor_priority::test_dp1_actor_can_respond_to_own_spell ... FAILED

failures:
---- test_dp1_non_active_player_crewing_retains_priority ----
assertion `left == right` failed: CR 117.3c: the crewing player retains priority
  left: Some(PlayerId(1))
 right: Some(PlayerId(2))

---- test_dp1_foretell_resets_players_passed ----
CR 117.4: an action was taken between passes, so the pass-round must restart

---- test_dp1_non_active_player_cycling_retains_priority ----
assertion `left == right` failed: CR 117.3c: the cycling player retains priority
  left: Some(PlayerId(1))
 right: Some(PlayerId(2))

---- test_dp1_actor_can_respond_to_own_activated_ability ----
assertion `left == right` failed: CR 117.3c: p2 retains priority after activating their own ability
  left: Some(PlayerId(1))
 right: Some(PlayerId(2))

---- test_dp1_non_active_player_casting_instant_retains_priority ----
assertion `left == right` failed: CR 117.3c: the caster, not the active player, receives priority after casting
  left: Some(PlayerId(1))
 right: Some(PlayerId(2))

---- test_dp1_actor_can_respond_to_own_spell ----
CR 117.3c: p2 should retain priority and be able to cast a second spell without passing:
NotPriorityHolder { expected: Some(PlayerId(1)), actual: PlayerId(2) }

test result: FAILED. 3 passed; 6 failed; 0 ignored
```

Matches the plan's prediction exactly: P1-P5 and P8 FAILED pre-fix; P6, P7, P9 passed
pre-fix (green-both-sides controls / incidentally-correct, as the plan called out for
P9 — `handle_turn_face_up` never writes `priority_holder` at all, so a manually-seeded
`Some(p2)` simply survives untouched). No probe needed to be rewritten or escalated —
none of P1-P5/P8 passed pre-fix (which would have signalled a test-validity bug per
conventions.md).

### Plan step 2-3 — Group A: `casting.rs` (1 site), the headline flip

`handle_cast_spell` (`casting.rs`): `:4712-4715` comment corrected (was misquoting CR
601.2i as "the active player receives priority" — replaced with the actual text and the
CR 117.3c/117.4 citations) and `priority_holder = Some(player)` (was
`Some(state.turn.active_player)`); companion `PriorityGiven { player }` push at the
former `:4907-4909` fixed in the same edit (was `player: state.turn.active_player`).
Also fixed two aspirationally-wrong doc comments not explicitly listed in the plan (the
module doc at the top of the file and the `handle_cast_spell` function doc), both of
which said "the active player receives priority" — a stale claim once the fix lands.
Comment-only, zero logic; flagged here as a minor plan-scope addition per
conventions.md's "aspirationally-wrong comments are correctness hazards" rule.

### Plan step 3 — Group A: `abilities.rs` (12 sites)

All 12 sites confirmed at the plan's exact line numbers (no drift): generic
`handle_activate_ability` (flip), `handle_cycle_card` (flip), `handle_activate_forecast`
(no-op, AP-gated), `handle_activate_bloodrush` (flip), `handle_unearth_card` (no-op,
AP-gated), `handle_ninjutsu` (flip), `handle_embalm_card` (no-op, AP-gated),
`handle_eternalize_card` (no-op, AP-gated), `handle_encore_card` (no-op, AP-gated),
`handle_crew_vehicle` (flip), `handle_saddle_mount` (no-op, AP-gated),
`handle_scavenge_card` (no-op, AP-gated). Each site: assignment changed to
`Some(player)`, `let active = ...` binding deleted, the companion `PriorityGiven` push
(found 6-190 lines below the assignment per the plan's R1 warning) changed to
`player`, and — for the 8 AP-gated sites — an appended note naming the actual gate line
that makes today's write an identity write. All 12 bogus `CR 602.2e` / `CR 116.3b`
citations replaced with `CR 602.2b -> 601.2i / CR 117.3c` (+ `CR 117.4` for the reset).
Verification, scoped to `abilities.rs` (the 12 sites): `rg "PriorityGiven \{ player:
active \}" crates/engine/src/rules/abilities.rs` → 0 hits; `rg "let active =
state.turn.active_player" crates/engine/src/rules/abilities.rs` → 0 hits (the one
remaining `let active = state.turn.active_player` in the file is `apnap_order`,
`:8453`, unrelated — verified not one of the 12 handlers).

**Deviation from the plan's §14 checklist, noted and justified**: the checklist says
`rg "602\.2e"` / `rg "116\.3[abcd]"` under **all of** `crates/engine/src` should return
0. After Step 9 (Group C), this is **not** literally true: 2 hits remain in
`abilities.rs` (lines 8803/9014) and 1 in the fn doc at `:129` — but all three are our
*own* explanatory asides quoting the bogus numbers to say they don't exist, in the
exact phrasing the plan's own §4 table prescribes for line 8788 ("(Neither "CR 602.2e"
nor "CR 116.3b" exists.)"). No live citation anywhere uses the bogus numbers as if they
were real. Similarly, `rg "PriorityGiven \{ player: active \}"` across all of
`crates/engine/src` returns 4 hits after Step 9 — `priority.rs:88`
(`grant_initial_priority`, CR 117.3a step-start grant), `resolution.rs:115/7752/7991`
(fizzle / after-resolution / after-countering, CR 117.3b) — all four are Group C sites
that the plan explicitly requires to stay byte-identical; they legitimately keep the
`active`-named binding and event. The plan's own checklist bullets are scoped too
broadly (should have read "scoped to Group A" rather than "crates/engine/src"); this is
a plan-internal inconsistency, not a fix-phase gap. Verified each of the 4 sites by
reading it in context (see Step 9 section below).

### Plan step 4 — Group A: `engine.rs` craft (1 site, no-op)

`handle_activate_craft`: comment replaced per plan (cites CR 702.167a -> 602.2b -> 601.2i
/ 117.3c, notes the `:1272` `is_active` gate makes this an identity write, cites CR
117.4 for the reset); `let active` binding deleted; assignment changed to
`Some(player)`. No `PriorityGiven` push exists in this handler (confirmed) — none added,
per the plan's explicit instruction not to (adding an event would be a wire-observable
change).

After steps 1-4: `cargo check -p mtg-engine` clean; probe re-run shows only P8
(foretell, Group D-b, not yet touched) still red — 8/9 green. Commit
`W6-prim: scutemob-149 -- PB-DP1 steps 1-4` (probes + Group A casting/abilities/craft).

### Plan step 5 — Group B ruling (echo / cumulative upkeep / recover), comment-only

`handle_pay_echo`/`handle_pay_cumulative_upkeep`/`handle_pay_recover` (`engine.rs`):
applied the plan's ruling verbatim — leave the AP-priority-reassignment behavior
exactly as-is (these are resolution-time choices; CR 117.3c's antecedent is false, no
player holds priority mid-resolution) and fix only the misleading "Grant priority to
the active player." comment at each of the 3 sites, citing CR 702.30a/702.24a/702.59a
respectively and pointing at OOS-DP1-1 (real fix is the DP-11 pause, PB-DP4's scope).
Verified logic byte-identical via `git diff` (comment lines only, confirmed by reading
the full diff — no `+`/`-` on any non-comment line). Staged and committed *separately*
from the Group D-c hunks in the same file using `git add -p` (verified the split with
`git stash push --keep-index` + a clean `cargo check` on the stash-held index before
popping back).

### Plan steps 6-8 — Group D dispositions

- **D-a** (`lands.rs`, comment-only): `handle_play_land` was already CR 116.3-correct
  by construction (`:31` priority guard, `:417` `players_passed` reset). Fixed 2 stale
  comments (module fn-doc + the `:417` inline comment) that described the *active*
  player retaining priority and cited CR 117.3b (wrong rule — that governs priority
  after a resolution, not a special action). Zero logic change, confirmed via diff.
- **D-b** (`foretell.rs`, `plot.rs`, `suspend.rs`, `commander.rs::handle_bring_companion`):
  added `state.turn.players_passed = imbl::OrdSet::new();` at the tail of each handler,
  immediately before `Ok(events)`, per the plan's exact template. `foretell.rs`/`plot.rs`/
  `suspend.rs` all have an entry priority guard (`:48`, `:55`, `:59` respectively,
  verified live at those lines) so the comment reads "no write is needed here."
  `handle_bring_companion` has **no** priority guard of its own — the CR 702.139a
  sorcery-speed gate (`:941`, `state.turn.active_player != player`) forces
  `player == active_player`, so the reset is still correct, and the missing guard is
  tracked as OOS-DP1-2, not fixed here (per plan §5 "Why not add the missing entry
  priority guards?" — new enforcement surface is DP-21's scope, out of this PB). Used
  `imbl::OrdSet::new()` fully-qualified in all four files (no new `use` added), per the
  plan's import-risk note.
- **D-c** (`engine.rs::handle_turn_face_up` / `handle_activate_loyalty_ability` /
  `handle_level_up_class`, SEPARATELY REVERTABLE): added the explicit
  `state.turn.priority_holder = Some(player);` write (previously absent) alongside the
  existing `players_passed` reset, with a corrected CR citation at all three
  (`handle_turn_face_up`'s old comment — "CR 116.2b: Special action; reset priority to
  active player." — was aspirationally wrong: no such write existed). None of the three
  has an entry priority guard, so this is not a strict identity write the way the
  Group-A AP-gated sites are — hence kept in **its own commit**
  (`7851da58`) per the plan's D-c escape hatch, so it can be reverted independently if
  the (out-of-scope, dispatched-separately) full-suite triage in plan step 12 traces a
  failure to one of these three specifically.

Verification after steps 5-8: `cargo check -p mtg-engine` clean at each incremental
step; full probe re-run after Group D-b showed all 9 green (P8 flipped from red to
green — verbatim: `test pb_dp1_actor_priority::test_dp1_foretell_resets_players_passed
... ok`).

### Plan step 9 — Group C citation fixes (comment-only, zero logic)

Fixed all 8 sites the plan's §6 table lists: `resolution.rs` module doc (`:8`),
`resolve_top_of_stack` fn doc (`:35`), cipher free-cast comment (`:5172`, now with the
CR 601.2i "controller did not have priority before casting" explanation added), suspend
free-cast comment (`:5836`, same shape, CR 702.62a), the after-resolution-and-triggers
comment (`:7748`), `priority.rs`'s `pass_priority` fn doc (`:14`) and
`next_priority_player` fn doc (`:50`), and `combat.rs`'s `handle_declare_blockers`
(`:1370`, CR 509.1 turn-based-action framing prepended, existing rationale kept intact)
and `handle_declare_attackers` (`:678`, CR 508.1/117.3a framing + `:46` guard
citation). Verified byte-identical logic via full `git diff` read (reproduced in this
session's transcript) — every hunk only adds/replaces comment lines. Re-ran the full
probe suite after: still 9/9 green (Group C changes cannot affect probe outcomes since
none of the 9 probes exercise a Group C code path).

### Plan step 10 — `mana.rs` comment-only fix (OPTIONAL — taken, not skipped)

Applied both comment corrections from the plan (module-level `handle_tap_for_mana` doc
at `:35-36`, and the `:622-623` inline "11." comment), replacing the mis-citation "CR
605.5" (mana abilities are activated abilities per CR 605.1a, not CR 116.2 special
actions) with CR 605.3b/117.3c/117.3b-parenthetical framing and an explicit pointer at
`test_dp1_mana_ability_does_not_reset_players_passed` (P7) as the pin. One correction
to the plan's suggested text: the plan said "the `:47` guard proves..." but the actual
priority guard in the current source is at `:52` (`if state.turn.priority_holder !=
Some(player)`) — used the verified line number instead of the plan's stale one.
**Zero code lines changed** — confirmed via `git diff` (every changed line begins with
`///` or `//`). Did not skip this step; the PRESERVE risk was assessed as near-zero
(pure doc-comment rewrite, no logic touched) and confirmed by re-running P7
(`test_dp1_mana_ability_does_not_reset_players_passed`) green immediately after.

### Plan step 11 — probe tests GREEN

Full probe suite, final state, run after Steps 5-10 (transcript below is the actual
`cargo test` output at the end of this session):

```
running 9 tests
test pb_dp1_actor_priority::test_dp1_mana_ability_does_not_reset_players_passed ... ok
test pb_dp1_actor_priority::test_dp1_foretell_resets_players_passed ... ok
test pb_dp1_actor_priority::test_dp1_non_active_player_casting_instant_retains_priority ... ok
test pb_dp1_actor_priority::test_dp1_non_active_player_crewing_retains_priority ... ok
test pb_dp1_actor_priority::test_dp1_active_player_casting_still_holds_priority ... ok
test pb_dp1_actor_priority::test_dp1_non_active_player_cycling_retains_priority ... ok
test pb_dp1_actor_priority::test_dp1_actor_can_respond_to_own_activated_ability ... ok
test pb_dp1_actor_priority::test_dp1_special_action_actor_holds_priority_after_turn_face_up ... ok
test pb_dp1_actor_priority::test_dp1_actor_can_respond_to_own_spell ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 735 filtered out
```

Also ran (both outside the plan's strict step-1-11 scope, but cheap and directly
relevant confirmations that nothing in scope broke the standing gates):
- `cargo build --workspace` — clean. Confirms plan §8's prediction that no exhaustive
  match (`state/hash.rs`, `view_model.rs`, `stack_view.rs`, simulator) needed a new arm,
  since no enum/struct shape changed.
- `crates/engine/tests/core/protocol_schema.rs` (17 tests) and
  `crates/engine/tests/core/hash_schema.rs` (21 tests) — both fully green. PROTOCOL 27 /
  HASH 63 confirmed unmoved; no sentinel needed re-pinning.

**NOT run** (explicitly out of scope for steps 1-11, per the task's stop directive):
`cargo test --all` (full suite — would surface the plan's forecast fallout in
`split_second.rs`, `casting.rs`, `cycling.rs`, `crew.rs`, `bloodrush.rs`, `ninjutsu.rs`,
`plot.rs`/`foretell.rs`/`suspend.rs`, and the golden-script corpus, all of which the
plan assigns to step 12/13, dispatched separately), `cargo clippy -D warnings` /
`cargo fmt --check` / `tools/check-defs-fmt.sh` (plan step 15, gated behind the
full-suite triage), `cargo test -p mtg-simulator` (plan step 14), and the
`primitive-impl-reviewer` + close-out steps (plan steps 10-11 of the WIP's own
higher-level checklist / plan steps 15-16).

**Commits this session** (4, each independently reviewable):
1. `794ea037` — Steps 1-4 (probes RED→partial-GREEN, Group A casting.rs/abilities.rs/
   craft).
2. `b0b2212c` — Steps 5-7 (Group B ruling, Group D-a, Group D-b).
3. `7851da58` — Step 8 (Group D-c, separately revertable per the plan's escape hatch).
4. `1bb13118` — Steps 9-10 (Group C citations, mana.rs comment).

## Fix cycle — scutemob-149 (review findings applied, all 11 dispositioned)

Review: `memory/primitives/pb-review-DP1.md` (0 HIGH / 3 MEDIUM / 8 LOW, verdict
needs-fix). Every finding applied this cycle; none declined.

### MEDIUM 1 — Group D-c write-without-guard → **APPLIED (add guard, ruled)**

Added the entry priority guard (identical shape to `foretell.rs:48-53` —
`if state.turn.priority_holder != Some(player) { return Err(NotPriorityHolder { .. }) }`)
to all three `engine.rs` handlers: `handle_turn_face_up`, `handle_activate_loyalty_ability`,
`handle_level_up_class`. Each guard is its own hunk; the tail `priority_holder = Some(player)`
write is now a true identity write (same shape as the Group-A AP-gated sites), and the
comments were updated to say so. Also fixed LOW 6 in the same edit (moved
`handle_turn_face_up`'s write to after the SBA check, matching craft's ordering).
Scope respected: no CR 606.3 "their own turn" sorcery-timing check was added (that stays
DP-21's scope) — verified by re-reading `handle_activate_loyalty_ability` /
`handle_level_up_class` after the edit: neither checks `state.turn.active_player`.

**Fallout**: none. Full suite stayed at 3,721/3,721 (3,713 baseline + 8 new fix-cycle
probes) after adding all three guards — no pre-existing test or script drove a
non-priority-holding actor through `TurnFaceUp`, `ActivateLoyaltyAbility`, or
`LevelUpClass`. No revert was needed; the escape hatch in the plan's §5 was not invoked.

### MEDIUM 2 — D-c zero test coverage → **APPLIED (8 new probes)**

Added P10-P17 to `crates/engine/tests/primitives/pb_dp1_actor_priority.rs` (17 probes
total, up from 9):
- P10 `test_dp1_turn_face_up_rejects_non_priority_holder` — guard probe.
- P11 `test_dp1_loyalty_activation_grants_actor_priority` — positive; p2 (non-active)
  controls the planeswalker and holds priority on p1's turn (loyalty has no "their own
  turn" gate today — OOS-DP1-2), so this is a genuine flip, not an identity write.
- P12 `test_dp1_loyalty_activation_rejects_non_priority_holder` — guard probe.
- P13 `test_dp1_level_up_class_grants_actor_priority` — positive, same shape as P11 for
  Class level-up.
- P14 `test_dp1_level_up_class_rejects_non_priority_holder` — guard probe.
- P15/P16/P17 — LOW 7's D-b coverage gap (plot / suspend / bring_companion
  `players_passed` reset, foretell was the only one tested before).

**Verified by construction, not by assertion** (every guard/reset probe, temporarily
reverting the corresponding engine line, confirming RED, restoring, confirming GREEN):
- Reverted the `handle_turn_face_up` guard → P10 FAILED:
  `TurnFaceUp should fail when the actor does not hold priority` (16 passed; 1 failed).
  Restored → 17/17 green.
- Reverted the `handle_activate_loyalty_ability` guard → P12 FAILED:
  `ActivateLoyaltyAbility should fail when the actor does not hold priority`
  (16 passed; 1 failed). Restored → 17/17 green.
- Reverted the `handle_level_up_class` guard → P14 FAILED:
  `LevelUpClass should fail when the actor does not hold priority` (16 passed; 1
  failed). Restored → 17/17 green.
- Reverted `plot.rs`'s `players_passed = imbl::OrdSet::new()` → P15 FAILED:
  `CR 117.4: an action was taken between passes, so the pass-round must restart`
  (0 passed; 1 failed, filtered run). Restored → green.
- Reverted `suspend.rs`'s reset → P16 FAILED with the identical message. Restored → green.
- Reverted `commander.rs::handle_bring_companion`'s reset → P17 FAILED with the identical
  message. Restored → green.

P9 (the original vacuous probe) was kept and its doc comment rewritten to explain why it
is no longer vacuous *in effect* (the guard now proves the precondition the tail write
merely echoes) while pointing at P10 as the actually-discriminating probe for the guard
itself — per the review's own framing ("Whichever way Finding 1 is dispositioned, this
probe must exist and must fail against the other disposition").

### MEDIUM 3 — mana-ability CR 117.4 citation → **APPLIED (citation fixed, seed filed)**

PRESERVE kept — zero behavioural change to `mana.rs` or the mana-ability path. Fixed the
citation in three places to stop attributing the `players_passed` non-reset to CR 117.3b
(which says nothing about `players_passed`):
- `crates/engine/src/rules/mana.rs` — both the module-level doc comment (`:35-46`) and
  the inline `:630-639` comment now name CR 117.4 explicitly as the rule being deviated
  from, and point at OOS-DP1-4.
- `test-data/generated-scripts/stack/066_krosan_grip_split_second_blocks_counterspell.json:187`
  — note rewritten to stop citing "CR 117.3b parenthetical" as authority for the
  `players_passed` non-reset; now describes it as a known engine deviation with a
  pointer to OOS-DP1-4.
- `crates/engine/tests/primitives/pb_dp1_actor_priority.rs` (P7) — doc comment and one
  assertion message reworded to separate the two claims (CR 117.3b governs the priority
  holder; the `players_passed` non-reset is the separate, deliberate CR 117.4 deviation).
- Bonus (same root cause, found while fixing LOW 9): `pb_ef8_exile_self_from_hand.rs`
  had the same miscitation baked in four places (module doc + T3's doc comment + two
  assertion messages) — same fix applied there.

**Seed filed**: **OOS-DP1-4** — "a mana ability's `players_passed` non-reset is a known,
deliberate deviation from CR 117.4 ('without taking any actions in between passing'),
not something CR 117.3b's parenthetical authorizes (that rule governs only who receives
priority, not `players_passed`). PRESERVE'd verbatim by PB-DP1 and pinned by
`test_dp1_mana_ability_does_not_reset_players_passed`. Closing it (making a mana
activation restart the pass-round) is a genuine behaviour change with test/script
fallout across the corpus and is not this PB's scope." Cross-referenced from
`docs/audits/decision-point-audit.md` via the PB-DP1 §8 row update below.

### LOW 4 — `commander.rs:1022-1026` non-sequitur → **APPLIED**

Reworded: "being the active player does NOT imply the player held priority (the active
player can pass and still be the active player)"; the unconditional `players_passed`
reset is correct only when the actor did hold priority, and the missing guard remains
OOS-DP1-2.

### LOW 5 — stale line refs → **APPLIED**

- `resolution.rs:5175` (cipher free-cast comment): `resolution.rs:7744` → `:7751` (the
  actual `priority_holder = Some(active)` write, verified live at that line).
- `lands.rs:24` and `:419-420`: `` `:31` guard `` → `` `:32` guard `` (the guard's `if`
  is at `:32`; `:31` is the comment line above it).

### LOW 6 — write-before-SBAs vs craft's write-after → **APPLIED (ruled: the ordering
matters, fixed to match craft)**

Moved `handle_turn_face_up`'s `priority_holder = Some(player)` write to after
`check_and_apply_sbas`, matching `handle_activate_craft`'s order (SBA check, then
`players_passed` reset, then priority write). Ruling: the pre-SBA position is reachable
in principle (INV-PI-02 would catch a priority_holder left on a since-departed player)
even though no current code path in `sba.rs` reassigns `priority_holder` — free to fix,
so fixed rather than argued as safe-by-luck.

### LOW 7 — D-b coverage 1-of-4 → **APPLIED**

Added P15 (`test_dp1_plot_resets_players_passed`), P16
(`test_dp1_suspend_resets_players_passed`), P17
(`test_dp1_bring_companion_resets_players_passed`) — see MEDIUM 2 above for the
verify-by-construction evidence (all three RED when their handler's reset is reverted).

### LOW 8 — residual "Active player retains priority" prose → **APPLIED**

Reworded all four sites to CR 117.3c/116.3 framing, noting the actor and the active
player coincide in these specific tests (so the assertion value is unchanged, only the
prose):
- `crates/engine/tests/casting/casting.rs:87`
- `crates/engine/tests/rules/abilities.rs:189`
- `crates/engine/tests/casting/mana_and_lands.rs:96` (doc comment) and `:118` (inline)

### LOW 9 — `pb_ef8_exile_self_from_hand.rs` mana ability called "a special action (CR
605.5)" → **APPLIED**

Fixed all 4 occurrences (`:5` module doc, `:184` test doc comment, `:209` and `:219`
assertion messages) to the same correction PB-DP1 already made in `mana.rs`: a mana
ability is an activated ability (CR 605.1a), not a CR 116.2 special action; CR 605.5
only defines what does NOT qualify as a mana ability (verified verbatim against the CR
text: "Abilities that don't meet the criteria specified in rules 605.1a-b and spells
aren't mana abilities" — says nothing about special actions). Also folded in MEDIUM 3's
citation fix here since the same file conflated the priority-holder claim (CR 117.3b)
with the `players_passed` claim (CR 117.4 deviation, OOS-DP1-4).

### LOW 10 — `pb_ef2_create_token_recipient.rs:311-315` comment/code mismatch →
**APPLIED**

The comment said "same shape as the happy-path test above" implying `[p2, p1]`, but the
call is `pass_all(&[p2, p1, p1, p2])` (and so is the happy-path test's own call, at
`:288` — its comment already correctly explains the 4-pass shape). Rewrote the decoy
test's comment to spell out the `[actor, other, other, actor]` reasoning explicitly
(single resolution counters AND removes the target, emptying the stack, so CR 117.3b
hands priority to the active player for the remaining pass round) instead of pointing at
the happy-path comment without restating it.

### LOW 11 — plan step 16 not done → **APPLIED**

Ran plan step 16:
- `docs/audits/decision-point-audit.md` §5 Tier-0 **DP-1** row: marked
  `SHIPPED (PB-DP1, scutemob-149)`, corrected the site list to the verified breakdown
  (14 Group A / 3 Group B / 8 Group D / 5 confirmed false positives — the five being
  `engine.rs:1759`/`:1805`, `combat.rs:1373` which are CR 117.3a, and the two
  `handle_activate_loyalty_ability`/`handle_level_up_class` sites the original roster
  missed entirely), and noted the fix-cycle guard addition.
- §8 **PB-DP1** row: marked `SHIPPED (scutemob-149)`.
- Seeds filed. **Durable home: `docs/audits/decision-point-audit.md` §8.1** — this file is
  rewritten wholesale by the next `/implement-primitive` run, so the copies below are a
  working record, not the inventory. §8.1 is to the PB-DP suite what
  `rider-seed-triage-2026-07-19.md` §1c is to the RS queue. Summaries:
  - **OOS-DP1-1** — echo / cumulative-upkeep / recover reassign priority to the AP out
    of band (Group B); correct fix is the DP-11 pause, owned by PB-DP4. No engine change
    made — comment-only, per the plan's ruling.
  - **OOS-DP1-2** — `handle_activate_craft` and `handle_bring_companion` still have no
    entry priority guard (craft is AP-gated by construction so this is lower-severity
    than the three D-c handlers, which now DO have a guard after this fix cycle);
    `handle_activate_loyalty_ability` and `handle_level_up_class` also lack the CR 606.3
    /  716.2a "their own turn" sorcery-timing check (a SEPARATE gap from priority,
    explicitly out of this PB's scope per the task's ruling — DP-21's scope). Partially
    closed by this fix cycle (the priority guard on 3-of-5 handlers); the "their own
    turn" gaps and craft/companion's missing priority guards remain open.
  - **OOS-DP1-3** — stale pre-renumber CR citations (`116.3a/b/c/d` for what is now
    `117.3a-d` / `117.4`) survive in ~60 golden-script `"note"` fields, one
    `cr_sections_tested` array, `docs/mtg-engine-milestone-reviews.md:326-327`, and
    seven `memory/abilities/*.md` records. Cosmetic; batch into a doc pass, not a PB.
  - **OOS-DP1-4** — see MEDIUM 3 above (mana-ability `players_passed` non-reset is a
    known CR 117.4 deviation, not CR 117.3b's).

### Final gate re-run (post fix-cycle)

- `cargo test --all --no-fail-fast` → **3,721 passed / 0 failed** (baseline 3,713 + 8
  net-new probes P10-P17; P9 kept, doc comment only changed).
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo build --workspace` → clean.
- `cargo fmt --check` → clean (after running `cargo fmt` once to normalize the new
  probe file's line-wrapping — verified `git diff --stat` afterward showed only
  whitespace/wrapping changes, no logic).
- `tools/check-defs-fmt.sh` → clean, 1,804 defs checked.
- `crates/engine/tests/core/protocol_schema.rs` (17 tests) and
  `crates/engine/tests/core/hash_schema.rs` (21 tests) → green;
  `PROTOCOL_VERSION == 27` (`rules/protocol.rs:260`), `HASH_SCHEMA_VERSION == 63`
  (`state/hash.rs:578`) confirmed unmoved.
- `git diff --stat` — no file under `crates/card-defs/`, no change to
  `docs/authoring-status.md`. Coverage unchanged at 1,139/1,804 = 63.1%.

### Prior state

PB-RS4 SHIPPED (`scutemob-146`, merge `9419d0e9`). The RS queue is paused at RS5; the user
directed (2026-07-26) that the whole PB-DP suite runs before RS5 / M11-S2. PB-DP1 is rank 1
of that suite. Audit: `docs/audits/decision-point-audit.md` (`scutemob-148`).
