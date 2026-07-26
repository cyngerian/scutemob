# Primitive WIP — PB-DP1 (DP-1 · priority after cast/activate/special action) · IMPLEMENT (steps 1-11 of 16 done)

<!-- last_updated: 2026-07-26 -->

- **PB**: PB-DP1 — priority after casting a spell, activating an ability, or taking a
  special action goes to the **actor**, not the active player (CR 117.3c / 116.3)
- **Task**: `scutemob-149`
- **Branch**: `feat/pb-dp1-priority-after-castactivatespecial-action-goes-to-the`
- **Class**: CORRECTNESS (live-wrong, core-reachable — no card required)
- **Phase**: IMPLEMENT — plan steps 1-11 of 16 complete (probes + Group A/B/C/D +
  mana.rs comment, all 9 probes green); **steps 12-16 remain** (full-suite fallout
  triage, golden scripts, simulator, gates, review, close-out) and are dispatched
  separately, not part of this session
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
- [ ] 7. Test + golden-script fallout triage, every update citing CR 117.3c — **OUT OF
      SCOPE for this invocation** (plan step 12/13; dispatched separately)
- [ ] 8. Simulator / LocalGame baselines updated knowingly — **OUT OF SCOPE** (plan step 14)
- [~] 9. PROTOCOL 27 / HASH 63 confirmed unchanged (wire sentinels green — see below);
      **full gate suite (clippy/fmt/`cargo test --all`) NOT run** — that requires the
      fallout triage (step 7/plan-step-12) first, which is out of scope here
- [ ] 10. `primitive-impl-reviewer` pass, findings dispositioned — **OUT OF SCOPE**
- [ ] 11. Close-out: audit doc §5/§8 rows shipped; seeds filed — **OUT OF SCOPE**

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

### Prior state

PB-RS4 SHIPPED (`scutemob-146`, merge `9419d0e9`). The RS queue is paused at RS5; the user
directed (2026-07-26) that the whole PB-DP suite runs before RS5 / M11-S2. PB-DP1 is rank 1
of that suite. Audit: `docs/audits/decision-point-audit.md` (`scutemob-148`).
